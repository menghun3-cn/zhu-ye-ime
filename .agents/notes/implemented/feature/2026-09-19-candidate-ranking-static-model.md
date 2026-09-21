# Agent Note: Candidate ranking static model for M2

Status: implemented

[中文](2026-09-19-candidate-ranking-static-model.zh.md) | English

## Problem

Candidates were ordered only by the unigram score in `CandidateSorter`, with no
context signal and no user-word signal. FR-002 requires frequency-aware ranking
that eventually combines unigram, bigram, and user learning, and T-006 will
produce bigram data without a stable storage format yet. The engine needed a
testable ranking interface now so that M1/M2 behavior stays deterministic while
the real data pipeline is still defined.

## Decision

`zhu-ye-core::candidate` now owns ranking instead of just sorting.

Candidate generation also moved into core: `generate_candidates`,
`candidate_from_entry`, and `deduplicate_and_sort` produce deterministic,
deduplicated candidates from a syllable segmentation path, so IME and CLI share
exactly the same pipeline. Core tests cover deterministic generation and
whole-word/segmentation dedup.

`RankingModel` is the injectable ranking interface; `StaticRankingModel` is the
current deterministic implementation. `RankingConfig` exposes the default
weights: `unigram_weight=1`, `bigram_weight=16`,
`bigram_frequency_cap=100_000`, `user_weight=48`,
`user_frequency_cap=10_000`. Scoring is integer-only with saturating multiply
and add, then sorted by score descending and text ascending.

`RankingContext` carries `previous_word` and the in-memory `UserDictionary`.
A user hit rewrites `CandidateSource` to `User`. `UserDictionary` gained
`frequency_by_word`, which sums over pinyin variants for the same text.

`BigramModel` is a separate data-source trait; `InMemoryBigramModel` supports
tests and local development, and `EmptyBigramModel` is the default. The
dictionary itself does not carry bigram counts, so T-006 can replace the
backend with an mmap or compressed index without touching ranking.

`InputEngine` now keeps `previous_word`: space and numeric selection update it
on commit, Enter clears it, Esc leaves it unchanged. The engine calls
`ranking.rank` after candidate generation. User-word persistence and selection
recording remain T-009; this task only wires the dictionary into ranking.

## Alternatives considered

**Put bigram tables inside `Dictionary`.** Rejected: dictionary lookup and
ranking have different lifetime and replacement cadence; T-006 storage work
would then leak into the ranking core.

**Extend `CandidateSorter` with bigram parameters.** Rejected: a plain static
function cannot be injected or replaced by an AI or mmap-backed ranking, and
making it configurable would obscure the deterministic base sort.

**Use floating-point or logarithmic probabilities.** Rejected: integer
saturating arithmetic is simpler, deterministic across platforms and
compilers, and sufficient for the weighted three-term formula.

**Build a text-indexed user dictionary now.** Deferred: linear
`frequency_by_word` is fine at current scale; T-009 will add an index or
persistent structure when user words grow.

## Consequences

The base sort stays deterministic, `de` still prefers `的` without context, and
the new tests cover bigram promotion, bigram-miss fallback, user-frequency
promotion, and two-run stability. The input engine records the previous word
after commit, which also gives TSF previews and future remembering hooks a
single source of truth.

The CLI now has `rank <dictionary> <pinyin> [previous-word]`, which feeds the
same generation and ranking path against a real v2 dictionary. Validation on
`data/artifacts/real.zyct`: `rank de` keeps `的` first with 3,957,141, and
`rank de 我们` keeps `的` first at 3,979,557 while `得` rises to 158,049; two
identical runs produced identical output. T-008 is complete.

Multi-syllable noise cleanup is implemented (T-021): when the full pinyin has
direct dictionary entries, `generate_candidates` no longer appends
syllable-combination candidates, so `jiao` no longer emits `给哦`, `xian` no
longer emits `洗按`, and `fazhan` no longer emits `发站`; fallback
combinations remain when there are no direct entries, such as `geio` →
`给哦`. Real-dictionary validation: `rank jiao` puts `叫` first with 92,723,
`rank xian` has no `洗按`, `rank fazhan` puts `发展` first with 3,680, and
`rank geio` keeps `给哦`. An explicit `xi an` separator remains a possible
future enhancement and does not block M2. AI-ranked output can later be a
new `RankingModel` implementation instead of extra fields on `Candidate`.
