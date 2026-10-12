# Agent Note: Toolbox panel global hotkey, search state, and position memory (T-151)

Status: implemented

## Problem

The toolbox panel (emoji 面板 / 符号大全) has worked since T-074 as a
non-activating, always-on-top popup opened from the settings window's toolbox
page: clicking a cell `SendInput`s the character into the user's foreground
window without stealing focus (`WS_EX_NOACTIVATE`, `WM_MOUSEACTIVATE→
MA_NOACTIVATE`). The user reported on 2026-10-10 that popup-style opening is
awkward in real typing flow and asked for a better interaction model. The
agreed direction: **global hotkey invocation with anchored positioning** —
`Ctrl+Alt+E` summoning the panel wherever the caret is, emoji/symbol switching
inside the panel, search that temporarily activates focus and returns it after
selection, and position memory with a first-open-at-cursor default.

## Decision

- **Hotkey host = the permanently running tray process** (`zhu-ye-tray`, HKCU
  Run auto-start). The settings window closes itself on window close, so a
  hotkey registered there dies with it; the tray is the only process guaranteed
  alive. `config.json` gains `panel_hotkey: Option<String>` (default
  `ctrl+alt+e`, serde default, no `CONFIG_FORMAT_VERSION` bump). Parsing lives
  in new `zhu_ye_core::hotkey`: pure numeric constants (`MOD_*`, VK codes),
  no Win32 API calls — the same pattern as the `identity` constants (S-10).
  Unparseable/custom strings fall back to the default with a log line (the
  tray appends `%APPDATA%\zhu-ye-ime\panel-hotkey.log` on registration failure
  or open failure; it never crashes the tray).
- **Single global panel**: the window class `ZhuYeSettingsPanel` is unique
  session-wide. Both entry points — settings window open and tray `WM_HOTKEY` —
  check `FindWindowW(CLASS)` first (settings page also guards its own
  `state.panel`). The hotkey *toggles*: panel open anywhere → `PostMessage
  (WM_CLOSE)`; not open → create with `remembered_kind()` (last tab or emoji).
- **Search state = a narrow, temporary activation exception** (S-32): clicking
  the search box removes `WS_EX_NOACTIVATE`, records `GetForegroundWindow()` as
  the return target, and answers `WM_MOUSEACTIVATE` with `MA_ACTIVATE` so that
  one click legitimately focuses the panel for keyboard input. `WM_CHAR`
  accumulates the query (ASCII only — the panel host never enters an IME, so
  pinyin alias filtering is the main path; Chinese group-name queries are pure
  logic and already unit-tested but unreachable from the window, a recorded
  limitation). `Backspace` deletes, `Tab` switches panel tab (query kept),
  `Left`/`Right` page through the *filtered* results, `Esc` clears the query,
  restores `NOACTIVATE`, and returns focus to the target. Selecting a cell
  while searching returns focus first, then delivers. After selection the
  panel goes back to fully non-activating so consecutive click-to-insert keeps
  working — this is the one-time exception the S-11 discussion rejected as a
  general "shift focus on every click" design.
- **Position memory** (S-33): `%APPDATA%\zhu-ye-ime\panel.json` (written
  atomically: tmp + rename, silent failure) records `{kind, left, top, width,
  height}` in `WM_DESTROY`; next open clamps the saved rect into the work area
  (`SPI_GETWORKAREA`) and reuses it. No memory yet → place 12px right/below the
  cursor (`GetCursorPos`). Shot mode (`--shot-panel`) deliberately neither
  reads nor writes memory so captured frames stay comparable.
- **Filtering is plain logic** (`panel::filter_entries`), reused by paint,
  hit-testing, paging, and hover: case-insensitive substring match on label /
  group / text, empty query returns everything; `panel::page_of` pages the
  filtered list (at least one page, clamped).

## Alternatives considered

- **Settings window registers the hotkey**: the window is not resident (closes
  with the window and often with the user closing it), so the hotkey would be
  dead exactly when the user is typing — rejected (S-30).
- **Another dedicated resident process for the hotkey**: duplicates the
  already-mandatory tray residency for one `RegisterHotKey`; the tray is the
  single self-starting process — rejected.
- **Keep popup-only opening, just anchor it**: the user explicitly rejected
  popup-style opening as the primary path; hotkey-with-anchor is the requested
  model — rejected.
- **Every click briefly activates the panel** (move focus around per click):
  breaks the "never steal focus while browsing" property that makes
  click-to-insert feel instant; only the search state needs keyboard, so the
  exception is scoped to it — rejected (echoes the S-11 rejection).
- **Position memory inside `config.json`**: runtime UI geometry is not user
  configuration and would churn the config file and its format-version rules on
  every close; a dedicated sidecar file keeps them separate — chosen.

## Consequences

- Both `zhu-ye-settings.exe` *and* `zhu-ye-tray.exe` change: the tray gains a
  dependency on the settings library (one-way, settings→core already exists, no
  cycle).
- Keyboard operation exists only in the search state; browsing stays
  non-activating, so repeated click-to-insert keeps focus in the typing app.
- Search input is ASCII-only (single-color GDI rendering remains, no IME hook
  in the panel) — pinyin aliases are the main use; Chinese group-name search is
  test-covered but not window-reachable.
- The panel footer button is relabeled 返回设置 → 关闭 since the tray-hosted
  panel has no settings window to return to; the hint line shows the effective
  hotkey read from config.
- Hotkey registration failure (e.g. occupied) degrades to tray-menu-only with a
  log line, never a crash.

## Related

- T-074 panel window design: [settings-window-process-and-registration-ownership](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)
- Decision log S-30..S-33 in [docs/设置窗口视觉设计.md](../../../docs/设置窗口视觉设计.md) §14
