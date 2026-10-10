# Agent Note: Settings window visual redesign (FR-064, T-148/T-149)

Status: implemented

## Problem

The settings window (`zhu-ye-settings.exe`) has used the Windows-native neutral
look since T-073 — white surfaces, light-blue selection (`#E1EEFB`/`#1E88E5`) —
and T-125 normalized its typography only. The window shares none of the
[bamboo brand](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.md)
that the official website established, so it reads as "any settings dialog".
After two grill-me rounds (2026-10-09) the user locked a visual-only redesign:
"change the face, not the skeleton." The structural
[settings window design](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)
and its accepted metrics and interaction mechanisms stay untouched (T-148
delivered the spec + mockup; user inspection passed 2026-10-10).

## Decision

The window carries the bamboo identity through **color plus one brand block**
(design doc [§3–§9](../../../docs/设置窗口视觉设计.md)); every visual change is
GDI-expressible (solid fills, 1px hairlines, round rects, ClearType text — no
gradient/shadow/translucency/animation, S-24).

1. **Scope — face only.** Colors, type-weight hierarchy, and control forms
   change; window 880×700, nav 200, row 76, title area 80, type scale
   16/13/24 (T-125 acceptance values), three-page information architecture,
   chips two-state / placeholder-expand / subview / float-panel interactions,
   and the candidate window's accepted blue palette are untouched.
2. **Brand block** at the top of the nav rail, inside the nav column only
   (`LOGICAL_BRAND` 64 + top gap 10; **zero body-metric impact**): the leaf
   glyph is `DrawIconEx`-drawn from `zhu.ico` (T-142 resource ID 1, 22px,
   `DI_NORMAL`), followed by the wordmark 「竹叶输入法」 16/600 and a 1px
   `bark` hairline. No mascot, no patterns, no extra graphics anywhere.
3. **Two themes follow the system** (D-30 unchanged; high contrast still takes
   system colors, S-23): light = bamboo paper `#F7F9F4` / ink `#22302A` / leaf
   `#3D7A4E`; dark = ink-green `#1F2822` / bright leaf `#80B98A`.
