# Agent Note: Candidate-window color emoji rendering via DirectWrite (T-101, T-074 evolution)

Status: proposed

[中文](2026-10-05-candidate-window-color-emoji.zh.md) | English

## Problem

The candidate window renders all text through GDI (`DrawTextW`, SimSun font,
decision note 2026-09-19-candidate-window-gdi-rendering). GDI cannot rasterize
color fonts (`COLR`/`CBDT` layers); emoji such as 😀 and 🚀 therefore appear as
monochrome outlines, and sequences with VS16 (❤️) lose their color form. The
user explicitly asked for "emoji rendered in color via DirectWrite" (batch
three, item 1 of the accepted 11-item list). The settings toolbox emoji panel
(T-074 note 2026-10-01-settings-window-process-and-registration-ownership)
documented this exact limitation; this note partially supersedes that paragraph
for the candidate window only — the settings panel stays on GDI.

## Proposal

1. **Detection (`contains_color_glyph`).** Scan UTF-16 units; trigger when a
   high surrogate `D83C..=D83F | D940` is followed by a low surrogate
   `DC00..=DFFF` (supplementary-plane emoji), or a BMP unit falls in
   `2600..=27BF | 2B00..=2BFF` (misc symbols/decoratives/arrows) or `FE0F`
   (VS16). Ordinary CJK and Latin never trigger → GDI path has zero regression.
2. **Single-point bypass in `draw_text`.** If the line contains a color glyph,
   route to `color_text::render_color_text` and composite the result with
   `AlphaBlend(AC_SRC_ALPHA)`; any failure inside the pipeline falls back to the
   original GDI text (color is an enhancement, never a correctness
   precondition). Font size is read from the DC's current font
   (`GetCurrentObject(OBJ_FONT)` + `GetObjectW`, negative `lfHeight` ≈ DWrite em
   size) so the colored line matches the GDI line metrics.
3. **Render pipeline (software, no GPU dependency).**
   `DWriteCreateFactory(ISOLATED)` → `IDWriteFactory::CreateTextFormat
   ("Segoe UI", 16.0)` where font fallback automatically selects Segoe UI Emoji
   → `CreateTextLayout` (em line size overridden per draw via `SetFontSize`) →
   `D3D11CreateDevice(D3D_DRIVER_TYPE_WARP, BGRA_SUPPORT)` → DXGI device →
   `D2D1CreateFactory → CreateDevice → CreateDeviceContext`
   (`ID2D1DeviceContext::DrawTextLayout`, the D2D 1.1 overload that enables
   color glyphs by default) targetting a `B8G8R8A8` premultiplied bitmap on the
   D3D11 render-target texture → `CopyResource` to a staging texture → `Map`
   readback of straight premultiplied BGRA → up to the caller for
   `AlphaBlend(AC_SRC_ALPHA)`.
4. **Caching.** The DWrite factory, text format, D3D11 WARP device/context and
   D2D1 device context are cached process-wide in one `OnceLock<Option<SharedCtx>>`;
   layouts, targets and brushes are rebuilt per draw. The WARP (software) driver
   keeps the TSF host free of GPU/swapchain dependencies and works in headless
   sessions.
5. **windows 0.61 API notes** (audit trail for anyone extending this): the
   `Win32_Graphics_Direct2D_Common` and `Win32_Graphics_Dxgi_Common` modules are
   separate features; `DWriteCreateFactory`/`D2D1CreateFactory` are generic
   1/2-arg functions returning `Result<T>`; `CreateTextFormat` takes 7 args;
   `ID2D1DeviceContext::DrawTextLayout` (4-arg overload) takes
   `windows_numerics::Vector2` (dependency `windows-numerics 0.2`, the type
   windows itself uses) and returns `()`; `EndDraw(None, None)` returns
   `Result<()>`; `CreateDIBSection` returns `Result<HBITMAP>`; `Map`/`Unmap`
   require the resource cast to `ID3D11Resource`.
6. **Demo support.** `candidate-demo --emoji` injects an emoji row so screenshots
   can be re-generated; the default output (base shots) is unchanged.

## Alternatives considered

- **`ID2D1RenderTarget::DrawTextLayout` with `D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT`**:
  the classic RT overload only knows the flag's "try color" hint and the
  rendered output stays monochrome for `COLR` fonts in practice; the correct
  contract is the D2D 1.1 device-context overload, where color is enabled by
  default → chosen pipeline uses the context overload.
- **`GetDC` color fonts via `EndPath`/`GetPath`**: outline extraction cannot
  recover layered color data → rejected.
- **Rendering into a premultiplied bitmap via D2D then `BitBlt`**: `BitBlt`
  ignores the alpha channel (copies RGB with black for transparent pixels) →
  `AlphaBlend(AC_SRC_ALPHA)` with a pre-multiplied 32bpp DIB is the correct
  compositor.
- **WIC (`IWICBitmap`) rasterization**: no DirectWrite text layout → rejected.
- **Per-draw `DWriteCreateFactory`**: object creation is expensive and ctfmon
  may call draw paths repeatedly → process-wide single instance cached instead.

## Acceptance criteria

- `cargo test --workspace` green (new tests: trigger ranges, mixed text, and a
  best-effort real render self-check asserting an opaque pixel with alpha > 0).
- `fmt --check` / `clippy --all-targets -- -D warnings` / `git diff --check` clean.
- Host screenshots archived at `data/artifacts/t101-shots/`: light/dark ×
  base/emoji; the emoji pair must show a distinct color palette vs the base
  pair (verified: base 5/6 colors vs emoji 42/43 distinct colors; emoji-only
  hues such as sky-blue and pink present).
- Demo `--emoji` exists and default (non-emoji) output is byte-identical in
  content to the pre-T-101 base (no regression marker remains in the base row).
- VM interactive verification (real TSF stack, typed emoji candidate rows)
  remains in the acceptance backlog like the other batch items.

## Risks

- **Fallback correctness**: any failure (missing WARP, missing Segoe UI Emoji,
  device re-creation) returns `None` and GDI draws — the candidate line stays
  readable but monochrome; no panics (all results are unwrapped via `?` inside
  `Option`).
- **Baseline/metric drift**: DWrite em size approximates GDI `-lfHeight`; the
  same metrics as GDI are requested by reading the live DC font, but exact
  pixel alignment is not byte-guaranteed — acceptable since each row rect owns
  its area.
- **Text overflow**: the DWrite path does not apply `DT_END_ELLIPSIS`; long
  colored text could overflow the row rect instead of ellipsizing. Currently
  the injected demo rows are short; candidate rows are width-capped by the
  dictionary/engine and the compositor draws within `rect` exactly.
- **`AlphaBlend` availability**: `msimg32` is available on all supported
  Windows versions; the call is fail-soft (return value ignored).
- **Per-frame cost**: layout + WARP texture readback per colored row is far
  heavier than GDI; acceptable because only emoji rows take this path and the
  candidate window already double-buffers full-frame.
