# Agent Note: Stable shell with runtime reload of versioned implementation

Status: implemented

[中文](2026-10-20-stable-shell-runtime-reload.zh.md) | English

## Problem

The T-175 stable-file deployment made the *file name* and the registry
pointer constant, but the host still loaded the **implementation itself**
into its address space. A host that was already open therefore kept the
implementation it had mapped, for as long as it stayed open.

That limitation had a measured cost. `user_words.json` is a single file
shared by every host. T-174 raised its format to v2 (adding a `transitions`
section), but nine long-lived processes still held a **v1** DLL and wrote the
file with a full overwrite on every committed candidate — writing `version: 1`
back and discarding the entire `transitions` section. The transition table
could therefore never accumulate. The v1 DLLs were the cause, and no
deployment trick could remove them from already-running processes.

Making the change reach an open host requires replacing the *implementation
inside* a host, not the file the host loaded.

Two observations constrained the design, both taken from the TSF debug log
at `C:\zhu-ye-test\tsf-debug.log`.

**A host does not re-fetch the class factory when it regains focus.** For
`pid=35348` the log records one class-factory request against three
`ActivateEx` calls, spanning three complete focus cycles including a real
`nihao` → 「你好」 commit. The DLL-level "shell" that Sogou uses — where the
host loads a thin loader which then loads the versioned DLL — cannot work
here: clicking away and back does not re-resolve anything.

**The external TSF profile APIs have no effect.** `DeactivateProfile`,
`ActivateProfile` and `ReleaseInputProcessor` with
`TF_RIP_FLAG_FREEUNUSEDLIBRARIES` all return `S_OK` and change nothing: the
input method stays active and the same nine processes keep their old DLLs
mapped.

One API *does* re-fetch the class factory: switching the input method
profile. For `pid=33100` the log records `call#1`/`call#2`/`call#3`, the
third after a `DeactivateProfile`. That is a valid reload trigger, but it
requires the user to act, which is exactly what the deployment is supposed
to avoid.

## Decision

The stable file the host loads contains no input method logic. It is a
**shell** that holds a proxy COM object; the proxy's inner implementation
pointer is swappable. The implementation lives in a version directory and
is selected by a pointer file.

```
C:\Program Files\zhu-ye-ime\tsf\
├── zhu-ye-ime.dll              ← the shell: the only file a host loads
└── versions\
    ├── current.txt             ← pointer: 8 hex digits of the active version
    └── 6fa6af97\
        └── zhu-ye-impl.dll     ← the implementation of that version
```

Because COM is itself a stable cross-module ABI, the proxy needs no custom
ABI. It forwards the exact interface set the host queries: 24 methods across
`ITfTextInputProcessorEx` (8), `ITfKeyEventSink` (6), `ITfCompositionSink`
(1), `ITfFunction` (1), `ITfFnConfigure` (1), plus `Activate` and
`Deactivate` from `ITfTextInputProcessor`.

**Publishing is writing `current.txt`.** The pointer is written to a
`.staging` file and renamed, so a host never reads a half-written pointer.
Writing the pointer is the release; no registry write is involved, because
`InprocServer32` and the language profile's `IconFile` keep pointing at the
stable name forever.

**The swap happens on `Activate`.** Every activation — each time the host
gains input focus — the shell reads the pointer and compares it with the
implementation it is currently proxying. If they differ, the shell loads the
new implementation, builds a new text service object and swaps it in, then
releases the old one. At that moment the old implementation has already been
`Deactivate`d and holds no composition session, so replacing it is safe.
When the pointer has not changed the check costs one small file read and
nothing else; the key hot path performs no file I/O.

**Failure degrades rather than breaks.** If the new implementation fails to
load — a missing file, a bad export, an unloadable module — the shell keeps
the old implementation running and records the error. Staying usable
outranks being current.

**The brand icon lives in the shell.** The stable name is what `IconFile`
resolves to, so `build.rs` injects the 「竹」 icon resource into the shell
crate's output.

Two crates now exist. `zhu-ye-ime-shell` produces the shell; `zhu-ye-ime`
produces the implementation. `zhu-ye-ime` is a leaf `cdylib` that nothing
depends on, so the split is safe — the shell does not link the
implementation, it loads it at runtime by path.

## Alternatives considered

**A DLL-level shell that loads the versioned implementation and re-exports
it.** This is Sogou's structure and it was the original plan. It is
disproved by the `pid=35348` evidence: the host does not re-request the
class factory on focus change, so the shell's already-loaded module stays
loaded and no new version is ever consulted. Kept as a rejected candidate
because it is the obvious first design and the measurement is what rules it
out.

**Reloading via the external TSF profile APIs.** `DeactivateProfile`,
`ActivateProfile` and `ReleaseInputProcessor` all return `S_OK` and do
nothing. Measured, not assumed.

