# Agent Note: Candidate-window header content-driven width for long pinyin

Status: implemented

## Problem

Typing a long pinyin syllable string such as `youmeiyoushenmeren` truncated the
candidate window's header composition line to `youmeiyoushenmere...` (right-side
ellipsis). Root cause: `CandidateMetrics::panel_width` is a fixed 360dp.
`header_hint_rect` (T-031) reserved a fixed 5/9 of the header for the tonal hint,
leaving only ≈141dp for the composition text (360 − 2×12 padding − 5/9 hint − gap).
17 ASCII pinyin characters at the 16dp header font measure ≈150px with
`estimate_text_width` (0.55×em per ASCII char), so the composition overflowed its
rect and `draw_text`'s `DT_END_ELLIPSIS` clipped it. The candidate-word pinyin line
(候选词上方拼音行) was unaffected — it already draws across the full row width.

## Decision

The header (and the whole panel) becomes content-width driven instead of fixed:

- `CandidateMetrics::header_widths(composition, hint)` returns `(main_w, hint_alloc)`
  using `estimate_text_width`. When the estimated total still fits the fixed 360dp
  panel, nothing changes: the panel stays 360dp and the hint gets its full estimated
  width. When the total exceeds the fixed width, the window expands to
  `max(360dp, main_w + gap + hint_alloc + 2×padding)` and the hint is capped at 2/3 of
  the combined estimated width so the composition always keeps at least ≈1/3 — the
  user's typing is never ellipsized; only the auxiliary hint may clip on extreme
  inputs.
- `panel_size_for(rows, composition, hint)` extends `panel_size` with that width.
  All three window-sizing call sites (create, demo, per-update resize) use it.
- `CandidateMetrics::with_panel_width(w)` returns a copy of the metrics laid out at an
  explicit panel width; `paint` builds `metrics = self.metrics.with_panel_width(width)`
  each frame so every rect (header text/hint, rows, footer) follows the real client
  width rather than the fixed 360dp base.
- Header painting uses `header_text_rect_with(hint_alloc)` /
  `header_hint_rect_with(hint_alloc)`; when the hint is empty or equals the
  composition, `hint_alloc` is 0 and the composition takes the full header row.
- `candidate-demo` gains a `--long` flag that renders the reported scene
  (composition `youmeiyoushenmeren`, hint `yǒu méi yǒu shén me rén`) for pixel
  verification.

## Alternatives considered

- **Widen the fixed panel globally (e.g. 480dp)**: wastefully wider for short
  inputs, and any fixed width would still truncate sufficiently long input; rejected
  because the problem is space allocation, not the base width.
- **Shrink the hint font / abbreviate the hint**: degrades readability of the tonal
  hint (just polished in T-122) and does not address the composition's own lack of
  room; rejected.
- **Shrink the composition font**: misplaces priority — the user's typed string is
  the primary reading; allocating space (wider window) beats compressing type; rejected.
- **Make the per-candidate pinyin line wrap or grow**: the reported truncation is on
  the header composition line, not the per-candidate pinyin line (which already spans
  the full row width); no change needed there.

## Consequences

- Long pinyin + tonal hint now widens the panel from 360dp to ≈421dp (measured
  421dp at 96dpi baseline); the composition renders completely, no ellipsis.
- Short inputs are pixel-identical to before (360dp, hint full width).
- Extreme inputs (hint far longer than the composition) clip the *hint*, which is
  acceptable auxiliary information; `place_window` still clamps x within the
  work-area, so an absurdly wide panel (>screen) remains a physical impossibility
  rather than a crash.

## Verification

- Unit tests in `candidate_ui.rs`: long composition + hint expands the panel and
  keeps the composition rect ≥ its estimated width; hint allocation is capped at 2/3
  with the composition rect still complete and non-overlapping; short input keeps the
  360dp width with a full hint.
- `candidate-demo --long` pixel probe (96dpi ×1.25 system scale): window 536px vs
  baseline 450px; header composition blue glyphs form one continuous run x=16..194
  (17 chars, no right side ellipsis; an ellipsized render would stop ≈178 + dots);
  tonal hint grey glyphs x=225..451.
- Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --workspace`, `git diff --check` all pass.
