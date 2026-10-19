# Agent Note: Transposed-input corrections come first and are displayed

Status: implemented

[中文](2026-10-19-transposed-correction-first.zh.md) | English

## Problem

Typing fast often swaps two adjacent letters (`zhegnque` instead of
`zhengque`). The user expects the IME to recognize the correction and show
the corrected candidate first ("输入zhegn'que也默认校正优先预判显示候选词
zheng'que，这种通常是用户输入太快导致的，应该可以智能识别该输入"). The
transposition tolerance existed (T-115) but was invisible in practice:

1. The correction group was gated on "the main candidate list is shorter than
   one page". With the real dictionary, `zhegnque` first matches the prefix
   completion path (`zhe` → 这/者/遮…), which fills the page, so the gate
   skipped the whole correction group — the user saw no 「正确」 at all.
2. Even when the group was produced, it was **appended** after the main
   candidates, so the corrected word fell outside the first page.
3. Inside `transposed_candidates` the swap variants were enumerated in
   position order and the candidate cap was consumed by the earlier,
   worthless variants: `zhegnque` produced 「则/咋/责/泽…」 (variants that
   cannot be segmented) and never reached `zhengque`, the variant that can.

## Decision

Three coordinated changes make the corrected candidate visible and first:

1. **`zhu-ye-core` variant ordering** (`candidate.rs`): the swap variants are
   split into *segmentable* variants (the real correction targets, e.g.
   `zhengque`) and *non-segmentable* ones (only useful for prefix
   completion). Segmentable variants are enumerated first, so the cap cannot
   be consumed by noise before the meaningful correction.
2. **Trigger condition** (`input.rs` `append_transposed_group`): the
   correction group is produced whenever the input **cannot be fully
   segmented** (`segment_all` empty) — the page-length gate is gone. The
   group is then **prepended** (`prepend_group`), so the corrected word is
   the first candidate, matching Sogou.
3. **Corrected display** (`input.rs` `composing_display`): when the first
   candidate is a `CandidateSource::Corrected` candidate whose pinyin is a
   **rearrangement of the current input** (same length, same character
   multiset) and can be segmented, the composition header shows the
   corrected pinyin with syllable separators (`zhegnque` → `zheng’que`).
   The rearrangement check keeps a stale candidate from hijacking the
   header after a backspace (`nihao` → `niha` must not display `ni’hao`).

## Alternatives considered

- **Keep the page-length gate and only reorder variants**: the prefix path
  fills the page before the gate is evaluated, so the group is still never
  produced for exactly the inputs that need it. The gate measured the wrong
  thing — the input's segmentability, not the page fill.
- **Append the group but make the corrected candidate selectable via a
  shortcut**: keeps the corrected word off-screen; Sogou shows it first.
- **Show the corrected pinyin for every corrected candidate, without the
  rearrangement check**: after a backspace the stale candidate's pinyin no
  longer corresponds to the input, and the header displayed a word the user
  never typed. Guarded by the multiset check instead.
- **Generate corrections only for inputs without a prefix match**: a user
  typing `zhegnque` legitimately wants the correction, even though `zhe`
  prefixes exist. Rejected.

## Consequences

- `zhegnque` (and `zhegn'que`, whose composing key is the de-noised
  `zhegnque`) now lists 「正确」 as the first candidate and shows `zheng’que`
  in the header.
- The correction group is generated for every non-segmentable input, so
  candidate lists for typo-like inputs start with the correction instead of
  prefix completions. Fully segmentable inputs are unaffected.
- The header text changes for corrected inputs; the underlying composing key
  stays the de-noised pinyin, so learning and backspace keep working on the
  typed letters.
- `transposed_candidates` now prefers segmentable variants, which changes
  the order of the correction group for existing typo inputs (the corrected
  word first instead of the first enumerated swap).

## Verification

- Unit test `错位串校正词置顶且页眉显示校正形式`: first candidate is 「正确」
  with `CandidateSource::Corrected`; `composing_display()` is `zheng’que`.
- Unit test `带分隔符错位串zhegnque出正确`: the separated input still
  produces the 「正确」 correction (`composing` is the de-noised key
  `zhegnque`).
- Real-dictionary end-to-end test `真实词典下错位串zhegnque出正确并显示校正形式`
  (`#[ignore]`, needs the deployed dictionary at
  `C:\Program Files\zhu-ye-ime\tsf\dictionary.zyct`): first candidate 「正确」,
  header `zheng’que`.
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace`, `git diff --check`.
