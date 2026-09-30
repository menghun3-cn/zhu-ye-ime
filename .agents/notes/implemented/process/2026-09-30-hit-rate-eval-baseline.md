# Agent Note: hit-rate eval set and first baseline

Status: implemented

[中文](2026-09-30-hit-rate-eval-baseline.zh.md) | English

Related notes: [ime-experience-optimization](../process/2026-10-02-ime-experience-optimization.md)
(describes the M7 features this eval measures and the accepted Top1/Top3 targets),
[cli-benchmark-metrics-and-acceptance](../testing/2026-09-21-cli-benchmark-metrics-and-acceptance.md)
(latency metrics: this note covers *accuracy*, that one covers *speed*), and
[polyphone-gap-patch](../process/2026-09-30-polyphone-gap-patch.md) (dictionary
side of hit-rate; its added readings are part of the ranked corpus).

## Problem

Every ranking change so far (T-044 static ranking, T-053 abbrev pinyin, T-054
fuzzy/correction, T-055 sentence beam) was validated by single-point cases
(`rank`, host-e2e assertions). There was no **scaled, reproducible, comparable**
hit-rate metric, so "did this change actually improve typing" stayed a matter of
spot checks, and the user's four required enhancements (context completion,
numeric/symbol/emoji candidates, mixed input, domain auto-switching) had no
baseline to hold against.

## Decision

Build a deterministic eval infrastructure and record a first baseline:

1. **Sample set (`data/eval/`, committed)** — words come from a generator,
   sentence samples are human-curated:
   - `zhu-ye-dict eval-set <CC-CEDICT> <wordfreq> <out.tsv> [--top N]`: parse
     CEDICT with the exact same cleaning as the real-dict import (pure CJK,
     syllable count == character count, tone letters/numbers normalized to the
     engine's tone-less key space, same word keeps its first CEDICT reading),
     intersect with wordfreq, sort by frequency desc (tie-break by word form
     for determinism), truncate to N. Output `词<TAB>拼音<TAB>词频`.
   - `data/eval/sentences-100.tsv`: 100 hand-written everyday sentences,
     `期望句<TAB>无调连拼`, all ≥ 4 syllables so the beam's short-string
     anti-shake (min 3 syllables) never blocks them.
2. **Judgement (`zhu-ye-cli eval <词典> <词样本> [整句样本] [--out-miss 文件]`)** —
   words go through the exact main-candidate path (`generate_candidates` +
   `StaticRankingModel`, no previous word, empty user dict): Top1 = first
   candidate is the target word, Top3 = target within the first three; Top1 is
   also reported per frequency bucket (≥1M / ≥100k / ≥10k / rest) to expose
   low-frequency degradation. Sentences go through `sentence_candidates`
   directly (the engine merely prepends its first element, so the sentence
   path's top-1 is the eval's top-1). Top1 misses and sentence misses are
   written to the MISS file (`词`/`句` typed rows).
3. **Determinism is a property, not a hope**: fixed iteration order over the
   TSV, empty user dictionary, same ranking weights as the engine — verified by
   two consecutive runs producing identical output and identical MISS file MD5.

## Alternatives considered

- **Reuse host-e2e assertions as the metric.** Rejected: pass/fail counts do
  not degrade gracefully, cannot be bucketed by frequency, and each addition is
  a new hand-written case instead of a data-driven set.
- **Draw word samples from the dictionary file itself.** Rejected without a
  dumper API: the loader only exposes `lookup`/`lookup_prefix`, and adding an
  enumeration API changes the core contract for a dev-only tool. CEDICT∩wordfreq
  also represents *what users type* better than "what the dictionary ranks
  highest".
- **Auto-transcribe sentences from kTGHZ** (per-char standard readings).
  Rejected: polyphonic characters need sentence context ("地", "了", "得") that
  a per-char table cannot pick; human-curated pinyin for 100 sentences is
  auditable and correctable, tiny enough to review in a diff.
- **Accept judge candidates from the engine's full path** (abbrev + correction
  groups). Rejected for v1: word samples almost always have a non-empty main
  group, and the sentence path is judged on the sentence candidates themselves;
  mixing all M7 groups into the metric conflates features. A future
  "-all-groups" mode can add it.

## Consequences

- First baseline on `real.zyct` (2026-09-30 build, T-056 included):
  **Top1 84.7%, Top3 97.2%** (N=2000), buckets ≥1M 100% / ≥100k 91.0% /
  ≥10k 87.9% / <10k 83.0%; sentences **21.0%** (N=100). Both runs byte-identical.
- Reading of the numbers: Top3 is already strong; Top1 sits at 84.7% with the
  low-frequency bucket (83.0%) driving most misses — the natural targets for
  the upcoming enhancements (context completion, numeric/symbol candidates).
  Sentence top-1 of 21% quantifies the beam's weakness on arbitrary sentences
  (mostly same-sound word/char choice: 迟到→吃到, 晚上→玩上): this is the
  baseline the context/completion work must beat.
- Every future ranking change (T-058 context completion, T-059 numeric/symbol
  candidates, mixed input, domain auto) must run the same eval and not regress
  Top1/Top3 relative to this table; the number goes back into
  `docs/命中率评测设计.md` §5.
- New tests: `zhu-ye-dict` +4 (cleaning/intersection/dedup/top-truncation/TSV
  render/syllable-mismatch rejection), `zhu-ye-cli` +4 (row parsers, bucket
  boundaries). `data/eval/` files are data, not code — reviewed like a sample
  of real usage.
- MISS files go to `target/` (not committed): they are evidence, and the set in
  `data/eval/` is the reproducible source of truth.
