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

## Decision

The host loads one file whose name never changes, and the registry
pointer never changes either.

```
C:\Program Files\zhu-ye-ime\tsf\
├── zhu-ye-ime.dll                    ← stable file, the only one a host loads
└── versions\
    └── <sha8>\
        ├── zhu-ye-ime.dll            ← archived implementation of that version
        └── .zhu-ye-ime.dll.replaced  ← the implementation the upgrade replaced
```

> **Updated by T-176.** The layout above is the T-175 form. Under the shell
> architecture the stable file holds the **shell** rather than the
> implementation, the archived file is named `zhu-ye-impl.dll`, and a
> pointer file `versions\current.txt` selects the active version:
>
> ```
> C:\Program Files\zhu-ye-ime\tsf\
> ├── zhu-ye-ime.dll              ← the shell: the only file a host loads
> └── versions\
>     ├── current.txt             ← pointer: 8 hex digits of the active version
>     └── <sha8>\
>         └── zhu-ye-impl.dll     ← the implementation of that version
> ```
>
> The mechanism described below — `ReplaceFileW` against the stable name,
> the `versions\<sha8>\` archive, the delayed cleanup — is unchanged and
> still in force. See
> [Stable shell with runtime reload of versioned implementation](2026-10-20-stable-shell-runtime-reload.md).

An upgrade copies the new build into `versions\<sha8>\` and then replaces
the stable file with `ReplaceFileW`. The registry is not touched: the
pointer already says `zhu-ye-ime.dll`, and it will keep saying so.

`ReplaceFileW` is the Windows API for exactly this job — replacing a file
that is in use. It preserves the old section mappings for the processes
that already have the file open, and puts the new content on disk for
every process that opens it afterwards. Measured on this machine:

- Replacing `zhu_ye_ime_937632A1.dll` while chrome and msedge had it
  mapped succeeded; both processes kept their old mapping, and the file
  on disk became the new build (SHA-256 `9D8188AC…`).
- Loading the stable file from a fresh process succeeded and mapped
  `zhu-ye-ime.dll` (handle `140712931753984`, Win32 0).

One platform detail shaped the implementation: `ReplaceFileW`'s
`lpBackupFileName` must name a real path. Passing `NULL` or an empty
string returns `ERROR_PATH_NOT_FOUND` (Win32 3) on this machine — the
same family of limitation already recorded for `MoveFileEx` in
`scripts/ime-identity.ps1`. The backup path therefore points into the
version archive, which also keeps the replaced implementation.

## Scope of this decision

This decision covers the **stable file identity and the upgrade
mechanism**. It does not make an already-open host run new code, and it
was not intended to. See "Limit of the guarantee" below.

## Limit of the guarantee

A host that has already loaded the file keeps running the bytes that were
mapped at that moment. This is not an implementation shortcoming; it is
the documented behaviour of the Windows loader.

Measured on the development machine after the migration: nine processes
still mapped the old DLLs (`notepad++` → `zhu_ye_ime_9ACBCEF1.dll`,
`Weixin` → `zhu_ye_ime_A4277E6B.dll`, and so on), and **the files those
processes mapped no longer exist on disk** — they were deleted by the
cleanup path. The processes kept executing the old code from memory pages
that Windows keeps alive for them until the process exits.

> **Updated by T-176.** This limit applies to the stable file as it was
> under T-175, where the stable file *was* the implementation. Under the
> shell architecture the stable file is a shell that loads the
> implementation at runtime and swaps it on activation, so an already-open
> host does pick up a new implementation without reopening anything. The
> nine processes above still hold v1 implementations until they restart,
> which is why the shared `user_words.json` conflict persists until the
> planned system restart. See
> [Stable shell with runtime reload of versioned implementation](2026-10-20-stable-shell-runtime-reload.md).

In-process reload is technically reachable: a controlled experiment showed
that `FreeLibrary` followed by `LoadLibraryW` returns a handle to the
newly replaced content within the same process. Reaching it in a real
host, however, requires the host to release every COM reference it holds
on the text service and then to re-activate the input method, which is
the host's decision, not ours. `ITfInputProcessorProfileMgr::ReleaseInputProcessor`
(`msctf.h`) is the TSF-side entry point for dropping that reference.

## Sogou's structure, measured

Sogou 15.4.0.1862 was inspected on this machine because the user
reported that Sogou does not require re-opening a host after an update.
The measurement shows a three-layer structure, and the top layer is
exactly the piece this note does not have:

| Layer | File | Loaded by | After an update |
| --- | --- | --- | --- |
| Stable shell | `C:\WINDOWS\system32\SogouTSF.ime` (4,672,832 B) | every host, via `CLSID {E7EA138E-…}` | never changes |
| Input method implementation | `D:\tools\SogouInput\15.4.0.1862\SogouTSF.dll` (96 KB) + `ImeFunc.dll` (2,644 KB) | the shell, at runtime | replaceable |
| Resources and helpers | `Resource.dll`, `SearchCand.dll`, … | some hosts directly | partially stale |

The shell carries its own loader — the strings
`UserEnv::t_envContainer::t_loader::Load`, `GetEnv`, `CopyFrom` and
`DumpFrom` are present in it, together with `LoadLibraryA`,
`LoadLibraryW`, `LoadLibraryExW` and `GetProcAddress` — and it locates
the version directory through `Software\SogouInput`. The shell references
`SogouTSF.dll` by bare name, and that DLL exists only inside the version
directory, so the shell resolves the version directory at load time and
loads the implementation from there. Component directories are versioned
per component (`Components\IChat\` holds `1.0.2.3040`, `1.0.2.3099` and
`1.0.2.3127` side by side), which is what makes a per-process version
switch possible at all.

The distinction that matters: **the host depends on the shell, and the
shell is what loads the implementation.** A host that loaded the
implementation directly — as this project does — freezes the
implementation in its own address space at load time and cannot be
upgraded in place.

Sogou also surfaces the remaining gap in its own binaries: `SGTool.exe`
carries 「以下程序使用了Sogou输入法,要避免重启,请先关闭这些程序.」 and
「立即重启」/「稍后重启」, so the shell's version switch does not cover
every process either.

### The shell-layer premise was measured and holds

Whether the shell layer can work depends on one thing: **does the host ask
TSF for the text service class factory again when the input method is
switched?** That was measured on this machine.

A call counter was added to `DllGetClassObject` and the probe build was
deployed under the stable name (`zhu-ye-ime.dll`, SHA-256 `08BD6566…`). A
program that imitates a TSF host then walked the full chain —
`LoadLibrary → DllGetClassObject → CreateInstance → ActivateEx →
Deactivate` — and called
`ITfInputProcessorProfileMgr::DeactivateProfile` after the first round to
simulate an input method switch. The sentinel log
(`C:\zhu-ye-test\tsf-debug.log`) recorded:

```
[1792463916] pid=33100 zhu-ye: DllGetClassObject call#1   ← round one
[1792463917] pid=33100 zhu-ye: DllGetClassObject call#2   ← round two
[1792463917] pid=33100 zhu-ye: DllGetClassObject call#3   ← after the profile switch
```

**The class factory is requested again every time a text service is
created, including after an input method profile switch.** The same holds
for a real host: msedge (PID 24300) mapped the stable `zhu-ye-ime.dll` and
recorded `call#1`.

