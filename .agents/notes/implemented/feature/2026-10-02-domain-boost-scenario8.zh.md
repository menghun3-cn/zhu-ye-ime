# Agent Note: 领域提权候选层（场景 8，M10）

Status: implemented

[English](2026-10-02-domain-boost-scenario8.md) | 中文

## Problem

场景 8（领域自动，FR-033/FR-034/FR-035）需要让**已启用领域包**（P-12：只有用户勾选的
包参与）在组合串整词命中时提权其词条。已确认口径：仅完整词提权（D-15）、仅领域候选
生效（用户词/联想/其他层零变化，D-17）、多包按包 id 字典序取首个（D-16）、插入点在
基础候选之后、追加组之前（D-13）、开关 `enable_domain_boost` 默认开（D-14）。

排序基线（T-050）不得漂移：领域包无命中时，候选清单必须与场景 8 之前的构建逐位一致。

## Decision

### 核心层：`zhu-ye-core/src/domain_boost.rs`

`domain_boost_candidates(packs, composing) -> Option<Vec<Candidate>>` 按调用方传入
顺序（装配层保证 id 升序，D-16）对各包做完整词 `lookup(composing)`，取首个非空包命中
（前缀天然不命中——`lookup` 是精确匹配）。返回候选的 `source = Domain`、保留词条
pinyin/译文、`score = i64::try_from(frequency).unwrap_or(i64::MAX)`，组内经共享的
`CandidateSorter` 按词频降序；`DOMAIN_BOOST_CAP`（8）截断组大小。无命中 / 空串 /
无关串返回 `None`。

### 配置

`ConfigFile.enable_domain_boost: bool`（`#[serde(default = "default_domain_boost")]`，
默认 `true`，D-14）持久化到 `config.json`。`PackPlan` 新增 `pack_ids: Vec<String>`——
与 `packs` 一一对应的包 id（配置顺序），供装配层排序。

### 引擎（`crates/zhu-ye-ime/src/input.rs`）

`InputEngine` 新增 `domain_packs: Vec<(String, Arc<dyn Dictionary>)>`（仅已启用包、
装配时按 id 升序）与 `enable_domain_boost: bool`（默认 true），经 `with_domain_packs` /
`with_domain_boost` 设置。`refresh_candidates` 在 `self.candidates = main;` 之后、
FR-030 追加组（英文/缩写/emoji）之前插入提权步：构造借用 `Vec<(String, &dyn Dictionary)>`
后经 `append_group(main, boosted)` 追加领域组（D-13）。

TSF 层（`crates/zhu-ye-ime/src/tsf.rs`）新增 `domain_engine(...)`：读
`plan.pack_ids.zip(plan.packs)`、按 id 升序排序、逐个 `DictionaryFile::open`
（打开失败静默跳过——composite 阶段已记 `pack-skipped`）、`debug_log` 记录启用的包 id
后挂载。**TSF 交互代码零改动**：提权只影响候选内容与顺序。

### 追加组去重是真实场景下的主导行为

`append_group(main, extra)` 同文本时保留先出现的（main）候选——领域词条若与基础路径
文本相同则被吸收、无可见变化。真实 base 是 28 万词条大通用词典：覆盖几乎所有单字音节
与大量多字组合，领域包词条绝大多数撞车。可见收益 = 基础路径产不出该精确文本的领域词
（典型为生僻技术音译词）。这是设计语义——提权只能"追加不撞车的文本"或什么都不加，
因此 T-050 永不漂移。

因此 `host-e2e --m11` 用**内存可控领域词表**断言（确定性命中/不命中、前缀、开关关闭、
D-16 字典序、基础<领域<emoji 位次），并在提供真实词典路径时复核真实 28 万词上下文的
"无命中 ⇒ 逐位一致"。

## Alternatives considered

- **并入主排序器融合评分**：否决——会扰动位次、破坏 T-050 基线；追加成组保持基础排序
  不动。
- **领域词并入基础词典**：否决——领域包必须可分离（启用/禁用、可换包），并入会污染
  通用词典。
- **TSF 层按键态介入领域**：否决——无按键语义差异，保持零改动（与场景 6/7 同原则）。
- **放宽匹配（前缀/低频）**：否决——D-15 规定仅完整词提权；前缀提权会挤占普通拼音输入。

## Consequences

- 真实 base 上提权很安静：多数领域命中撞车被去重——这正是不漂移保证；少量不撞车的
  技术词条变为可见。
- `enable_domain_boost` 默认开；偏好纯基线的用户可在 `config.json` 关闭（D-14/D-17），
  关闭后恢复包的追加语义。
- 确定性：同输入 + 同包集合 ⇒ 同候选清单（FR-002）；包选择按 id 字典序、组内按词频
  降序。
- 领域候选标注 `source = Domain`；UI 不为此新增标签（除已规划的开关外无 UI 变化）。
- 新增验收命令 `host-e2e --m11 [<dictionary.zyct>]`；本批次后 T-057 eval 基准验证
  不变（Top1 84.7% / Top3 97.2% / 整句 21.0%）。

## Related notes

- [2026-09-30-format-symbol-emoji-candidates](2026-09-30-format-symbol-emoji-candidates.zh.md)
  与
  [2026-10-02-english-candidates-and-email-url-formats](2026-10-02-english-candidates-and-email-url-formats.zh.md)
  定义了排在领域组之后的追加组（D-13 顺序）。
