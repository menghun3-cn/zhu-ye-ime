# Agent Note: Shift commits the raw pinyin while composing

Status: implemented

[中文](2026-10-19-composing-shift-raw-commit.zh.md) | English

## Problem

Sogou's IME switches to English mode when the user presses Shift while a
candidate list is open, and the **composing pinyin itself** is committed
("当输入时，候选框已经出来内容，此时按键shift时，应该直接把输入的内容直接
上屏，和搜狗的表现一样"). 竹叶输入法 did the opposite: Shift was only a
modifier while composing (T-114 pending-toggle state machine), and the
combination was never bound to any commit action.

## Decision

While the engine is composing (`engine.is_active()`), a Shift key-down is
rewritten into a `KeyAction::Enter` action and the regular edit-session path
commits the **raw pinyin** (`nihao` → `nihao`, not the first candidate). The
mode then switches to English.

1. `shift_down_action(combinational, action)` (`tsf.rs`) is the pure rewrite
   rule: composing → `Some(KeyAction::Enter)`; idle → the original action.
2. `OnKeyDown` applies the rewrite and falls through to the normal
   `RequestEditSession` flow, so the text reaches the document through
   `commit_text` + `finish_composition` exactly like the Enter key.
3. After the edit session, the key handler issues `sync_engine(ToggleMode)`
   and refreshes the candidate window, which also syncs the system keyboard
   layout (T-168).

## Alternatives considered

- **Commit the first candidate instead of the raw pinyin**: the user
  explicitly corrected this ("按space是第一个候选词上屏，但是输入例如nihao，
  用shift切换中英格式时，期望的是nihao拼音上屏，而不是'第一个候选词上屏'").
  `Enter` semantics (raw pinyin) are what Sogou does.
- **Call `engine.handle_space()` directly from the key handler**: the engine
  state changes, but **no text is written to the document** — text commits in
  this IME only inside a TSF edit session (`apply_action` →
  `commit_text` → `finish_composition`). The first implementation did this
  and the user reported 「nihao并没有上屏」. Rejected on that observation.
- **Toggle the mode without committing**: keeps the composition but then
  the raw text is lost or the mode switch leaves a dangling composition;
  Sogou commits the pinyin.
- **Use `KeyAction::Space`** (first candidate): same defect as above, and it
  contradicts the requested semantics.

## Consequences

- Shift while composing now commits the raw pinyin and switches to English;
  the user accepted this behavior in the same round that introduced it.
- The idle-state Shift behavior is unchanged: it still arms a pending toggle
  that fires on key-up (T-114).
- Because the Shift key-down is consumed in composing state, the host does
  not learn about the Shift press for that key; English uppercase input
  afterwards requires a fresh Shift press.
- No new engine state: the existing `is_active()` / `handle_enter` /
  `toggle_mode` primitives cover the behavior.

## Verification

- Unit test `组合态shift改写为原文上屏空闲态不变`: composing → `Enter`;
  idle → `ToggleMode`; idle without action → `None`.
- Unit test `回车提交拼音原文而非首候选`: `preview_enter()` yields `nihao`
  for the composing `nihao` (the raw pinyin, not 你好).
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace`, `git diff --check`.
