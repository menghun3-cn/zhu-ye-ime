# Agent Note: Language-bar button left-click toggles Chinese/English mode (T-100, T-046 extension)

Status: proposed

[中文](2026-10-05-langbar-click-toggle.zh.md) | English

## Problem

The language-bar icon ("中"/"英", implemented note 2026-09-28-language-bar-mode-icon)
already tracks the engine mode, but the button's `OnClick` was a no-op: at that time the
engine state was `Rc<Mutex<EngineState>>` (not `Send`), and ctfmon may enter item COM
methods from arbitrary RPC threads, so touching the engine cross-thread violated the
existing thread-model assumption. The user explicitly asked for "language-bar button
click toggles Chinese/English mode" (batch two, item 1 of the accepted 11-item list).

## Proposal

1. **Left-click toggles the mode.** On `TF_LBI_CLK_LEFT` (1) the injected click handler
   runs; the returned mode is applied to the button (icon refresh via the advised
   `ITfLangBarItemSink::OnUpdate(TF_LBI_ICON)`). Other clicks (e.g. right-click menu)
   keep the previous behavior.
2. **Engine shared state migrates to `Arc<SharedEngine>`.** `SharedEngine(Mutex<EngineState>)`
   is `unsafe impl Send + Sync` with a three-part safety argument (see the type comment in
   tsf.rs): ① every non-`Send` field (COM interfaces, candidate-window raw pointer) is only
   accessed while holding the lock, and moving the lock owner does not move ownership of the
   locked data; ② the last strong reference is dropped only on the keyboard/activation
   thread — the language bar's `OnClick` produces only a temporary strong reference via
   `upgrade` while `self.state` always keeps the state alive — so the drop thread is the same
   as in the `Rc` era (candidate window's `DestroyWindow` in `Drop` never runs cross-thread);
   ③ `IUnknown::Release` is thread-agnostic. Consequence for future work: prefer the
   "newtype + `unsafe impl`" wrapper for a `Mutex<T>` whose fields are all lock-guarded when
   cross-thread migration is needed, instead of bolting `Send` onto individual fields
   (orphan rule) or spreading ad-hoc unsafe impls.
3. **No reference cycle.** `LangBarModeButton` gains
   `click_handler: Box<dyn Fn() -> Option<InputMode> + Send + Sync>` that captures only a
   `Weak<SharedEngine>`; `EngineState.lang_bar → LangBarModeButton → Weak` stays acyclic.
   A failed `upgrade` (engine unreachable) makes the click a safe no-op (returns `None`,
   mode unchanged).
4. **Semantics identical to Shift toggling.** The handler mirrors `sync_engine::ToggleMode`:
   `toggle_mode()` under the engine lock, icon notification outside the lock; a composition
   in progress is preserved (existing `InputEngine` semantics).

## Alternatives considered

- **Whole `Arc<Mutex<EngineState>>` with `EngineState: Send`**: requires `Send` on windows
  interfaces (`ITfKeystrokeMgr`, `ITfComposition`, …), which the orphan rule forbids to
  impl locally → rejected; the newtype transfers the lock-discipline argument instead.
- **Cross-thread message post (PostMessage to the candidate-window thread)**: adds a
  message-pump dependency and queuing latency with no benefit over lock-guarded toggling →
  rejected.
- **Dual-source mode atomics (engine + button each hold an `AtomicU8`)**: introduces two
  sources of truth that drift → rejected; the button mode is only a display cache, the
  engine lock is authoritative, and both write paths (Shift sync, `OnClick`) converge.

## Acceptance criteria

- Win11 desktop language bar enabled (Settings → Personalization → Taskbar → Input method
  indicator → Use desktop language bar): the "中" button appears; left-click switches the
  icon to "英" immediately; click again back to "中".
- Toggling changes real behavior: in Chinese mode letters start a pinyin composition; in
  English mode letters pass straight through.
- Clicking mid-composition preserves the pinyin composition (same as Shift toggling).
- After deactivating the IME (switch away and back), button and engine mode agree, no
  stale state.
- Rapid/repeated clicks: no crash, no icon corruption (toggles serialize under the engine
  lock).

## Risks

- The button `mode` and engine `mode` remain two values; the button is only a display
  cache, but a reader could briefly see a stale icon after a concurrent toggle — cosmetic
  only, behavior authority stays inside the engine lock.
- New `EngineState` fields must re-check the "accessed under lock" discipline that the
  `SharedEngine` argument depends on; the note in tsf.rs is the audit anchor.
- ctfmon RPC-thread concurrency with the key sink is serialized by the same lock, but the
  lock is a `std::sync::Mutex` (not reentrant): the handler must not call back into
  `sync_engine` while holding the lock — the implementation keeps the toggle (locked) and
  the icon notification (unlocked) strictly separated.
