# Agent Note: Distribution package layout and installer semantics

Status: implemented

[中文](2026-10-02-distribution-package-layout-and-installer.zh.md) | English

## Problem

FR-045 requires the settings window to reach clean machines: the old
`install.ps1` built the DLL from the source tree (falling back to an on-site
`cargo build`), copied only the TSF service and one dictionary, created no
`packs/`, and made no shortcut. It could neither ship `zhu-ye-settings.exe`
nor serve as a repair/upgrade backend, and nothing on the target machine was
installed that the settings window could later re-run to fix itself.

## Decision

`package-portable.ps1` now produces a **distribution package** with a fixed
layout, and `install.ps1` reads from it instead of the source tree:

```
zhu-ye-ime-<version>-test/
  bin/zhu_ye_ime.dll              TSF service DLL (versioned copy at install)
  bin/zhu-ye-settings.exe         settings window
  bin/zhu-ye-updater.exe          dictionary updater (only networked component)
  bin/dictionary.zyct             base dictionary (data/artifacts/base.zyct)
  packs/it.zyct med.zyct slang.zyct   preseeded domain packs (D-46, offline verifiable)
  scripts/ime-identity.ps1 install.ps1 uninstall.ps1
          verify-tsf-dll.ps1 verify-tsf-identity.ps1
  docs/licenses.md 数据清单.md
```

- `install.ps1`: resolves `-PackageRoot` (default: parent of the script dir),
  validates the package contents, versioned-copies and export-verifies the DLL,
  copies the base dictionary (`bin/dictionary.zyct`), installs the two exes to
  `%ProgramFiles%\zhu-ye-ime\bin\`, preseeds the three domain packs to
  `%APPDATA%\zhu-ye-ime\packs\`, registers the TSF trees with the original
  rollback transaction, and creates the Start Menu shortcut (D-26 launch entry).
  `-SkipBuild` is now a no-op (the package mode never builds). The new
  `-SkipRegistration` switch skips HKLM registration and the shortcut, so the
  copy/preseed logic is exercisable without an admin shell or a TSF write.
  Idempotent: same DLL hash overwrites, exes and packs overwrite.
- `uninstall.ps1`: additionally removes the two exes (delayed cleanup when in
  use), the Start Menu shortcut, and empty `tsf\`/`bin\`/app-root directories.
  **`%APPDATA%\zhu-ye-ime` is deliberately kept** — configuration, domain
  packs and the user dictionary are user data; uninstall removes program files
  only.
- The base dictionary source changes: `install.ps1` previously preferred
  `data/artifacts/real.zyct` (dev artifact); the package mode ships
  `base.zyct` (the read-only base pack, per the architecture doc layout).

## Alternatives considered

**Keep install-time building and make package-portable produce the old format
plus the exes.** Rejected: FR-045 explicitly requires "no dependency on the
source directory and cargo" on the clean machine; keeping a build path would
preserve two code paths in the installer (source-tree vs package) that the
acceptance run cannot distinguish.**Uninstall removes `%APPDATA%\zhu-ye-ime` entirely.** Rejected: the user
dictionary and configuration are user data accumulated over time; deleting
them on uninstall would violate the same "never delete user data" principle
as the level-1 repair (D-41). Manual removal is documented in
docs/安装与使用.md.

**Shortcut always created even in the dry-run.** Rejected: the
`-SkipRegistration` drill would leak a Start Menu entry pointing at a
temporary target; the switch groups all system-level side effects.

## Consequences

- A clean machine needs no Rust toolchain: unzip and run `install.ps1`
  (acceptance 13.1 FR-045 rows).
- The installer now writes only to deterministic locations
  (`Program Files\zhu-ye-ime\{tsf,bin}`, Start Menu, `%APPDATA%`), which
  the settings window can re-verify in 管理输入法.
- The D-46 preseeded packs make FR-042 verifiable offline; online updates
  remain as the upgrade path (design §8).
- The dry-run switch (`-SkipRegistration`) is a drill hook: it skips HKLM
  writes and the shortcut but still exercises copy/preseed/validation; the VM
  acceptance runs the full path as an admin.
- Uninstall keeps user data by design; the acceptance row "卸载后注册表与
  文件无残留" is scoped to program files and registration, documented in
  docs/安装与使用.md §9.

## Related notes

- [TSF registration and lifetime](../architecture/2026-09-18-tsf-registration-and-lifetime.md):
  the registry trees, rollback transaction and versioned-DLL cleanup semantics
  are unchanged; this note only changes where the installer reads its inputs.
- [Portable scripts encoding](../bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md):
  the UTF-8 BOM convention still applies to every script shipped in the package
  (package-portable re-writes them with BOM).
