# Agent Note: Candidate ranking static model for M2

Status: implemented

[English](2026-09-19-candidate-ranking-static-model.md) | 中文

## 问题

候选此前只用 `CandidateSorter` 的 unigram 分数排序，没有上下文信号，也没有用户词信号。FR-002 要求具备词频感知的排序，最终要组合 unigram、bigram 与用户学习；T-006 将产出 bigram 数据，但存储格式尚未定型。引擎需要在现在就有一个可测的排序接口，让 M1/M2 行为保持确定，同时真实数据管线仍在设计中。

## 决策

`zhu-ye-core::candidate` 现在负责排序模型，而不只是简单排序。

候选生成也迁入 core：`generate_candidates`、`candidate_from_entry` 与 `deduplicate_and_sort` 从音节切分路径产出确定且去重的候选，IME 与 CLI 共用同一管线；core 测试覆盖确定性生成与整词/切分去重。

`RankingModel` 是可注入的排序接口；`StaticRankingModel` 是当前确定性实现。`RankingConfig` 暴露默认权重：`unigram_weight=1`、`bigram_weight=16`、`bigram_frequency_cap=100_000`、`user_weight=48`、`user_frequency_cap=10_000`。计分全部使用整数饱和乘加运算，随后按分数降序、文本升序排序。

`RankingContext` 携带 `previous_word` 与内存 `UserDictionary`。用户词命中时把 `CandidateSource` 改写为 `User`。`UserDictionary` 新增 `frequency_by_word`，对同一文本的不同拼音变体求和。

`BigramModel` 是独立的数据源 trait；`InMemoryBigramModel` 用于测试与本地开发，`EmptyBigramModel` 是默认实现。词典本身不承载 bigram 计数，T-006 后续可以换成 mmap 或压缩索引后端，排序逻辑无需改动。

`InputEngine` 现在维护 `previous_word`：空格与数字选择提交后更新，回车清空，Esc 保持不变。引擎在候选生成后调用 `ranking.rank`。用户词持久化与选择记录仍归 T-009；本任务只把用户词典接入排序。

## 曾考虑的替代方案

**把 bigram 表放进 `Dictionary`。** 落选：词典查询与排序的生命周期和替换节奏不同；T-006 存储工作会泄漏进排序核心。

**给 `CandidateSorter` 增加 bigram 参数。** 落选：普通静态函数无法注入，也无法被 AI 或 mmap 后端排序替换；可配置化反而会模糊确定性的基础排序。

**使用浮点或对数概率。** 落选：整数饱和运算更简单、跨平台跨编译器确定，且足以支撑三项加权公式。

**现在构建按文本索引的用户词典。** 延期：当前规模下线性 `frequency_by_word` 已经够用；T-009 在用户词增长后再加入索引或持久化结构。

## 后果

基础排序保持确定，无上下文时 `de` 仍首选 `的`；新增测试覆盖 bigram 提升、bigram 未命中回退、用户词频提升与两次运行结果稳定。输入引擎在提交后记录前词，也为 TSF 预览和未来的记忆钩子提供单一事实来源。

CLI 新增 `rank <词典文件> <拼音> [前词]`，用同一生成与排序路径直接验证真实 v2 词典。`data/artifacts/real.zyct` 验证结果：`rank de` 的仍以 3,957,141 居首；`rank de 我们` 的以 3,979,557 居首、得提升至 158,049；相同命令两次运行输出一致。T-008 验收完成。

多音节噪声治理已落地（T-021）：整词拼音存在直接词典条目时，`generate_candidates` 不再追加音节切分组合，`jiao` 不再产出 `给哦`、`xian` 不再产出 `洗按`、`fazhan` 不再产出 `发站`；无直接条目时仍保留回退组合，如 `geio` → `给哦`。真实词典复验：`rank jiao` 的 `叫` 92,723 居首，`rank xian` 无 `洗按`，`rank fazhan` 的 `发展` 3,680 居首，`rank geio` 保留 `给哦`。后续可考虑用 `xi an` 分隔符显式表达多音节词边界，当前不阻塞 M2。AI 排序结果以后可以新增 `RankingModel` 实现承载，而不是给 `Candidate` 增加字段。
