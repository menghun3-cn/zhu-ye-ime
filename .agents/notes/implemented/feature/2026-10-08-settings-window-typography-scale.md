# Agent Note: Settings window typography normalization

Status: implemented

## Problem

The settings window (zhu-ye-settings, Win32 GDI self-drawn) inherited the system
message font at 12px for body text, a 15px semibold page title, and dense 62px item
rows in an 880×620 window. The user asked for a Mac-like modern IME settings layout
(搜狗输入法-style): larger base type (14-16px), 1.5-1.8 line breathing room, a
roomier sidebar with bold selected item, a 22-24px bold page title, and list items
with 16px titles, 13-14px grey descriptions, and 16-20px vertical padding.

## Decision

There is no CSS in this crate — the window is GDI self-painted, so the redesign maps
to the font set (`gdi.rs`), the layout constants (`layout.rs`), and the paint code
(`window.rs`).

- **Font set (gdi.rs)**: body 16px (system message font ×4/3, within the user's
  14-16px band), page title 24px `FW_BOLD` (×6/3), description 13px (×13/12), and a
  new `nav_active` font — 16px `FW_SEMIBOLD` — used only for the selected sidebar
  item, which already keeps its pale-blue `nav_selected` background. Line breathing
  room comes from taller text rects plus the new row heights (≈1.5-1.8 em of space
  per text band).
- **Layout constants (layout.rs)**: window 880×620 → **880×700**; sidebar column
  184→200 and nav row 46→**54** (≈12px padding above/below a 16px label); title zone
  60→**80** (24px title + 24px spacing below it before content); item row 62→**76**
  (≈20px vertical padding on each side of the two-line title+description block);
  expanded note zone 46→54; content padding 22→26; choice chips 82×30→88×32; emoji
  panel cells 52→58 and header 46→54 (the 24px glyph/title font now used there).
- **Paint (window.rs)**: the selected nav item draws with `nav_active` (bold); item
  title and description are laid out as one vertically centered block inside the row
  instead of hugging the top, giving even whitespace and a natural divider gap below
  each row's 1px `border` line.

## Alternatives considered

- **Keep the single ratio-based font ladder (body 1×, title 5/4×, small 4/5×)** and
  only bump the base: the title would reach only 20px and the description 12.8px,
  missing the user's explicit 22-24px title and 13-14px description targets; rejected
  in favor of explicit per-role sizes.
- **Add a real scrollbar for the 15-item 常用设置 page** instead of raising row
  heights: scrolled lists are not implemented in this GDI window (rows beyond the
  content area are clipped by design), and the task was typography/spacing, not new
  interaction; deferred.
- **Keep the 620px window height**: with 76px rows the visible count would drop from
  7 to 5.6 rows on 常用设置; widening the window to 700 keeps 7 visible rows while
  making every row roomier.

## Consequences

- The 工具箱 page (3 items) and 关于与更新 page now read as spacious, modern settings
  pages; 常用设置 keeps 7 of its 15 items visible at the default 700px height (same
  count as before, with taller rows), and remains clipped by the existing
  clip-not-scroll mechanism.
- The emoji/symbol panel pick-up grid scales with the larger fonts (cell 58px, glyph
  24px), so the toolbox overlay stays visually consistent.
- All sizes remain DPI-scaled through `scale()`/the font DPI ladder; high-contrast
  and dark themes are untouched (colors unchanged).

## Verification

- New layout test `默认窗口高度下常用设置页至少显示七行`; settings crate 103/103,
  workspace full suite green, fmt/clippy/diff-check clean.
- Pixel probe of `--shot` captures at 125% system scale (window 880×700 → 1100×875):
  page title ink height 29-30px = **24dp**; item title band ≈20px = **16dp**;
  description grey band ≈16px = **13dp** (#757575, within #666-#999); item row pitch
  95px = **76dp**; nav row pitch 67px = **54dp**; selected 工具箱 glyph span per char
  +16% vs unselected rows (bold); divider 1px line with ≈30px whitespace below the
  description.
