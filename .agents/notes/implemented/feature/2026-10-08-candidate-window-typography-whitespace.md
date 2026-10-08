# Agent Note: Candidate-window typography and whitespace polish (T-126)

Status: implemented

## Problem

User feedback on the candidate window (T-126): the pinyin/tone line (`nǐ hǎo`) sat
almost on top of the hanzi (`你好`) with no breathing room; tonal marks (ǐ/ǎ)
rendered misaligned and fragmented (font issue); the pinyin line was too large and
too dark to read as an auxiliary hint; index/pinyin/hanzi alignment was loose; and
the header composition buffer (`nihao` + grey `ni hao`) felt cramped. The requested
spec (user-supplied CSS-style parameters, mapped onto the GDI self-painted window):

1. pinyin↔hanzi vertical gap ≈4-8px plus candidate-row vertical padding;
2. a better font face for the pinyin line so tone marks sit tightly on vowels
   (user suggested `"Segoe UI", "Helvetica Neue", "Pinyin", Arial, sans-serif`);
3. pinyin smaller (12/13px) and lighter (≈#888) as auxiliary; hanzi larger/bolder
   (16-18px) with high-contrast red/blue as the visual focus;
4. index/pinyin/hanzi aligned on one horizontal line; pinyin width aligned with the
   hanzi column (or centred) so pinyin does not drift;
5. more air and vertical centring/padding in the top composition buffer.

## Decision

### Layout (candidate_ui.rs `CandidateMetrics`)

- `pin_line_gap` 4→8dp (top of the user's 4-8px band; the pinyin line's
  `margin-bottom`). The pin band (`pin_font_height 13 + gap 8 = 21dp`) starts at the
  row top; the main text block moves below it, so the frame-level pinyin↔hanzi gap
  is exactly 8dp and the glyph-level visual gap lands at ≈6px after anti-aliasing.
- `row_height` 36→48dp with `row_gap` 2→4dp: the main zone
  (48 − 21 = 27dp) centres a 16dp glyph with ≈5-6dp of padding above and below —
  the candidate row breathes.
- `padding_y` 10→12dp (panel top/bottom air) and `header_height` 38→48dp (the top
  composition buffer: a 16dp composition glyph is vertically centred with ≈16dp of
  horizontal-rhythm padding above and below).
- Nine-row panel height 398→536dp.

### Typography (candidate_window.rs)

- Main text font: SimSun 16dp `FW_NORMAL` → `FW_SEMIBOLD` (16px, bold weight —
  the hanzi tier becomes the visual focus; unselected #1E88E5 blue / selected
  #D32F2F red unchanged). SimSun has no true bold so GDI synthesises it, which at
  16px reads cleanly (the 13px blotchy-fake-bold concern from T-122 does not apply
  at the main size).
- Pinyin font: SimSun 13dp `FW_SEMIBOLD` → **Segoe UI 13dp `FW_NORMAL`**.
  `create_font` gains a `face` parameter. Segoe UI renders the precomposed tonal
  characters (ǐ U+01D0, ǎ U+01CE, etc.) with well-formed single glyphs that keep the
  tone mark on the vowel, fixing the fragmented/misplaced marks seen with SimSun.
- `create_font(metrics.font_height, FW_SEMIBOLD, "SimSun")` and
  `create_font(metrics.pin_font_height, FW_NORMAL, "Segoe UI")` are used at both
  construction (`new_with_quit`) and `apply_dpi`.

### Color (candidate_ui.rs `theme`)

- Light `pin` #555555 → **#888888** (user-specified auxiliary grey; ≈3.4:1 on
  white, deliberately weaker than translations/index #999999). Dark #C9C9C9 and
  HighContrast gray_text unchanged. This deliberately reverses T-122's darkening
  decision — the user now explicitly wants the pinyin as a light auxiliary hint
  rather than a high-contrast detail; readability is carried by the Segoe UI face.

### Alignment (candidate_ui.rs new pure function)

- `CandidateMetrics::pin_row_rect(row, main_col, pin)` returns the pinyin rect:
  `left = main_col.left` (index column, then pinyin and hanzi share one left edge —
  strict vertical alignment), top = row top, bottom = top + pin band, and
  `right = max(main_col.right, main_col.left + estimate_text_width(pin, 13dp))`
  clamped to the row's right edge. Short pinyin therefore matches the hanzi column
  width exactly, and long pinyin (e.g. `zhang hao`) is fully accommodated without
  ellipsis — the pinyin never drifts.
- `paint` uses `pin_row_rect` and starts the main/translation rects at
  `pin_rect.bottom`. The index is still right-aligned into the marker column
  (T-122) and vertically centred on the row; with the taller row the number now
  sits beside the whole pinyin+hanzi block rather than just the hanzi.

## Alternatives considered

- **Keep SimSun for pinyin and rely on combining marks**: SimSun's glyphs for the
  tonal characters are what produced the reported fragmentation; combining
  sequences are worse. Segoe UI (present on every supported Windows) is the
  low-risk fix; no font file ships with the product.
- **Set the main weight to FW_BOLD**: unnecessary weight at 16px; SEMIBOLD reads
  as clearly bold while keeping ClearType render smooth.
- **Shrink pinyin to 12dp**: user allowed 12 or 13; 13 matches T-122's readability
  gain and keeps the tone marks legible.
- **Pin the pinyin right edge to the hanzi column unconditionally**: would
  ellipsize `zhang hao`-style corrections that are wider than the hanzi; the
  max(widths) rule keeps alignment for the common case and completeness for the
  long case.
- **Shrink the panel on short input / add a scroll bar**: out of scope; the 9-row
  × 48dp panel already fits the 700px-high settings-era workflow and item clipping
  is pre-existing (T-124 header content-width only).

## Consequences

- Candidate rows visibly breathe (48dp rows, 8dp pinyin gap, 4dp row gap); the
  header buffer is airy (48dp with centred composition); the pinyin line reads as
  a light auxiliary tier (#888 Segoe UI 13px); hanzi carry the visual weight
  (SimSun 16px semibold, blue/red).
- Panel grows 398→536dp tall (9 rows); width logic unchanged (T-124
  content-driven width untouched).
- `pin_row_rect` is pure and unit-tested; `create_font`'s face parameter is a
  Win32-side implementation detail covered by pixel evidence.

## Verification

- Unit tests (`candidate_ui.rs`): `pin_line_gap` exactly 8 (4-8px band top);
  pinyin band + main glyph still fit the 48dp row with ≥4px breathing; `pin_row_rect`
  left edge equals the hanzi column left edge, short pinyin spans exactly the hanzi
  column width, `nǐ hǎo` (wider than 你好) extends to its estimated width without
  clipping, never past the row right edge; light theme `pin == #888888`.
  `candidate_ui` 24/24; workspace suites green (core 326, ime 215, settings 103, …);
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `git diff --check` pass.
- Pixel probe on `candidate-demo --shot` BMPs (96dpi light):
  row pitch 52px (48 row + 4 gap); header composition band y=32..41, centre y≈36.5
  of the 48px header; pin→hanzi frame gap exactly 8px with a blank run y=128..132
  at row 1; red selected glyph band 15px tall (semibold weight visible); hanzi
  left edge x=34 matches pinyin left edge; pinyin grey pixels in the 110-158 range
  (no #555 deep grey); blue #1E88E5 / red #D32F2F matched exactly.
- 125% (user machine scale) shot: 450×695px window, identical rendering.
- Landed as PR #136; release `zhu_ye_ime.dll` (2,198,016 B, sha8 FEAEF72E, TSF
  identity 7/7) deployed to the user machine via `copy-dll-ver`
  (`zhu_ye_ime_FEAEF72E.dll`, CLSID/IconFile pointer switch, ctfmon restarted).

Related: [candidate-window horizontal layout and pinyin typography](../../implemented/feature/2026-10-08-candidate-window-layout-pinyin-typography.md)
(T-122) stays active — this note updates its pinyin parameters (weight/color/gap/
face, main text bold, row height) in place; the index right-alignment and dedicated
`pin` theme key decisions are unchanged.

## Deployment-channel note (process, batteries)

The user machine deploy channel is the scheduled task `ZhuYeImeElevated`
(`InteractiveToken` + `RunLevel HighestAvailable` +
`DisallowStartIfOnBatteries=true`). On a laptop running on battery that task stays
`Queued` forever — verified twice (21:53 and 21:58) with Last Result 0 and an
unchanged log. Workaround used for T-126: run
`Start-Process powershell -Verb RunAs -File <tsf>\elevated-worker.ps1` (one UAC
confirmation) — the worker's single-shot request/response model works with a plain
elevated process, no scheduled task needed. Recreating the task via
`schtasks /Create /XML` to drop the battery restriction is denied at Medium IL, so
the UAC path (or plugging in AC power) is the alternative. Memory for next
deploys.
