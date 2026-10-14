# Agent Note: Candidate window placement avoids covering the host input line

Status: implemented

[中文](2026-10-15-candidate-placement-avoid-overlap.zh.md) | English

## Problem

The candidate window must never cover the host input line. The previous
`place_at` strategy always preferred the space below the anchor
(`anchor.y + gap`): when the input line sat near the bottom of the monitor
work area the window could not fit below, so it was pushed back up
(`top = work.bottom - height`), landing directly on top of the input line
and hiding what the user was typing. The user requested a three-tier strategy
instead: prefer below, fall back to the right of the input line, and only
then above. During real-host acceptance the right tier was tightened: when
the space below is insufficient, the panel must also sit entirely above the
input line's text (its bottom edge must not cross the text's top edge) —
i.e. at the caret's upper-right corner.

## Decision

Placement is decided by a pure function `place_in_work_area(work, anchor_x,
anchor_y, anchor_top, anchor_right, width, height, gap)` in
`candidate_window.rs`:

1. **Below** — top = `anchor.y + gap`; used whenever it fits vertically in the
   work area (horizontal is still clamped into the work area).
2. **Upper-right corner** (user-acceptance tightening: when below does not fit
   the panel lands at the caret's upper-right) — left = `anchor_right + gap`
   and the panel sits entirely above the input line's text:
   top = `anchor_top − gap − height` (bottom edge = text top − gap, so the
   panel never overlaps the text horizontally or vertically). Taken only when
   both the horizontal and the vertical constraints fit.
3. **Above** — the same vertical constraint (entirely above the text), with
   the horizontal aligned to the input line's left edge and clamped into the
   work area; used when there is no room for right-side placement.
4. **Fallback** — clamp into the work area (bottom-aligned) so the window
   stays visible, for the extreme case where the panel is taller than the
   work area.

`CandidateWindowPlacement` carries the input line's bottom-left (`anchor`),
top (`anchor_top`), and right edge (`anchor_right`) in screen pixels, filled
by `selection_placement` from the existing `ITfContextView::GetTextExt` rect.
This extends the placement mechanism described in the [TSF integration
note](../../implemented/feature/2026-09-20-candidate-window-tsf-integration.md):
anchoring to the caret still comes from `GetTextExt`; only the expansion
strategy is new. The gap constant is `PLACEMENT_GAP = 4` (unchanged from the
previous behavior).

## Alternatives considered

- **Always place above.** A common IME default, but the user explicitly wants
  below-first and only accepts above as a last resort; flipping the default
  would change the familiar position for every normal typing session.
  Rejected.
- **Keep pushing the window up into the work area (status quo).** That is
  exactly the behavior that hides the input line; rejected as the complaint's
  root cause.
- **Place to the left instead of the right.** A mirrored fallback; the user
  explicitly named the right side, and left placement would feel disconnected
  from reading order. Rejected.
- **Shrink or reflow the panel.** Changes the polished T-122/T-124 layout and
  still cannot solve a work area smaller than the panel. Rejected.
- **Move the panel to another monitor.** Invasive to host desktops; rejected.
  The panel stays clamped to the nearest monitor's work area.

## Consequences

- Near the bottom of the screen the window appears to the right of the input
  line and entirely above the input line's text (bottom edge at text top −
  gap) — the caret's upper-right corner — so it covers neither the text nor
  the space below the line.
- At the screen's bottom-right corner it moves above the line, left-aligned
  with the line and clamped into the work area.
- Only when the panel is taller than the whole work area does the fallback
  clamp it in place; the panel will then cover the line, which is unavoidable
  at that size.
- `anchor_top`/`anchor_right` come from an API already used for anchoring
  (`GetTextExt`), so there is no new COM surface; the engine and keyboard
  routing are untouched, keeping the change purely presentational.

## Verification

- 6 unit tests on `place_in_work_area`: below-first; lower-right overspill →
  upper-right corner with the bottom edge not crossing the text top; the
  bottom-right corner → above (same vertical constraint, left-aligned);
  fallback clamp when all three tiers fail; upper-right tier rejected when
  the vertical space above is insufficient (never wedges into the text band);
  horizontal clamping into the work area.
- `cargo test -p zhu-ye-ime`: lib 244 (candidate_window 13), tsf 44,
  host_e2e 167 — all green. `cargo fmt --check` and
  `cargo clippy --all-targets -- -D warnings` clean.
- Acceptance row under FR-009 (需求规格说明书, 验收标准), design section
  §5.1, CHANGELOG [Unreleased] entry, and todos T-162 updated to the
  upper-right-corner semantics.
- First deploy to the user machine via the elevated worker `copy-dll-ver`
  (2026-10-15): DLL `zhu_ye_ime_878D26F8.dll` (2,244,096 B) landed, CLSID
  InprocServer32 + LanguageProfile IconFile pointers switched, ctfmon
  restarted; `verify-tsf-dll` (3/3 exports) and `verify-tsf-identity` (7/7)
  passed. A second deployment carrying the upper-right-corner tier follows the
  same channel; new host processes (e.g. a fresh Notepad window) load the new
  DLL. The user's real-host visual acceptance stays pending.
