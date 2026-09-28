# Agent Note: Candidate window GDI rendering for T-012

Status: implemented

[中文](2026-09-19-candidate-window-gdi-rendering.zh.md) | English

## Problem

FR-009 requires a visible candidate window for the IME: rendered candidates,
translation text, selection state, and page indicators. FR-011 additionally
requires the window to support light/dark palettes and high-contrast mode (the
default palette is fixed light since T-030), plus per-monitor DPI. Before this
task the repository had no UI code at all, so there was no way to verify the
layout model or the rendering path until TSF integration.

## Decision

The candidate window is split into a platform-independent view layer and a
Win32 GDI presentation layer.

`candidate_ui` owns the snapshot model, layout metrics, and themes in pure
Rust. `CandidateUiView` carries composition, pinyin hint, page, selection,
translation mode, and items; `CandidateMetrics` computes all geometry from a
base 96 DPI design scaled linearly; `UiThemeKind` plus `CandidateUiTheme`
define light, dark, and system high-contrast palettes. It also provides index
markers and text-width estimation helpers, and stays free of Windows
dependencies for unit testing.

`candidate_window` implements the Win32 popup window with GDI double
buffering. The window is `WS_POPUP` with `WS_EX_NOACTIVATE`, `WS_EX_TOOLWINDOW`,
and `WS_EX_TOPMOST` so it does not steal focus from the composition target.
`WM_PAINT` renders into a memory DC and `BitBlt`s the frame; background erase
returns immediately to avoid flicker. Theme resolution reads
`SPI_GETHIGHCONTRAST`; since T-030 the default palette is fixed light and no
longer follows `AppsUseLightTheme` (dark is only reachable through an explicit
`ThemePreference::Dark`; see
2026-09-25-candidate-window-light-default-and-empty-panel.md), `WM_THEMECHANGED` and `WM_SETTINGCHANGE` refresh the
palette, and `WM_DPICHANGED` rebuilds metrics and the font. The candidate
font face is SimSun since T-034 (see
2026-09-25-candidate-window-font-simsun.md). Fonts, bitmaps,
DCs, and the boxed state are released on every path, including window
destruction.

The demo binary `candidate-demo` renders a fixed nine-item snapshot and
supports `--theme`, `--dpi`, `--seconds`, `--shot`, and `--translation-mode`.
`--shot` captures the client area through `GetDIBits` into a top-down 32-bit
BMP for visual verification. TSF wiring, caret-based positioning, and key
handling are explicitly deferred to T-013; this window currently only paints
a static snapshot.

## Testing

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test --workspace` pass. Coverage includes page slicing, selection on
page, DPI scaling ratios, non-overlapping row and header rectangles, theme
palette distinction, UTF-16 conversion, and BMP header consistency.

The demo was run at 96 and 192 DPI in light and dark themes and produced
readable BMP snapshots. Pixel histograms match the project palettes: light
background `FAFAFA`, dark background `202020`, and the selected-row highlight
colors, confirming that the window and text surfaces are actually painted.

## Alternatives considered

**Direct2D/DirectWrite.** Rejected for this milestone: the text volume is a
few short lines and GDI rendering is sufficient; DirectWrite adds a larger
API surface and more careful resource lifetime work inside a TSF host used by
32-bit and 64-bit processes. If text quality or shaping requirements grow,
the `CandidateUiView` boundary lets T-013 later swap the renderer without
touching layout or tests.

**Browsing-engine overlay (WebView/HTML).** Rejected outright: an embedded
web runtime contradicts the project's anti-bloat principle, adds startup
cost, and creates an unnecessary attack/signature surface inside the input
process.

**Direct single-buffered painting.** Rejected: erase and repaint on the
window DC produces flicker and tearing on every theme or DPI change; a
memory DC plus `BitBlt` is the established low-cost GDI pattern.

**Custom text truncation based on estimated width.** Rejected: `DrawTextW`
with `DT_END_ELLIPSIS` is already correct and locale-aware; the pure-Rust
estimator remains as a measurement helper for future layout work instead of
being the rendering authority.

## Consequences

The M3 candidate window rendering foundation is now in place and verifiable
outside TSF. Win32 calls stay confined to `zhu-ye-ime`; `zhu-ye-core` remains
platform-independent. The transparent snapshot and screenshot path give the
VM verification round a visual baseline before T-013 wires the window to a
live composition.

A few UI details still depend on the TSF round: real composition text,
caret-anchored positioning, key routing, page switching, and showing/hiding
the window. T-012 therefore remains in progress until the VM end-to-end check.
If later text-quality requirements appear, DirectWrite is a local replacement
behind the same view contract rather than a rework of the whole candidate
window.
