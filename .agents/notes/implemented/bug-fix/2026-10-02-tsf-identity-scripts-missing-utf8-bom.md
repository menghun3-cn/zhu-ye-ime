# Agent Note: T-079 identity scripts shipped without the UTF-8 BOM

Status: implemented

[中文](2026-10-02-tsf-identity-scripts-missing-utf8-bom.zh.md) | English

## Problem

The T-079 gate scripts `scripts/verify-tsf-identity.ps1` and (dotted into it)
`scripts/ime-identity.ps1` were committed without the UTF-8 BOM that
`2026-09-22-portable-scripts-windows-powershell-5-1-encoding` established as
the convention for PowerShell 5.1 compatibility. PowerShell 5.1 decodes
BOM-less files as ANSI, so the Chinese comments and strings in both files
mangled the parser; the gate failed with "Unexpected token" on the GUID hash
literal lines and never actually ran its comparisons.

## Decision

Both scripts are re-saved as UTF-8 with BOM (re-encoding only, content
unchanged). The gate now runs and reports the 6/6 identity comparisons on the
host. The convention in the 2026-09-22 note is unchanged and applies to every
new script under `scripts/`; T-079's two files were the gap, because the
gate itself could not execute its own sub-check at the time.

## Alternatives considered

**Rewrite the two scripts in ASCII-only Chinese-free form.**
Rejected: the strings they compare include Chinese labels and the translation
pairs are part of the audit trail; the system is the file encoding, not the
content.

## Consequences

`scripts/verify-tsf-identity.ps1` executes on PowerShell 5.1 as intended.
The T-076 change that needed this gate (settings registry repair writing the
same constants) could be verified locally; any future PR that touches the
identity constants must keep both files BOM-encoded.
