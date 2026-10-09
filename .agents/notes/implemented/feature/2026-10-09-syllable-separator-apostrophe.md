# Agent Note: Syllable separator `'` (auto boundaries + manual override for ambiguous pinyin)

Status: implemented

[中文](2026-10-09-syllable-separator-apostrophe.zh.md) | English

## Problem

The raw pinyin string (e.g. `nihao`) is shown verbatim above the candidate list,
with no way to express syllable boundaries. Ambiguous keys such as `xian`
(`xian` or `xi an`) could not be disambiguated by the user, and long strings
were hard to read (`youmeiyoushenmeren`). The candidate window header merged
everything into one unbroken run. T-021 deliberately declined multi-syllable
combination candidates for whole-word-hit keys and left "an explicit `xi an`
separator" as a possible future enhancement — this note is that enhancement.

User decisions (2026-10-09, four clarifying questions):

1. **Separator goes in the composition** (`sep_in_composing`, Recommended):
   the header shows `xi'an`; backspace deletes the leading `'` first.
2. **Query normalization + boundary constraint combined** (`query_contract`,
   Recommended): every query treats `ni'hao ≡ nihao ≡ ni hao` (separator is
   noise for the dictionary key) **and** the separator constrains syllable
   boundaries for candidate generation.
3. **Auto separator on every syllable boundary** (`auto_scope`, user chose the
   non-recommended option): every boundary of the preferred segmentation
   receives `'` (e.g. `ni'hao`), so a preferred-segmentation rule is required.
4. **Idle `'` passes through to the host** (`idle_apostrophe`, Recommended):
   halfwidth `'` types normally when no composition is active.

## Decision

- **Query key stays plain**: `InputEngine::composing` never contains `'`.
  Manual separators live in a new `manual_seps: Vec<usize>` field (character
  boundary offsets in the plain key, sorted). `insert_separator()` appends a
  separator at the current buffer end (no cursor model exists), rejecting a
  duplicate at the same position. Backspace first removes a trailing manual
  separator before popping a letter, and auto boundaries recompute on every
  insert/delete.
- **Display string**: `composing_display()` inserts `'` at the union of
  manual separators and auto boundaries. Auto boundaries come from the
  `segment_all` first plan (preferred segmentation, longest-match-first) via
  `preferred_segment_boundaries` in the input engine; a trailing manual `'` is
  drawn explicitly. Auto insertion applies only in Chinese mode (English
  composition stays verbatim). `candidate_ui_view().composition` now uses the
  display string, so the candidate window header shows `ni'hao`/`xi'an`.
- **Core constrained segmentation**: `pinyin::segment_constrained(table,
  input, hard)` filters `segment_all` plans to those whose syllable-end
  positions include every hard boundary (boundary `0` trivially holds; any
  unsatisfiable boundary empties the result).
- **Constrained candidate group**: `candidate::constrained_segment_candidates`
  takes the **first** plan satisfying the hard boundaries and builds a single
  combined candidate (highest-frequency word per syllable, same rule as
  `generate_candidates`). Only the first plan is used so multi-plan noise
  (`nihao` also segments as `ni,ha,o`) cannot leak in. The input engine
  prepends this group via the existing `prepend_group` after all other groups
  are assembled, so a manual separator expresses strong intent (`xi'an` →
  西安 on top) without touching the ordinary pipeline when no manual separator
  exists (T-050 baseline: zero drift without manual `'`).
- **Key routing**: `classify_key` maps `VK_OEM_7` without Shift to
  `KeyAction::Separator` (Shift+`'` = `"` passes through). `plan_action`
  accepts it only in Chinese mode with an active composition; idle leaves it to
  the host (halfwidth `'`). `sync_engine` routes it to `insert_separator`.
- **Query normalization is structural**: because `'` never enters `composing`,
  `ni'hao` and `nihao` share one dictionary key; the separator only constrains
  segmentation and display.

## Alternatives considered

- **Store the separator inside the composition string** (display string = query
  key): every dictionary lookup and candidate path would have to strip `'`,
  and backspace/cursor semantics would need a character-classification pass.
  Rejected: the plain-key + side-channel design keeps all existing queries
  untouched.
- **Constrained group generated for every plan**: user-typed `ni'hao` would
  emit the `ni,ha,o` combination (`你哈哦`) alongside `你好`. Rejected: the
  first-plan rule matches the displayed boundaries and is noise-free.
- **Separator also blocks whole-word hits** (e.g. `xi'an` must never show 先):
  T-021 keeps whole-word hits first for exact keys; removing them behind a
  separator would change page composition unpredictably. Rejected: the
  constrained group sits on top instead, so the user still gets 西安 first with
  normal candidates retained.
- **A cursor model with mid-string separator insertion**: TSF has no
  composition cursor today; the user-confirmed scope (append + trailing
  backspace priority) works without one. Deferred.

## Consequences

- Ambiguous keys are now visibly and functionally controllable: `xian` shows
  plain (preferred `[xian]`), and typing `xi` + `'` + `an` shows `xi'an` with
  西安 promoted first.
