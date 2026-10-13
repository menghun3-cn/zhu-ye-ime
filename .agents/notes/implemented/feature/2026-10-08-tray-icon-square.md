# Agent Note: Square tray state icons — orange badge with white 中/英 (T-123)

Status: implemented

[中文](2026-10-08-tray-icon-square.zh.md) | English | bilingual mirror

## Problem

The batch-6 tray state icons (`crates/zhu-ye-tray/assets/tray-zh.ico` /
`tray-en.ico`) are **white 中/英 glyphs on a fully transparent background**
(single 32×32 frame, alpha=0 at all four corners and edge midpoints). Windows 11
applies a round mask to transparent tray icons, so the visual result reads as a
"circular badge with 中/英 in the middle". User visual feedback (batch 8):
**make the tray icon square**.

## Decision

Give both tray icons a **brand-orange square badge**, matching the indicator
icon family already used by the IME DLL:

- Background `#E5881E` (brand orange, same as the `zhu-16.png` / `zhu.ico`
  design source);
- Right-angled, fully opaque square (alpha=255 at corners), white bold
  centered 中/英 glyph (Microsoft YaHei Bold) instead of a floating glyph on
  transparency;
- Three PNG frames 16/32/48 in the ICO (Vista+ `LoadImageW` natively supports
  PNG frames; the tray loads 16px via `SM_CXSMICON`, scaling to 32/48 on
  high-DPI);
- Generation script `scripts/make-tray-icons.ps1` committed for reproducibility
  (System.Drawing rendering + hand-rolled ICO container, no third-party tools);
- `build.rs` keeps the 101/102 resource semantics; the whole tray mechanism
  (resident process, state-file bridge, single-instance mutex) is untouched.

## Alternatives considered

- **Keep the transparent background.** Rejected: the white-glyph-on-transparent
  combo is exactly what reads as a "circular badge" under Win11's round mask —
  the opposite of the request.
- **Rounded-square badge.** Rejected: the user asked for square; right angles
  also match the `zhu.ico` family (`zhu-16.png` corners are opaque).
- **Different badge color per mode.** Rejected: keep one brand look; mode is
  still signaled by the 中/英 glyph itself (same structure as the system
  indicator's 竹/中), no second palette for the tray.

## Consequences

- At any size (16/32/48) the tray icon is an orange square block with a white
  中/英 glyph; the system round mask now only clips the outermost corners, and
  the badge no longer reads as circular.
- `zhu-ye-tray.exe` grows slightly (three embedded PNG frames, ~ +2 KB).
- Icons are generated artifacts; source and command are documented in the
  script header; after touching the assets, `cargo build -p zhu-ye-tray
  --release` re-embeds them.

### Verification

- Pixel check (GDI+ load of the new ICOs): corners and edge midpoints =
  `(229,136,30)` alpha=255 (opaque square), center white (centered glyph);
  white pixel counts 190/261 for the two icons.
- Extracted all 6 PNG frames (3×zh + 3×en) from the rebuilt
  `target\release\zhu-ye-tray.exe` and byte-compared them against the generated
  assets — identical.
- Gates: `cargo fmt --check`, `cargo clippy -p zhu-ye-tray -- -D warnings`,
  `git diff --check`, and the generator script's `ParseInput` all pass.
- Deployment: user machine worker `copy-exe` overwrote
  `bin\zhu-ye-tray.exe` (262,144B, sha256
  `BC6CFFC9C34F9710608C08E879AA78A3539F458F99FD8B1CE3560A236C217580`) and the
  tray restarted with normal 中/英 polling; the VM got the same binary. VM tray
  pixel probe: 16×16 orange square bbox (x62..77, y40..55) with white glyph —
  Chinese mode 190 orange / 168 white px, English mode 174 / 165 px (shots:
  `target/t122-deploy/t123-shots/tray-zh.png`, `tray-en.png`).

Related: [Batch-6 system-tray 中/英 mode icon](../../implemented/feature/2026-10-07-system-tray-mode-icon.md)
— mechanism unchanged; only the icon look moves from "white glyph on transparent"
to "orange square badge with white glyph" (that note is maintained in the same
batch).
