# Agent Note: 网络语扩充：social-media-chinese-words（T-087）

Status: implemented

English | [中文](2026-10-04-social-media-slang-expansion.zh.md)

## Problem

FR-047（M13）扩充网络语包。M6 交付的 slang 包完全由人工种子表构建（413 行，
T-045），对圈子黑话覆盖不足；[暂缓记录](../../implemented/process/2026-09-28-slang-pack-and-content-gate.zh.md)
当时便把 `social-media-chinese-words`（MIT，约 107.7 万行）列为未来候选词池。
D-52（用户 2026-10-02 确认）承诺把该词池清洗出高频子集约 1 万条并入 slang
包，并持续扩充人工种子表。约束：完全离线且确定性、输入 SHA 锁定（A-1/
NFR-008）、可复现、缩写路径（FR-017）与用户词学习不回退、包体量/加载预算与
既有 slang 包同量级。

## Decision

"锁定下载 + 确定性清洗 + 合并构建"管线，且**清洗子集提交进仓库**，让每次
build-slang 的输入都留在仓库内（与 T-045 种子表同一规则）：

- **获取**（`scripts/fetch-social-media.ps1`，PS 5.1）：下载 7 个分类 txt
  （每行 `词 词频`），以 UTF-8 无 BOM 合并（每分类前插 `# CATEGORY:` 注释），
  SHA-256/大小与 pin `data/pins/social-media-zh.json`（D-020）比对，漂移即
  失败并保留旧缓存（LICENSE/README 另存 `data/cache/social-license.txt` /
  `social-readme.txt` 审计）。通用 `fetch-sources.ps1` 只面向单 URL，不负责
  合并步骤。
- **清洗**（`zhu-ye-dict social-clean`，新模块 `social.rs`；先跑
  `source-check`）：解析 `词 词频` → 过滤（词形=汉字+ASCII 字母/数字且 ≥1
  汉字；长度 2-12 字符）→ 把关负面清单（与 build-slang 同一 `Gate`）→ 与
  种子表词条完全重复剔除（权重已并入种子既有项，不新增）→ 按 ASCII 小写
  归一去重保首现（同形同义）→ 按原语料词频降序稳定排序 → 截取 10,000。
  因源含频次列，§14.4"频次或回退分支"落到语料词频。产物
  `data/slang/social-words.tsv`（D-021，4 列同 `seed.tsv`，来源列
  = `social-media-chinese-words`），头注释记录规则版本 v1 与输入 SHA；两次
  运行字节一致（已验证）。
- **构建**（`slang.rs` 的 `build-slang`）：`parse_seed` 两文件，种子行为先、
  social 行随后，单把关+注音+去重+v2 编译（词频统一 5000 与既往一致）。
  social 行全为 `词`（绝不缩写），FR-017 缩写键规则零改动。social 文件
  缺失时降级 `social_rows = 0`（报告新增字段）而不失败，旧检出仍可构建
  经典包。
- **种子表扩充**：追加 19 行（2026-10 分区）逐行梗源注释
  （`维护者整理 2026-10（…）`），如 瑞思拜/泼天的富贵/雪糕刺客/已老实/
  修勾/吗喽/双向奔赴，均经与既有 413 行查重、字经 kTGHZ2013 底表注音路径
  校验。

**实测（2026-10-04，release）**：清洗输入 1,075,792 行 → 形状 88,075 /
把关 513 / 种子重复 922 / 同形去重 270,253 → 输出 10,000；多次运行字节一致
（已验 3 次，SHA `65697A83…`）；构建 →
slang.zyct 9,900 词条 / 661,803 字节（26KB→646KB），把关负例 310/310、
误杀 1/453（0.2%）不变；eval 复跑与 T-057 基准一致（Top1 84.7% /
Top3 97.2% / 整句 21.0%）；`source-check` 15/15 源锁定（16 个 pin json 含
1 快照数据文件、不计源）。

## Alternatives considered

- **独立包（如 `slang-social.zyct`）**：否决——D-46 保持"三领域包"预置口径
  不变；网络语与 slang 语义重叠，独立包徒增装配复杂度。
- **build-slang 直接读原始缓存语料、不提交中间子集**：否决——构建输入将落在
  仓库外，违反"输入仓库内"规则，也无法 diff/审计实际发版内容；中间 tsv
  很小（约 200KB）且本身就是审计产物。
- **CJK+符号词形（·、&…）、纯英文、长度 1 或 >12**：否决——§14.4 词形
  规则为"纯 CJK 或 CJK+字母数字"；放宽只会放大已记录的语料噪声（品牌名、
  人名、非汉字原始行）。
- **把关抽查样例扩充**：未做——误杀率已 0.2%，远低于 5% 门槛；扩样例等于
  改变把关尺度，需要单独决策，且 §14.1.3 只要求"不回退"。
- **保留繁体/专名词形**：接受——CJK 范围天然含繁体；构建期注音失败自然
  排除（532 条 social 行），与 T-045 同策略。

## Consequences

- `build-slang` 新增第二输入文件与 `social_rows` 报告字段；`social-clean`
  新增 CLI 子命令；`source-check` 计数 15→16。
- 词条数 5,900% 增长比例远超 base/领域包，但 646KB 仅约 base.zyct 的 2.2%，
  加载为 mmap+布局校验，量级预算成立；slang 包更新体积相应增大
  （manifest/哈希随每次发行重算，无契约变化）。
- 质量：子集为算法抽取（品牌/人名/繁体按规则通过词形检查）；注音底表读
  不出的行在构建期丢弃；把关清单在清洗与构建两处各跑一遍（双保险）。
- 后续扩充=三步命令管线（`fetch-social-media.ps1` → `social-clean` →
  `build-slang`），哈希全锁。

相关记录：[网络语包与内容把关（T-045——词池在此暂缓、本任务解除；把关机制
本体未变）](../../implemented/process/2026-09-28-slang-pack-and-content-gate.zh.md)、
[复合词典与缩写路径（FR-017 未触动）](../../implemented/architecture/2026-09-29-composite-dictionary-and-abbreviation-path.zh.md)、
[en.zyen 双源（同类数据 pin 模式）](../../implemented/feature/2026-10-04-en-wordbook-zyen-v1.md)。
