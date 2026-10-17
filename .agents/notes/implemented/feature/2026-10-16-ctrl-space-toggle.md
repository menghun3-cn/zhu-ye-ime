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

In `tsf.rs`, treat `Ctrl+Space` as a Chinese/English toggle with identical
semantics to the Shift toggle (both idle and composing states respond;
composition content is kept):

1. `ctrl_key_down()` reads `GetKeyState(VK_CONTROL)`; the pure predicate
   `is_ctrl_space(wparam, ctrl_pressed)` (space VK + Ctrl held) is extracted
   for testability.
2. `OnTestKeyDown` explicitly returns TRUE for Ctrl+Space — modifier
   combinations are classified as `None` by `plan_action`, so without this
   the event would never reach `OnKeyDown`.
3. `OnKeyDown` intercepts Ctrl+Space right after the Shift pending-toggle
   bookkeeping (and clears any pending Shift toggle), then runs
   `sync_engine(ToggleMode)` + `refresh_candidate_window` and returns TRUE,
   consuming the key. No TSF edit session is needed (state-only action, same
   path as the Shift-up toggle and the language-bar click, T-100).
4. The language-bar tooltip text now reads 「按 Shift / Ctrl+Space 切换」
   (`lang_bar.rs` `GetTooltipString`).

## Alternatives considered

- **Rely on plan_action to classify Ctrl+Space**: modifier-held keys already
  return `None` (only Ctrl+digit translation shortcuts are special-cased),
  so the event would fall through to the host; an explicit `OnTestKeyDown`
  accept is required.
- **Toggle only when idle**: the Shift toggle already behaves this way by
  design (composing keeps Shift as a modifier); for Ctrl+Space the user
  expects an explicit switch even mid-composition, so we keep composition
  and switch — matching the language-bar click semantics (T-100).

## Consequences

- Ctrl+Space and Shift both toggle Chinese/English; composition content is
  preserved in both cases; the candidate window and language-bar icon refresh
  through the existing mode-sync path.
- The key is consumed (host does not receive Space), so no accidental
  candidate-commit or space input happens on Ctrl+Space.
- Tooltip and user docs list both shortcuts; the engine has no new state
  (reuses `toggle_mode`/`set_mode`).

## Verification

- Unit test `ctrl空格判定`: Ctrl+Space → true; Space without Ctrl → false;
  Ctrl+A → false.
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace` (ime lib 248, tsf 45, core 352,
  host_e2e 169), `git diff --check`; docs gates clean.
- Docs updated: FR-004 (需求规格说明书), acceptance row 「Ctrl+Space 切换
  （T-165）」under FR-004 (验收标准), 安装与使用 §5 shortcut table,
  CHANGELOG [Unreleased], todos T-165.
- Deployed (2026-10-16, PR #207 → develop e4afddb, together with the T-163
  round-2 fix): DLL `zhu_ye_ime_3079725B.dll` (2,260,480 B) landed with
  pointers switched and ctfmon restarted; verify-tsf-dll/identity passed.
  Real-host verification of the Ctrl+Space toggle stays pending (new
  host-process window).
