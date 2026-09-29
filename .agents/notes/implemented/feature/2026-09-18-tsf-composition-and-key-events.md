# Agent Note: TSF composition and key handling for M1

Status: implemented

[中文](2026-09-18-tsf-composition-and-key-events.zh.md) | English

## Problem

A registered TSF text service is still unusable until key events can become visible
text in a host edit session. Without key filtering, composition, and commit, the IME
appears in Windows but typing produces nothing; without a shared engine contract, the
TSF adapter and the future candidate UI can drift apart.

## Decision

`TextService` implements `ITfTextInputProcessorEx`, `ITfKeyEventSink`, and
`ITfCompositionSink`. During `Activate` it resolves `ITfSource` from the thread
manager and registers the key event sink, keeping the sink cookie. `Deactivate` ends
any composition, clears engine state, and unadvises the sink. An empty thread manager
is accepted so lifecycle unit tests and self-checks can run without a TSF host.

Key events map to a small `KeyAction` set: a-z become letters, Backspace deletes from
the composition, and Space/Enter/Esc commit or cancel only while a composition is
active. Digits 0-9 are classified as `KeyAction::Digit` and resolved in `plan_action`
by a composition-aware rule: they enter the composition when they prefix a
digit-bearing abbreviation key, and otherwise fall back to candidate selection or
literal release (see the
[digit abbreviation key note](../../implemented/bug-fix/2026-09-29-digit-abbreviation-key.md)).
`OnTestKeyDown` reports whether the IME wants the key; `OnKeyDown` requests a
synchronous read-write edit session with `TF_ES_SYNC | TF_ES_READWRITE`. If the edit
session cannot be granted, the engine is still advanced so later keystrokes are not
based on drifted state.

Inside the edit session, `ITfEditSession::DoEditSession` runs one `apply_action`
callback. Letter/Backspace updates locate the insertion point with
`ITfInsertAtSelection` in query-only mode, then write through
`ITfContextComposition::StartComposition` and `ITfRange::SetText`; Space, Enter,
Esc, and digits end the composition through `EndComposition` or insert committed
text directly when no composition exists. The engine is synchronized after TSF
text is written. The `InsertAtSelection` **write** branch is deliberately not
used — see the [write-path crash note](../../implemented/bug-fix/2026-09-23-tsf-insert-at-selection-write-path-crash.md).

The input behavior itself lives in `crates/zhu-ye-ime/src/input.rs` as a pure Rust
`InputEngine` with the M1 seed dictionary, so candidate generation, ordering,
selection, and preview methods are unit-testable without Windows. Shared runtime state
is `Rc<Mutex<EngineState>>`, not `Arc`: TSF apartment callbacks stay on one thread and
the stored COM interfaces are not `Send + Sync`. Each edit-session callback object
carries a `Box<dyn FnOnce>` and runs once.

No display attributes or candidate window are implemented yet. Space commits the
first candidate or the pinyin text, Enter commits pinyin, and Esc cancels. T-012 and
T-013 build on this interface.

## Alternatives considered

**Keep composition state entirely in the TSF adapter.** Rejected: candidate logic
would be embedded in COM plumbing and impossible to test on a dev host; the pure
engine is the unit-testable core for the later dictionary and sorting work.

**Mutate the engine first, then write TSF, rolling back on failure.** Rejected: TSF
text changes cannot be rolled back atomically, so write-then-sync avoids permanent
engine skew.

**Recreate the composition on every update.** Rejected: storing the `ITfComposition`
and reusing `GetRange`/`SetText` is simpler and avoids focus surprises.

**Use `Arc<Mutex<EngineState>>` and force `Send + Sync`.**
Rejected: TSF runtime COM objects are not guaranteed `Send + Sync`;
`Rc<Mutex<_>>` matches the single apartment thread and keeps the clippy
`-D warnings` gate green.

**Request async edit sessions.** Deferred to M1: synchronous sessions keep one
key-action pipeline and avoid interleaving; async can be added if a host stalls on
long operations.

## Consequences

Letters, Backspace, Space, Enter, Esc, and digits 1-9 now produce TSF composition or
committed text, and the M1 seed dictionary turns `nihao` into `你好` in the
composition path. Twenty unit tests pass across the engine and TSF adapter,
`cargo clippy --workspace --all-targets -- -D warnings` passes, and the release DLL
export check passes.

T-011 passed VM acceptance on 2026-09-23: Notepad typing, composition, commit,
paging, layer and mode switching, and user-word learning all verified with zero
crashes; the crash root causes and their fixes are recorded in the
[write-path crash note](../../implemented/bug-fix/2026-09-23-tsf-insert-at-selection-write-path-crash.md)
and the [page-slice note](../../implemented/bug-fix/2026-09-23-candidate-window-page-slice-hidden.md).
The earlier registration note's claim that M1 has no key processing no longer
matches this build; this note is authoritative for the current TSF adapter.
