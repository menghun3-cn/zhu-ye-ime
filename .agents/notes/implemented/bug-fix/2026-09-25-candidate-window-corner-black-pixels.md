# Agent Note: Candidate window corner black pixels for T-037

Status: implemented

[中文](2026-09-25-candidate-window-corner-black-pixels.zh.md) | English

## Problem

The candidate window showed black specks at its four corners. The root cause
was in `paint_background`: it drew only a `RoundRect`, so the four corner
regions outside the rounded rectangle were **never filled**. Painting was
double-buffered into a memory DC (`CreateCompatibleBitmap`), whose initial
pixels — uninitialized, typically black/garbage — were blitted through in
those corner squares.

## Decision

`paint_background` now fills the entire client rectangle with the background
brush **first**, then draws the `RoundRect` (border) on top of that backdrop.
The corner regions outside the rounded shape therefore become background
color instead of uninitialized pixels. No other drawing changes.

## Alternatives considered

**`WS_EX_LAYERED` with per-pixel alpha for truly transparent corners.**
Rejected for this milestone: layered windows complicate positioning, DPI and
theme handling for a cosmetic region, and the chrome around the rounded
rect is a few pixels wide.

**Drop the border pen entirely.** Rejected: FR-009 keeps the blue rounded
border (#1E88E5 light theme); the border was not the source of the specks.

**Square corners (radius 0).** Rejected: changes the established visual
style instead of fixing the defect.

## Consequences

On the VM (v0.1.7) all four 3×3 corner squares inside the window are pure
white, while the previous build had non-white pixels in those squares.
Visually the corners are now background-colored; combined with the row
layout change this is the T-037 candidate-window polish. Related notes:
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
and
[2026-09-19-candidate-window-gdi-rendering.md](2026-09-19-candidate-window-gdi-rendering.md).