**Making the user switch the input method profile to trigger a reload.**
This does re-fetch the class factory (`pid=33100`), but it requires user
action, which is the problem being solved. Rejected as a primary mechanism;
it remains the fallback that any profile-switching user performs anyway.

**Copying the new implementation over the stable name and hoping the host
re-reads it.** The host has already mapped the file; replacing the file on
disk does not change what is mapped. Measured in T-175: `ReplaceFileW`
succeeds and the disk content changes, but already-open hosts keep the old
handle.

**A custom ABI between shell and implementation.** Unnecessary: COM is
already a stable cross-module ABI, and the host's own interface set is
what the proxy must implement anyway. A custom ABI would have added a
version-negotiation surface with no benefit.

**Swapping the implementation on every keystroke instead of on `Activate`.**
The key hot path must not perform file I/O or lock contention. `Activate` is
the earliest point at which the host has released its composition session,
which makes it both cheap and safe.

## Consequences

**An open host picks up a new version without reopening anything.** The
version pointer is consulted on every activation, so the next time the user
clicks into a field the host proxies to the new implementation. The
deployment no longer needs to ask the user to restart anything, which is
the difference from Sogou, whose own `SGTool.exe` carries "立即重启" /
"稍后重启" prompts precisely because its shell cannot do this.

**The shared `user_words.json` conflict is not resolved by this change
alone.** The v1 DLLs in the nine running processes keep writing v1 until
those processes restart and load the shell. The conflict disappears only
once every host runs the shell — which is the planned system restart, not
this task. This task removes the *mechanism* that made the conflict
permanent; the restart removes the *instances*.

**Two DLLs ship instead of one.** The release package contains
`zhu-ye-ime.dll` (the shell) and `zhu-ye-impl.dll` (the implementation).
The deployment scripts, the package layout and the payload guard were
updated accordingly.

**The shell is a second thing that can go wrong.** A malformed pointer, a
missing implementation or a broken module must not take the input method
down. The integration tests cover the malformed-pointer and
missing-implementation paths explicitly, and the failure path keeps the old
implementation alive.

**Order of release in `install` matters.** The COM object must be released
*before* its module is unloaded. Assigning the module handle first would
unload the old implementation's code while its COM object was still alive,
and the object's later `Release` would land in unmapped memory — an access
violation (`0xC0000005`). This was reproduced during development and the
assignment order is now fixed and commented at the call site.

## Testing

- `cargo clippy -p zhu-ye-ime-shell --all-targets -- -D warnings` clean.
- Six unit tests in the shell crate: pointer missing, pointer round-trip,
  invalid version id rejected without writing to disk, on-disk garbage
  rejected, implementation path lands inside the version directory, shell
  directory resolves.
- Four integration tests (`crates/zhu-ye-ime-shell/tests/shell_integration.rs`):
  the shell loads a versioned implementation and returns a proxy; the proxy
  can be queried for every interface the host uses; **changing the version
  pointer swaps the implementation inside the same process** (asserting the
  old module is unloaded and the new one loaded); a malformed pointer does
  not crash the shell; a pointer to a nonexistent implementation does not
  crash the shell.
- The integration tests were re-run against the **deployed files**
  (`C:\Program Files\zhu-ye-ime\tsf\zhu-ye-ime.dll`, 282,112 bytes, and
  `versions\6fa6af97\zhu-ye-impl.dll`, 2,321,920 bytes) and passed.
- On the development machine the shell was loaded by a real TSF host, which
  logged `zhu-ye-shell: 已换用实现 6fa6af97`, confirming that the
  cross-DLL chain works against a real host rather than only in a harness.

## Deferred

The interactive acceptance — that an already-open host visibly switches to
the new implementation after the pointer changes — is still outstanding: the
session was locked during the deployment (`LockApp` held the foreground) and
synthetic input cannot wake it. The mechanism is verified at process level
by the integration tests; the end-to-end UI check is a separate step.

## Related

- [Stable DLL shell deployment](2026-10-20-stable-dll-shell-deployment.md)
  (T-175) — the constant file name and registry pointer this builds on. Its
  `ReplaceFileW` replacement, `versions\<sha8>\` archive and delayed-cleanup
  mechanisms remain in force; what changed is that the archived file is now
  the implementation and the stable name is now the shell.
- [Versioned DLL deployment and dictionary location](2026-09-23-versioned-dll-deployment-and-dictionary-location.md)
  (T-026) — superseded in part by T-175, and the file naming it introduced is
  superseded again here.
- [Taskbar icon 「竹」](2026-10-06-taskbar-icon-zhu.md) — the icon resource is
  now embedded in the shell, because the stable name resolves to the shell.
