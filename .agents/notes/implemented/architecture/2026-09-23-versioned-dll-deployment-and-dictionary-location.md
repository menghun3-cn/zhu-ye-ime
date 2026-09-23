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

## Alternatives considered

**Stop the locking process before overwrite.** Rejected: no reliable lock holder
exists (`handle64.exe` showed many; some are transient), and killing unrelated
processes degrades the interactive session used for acceptance.

**Always remove the old versioned DLL right after the registry switch.**
Rejected: a process that still had the old image mapped (e.g. a lingering
Notepad) keeps using the file; immediate delete fails or corrupts that session.
Deferred cleanup with `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` is the intended
end state in T-026.

**Keep a plain unversioned name and cache the install directory at first load.**
Rejected: the first load could come from an old image; the module-anchor lookup is
cheap (once, at engine creation) and always reflects the actually-loaded module.

## Consequences

Versioned deployment is exercised on the VM as v4 → v5 without any lock fight,
and `dict-ok path="C:\zhu-ye-test\tsf\dictionary.zyct"` confirms the anchor-based
lookup finds the real dictionary under the versioned file name. Explorer restarts
automatically and loads the registry-pointed build, which doubles as the
upgrade/reload mechanism. T-026 is tracked in the todos list to solidify the
mechanism in `install.ps1`/`uninstall.ps1` (copy new version, switch registry,
`MoveFileEx` delayed cleanup, rollback path).
