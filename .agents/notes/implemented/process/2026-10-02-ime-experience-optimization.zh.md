# Agent Note: 输入法体验优化——简拼、模糊音纠错、整句 Beam Search

Status: implemented

[English](2026-10-02-ime-experience-optimization.md) | 中文

## 问题

M7 范围（docs/输入法体验优化.md）对准拖累输入法的验收指标：Top1 ≥ 85–90%、
Top3 ≥ 95%、整句准确率 90%+、首屏 < 50ms、个性化、接近搜狗体验。用户三项全选，
且带硬约束：必须离线、自研、确定性，复用现有音节表 + bigram + 用户词基础设施，
不得改动 v2 词典格式、构建管线与更新链路（决策 O-01..O-06）。

具体三个缺口：

1. **没有简拼/首字母输入。** `nh`→你好、`wsm`→为什么 打不出来：组合串不可切分，
   常规管线产不出任何候选。
2. **没有模糊音纠错。** `zongguo`（zh↔z）、`niha`（缺尾部 `o`）产不出像样的
   候选，也没有机制在不污染常规排序的前提下提出纠错形态。
3. **没有整句搜索。** `woxiangmingtianqubeijing` 这类长串按音节逐字拼接（每音节
   取首词），完全忽略 bigram 连贯性与跨音节整词。

## 决策

### 1. 简拼（FR-023）

`pinyin::INITIAL_SYLLABLE_TABLE` 是自维护的静态表：22 个首字母
（`b c d e f g h j k l m n o p q r s t w x y z`）各映射 4–6 个常用音节，
按语言事实的频率排序（无需许可证登记）。`initial_syllables(initial)` 返回切片；
未收录字母返回空。

`candidate::initial_candidates(dictionary, initials)`：

- 长度 2..=4，仅小写 ASCII——数字与单字符直接拒绝（拒绝单字符扇出；
  数字归属 T-049 数字缩写语义）；
- 每字母音节集按前序笛卡尔积展开，每个组合在词典中**整词**查询；
- 任一字母无表项、或组合数超过 128，整个函数返回空（防爆炸：4 位 × 6 音节 =
  1296 种组合原本会被枚举）；
- 结果上限 `INITIAL_COMPLETION_CAP = 32`，顺序 = 表序展开序（确定性）。

引擎集成刻意收窄（防污染，O-02）：简拼路径**仅**在主候选为空、组合串为
2–4 位小写字母、且 `segment_all` 不可切分时运行。`wo`、`nih` 等合法拼音
永不进入该路径，既有行为不受影响。

### 2. 模糊音纠错（FR-024）

`pinyin::FUZZY_GROUPS` 持有设计文档的 7 组映射：zh↔z、ch↔c、sh↔s、n↔l、
f↔h、an↔ang、en↔eng、in↔ing。`fuzzy_variants(syllable)` 每次恰好**单处**替换，
HashSet 去重、不含原音节、确定性。

`candidate::corrected_candidates(table, dictionary, pinyin)` 仅当输入串
**可完整切分**且**无整词命中**（精确查询为空）时触发：

- **A — 模糊替换**：对首个切分方案的每个音节做 `fuzzy_variants` 原地替换，
  重新拼接后整词查询；
- **B — 少字母补全**：仅对**最后一个**音节用 `complete_syllables_with_prefix`
  扩展（如 `ha`→`hao`），同样整词查询。

命中候选标注 `CandidateSource::Corrected`，作为独立组返回（UI 不新增标签）。
引擎用 `append_group` 追加在**主候选之后、缩写组之前**；文本冲突时主候选优先。
数量上限 `CORRECTION_VARIANT_CAP = 24`。

### 3. 整句 Beam Search（FR-025）

`candidate::sentence_candidates` 仅当输入串可完整切分、音节数 ≥ 3、且无整词
命中时触发。关键实现洞察：**跨音节整词匹配**。朴素的逐音节 beam 永远选不出
`明天`（拼音 `mingtian`，两个音节），因为每步只查一个音节。搜索改为在每个位置
枚举所有可完整切分的子串（1 至 `SENTENCE_MAX_WORD_CHARS = 12` 字符，即最多 4
个音节）并整词查询。

评分按**词间转移**建模而非裸频率相加；M7-A 在真实词库上把三个事实调定后才通过
验收用例：

1. **unigram 上限**（`SENTENCE_UNIGRAM_CAP = 100_000`）：不设上限时超高频虚词单字
   （如介词「被」）以原始词频碾压整词（真实 base 包中「北京」unigram 仅 1088），
   beam 退化为逐字拼接。
2. **bigram 缺失惩罚 = cap**（`SENTENCE_BIGRAM_MISS_PENALTY = 100_000`）：只要罚额
   < cap，被 cap 的高频单字罚后残值仍为正，两个残值相加即可压过低频整词；罚额 =
   cap 使无证据转移贡献 ≤ 0，「去被敬」（去→被 无 bigram 证据）必然深于有证据的
   「去北京」。
3. **完成路径跨轮保留**：整词步长（「明天」= 8 字符）让胜出路径提前完成，而逐字
   路径仍在推进；没有独立的 `completed` 集合，慢路径会在后续轮次不断替换已完成的
   路径，top 候选就退化。

评分 = min(词频, cap) × unigram权重 + [前词→本词有证据(>0)时 min(bigram, cap) ×
bigram权重；无证据时减惩罚]，全部饱和 i64（确定性）。beam 状态为（词序列，已消费
字符数，累计分）；每步保留 `BEAM_WIDTH = 8` 条路径，每个子串至多贡献
`BEAM_WORD_CAP = 4` 词；同分按词文本升序（确定性）。每步至少消费 1 字符，保证
≤ len 步终止。完整路径按整句文本去重（取最高分），返回前 `SENTENCE_TOP_N = 5`，
`pinyin` = 原输入串。

