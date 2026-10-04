# Agent Note: proptest 属性测试门禁（T-089）

Status: implemented

English | [中文](2026-10-04-proptest-property-gate.zh.md)

## Problem

FR-049（M13-5）要求对 core 四个"组合性正确"的算法域建立属性测试门禁：音节切分、
候选排序（确定性、比较器全序）、bigram 模型与整句 beam 搜索。纯 fixture 单测无法
在全值域上论证"不 panic"与排序不变量；且排序路径在 M7-A 实测中已经出现过一次
beam 退化为逐字拼接的回归，属性层本可提前捕获。预算：新套件必须满足 CI 时长增量
≤20%，且不降低 T-057 eval 基准。

## Decision

把 `proptest = "1"` 作为 **zhu-ye-core 的 dev-dependency**（仅测试期；运行时依赖
图零变化），套件放 `crates/zhu-ye-core/tests/properties.rs`（集成测试，`lib.rs`
表面零改动）。四组共 8 项属性：

- **切分**（`pinyin::segment_all`）：任意 1..=8 个标准音节拼接必得非空方案集、每个
  方案重新拼接必精确还原输入且只含完整音节（"不遗漏音节"即 拼接 == 原串 不变量）；
  任意 ASCII 小写串（含非法音节组合）不 panic 且方案保持拼接守恒；含非 ASCII 的串
  恒返回空。确定性：两次调用逐位相等。
- **排序**（`CandidateSorter::sort`）：排序幂等（再次排序结果不变），相邻项满足全序
  （score 降序、同分按 text 字典序升序）；结果等于按等价元组键 `(Reverse(score),
  text)` 排序的结果。
- **bigram**（`InMemoryBigramModel`）：随机批量插入后，每个键的
  `frequency(previous, word)` 等于该键最后一次写入的频次（此断言固化 `insert` 既有
  的覆盖语义——同键重复插入以末次为准，见 Consequences）；后继列表按频率降序、同频
  词形字典序、条数不超过 `limit`、`limit = 0` 必空；未知前词返回空且不 panic。
- **beam**（`candidate::sentence_candidates`）：随机词典 × 随机合法拼音输入下，输出
  候选不 panic、文本不重复、条数不超过 `SENTENCE_TOP_N`、文本非空，且 beam 真正
  启动时 `pinyin == 输入`；同输入两次调用输出一致。

失败策略：proptest 默认最小反例收缩；收缩出的最小反例适合的固化为模块内单测。采样
数用库默认（256）；beam 共享同一默认——整套实测仅约 0.2s。

门禁：套件随 `cargo test --workspace` 与 `cargo clippy --all-targets -- -D warnings`
正常测试目标纳入。时长预算：独立实测 0.16~0.21s；workspace 增量 <0.5s，远低于
20% 阈值，未触发裁剪。

## Alternatives considered

- **proptest 内嵌各模块 `#[cfg(test)]`**：否决——属性套件跨模块（组合 `pinyin` +
  `candidate` + `bigram` + `dict`）；集成测试边界更干净，保持 `lib.rs` 零改动，并
  把 proptest 依赖树限制在测试构建内。
- **给 `Candidate`/词典派生 `Arbitrary`**：否决——手工策略函数（`any_candidate`、
  `any_dictionary`）用更少代码精确表达目标形状（空/重复文本、i64::MIN 分数、CJK
  词串）。
- **独立 `zhu-ye-proptest` crate**：否决——core 之外无消费者；被测 crate 内一个
  dev-dependency 足矣。
- **仅 CI 调用（不并入本地 `cargo test --workspace`）**：否决——验收（14.1.5）要求
  门禁落在常规本地门禁里，每位开发者都会跑。
- **提高采样数到 1024**：设计期否决——即便在较慢 CI 上也守 20% 预算；256 + 收缩
  已覆盖需要关注的退化值（空、边界、重复）。

## Consequences

- **bigram 属性固化的行为事实**：`insert` 对重复键是覆盖旧频次（代码注释写"插入或
  累加"，实现是覆盖）。属性按实际语义断言（"末次写赢"）并钉死；未改实现——今天没有
  任何代码依赖累加语义。将来若要累加，是一行语义改动 + 本属性同步更新。
- `Cargo.lock` 新增 proptest 及其测试期传递树（bit-set、rand、quick-error 等）；
  运行时依赖零变化。
- 排序序契约在属性层获文档化：score 降序、同分按 text 升序——与
  `CandidateSorter::sort` 比较器一致。
- eval 基准仍是排序改动的对照锚（T-057）：任何排序/beam 词典改动后复跑并对比
  Top1/Top3/整句；本任务复验与基准完全一致（84.7% / 97.2% / 21.0%）。

验证（2026-10-04）：属性套件 8/8 通过；workspace 全绿（core lib 269 + 属性 8 +
其余 crate）；fmt/clippy `-D warnings` 与 `git diff --check` 干净；eval 复跑与
T-057 基准一致。

相关记录（均保留活跃并交叉链接）：[命中率评测基准
（process/2026-09-30-hit-rate-eval-baseline.zh.md）](../../implemented/process/2026-09-30-hit-rate-eval-baseline.zh.md)
——本门禁不得降低的 eval 参照；[候选排序静态模型
（feature/2026-09-19-candidate-ranking-static-model.zh.md）](../../implemented/feature/2026-09-19-candidate-ranking-static-model.zh.md)
——全序属性所编码的排序语义出处。
