# Agent Note: Updater spawned from the settings GUI must not flash a console window

Status: implemented

[中文](2026-10-13-updater-spawn-no-console-window.zh.md) | English

## Problem

User reported (T-153): with "启用在线更新" enabled, clicking "检查更新" on the
关于与更新 page pops up a cmd console window. Root cause: `zhu-ye-updater.exe`
is a **console-subsystem** program, while the settings process is a
GUI-subsystem process (no console). When `updater::run` spawned it with plain
`Command::output()`, Windows created a brand-new **visible** console for the
child on every check/apply — the flashing black window. The IME side's
`check-once` spawn (S-8, `CREATE_NO_WINDOW`) and T-133 (dialogs converted to
GUI subsystem) had already handled this family of problems; only the settings
window's manual check/apply path was missing the flag.

## Decision

In `crates/zhu-ye-settings/src/updater.rs`, `run()` now goes through a
`suppress_console_window` helper before `Command::output()`:

- **Windows**: `creation_flags(CREATE_NO_WINDOW)` via
  `std::os::windows::process::CommandExt` — part of std, no third-party
  dependency, keeping the module's "no Windows API dependency" property
  (the extension is std's own surface). One place covers both the
  `check` and `apply` entry points.
- **Non-Windows**: no-op, so the module still compiles anywhere.

`CREATE_NO_WINDOW` still creates a console for a console-subsystem child (so
crt startup works), it just hides the window; stdout/stderr are collected
through pipes as before, and exit-code semantics are unchanged. The updater
remains a console program by design: it is also run manually for diagnosis,
where a visible console is a feature, so converting it to GUI subsystem
(T-133 style) was deliberately not applied here.

## Alternatives considered

**Convert `zhu-ye-updater` to GUI subsystem (`windows_subsystem = "windows"`).**
Rejected: manual diagnostic runs (admin console) would lose their console
output; the child already has no console output need in the GUI spawn path
(settings collects via pipes). Mechanism is closer to the IME-side
`CREATE_NO_WINDOW` precedent (S-8) and keeps the binary usable standalone.

**Set `creation_flags` only inside `start_update_task` (window.rs).**
Rejected: the flag belongs to the process-spawn abstraction (`run`) that owns
launch semantics; putting it in the caller would leak Windows spawn details
into window logic and invite a miss in future callers.

## Consequences

- Manual "检查更新" and "应用更新" from the settings window no longer flash a
  console; the results area still shows the updater's piped output.
- Regression test `无窗口标志下子进程输出收集正常` (Windows-only) pins the
  output path; existing `run` tests now use ASCII assertions because under
  `CREATE_NO_WINDOW` the child has its own console at the system-default code
  page (936), so cmd-echo of Chinese bytes would be GBK — the real updater
  prints UTF-8 through the pipe via Rust `println!` and is unaffected.
- Recurring rule for this codebase: any GUI process spawning a console child
  must add `CREATE_NO_WINDOW`; any console child spawned silently must not be
  expected to inherit the parent's console.

## Related

- IME-side silent `check-once` spawn precedent (S-8):
  [2026-10-04-settings-window-five-upgrades.md](../../implemented/feature/2026-10-04-settings-window-five-upgrades.md)
- T-133 GUI-subsystem conversion for resident windows ("托盘弹 cmd 窗口"):
  [2026-10-07-system-tray-mode-icon.md](../../implemented/feature/2026-10-07-system-tray-mode-icon.md)
- Acceptance: [docs/验收标准.md](../../../docs/验收标准.md) §21 (T-153)
