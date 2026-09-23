# Agent Note: TSF composition write path avoids InsertAtSelection write branch

Status: implemented

[中文](2026-09-23-tsf-insert-at-selection-write-path-crash.zh.md) | English

## Problem

On the acceptance VM (Windows Server 2019, msctf.dll 17763) the IME crashed every
Notepad session with access violation `0xC0000005` (`MSCTF+0x64314`) as soon as a
second key was typed. The local Windows 11 host (msctf.dll 26100) crashed the same
way in a minimal TSF host probe. Bisecting variants in `examples/tsf_min_host.rs`
narrowed the fault to `ITfInsertAtSelection::InsertTextAtSelection` when called on
the **write** branch (`dwFlags` without `TF_IAS_QUERYONLY`) with a **non-null
`pprange`** — the documented way to learn where the inserted text landed. The crash
occurred in both msctf versions on the same call, before any project code touched
the range pointer. Read-only writes via `TF_IAS_QUERYONLY` with a null `pprange`,
`ITfRange::SetText`, `ITfContextComposition::StartComposition`, and
`ITfRange::GetText` all returned cleanly in every variant.

## Decision

The TIP no longer uses the `InsertTextAtSelection` write branch at all.

- Starting a composition: `InsertTextAtSelection(ec, TF_IAS_QUERYONLY, text)` with
  a **null** `pprange` only locates the insertion point range; `ITfRange::SetText`
  writes the pinyin; `ITfContextComposition::StartComposition` on that range opens
  the composition; on failure the composition is ended and cleaned up.
- Updating an existing composition: the stored `ITfComposition::GetRange()` is
  reused with `ITfRange::SetText` (unchanged from the pre-crash design).
- Committing with text: the same QUERYONLY insertion-point range is used with
  `SetText`; the composition is then ended.

Every step logs `comp-insert-begin/ok`, `comp-ccomp-begin/cast-ok`,
`comp-start-ok/err`, `comp-settext-ok/err` so remote VM acceptance can verify the
path from `C:\zhu-ye-test\tsf-debug.log`.

## Alternatives considered

**Keep the write branch and pass a null `pprange`.** Rejected: the variant matrix
showed `InsertTextAtSelection` with write flags and null `pprange` returns
`E_INVALIDARG` (0x80070057) and writes nothing — the write branch is unusable on
both tested msctf builds.

**Use `ITfInsertAtSelection::InsertEmbeddedAtSelection`.** Rejected: it is for
embedded objects, not plain text, and does not address the underlying write-branch
fault.

**Report the crash upstream as an msctf bug and wait.** Rejected: the IME must ship
now; QUERYONLY + `SetText` is a supported, documented combination and removes the
dependency on the broken branch.

## Consequences

The composition, commit, and candidate pipeline no longer crashes on either
msctf build. VM acceptance types `nihao` + space and multiple composed sessions
with zero crashes; the composition range covers newly inserted text (verified with
`GetText` char counts `1` → `2`). The write-branch trap is documented for any
future code path that needs to insert text: prefer `TF_IAS_QUERYONLY` for locating
the insertion point and write through `ITfRange::SetText`.

T-011 remains in progress until the full VM end-to-end acceptance in T-009–T-013
is closed, which is tracked in the todos list.
