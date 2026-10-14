# Agent Note: Check results speak to users; Apply button gated on found updates (T-161)

Status: implemented

## Problem

Verbatim request on 2026-10-14:

> 点击检查更新时，输入的信息不要包含 清单地址、清单、签名校验、可用更新（未应用...）
> 等信息，如果版本是最新的，提示已经是最新版本，如果不是最新版本，提示可用版本及
> 版本更新的日志，有最新版本后，更新按钮才可以点击

The settings window's "Check for updates" result area pasted the updater's raw
diagnostic output verbatim — manifest URL, manifest summary (schema/published/pack
count), signature verification result, and the `可用更新: [...]（未应用…）` pack
list. The user wants a user-facing verdict, not internal diagnostics, and wants the
"Apply updates" button disabled until a check actually found an update.

## Decision

1. **`check --json` structured contract.** `zhu-ye-updater check` gained a `--json`
   flag that prints one `UpdateStatus` object — the same serde type and semantics as
   `check-once`'s `update_status.json` write (error field on failure; JSON emitted
   even on network/verify failure, process exits 0). Human diagnostic lines are
   preserved for the CLI (`check` without `--json` unchanged), collected in
   `CheckInfo.human_lines` and printed only in human mode. Offline mode
   (`online_update=false`) also emits JSON (`failed` with a clear error) so the
   contract is total.
2. **Settings checks via `run_check`.** The settings window's Check task calls
   `updater check --json` and parses the result into `UpdateStatus`
   (`zhu_ye_core::parse_update_status`, shared with the status file). The result
   area is rendered from structured data, not raw output — the old text path remains
   only for Apply (download/apply progress feedback).
3. **User-facing rendering (`summary_lines`).** Four states, one shared pure
   function used by both the manual-check path and the startup status-file path
   (`update_status_lines` now extends `summary_lines` + a check-time line):
   - failure → `检查失败：{error}`;
   - IME app update → `发现输入法新版本 vX（当前 vY）` + `更新日志：` + note lines,
     plus one line "同时有 N 个词库文件可更新，点「应用更新」更新。" when packs are
     outdated, and the "主程序升级请到官网…" guidance line;
   - up to date → `输入法已经是最新版本（当前 vX）。`;
   - only packs outdated → the line above plus `词库有更新内容（N 个），点「应用更新」更新。`
   Pack ids and `latest_version` dates are no longer shown (packs are summarized by
   count only) — the pack dimension stays actionable without leaking internals.
4. **Apply button gated on found updates.** `can_apply_updates` = old
   `can_run_updater` AND `check_has_update` (most recent check result first, else the
   local status file). No update found → button disabled (drawn disabled, hit
   ignored), consistent with the "latest version" message right above it.
5. **Deployment coupling.** New settings against an old updater fails the JSON parse
   and shows a clear parse error in the result area. settings and updater must ship
   to the install slot as a pair (T-161 deployment rebuilds both; the updater's
   embedded public key is extracted from the deployed binary, it is a public key —
   `832dc4a5…67cce` — not a secret).

## Alternatives considered

1. **Parse human output in settings** (regex/scans for 「清单地址」「可用更新」…):
   contract lives in localized prose; cheap but brittle. Rejected — a structured
   contract is one serialization away and already exists (`UpdateStatus`).
2. **Manual check via `check-once`** (reuse the status file): `check-once` exits
   early inside the 7-day interval, so a manual "Check now" could silently skip the
   network check. Rejected.
3. **Keep raw output, just hide rows**: filtering UI rows re-implements parsing in
   the view layer and cannot drive the button gate robustly. Rejected.

## Consequences

- `zhu-ye-updater`: `check [--json]`; `CheckInfo` collects manifest/app-meta/outdated
  + human lines; `to_status()` mirrors `check-once`'s status synthesis.
- `zhu-ye-settings`: `OutcomePayload::{Summary, Text}` in `UpdateOutcome`; new
  `run_check`/`parse_check_output` in the updater bridge; `summary_lines`
  (manual + startup rendering), `can_apply_updates`/`check_has_update`/
  `status_has_update`.
- Tests: updater JSON roundtrip; settings `parse_check_output` (valid + invalid),
  `summary_lines` (all four states + no-internal-words assertions), `update_status_lines`,
  `status_has_update` (temp-dir status files). settings 128 / updater 3 green.
- Verified against the real feed: `check --json` → `available=true`,
  `outdated_packs=[it,med,slang]`, no error (real public key verification passed).
- Future note-takers: the "Apply" gate truth is the `UpdateStatus` fields
  `app_version`/`outdated_packs`; keep the shared `summary_lines` as the single
  copy of user-facing copy.

## Related notes

- [Online updates on by default; IME version + changelog via signed app.json (T-154)](../../implemented/feature/2026-10-13-online-update-default-on-app-meta.md)
  — contract and copy built on top of; both remain valid.
- [Settings about-and-update page (T-143/T-150)](../../implemented/feature/2026-10-02-settings-about-and-update-page.md)
  — page structure and the D-44 single-network-component boundary this task keeps.
- Acceptance: docs/验收标准.md §22 (online update); todos-list T-161.
