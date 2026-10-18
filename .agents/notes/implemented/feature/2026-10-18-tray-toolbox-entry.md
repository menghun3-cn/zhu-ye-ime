# Agent Note: Tray entry to the toolbox panel (T-169)

Status: implemented

English | [中文](2026-10-18-tray-toolbox-entry.zh.md)

## Problem

The toolbox panel (emoji / symbol picker, FR-070) has exactly one entry point: the
global hotkey `Ctrl+Alt+E` registered by the resident tray process. Nothing on
screen reveals that the feature exists — a user who does not remember the hotkey
cannot find it at all. The tray menu could not help either: it offered only
"status / open settings / exit".

The user compared this with Sogou's tray, which exposes two clickable input-method
entries (a grey-line "中" state icon and the orange "S" brand icon), each opening a
menu of real actions. The gap was not a missing icon but a **missing mouse entry
into the panel** (FR-074).

## Decision

`zhu-ye-tray` gains one mouse entry and keeps double-click as it was:

- **Menu item "工具箱(&T)…"** between the status line and "打开设置(&S)…".
  It calls the same toggle path as the hotkey (global single panel, remembered tab
  and position).
- **Double-click still opens settings**: the first implementation made it toggle the
  panel, and the user asked for the original behaviour back after accepting the
  change on 2026-10-18. `WM_LBUTTONDBLCLK` therefore calls `open_settings()`
  directly; the toolbox never takes the double-click slot.
- **Click refreshes immediately**: both left and right click now re-read
  `tray-state` before building the menu, so the status line and the tray icon never
  lag behind the 500 ms poll.

Two mechanisms make this safe:

1. **Modal-menu isolation (S-35).** `TrackPopupMenu` runs a modal loop
   (`WM_ENTERIDLE`) that does not dispatch ordinary messages to the calling
   thread. A panel created inside that loop — a window with its own message pump —
   would never receive paint or click messages. The menu therefore only *collects*
   the command; every action is dispatched **after** `TrackPopupMenu` returns.
2. **Serial task-queue thread (S-35).** A `PanelWindow` is bound to the thread that
   created it: `open` creates the window and `Drop` destroys it. The tray main
   thread must stay free to pump the tray loop and modal menus, and creating one
   throw-away thread per request would allow a second toggle to tear down a window
   that is mid-destruction. A single long-lived worker thread owns the panel:
   the tray posts `PanelAction::{Open, Close}` over an mpsc channel, the worker
   creates/closes the panel, pumps the panel's own messages every 15 ms while it is
   open, and reports back with `PostMessage(WM_TASK_DONE, 0, panel_open)`. The tray
   thread only reads that cached flag to decide open vs. close.

Panel usability from the tray entry: `PanelWindow::listen_escape(1500)` removes
`WS_EX_NOACTIVATE` for the first 1.5 s so that a user who just summoned the panel
can press `Esc` to close it. Moving the mouse into the panel or timer expiry calls
`end_escape_window`, restoring "click a cell, text is delivered, the panel never
steals focus" (S-11 unchanged). The hotkey path does not take the keyboard at all.

## Alternatives considered

- **Single click opens the panel (Sogou-like).** Rejected: single click must stay
  the stable menu entry (status check, exit) — guessing intent from click timing is
  less reliable than a plain left/right-click menu (S-34).
- **Double-click toggles the panel (first implementation).** Reverted: after
  accepting the change on 2026-10-18 the user asked for double-click to keep opening
  settings, so the panel keeps only the menu item and the hotkey. Rejected because
  changing an established gesture costs more than the new feature gains (S-36).
- **Create the panel directly in the menu handler.** Rejected: the modal menu loop
  blocks paint/click dispatch for the new window, so the panel would appear dead
  until the first click ends the menu (S-35).
- **One thread per toggle request.** Rejected: two quick requests can overlap on the
  same window (destroy mid-flight), and each thread would need its own message loop
  for the panel.
- **Second tray icon for the panel (mirroring Sogou's two icons).** Rejected: the
  menu already exposes every action; a second icon adds click surface without adding
  capability, and T-123 fixed the tray to a single square "中/英" icon.
- **Rewrite the panel into the tray process.** Rejected: `PanelWindow` already lives
  in the `zhu-ye-settings` library and is shared with the settings window (S-31
  single panel); duplicating it in the tray would fork the rendering code.

## Consequences

- The panel is reachable by mouse through the menu item in addition to the hotkey
  (double-click still opens settings); the tray itself is unchanged in shape — one
  icon, richer menu.
- `zhu-ye-tray` now owns a worker thread for its whole lifetime. `Stop` is sent on
  exit and on `WM_DESTROY`; the thread drops the panel (destroying the window) before
  returning. A failure to spawn the thread degrades to "menu item and hotkey report
  an error line in `panel-hotkey.log`", never to a crash.
- `listen_escape` briefly makes the panel the foreground window, and only the menu
  entry calls it. Any mouse activity inside the panel ends that window; a user who
  never touches the panel gets at most 1.5 s of "the panel has focus".
- `WM_ENDSESSION` removes the icon on logoff/shutdown, avoiding ghost icons left in
  the notification area.
- Double-click semantics match the pre-T-169 tray: it opens settings. The panel's only
  mouse entry is therefore the menu item, and the hotkey stays the primary entry.

## Testing

- Unit: menu command ids are distinct and ordered (toolbox → settings → exit);
  source-contract test that `TrackPopupMenu(` precedes every menu action dispatch and
  that `DestroyMenu` sits in between; source-contract test that `PanelWindow::open`
  never appears in the tray's production code; queue FIFO/dedup behaviour; dispatch
  fails explicitly before the queue starts.
- Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --workspace`, `git diff --check`,
  `scripts/verify-agent-notes.ps1`, `scripts/verify-translation-pairs.ps1`.
- Interactive checks (acceptance §23.1) need a real desktop: menu contents and
  order, single-click refresh, menu-opened panel usability, re-toggle closing,
  double-click still opening settings, single-panel mutex with the settings window,
  remembered tab/position, and a 10-cycle rapid stress run.

## Related

- Panel interaction design (S-30..S-33): [panel-hotkey-search](../../implemented/feature/2026-10-12-panel-hotkey-search.md)
- Tray icon mechanism: [system-tray-mode-icon](../../implemented/feature/2026-10-07-system-tray-mode-icon.md)
- Design decisions S-34, S-35: [docs/设置窗口视觉设计.md](../../../docs/设置窗口视觉设计.md) §14
- Requirement FR-074: [docs/需求规格说明书.md](../../../docs/需求规格说明书.md) §24
