# Agent Note: Installed dictionary deployment for TSF

Status: implemented

English | [中文](2026-09-21-installed-dictionary-deployment.zh.md)

## Problem

The installer previously copied only the TSF DLL. The runtime searched for a
dictionary only under `%APPDATA%\zhu-ye-ime\seed.zyct`, so a fresh install
or a portable package on another machine fell back to the 20-entry in-memory
demo dictionary. That hid the real dictionary pipeline from VM/real-machine
acceptance and left the packaged test guide out of date.

## Decision

The installer and portable package now ship a v2 dictionary as
`dictionary.zyct` next to the DLL. Source resolution prefers an explicit
`-DictionaryPath`, then the portable staging root, then
`data/artifacts/real.zyct`, then `data/artifacts/seed.zyct`, and builds the
demo seed only when none exists.

`install.ps1` copies the chosen dictionary to the same installation directory
as the DLL, so the product data is machine-wide and uninstallable.
`uninstall.ps1` removes `dictionary.zyct`, the legacy `seed.zyct`, and the DLL.
`package-portable.ps1` bundles the chosen dictionary, the install/uninstall
scripts, the DLL, and the license/data inventory docs.

`zhu-ye-ime` resolves the runtime dictionary in this order: `ZHU_YE_DICT_PATH`
environment override, the DLL's own directory,
`%APPDATA%\zhu-ye-ime\dictionary.zyct`, then `dictionary.zyct` in the
working directory. `create_engine` still falls back to the in-memory demo
dictionary when the file is missing, corrupt, or unverifiable.

## Alternatives considered

**Keep only the per-user `%APPDATA%` copy.** Rejected: machine-wide TSF
installations would need per-user copies, and uninstall could not clean the
product dictionary without touching user data.

**Embed the dictionary in the DLL.** Rejected: it bloats the binary and ties
data updates to a full rebuild, violating the data/engine separation.

**Leave dictionary placement to the test operator.** Rejected: it is fragile
for VMs and invalidates the "copy and run" portable workflow.

## Consequences

A test machine can install from the portable package and immediately use the
real dictionary when it was generated. The package includes attribution and
license docs for the derived dictionary. Uninstall removes product data while
preserving `%APPDATA%\zhu-ye-ime\user_words.json`. The env override keeps a
debug/development escape hatch without changing the installed behavior.
