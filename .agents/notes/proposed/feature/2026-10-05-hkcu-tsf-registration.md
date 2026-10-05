# Agent Note: HKCU per-user TSF registration, a no-admin acceptance channel (item 5, batch three)

Status: proposed

[中文](2026-10-05-hkcu-tsf-registration.zh.md) | English

## Problem

The language-bar "中/英" visual acceptance item (T-100, T-046) was blocked because the
build host has no administrator rights (`admin=False`), so the `install.ps1` HKLM TSF
registration (`HKLM\SOFTWARE\Microsoft\CTF\TIP` + `HKLM\SOFTWARE\Classes\CLSID`) could
not run, and the Server 2019 acceptance VM has no desktop language bar (T-046 finding).
A user-approved, fully rollback-able trial on the local Win11 26200 host was needed to
either complete the visual acceptance or prove the exact blocking conditions.

## Proposal

TSF text input processors can also be registered per-user. Mirror the HKLM layout under
HKCU (no elevated rights needed):

- DLL and dictionary in the user's own dirs: `%LOCALAPPDATA%\ai-zhu-ye-ime\tsf\`
  (versioned `zhu-ye-ime-<sha8>.dll` + `dictionary.zyct`); the base-dir resolver
  (`tsf.rs resolve_base_dir`) falls back to `%APPDATA%\ai-zhu-ye-ime` for packs, so
  `en.zyen`/extra packs go there.
- `HKCU\SOFTWARE\Microsoft\CTF\TIP\{TipClsid}` with the same sub-trees as HKLM:
  `Category\Category\{KeyboardCategoryGuid}\{TipClsid}` (empty key),
  `Category\Item\{TipClsid}\{KeyboardCategoryGuid}\Description`,
  `LanguageProfile\{0x00000804}\{ProfileGuid}` with `Description` / `Display
  Description` / `Enable` (DWORD 1) / `IconFile` = DLL / `IconIndex` (DWORD 0).
- `HKCU\SOFTWARE\Classes\CLSID\{TipClsid}` (default = display name) +
  `\InprocServer32` (default = DLL path, `ThreadingModel` = `Apartment`).
- Forensics without admin: `config.json` under `%APPDATA%\ai-zhu-ye-ime` with
  `log_level: "debug"` turns on the product log track
  (`%LOCALAPPDATA%\ai-zhu-ye-ime\logs\ime.log`), since the `C:\zhu-ye-test` sentinel
  path needs an admin writable root.
- Verify the registration is genuinely accepted by the system, without any window
  focus, through the TSF COM API: get the profiles object from
  `msctf.dll!TF_CreateInputProcessorProfiles` (export, no COM class GUID needed), QI the
  object as `ITfInputProcessorProfiles` — authoritative IID from the local
  windows-0.61.3 crate (`Windows.Win32.UI.TextServices`):
  **IID_ITfInputProcessorProfiles = {1F02B6C5-7842-4EE6-8A0B-9A24183A95CA}** — and call
  `EnableLanguageProfile(clsid, 0x0804, profile, 1)` and `ActivateLanguageProfile`.
  Method order (vtable slots after IUnknown) taken from the same crate: Register,
  Unregister, AddLanguageProfile, RemoveLanguageProfile, EnumInputProcessorInfo,
  GetDefaultLanguageProfile, SetDefaultLanguageProfile, **ActivateLanguageProfile**,
  GetActiveLanguageProfile, GetLanguageProfileDescription, GetCurrentLanguage,
  ChangeCurrentLanguage, GetLanguageList, EnumLanguageProfiles, EnableLanguageProfile,
  IsEnabledLanguageProfile.
- Rollback is the inverse script: delete the two HKCU trees, restore the default input
  method override, remove `config.json`/user dirs, restart `ctfmon`.

## Results (local Win11 26200, session 1, no admin)

- `EnableLanguageProfile` → S_OK and `GetCurrentLanguage` → 0x0804: the system accepted
  the HKCU-registered TIP.
- `ctfmon` loaded the user DLL and instantiated the engine (multi-thread dict-init lines
  in `ime.log`); DLL export verification passed.
- Competing pre-existing HKCU TIPs (`{81D4E9C9…}`, `{8613E14C…}`) confirm the channel is
  actively used by other IMEs on this machine.
- Cleanup restored the machine exactly: HKCU TIP back to the two original keys,
  `Get-WinDefaultInputMethodOverride` back to unset, `ctfmon` restarted single-instance.

## Visual acceptance remains blocked (two independent causes)

1. **No interactive input focus in this session.** `GetForegroundWindow` stays at a
   stale value or 0; `SetForegroundWindow` is refused; ALT-key simulation and
   `SPI_SETFOREGROUNDLOCKTIMEOUT=0` do not unlock. Three host attempts failed
   (cmd.exe/conhost window was 0x0-sized, a WinForms STA-runspace window could be
   created but never gained foreground) — no real keystrokes can be injected, so
   candidate-window screenshots are unobtainable.
2. **Windows 11 removed the classic desktop language bar.** Registering
   `HKCU\Software\Microsoft\CTF\LangBar\ShowStatus` produces no language-bar window
   (same as observed on Server 2019). Therefore the T-046/T-100 language-bar button
   (an `ITfLangBarItemButton`) has no visible UI on Win11; only the system tray input
   indicator exists, which does not host third-party language-bar items.

## Alternatives considered

- **Elevating to admin and using the HKLM installer**: rejected — the build host has no
  elevation path, and asking the user to grant admin only for acceptance contradicts the
  minimal-touch rule for their machine.
- **Waiting for a privileged VM (the Server 2019 acceptance VM)**: rejected for the
  visual check — that VM has no desktop language bar and no interactive input focus for
  real typing (previous rounds proved `ShowStatus` does nothing there).
- **Shipping the binary to another Win10/11 client machine**: not available at the time;
  the local Win11 host was the only client-class machine, so a rollback-able trial there
  was the highest-information move, with the uninstall script kept next to the trial.
- **Faking the keystroke channel via low-level input injection without a window**:
  impossible — TSF needs a foreground text-service client thread; no injection path works
  without session input focus.

## Acceptance criteria

- `ITfInputProcessorProfiles::EnableLanguageProfile(clsid, 0x0804, profile, 1)` returns
  S_OK while the TIP is registered only under HKCU (system accepts the per-user channel).
- `GetCurrentLanguage()` returns 0x0804 after the call sequence.
- `ctfmon` loads the user-directory DLL and the engine initializes (product log shows
  multi-thread dict init lines) with `log_level: "debug"` config.
- DLL export verification passes for the deployed copy.
- Cleanup restores the machine exactly: HKCU TIP tree back to the original keys only,
  `Get-WinDefaultInputMethodOverride` back to unset, `config.json`/user dirs removed,
  `ctfmon` running single-instance.
- Blocked visual items are recorded with the exact blocking causes (no input focus;
  Win11 has no classic language bar), not silently dropped.

## Risks

- Do not run the HKCU trial on a machine you cannot roll back; keep the uninstall script
  with the trial. All registry writes are user-scope and vanish with the profile.
- Restarting `ctfmon` interrupts the input session briefly; harmless on an idle desktop.
- If a future build changes the TIP CLSID/profile GUID, both the HKLM installer and this
  channel must be updated together (single source: scripts/ime-identity.ps1).