This establishes the premise the shell layer needs: **the shell can decide
which implementation a session uses each time it is asked for a class
factory.** The host does not have to be reopened — it switches the input
method and switches back, asks the shell for a fresh class factory, and the
shell hands it the current implementation.

A companion in-process module experiment confirmed the shell's ability to
change versions: implementations at different paths are independent modules
(`LoadLibrary(v1)` and `LoadLibrary(v2)` yield different handles), and
`FreeLibrary` followed by `LoadLibrary` yields the module at that path.

### Two points the shell layer still has to settle

1. **COM object lifetime.** The host holds interface pointers, so the
   shell must not unload an implementation while the host still holds
   pointers into it — that would crash on the next call. The correct
   strategy is therefore **per-session pinning**: the implementation
   returned by a given `DllGetClassObject` stays loaded for as long as that
   text service lives, and a new version takes effect only in the next
   session. The cost is that a host that has used an old version keeps one
   extra DLL's worth of memory mapped.
2. **Reading the version pointer.** The shell must resolve the current
   version **per process** rather than baking it in, or different hosts
   would disagree about the version. Sogou achieves the same thing through
   the `Software\SogouInput` registry key.

## Alternatives considered

- **Keep the versioned filename and only change the cleanup**: this fixes
  the directory growth but not the pointer churn, and the user still has
  to guess which windows are running the old code. It also leaves the
  deployment directory listing as a history of builds rather than a
  statement of what is currently loaded.
- **Write a forwarding shell that loads the versioned implementation at
  runtime** (Sogou's structure): this is the only structure that lets an
  already-open host pick up a new implementation, and it is the natural
  successor to this note. It was not adopted here because the stable-file
  identity and the cleanup problem are worth fixing on their own, and
  because the shell only pays off once the host-side release-and-
  re-activate path is validated. The shell must be a separate artifact
  that forwards every TSF entry point, and the version pointer it
  resolves must be read per process, not baked in.
- **Ask the user to close every program that uses the input method after
  each upgrade** (the prompt Sogou shows): a worse experience than the
  one the current layout already gives, and unnecessary here.
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
- An already-open host keeps the implementation it started with.
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
  produced `zhu-ye-ime.dll` plus `versions\9d8188ac\`; a second install
  with different content replaced the stable file and archived the new
  version.
- Dry run with the stable file held open by another process: the
  replacement still succeeded.
- `verify-tsf-dll.ps1` on the replaced file: PE header check passed and
  all three exports (`DllGetClassObject`, `DllCanUnloadNow`,
  `dll_probe`) resolved.
- Development machine after migration: the registry pointer reads
  `C:\Program Files\zhu-ye-ime\tsf\zhu-ye-ime.dll` and the file's
  SHA-256 matches the release build (`9D8188AC…`); a fresh process maps
  `zhu-ye-ime.dll`.

## Related

- [Versioned DLL deployment and dictionary location](../architecture/2026-09-23-versioned-dll-deployment-and-dictionary-location.md)
- [Personalized candidate ranking](../feature/2026-10-20-personalized-candidate-ranking.md)
