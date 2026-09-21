# Agent Note: CLI benchmark metrics and acceptance script

Status: implemented

[English](2026-09-21-cli-benchmark-metrics-and-acceptance.md) | 中文

## 问题

FR-011 要求候选刷新延迟可量化，FR-014 要求提供可诊断的基准命令。此前的 `zhu-ye-cli bench` 只覆盖拼音切分，没有针对词典查找、bigram 或双语翻译的可复现测量，也没有把输出变成验收结果的脚本。

## 决策

`zhu-ye-cli bench` 现在测量五类操作：拼音切分、五个代表性拼音串的候选词典查找、bigram 共现频次、中文到英文正查、英文到中文反查。每项测量输出操作数、总耗时与单次平均耗时。命令末尾输出一行可解析结果：

```text
指标: segment_us=6.051 lookup_us=2.056 bigram_us=1.012 zh_en_us=0.747 en_zh_us=1.434
```

`zhu-ye-cli self-check` 复用同一套测量函数，用较小的运行次数输出快速性能基线。`scripts/bench.ps1` 会确保 `data/artifacts/seed.zyct` 存在（缺失时用 `zhu-ye-dict` 构建），执行 `bench`，解析 `指标:` 行，任一 `*_us` 指标超过阈值即失败。脚本默认阈值 1000us/次，支持 `-Release` 做 release 构建验收，也支持 `-MaxUsPerOp` 针对具体机器收紧阈值。

## 曾考虑的替代方案

**只用 cargo bench。** 否决：项目验收流程是 Windows PowerShell 工作流，且 `cargo bench` 没有 `scripts/bench.ps1` 需要的“可读 + 可解析”汇总行。

**把阈值写死在 Rust。** 否决：通过与失败策略属于验收工具，且因机器而异；CLI 只需要输出事实。PowerShell 脚本持有默认阈值，并允许运行时显式覆盖。

**只输出 JSON。** 否决：单行带标签的 `指标:` 既便于验收脚本解析，也方便开发者在终端阅读。

## 影响

切分、查找、bigram 与两个翻译方向的延迟事实现在一个命令即可获得，并可在发布前反复做门槛检查。debug 构建明确标注为仅供趋势参考；正式验收仍在目标机器上按 `docs/验收标准.md` 的阈值执行 `scripts/bench.ps1 -Release`。
