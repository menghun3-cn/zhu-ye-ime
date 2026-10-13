# Agent Note: Candidate selection pill flush with panel borders (T-043)

Status: implemented

[中文](2026-09-28-candidate-window-highlight-flush-borders.zh.md) | English

## Problem

User feedback: the selected-candidate highlight pill has too little horizontal
padding — its outer edges sit too far from the candidate panel's left/right
borders, while the number index on the left is jammed against the pill's left
edge. The result looks cramped and disproportionate.

The old implementation drew the highlight directly on `row_rect` (inset by
`padding_x` = 12dp on both sides): at 96dpi the pill sat 12/13px from the
panel borders, while the index column started exactly at the pill's left edge
and the digit glyph was only about 2px from it.

## Decision

The highlight is now a **full-width row rect flush with the panel borders**
(Sogou-style whole-row selection block), while the row content layout is
unchanged:

- `CandidateMetrics` gains `highlight_inset_x` = 1 (fixed 1 physical pixel,
  **not DPI-scaled**: the panel border is a 1px GDI pen stroke at any DPI, so
  a 1px inset makes the pill flush while keeping the border line visible).
- New `highlight_rect(index)`: top/bottom match `row_rect`; left/right span
  from `highlight_inset_x` to `panel_width - highlight_inset_x`.
- `paint()` draws the selection pill on `highlight_rect`; the index, primary
  text and translation still use `row_rect` layout (`padding_x` = 12dp
  unchanged), so the index keeps roughly 12dp of inner padding from the
  pill's left edge.

*(T-137 revised the index placement: `CandidateMetrics::marker_left` = 2dp
moves the marker column flush to the panel's left edge — the index now sits
just 1px right of the pill's left edge instead of ~12dp — per the user's
"序号离左边缘 2px即可". The pill-flush decision itself is unchanged. See the
[typography-whitespace note](../../implemented/feature/2026-10-08-candidate-window-typography-whitespace.md)
"Index column flush (T-137)" section.)*

Measured result (96dpi): the pill's outer edges move from 12/13px to 1/2px
from the panel borders; the digit's distance from the pill's left edge grows
from 2px to 13px. Re-verified at 192dpi in the dark theme (1/2px), confirming
the inset is a physical pixel rather than a DPI-scaled value.

## Alternatives considered

**Zero inset (pill exactly flush, covering the border line).** Rejected: the
pill would overwrite the outermost 1px blue border, making the border appear
broken along the selected row; a 1px inset looks identical while preserving
the border.

**Keep the 12dp inset and shift only the index column right.** Rejected: this
fixes only the "index too close" half — the pill would remain too far from
the panel borders — and it would undo the T-037 narrowing that keeps the
candidate word close to its index.

**Shrink the pill into a content-hugging capsule (width follows text).**
Rejected: inconsistent with the Sogou-style whole-row selection block, and a
short candidate would leave the pill even farther from the panel borders,
opposite to the request.

## Consequences

The selection pill is now a whole-row block flush with the panel's left/right
borders, the index no longer touches its left edge, and the border line stays
intact. The index column width, primary/translation split (T-037) and footer
(T-040) are unaffected. Unit tests lock the geometry invariants (flush
left/right, row-height top/bottom, index-column start = `marker_left` (2dp,
T-137) — 1px right of the pill's left edge, `highlight_inset_x` not
DPI-scaled). Cross-references:
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
(row layout metrics) and
[2026-09-25-candidate-window-page-footer-indicator.md](2026-09-25-candidate-window-page-footer-indicator.md)
(panel height metrics).
