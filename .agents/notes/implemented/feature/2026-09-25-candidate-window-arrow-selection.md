# Agent Note: Candidate window arrow-key selection for T-039

Status: implemented

[中文](2026-09-25-candidate-window-arrow-selection.zh.md) | English

## Problem

User feedback (T-039): after the first candidate was pre-selected, the up/down
arrow keys could not move the selection to the second, third, and following
candidates. Two root causes: the engine had no per-page selected-index state
at all (`candidate_ui_view` hard-coded `selected: 0`, so every candidate
window opened on row 1), and the TSF key router did not map `VK_UP`/`VK_DOWN`,
so the keys fell through to the host (moving the host caret) instead of
moving the selection.

## Decision

Add a real page-local selection and route the arrow keys to it.

- `InputEngine` gains `selected_on_page` plus `select_up()` / `select_down()`
  (clamped to the page's visible candidates, so movement stops at page
  boundaries). Selection resets to 0 when the input changes
  (`refresh_candidates`) and when composition ends (`clear_composition`);
  page turns and layer switches keep the row position, clamped to the new
  page (`clamp_selected` inside `clamp_page`, also called by
  `next_page`/`previous_page`).
- `CandidateUiView.selected` now comes from the engine's page-local index, so
  the highlight follows the arrows.
- **Space commits the currently selected row** (`handle_space` /
  `preview_space` index into `visible_candidates` by `selected_on_page`
  instead of `.first()`). With the default selection of 0 the previous
  behavior is preserved exactly.
- TSF routing: `VK_UP` → `SelectUp`, `VK_DOWN` → `SelectDown`. The actions
  are pure state transitions (no edit session), handled like paging; they are
  only consumed while a composition is active, and Ctrl/Alt-held keys still
  pass through — so arrows keep their host meaning when no candidate window
  is open.

## Alternatives considered

**Use `ITfCandidateList`/the TSF candidate UI.** Rejected: the project
deliberately renders a custom GDI window; pseudo-candidate integration would
duplicate state for no benefit.

**Move selection across pages (arrow at page end turns the page).** Rejected
for this milestone: it couples two mechanisms and changes paging behavior;
page-local clamping is predictable, and `-`/`=` paging already exists
(2026-09-25-page-keys-minus-plus.md).

**Reset selection to 0 on page turns.** Rejected: 微软拼音/搜狗 keep the row
position when paging; keeping it feels native and costs nothing.

## Consequences

Arrows now move the highlight within the page and Space commits the
highlighted candidate; number keys, paging, and the default first-row
selection are unchanged. Behavior is fully covered by unit tests at the
engine and routing layers. Cross-references:
[2026-09-25-page-keys-minus-plus.md](2026-09-25-page-keys-minus-plus.md)
(paging keys),
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
(row layout), and
[2026-09-18-tsf-composition-and-key-events.md](2026-09-18-tsf-composition-and-key-events.md)
(key routing foundation).