引擎用 `prepend_group` 把整句组置于**最前**（文本冲突时主候选让位）。若全部
beam 路径耗尽，函数返回空，调用方保留普通逐音节候选，保证 ≥ 1 候选。

### 引擎装配

`InputEngine` 新增 `bigram: Arc<dyn BigramModel>` 字段（beam 需要；
`StaticRankingModel` 仍自带一份）。`new` 用 `EmptyBigramModel`，`with_bigram` /
`with_user_store_and_bigram` 及文件构造器把同一实例同时交给排序模型与 beam。
`refresh_candidates` 流程：前缀组（T-029，不变）→ 主候选 → 整句组（有则最前）→
纠错组（主候选之后）→ 简拼（仅主候选仍空且不可切分时）→ 缩写尾部组（M6-R，
不变）。新增 `prepend_group` / `append_group` 两个小助手，按文本去重同时保持
组间固定顺序。

## 备选方案

**更宽的简拼触发（单字符、或字母+数字）。** O-02 否决：单字符必然跨音节扇出
污染；数字归属 T-049 缩写语义。窄门槛（主候选为空 + 不可切分 + 2–4 位小写）
才是 `wo`/`nih`/`u1s1` 被排除的原因。

**逐音节 beam。** 否决：选不出跨音节整词（`明天` = `mingtian`），验收头条用例
`woxiangmingtianqubeijing` → 我想明天去北京 只能靠单字频率、常常劣化。子串整词
查询是有界枚举（每状态 ≤ len × 12），远在时延预算内。

**整词命中也跑 beam。** 否决：`nihao` 必须照旧只出 你好/尼好；不漂移回归底线
禁止改变短串行为。

**纠错候选参与常规排序。** 否决：静态词频高的模糊命中可能压过用户意图的拼音
候选；独立标注组置于主候选之后是 O-03 约定的防污染位置。

**纠错只用首个切分方案。** 否决：DP 顺序不保证最长词优先；最终实现根本**不选定**
某个切分方案——beam 按位置枚举可完整切分的子串，音节数下限只作触发防御。

## 后果

- `CandidateSource` 新增 `Corrected` 变体（无 UI 标签），作为独立组追加；
  `corrected_candidates` 与 `sentence_candidates` 是 `zhu-ye-core` 的纯函数，
  无 Windows 依赖。
- 既有行为恰好两处可观察变化，均为预期：`niha` 现在给出 你好/尼好（纠错组），
  长不可切分串获得简拼结果；页码收敛测试相应更新（断言改为退格三次到 `ni` 时
  页码归零，`niha` 阶段因纠错候选仍可能多页）。
- 确定性：三条路径全部顺序确定（表序、HashMap 仅用于去重且随后显式排序）；
  无随机、无墙钟。
- 性能：每条路径都有上限（≤128 组合、≤24 纠错、8×4 beam×5）；整句路径在每个
  位置做有界子串枚举。首屏时延（<50ms）的 VM 验收在 M7-A。
- 测试：core +13（pinyin 4：表覆盖/查询未收录/单处替换/无映射空；candidate 9：
  简拼 5、纠错 3、整句 3——含头条用例 `woxiangmingtianqubeijing` →
  我想明天去北京、bigram 引导、M7-A 追加的超高频单字对抗用例、不漂移守卫）；
  zhu-ye-ime 引擎级 +10（nh/wsm、zongguo、niha、整词不触发、整句居首、短串
  不漂移）。全套门禁绿：fmt、clippy `-D warnings`、`cargo test --workspace`
  （core 132、ime 99）、`git diff --check`、host-e2e 种子 19/19 + 多包 7/7 +
  新增 `--m7` 断言组真实词典 22/22。
- 性能（实测，release + 真实 base 包，基准 8.2）：`zhu-ye-cli bench` 新增三路径
  场景；简拼 13.6 µs、纠错 5.7 µs、整句 1630 µs（≈1.6 ms）每次刷新——全部落在
  ≤30ms 验收值（目标 ≤15ms）内。零网络由构造保证（O-05）。
- 关联活跃 note（保持活跃并交叉引用）：切分基础设施
  （feature/2026-09-19-full-pinyin-segmentation-core）、前缀候选
  （feature/2026-09-24-prefix-candidates-incomplete-segmentation，简拼与之互补
  处理不可切分串）、排序权重
  （feature/2026-09-19-candidate-ranking-static-model，beam 复用其权重）。
  M7 批次未取代任何既有 note；语言事实表（简拼表、模糊音组）自研维护，
  无需 licenses.md 登记。
- VM 交互验收（验收标准 8.1）在验收 VM（Windows Server 2019 zh-CN，真实 TSF
  栈、四包 + 词典）端到端通过：6/6 断言全绿——`nh` 居首你好、`wsm` 居首为什么、
  `zongguo` 候选窗含中国（数字键 3 上屏）、`niha` 候选窗含你好（数字键 3 上屏）、
  整句用例居首我想明天去北京、`nh`+数字 1 上屏你好。证据：TSF 调试日志
  `cand-show first=`/`commit`、六张 MD5 各异的截图（画面真实变化）、像素取证
  （候选窗主题色包围盒：s1–s5 可见、s5 最宽与 items=9 一致、s6 上屏后隐藏）。
  部署备忘：TSF 进程内宿主会 mmap 锁词典文件，升级安装改装入全新目录
  （`tsf-m7`）规避占用，事后清理遗留目录。
