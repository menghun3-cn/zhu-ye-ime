# Agent Note: Settings about-and-update page (M12-5, T-077)

Status: implemented

[中文](2026-10-02-settings-about-and-update-page.zh.md) | English

## Problem

The About & Updates page (FR-044) must give users a visible "check for
updates" affordance and a diagnostic view, while keeping the settings window
itself **completely offline** (D-44): every network connection must originate
from the separate `zhu-ye-updater.exe` process. Updates are opt-in by default
(P-03): with `online_update=false` the UI must disable the check/apply buttons
and explain why, and the window must not spawn any process — keeping "zero
outbound connections when disabled" verifiable (acceptance 13.1 FR-044).

Two additional constraints: applying an update replaces local program files,
so it needs a second confirmation; and diagnostics (config path, data
directory, log directory, installed packs) must be shown without spawning the
updater (T-077 reads them locally).

## Decision

### Updater bridge `crates/zhu-ye-settings/src/updater.rs`

The only network-bearing abstraction: `updater_exe_path()` locates
`zhu-ye-updater.exe` next to the current executable (`std::env::current_exe()`
directory — same `bin\` folder the installer ships both exes into, T-078);
`run(program, args)` spawns the child via `Command::output`, concatenates
stdout+stderr, and returns `Err(text)` on non-zero exit so failures are shown
verbatim; `output_lines(text)` trims blank lines for the result pane. The
module has no Win32 dependency (pure `std::process`), unit-testable headless.

### The window never spawns while updates are off

`can_run_updater` gates every check/apply path on three conditions:
`online_update` (from `config.json`), updater exe presence, and no task in
flight. The disabled button is rendered in control-background +
placeholder-text and is a no-op on hit; the info pane states the reason
("updates are off, D-44; enable in the list page" / "updater missing:
install the full package"). That gate sits **before** any spawn, so
`online_update=false` ⇒ zero processes and zero connections, measurable.

### "Enable online updates" is a runtime switch, first About item

`ChipValue::OnlineUpdate(bool)` reuses the two-option chip mechanism
("关闭"/"开启", off first per P-03). Clicking persists immediately via
`config::save_online_update` (re-read before write, S-8), like theme/mode.
Selecting the row itself does nothing.

### Check-update and diagnostics are flat sections, not subviews (T-143)

Since T-143 the About & Updates page is a single flat layout — there are no
subviews and no "返回关于与更新" button: the first row is the online-update
switch item (reusing `item_rows`), below it the update section (status line
"在线更新：已开启/已关闭" with disable reason / updater path, result pane with
updater output shown truthfully, then the [检查更新][应用更新] buttons), and
below that a "版本与诊断信息" section header with the diagnostic lines. The
left nav switches pages; diagnostics are lazily collected the first time the
page is drawn. `about_layout` in `layout.rs` cuts diagnostic rows at the
content bottom (fixed result area of ~3 rows).

### Background task thread + message-loop handoff

`Command::output` blocks, so running it on the UI thread would freeze the
message loop. `start_update_task` spawns a thread: it runs the updater, sends
the outcome over `mpsc`, **then** posts `WM_UPDATER_DONE` (WM_USER+0x120); the
window proc takes the receiver, `try_recv`s immediately, and invalidates.
Because the send happens before the post, the result is always ready when the
message arrives. `WM_UPDATER_DONE` carries the task sequence number in
`wparam`; a stale receipt from a superseded task is ignored. `HWND` is not
`Send`, so the thread receives `hwnd.0 as isize` and re-wraps it.

### Apply needs a second confirmation

The apply button opens a `MessageBox` (YESNO, D-44) before spawning
`zhu-ye-updater apply`.

### Diagnostics section is local-only

`build_diagnostics` reads `zhu_ye_core::core_version()`, `config.json` path,
data directory, acceptance log directory, and the installed-pack list via the
existing local inventory scan (`list_packs_now`) — no spawn, no network.
Format lines are drawn like the manage/repair subviews (first line stronger).

### Forensics hooks

`--shot --page about` captures the flat About page (T-143; the former
`--update` / `--diag` subview entries were removed with the subviews).

## Alternatives considered

1. **HTTP client inside the settings window** (e.g. reqwest or a curl child):
   violates D-44's single-networking-component boundary (the updater process);
   rejected — the window must stay provably offline.
2. **Synchronous updater call on the UI thread**: `Command::output` blocks, so
   a direct call would freeze the message loop for the whole check/download;
   rejected — a background thread with mpsc + `PostMessageW` handoff keeps the
   window live and the updater's long tail out of the loop.
3. **Apply without confirmation**: applying replaces local program files;
   rejected — a MessageBox second confirmation precedes every apply (D-44).
4. **Diagnostics via `zhu-ye-updater status`**: adds a process dependency for
   data the window already reads locally (config paths, core version,
   `installed.json` inventory); rejected — spawn stays limited to check/apply.
5. **Reusing an existing app message for the handoff**: the panel close
   message has different semantics; rejected — a dedicated `WM_USER+0x120`
   receipt carries a task sequence number so stale receipts from superseded
   tasks are ignored.

## Consequences

- `load_online_update`/`save_online_update` added to `config.rs` with a
  round-trip and field-preservation test (S-8 re-read keeps `last_check`).
- Settings lib tests 95→97 (model: About items ready + switch default off +
  subview transitions; layout: update/diagnostics geometry; updater bridge:
  line filtering, exit-code and spawn-failure paths run against `cmd.exe`).
- T-143: `Subview::Update/Diagnostics` and `ItemControl::OpenUpdate/
  OpenDiagnostics` deleted; `ABOUT_ITEMS` keeps only the switch item; the two
  former subview layouts were replaced by `about_layout` (106 lib tests).
- Host forensics (2026-10-02): three BMPs (diagnostics page, update-off,
  update-on) — the off/on pair differs at 3650 sampled pixels; real updater
  behavior confirmed: `online_update=false` ⇒ `check` exits immediately with
  no network ("未发起任何网络请求"), `true` ⇒ the updater process performs the
  connection and its manifest-404 failure is what the window shows.
- Host forensics (2026-10-09, T-143): flat About page BMP at 96 DPI — switch
  row, update section, diagnostics section all within one page, no back
  button; the "zero outbound connections when disabled" guarantee is unchanged
  (`can_run_updater` still gates before any spawn).
- Pending: the interactive VM items (explicit enable, check click, second
  confirmation, packet capture) remain open under acceptance 13.5.

## Related notes

- [Distribution package layout and installer](../process/2026-10-02-distribution-package-layout-and-installer.md)
  — ships `zhu-ye-updater.exe` next to the settings exe (T-078); this page
  depends on that delivery.
- [AI service contract and offline-by-default](../architecture/2026-09-21-ai-service-contract-and-offline-default.md)
  — precedent for "offline unless explicitly enabled": P-03 extends the same
  stance to the settings window.
- [TSF registration and lifetime](../architecture/2026-09-18-tsf-registration-and-lifetime.md)
  — registration ownership behind the manage/repair page; the diagnostics
  view shows the related paths without touching the registry.
- [Candidate window GDI rendering](../feature/2026-09-19-candidate-window-gdi-rendering.md)
  — shared theme/metrics primitives reused by the settings window (T-081
  extracts them into `zhu-ye-ui`).
