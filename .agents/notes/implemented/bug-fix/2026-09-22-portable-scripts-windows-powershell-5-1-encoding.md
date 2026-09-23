# Agent Note: Portable package scripts compatible with Windows PowerShell 5.1

Status: implemented

[中文](2026-09-22-portable-scripts-windows-powershell-5-1-encoding.zh.md) | English

## Problem

VM acceptance of the portable package stopped before TSF installation could start: running `./scripts/install.ps1 -SkipBuild` produced PowerShell parser errors, and the Chinese messages appeared as mojibake. The packaged `.ps1` files are UTF-8 without BOM; Windows PowerShell 5.1 on Windows 11 decodes them with the legacy ANSI code page, so multi-byte Chinese comments and string literals become invalid syntax and string terminators are lost. Local PowerShell 7 did not expose the problem because its default decoding is UTF-8.

## Decision

All repository PowerShell scripts (`scripts/*.ps1` and `.agents/skills/git-publish/scripts/pre-publish-check.ps1`) now carry a UTF-8 BOM. `scripts/package-portable.ps1` re-encodes the four scripts it copies into the portable package with `[System.Text.UTF8Encoding]::new($true)`, so future packages keep the fix even if a source file loses its BOM. `README-测试.txt` is also written with UTF-8 BOM so the test guide opens correctly under Windows tools.

The repacked zip was verified before replacement: extracted scripts start with a BOM, and `install.ps1` parses under Windows PowerShell 5.1 and shows correct Chinese in the requires-admin error path.

## Alternatives considered

**Convert scripts to ASCII-only, removing Chinese comments and strings.** Rejected: the project convention mandates Chinese comments and user-facing messages; stripping them would reduce maintainability, and the same trap would return with the next Chinese string.

**Rely on `chcp 65001` or console font settings.** Rejected: parsing happens before script body execution and is controlled by the invoking host; a `chcp` call cannot change how the file itself is decoded.

**Keep no BOM and require PowerShell 7.** Rejected: the VM and many end-user machines default to Windows PowerShell 5.1, so the portability target controls.

## Consequences

T-025 is complete: scripts in the portable package parse under Windows PowerShell 5.1, and the parser blocker on the Windows 11 VM is removed. Existing script content is unchanged apart from the BOM bytes, and future `package-portable.ps1` output remains compatible. Real TSF registration still requires VM execution with administrator privileges; this fix only clears the encoding barrier before `T-010`/`T-011` VM acceptance.
