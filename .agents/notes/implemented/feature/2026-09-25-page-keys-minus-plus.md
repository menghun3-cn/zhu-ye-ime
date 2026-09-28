# Agent Note: Page keys changed to Minus/Equal (Plus)

Status: implemented

[中文](2026-09-25-page-keys-minus-plus.zh.md) | English

## Problem

Paging the candidate popup used `,` (previous page) and `.` (next page),
bound in `classify_key` as `VK_OEM_COMMA`→`PageUp`, `VK_OEM_PERIOD`→
`PageDown`. The user directive: with the candidate popup open, the `-` and
`+` keys page, replacing the `,`/`.` keys.

## Decision

- `VK_OEM_MINUS` (`-` key) → `PageUp`; `VK_OEM_PLUS` (the `=` key; with
  Shift held it produces `+`) → `PageDown`. Mapping the VK covers both the
  unshifted `=` and the shifted `+` — only one virtual code exists.
- `,` and `.` are no longer mapped; they fall through to the host (S_FALSE /
  BOOL(0)), like any unmapped key.
- The blocking obstacle was Shift itself: `+` = Shift+`=`, but a Shift
  key-down triggered `ToggleMode` immediately, so paging with `+` mid-
  composition would have silently flipped to English mode while the popup
  stayed visible. Fix: `ToggleMode` is only accepted in `plan_action` when
  the engine has no active composition (`!engine.is_active()`); while
  composing, the Shift that precedes `+` is a plain modifier and falls
  through. The engine-level `toggle_mode` semantics ("composition preserved
  when toggling") are unchanged for the non-composing case.
- Caps/quirk note: `classify_key` stays a pure VK→action map; the
  composition-aware gating lives in `plan_action`, which already owns the
  letter-vs-active rules.

## Alternatives considered

**Track a pending Shift tap (toggle only on key-up if no other key was
pressed in between).**
Rejected for this change: it is a larger state-machine change across
`OnKeyDown`/`OnKeyUp`, and TSF key-up delivery for consumed keys is not
worth betting on for a key-binding fix. Gating on active composition
achieves the same user-visible goal for `+` paging with two lines, and the
existing "Shift 单击切换" acceptance still holds (tapping Shift outside a
composition toggles exactly as before).

**Map only the shifted `+` (require Shift held) and leave `=` unbound.**
Rejected: the physical key is one VK; distinguishing shift at classify time
would need `GetKeyState`, and unbinding `=` would surprise users typing
`=` expecting common IME `-`/`=` paging. Both `=` and `+` page down.

## Consequences

The key table in 需求/方案设计/安装与使用/验收标准 (FR-001 翻页 row) is
updated; the TSF-integration note's comma/period claims are superseded (see
2026-09-20-candidate-window-tsf-integration.md). New pure-function tests
cover VK_MINUS/VK_PLUS mapping, comma/period unbound, Shift gated while
composing, and English-mode fallthrough of the new page keys. VM drill on
v0.1.4 (test10) asserts the TSF log: `key 0xBB state action PageDown`,
`key 0xBD action PageUp`, no ToggleMode action after pressing `+`, and no
page action for `,`/`.`.
