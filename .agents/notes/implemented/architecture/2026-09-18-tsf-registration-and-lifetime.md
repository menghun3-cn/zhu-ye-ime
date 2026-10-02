# Agent Note: TSF registration and COM lifetime for M1

Status: implemented

[中文](2026-09-18-tsf-registration-and-lifetime.zh.md) | English

## Problem

A TSF text input processor is not usable until Windows can discover its CLSID, load
its DLL in a host process, and unload it after the last reference. The registration
contract spans Rust code, installer scripts, and the system registry; without a fixed
identity and a verified lifecycle, installing or uninstalling the IME would corrupt
other text services or leave stale references.

## Decision

`zhu-ye-ime` is a Rust `cdylib` named `zhu-ye-ime.dll` in its install directory and
exports exactly two COM entry points, `DllGetClassObject` and `DllCanUnloadNow`, plus
the development probe `dll_probe`. All exports use `#[no_mangle] extern "system"`.

The TIP identity is fixed: CLSID
`{E54D6682-8650-40E7-A9EE-6FD1137849AE}`, zh-CN language profile
`{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}`, and keyboard category
`{34745C63-B2F0-4784-8B67-5E12C8701A31}`. The Rust-side single source of truth is
`crates/zhu-ye-core/src/identity.rs` (since T-081, see
[2026-10-02-zhu-ye-ui-crate-and-rlib-removal.md](2026-10-02-zhu-ye-ui-crate-and-rlib-removal.md));
`zhu_ye_ime::tsf` re-exports it, and `scripts/ime-identity.ps1` stays in sync, with
`scripts/verify-tsf-identity.ps1` cross-checking both sides.

Registration is owned by `scripts/install.ps1` and `scripts/uninstall.ps1`, never by
the DLL. Install writes the HKLM TIP key, the keyboard Category `Category` and `Item`
keys, the `LanguageProfile\0x00000804\{ProfileGuid}` profile with `Enable=1`, and
`SOFTWARE\Classes\CLSID\{Clsid}\InProcServer32` with `ThreadingModel=Apartment`.
Uninstall deletes both top-level trees while tolerating absence. Both operations are
idempotent, and install regenerates the registry after removing stale keys so no old
DLL path survives.

COM lifetime is explicit: the class factory implements `IClassFactory`, rejects
aggregation via `CLASS_E_NOAGGREGATION`, and `DllCanUnloadNow` returns `S_OK` only
when both the active-object count and the `LockServer` count are zero. Outer
aggregation is not supported because this text service has no control-unknown
semantics; a null outer pointer is accepted as normal COM non-aggregated creation.

The install target is x64-only process-in-COM under HKLM; the DLL is deployed to
`C:\Program Files\ai-zhu-ye-ime\tsf\zhu-ye-ime.dll`. M1 does not implement key
processing, composition, display attributes, or candidate windows; those enter with
T-011 and T-012.

## Registry contract

```text
HKLM\SOFTWARE\Microsoft\CTF\TIP\{TipClsid}
  Category\Category\{34745C63-B2F0-4784-8B67-5E12C8701A31}\{TipClsid}
  Category\Item\{TipClsid}
    Description = 竹叶输入法
  Category\Item\{TipClsid}\{34745C63-B2F0-4784-8B67-5E12C8701A31}
  LanguageProfile\0x00000804\{ProfileGuid}
    Description = 竹叶输入法
    Display Description = 竹叶输入法
    Enable = 1 (DWORD)
    IconFile = <dll path>
    IconIndex = 0 (DWORD)
HKLM\SOFTWARE\Classes\CLSID\{TipClsid}
  (Default) = 竹叶输入法
  InProcServer32
    (Default) = <dll path>
    ThreadingModel = Apartment
```

The layout was validated against a real Windows 11 HKLM TIP tree; the Rust code and
`scripts/verify-tsf-dll.ps1` verify exports, while `Test-TsfRegistration` verifies
the profile and InProcServer32 after install.

## Portable test packaging

`scripts/package-portable.ps1` assembles a zip under ignored `target/portable/` that
contains the release DLL, the four install/uninstall/verify scripts, and a short test
guide. The test machine can extract and run `scripts/install.ps1 -SkipBuild` without
installing Rust, so TSF registration and uninstall can be validated on a clean or
snapshotted Windows machine before the developer host is changed.

## Alternatives considered

**Register through `LocalServer32` like Microsoft Pinyin.** Rejected: an out-of-process
TSF server adds IPC, lifetime, and service management cost without benefit for M1; the
project's architecture already targets a process-in DLL.

**Write registry keys from the Rust DLL during installation.** Rejected: low-level
registry access under admin is an installer concern, rollout and rollback are easier
in PowerShell, and the DLL remains loadable by ordinary processes without elevation.

**Dynamically generate GUIDs per machine or per install.** Rejected: fixed identity is
required for a stable language profile, update compatibility, and a verifiable
uninstall.

**Register both 32-bit and 64-bit hosts.** Rejected: the project targets x64 for M1;
the 32-bit Wow6432Node registration can be added later without changing the DLL
contract.

**Implement COM interfaces with a hand-written vtable layer.** Rejected: `windows`
0.61 generated bindings provide safe `#[implement]` wrappers, smaller surface, and
the same ABI; a direct `windows-core` dependency makes the macro expansion resolvable
without exposing implementation details.

**Support COM aggregation.** Rejected: no control-unknown requirement exists for a TSF
text service, and rejecting aggregation keeps lifetimes and tests trivial.

## Consequences

Install and uninstall now form a repeatable, idempotent loop with export and registry
checks, so a failed or stale deployment can be recovered by rerunning uninstall.
Unit tests cover factory creation, aggregation rejection, probe exports, and unload
state without touching the system registry.

The scripts require administrator privileges, and `verify-tsf-dll.ps1` maps the DLL
with `DONT_RESOLVE_DLL_REFERENCES` so build-time checks never execute `DllMain`.
Actual host activation still needs a real Windows registration run and is tracked by
T-010; T-010 remains in progress until that installation has been performed once and
the input method appears in the system language list. T-011 adds the first real key
input and text insertion path.
