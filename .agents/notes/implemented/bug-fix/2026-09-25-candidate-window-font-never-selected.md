# Agent Note: Candidate window HFONT was never selected into the DC

Status: implemented

[中文](2026-09-25-candidate-window-font-never-selected.zh.md) | English

## Problem

The candidate popup was supposed to render with the face produced by
`create_font` (`LOGFONTW` + `CreateFontIndirectW`). In practice
**`SelectObject` was never called for the HFONT on any paint path**, so every
`DrawTextW` used the DC's default font (`DEFAULT_GUI_FONT`). The
`create_font` handle was created, stored, and destroyed, but never used — the
face name in the source had zero rendering effect. Symptom: changing the face
name in code changed nothing on screen.

Why it stayed invisible for so long: on the user's desktop Windows the DC
default looks close to the then-intended "Microsoft YaHei UI", while on the
VM (Windows Server, zh-CN) the default GUI font is SimHei (黑体) — so the
"YaHei" build and the "SimSun" build captured pixel-identical candidate
panels (see the A/B evidence under Consequences).

## Decision

Select the font in the paint path instead of creating it unused:

- `CandidateWindowState::paint` now calls `SelectObject(hdc, self.font)` at
  the start and restores the previous object before returning.
- `paint_window` builds a fresh memory DC per `WM_PAINT`, so the restore is
  defensive hygiene; the font handle lifecycle (create on build, rebuild on
  DPI change, delete on drop) is unchanged.
- This fix is part of T-034 (font face → SimSun); see
  [the font face note](../feature/2026-09-25-candidate-window-font-simsun.md).

## Alternatives considered

**Leave the font unselected and rely on `DEFAULT_GUI_FONT`.**
Rejected: the face name in `create_font` was dead code, and the look then
depended on the OS locale/tuning (雅黑 on desktop zh-CN, 黑体 on zh Server),
which is exactly the non-determinism the user's font request is about.

**Select the font inside `draw_text` per call.**
Rejected: one selection per frame is enough; per-call selection would add
noise without benefit since no other code path changes the DC font mid-frame.

## Consequences

- The face name now determines the rendered glyphs. VM v0.1.6 A/B (real TSF
  pipeline): SimHei arm renders identically to the old no-selection builds
  (the VM's default GUI font is 黑体), while SimSun differs across the whole
  panel — 5964 (page0) / 5327 (page1) differing pixels spanning y 24..377.
- The pre-fix behavior is now documented as a fixed defect; screenshots under
  shots11-simsun / shots11-hei, gates green.
