# Agent Note: Candidate window light-default theme and always-visible panel

Status: implemented

[中文](2026-09-25-candidate-window-light-default-and-empty-panel.zh.md) | English

## Problem

Two user reports after the v0.1.1 VM drill:

1. **Black background.** The VM runs Windows dark mode
   (`AppsUseLightTheme = 0`), and the candidate window’s `Auto` theme followed
   the system, rendering the dark palette (`#202020` background). The user
   expects the Sogou-classic light look by default, so "black instead of
   white" reads as a requirement violation even though dark-follow-system had
   been the approved T-028 behavior.
2. **Missing window.** When the composition cannot produce candidates (e.g.
   `z`, or a word missing from the active dictionary), the window was hidden
   entirely (`visible_items().is_empty() → hide`). The user expects the
   candidate window to stay visible while the composition is non-empty, even
   with zero candidate rows.

## Decision

- **T-030 (default light):** `ThemePreference::Auto` no longer reads
  `AppsUseLightTheme`. It resolves to `UiThemeKind::Light` unless system high
  contrast is on, in which case it still resolves to `HighContrast` (system
  colors). `UiThemeKind::Dark` remains reachable only through an explicit
  `ThemePreference::Dark` (the `candidate-demo --theme dark` path and any
  future user-visible setting). The `apps_use_light_theme` helper and its
  Registry imports were deleted.
- **T-031 (always-visible panel):** the show/hide contract moves from
  "visible items" to "composition". `candidate_window::update` hides the
  window only when `view.composition.is_empty()`; otherwise it creates (as
  needed) and shows the window, including when `visible_items()` is empty.
  An empty panel renders as a header bar: background, rounded border, and the
  composition string plus pinyin hint, with zero candidate rows.
  `CandidateMetrics::panel_size(0)` is now a real zero-row layout
  (header + vertical padding, 58 px at 96 DPI) instead of clamping to one
  row; the arithmetic was rewritten so `rows = 0` cannot underflow or produce
  negative heights. TSF’s `refresh_candidate_window` logs
  `cand-hide (no composition)` on commit/cancel and `cand-show items=N`
  (N may be 0) otherwise.
- The engine and candidate generation are untouched: `z`/`zh` still produce
  no candidate words (T-029 semantics); only the window presentation changed.

## Alternatives considered

**Keep `Auto` following the system.**
Rejected per user decision: the light Sogou-style palette must be the default
regardless of system theme; system-dark-awareness caused the reported
black-window surprise.

**Show a full-size window with empty rows.**
Rejected per user decision: the header bar matches Sogou/QQ IME behavior and
keeps the panel visually lightweight.

**Distinguish "no candidate" from "no composition" only at the TSF layer.**
Rejected: the decision belongs in the window controller so every caller —
TSF refresh, demo, future integrations — shares the same contract; duplicating
it in callers invites drift.

## Consequences

- The candidate window is now white-backed (`#FFF`, Sogou-classic palette) on
  dark systems too; screenshots from the dark-mode VM drill are the
  verification artifact (shots8).
- Typing `z`, `zh`, or any empty-candidate input keeps a visible header bar,
  which also gives visual confirmation that the IME is active and in
  composition.
- `panel_size(0)` is now a tested layout; the existing 9-row geometry is
  unchanged (regression assertion locks it).
- FR-009 acceptance criteria were updated: "深浅色 (T-030)" and
  "皮肤色板 (T-028/T-030)" state the light default explicitly, and
  FR-001 gained a "无候选页眉条 (T-031)" row.
- Partial supersession of previous notes: the GDI-rendering note’s
  theme-following facts and the prefix-candidates note’s window-hide
  consequence were rewritten in place and cross-linked here.

Verification: `panel_size` unit test for the zero-row bar; fmt, clippy
`-D warnings`, and the full workspace test suite pass; host-e2e seed
(19/19) and real-dictionary smoke (6/6) pass. VM drill on v0.1.2 (Windows
kept in dark mode) confirms both behaviors: `z`, `zh`, and `w` log
`cand-show items=0` with the window visible — the header bar measures exactly
58 px (360×58 rect) — while `nihao`/`zhidao`/`wo` render full panels and
commits spell 你好知道我 (GBK `C4 E3 BA C3 D6 AA B5 C0 CE D2`); commit and
Escape log `cand-hide (no composition)`. Pixel histograms of the archived
screenshots (shots8) match the then-current light palette: white `FFFFFF`
background, `E6F2FE` highlight block, `999999` secondary, `0B57D0` selected
text — all on a dark-mode system, proving the fixed-light default. The
palette values were later replaced by T-032 (blue border/text with red
selected text, see
2026-09-25-candidate-window-blue-red-palette.md); the light-default and
always-visible-panel decisions here remain in force.
