# Agent Note: Candidate window font changed to SimSun (宋体)

Status: implemented

[中文](2026-09-25-candidate-window-font-simsun.zh.md) | English

## Problem

The candidate popup was rendered with the system UI sans face
`Microsoft YaHei UI` (created in `candidate_window::create_font`). The user
directive: the candidate window should use 宋体 (SimSun).

## Decision

- `create_font` now uses the Latin face name `SimSun` (the standard Chinese
  serif 宋体 face present on every Chinese Windows). Height, weight, charset,
  ClearType quality, DPI scaling (`font_height` via `dp(16.0)` at 96 DPI) and
  the `DEFAULT_GUI_FONT` stock fallback are unchanged.
- The width heuristics in `candidate_ui` (`estimate_text_width`: 0.55×height
  per ASCII, 1.0×height per CJK) stay as-is: they are font-independent
  approximations and `fit_text`/`DT_END_ELLIPSIS` still bound overflow.
- Implementing this exposed a pre-existing bug: the created HFONT was never
  `SelectObject`-ed into the paint DC, so the face name had no effect at all
  (see
  [the bug-fix note](../bug-fix/2026-09-25-candidate-window-font-never-selected.md)).
  `CandidateWindowState::paint` now selects the font; the SimSun change is
  only visible because of that fix.

## Alternatives considered

**Use the Chinese face name `宋体` instead of `SimSun`.**
Rejected: the Latin name is locale-independent and unambiguous in generic
`LOGFONTW` construction; `SimSun` is what Windows itself resolves 宋体 to.

**Keep YaHei and only change weight/size.**
Rejected — the user asked for the serif face, not a tweak of the sans.

## Consequences

Since fonts were not pinned in any doc, only the FR-009 acceptance row gains
a font check (T-034). The GDI rendering note's generic "fonts" wording stays
accurate. Verification on VM v0.1.6 (test11), real TSF pipeline:

- SimSun build vs SimHei A/B arm: the whole panel differs — 5964 (page0) /
  5327 (page1) pixels over y 24..377 — the selected face reaches the DC.
  On this VM the default GUI font is SimHei (黑体), so the SimHei arm equals
  the old no-selection builds pixel-for-pixel (which also explains why the
  earlier "YaHei" screenshots never looked like YaHei).
- The installed DLL embeds the `SimSun` string; screenshots archived under
  shots11-simsun (final) and shots11-hei (A/B arm); gates green.
