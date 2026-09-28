# Agent Note: Candidate row layout — compact markers and dynamic translation column for T-037

Status: implemented

[中文](2026-09-25-candidate-window-row-layout-dynamic.zh.md) | English

## Problem

User feedback on the candidate window (T-037): candidates sat too far from
their index numbers, and English translation text was pinned into a fixed
right-hand column (one third of the row), so long English words were
truncated with an ellipsis and the translations looked detached from the
candidates. The fixed geometry wasted horizontal space on both sides: a
40 px marker column around a ~10 px digit, and a right column that could not
grow to fit its content.

## Decision

Compact the markers and make the row a content-driven, two-part split.

- `CandidateMetrics::marker_width` goes from 40 to 26 dp and
  `translation_gap` from 16 to 8 dp — candidates sit closer to their numbers.
- The fixed `text_rect`/`translation_rect` split is replaced by a single
  `CandidateMetrics::row_split(row, main, translation)`:
  - the main text width is estimated with `estimate_text_width`
    (ASCII 0.55×em, CJK 1×em; slightly conservative, so no overlap);
  - with a translation present, the main text is capped so the translation
    area (including the gap) keeps at least one third of the usable width;
    without a translation the main text may fill the whole row;
  - the translation column starts right after the main text (plus
    `translation_gap`) and extends to the row end — left-anchored, roughly
    twice as wide as before, so long English words survive without ellipsis.
- `paint()` in `candidate_window` draws main and secondary text through
  `row_split`; selected/unselected colors are unchanged.

## Alternatives considered

**Keep fixed columns and only shrink the marker.** Rejected: the right-hand
third stayed cramped and detached; the complaint was about placement, not
just width.

**Measure text with `GetTextExtentPoint32W` in the renderer.** Rejected for
this milestone: `estimate_text_width` is deliberately conservative and keeps
`candidate_ui` platform-free and unit-testable; the layout stays the single
authority for row splitting.

**Always reserve one third even when the translation is empty.** Rejected:
rows without a translation would keep a pointless void on the right; the
empty case now lets the main text use the row.

## Consequences

The row API changes: `text_rect`/`translation_rect` are removed and
`row_split` is the only layout entry point; unit tests moved to the new
semantics and cover short main text, over-long main text, and the
no-translation case. Rendering geometry is verified on the VM (v0.1.7): the
gray translation pixels' leftmost x moves from 528 to 333 (≈ −195 px, now
right after the candidate), and the row-main text left edge moves from
271/273 to 255 (≈ −16 px, matching the 14 px marker shrink plus antialias
rounding). See also
[2026-09-25-candidate-window-corner-black-pixels.md](2026-09-25-candidate-window-corner-black-pixels.md)
for the accompanying corner fix and
[2026-09-19-candidate-window-gdi-rendering.md](2026-09-19-candidate-window-gdi-rendering.md)
for the rendering foundation.
