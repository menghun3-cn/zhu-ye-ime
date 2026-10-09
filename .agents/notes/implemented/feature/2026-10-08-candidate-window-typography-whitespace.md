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

- `pin_line_gap` 4→8dp for the first shipped cut (top of the user's 4-8px band),
  then **→4dp** after the user eyed the result and asked "拼音和中文距离减少一半"
  (see the follow-up section below).
- `row_height` 36→48dp with `row_gap` 2→4dp: the main zone
  (48 − 17 = 31dp with the final 4dp gap) holds a 16dp glyph with headroom below —
  the candidate row breathes.
- `padding_y` 10→12dp (panel top/bottom air) and `header_height` 38→48dp (the top
  composition buffer: a 16dp composition glyph is vertically centred with ≈16dp of
  padding above and below).
- Nine-row panel height 398→536dp (unchanged by the gap shrink: row height owns it).

### Typography (candidate_window.rs)

- Main text font: SimSun 16dp `FW_NORMAL` → `FW_SEMIBOLD` (16px, bold weight —
  the hanzi tier becomes the visual focus; unselected #1E88E5 blue / selected
  #D32F2F red unchanged). SimSun has no true bold so GDI synthesises it; at 16px
  it reads cleanly, but the synthetic bold also extends the ink horizontally —
  see the truncation regression fix below.
- Pinyin font: SimSun 13dp `FW_SEMIBOLD` → **Segoe UI 13dp `FW_NORMAL`**.
  `create_font` gains a `face` parameter. Segoe UI renders the precomposed tonal
  characters (ǐ U+01D0, ǎ U+01CE, etc.) with well-formed single glyphs that keep the
  tone mark on the vowel, fixing the fragmented/misplaced marks seen with SimSun.

### Color (candidate_ui.rs `theme`)

- Light `pin` #555555 → **#888888** (user-specified auxiliary grey; ≈3.4:1 on
  white, deliberately weaker than translations/index #999999). Dark #C9C9C9 and
  HighContrast gray_text unchanged. This deliberately reverses T-122's darkening
  decision — the user now explicitly wants the pinyin as a light auxiliary hint
  rather than a high-contrast detail; readability is carried by the Segoe UI face.

### Alignment (candidate_ui.rs pure function + paint)

- `CandidateMetrics::pin_row_rect(row, main_col, pin)` returns the pinyin rect:
  `left = main_col.left` (index column, then pinyin and hanzi share one left
  edge — strict vertical alignment), top = row top, bottom = top + pin font
  height (**pure glyph band, gap NOT included** — see follow-up), and
  `right = max(main_col.right, main_col.left + estimate_text_width(pin, 13dp))`
  clamped to the row's right edge. Short pinyin matches the hanzi column width
  exactly; long pinyin (e.g. `zhang hao`) is fully accommodated without ellipsis.
- `paint` draws the pinyin into that rect, then starts the main/translation rects
  at `pin_rect.bottom + pin_line_gap` with height = `font_height`. The index is
  right-aligned into the marker column (T-122) and, when a pin line is shown,
  drops with the main band so the number and the hanzi share one vertical centre.

## Alternatives considered

- **Keep SimSun for pinyin and rely on combining marks**: SimSun's glyphs for the
  tonal characters are what produced the reported fragmentation; combining
  sequences are worse. Segoe UI (present on every supported Windows) is the
  low-risk fix; no font file ships with the product.
- **Set the main weight to FW_BOLD**: unnecessary weight at 16px; SEMIBOLD reads
  as clearly bold while keeping ClearType render smooth.
- **Shrink pinyin to 12dp**: user allowed 12 or 13; 13 keeps the tone marks legible.
- **Pin the pinyin right edge to the hanzi column unconditionally**: would
  ellipsize `zhang hao`-style corrections wider than the hanzi; the max(widths)
  rule keeps alignment for the common case and completeness for the long case.
- **Keep the gap inside the pin band (original band-bottom = top + font + gap)**:
  `draw_text` vertically centres text in the rect, so a band taller than the
  glyph absorbs the gap as centring padding — a frame gap of 8 vs 4 then shows
  identical ~9.5px ink spacing. The glyph-band rects (bottom = top + font,
  main top = bottom + gap) make the *visible* gap exactly frame-driven.

## Consequences

- Candidate rows visibly breathe (48dp rows, 4dp pin gap → ≈6px visible, 4dp row
  gap); the header buffer is airy (48dp with centred composition); the pinyin
  line reads as a light auxiliary tier (#888 Segoe UI 13px); hanzi carry the
  visual weight (SimSun 16px semibold, blue/red); the index number sits on the
  same vertical centre as the hanzi.
- Panel grows 398→536dp tall (9 rows); width logic unchanged (T-124
  content-driven width untouched).

## Verification

- Unit tests (`candidate_ui.rs`): `pin_line_gap` exactly 4 (inside the 4-8px
  band); pinyin glyph band + main zone fit the 48dp row; `pin_row_rect` left edge
  equals the hanzi column left edge, short pinyin spans exactly the hanzi column
  width, `nǐ hǎo` (wider than 你好) extends to its estimated width without
  clipping; main-zone top = glyph bottom + gap; light theme `pin == #888888`.
  `candidate_ui` 24/24 and `zhu-ye-ui` 35/35; workspace suites green
  (core 326, ime 215, settings 103, …); fmt/clippy -D warnings/diff-check pass.
- Pixel probe on `candidate-demo --shot` BMPs (96dpi light, final parameters):
  row pitch 52px; header `nihao` blue span 53px complete at band y31..41
  (centre 36/48); row1 blue hanzi span 50px (three chars complete), row0 red
  span 33px (two chars); index red y-centre 85 vs hanzi 84 (same band); pin
  (Segoe UI) ink y117..123 → hanzi ink y129..144 = **visible gap 6px**;
  pinyin grey in the 110-158 range (no #555 deep grey); blue #1E88E5 / red
  #D32F2F matched exactly.
- 125% (user machine scale) shot: 450×695px window, identical rendering.
- Landed as PR #136; release `zhu_ye_ime.dll` (2,198,016 B, sha8 FEAEF72E, TSF
  identity 7/7) deployed to the user machine via `copy-dll-ver`
  (`zhu_ye_ime_FEAEF72E.dll`, CLSID/IconFile pointer switch, ctfmon restarted).

## User eyeball follow-up (same day, after first deploy)

The user typed `nihao` on the real machine and saw the candidate word truncated to
`你...`. Root cause: T-126 made the main text `FW_SEMIBOLD`; SimSun has no true
bold, GDI's synthetic bold expands the ink horizontally (~1px/char), while
`estimate_text_width` sized the rect at the exact normal advance width — so
`DT_END_ELLIPSIS` replaced the last glyph with an ellipsis. Fix: `estimate_text_width`
now uses conservative coefficients (ASCII 0.55→0.58em, CJK 1.0→1.06em, same change
in `fit_text`) so every consumer rect (row split, header, pin) fits the real
rendered ink. The header composition `nihao` had the same latent issue and is now
verified complete (53px span, no ellipsis).

The user also asked two adjustments:

1. "序号和中文那行对齐，不要和拼音对齐" — the index number now drops with the
   main-text band (`marker_rect.top/bottom = main rect` when `show_pin`) so the
   number and the hanzi share one vertical centre; without a pin line the marker
   stays row-centred as before.
2. "拼音和中文距离减少一半" — `pin_line_gap` 8→4 and `pin_row_rect` redefined as
   the **pure glyph band** (bottom = top + pin font height, gap not included)
   with the main-text rect top = glyph bottom + `pin_line_gap`. As noted under
   Alternatives, this is what makes the change *visible*: measured ink gap 6px
   (Segoe UI keeps ~2px above the 13px cell) vs 9.5px before — roughly halved
   and inside the user's original 4-8px band. Follow-up change landed together
   with the truncation fix in one branch/PR (fix/T-126 follow-up).

Related: [candidate-window horizontal layout and pinyin typography](../../implemented/feature/2026-10-08-candidate-window-layout-pinyin-typography.md)
(T-122) stays active — this note updates its pinyin parameters (weight/color/gap/
face, main text bold, row height) in place; the index right-alignment and dedicated
`pin` theme key decisions are unchanged.

## Pinyin glyph band uses real tmHeight (T-130, user report "拼音下方被截断")

User report (2026-10-09): pinyin under some words was cut off at the bottom.
Root cause: `pin_row_rect` sizes the pin band at `pin_font_height` (13px), but
that is the **character height** (`lfHeight = -13`); Segoe UI 13px has a true
`tmHeight` ≈ 16px. `DrawTextW` clips at the rect, so the descender (~3px) of
letters like `g`/`j`/`p`/`q`/`y` was shaved off while the delimited row is
still positioned from the 13px band.

Fix (kept inside `candidate_window.rs`, layout constants unchanged): after
creating the pin font, `query_font_tm_height` reads `GetTextMetricsW`"s
`tmHeight` via a scratch compatible DC and the window state stores
`pin_tm_height`. In `paint`, the pin **draw rect** and the derived main-text /
marker tops all extend by `pin_tm_height - pin_font_height`, so the glyph band
is the real tmHeight and the visual ink gap stays `pin_line_gap` (4px).

Pixel proof (@96dpi light, `gāo fēng` demo row): before the fix the pin ink ran
y=234..248 (clipped flat at the 13px line); after the fix it runs y=237..252
(the `g` descender is intact) with the same 4px gap to the main text below.

## Candidate-row content band is vertically centred (T-132, user report "选项行内部下留白过多")

User report (2026-10-09, right after T-131 deploy): each candidate row had too
much whitespace below the content, looking loose.

Root cause: rows with pinyin anchored the content band at the row **top** —
pin glyph band (13px + ~3px tmHeight extension) then `pin_line_gap` (4px) then
the 16px main-text band = 36px inside the 48px row — leaving ~12px of dead
space at the bottom (plus `row_gap` 4px), which read as "松散". Rows without
pinyin were already vertically centred (`DT_VCENTER`).

Fix (T-132, `candidate_window.rs` paint only; panel size / pagination
unchanged): new pure `CandidateMetrics::pin_band_top(row, pin_tm_height)`
returns the content-band top so the whole band (pin band + gap + main band) is
centred in the row; the pin draw rect, main/translation rects and the
row-number rect (which follows the main band per T-126 "序号跟中文对齐") all
derive from that top. With 96dpi constants the band sits 6px/6px top/bottom
instead of 0px/12px.

Pixel proof (@96dpi light, demo row with pinyin `nǐ men hǎo`): before, ink ran
y=116..148 inside row 112..160 (top pad ≈4px, bottom ≈11px); after, ink runs
y=121..153 (top pad ≈9px — pin glyph top inset — bottom ≈6px), i.e. the band
moved down ~5px and the row now balances. Non-pinyin rows (translation mode,
pin toggle off) are untouched (16px/16px, already centred).

## Header band compressed 30% (T-136, user report "候选框顶部留白过多，视觉重心偏下")

User (2026-10-09): the candidate window's top area has too much whitespace
and the visual center of gravity sits low; compress the top pinyin band by
30% and tuck the first candidate row right under it (less vertical margin).

Fix (T-136): `CandidateMetrics::new` `header_height` 48 → **34** dp
(48 × 0.7 = 33.6 → 34; a 16px header glyph with ≈9px space above and
below). `row_rect` was already `top = padding_y + header_height` — the
header bottom and first-row top share one edge with no explicit margin —
so shrinking the header moves the whole candidate stack up: first row top
60 → 46, nine-row panel 536 → 522 (−14 dp). Row height and the row-inner
centred content band (T-132) and `padding_y` are untouched (not called out
by the user). Verified: candidate_ui tests green with the panel-size
assertion adjusted 536 → 522, workspace all-green; 96dpi demo pixel probe —
header glyph y29..34 (vertically centred in the 34 band), first-row ink
top 72 → 58 (up 14 px), panel height 542 (522 + 20 footer).

## Show-pin toggle (T-127)

User request: "设置里面可以设置候选框是否显示拼音及声调" — a settings toggle
for the candidate-window pinyin (with tone marks) line.

Decision and shape (followed the FR-023/FR-024 switch precedent, T-103):

- **Config format**: `ConfigFile.candidate_show_pin: bool`, default `true`
  (current behaviour preserved; old configs without the field load as on —
  lenient parse, no `CONFIG_FORMAT_VERSION` bump, same pattern as
  `enable_abbreviation`/`enable_fuzzy`).
- **Assembly item, same contract as theme**: the TSF side reads the flag once at
  assembly (`configured_candidate_window`), so the setting takes effect after
  restarting the input method — the settings window says so instead of faking
  an instant effect.
- **Wiring**: `CandidateWindow`/`CandidateWindowOptions`/`CandidateWindowState`
  all carry `show_pin`; the paint short-circuit is
  `self.show_pin && !pin_text.is_empty() && pin_text != base`. When off, the row
  falls back to the existing no-pin paint path (main text and index number
  full-row vertically centred) — no new layout math, translation mode pin
  suppression and long-pinyin behaviour untouched. New constructors
  `with_theme_and_show_pin` / `with_custom_theme_and_show_pin`; the old ones
  keep default on. Demo gets `--no-pin` for screenshot verification.
- **Settings UI**: new 「候选拼音」 row on the 常用设置 page (between
  自定义主题 and 英文输入法), 关闭/显示 two-chip control reusing the
  theme/mode/online-update chip machinery (`ItemControl::CandidatePin`,
  `ChipValue::CandidatePin`, `candidate_pin_chips`); `load_/save_candidate_show_pin`
  follow the S-8 re-read-before-save rule like every other single-field writer.

Verified: unit tests across core (serde default-on + round trip), settings
(state load, chips geometry, persistence preserve-fields), ime suite; pixel
probe on 96dpi demo shots — `--no-pin` shows 0 grey pixels in the pin band
(y117–127) vs 25 with pinyin on, main-text band intact in both.

## Deployment-channel note (process, batteries)

The user machine deploy channel is the scheduled task `ZhuYeImeElevated`
(`InteractiveToken` + `RunLevel HighestAvailable` +
`DisallowStartIfOnBatteries=true`). On a laptop running on battery that task stays
`Queued` forever — verified twice (21:53 and 21:58) with Last Result 0 and an
unchanged log. Workaround used: run
`Start-Process powershell -Verb RunAs -File <tsf>\elevated-worker.ps1` (one UAC
confirmation) — the worker's single-shot request/response model works with a plain
elevated process, no scheduled task needed. Recreating the task via
`schtasks /Create /XML` to drop the battery restriction is denied at Medium IL, so
the UAC path (or plugging in AC power) is the alternative. Memory for next
deploys.
