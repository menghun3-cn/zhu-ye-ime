# Agent Note: proptest 属性测试门禁（T-089）

Status: implemented

English | [中文](2026-10-04-proptest-property-gate.zh.md)

## Problem

FR-049 (M13-5) asks for a property-testing gate over the four core algorithm
areas whose correctness is compositional rather than fixture-based: syllable
segmentation, candidate ranking (deterministic order, total-order comparator),
the bigram model, and the sentence beam search. Fixture-only unit tests cannot
argue absence-of-panic or ordering invariants across the full value space, and
the ranking path had already regressed once in M7-A practice (beam degrading to
per-character concatenation) that a property layer would have caught. Budget:
the new suite must fit within a CI duration increase of ≤20% and must not lower
the T-057 eval baseline.

## Decision

Add `proptest = "1"` as a **dev-dependency of `zhu-ye-core` only** (testing
gle only; the runtime dependency graph is untouched) and put the suite in
`crates/zhu-ye-core/tests/properties.rs` (integration test, no `lib.rs`
surface change). Eight properties in four groups:

- **Segmentation** (`pinyin::segment_all`): any concatenation of 1..=8 standard
  syllables must yield a non-empty plan set, every plan must re-concatenate to
  exactly the input and consist only of complete syllables (no syllable can be
  dropped — the "拼接 == 原串" invariant); arbitrary ASCII lowercase strings
  (including illegal syllable mixes) never panic and plans keep the
  concatenation invariant; strings containing non-ASCII characters always
  return empty. Determinism asserted by calling twice and comparing.
- **Ranking** (`CandidateSorter::sort`): sorting is idempotent (re-sorting a
  sorted vector changes nothing) and adjacent pairs satisfy the total order
  (score descending, text ascending lexicographic on ties); the result equals
  the same vector sorted by the equivalent tuple key `(Reverse(score), text)`.
- **Bigram** (`InMemoryBigramModel`): after inserting a random row batch,
  `frequency(previous, word)` for every key with a last-write record equals
  that last frequency (this solidified the existing overwrite semantics of
  `insert` — repeated keys take the last value; see Consequences); successors
  are ordered by frequency descending with lexicographic ties, never exceed
  `limit`, and `limit = 0` returns empty; unknown preceding words return empty
  without panicking.
- **Beam** (`candidate::sentence_candidates`): under a random dictionary ×
  random valid pinyin input, output candidates never panic, contain no
  duplicate texts, stay within `SENTENCE_TOP_N`, keep non-empty texts, and —
  when the beam actually starts — carry `pinyin == input`; two calls with the
  same inputs produce identical output.

Failure handling: proptest's default shrinking produces minimal counterexamples
that are then "固化" into the module unit tests where applicable. Case counts
use the library default (256); the beam case is shared under the same default
because the whole suite runs in ~0.2s.

Gates: the suite is part of `cargo test --workspace` and `cargo clippy
--all-targets -- -D warnings` via the normal test target. Duration budget:
measured independently at 0.16–0.21s; the workspace delta is <0.5s, far under
the 20% threshold, so no case-count trimming was needed.

## Alternatives considered

- **Proptest inside module `#[cfg(test)]` blocks**: rejected — the property
  suite is cross-module (it composes `pinyin` + `candidate` + `bigram` +
  `dict`); an integration test is the cleaner boundary, keeps `lib.rs`
  untouched, and confines the proptest dependency tree to the test build.
- **Arbitrary implementations for `Candidate`/dictionaries via `Arbitrary`
  derives**: rejected — hand-written strategy functions (`any_candidate`,
  `any_dictionary`) express the exact shape (empty/duplicate texts, i64::MIN
  scores, CJK word strings) with far less code.
- **A separate `zhu-ye-proptest` crate**: rejected — no consumer needs these
  properties outside core; one dev-dependency in the crate under test is
  enough.
- **CI-only invocation (not in `cargo test --workspace`)**: rejected — the
  acceptance (14.1.5) requires the gate to live in the normal local gates so
  every contributor runs it.
- **Higher case counts (1024)**: rejected at design time to honor the 20%
  budget even on slower CI machines; 256 + shrinking already exercises the
  degenerate values that matter (empty, boundary, duplicates).

## Consequences

- **Behavioral fact consolidated by the bigram property**: `insert` on a
  repeated key overwrites the previous frequency (the code comment says
  "插入或累加" but the implementation is overwrite). The property asserts the
  actual semantics ("last write wins") and pins it; no sanitizer change was
  made because nothing relies on accumulate semantics today. If accumulation is
  ever wanted, it is a one-line semantic change plus this property's update.
- `Cargo.lock` gains proptest and its test-only transitive tree (bit-set,
  rand, quick-error…); zero runtime dependency change.
- The ranking order contract is now documented at the property level:
  `score` descending, ties broken by `text` ascending — identical to the
  comparator in `CandidateSorter::sort`.
- The eval baseline stays the reference for ranking changes (T-057): re-run
  after any ranking/beam-dictionary change and compare Top1/Top3/整句; this
  task re-verified it identical (84.7% / 97.2% / 21.0%).

Verification (2026-10-04): property suite 8/8 passes; workspace all green
(core lib 269 + properties 8 + other crates); fmt/clippy `-D warnings` and
`git diff --check` clean; eval re-run identical to T-057 baseline.

Related notes (kept active, cross-linked): [hit-rate eval baseline
(process/2026-09-30-hit-rate-eval-baseline.md)](../../implemented/process/2026-09-30-hit-rate-eval-baseline.md)
— the eval reference this gate must not lower; [candidate ranking static model
(feature/2026-09-19-candidate-ranking-static-model.md)](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
— ranking semantics the total-order properties encode.
