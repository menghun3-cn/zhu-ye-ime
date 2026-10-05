# Agent Note: Productized file logging for TSF diagnostics (T-091)

Status: implemented

[中文](2026-10-04-productized-file-logging.zh.md) | English

## Problem

FR-060 asks for a product-grade diagnostic log. The acceptance-era file log
(`C:\zhu-ye-test\tsf-debug.log`, written unconditionally through `debug_log`
when the sentinel `C:\zhu-ye-test\tsf-debug.enable` exists) does not exist on
user machines, so errors surface only through the debugger output channel
(`OutputDebugStringW`), which is invisible in the field. Any failure path that
only logged went fully undiagnosable after acceptance. Decision D-72/D-73
locked the shape: default level `warn` with zero file writes on the hot path
(D-72), and a dual-track sentinel that keeps the acceptance path byte-identical
to what vm-accept-sop asserts (D-73).

## Decision

Three layers:

- **core (pure std, zero Windows API):**
  - `zhu_ye_core::log_level::LogLevel` — `Error < Warn < Info < Debug`
    (`Ord` backs the level gate), `parse()` is case-insensitive and falls back
    to `Warn` on unknown/empty, serde deserialization is lenient (missing /
    non-string / unknown value → `Warn`; same family as the `theme` widening),
    serialization writes lowercase `as_str()` text.
  - `zhu_ye_core::file_log::FileLogger` — no internal lock (TSF side wraps it
    in a `Mutex`): `new(path, level)`, `with_size_limit` (test injection),
    `write(level, tid, line)` applies the level gate, checks
    `len > size_limit` before every write and rotates (`ime.log` →
    `ime.1.log`, remove stale first), then `ensure_dir()` (idempotent
    `create_dir_all`) and appends. All steps are best-effort (any failure
    silently skips). `format_line(level, tid, message)` renders
    `[{unix_now}] pid={pid} tid={tid} <LEVEL> {message}`; `tid` is passed by
    the caller (core must not depend on Windows thread ids) — a signature
    offset from the design draft's two-parameter `format_line`.
- **ime (tsf.rs):** `product_log(level, message)` — dual track:
  1. sentinel `C:\zhu-ye-test\tsf-debug.enable` exists → write the full
     `C:\zhu-ye-test\tsf-debug.log` line exactly as before (sentinel check
     stays behind the `FILE_LOG_ENABLED`/`FILE_LOG_CHECKED` static cache;
     `debug_log` keeps its old signature and maps to `LogLevel::Debug`);
  2. otherwise → level gate against `config.log_level`, then a lazily
     assembled `FileLogger` behind
     `static PRODUCT_LOGGER: Mutex<Option<FileLogger>>` writing
     `%LOCALAPPDATA%\zhu-ye-ime\logs\ime.log` (1 MiB rotation).
  `LOCALAPPDATA` missing → skip file writing, keep only `OutputDebugStringW`.
  The 53 call sites were graded (8 error / 7 warn / 14 info / 24 debug; see
  the 诊断产品化设计 §4.1 table); error/warn/info sites call
  `product_log(zhu_ye_core::LogLevel::X, …)` directly, debug sites keep
  `debug_log`. `zhu-ye: ` message prefix and `OutputDebugStringW` are
  preserved (VM forensics rely on them). Configuration level is cached once
  per process (`OnceLock`; P-12 does not hot-swap settings).
- **Hot-path short-circuit:** the per-keystroke debug sites
  (TestKeyDown / key / v-consume / replace-last / compose-text / commit-text /
  cand-show / cand-hide / comp-update / commit) are wrapped in
  `if should_log(LogLevel::Debug)` so no `format!` string is constructed on
  the keystroke path when the config level is `Warn`.
- **settings (window/shell/model/config):** `acceptance_log_dir` became
  `product_log_dir() -> Option<PathBuf>` =
  `%LOCALAPPDATA%\zhu-ye-ime\logs` (same mechanism as `data_dir`);
  diagnostics and the "open log dir" action use product wording, and the open
  action creates the directory idempotently first (the settings process may
  open it before TSF ever wrote a line).

`config.json` gains `log_level` (`#[serde(default, deserialize_with =
deserialize_log_level)]`); `CONFIG_FORMAT_VERSION` stays 1 (T-073 lenient
pattern: unknown field never breaks an existing config).

## Alternatives considered

**Write everything to user-data `%APPDATA%` together with `config.json`.**
Rejected: logs are disposable diagnostics, not user data; separation also
keeps TSF hot-path writes off the config path.

**A macro/logger crate for leveled logging.** Rejected: 53 call sites with
explicit level arguments cost nothing to convert, avoid a new dependency and
format-macro churn; the two hot-path guards cover the real performance
concern.

**Pre-construct all messages regardless of level.** Rejected for the 11
hot-path sites (guard is one line); accepted for the remaining debug sites
(a `format!` per event even when filtered) — recorded here as an explicit
trade-off: file I/O and directory syscalls are the dominant cost and stay
zero, `format!` alone is sub-µs.

**FileLogger with an internal lock.** Rejected: core has no threading model;
the single TSF consumer wraps it in a static `Mutex` (cross-thread callbacks
possible).

## Consequences

- Default installs (no `log_level` key): zero file writes and zero directory
  syscalls unless a `Warn`/`Error` fires; every `Error`/`Warn` lands in
  `%LOCALAPPDATA%\zhu-ye-ime\logs\ime.log`, rotated at 1 MiB.
- Acceptance regressions are impossible by construction while the sentinel
  file exists: that branch reuses the old formatting and path unchanged.
- `debug` messages keep flowing to `OutputDebugStringW` at every level (no
  behavioral change for debugger-assisted troubleshooting), and to file only
  when `log_level = "debug"`.
- The settings window locates the same directory TSF writes to, creating it
  on demand — no "not found" system dialog.
- Privacy: only engine state and diagnostics are logged, never candidate or
  committed input text.

Verification (2026-10-04): core 6 (log_level) + 6 (file_log) + 3
(pack_config lenient `log_level`) new tests; ime `log_tests` 2
(`product_log_path` with LOCALAPPDATA injection incl. missing-variable, and
sentinel existence); settings 1 (`product_log_dir`). Workspace suites all
green (core lib 289, ime 177, settings 102, unchanged crates green);
fmt/clippy `-D warnings`/`git diff --check` clean; acceptance §16.2 FR-060
rows backfilled; host-e2e rerun as regression evidence.

Related notes: [prefix word expansion (T-090,
2026-10-04-prefix-word-expansion.md)](../../implemented/feature/2026-10-04-prefix-word-expansion.md)
— acceptance-era note; its expansion group changed one host-e2e assertion
semantics: the multi-pack "no order drift" check was tightened from a prefix
match to a subsequence check (a slang-pack word such as 你好安怡 may legally
enter the expansion group above base prefix words under D-71 frequency
ordering). [domain boost](2026-10-02-domain-boost-scenario8.md)](../../implemented/feature/2026-10-02-domain-boost-scenario8.md),
[contacts (2026-10-02-contacts-scenario9.md)](../../implemented/feature/2026-10-02-contacts-scenario9.md),
[en wordbook (2026-10-04-en-wordbook-zyen-v1.md)](../../implemented/feature/2026-10-04-en-wordbook-zyen-v1.md)
— consumers whose `debug_log` diagnostics are now graded by the §4.1 table.
