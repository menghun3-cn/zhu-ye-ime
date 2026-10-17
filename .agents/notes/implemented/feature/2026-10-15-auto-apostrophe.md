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
   **complete syllables (longest-first) before single-letter slots**
   (`wsshui` → `[w,s,shui]` before `[w,s,s,hui]`). This ordering is the
   T-163 acceptance fix: with single-letter-first, the noisy plan
   `[w,s,s,hui]` (我试试会…) filled the candidate quota before the intended
   `[w,s,shui]`, so 我是谁 never surfaced on the real 440k dictionary.
2. **Expansion** (`expand_slot_combinations`): expand each single-letter slot
   to its common-syllable list and keep complete syllables as-is, then take
   the prefix Cartesian product into full pinyin strings (capped at
   `APOSTROPHE_COMBINATION_CAP` = 48, truncated keeping the table-order most
   common spellings). `[w,s,shui]` → `woshishui` (wo+shi+shui).
3. **Candidates** (`apostrophe_candidates`): for each combined pinyin string,
   a dictionary whole-word hit yields candidates directly (`pinyin` = the
   expanded string, reused for user-word learning and the pinyin row); if
   there is no hit and the plan had ≥3 syllables, first run whole-word
   combination (`generate_candidates` — per-syllable first entries give
   `woshishui` → 我是谁 with a positive score, putting it at the top), then
   sentence assembly (`sentence_candidates`) to fill in non-first
   combinations (same-text dedup against the combined group). Total results
   capped at `APOSTROPHE_CANDIDATE_CAP` (10).
4. **Display** (`apostrophe_boundaries` + input.rs `apostrophe_seps`): the
   header composing string shows the separator form `w's'shui` (U+2019,
   same display convention as T-128). `refresh_candidates` records the
   preferred-slot boundaries **only when the auto-apostrophe group actually
   produced candidates**; every other refresh round clears them, so the
   T-128 "longest segmentable prefix" degraded display (`xianjingd` →
   `xian'jingd`) is untouched.
5. **Gate** (input.rs): this group joins the shorthand tier as a last
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
- **Show the separator unconditionally for every unpairable string**: would
  corrupt the T-128 "longest segmentable prefix" display (`xianjingd`);
  rejected — the separator shows only when auto-apostrophe candidates hit.

## Consequences

- `wsshui` now yields 我是谁 as the **first** candidate, matching
  Sogou/Microsoft Pinyin behavior for shorthand+full-syllable mixes, without
  touching the dictionary (verified against the real 440k dictionary: first
  hit 我是谁, score 19940).
- The shorthand *family* switch (`enable_abbreviation`) controls the new
  group; normal full-pinyin, whole-word, prefix, and 2-4 letter shorthand
  behavior is unchanged (guarded by "main candidates empty" and the
  segmentable/whole-word early returns).
- Candidate `pinyin` holds the expanded full spelling (user-word learning
  keeps working); the candidate row's pinyin display shows the expanded
  form; the composing-string header shows the separator form when the group
  hits (U+2019, T-128 convention).
- The header separator is gated on candidate hits: strings that never reach
  the auto-apostrophe path (e.g. prefix/complete-able mixes like `xianjingd`)
  keep the T-128 display exactly as before.

## Verification

- Core unit tests (5): `wsshui` → 我是谁 as the first candidate with the
  preferred plan `[w,s,shui]` guaranteed to expand before the noisy plan
  (`woshishui` prefix present — the ordering regression lock, using a
  pressure dictionary with many competing shi/hui homophones); whole-word
  hit priority (`我是谁`/`woshishui` entry wins); no pollution of segmentable
  strings or whole-word-hit inputs; length/character gates; `apostrophe_boundaries`
  display boundaries (`wsshui` → `[1,2]`, empty for segmentable strings).
- Engine integration tests (3): typing `wsshui` yields 我是谁; `nihao` and
  `wsm` behavior is unchanged (no takeover); `composing_display()` shows
  `w's'shui` (U+2019) while `composing()` stays `wsshui`.
- Real-dictionary verification (temporary probe against the shipped
  440k-entry `dictionary.zyct`): `apostrophe_candidates("wsshui")` first hit
  is 我是谁 (pinyin `woshishui`, score 19940). The probe was removed after
  use.
- T-128 display regression suite (4 tests: 自动分隔符按首选切分显示,
  尾部残段退化, 退格后实时重算, 无手动分隔约束组不介入) all green — the
  separator is gated on auto-apostrophe candidate hits.
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace` (core 352, ime lib 248, tsf 45,
  host_e2e 169, candidate_window 13), `git diff --check`; docs pair and
  agent-note gates (verify-translation-pairs 73 pairs, verify-agent-notes)
  clean.
- Docs updated: FR-023 auto-apostrophe bullet (top-hit + display), §12.4
  decision O-07, design §12.2.5, acceptance rows under §8.1 FR-023,
  CHANGELOG [Unreleased], todos T-163 (plus T-164 Ctrl+Shift cycling and
  T-165 Ctrl+Space toggle).
- Deployed via the elevated worker `copy-dll-ver` (2026-10-16, first round):
  DLL `zhu_ye_ime_74C68A5A.dll` (2,255,360 B) landed with CLSID
  InprocServer32 + LanguageProfile IconFile pointers switched and ctfmon
  restarted (PID 40784); `verify-tsf-dll` (3/3 exports) and
  `verify-tsf-identity` (7/7) passed on the release binary. User acceptance
  reported 我是谁 missing and no `w's'shui` separator; the ordering/display
  fixes above were then made. Second-round deployment (with the T-165
  Ctrl+Space toggle, PR #207 → develop e4afddb, 2026-10-16): DLL
  `zhu_ye_ime_3079725B.dll` (2,260,480 B) landed, pointers switched, ctfmon
  restarted (PID 36032), verify-tsf-dll/identity passed. Real-host retest
  pending (new host-process window).
