# Agent Note: Candidate window m/n page-footer indicator for T-040

Status: implemented

[中文](2026-09-25-candidate-window-page-footer-indicator.zh.md) | English

## Problem

User request (T-040 part 2): when paging beyond the first page, the candidate
window should show an indicator like `m/n` in a footer at the bottom of the
panel. A follow-up check confirmed that arrow/number selection already works
(see 2026-09-25-candidate-window-arrow-selection.md); the genuinely new work
is the footer.

## Decision

Add a fixed-height footer strip to the metrics and render a right-aligned,
gray `m/n` label when there is more than one page.

- `CandidateMetrics.footer_height` = 20dp (proportional at every DPI, like all
  other metrics). `panel_size(rows)` adds the footer only when `rows > 0`, so
  a zero-candidate panel stays a header-only strip (T-031 behavior preserved).
- New pure function `page_footer_label(page, page_count) -> Option<String>`:
  returns `None` when `page_count <= 1`; otherwise `m/n` in 1-based current
  page / total pages, clamping an out-of-range page to the last page.
- `CandidateUiView` carries `page_count` (filled by the engine in both the
  Chinese and English-mode branches and by the demo binary); the view exposes
  `footer_label()`.
- `paint()` draws the label with a new right-aligned helper
  `draw_text_right` (DT_RIGHT) in the secondary color inside
  `footer_rect(rows)`, which sits between the last row and the bottom padding
  so it never overlaps rows.
- Number-key selection: unchanged code path (engine `select_index` +
  `KeyAction::Select`), re-verified on the VM instead of re-implemented.

## Alternatives considered

**Only show the footer while a next page exists.** Rejected: `m/n` is
conventional (微软拼音/Sogou both show it continuously when paging is
possible) and a disappearing position would make the panel jump; `m/n` is
always shown when `page_count > 1`, including on the last page.

**Show `页 1/3` or arrows instead of `m/n`.** Rejected: the user explicitly
asked for something like `m/n`; keep it minimal and unambiguous.

**Overlay the footer on the last row.** Rejected: a dedicated footer strip is
cleaner and costs 20dp; row geometry tests assert no overlap.

## Consequences

Multi-page candidate lists now communicate paging state at the bottom-right;
single-page lists and the no-candidate header strip render exactly as before.
Covered by unit tests (label format/gating/clamping, footer rect vs rows no
overlap, updated panel-size regression) and by VM v0.1.9 forensics (footer
gray text at x≈535-563 present on both pages of `shi`, absent for single-page
`nihao`). Cross-references:
[2026-09-25-page-keys-minus-plus.md](2026-09-25-page-keys-minus-plus.md)
(paging keys),
[2026-09-25-candidate-window-arrow-selection.md](2026-09-25-candidate-window-arrow-selection.md)
(arrow selection), and
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
(row layout metrics).