4. **One selection language** (S-22): nav and chips select as light leaf-tint
   background + deep leaf text. Solid leaf is reserved for buttons (and the
   panel's hovered cell). Dark-mode solid buttons use **ink text** `#10241A`
   on `#80B98A` (7.15:1) because white text fails at 2.3:1 (S-20).
5. **Config compatibility** (S-25): `SettingsTheme` grows from 15 to **22
   keys** — seven new optional keys (`on_accent`, `chip_border`, `tag_bg`,
   `tag_text`, `sprout`, `ok_text`, `expanded_bg`); `zhu-ye-core`'s
   `SettingsPalette` mirrors them as 22 optional fields. Custom
   `themes\*.json` missing any new key falls back to built-in defaults; no
   `CONFIG_FORMAT_VERSION` bump; the About status line reports "applied N/22
   keys". `bark` is a brand-fixed constant, **never** overridden by theme
   files. High contrast maps all 22 keys from `SystemColors` (8 new mappings:
   `on_accent`→highlight foreground, `chip_border`/`expanded_bg`→border/window,
   tag pair/sprout/ok_text→foreground, etc.) and does not read theme files.
6. **Component language** (T-149): solid buttons `accent` + `on_accent`
   (`small_medium` 500); outlined buttons `control_background` + 1px
   `chip_border` + `item_text` (返回/恢复/打开系统输入法设置/二级修复/可点的应用更新);
   disabled = same outline + `secondary_text`; planned tag = pill
   `tag_bg`/`tag_text` + 6px `sprout` dot, 13px; placeholder expand =
   `expanded_bg` fill + `line` outline, 13px secondary text; About result box
   (new container) = same expand surface + empty hint; status lines use
   `ok_text`/`warn_text` (manage registered/abnormal, About online/offline);
   chips 13px with 600 weight when selected; panel cells highlight on mouse
   hover (TrackMouseEvent `TME_LEAVE` lifecycle) as the selection-cell visual —
   non-activated float panels have no persistent selection (click commits
   immediately, S-11 unchanged); panel borders `chip_border`.
7. **Brand/typography locks**: chips/buttons at 13px and tag at 13px per the
   T-125 16/13/24 scale (draft 14px superseded, S-28); `logo`/`zhu.ico` is
   reused as-is — no new asset files (S-26).

## Alternatives considered

- **Full bamboo branding everywhere** (mascot, leaf patterns, brand bar on
  every page): rejected — the settings window is a daily, high-frequency tool;
  identity through color plus one brand block is the restrained match for it,
  while the website stays the rich-expression venue.
- **Keep the current neutral look, polish only**: rejected — the user's brief
  and the grill rounds asked for a distinguishable identity; this option keeps
  the "any settings dialog" deficit.
- **Full-width top brand band above the content**: rejected — it consumes body
  metrics and would weaken a T-125 acceptance assertion; the nav-rail position
  carries the same visual weight with zero metric impact.
- **Gradients / shadows / translucency**: rejected — GDI cannot express them;
  a D2D path for decorative effect contradicts the window's zero-framework
  architecture (S-24).
- **Brand glyph via `PolyBezier` control-point table or pre-rendered DIB**
  (draft candidates): rejected at implementation — the curve table is costly
  to maintain and the DIB breaks the zero-new-asset rule; `DrawIconEx` reuses
  the shipped `zhu.ico` 1:1 (S-26).
- **White text on dark solid buttons**: rejected — measured 2.3:1, below AA;
  ink text `#10241A` on `#80B98A` passes at 7.15:1 (S-20).
- **Persistent selection state in float panels**: rejected — non-activated
  panels commit on click (S-11); a hover highlight expresses the current cell
  without changing the mechanism (S-27).
- **Placeholder expand reusing `tag_bg`** (draft): rejected at implementation —
  a dedicated `expanded_bg` key lets theme files control the surface (S-25).

## Verification

- Unit tests: `设置窗视觉改版新键可选解析` (theme_file), `视觉改版新字段随深浅预设且深浅互异`
  and `主题文件新键覆盖与缺键回退且bark不可覆盖` (theme) plus the existing
  suites — all green via `cargo test --workspace`.
- Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `git diff --check` all clean.
- Pixel evidence (host `--shot`, 125% DPI, frames 1100×875): 22 light/dark
  frames; per-frame token histograms match the light (`#F7F9F4`/`#EFF3EA`/…)
  and dark (`#1F2822`/`#192019`/…) palettes exactly; the bark hairline sits at
  logical y=63; nav rail-sel and chip surfaces count as designed; the ≥7-row
  first-screen assertion is unchanged; About result box shows its expand
  surface. Full table in 验收标准 §18.2.
- VM interactive inspection (dark real-machine cursor hover, high-contrast
  live colours) stays pending under the established settings-window
  acceptance口径 (vm-accept-sop).

## Consequences

- **Costs**: theme surface is larger (22 keys + the reserved constant), theme
  files can now tune seven more surfaces; the panel gains a small
  TrackMouseEvent hover lifecycle; About text is inset by one `gap` inside its
  new result container.
- **Benefits**: the window finally reads as part of the bamboo brand; the dark
  accent contrast is audited to AA (7.15:1 vs the rejected 2.3:1 white);
  existing custom themes keep parsing and fall back gracefully; zero new
  assets and zero new dependencies; the candidate window's blue palette and
  every interaction mechanism stay untouched (S-15 not triggered).

## Was proposed

Moved from `proposed/feature/2026-10-10-settings-window-visual-redesign.{md,zh,i18n.yaml}`
(design-batch Agent Note) when T-148's user inspection passed and the T-149
implementation landed.

## Related

- [Official website and brand assets](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.md):
  token family, `logo`/`zhu.ico` and the T-142 title-bar icon reused by the
  brand block.
- [Settings window process and registration ownership](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md):
  the structural settings-window design this visual pass deliberately leaves
  alone.
- Design/acceptance docs: [设置窗口视觉设计](../../../docs/设置窗口视觉设计.md)
  (spec, §3–§9, S-19–S-29) and [验收标准 §18](../../../docs/验收标准.md)
  (T-148/T-149 acceptance, 18.2 pixel evidence table).
