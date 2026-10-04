# Agent Note: Digit keys reachable for digit-bearing abbreviation keys

Status: implemented

[中文](2026-09-29-digit-abbreviation-key.zh.md) | English

## Problem

The slang pack (T-045) ships abbreviation keys that carry ASCII digits — `u1s1`,
`996`, `520`, `1314`, `886`, `88`, `233`, `666`, `555`, `7456`. The keys are
correct in the package: querying `slang.zyct` directly returns every one of them
with its word and frequency. They were unreachable from the keyboard.

Two independent causes, both in the key path:

1. `InputEngine::handle_letter` accepted only `c.is_ascii_lowercase()`, so a digit
   arriving from the TSF adapter was dropped without entering the composition.
2. `classify_key` bound `VK_1..VK_9` directly to `KeyAction::Select`, because
   digits 1-9 select candidates (FR-006, accepted in T-040). A digit key therefore
   never had a chance to mean "part of an abbreviation".

VM acceptance (T-048) confirmed the split: pure-letter abbreviation keys worked
(`yyds` → 永远的神, `xswl` → 笑死我了, `zqsg` → 真情实感, `dbq` → 对不起,
`yysy` → 有一说一, `awsl` → 啊我死了, `gkd` → 搞快点), while all ten
digit-bearing keys produced no candidate at all.

The design constraint is that the two meanings cannot be separated per keystroke:
when the user presses the first `9` of `996`, the IME has no way to know whether
that is a literal digit, the 9th candidate selection, or the start of `996`.

## Decision

Digit keys keep both meanings, disambiguated by a dictionary lookup.

`InputEngine` gains two methods:

- `handle_digit(c)` — pushes an ASCII digit into the composition; rejects English
  mode and non-digits, mirroring `handle_letter`.
- `is_abbreviation_prefix(text)` — true when the dictionary holds any entry whose
  pinyin key starts with `text` **and** contains an ASCII digit. The digit test is
  what keeps ordinary pinyin entries from being mistaken for abbreviation keys, so
  `n`/`ni` never divert a digit away from candidate selection.

`classify_key` now maps `VK_0..VK_9` to `KeyAction::Digit(char)`, making it a pure
VK→action map again; the composition-aware gate lives in `plan_action`, matching
the convention recorded for the `-`/`=` page keys.

`plan_action` resolves a digit in Chinese mode as follows:

1. If `composing + digit` is an abbreviation prefix → `KeyAction::Digit`, entering
   the composition string.
2. Otherwise, if the composition is active → `KeyAction::Select(index)`, preserving
   FR-006 candidate selection unchanged.
3. Otherwise → `None`, releasing the key so the host inserts a literal digit.

A digit also keeps accumulating when the composition is already all digits, so
`996` stays typeable once started. `compose_text`, `sync_engine`, and
`apply_action` carry the new `Digit` variant through the same paths as `Letter`.

## Alternatives considered

**Let digits always enter the composition, moving candidate selection to another
key.** Rejected: FR-006 and T-040 accept digit selection, and the VM acceptance
evidence for it is already recorded; changing that binding would regress an
accepted behaviour to fix a smaller one.

**Drop the ten digit-bearing keys from `seed.tsv` and keep digits purely for
selection.** Rejected: the keys are legitimate slang (`996`, `520`, `666`), the
build pipeline already accepts and gates them, and removing them would leave the
package inconsistent with its own stated key rule.

**Release pure digit strings to the slang lookup once a terminator is seen.**
Rejected: a per-keystroke model has no terminator event, so the IME cannot know
when a digit string has ended; this was the earlier S-2 sketch in the design doc
and it does not survive contact with the key path.

**Gate on "no candidates exist" instead of on the dictionary prefix.** Rejected:
after typing `9`, the composition is empty and there are no candidates, so this
test would send every digit into the composition and break literal digit entry.

## Consequences

- All ten digit-bearing abbreviation keys become reachable; `u1s1` works through
  the mixed letter/digit path and `996` through the pure-digit path.
- Cost: because the digit test requires a dictionary lookup per digit keystroke,
  digits are no longer free. The lookup is a binary search over the pinyin index
  on the mmap package, on the same order as the existing prefix-completion query
  that already runs on every letter.
- Cost: a composition that is already all digits keeps accepting digits, so a user
  who wants to select a candidate with a digit must not be mid-digit-string. This
  is the accepted trade for making `996` typeable.
- `classify_key` no longer encodes selection; any future change to digit semantics
  belongs in `plan_action`, and the unit test asserting the classification now
  expects `Digit`.
- Tests: five new unit tests cover prefix recognition without misclassifying
  pinyin entries, digit entry reaching an abbreviation word, English-mode and
  non-digit rejection, preserved selection/literal-release semantics, and
  prefix-driven composition. `zhu-ye-ime` runs 54 tests; the workspace gate is
  green, and host-e2e still passes its digit-selection check.
