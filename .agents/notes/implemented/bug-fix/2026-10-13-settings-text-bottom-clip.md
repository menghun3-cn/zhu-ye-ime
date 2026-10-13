# Agent Note: Settings window description text bottom clipped by text-rect height

Status: implemented

[中文](2026-10-13-settings-text-bottom-clip.zh.md) | English

## Problem

User reported (T-152) that in the settings window several description lines look
"clipped at the bottom", e.g. the "从 themes\*.json 加载配色…" line under
"自定义主题" in the common page; the lower part of the glyphs is cut off flat in
other descriptions as well. Root cause: layout rectangles for text were sized to
the font's em height (`lfHeight.abs()`), but GDI typesets text at
`tmHeight + tmExternalLeading` (the system UI font reports `tmHeight ≈ lfHeight`
but adds 2–3 px external leading; CJK fallback glyphs also carry extra visual
room above and below the em box). `DrawTextW` clips to the rectangle by default,
so the bottom 1–2 px of the glyphs were sliced off; on the user's 125 % display
scale the mismatch is more visible.

## Decision

Two complementary changes in `crates/zhu-ye-settings/src/gdi.rs`:

- **实测行高**：`Fonts::ensure` now measures each font with `GetTextMetricsW`
  and stores `tmHeight + tmExternalLeading` as `body_height` / `small_height`,
  falling back to `lfHeight.abs()` when measurement fails. Every layout
  rectangle computed from these heights (item title/description blocks, chip
  rows, subview notes) automatically grows enough to contain the true line box.
- **单行防护**：`draw_text` forces `DT_NOCLIP` whenever the caller passes
  `DT_SINGLELINE`, so any residual 1–2 px of vertical overhang can never be
  clipped. This is safe for single-line text because the overhang lands in the
  neighbouring whitespace (row blocks reserve ≥ 3 px gaps); multi-line
  `DT_WORDBREAK` keeps clipping so text cannot spill out of a panel. The flag
  transformation lives in a pure helper `singleline_noclip` with 2 unit tests.

## Alternatives considered

**Increase every text-rect height independently at each call site.**
Rejected: dozens of single-line call sites across the window and the toolbox
panel, easy to miss and to drift; fixing the two shared sources (font height
semantics + draw_text) covers both windows at once.

**Add `DT_NOCLIP` unconditionally for all text.**
Rejected: multi-line notes (`DT_WORDBREAK`) inside expanded panels would draw
outside their panel bounds when a rect is too short; rect height should grow,
not clipping be disabled wholesale.

**Switch UI font row metric to `tmHeight` only (without leading).**
Rejected: `tmHeight` alone equals the em box again; the external leading is
exactly the slack that absorbs fallback overhang.

## Consequences

- All text rectangles in the settings window and the toolbox panel grow by the
  external leading (2–3 px at typical scales); item blocks re-center inside the
  fixed 76 px row, so rows shift by ≤ 1–2 px. Frame diff before/after shows only
  these expected vertical nudges, and every description band has ≥ 3 px of pure
  background below it (no bottom clipping).
- `Fonts` heights used by `panel_window.rs` benefit automatically (shared
  `Fonts`). Multi-line behavior is unchanged.
- A first-principles note for future GDI work: never derive text-rect heights
  from `lfHeight` alone; measure `tmHeight + tmExternalLeading` and keep
  `DT_NOCLIP` for single-line draws.

## Related

- Settings window visual design (font roles, 16/13/24 system, row metrics):
  [docs/设置窗口视觉设计.md](../../../docs/设置窗口视觉设计.md) §5
- Acceptance: [docs/验收标准.md](../../../docs/验收标准.md) §20 (T-152)
- T-125 typography normalisation context: same design doc §5/§6.3
