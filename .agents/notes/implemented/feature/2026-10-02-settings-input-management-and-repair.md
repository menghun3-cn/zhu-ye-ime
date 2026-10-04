# Agent Note: Settings input-method management and two-level repair

Status: implemented

[中文](2026-10-02-settings-input-management-and-repair.zh.md) | English

## Problem

FR-043 requires the settings window to let the user check and repair the TSF
registration of the input method without reinstalling. When the language bar
or the IME stops working the user has no in-product way to see why: the
registration state (profile key, `Enable`, `InProcServer32`) is invisible, and
the only remedies are reinstalling or re-running the installer. The task
carries three decision constraints: report before acting (D-41), never delete
user data (rename only), and keep the elevation surface as small as possible
(D-40).

## Decision

Three new `Ready` entries on the 常用设置 page open two subviews and one
direct action:

- **恢复状态栏** (`RestoreLangBar`): after a confirmation dialog (D-43),
  terminates `ctfmon` via `taskkill /f /im ctfmon.exe` and reports that the
  system reloads the language bar on demand.
- **管理输入法** (`OpenManage`): read-only probe subview. `RegistryProbe`
  reads six facts (language profile key exists, `Enable` DWORD, `InProcServer32`
  default value, `ThreadingModel`) from `HKLM\SOFTWARE\Microsoft\CTF\TIP\…`
  and `HKLM\SOFTWARE\Classes\CLSID\…`; `evaluate_registration` turns them into
  `RegisterIssue`s and a headline. The subview also offers
  **打开系统输入法设置**, which opens the `ms-settings:keyboard` URI via
  `ShellExecuteW("open", …)`.
- **修复输入法** (`OpenRepair`): report-first subview. Entering it runs
  `scan_l1` (read-only) and shows `summary_lines()`; the bottom buttons act
  only on explicit clicks:
  - **一级修复（无需管理员）**: recreates a missing `packs/` dir,
    quarantines (renames to `.bak`, then `.bak.1`, … — never deletes) and
    rebuilds broken `.zyct` packs, a broken `config.json`, and a corrupt
    `user_words.json`; a `user_words.json` with a newer version is only
    reported ("请升级输入法"), never renamed; a broken base dictionary is
    only reported. Each item's outcome is independent (`Vec<L1Outcome>`).
  - **二级修复（需管理员，UAC）**: `ShellExecuteExW` with the `runas` verb
    launches `zhu-ye-settings --repair-registry`, waits with
    `SEE_MASK_NOCLOSEPROCESS`, and reads the exit code. The subcommand
    deduces the DLL path from `InProcServer32` (fallback: newest
    `zhu-ye-ime*.dll` under `exe_dir\tsf`), deletes the TIP and CLSID trees,
    recreates the six keys with the same values as
    `ime-identity.ps1 New-TsfRegistration`, is silent on success (exit 0) and
    pops an error box on failure (exit 1). It never accepts an arbitrary path
    argument. On success the parent re-probes and asks again whether to
    restart `ctfmon` (D-43).

Backup naming reuses the `UserDictStore::load` precedent (`user_words.json` →
`user_words.bak`, extension replaced), so the runtime auto-recovery and the
repair entry produce the same survivor name.

The subcommand short-circuits before the single-instance guard in `main.rs`.
The registry values written mirror `scripts/ime-identity.ps1`, whose constants
are guarded byte-for-byte by the T-079 gate `scripts/verify-tsf-identity.ps1`.

## Alternatives considered

**Run the whole settings process elevated** (app manifest
`requireAdministrator`). Rejected: every launch would show the UAC prompt and
grant admin to a GUI surface that only needs it for one action; D-40 requires
the smallest surface, so only the `--repair-registry` child is elevated.

**Delete and recreate broken files in level 1.** Rejected: D-41's "rename
instead of delete" keeps user data recoverable; quarantine renames with the
`.bak` counter and never overwrites an existing backup.

**Level 1 handling a newer-version `user_words.json`.** Rejected: renaming it
would silently discard data written by a newer client; the probe reports
"请升级输入法" only.

**Restore the language bar by invoking the installer.** Rejected: the
portable deployment has no installer dependency inside the settings window;
`ctfmon` reload is in-process and self-contained (open item, §6.4 of the
design doc, to be confirmed on the VM).

**Show the registration state inline in the item row.** Rejected: six facts
plus repair buttons do not fit a row; the subview pattern (already used by
「添加词库」) reuses the whole content area, bottom buttons, and back
navigation.

## Consequences

- Registry writes happen only when the user explicitly clicks 二级修复; the
  probe and level-1 scan are read-only.
- The TSF constants stay dual-maintained; T-076 adds no new constant, and the
  T-079 gate still passes 6/6.
- `restart_ctfmon` is the open implementation item of design §6.4: killing
  ctfmon and relying on on-demand reload is the current behavior; the VM run
  will confirm whether the system relaunches it (and how quickly), and the
  design doc will be updated then.
- Settings tests: 84 pass (model subview, layout geometry for
  manage/repair, repair logic, registry G/UID-path constants); the core
  probe test adds 7. Host screenshots (1100×775) verified by accent-button
  pixel scan: common page has no bottom buttons, manage has two,
  repair has the three-column layout.
- Cross-references
  [2026-09-18-tsf-registration-and-lifetime](../../implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)
  (registration structure and lifetime) and
  [2026-09-22-portable-scripts-windows-powershell-5-1-encoding](../../implemented/bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md)
  (the encoding convention that the two T-079 identity scripts initially
  missed; fixed together with this change so the gate runs on PowerShell 5.1).
