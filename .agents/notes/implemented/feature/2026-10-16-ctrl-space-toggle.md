# Agent Note: Ctrl+Space toggles Chinese/English mode

Status: implemented

[中文](2026-10-16-ctrl-space-toggle.zh.md) | English

## Problem

The IME's Chinese/English toggle was bound to a lone Shift press only
(T-114 pending-toggle state machine), and the user asked for a dedicated
`Ctrl+Space` shortcut for the same toggle ("ctrl+空格，改竹叶输入法为切换中英文").
Note this is distinct from the system-level Ctrl+Shift input-language
cycling (T-164, fixed in the OS hotkey configuration).

## Decision

`Ctrl+Space` toggles Chinese/English mode. The switch is decided on the
**Space key-up event**, not on key-down:

1. `OnKeyDown(VK_CONTROL)` sets `ctrl_down` and clears the pending
   "Ctrl just released" timestamp; `OnKeyUp(VK_CONTROL)` clears `ctrl_down`
   and records `ctrl_up_at`.
2. `OnKeyUp(VK_SPACE)` computes the age of the last Ctrl release and applies
   the pure predicate `is_ctrl_space_up(ctrl_age_ms, ctrl_down, intervened)`
   (`tsf.rs`): the switch fires when no other key was pressed while Ctrl was
   held **and** either Ctrl is still down or Ctrl was released no longer than
   `CTRL_SPACE_UP_WINDOW_MS` (200 ms) before the Space key-up.
3. A match runs `sync_engine(ToggleMode)` + `refresh_candidate_window` and
   consumes the key. The `ctrl_intervened` flag (set when any non-Space key
   arrives while Ctrl is held) prevents `Ctrl+C` followed by Space from
   toggling the mode.
4. The 200 ms window is what makes **fast taps** work: the user reported
   that quickly tapping Ctrl+Space ("有时候感觉这俩按键连着快一些的时候未
   触发") failed, while the same speed worked in Sogou. It covers the
   "Ctrl released slightly before Space" ordering.
5. The language-bar tooltip text reads 「按 Shift / Ctrl+Space 切换」
   (`lang_bar.rs` `GetTooltipString`).

## Alternatives considered

- **`GetKeyState` / `GetAsyncKeyState` for the modifier state**: the sentinel
  log shows both always report "not pressed" inside TSF key callbacks (the
  IME's key sink runs on a different thread than the raw-input state), so
  neither can classify the combination. Rejected on direct observation.
- **`PreserveKey(Ctrl+Space)` + `OnPreservedKey`**: registration succeeds, but
  the system routes the reserved combination to whichever IME registered it
  first — the callback never fires for 竹叶输入法 when Sogou is also
  installed. Rejected on direct observation.
- **Tray-side `RegisterHotKey(Ctrl+Space)` bridge**: it does work, but it is
  a global exclusive grab that breaks Sogou for the user. Rejected.
- **Low-level keyboard hook in the tray + synthetic F24**: the hook never
  observed the injected key, and it added a privileged component. Rejected.
- **Deciding on Space key-down**: the system IME-toggle layer consumes the
  Ctrl+Space key-down, so the IME never sees it (sentinel log: Ctrl down,
  Space **up**, Ctrl up all arrive; Space down does not). Only the key-up
  path is observable.
- **Waiting for Ctrl to still be held at Space key-up** (the original
  implementation): fails on fast taps where the user releases Ctrl before
  Space. The 200 ms window is the compromise that keeps the fast-tap case
  working without reacting to unrelated later Space presses.

## Consequences

- Ctrl+Space and Shift both toggle Chinese/English; composition content is
  preserved; the candidate window and language-bar icon refresh through the
  existing mode-sync path.
- The Space key is consumed on the key-up, so no accidental candidate commit
  or space input happens on Ctrl+Space.
- The engine carries three new state fields (`ctrl_down`, `ctrl_up_at`,
  `ctrl_intervened`); the mode toggle itself reuses `toggle_mode`/`set_mode`.
- A Space press within 200 ms after a Ctrl release (with no intervening key)
  also toggles the mode — intentional, and bounded by the window.

## Verification

- Unit test `ctrl空格弹起判定`: Ctrl held → true; Ctrl released 120 ms ago
  → true (fast tap); released 300 ms ago → false; no Ctrl → false; Ctrl with
  an intervening key → false.
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace`, `git diff --check`.
- User-verified on a real host: 「ctrl+space已经支持切换中英模式」
  (2026-10-19).
- Sentinel evidence (TSF debug log): `zhu-ye: ctrl+space ToggleMode (space-up)`.
- Deployed: DLL `zhu_ye_ime_C6ADEA46.dll` (fast-tap window) and later
  `zhu_ye_ime_596BD111.dll` (T-172 batch) landed with pointers switched and
  ctfmon restarted.
- Docs updated: FR-004 (需求规格说明书), acceptance row 「Ctrl+Space 切换
  （T-165）」under FR-004 (验收标准), 安装与使用 §5 shortcut table,
  CHANGELOG [Unreleased], todos T-165.
