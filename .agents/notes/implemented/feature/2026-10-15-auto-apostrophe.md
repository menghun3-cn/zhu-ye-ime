# Agent Note: Auto-apostrophe syllable boundaries in unpairable long inputs

Status: implemented

[中文](2026-10-15-auto-apostrophe.zh.md) | English

## Problem

Typed sequences that mix shorthand initials with full syllables (e.g.
`wsshui` for 我是谁, where `w` and `s` are initials and `shui` is the full
syllable) cannot be segmented by `segment_all` (it only accepts complete
syllables) and exceed the shorthand path's `INITIAL_MAX_LEN = 4`, so every
fallback path (shorthand, transposition, abbreviation, prefix) misses them
and the engine produces no candidates. The user reported in a real-host
comparison that Sogou types the separator automatically (`w's'shui`) and
offers 我是谁, while Zhu-Ye cannot type them at all, and asked us to match
Sogou.

## Decision

Add an **auto-apostrophe** candidate group (`T-163`, an extension of the
FR-023 shorthand path) in `zhu-ye-core/candidate.rs` plus its engine hook in
`zhu-ye-ime/input.rs`:

1. **Slot partitioning** (`slot_plans`): split the unpairable input into
   slots, each a complete syllable or a single shorthand letter (a letter
   present in `pinyin::INITIAL_SYLLABLE_TABLE`). Segment count 2..=6, plans
   capped at `APOSTROPHE_PLAN_CAP` (32). Enumeration order is deterministic:
   single-letter slots first, then complete syllables longest-first
   (`wsshui` → `[w,s,shui]` before `[w,s,s,hui]`).
2. **Expansion** (`expand_slot_combinations`): expand each single-letter slot
   to its common-syllable list and keep complete syllables as-is, then take
   the prefix Cartesian product into full pinyin strings (capped at
   `APOSTROPHE_COMBINATION_CAP` = 48, truncated keeping the table-order most
   common spellings). `[w,s,shui]` → `woshishui` (wo+shi+shui).
3. **Candidates** (`apostrophe_candidates`): for each combined pinyin string,
   a dictionary whole-word hit yields candidates directly (`pinyin` = the
   expanded string, reused for user-word learning and the pinyin row); if
   there is no hit and the plan had ≥3 syllables, run sentence assembly
   (`sentence_candidates`) so 我是谁 emerges from the single characters
   我/是/谁. Total results capped at `APOSTROPHE_CANDIDATE_CAP` (10).
4. **Gate** (input.rs): this group joins the shorthand tier as a last
   fallback — it only runs when the main candidate list is still empty, the
   input is pure lowercase ASCII, 3..=`APOSTROPHE_MAX_INPUT` (16)
   characters, and cannot be fully segmented; it follows the
   `enable_abbreviation` switch (the shorthand-family switch). Segmentable
   strings and 2-4 letter shorthand keep their existing behavior.

Defensive rules: no complete segmentation and no whole-word hit → empty;
length or character-shape violations → empty. All enumeration is
deterministic; cost is bounded (32 plans × 48 combos, a handful of lookups
per combo, at most a few beam runs), far below the 30 ms refresh budget. The
dictionary format is untouched (O-06).

## Alternatives considered

- **Extend shorthand to longer strings** (raise `INITIAL_MAX_LEN`): would
  explode the Cartesian product (6⁶ ≈ 46k lookups) and dilute the
  two-to-four-letter shorthand semantics; rejected in favor of a separate
  slot model that mixes initials and full syllables.
- **Type-ahead auto-completion of every unpaired prefix**: this is the
  existing prefix path; it cannot bridge an unpairable *head* like `w`+`s`.
- **Only whole-word lookups, no sentence assembly**: `woshishui` is not in
  the shipped dictionary, so 我是谁 would not appear; sentence assembly is
  required and reuses the existing FR-025 beam.
- **Displaying the separator in the composing string** (`w's'shui`): Sogou
  shows it in the input; our first version keeps the raw composing text and
  shows the expanded pinyin on the candidate row instead. Display polish can
  follow in a later iteration.

## Consequences

- `wsshui` now yields 我是谁, matching Sogou/Microsoft Pinyin behavior for
  shorthand+full-syllable mixes, without touching the dictionary.
- The shorthand *family* switch (`enable_abbreviation`) controls the new
  group; normal full-pinyin, whole-word, prefix, and 2-4 letter shorthand
  behavior is unchanged (guarded by "main candidates empty" and the
  segmentable/whole-word early returns).
- Candidate `pinyin` holds the expanded full spelling (user-word learning
  keeps working); the candidate row's pinyin display shows the expanded
  form, not the separator form.

## Verification

- Core unit tests (4): `wsshui` → 我是谁 via sentence assembly; whole-word
  hit priority (`我是谁`/`woshishui` entry wins); no pollution of segmentable
  strings or whole-word-hit inputs; length/character gates (2-letter
  shorthand stays with `initial_candidates`, digits/uppercase/non-ASCII are
  rejected, no candidates without pinyin).
- Engine integration tests (2): typing `wsshui` yields 我是谁; `nihao` and
  `wsm` behavior is unchanged (no takeover).
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace` (core 351, ime lib 246, tsf 44,
  host_e2e 169, candidate_window 13), `git diff --check`; docs pair and
  agent-note gates (verify-translation-pairs 72 pairs, verify-agent-notes)
  clean.
- Docs updated: FR-023 auto-apostrophe bullet, §12.4 decision O-07, design
  §12.2.5, acceptance rows under §8.1 FR-023, CHANGELOG [Unreleased],
  todos T-163 (plus T-164 for the separate Ctrl+Shift cycling investigation).
- Deployment and real-host acceptance of `wsshui` → 我是谁 stay pending.
