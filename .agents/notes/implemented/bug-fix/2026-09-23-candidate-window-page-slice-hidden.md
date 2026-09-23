# Agent Note: Candidate window hid after paging due to double page slicing

Status: implemented

[中文](2026-09-23-candidate-window-page-slice-hidden.zh.md) | English

## Problem

VM acceptance of T-013 found the candidate popup disappearing the moment the user
paged: typing `yi` showed page 1 (9 items), pressing `.` hid the window
(`tsf-debug.log` logged `cand-hide (no items)`), yet pressing `1` still committed
the first candidate of page 2 (`依`), so the engine had paged correctly while the
window went blank. `CandidateUiView::visible_items()` slices `items` by
`page_size` again — `start = page * page_size` — but `InputEngine::candidate_ui_view`
filled `items` with the engine's **current page** candidates
(`visible_candidates()`), not the full candidate list. After paging, `page >= 1`
made the second slice start `>= items.len()`, yielding an empty visible range:
the refresh logic correctly concluded "no items" and hid the window.

## Decision

`InputEngine::candidate_ui_view` fills `items` from
`current_layer_candidates()` — the whole candidate list of the active layer
(Chinese candidates, or translation-filtered candidates in the translation
layer) — and lets `CandidateUiView::visible_items()` do the page slicing, which
is exactly what its `items` field documents ("全部候选；窗口只展示当前页").
The English-mode early return still produces an empty `items` vector so the
popup stays hidden while typing English.

## Alternatives considered

**Make `CandidateUiView::visible_items()` tolerant of an over-long page index.**
Rejected: clamping or returning an empty slice would mask, not fix, the contract
violation; the view would continue to receive a page-sized list and could never
render pages beyond the first.

**Have the TSF adapter pass the engine page slice directly to the window,**
**bypassing `visible_items()`.**
Rejected: two slicing paths would have to stay in sync manually; keeping one
authoritative slice in `CandidateUiView` preserves the existing rendering
contract and unit tests.

## Consequences

Paging now keeps the popup visible with the correct page content. A regression
test (`翻页后视图快照可见项跟随当前页`) asserts that
`candidate_ui_view().visible_items()` follows the engine's current page after
`next_page`; it fails against the pre-fix code. VM re-run of the paging stage
shows `candidate-window visible=True` after `.`, and the log records
`cand-show items=9 first=依` before `Select(0)` commits it. T-013 closes with
the remaining VM acceptance items in the todos list.
