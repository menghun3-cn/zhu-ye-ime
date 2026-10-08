# Agent Note: Candidate-window horizontal layout and pinyin typography

Status: implemented

## Problem

The user's visual pass over the shipped candidate window (T-122, 批七) reported two
readability issues.

1. Extra whitespace sits between the candidate index number and the candidate word.
   The word column used to start at `padding_x + marker_width` and the number was
   left-aligned inside that wide column, so single-digit indexes left ~19px of dead
   space before the word.
2. The pinyin/tone line above each candidate (`nǐ hǎo`) rendered too faintly: 11px
   SimSun at normal weight in the translation/index `secondary` grey (#999999 on
   light), with only 3px of breathing room below it. At small size this reads as
   blurry and cramped.

The user asked for a 4-8px gap between index and word (word pulled left), and a pinyin
line with higher contrast, a slightly larger size, a heavier weight, and more spacing
above the word.

## Decision

### Index-to-word spacing: right-aligned index + fixed gap

`CandidateMetrics` keeps `marker_width` as the word's left offset but shrinks it from
26dp to 22dp, adds `marker_text_gap` (6dp, inside the user's 4-8px band), and the
paint loop now draws the index right-aligned (`draw_text_right`) into the marker
column after **subtracting** `marker_text_gap` from its right edge. The gap is
therefore constant for every index width ("1" through "18"), no longer drifting with
digit count. Word start moves from x=38 to x=34 at 96dpi (padding 12 + 22).

### Pinyin line typography and dedicated color

`CandidateMetrics::pin_font_height` grows 11→13dp and `pin_line_gap` 3→4dp;
`create_font` takes a `weight` argument and the pin font is created with
`FW_SEMIBOLD` (600) while main text stays `FW_NORMAL`. The pinyin line no longer
reuses the translation/index `secondary` color: `CandidateUiTheme` gains an 8th key,
`pin` — #555555 (deep grey, ≈7:1 contrast on white, weaker than the blue main text)
for Light, #C9C9C9 (brighter grey) for Dark, and the system `gray_text` for
HighContrast. The theme-file palette (`CandidatePalette`/`parse_candidate`) gains an
optional 8th `pin` key; missing keys fall back to the defaults via the existing
`theme_with_candidate` overlay, so existing theme files keep working unchanged.

> T-126 (typography-whitespace note) later changed the shipped parameters in place:
> pin face SimSun → Segoe UI, pin weight FW_SEMIBOLD → FW_NORMAL, pin light color
> #555555 → #888888 (user now wants an auxiliary-light tier), `pin_line_gap` 4→8dp,
> `row_height` 36→48dp, and the main text weight FW_NORMAL → FW_SEMIBOLD (16px
> bold as the visual focus). The index right-alignment and the dedicated `pin`
> theme key decisions below are unchanged.

Both layout constants are covered by new unit tests (gap in 4..=8, word start left of
the old 26dp, pin size >12dp and still below main 16dp, `pin_band + font_height ≤
row_height`).

## Alternatives considered

- **Shrink the marker column instead of right-aligning.** Keeping the index
  left-aligned and cutting `marker_width` to ~13dp gives a nice gap for single
  digits but breaks two-digit indexes ("10".."18" on later pages), which would
  overlap the word. Right-alignment with a fixed gap keeps every page correct.
- **Darken the shared `secondary` color.** This would also darken translations,
  hints and the page footer, changing the whole "weak text" tier and contradicting
  the T-032 palette contract for those elements. A dedicated `pin` key (with theme
  file support) keeps the pinyin line the only element whose contrast changed.
- **Use `FW_BOLD` for the pin font.** SimSun has no true bold; GDI synthesises a
  heavy fake-bold that looks blotchy at 13px. `FW_SEMIBOLD` (600) reads clearly
  without the smear.
- **Raise `row_height` to give the pinyin line more room.** The pin band
  (13+4=17dp) plus the 16dp main glyph still fits inside the 36dp row; growing the
  row would inflate the whole panel without a readability gain.

## Consequences

- The gap between index and word is now visually stable at ~6px at 96dpi for every
  page, matching the user's 4-8px request; word start moves 4px left.
- Pinyin/tone text reads clearly at small size: deeper grey (or brighter grey on
  dark), 13px, semi-bold, with a 4px gap above the word. (As of T-126 the shipped
  pinyin line is instead Segoe UI 13px regular #888888 on light with an 8px gap,
  and the main text is semibold 16px — see the T-126 note.)
- Theme authors may optionally add `"pin"` to the `candidate` section of a theme
  file; old files are unaffected (key missing → default).
- `CandidateUiTheme` is now an 8-key structure; every construction site was updated
  (`theme()` ×3, `theme_from_system_colors`, `theme_with_candidate`) and the
  candidate-demo's test items now carry toned pinyin (`你 好` → `nǐ hǎo`) so
  screenshots exercise the pin line.

### Verification

- Pixel evidence from `candidate-demo --shot` BMPs at 96dpi light: first candidate
  word blue left edge x=38 → x=34; deep-grey (0x55-band) pixels 0 → 377 (pin line
  now renders) while the 0x99 shallow-grey tier still drives translation/index.
- Unit tests: `candidate_ui` 20/20 including the new gap/typography assertions;
  `theme_file` parse tests cover the `pin` key and its default (`None`) fallback.
- Full gate: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --workspace`, `git diff --check` all green; host-e2e re-run green.
- Landed as PR #128 (merge 3786e00 onto develop); release rebuild run
  37735245588 succeeded, upload refreshed v0.1.2 zip/assets (22,653,533 B zip).
- Locally built release DLL `zhu_ye_ime.dll` (AEE25CA1, 2,196,480 B, TSF exports
  verified) deployed to the user machine via the worker `copy-dll-ver`
  (CLSID InprocServer32 + profile IconFile pointer switch,
  `zhu_ye_ime_AEE25CA1.dll` in `C:\Program Files\zhu-ye-ime\tsf`, ctfmon
  restarted); `verify-tsf-identity` 7/7 PASS.
- VM (Server 2019) installed from a staging package (install.ps1 package
  contract, upgrade from f143c14e → `tsf\zhu-ye-ime-aee25ca1.dll`) and the same
  pixel probes re-run on the VM screenshots: word left edge x=34, index-to-word
  visual gap 8px (frame in the 4-8px band), deep-grey pin pixels present.
- Note: the release zip asset could not be downloaded from
  `release-assets.githubusercontent.com` (all reachable GitHub edge IPs reject
  that SNI from this network); the locally built DLL is the CI build's binary
  equivalent (same develop commit, same toolchain), and the dictionary trio SHAs
  are unchanged by this batch, so the staged package reused the previous
  v0.1.2 portable zip with only `bin\zhu_ye_ime.dll` swapped.

Related: [batch-4 candidate pinyin truncation and tone map](../../implemented/feature/2026-10-07-acceptance-fix-batch-4.md) remains active — this note changes the pin line's appearance parameters, not its truncation-by-row or tone-source semantics.
