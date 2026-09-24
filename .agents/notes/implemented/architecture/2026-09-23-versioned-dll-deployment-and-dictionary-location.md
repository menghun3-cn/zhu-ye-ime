# Agent Note: Versioned DLL deployment and dictionary located by module anchor

Status: implemented

[中文](2026-09-23-versioned-dll-deployment-and-dictionary-location.zh.md) | English

## Problem

During VM acceptance, `explorer.exe` holds the IME DLL loaded for each session and
cannot be forced to drop it, so overwriting `zhu-ye-ime.dll` in place fails or
silently keeps the old code. The pragmatic workaround — copying each build under a
versioned file name (`zhu-ye-ime-v4.dll`, `zhu-ye-ime-v5.dll`) and pointing the
CLSID registry key at the new name before restarting `explorer` — has a second
side effect: the old dictionary loader located the install directory with
`GetModuleHandleW("zhu-ye-ime.dll")`, which no longer matches the versioned file
name, so the real dictionary was skipped and the IME fell back to the embedded
seed dictionary.

## Decision

- **Deployment:** every VM deployment copies the fresh DLL as
  `zhu-ye-ime-v<N>.dll` next to the dictionary, sets
  `HKLM\SOFTWARE\Classes\CLSID\{E54D6682-…}\InProcServer32` to the new file,
  clears acceptance logs, kills `explorer.exe` (restarts in ~25 s) and then
  verifies the loaded module path. The versioned name also makes the locking
  problem moot: the old DLL stays on disk untouched. Regression to an earlier
  build is a one-line registry point change plus explorer restart; deferred
  cleanup composes with this scheme (see T-026).
- **Dictionary location:** `installed_dictionary_path` now anchors on an
  address inside the very module that runs the loader — a dedicated
  `dictionary_module_anchor` function — resolved with
  `GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
  GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT)`, then
  `GetModuleFileNameW` for the directory. The mechanism works under any
  deployment file name, versioned or not.
- **Solidified installer/uninstaller (T-026).** `install.ps1` runs the
  deployment as one transaction: copy the new DLL as `zhu-ye-ime-<Version>.dll`
  (or `zhu-ye-ime-<sha256-8>.dll` when `-Version` is omitted), statically
  verify the PE header (MZ magic + `e_lfanew` PE signature) before the
  `LoadLibraryEx(DONT_RESOLVE_DLL_REFERENCES)` export check — the loader call
  alone accepts some invalid files, so the header check is mandatory — copy the
  dictionary (skipped when source equals target), then switch the CLSID
  `InProcServer32`. The previous DLL path is snapshotted before the switch, so a
  late failure rolls back `InProcServer32`/`IconFile` to the old build or, when
  there is none, removes the registration entirely. Old versioned DLLs go
  through `Add-TsfDelayedCleanup`: immediate delete when unlocked, otherwise a
  deferred rename to `<name>.zy-del` that the next install/uninstall sweeps.

## Alternatives considered

**Stop the locking process before overwrite.** Rejected: no reliable lock holder
exists (`handle64.exe` showed many; some are transient), and killing unrelated
processes degrades the interactive session used for acceptance.

**Always remove the old versioned DLL right after the registry switch.**
Rejected: a process that still had the old image mapped (e.g. a lingering
Notepad) keeps using the file; immediate delete fails or corrupts that session.
Deferred cleanup with `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` is the end
state. Measured on the VM platform (Windows Server 2019), the delete-only
variant (null destination) fails with `ERROR_PATH_NOT_FOUND` even for valid
locked files — only the rename variant registers — so the deferred mechanism
renames to `<name>.zy-del` and sweeps on the next install/uninstall.

**Keep a plain unversioned name and cache the install directory at first load.**
Rejected: the first load could come from an old image; the module-anchor lookup is
cheap (once, at engine creation) and always reflects the actually-loaded module.

## Consequences

Versioned deployment is exercised on the VM as v4 → v5 without any lock fight,
and `dict-ok path="C:\zhu-ye-test\tsf\dictionary.zyct"` confirms the anchor-based
lookup finds the real dictionary under the versioned file name. Explorer restarts
automatically and loads the registry-pointed build, which doubles as the
upgrade/reload mechanism. The mechanism is now solidified in
`install.ps1`/`uninstall.ps1` (T-026) and passed the full VM drill on
2026-09-24: a text-file "bad DLL" source is rejected by the PE-header check with
the registration untouched and the bad copy removed; v6 → v7 upgrades switch the
registry with zero lock conflicts (old DLLs deleted immediately since nothing
holds them); uninstall clears the registration plus all DLLs/dictionary and
Program Files residue; reinstall restores from a backup dictionary. Explorer
module-load verification is unavailable on the acceptance VM (session 1 is stuck
in Disc state and cannot restart the shell), so mechanism verification relies on
the registry switch + file cleanup + PE/export checks + `Test-TsfRegistration`
instead. The anchor-based dictionary lookup keeps working under any versioned
file name.
