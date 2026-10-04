# Agent Note: 格式候选：数字格式 / v 模式符号 / emoji 队尾追加 (T-061)

Status: implemented

[中文](2026-09-30-format-symbol-emoji-candidates.zh.md) | English

## Problem

User enhancement item ③: when typing raw digits, symbols, or emoji, the IME
offers no formatted alternatives — no date/amount/phone layouts for a digit
run, no symbol quick-entry, no emoji shortcut. The feature must coexist with
the existing candidate pipeline: it must not regress the T-057 hit-rate
baseline (Top1 84.7% / Top3 97.2% / sentence 21.0%) and must not disturb the
normal pinyin ranking path.

## Decision

Three independent candidate sources live behind the idle candidate window
(`suggestion > digit > v`, FR-029 D-05) and never join pinyin ranking.

**core** (`zhu-ye-core`): `format.rs` — `format_candidates` maps a digit run
to laid-out texts (8-digit date ×4, 6-digit year-month ×2, 4-digit year ×1,
amount: thousands separators + a Chinese reading via `cn_numeral`, 11-digit
phone ×2, ≥5-digit plain thousands separators; empty when too long/illegal);
`symbols.rs` — `symbol_group('1'..='9')` serial numbers ①-⑨, `x` math symbols
±×÷≈≠≤≥∞％, `h` punctuation ，。！？、；："" — one page of 9 each;
`emoji.rs` — a static 381-entry alias table sorted by alias bytes with binary
`emoji_for` lookup; `CandidateSource` gains `NumberFormat`/`Symbol`/`Emoji`.

**engine** (`input.rs`):
- `digit_buffer`: in the idle state a digit key enters digit mode and the
  character is immediately written to the document (digit-append accepts ASCII
  digits and `.`); fewer than 5 digits yields no candidates; while `digit_active`,
  Digit/Backspace/Space/Escape are all digit-mode semantics; any letter/pinyin
  input exits digit mode first (`push_composing` begins with `exit_digit`).
  `commit_digit` clears the buffer and sets `previous_word` to the formatted
  text so the following idle window shows bigram suggestions (D-05).
- `v_buffer`: idle-state `v` cold-starts v mode (`v_start` refuses when a
  composition, suggestion, digit mode, or v mode is already active — `v` is a
  legal nv/lv pinyin character); type codes via `v_code`; illegal letters
  (`vi`, `vv`) consume to pinyin via `v_consume`; `v_backspace` unwinds the
  code, Escape exits; in v mode digits select symbols (out-of-range passes
  through unscreened), Space selects symbol 0.
- emoji: when the full composing string equals an alias, a
  `Candidate{score: i64::MIN, source: Emoji}` is appended after the slang
  group — never ranked, coexists with real words.
- Candidate window: `candidate_ui_view` shows one source at a time —
  suggestion (empty pinyin), else digit (pinyin_hint = digit buffer), else v
  (pinyin_hint = code); pinyin candidates appear only when composing is
  non-empty.

**TSF** (`tsf.rs`):
- `KeyAction` grows 8 variants: `Dot`, `BufferDigit(char)`, `DigitBackspace`,
  `SelectAndReplace(usize)`, `VStart`, `VCode(char)`, `VConsume(char)`,
  `VBackspace`.
- `plan_action` opens with auto-exit guards: when `digit_active` and the key is
  not digit-mode-keeping, `exit_digit`; the same for v mode. Digit-mode keys:
  Space = select the current row (`SelectAndReplace`), Backspace =
  `DigitBackspace`, Escape exits; v-mode keys: Space = select symbol 0,
  Backspace = `VBackspace`, Escape exits; Enter/non-kept keys exit the mode and
  pass through. `plan_digit` dispatches digit keys across v mode / digit mode /
  suggestion / abbreviation-composition / digit mode.
- `BufferDigit` commits the single character directly through
  `finish_composition` (no composition); `SelectAndReplace` runs
  `replace_last_chars` — `GetSelection` → clone range → `ShiftStart(-len)` →
  `SetText` → `Collapse(TfAnchor(1))` → `SetSelection` with
  `TF_SELECTION { range: ManuallyDrop::new(Some(range)),
  style: TF_SELECTIONSTYLE { ase: TF_AE_END, fInterimChar: BOOL(0) } }` —
  replacing exactly the UTF-16 length of the digit buffer appended so far;
  `DigitBackspace` is `replace_last_chars(context, ec, 1, "")`;
  `VConsume(c)` opens a composition `"v{c}"` back into pinyin; `V*`/`Dot` are
  pure engine-state actions (sync + refresh).
- `.` maps from `VK_OEM_PERIOD`/`VK_DECIMAL` to `BufferDigit('.')` inside digit
  mode (amount decimals); outside digit mode it passes through to the host.

**host-e2e** `--m9`: 15/15 assertions against the acceptance-set cases 9.1-1/2/
4-11 (8-digit date layouts, replace length 8, no residue, <5 no candidates,
amount, phone, v1/vx/vh, vi fallback, emoji tail, no-alias no-append).

## Alternatives considered

**Buffer-first insertion (备选 A).** Digits accumulate in a buffer and one
inserted text is committed on selection. Rejected as the primary path: the
user-confirmed "边输边上屏" behavior matches mainstream IMEs and the TSF
replace chain (GetSelection → ShiftStart → SetText → Collapse → SetSelection)
runs entirely inside a single edit session, which the existing
commit-no-comp path already exercises. 备选 A stays recorded in the design doc
as the fallback if the VM replace chain ever fails in practice.

**Digit/emoji candidates join pinyin ranking.** Rejected: any digit candidate
in the ranked list would displace words and regress the T-057 benchmark;
emoji at i64::MIN guarantees observable-but-unranked coexistence.

**v mode activated in any state.** Rejected: `v` is a legal suffix of nv/lv
pinyin; only the idle (empty composition, no suggestion, no digit mode) state
intercepts it.

**v1..v9 as abbreviation keys entering the composition (u1s1-style).**
Rejected: conflicts with D-02's v-mode numbering; v-mode number semantics win.

## Consequences

- Digit and emoji candidates never enter any pinyin ranking path — T-057
  re-run after this change matches the baseline exactly (Top1 84.7% / Top3
  97.2% / sentence 21.0%).
- Committing a formatted digit text sets `previous_word`, so the idle window
  switches to bigram suggestions for that text (suggestion > digit priority).
- Inside digit mode the `.` key is consumed as a decimal point; keys outside
  digit mode pass through untouched — a user cannot end a sentence with `.`
  while digit mode is active (exit via Escape/letter first).
- Digit keys in a composition keep their meaning unchanged (including
  digit-abbreviation keys such as 996/u1s1).
- The emoji table ships with 381 entries in static binary-search order; T-062
  expanded it from the initial 109 in a pure-data batch (aliases stay lowercase,
  ordering enforced by test).
- The shui candidate-coverage item (option A) stays pending — no overlap with
  scenario 7.

## Deferred

VM interactive acceptance (acceptance criteria 9.1, vm-accept-sop): the engine
side of every case is asserted by host-e2e `--m9`; the on-VM replace-chain and
composition checks run after this PR lands and the results are backfilled into
acceptance criteria §9.5.

## Related

- Design: [docs/格式候选设计.md](../../../docs/格式候选设计.md) (T-060)
- Requirements: [docs/需求规格说明书.md §13](../../../docs/需求规格说明书.md) (T-060)
- Acceptance: [docs/验收标准.md §9](../../../docs/验收标准.md) (T-060)
- Tasks: [docs/todos-list.md](../../../docs/todos-list.md) T-061 / T-062
