# Agent Note: 拼音未完整切分时的前缀候选

Status: implemented

[English](2026-09-24-prefix-candidates-incomplete-segmentation.md) | 中文

## Problem

T-029 之前，引擎只在输入能完整切分为音节时才出候选（`nihao` → 你好）。
输入 `nih` 的过程里候选窗一直为空，尽管词典已经能把字母补全成词（`nihao` →
你好），而且最后完整音节 `ni` 本身也有单音节候选（你/泥…）。候选窗直到
`nihao` 才出现，实时反馈很弱——这正是 T-029 要解决的诉求。

## Decision

- **core：** `generate_prefix_candidates(table, dictionary, pinyin, completion_cap)`
  仅当 `pinyin` 非空、`segment_all` 无法完整切分、且存在至少一个可完整切分的前缀时，
  返回 `PrefixCandidateGroups { completions, completed }`：
  - 组 2（`completions`）：`dictionary.lookup_prefix(pinyin)` 映射为候选，
    按词频降序，截断到 `completion_cap.max(1)` 条（ime 引擎传入 32）；
  - 组 1（`completed`）：对最长完整前缀 `P`（从尾部向前切，直到
    `segment_all` 成功）跑常规 `generate_candidates`；
  - 不存在任何完整前缀时（`z`、`zh`），两组皆空——完全不出候选，防止前缀泛滥。
- **词典契约（跨模块）：** `Dictionary` trait 新增必选方法
  `lookup_prefix(&self, &str) -> Vec<DictionaryEntry>`，所有实现都必须提供。
  `InMemoryDictionary` 过滤 pinyin 以该前缀开头的词条并按词频降序（同频稳定）；
  `DictionaryFile` 在有序拼音索引上做下界二分找到第一个 >= 前缀的键，
  线性展开连续匹配的拼音段，再按词频降序。v2 文件格式不变。
- **排序：** `merge_candidate_groups` 先组 2 后组 1 拼接，按候选文本去重并
  保留组 2 条。ime 引擎对两组分别用 `StaticRankingModel`（前词 + 用户词典
  上下文）排序后再融合，模型权重无法打乱组间边界。
- **交互不变量：** 数字选中即提交并清空整个组合串，残留尾部随之丢弃（`nih`
  选"你"→ 上屏"你"、残留 h 清掉）；Enter 仍上屏拼音原文；Backspace 天然重算
  两组；融合列表按 9 项/页跨组线性翻页；排序确定。
- **CLI 边界：** `rank` 子命令刻意保持"完整切分候选"语义；前缀行为由引擎
  单测与 host-e2e 断言覆盖。

## Alternatives considered

**只对尾部字母做词典补全、不含组 1。**
否决：丢掉了用户已经敲出的最后完整音节候选（`nih` 应在 `ni` 敲完时就能给出
"你"），且 Backspace 反馈不一致。

**把残尾当作一个音节复用 `generate_candidates`。**
否决：整词优先、音节组合的语义会被扭曲，也没有天然位置承载前缀补全排序。

**融合后的列表整体过一次全局排序模型。**
否决：上下文权重会把组 1 与组 2 交错得过于激进；组 2 必须按设计前置。

**组 2 不设上限。**
否决：`n` 这类短前缀可命中上千条；上限（引擎 32）约束查询与排序成本，
同时仍能覆盖多页 9 项。

**任何前缀命中都出候选，包括无完整音节的输入（`z`、`zh`）。**
否决：纯声母短输入会刷屏；此类输入两组必须为空。

**维持原状（完整切分前不出候选）。**
否决：那就是 T-029 的出发点。

## Consequences

输入现在有实时候选——`nih` → 你好/尼好（组 2，按词频）再加"你"（组 1），
顺序确定。`Dictionary` 是 crate 级契约，未来新实现（如 sqlite 词典）必须
提供 `lookup_prefix`；v2 文件格式未动，新查询路径是二分 + 受补全上限约束的
连续段扫描。

验证：单测覆盖生成（组优先级、空用例、上限截断、确定性）与融合（去重保组 2）、
内存与文件词典的前缀查询、ime 行为（选中残留丢弃、Backspace 重算、`z`/`zh`
无候选词）；host-e2e 增前缀断言。VM v0.1.1 种子词典演练全场景通过：`nih`
候选窗可见 3 项、`zh` 无候选词、Backspace 重算回 `ni`、数字 3 上屏"你"且
残留 h 丢弃、落盘字节（GBK `C4 E3 BA C3 C4 E3 C4 E3`）即"你好你你"。截图
（shots6）留档待人工审查。T-031 起，组合串非空但两组皆空时候选窗仍保持
可见、只显示页眉条（见 2026-09-25-candidate-window-light-default-and-empty-panel.zh.md）。