- Header width grows slightly (`ni'hao` is one glyph wider than `nihao`);
  T-124 header auto-width handles it (no truncation).
- `Enter`/raw commit still emits the plain pinyin (`xian`), never the `'` —
  the separator is a display/query device, not a commit character.
- Prefer-first plan means auto `'` follows longest-match-first, so
  `zhuan` stays `zhuan` (not `zhu'an`) unless the user inserts a manual `'`.

## Testing

- `segment_constrained`: hard boundary `[2]` keeps only `[xi,an]` for `xian`;
  unsatisfiable/out-of-range boundaries empty the result; `0`/`len` boundaries
  are no-ops; boundary-0 no-constraint equivalence is asserted.
- `constrained_segment_candidates`: `xi'an` → 西安 single candidate; multi-plan
  inputs yield only the preferred plan (`nihao` → 你好, no `你哈哦`);
  single-syllable or unsegmentable cases empty.
- Engine: display strings (`ni'hao`, `xian`, `xi'huan`), trailing `'` deletion
  priority, auto resegmentation on backspace, manual-separator-preservation
  across deletes, constrained-group-first candidate order, view composition
  (`xi'an`), and baseline equality without manual separators.
- TSF: `classify_key` (`VK_OEM_7` → Separator, Shift → None) and `plan_action`
  (composing enters, idle/English releases).

## Display glyph revision (T-134, user report "分隔符看起来是个逗号在上面")

User (2026-10-09): the on-screen separator "looks like a comma at the top
of the line" — it should read as a plain apostrophe like Sogou's, i.e. the
**English-style** apostrophe. Root cause: T-128 displayed the keyboard
straight apostrophe U+0027, but the header composing string is rendered in
**SimSun bold** (candidate_window `create_font(…, "SimSun")`, FW_SEMIBOLD);
SimSun's U+0027 glyph is a hook-topped vertical line, which reads as a
floating comma at bold weights. Sogou/Microsoft Pinyin use an English curly
apostrophe.

Fix (T-134): new `pub const SYLLABLE_SEP_DISPLAY: char = '\u{2019}'` in
`input.rs` (RIGHT SINGLE QUOTATION MARK, the typographically correct English
apostrophe); `composing_display` pushes it on both paths. Display-only — the
query key `composing` stays separator-free, `manual_seps` stay character
indices, key semantics unchanged. Bonus fix: `preview_after_backspace`
removed the trailing separator by byte slicing `[..len-1]`, which would have
split a 3-byte U+2019 mid-character (potential panic); now `pop()`s by char.
Tests assert `\u{2019}` escapes. Candidate-row pinyin (Segoe UI, dictionary
data ASCII `'` for erhua `nǎ'er`) is untouched — its glyph reads fine.

## Header separator drawn as a straight apostrophe (T-139, user: "looks like a comma on top" — round 2)

T-134 switched the display glyph to U+2019, which under SimSun (the header font)
still renders as a hefty "top comma" (round-bulb head + hook). Pixel evidence at
96dpi: the SimSun U+2019 glyph is 4px wide × 5px tall with a round head
(12 dark px); the Segoe UI U+0027 straight apostrophe is a 2px-wide slim
vertical sliver (6 px). User requested a real "apostrophe" look.

Fix (T-139): the header renderer now draws per-segment. `draw_header_mixed`
splits the display string at `SYLLABLE_SEP_DISPLAY` (U+2019) and draws every
segment with its own font on a shared baseline: main segments keep SimSun
SEMIBOLD (`main_font`), and each separator segment is drawn as the ASCII
straight apostrophe `'` (`SYLLABLE_SEP_APOSTROPHE: char = '\u{0027}'`) in a
Segoe UI REGULAR font (`sep_font`, created at the header font size). Pure
display layer — `composing` / `manual_seps` / key semantics unchanged.
Helpers: `draw_seg` (SelectObject the segment font, then DrawTextW in a
per-segment rect: top = baseline − that font's ascent, height = tmHeight) and
`text_extent` (GetTextExtentPoint32W on the selected font for advance width).
The separator rect gets an +8px right pad: `GetTextExtentPoint32W` returns the
advance (which for `'` sits left of the ink), and a tight right edge clipped
the glyph body to a 1–3 px fringe. Verified on the demo: `xi'an` renders
`x`(SimSun) `'`(Segoe UI straight, sitting above x-height) `a n`(SimSun) with
unchanged spacing; light/dark themes both clean. Not applied to candidate-row
pinyin (`nǎ'er`) — that is already fine.

## Related

- [full pinyin segmentation core (T-007)](../../implemented/feature/2026-09-19-full-pinyin-segmentation-core.md) — `segment_all` and the first-plan rule this design builds on.
- [candidate ranking static model / T-021 noise cleanup](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md) — the "explicit `xi an` separator" enhancement this note implements; whole-word-first stays intact.
- [candidate window header content width (T-124)](../../implemented/feature/2026-10-08-candidate-window-header-content-width.md) — header renders the display string.
