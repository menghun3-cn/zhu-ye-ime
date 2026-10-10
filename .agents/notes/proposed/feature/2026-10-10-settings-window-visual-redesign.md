# Agent Note: settings window visual redesign (FR-064, T-148)

Status: proposed

[中文](2026-10-10-settings-window-visual-redesign.zh.md) | English

## Problem

The settings window (`zhu-ye-settings.exe`) has used the Windows-native neutral
look since T-073 — white surfaces, light-blue selection (`#E1EEFB`/`#1E88E5`) —
and T-125 normalized its typography only. The window shares none of the
[bamboo brand](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.md)
that the official website established, so it reads as "any settings dialog".
After two grill-me rounds (2026-10-09) the user locked a visual-only redesign:
"change the face, not the skeleton." The structural
[settings window design](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)
and its accepted metrics and interaction mechanisms stay untouched.

## Proposal

1. **Scope — face only.** Colors, type-weight hierarchy, and control forms
   change; window 880×700, nav 200, row 76, title area 80, type scale
   16/13/24 (T-125 acceptance values), three-page information architecture,
   chips two-state / placeholder-expand / subview / float-panel interactions,
   and the candidate window's accepted blue palette are all untouched.
2. **Brand posture — one element only.** Tokens come from the website's bamboo
   family with UI lightness tiers (same hue, interaction-safe luminance).
   The single graphic is a **brand block** at the top of the nav rail
   (22px leaf glyph from the logo curve family + 「竹叶输入法」 16px/600 +
   1px bark hairline). No mascot, no patterns, no gradients anywhere else.
3. **Two themes, following the system** (D-30 unchanged; high contrast still
   takes system colors): light = bamboo paper `#F7F9F4` / ink `#22302A` /
   leaf `#3D7A4E`; dark = ink-green base `#1F2822` / bright leaf `#80B98A`.
4. **One selection language.** Nav items and chips select as light leaf-tint
   background + deep leaf text (mechanism unchanged, colors swapped). Solid
   leaf is reserved for buttons and the panel's selected cell. Dark-mode solid
   buttons use **ink text** `#10241A` on `#80B98A` (7.15:1) because white text
   fails at 2.3:1.
5. **GDI boundary.** Every surface is expressible today: solid fills, 1px
   hairlines, round rects, ClearType text. No gradient, shadow, translucency,
   or animation; future D2D only as a documented non-goal.
6. **Config compatibility.** New theme keys (`on_accent`, `chip_border`,
   `tag_bg`/`tag_text`/`sprout_dot`, `ok_text`) are all optional — custom
   `themes\*.json` missing them falls back to built-in defaults; no
   `CONFIG_FORMAT_VERSION` bump (same lenient-parse culture as T-088).
   High-contrast mapping overrides everything as today.
7. **Delivery.** Single-file HTML mockup
   (`design/settings-window/mockup.html`, `file://`-openable, minimal inline
   JS for demo only) plus
   [docs/设置窗口视觉设计.md](../../../docs/设置窗口视觉设计.md) carrying the
   measured WCAG contrast table and a GDI implementation mapping (every token →
   existing or new `theme.rs` field, §9 of the doc). GDI implementation is a
   separate task (T-149) started after the mockup passes user inspection.

## Alternatives considered

- **Full bamboo branding everywhere** (mascot, leaf patterns, brand bar on
  every page): rejected — the settings window is a daily, high-frequency tool;
  identity through color plus one brand block is the restrained match for it,
  while the website stays the rich-expression venue.
- **Keep the current neutral look, polish only**: rejected — the user's brief
  and the grill rounds asked for a distinguishable identity; this option keeps
  the "any settings dialog" deficit.
- **Full-width top brand band above the content**: rejected — it consumes
  48px of body metrics and would weaken a T-125 acceptance assertion; the
  nav-rail position carries the same visual weight with **zero** metric impact.
- **Gradients / shadows / translucency**: rejected — GDI cannot express them;
  adding a D2D path for decorative effect contradicts the window's
  zero-framework architecture.
- **White button text in dark mode**: rejected — measured 2.3:1, below WCAG;
  ink text `#10241A` on `#80B98A` reaches 7.15:1.
- **Recolor the candidate window to match**: out of scope — its blue palette
  is locked by acceptance (T-030/T-032/T-037/T-043) and S-15's website-illustration
  sync obligation; blue/green coexistence already has precedent on the site.

## Acceptance criteria

- Mockup user inspection passes: `file://` direct open, no external requests;
  light and dark each over all three pages, placeholder expand, repair and
  diagnostics subviews, emoji and symbol panels.
- Every final token pair measured ≥4.5:1 (table in design doc §4; regular
  text and dark surfaces mostly reach AAA).
- Implementation mapping table lands 1:1: no un-mapped visual element in the
  shipped window.
- Implementation batch (T-149): `--shot` BMPs for both themes — pixel samples
  equal the tokens, light/dark pairwise differ; custom theme missing new keys
  falls back without crash (unit test); high-contrast uses system colors;
  first-screen row-count assertion unchanged (brand block lives in the rail).
- Gates green: fmt / clippy `-D warnings` / workspace tests / diff-check /
  verify-agent-notes / verify-translation-pairs.

## Risks

- **"Sidebar + green accent" drift toward the generic**: mitigated by three
  project-specific signals — paper-surface hierarchy (rail one step deeper
  than the paper), bark used only under the brand block, sprout used only as
  the "planned" marker. If user review still finds it weak, the fallback lever
  is strengthening bark's divider role.
- **Implementer "correcting" the dark ink button back to white**: documented
  in the design doc with the measured 2.3:1 rejection; mapping table states
  the ink color explicitly.
- **Custom theme files vs new keys**: optional-key fallback is unit-tested;
  behavior keeps the T-088 lenient-parse culture.
- **Color-emoji vs GDI monochrome panel cells**: mockup renders color system
  glyphs for layout only; the T-074 known limitation (monochrome outlines +
  pinyin alias labels) is restated in the mockup footnote.
- **Token drift with the website**: the bamboo family is shared; a website
  palette change must sync this design (same spirit as S-15, recorded in the
  design doc §12).
