# Agent Note: 无法切分的长串自动隔音识别音节边界（T-163）

Status: implemented

[English](2026-10-15-auto-apostrophe.md) | 中文

## Problem

简拼单字母与全拼音节混合的输入串（如 `wsshui`＝我+是+谁，`w`、`s` 为简拼
首字母、`shui` 为全拼音节）既无法被 `segment_all` 切分（它只接受完整音节），
又超出简拼路径的 `INITIAL_MAX_LEN=4` 上限，简拼/错序/缩写/前缀全部兜底路径
都漏掉，引擎零候选。用户在真机对比中点名：搜狗自动打隔音符（`w's'shui`）
并出「我是谁」，竹叶打不出来，要求对齐搜狗。

## Decision

新增**自动隔音**候选组（T-163，FR-023 简拼路径扩展），实现于
`zhu-ye-core/candidate.rs` 并在 `zhu-ye-ime/input.rs` 接入：

1. **槽划分**（`slot_plans`）：把不可切分串切成若干「槽」，每槽为完整音节或
   简拼单字母（该字母在 `pinyin::INITIAL_SYLLABLE_TABLE` 有常用音节）；段数
   2..=6、方案数 ≤`APOSTROPHE_PLAN_CAP`（32）。枚举顺序确定：**完整音节槽
   （长段从长到短）优先、单字母槽随后**——`wsshui` 先得 `[w,s,shui]` 再得
   `[w,s,s,hui]`。该顺序是 T-163 验收修复的关键：原先"单字母优先"会让噪声
   划分 `[w,s,s,hui]`（我试试会…）在真实 44 万词典上先耗尽候选配额，
   「我是谁」永远轮不到。
2. **展开**（`expand_slot_combinations`）：单字母槽展开为该字母常用音节表、
   完整音节槽原样，前序笛卡尔积拼接成完整拼音串（≤`APOSTROPHE_COMBINATION_CAP`＝48，
   超限截断保留表序最常用拼法）。`[w,s,shui]` → `woshishui`（wo+shi+shui）。
3. **产出**（`apostrophe_candidates`）：组合串 `dictionary.lookup` 整词命中
   → 直接出候选（`pinyin`＝展开串，供用户词学习与拼音行展示）；未命中且该
   划分 ≥3 音节 → 先**整词组合**（`generate_candidates`：每音节词典首条，
   `woshishui`→「我是谁」，正分置顶）再由**整句 beam**（`sentence_candidates`，
   同 FR-025 语义）补足非首条组合（同文本去重）。总候选
   ≤`APOSTROPHE_CANDIDATE_CAP`（10）。
4. **显示**（`apostrophe_boundaries` + input.rs `apostrophe_seps`）：页眉
   组合串显示隔音形式 `w's'shui`（U+2019，与 T-128 同一显示口径）。
   `refresh_candidates` **仅在自动隔音组真正产出候选时**记录首选槽边界，
   其余每轮清空——不污染 T-128「最长可切分前缀」退化显示
   （`xianjingd` 仍显示 `xian'jingd`）。
5. **接入门槛**（input.rs）：与简拼同位兜底——仅当主候选为空、纯小写 ASCII、
   长度 `3..=APOSTROPHE_MAX_INPUT`（16）且整串不可切分时介入；开关跟随
   `enable_abbreviation`（简拼族整体开关）。可切分串与 2-4 位简拼保持既有
   行为不变。

防御规则：可完整切分或整词已命中 → 返回空；长度/字符形态不合规 → 返回空。
全部枚举确定；成本受控（32 方案 × 48 组合、每组合数次 lookup、至多数次
beam），远低于 30ms 刷新门槛。词典二进制格式零改动（O-06）。

## Alternatives considered

- **放开简拼长度上限**（如把 `INITIAL_MAX_LEN` 调大）：笛卡尔积爆炸
  （6⁶≈4.6 万次 lookup）且稀释 2-4 位简拼语义；否决，改为独立槽模型混合
  简拼与全拼。
- **逐前缀自动补全**：即既有前缀路径，无法桥接 `w`+`s` 这类**不可切分的串首**。
- **只做整词命中、不做整句组装**：发行词典无 `woshishui` 整词条，
  「我是谁」出不来；必须复用既有 FR-025 beam 组装。
- **对所有不可切分串无条件显示隔音符**：会破坏 T-128「最长可切分前缀」
  显示（`xianjingd`）；否决——隔音符只在自动隔音候选命中时显示。

## Consequences

- `wsshui` 现在**首选**出「我是谁」，对齐搜狗/微软拼音对「简拼+全拼」混合串
  的行为，词典零改动（真实 44 万词典验证：首条 我是谁，score 19940）。
- 简拼族开关 `enable_abbreviation` 同时管控新组；正常全拼/整词/前缀/2-4 位
  简拼行为不变（由「主候选为空」门槛与可切分/整词命中早退双重保证）。
- 候选 `pinyin` 为展开后的完整拼写（用户词学习不受影响）；候选行拼音行
  展示展开拼写；自动隔音命中时**页眉组合串显示隔音形式**（U+2019，
  T-128 口径）。
- 页眉隔音符以候选命中为门控：从未进入自动隔音路径的串（如 `xianjingd`）
  显示与 T-128 完全一致、零漂移。

## Verification

- core 单测 5：`wsshui`→首选「我是谁」且首选划分 `[w,s,shui]` 先于噪声划分
  展开（`woshishui` 拼音存在——顺序回归锁定，用含大量 shi/hui 同音词的
  压力词典）；整词命中优先（`我是谁`/`woshishui` 词条胜出）；不污染可切分串
  与整词命中串；长度/字符门槛（2 位简拼仍归 `initial_candidates`、含数字/
  大写/非 ASCII 拒绝、不产无拼音候选）；`apostrophe_boundaries` 显示边界
  （`wsshui`→`[1,2]`、可切分串空）。
- ime 引擎集成 3：输入 `wsshui` 出「我是谁」；`nihao` 与 `wsm` 行为不变
  （不被抢占）；`composing_display()` 显示 `w's'shui`（U+2019）而
  `composing()` 仍为 `wsshui`。
- 真实词典验证（临时探针，跑完即删）：`apostrophe_candidates("wsshui")`
  首条=「我是谁」（woshishui，19940）。
- T-128 显示回归 4 项（自动分隔符按首选切分显示/尾部残段退化/退格后实时
  重算/无手动分隔约束组不介入）全绿——隔音符以候选命中为门控。
- 全部门禁绿：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`（core 352、ime lib 248、tsf 45、host_e2e 169、
  candidate_window 13）、`git diff --check`；verify-translation-pairs
  （73 配对）与 verify-agent-notes 全绿。
- 文档同步：需求规格说明书 FR-023 自动隔音条目（首选+显示）+ §12.4 决策
  O-07、方案设计 §12.2.5、验收标准 §8.1 FR-023 自动隔音三行、CHANGELOG
  [Unreleased]、todos T-163（另有 T-164 输入法切换排障与 T-165 Ctrl+Space
  中英切换）。
- 已部署首轮（2026-10-16）：`zhu_ye_ime_74C68A5A.dll`（2,255,360 B）落地并
  切换指针、ctfmon 重启（PID 40784），verify-tsf-dll/identity 通过；用户验收
  反馈「我是谁 未打出、无 `w's'shui` 隔音符」后完成上述顺序/置顶/显示修复，
  **二轮部署**随后跟进。真机复验（新宿主进程窗口）待用户确认。
