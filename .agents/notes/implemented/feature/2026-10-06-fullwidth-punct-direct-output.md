# Agent Note: Full-width punctuation direct output

Status: implemented

## Problem

On 2026-10-06 the user reported that the character `、` (ideographic comma) could
not be typed at all, no matter whether they toggled with Ctrl+Space or Shift.
Investigation showed every punctuation key was passed through to the host in both
the composing and idle states of Chinese mode, so the IME could not emit `、`；the
only source of `、` was an entry in the v-mode `vh` symbol group. The same gap
applies by inference to `，` `。` `；` `：` `？` `！` `（` `）` and the paired
quotes — full-width punctuation is part of core Chinese typing workflow, and the
user confirmed the goal of supporting the common full-width punctuation set.

## Decision

In **Chinese mode (both composing and idle states)**, the mapped punctuation keys
now commit their full-width counterpart directly through
`InputEngine::commit_fullwidth_punct` instead of being passed to the host. A
composing buffer, when present, is cleared (unconfirmed pinyin is discarded, like
mainstream Chinese IMEs), the previous-word hint is reset, and the quote
alternation state is reset with the composition.

Mapping (first release): `\`→`、`；`,`(VK_OEM_COMMA)`→`，`；`.`(VK_OEM_PERIOD)`→`。`；
`;`(VK_OEM_1)`→`；`；Shift+`;`→`：`；Shift+`/`(VK_OEM_2)`→`？`；Shift+`1`→`！`；
Shift+`9`/`0`→`（`/`）`; `"` (Shift+`'`, VK_OEM_7) alternates `“`/`”`; `'`
(VK_OEM_7) alternates `‘`/`’`.

Key path: `classify_key` produces `KeyAction::FullWidthPunct(char)`；`plan_action`
accepts it only in Chinese mode; `apply_action` calls
`commit_fullwidth_punct` inside the engine lock and commits the returned text via
`finish_composition`; `sync_engine` only refreshes the (now empty) candidate
window. `:` follows the same rule **except** when the composing string is a
URL/email intent (`http:`/`me@163.com`, T-066/D-11), which still appends `:`
into the composition; `@` and `/` remain passed through to the host.

Not triggered: English mode (host receives the half-width key); `-`/`=` remain
paging keys (T-033); digit-format mode keeps `.` as decimal and its buffered
digits; v mode keeps its symbol groups. Punctuation output never enters user-word
learning.

## Alternatives considered

- **Only composing-state punctuation mapping.** Rejected: the user's reported
  scenario includes typing `、` with an empty composing buffer; idle-state full
  width matches mainstream IME behavior.
- **Pass through and let the host convert to full width.** Rejected: conversion
  is not the IME's responsibility and is unreliable across host apps.
- **Append punctuation into the composition for selection.** Rejected: every
  mainstream Chinese IME commits punctuation directly; immediate commit does not
  disturb the pinyin being typed.
- **Keep `.` passed through in Chinese mode (pre-existing behavior).** Rejected:
  without this key there is no full-width period path; the behavior change is
  covered by updated tests and the FR-067 spec.

## Related

- [TSF composition and key events](../../../implemented/feature/2026-09-18-tsf-composition-and-key-events.md) — the classify/plan/apply key path this mapping extends.
- [Format symbols, emoji, and candidate sources](../../../implemented/feature/2026-09-30-format-symbol-emoji-candidates.md) — `@`/`.`/`/`/`:` URL/email-intent semantics preserved by the `:` exception.

## Consequences

- `.` in Chinese mode now commits `。` instead of passing through; the digit-format
  decimal and URL/email-intent paths are preserved, and the affected tests were
  updated to the new semantics.
- Full-width punctuation is deterministic and offline; no new dependencies.
- Behavior changes are user-visible and were spec'd as FR-067 (§22) with the
  toggle/quote state documented; host applications see a normal committed text.
