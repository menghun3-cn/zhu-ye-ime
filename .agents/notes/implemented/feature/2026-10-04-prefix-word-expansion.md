# Agent Note: Prefix word expansion for sparse whole-word hits (T-090)

Status: implemented

[中文](2026-10-04-prefix-word-expansion.zh.md) | English

## Problem

FR-059 (M14, follow-up of D-06) addresses the fact that the candidate window
is nearly empty for complete pinyin that have few dictionary entries. T-060
measured that 80 of 410 syllables produce fewer than one page (<9 whole-word
hits; `lue`/`nue`/`cei` have zero, 43 have 1–4, 34 have 5–8). The engine's
whole-word-first matching (`generate_candidates` on the complete pinyin key)
leaves these syllables with a sparse first screen — e.g. `shui` shows only
说/谁/睡/水 — while the dictionary already holds deeper words starting with
the same pinyin (水稻/水果/水平/睡觉…). The decision D-70, D-71 locked the
scope: expand only when the candidate count is below one page, and never touch
the complete-pinyin eval path.

## Decision

A separate **prefix word expansion group** is appended after the whole-word
hit group, only when `direct_hit && main.len() < page_size`:

- **core** `Dictionary::lookup_prefix("shui")` infrastructure (used by the
  FR-023 prefix-completion path and the composite dictionary) is reused.
  `prefix_expand_candidates(dictionary, pinyin, fill, already)`:
  1. `fill == 0 || pinyin.is_empty()` → empty;
  2. maps `lookup_prefix(pinyin)` through `candidate_from_entry` with
     `with_source(CandidateSource::PrefixExpand)`;
  3. de-duplicates against `already` (the whole-word group) and internally by
     candidate text;
  4. sorts score-descending / text-ascending and truncates to `fill`.
- **engine** (`refresh_candidates`): after the whole-word group `main` has
  been ranked, `fill = page_size - main.len()`; the expansion is ranked with
  the same ranking context but as an independent group and merged with the
  existing `append_group` (main-group-first text de-dup therefore applies),
  so model weights cannot interleave expansion entries into the main group.
- **position chain** (pinned): whole-word hits → **prefix expansion** →
  domain boost (D-13) / contacts (D-21) → emoji tail. The expansion group does
  not participate in domain boosting (D-15 remains whole-word only) and has no
  new UI label (it renders like Static candidates).
- **interactions pinned by tests**: incomplete pinyin (`shuip`) keeps the
  FR-023 prefix-completion path exclusively (no expansion entries); common
  pinyin with ≥9 whole-word hits expand zero entries (T-050 baseline stays
  byte-identical); injecting a smaller `page_size` rescales `fill`; the merged
  list never repeats a candidate text.

## Alternatives considered

**Status quo (no expansion).** Rejected: 80 syllables keep a sparse first
screen; that is the exact complaint that motivated FR-059.

**Expand only into a second page instead of filling the first.** Rejected:
the D-70 decision is to fill the first page up to 9 items; page-2 filling
changes pagination semantics for no user-visible gain.

**Change complete-pinyin matching (option B, dictionary-level).** Rejected as
dangerous and out of scope: it would perturb the evaluated whole-word path.

**Insert expansion entries into the main group by final score.** Rejected
(D-71): an independent group keeps the whole-word group byte-identical (T-050)
and makes eval regression trivially provable.

## Consequences

- 80 low-frequency syllables now reach a full first page; common pinyin are
  untouched (verification: T-050 host-e2e m7–m13 suites, plus the new `--m14`
  group, all pass and `de` with 10 whole-word hits shows zero expansion).
- **Eval zero-regression (strongest evidence):** re-run against the T-057
  baseline on the real dictionary is identical on every metric — Top1 84.7% /
  Top3 97.2% / 整句 21.0% — and the MISS list (385 lines) is line-for-line
  identical to a baseline re-run (stashed working tree),
  `data/eval/miss-base.tsv` vs the expansion run.
- **Domain-pack word interception:** a domain-pack word whose pinyin is the
  typed syllable (e.g. 谁 `shui` in the `it` pack) is fetched by the composite
  `lookup_prefix` first and shown once as `PrefixExpand`; the later domain
  boost `append_group` de-duplicates it away. The display is equivalent to the
  D-13 position (after the base candidates), so this is a no-op for users and
  pinned by the `领域包词被展开截获不重复` engine test.
- Expansion words are not user-dictionary-learned by ranking: learning happens
  only at commit time, and `StaticRankingModel::rank` only re-labels a
  candidate `User` when the word already has user frequency (semantics of
  pre-existing user words are preserved).
- One pre-existing engine test assertion changed because it exercised a sparse
  syllable by design: `backspace从残缺回到完整音节重算候选` now sees
  `ni → 你/你好/尼好` (expansion of the `ni` prefix) instead of only 你.
- `CandidateSource::PrefixExpand` is a new public enum variant; no UI label
  change (renders like Static). Candidate totals stay capped at one page in
  the common case (fill ≤ `page_size − main.len()`), so pagination length does
  not grow.

Verification (2026-10-04): core +5 unit tests (fill truncation, de-dup against
`already` and internally, empty input/fill, determinism, source labeling);
ime +5 integration tests (fill-to-page, no-expansion-when-full, `page_size`
injection, incomplete-pinyin exclusivity, domain interception dedup);
host-e2e `--m14` 6/6 (memory word list + real dictionary mechanism
consistency: `shui` main group 8 → expansion 1); workspace suites all green
(core lib 274, properties 8, ime 175, others unchanged); fmt/clippy
`-D warnings`/`git diff --check` clean; eval as above.

Related notes: [prefix candidates (FR-023,
2026-09-24-prefix-candidates-incomplete-segmentation.md)](../../implemented/feature/2026-09-24-prefix-candidates-incomplete-segmentation.md)
— the incomplete-pinyin path that stays exclusive and provides the
`lookup_prefix` contract; [composite dictionary
(2026-09-29-composite-dictionary-and-abbreviation-path.md)](../../implemented/architecture/2026-09-29-composite-dictionary-and-abbreviation-path.md)
— the dictionary layering through which domain words surface in the expansion;
[candidate ranking static model
(2026-09-19-candidate-ranking-static-model.md)](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
— the ranking semantics the expansion group reuses.
