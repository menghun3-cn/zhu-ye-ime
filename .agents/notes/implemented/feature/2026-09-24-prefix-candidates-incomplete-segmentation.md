# Agent Note: Prefix candidates when the pinyin input is not fully segmentable

Status: implemented

[中文](2026-09-24-prefix-candidates-incomplete-segmentation.zh.md) | English

## Problem

Until T-029, the engine produced no candidates until the input could be fully
segmented into syllables (`nihao` → 你好). While typing `nih`, the candidate
window stayed empty even though the dictionary could already complete the
letters to words like 你好 (`nihao`) and the last complete syllable `ni`
already has single-syllable candidates (你/泥…). The window only appeared at
`nihao`, which is weak live feedback and the core complaint driving T-029.

## Decision

- **core:** `generate_prefix_candidates(table, dictionary, pinyin,
  completion_cap)` returns `PrefixCandidateGroups { completions, completed }`
  only when `pinyin` is non-empty, cannot be fully segmented by
  `segment_all`, and has at least one complete-syllable prefix:
  - group 2 (`completions`): `dictionary.lookup_prefix(pinyin)` mapped to
    candidates, frequency-descending, truncated to `completion_cap.max(1)`
    (the ime engine passes 32);
  - group 1 (`completed`): the regular `generate_candidates` result for the
    longest complete prefix `P`, found by cutting from the tail backward
    until `segment_all` succeeds;
  - when no complete prefix exists (`z`, `zh`), both groups are empty — no
    candidates at all, which prevents prefix flooding.
- **dictionary contract (cross-module):** the `Dictionary` trait gains a
  required method `lookup_prefix(&self, &str) -> Vec<DictionaryEntry>`;
  every implementor must provide it. `InMemoryDictionary` filters entries
  whose pinyin `starts_with` the prefix and sorts frequency-descending with
  stable ties. `DictionaryFile` locates the first pinyin index whose key is
  `>=` the prefix via lower-bound binary search over the sorted pinyin index,
  expands the contiguous matching pinyins linearly, and sorts
  frequency-descending. The v2 file format is unchanged.
- **ordering:** `merge_candidate_groups` concatenates group 2 then group 1
  and de-duplicates by candidate text, keeping the group-2 entry. The ime
  engine ranks each group separately with `StaticRankingModel` (previous-word
  + user-dictionary context) and only then merges, so model weights cannot
  reorder the group boundary.
- **interaction invariants:** digit selection commits the candidate and
  clears the whole composition, so a residual tail is dropped with it (`nih`
  selecting 你 commits 你 and discards h); Enter still commits the raw pinyin;
  Backspace recomputes both groups; the merged list is page-sliced 9 items
  per page linearly across groups; ordering is deterministic.
- **CLI boundary:** the `rank` subcommand intentionally keeps the
  complete-segmentation semantics; prefix behavior is covered by engine unit
  tests and host-e2e assertions.

## Alternatives considered

**Suggest only dictionary completions for the tail letters, without group 1.**
Rejected: it drops the last-complete-syllable candidates the user already
typed (`nih` should offer 你 as soon as `ni` was typed) and makes Backspace
feedback inconsistent.

**Reuse `generate_candidates` by treating the tail as another syllable.**
Rejected: whole-word-first and segment-combination semantics would have to be
distorted, and there is no natural place for the prefix-completion ordering.

**Run the merged list through the global ranker once.**
Rejected: context weights would interleave group-1 and group-2 entries too
aggressively; group 2 must stay in front by design.

**Rank group 2 without a cap.**
Rejected: short prefixes like `n` can match thousands of entries; the cap
(engine: 32) bounds query and ranking work while still covering many 9-item
pages.

**Emit candidates whenever any prefix matches, including inputs with no
complete syllable (`z`, `zh`).**
Rejected: short consonant-only prefixes would flood the window; both groups
must be empty for such inputs.

**Status quo (no candidates until the input fully segments).**
Rejected: that is the complaint that started T-029.

## Consequences

Typing now gives live candidates — `nih` → 你好/尼好 (group 2, frequency
order) then 你 (group 1) — with deterministic ordering. `Dictionary` is a
crate contract, so future implementors (e.g. a sqlite-backed dictionary) must
implement `lookup_prefix`; the v2 file format is untouched and the new query
path is a binary search plus a contiguous scan bounded by the completion cap.

Verification: unit tests cover generation (group priority, empty cases,
cap truncation, determinism) and merging (dedup keeps group 2), in-memory and
file-backed prefix queries, and ime behavior (residual drop on selection,
Backspace recompute, no candidates for `z`/`zh`); host-e2e gained prefix
assertions. The VM drill on v0.1.1 with the seed dictionary passed every
scene: `nih` shows 3 candidates with the window visible, `zh` shows no
candidate words, Backspace recomputes to `ni`, digit 3 commits 你 and discards
the residual h, and the saved file bytes (GBK `C4 E3 BA C3 C4 E3 C4 E3`) spell
你好你你. Screenshots (shots6) are archived for human review. A follow-up
drill on the real dictionary (30 MB, v2) re-confirmed parity on v0.1.1:
`zhidao` lists 7 candidates with 知道 first, `nih` shows the prefix groups
(你好 first) plus the completed 你, and `wo` commits 我; screenshots (shots7)
archived. The real-dictionary smoke (`host-e2e --real-smoke`) gained
`zhidao`-first and prefix-parity assertions so seed-vs-real dictionary
regressions cannot hide behind the tiny demo wordlist. Since T-031 the
candidate window stays visible while the composition is non-empty even when
both groups are empty, showing only the header bar (see
2026-09-25-candidate-window-light-default-and-empty-panel.md).
