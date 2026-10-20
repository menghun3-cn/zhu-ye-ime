# Agent Note: Stable DLL shell deployment

Status: implemented

[中文](2026-10-20-stable-dll-shell-deployment.zh.md) | English

## Problem

The TSF text service DLL was deployed under a content-addressed name
(`zhu_ye_ime_<sha8>.dll` / `zhu-ye-ime-<sha8>.dll`), and every upgrade
repointed the registry at a different file. Two problems followed.

**The registry pointer moved on every upgrade.** The TSF registration
(`CLSID\<tip>\InprocServer32` and the language profile's `IconFile`) is
read by the host when it activates the input method. After an upgrade, a
newly started host picked up the new DLL, but every host that was already
open kept the old implementation for as long as it stayed open. The
upgrade was therefore only partially visible: the user could see the new
behaviour in a newly opened window and the old behaviour in the window
they were actually looking at.

**The deployment directory accumulated every version.** The upgrade path
had no cleanup step, so the directory grew without bound. Measured on the
development machine after 49 deployments: 50 DLL files totalling 106.2 MB
on disk, of which 48 were unreachable.

This is also how Sogou behaves, and its own binaries say so. Extracted
from `SGTool.exe` (Sogou 15.4.0.1862):

- 「以下程序使用了Sogou输入法,要避免重启,请先关闭这些程序.」
- 「本设置重启应用程序后生效」
- 「上一次的操作需要重启才生效」
- 「立即重启」/「稍后重启」
- 「重启后才能生效」

Sogou therefore does not hot-replace the running code either. It keeps a
stable shell in `C:\WINDOWS\system32\SogouTSF.ime` and a versioned
implementation under `D:\tools\SogouInput\<version>\`, and it tells the
user which programs still hold the old version.

## Decision

The host loads one file whose name never changes, and the registry
pointer never changes either.

```
C:\Program Files\zhu-ye-ime\tsf\
├── zhu_ye_ime.dll                    ← stable shell, the only file a host loads
└── versions\
    └── <sha8>\
        ├── zhu_ye_ime.dll            ← archived implementation of that version
        └── .zhu_ye_ime.dll.replaced  ← the implementation the upgrade replaced
```

An upgrade copies the new build into `versions\<sha8>\` and then replaces
the stable file with `ReplaceFileW`. The registry is not touched: the
pointer already says `zhu_ye_ime.dll`, and it will keep saying so.

`ReplaceFileW` is the Windows API for exactly this job — replacing a file
that is in use. It preserves the old section mappings for the processes
that already have the file open, and puts the new content on disk for
every process that opens it afterwards. Measured on this machine:

- Replacing `zhu_ye_ime_937632A1.dll` while chrome and msedge had it
  mapped succeeded; both processes kept their old mapping, and the file
  on disk became the new build (SHA-256 `9D8188AC…`).
- Loading the stable file from a fresh process succeeded and mapped
  `zhu_ye_ime.dll` (handle `140712931753984`, Win32 0).

One platform detail shaped the implementation: `ReplaceFileW`'s
`lpBackupFileName` must name a real path. Passing `NULL` or an empty
string returns `ERROR_PATH_NOT_FOUND` (Win32 3) on this machine — the
same family of limitation already recorded for `MoveFileEx` in
`scripts/ime-identity.ps1`. The backup path therefore points into the
version archive, which also keeps the replaced implementation.

## Alternatives considered

- **Keep the versioned filename and only change the cleanup**: this fixes
  the directory growth but not the pointer churn, and the user still has
  to guess which windows are running the old code. It also leaves the
  deployment directory listing as a history of builds rather than a
  statement of what is currently loaded.
- **Write a forwarding shell that loads the versioned implementation at
  runtime** (Sogou's structure): this would let the shell resolve a
  version pointer per process, but it adds a second DLL to build, sign
  and debug, and a forwarding layer that every TSF entry point has to
  cross. `ReplaceFileW` reaches the same user-visible outcome — a newly
  opened host runs the new code — without that layer, so the extra
  machinery would buy nothing observable.
- **Ask the user to close every program that uses the input method after
  each upgrade** (Sogou's approach): this is a worse experience than the
  one the current layout already gives, and it is unnecessary here. The
  only remaining gap is that already-open windows keep the old code,
  which is inherent to a running process and is also true of Sogou.
- **Overwrite the stable file in place with `CopyFile`**: rejected by
  measurement. The file is mapped by running hosts, and the write is
  refused (`Access to the path … is denied`).
- **Rename the stable file aside, then copy the new one into the freed
  name**: viable — renaming a mapped DLL succeeds on this machine (it was
  used to move `zhu_ye_ime_937632A1.dll` out from under chrome and msedge)
  — but it leaves a window in which no stable file exists, and it is
  strictly more fragile than `ReplaceFileW`, which is atomic.

## Consequences

- The registry pointer is now invariant. An upgrade never changes
  `CLSID\<tip>\InprocServer32` or the language profile's `IconFile`.
- A newly opened host always runs the current build, with no need to
  restart the shell or the input method framework.
- An already-open host keeps the implementation it started with. This is
  a property of the running process, not of the deployment; Sogou has the
  same property and surfaces it as a prompt.
- The deployment directory holds one stable file plus a bounded set of
  version archives. `Remove-TsfOldVersionDlls` keeps the three most
  recent archives and sweeps the rest.
- Files still held by a running host cannot be deleted, only renamed.
  The existing `Add-TsfDelayedCleanup` path handles this: rename to
  `*.zy-del`, then `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` for the
  reboot-time removal. The five files left on the development machine
  are in that state — they are held by chrome, msedge, explorer, Weixin,
  Feishu, notepad++, Snipaste, WindowsTerminal and the harness itself.

## Verification

- `ReplaceFileW` on a mapped file: succeeded with a real backup path
  (`Win32=203`), failed with `NULL` (`Win32=3`).
- Rename of a mapped file: succeeded, and the holding processes kept
  their old mapping.
- Dry run of `install.ps1` into a temporary directory: first install
  produced `zhu_ye_ime.dll` plus `versions\9d8188ac\`; a second install
  with different content replaced the stable file and archived the new
  version.
- Dry run with the stable file held open by another process: the
  replacement still succeeded.
- `verify-tsf-dll.ps1` on the replaced file: PE header check passed and
  all three exports (`DllGetClassObject`, `DllCanUnloadNow`,
  `dll_probe`) resolved.
- Development machine after migration: the registry pointer reads
  `C:\Program Files\zhu-ye-ime\tsf\zhu_ye_ime.dll` and the file's
  SHA-256 matches the release build (`9D8188AC…`); a fresh process maps
  `zhu_ye_ime.dll`.

## Related

- [Versioned DLL deployment and dictionary location](../architecture/2026-09-23-versioned-dll-deployment-and-dictionary-location.md)
- [Personalized candidate ranking](../feature/2026-10-20-personalized-candidate-ranking.md)
