# Agent Note: CLI benchmark metrics and acceptance script

Status: implemented

English | [中文](2026-09-21-cli-benchmark-metrics-and-acceptance.zh.md)

## Problem

FR-011 requires measurable candidate-refresh latency and FR-014 requires a
diagnostic benchmark command. The earlier `zhu-ye-cli bench` covered only
syllable segmentation, so there was no repeatable measurement or gate for
dictionary lookup, bigram, or bilingual translation work, and no script to
turn the output into an acceptance result.

## Decision

`zhu-ye-cli bench` now measures five operations: pinyin segmentation,
candidate dictionary lookup across five representative pinyin queries, bigram
frequency, Chinese-to-English translation, and English-to-Chinese reverse
translation. Each measurement prints the operation count, total elapsed time,
and average per operation. The command ends with one parseable line:

```text
指标: segment_us=6.051 lookup_us=2.056 bigram_us=1.012 zh_en_us=0.747 en_zh_us=1.434
```

`zhu-ye-cli self-check` reuses the same measurement functions with a small run
count and prints a quick performance baseline line. `scripts/bench.ps1`
ensures `data/artifacts/seed.zyct` exists (building it with `zhu-ye-dict` when
missing), runs `bench`, parses the `指标:` line, and fails when any `*_us`
metric exceeds the threshold. The script defaults to 1000us per operation,
supports `-Release` for release-build acceptance runs, and accepts
`-MaxUsPerOp` to tighten the threshold for a specific machine.

## Alternatives considered

**Use cargo bench only.** Rejected: the project's acceptance procedure is a
Windows PowerShell workflow, and `cargo bench` does not produce the same
human-readable plus machine-parseable summary needed by `scripts/bench.ps1`.

**Embed thresholds in Rust.** Rejected: pass/fail policy belongs with the
acceptance tooling and varies by machine; the CLI should only emit facts.
The PowerShell script owns the default threshold and allows an explicit
override at run time.

**Emit JSON only.** Rejected: a single labeled `指标:` line is easy to parse
for the acceptance script while staying readable in a developer terminal.

## Consequences

Latency facts for segmentation, lookup, bigram, and both translation
directions are now available in one command and can be gated repeatedly before
release. Debug builds are deliberately labeled as trend-only; formal
acceptance still runs `scripts/bench.ps1 -Release` on the target machine
against the thresholds in `docs/验收标准.md`.
