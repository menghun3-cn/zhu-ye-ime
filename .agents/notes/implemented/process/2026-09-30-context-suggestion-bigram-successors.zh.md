# Agent Note: 上下文联想：bigram 后继检索

状态：已实现

[English](2026-09-30-context-suggestion-bigram-successors.md) | 中文

相关笔记：[hit-rate-eval-baseline](../process/2026-09-30-hit-rate-eval-baseline.md)
（用户四项增强必须守住准确率基准；本笔记为第②项"上下文联想"提供检索能力）与
[ime-experience-optimization](../process/2026-10-02-ime-experience-optimization.md)
（本能力归属的 M7 功能族：交互语义沿用候选窗既有约定）。

## Problem

用户场景 5（上下文联想）：打完"今天天气"后，候选窗应出现"怎么样 / 不错 / 很好"——
引擎要结合**已上屏前文**，而不是孤立看当前拼音。引擎已有 `frequency(previous, word)`
点查用于带上下文排序，但没有"这个词的高频后继是什么"的询问能力；而朴素做法
（"遍历整个 bigram 表"）需要一个新词典格式或新分区。

## Decision

新增只读后继检索，**词典格式零改动**：

1. **`BigramModel::successors(previous, limit) -> Vec<(String, u64)>`** —— trait
   方法带**默认空实现**：没有后继索引的模型退化为"不联想"，不触碰现有排序语义。
   `InMemoryBigramModel` 按频率降序、同频词形字典序取 top；`EmptyBigramModel` 恒空。
2. **`DictionaryFile` 复用现有 bigram 表布局** —— 表按 `(previous, word, frequency)`
   排序构建（见 `dict_builder`），同一前词的记录**连续**；`successors` 对
   `(previous, "")` 二分取下界后顺序扫描连续区段，按频率降序截取 `limit` 条。
   无字节布局改动、无新分区，对既有 `.zyct` 文件完全兼容。
3. **`CompositeDictionary` 各源取后继并按词合并取 max**（与 `frequency` 同口径）
   再截取——多包聚合口径保持一致。
4. **`suggestion_candidates(model, previous) -> Vec<String>`** —— 面向用户的列表：
   Top5 后继整词 + 至多 3 条 **"前词+后继"** 两词短语（明天见→明天见你），短语置
   整词之后、总量 ≤8；`previous` 为空返回空（联想只在刚上屏过一个词后触发）；
   后继与前词同形的短语（无信息量）跳过。
5. **CLI 抓手**：`zhu-ye-cli suggest <词典> <前词>` 调用与引擎（T-059）完全同一
   函数，现在就提供调试与 VM 探针入口。

### 真实词典验证（2026-09-30 构建）

| 前词 | 后继（前 5） | 短语 |
| --- | --- | --- |
| 明天 | 的 / 早上 / 会 / 我们 / 又 | 明天的 / 明天早上 / 明天会 |
| 天气 | 型态 / 很 / 也 / 变得 / 里 | 天气型态 / 天气很 / 天气也 |
| 今天 | 的 / 是 / 早上 / 我 / 在 | 今天的 / 今天是 / 今天早上 |

机制成立；结果**语体**受 bigram 语料（GlobalVoices 新闻）限制，"怎么样/不错"等
口语后继不在高频区——这是数据属性而非机制缺陷，换口语语料属数据项，记入 T-059 工作。

## Alternatives considered

- **给词典格式加独立"后继索引"分区。** 拒绝：现有排序 bigram 表的连续区段特性
  使分区成为冗余，还会造成数据双份、格式升级与全量重建；且扫描只发生在空闲
  候选窗（非热路径），O(区段长) 完全可接受。
- **只复用现有 `RankingContext` 前词排序。** 拒绝：排序只影响**输入拼音时**候选
  顺序，无法产出空闲窗的后继列表——而用户明确选择的就是"上屏后空闲候选窗"。
- **短语 = 后继 + 其最佳再后继**（明天见→你吧）。评审时拒绝：用户例子明确是
  **前词+后继**（明天见→明天见你），短语以"已上屏词 + 下一词"拼接。
- **联想数据预计算成静态表入库。** 拒绝：与 bigram 状态重复、会与词典漂移，
  且最终仍需要同样遍历。

## Consequences

- `zhu-ye-core` 139 测试全绿（bigram +2：后继排序/空；suggestion +4：整词顺序、
  空守卫、短语拼接含同形跳过；loader 回归断言并入现有 roundtrip 测试）。
  `zhu-ye-cli` 带新命令与其 usage 行构建通过。
- 确定性：后继列表直接来自（确定性的）bigram 表——同词典同列表；引擎可以缓存
  空闲窗结果，但每次上屏后必须重算。
- hit-rate-eval-baseline 的基准契约**不受影响**：联想只在拼音为空时出现，从不参与
  候选排序；后续排序改动仍不得回退 Top1/Top3。
- T-059（引擎承接，不在本次改动内）：空闲候选窗状态机、从候选窗上屏、任意输入
  退出联想、host-e2e `--m8` 断言、VM UI 验收。

## Supersession 检查

无既有活跃笔记被取代：本笔记在 bigram 表之上新增检索能力；hit-rate-eval-baseline
（准确率）、ime-experience-optimization（M7 语义）、cli-benchmark-metrics-and-acceptance
（时延）均保持活跃并已互相交叉引用。
