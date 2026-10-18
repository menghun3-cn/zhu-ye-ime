# Agent Note: Chinese/English toggle syncs the system keyboard layout

Status: implemented

[中文](2026-10-18-mode-layout-sync.zh.md) | English

## Problem

The user observed that Sogou shows the system grey "A/" indicator when it
switches to English, while Zhu-Ye never does. Root cause: the grey "A/" is a
**system-level** "English layout active" indicator; Sogou's English mode
actually switches the system keyboard layout to US English (00000409),
whereas Zhu-Ye's English mode was only an engine-internal state — the system
layout stayed Chinese (00000804), so the indicator never appeared. The code
already assumed an English layout for English-mode key mapping (T-066:
Shift+2=`@`, `;`=`:`, `/`=`?`) but never switched the layout, leaving the
semantics unrealized.

## Decision

In `tsf.rs`, hook the mode toggle so the system keyboard layout follows the
Chinese/English mode (T-168), matching Sogou:

1. `system_layout_for(mode)` — pure function: English → `0x0409_0409`
   (US English), Chinese → `0x0804_0804` (Chinese-US keyboard).
2. `sync_system_layout(mode)` — calls `ActivateKeyboardLayout` with that
   handle; affects only the calling thread (the host UI thread on the
   Shift/Ctrl+Space toggle paths), returns silently on failure so an
   unavailable layout never breaks input.
3. Hooked at both toggle entry points: the `ToggleMode` branch of
   `sync_engine` (which all Shift-up / Ctrl+Space / settings-callback toggles
   already funnel through) and the language-bar click handler (Win10 path —
   if that callback thread is not the host input thread, the layout switch is
   inert, not harmful).

English mode therefore activates the real English layout: the taskbar shows
the system grey "A/" indicator (identical to Sogou) and the T-066 shifted
symbol mapping holds naturally.

## Alternatives considered

- **Keep engine-internal state only (status quo)**: no system indicator, the
  exact complaint; rejected.
- **Toggle the layout only on the keyboard paths, not the langbar click**:
  rejected for consistency — a single helper is called everywhere, and the
  inert case (ctfmon RPC thread) is harmless.
- **Persist/restore the host's original layout**: overkill; Sogou does not,
  and Chinese mode restoring 00000804 is the agreed convention.

## Consequences

- Chinese mode → system layout 00000804, no grey "A/"; English mode →
  00000409, grey "A/" appears — visible parity with Sogou on the taskbar.
- `ActivateKeyboardLayout` is thread-scoped; the reliable paths are the TSF
  host-UI-thread toggles (Shift up, Ctrl+Space). Silent failure keeps input
  immune to layout problems.
- The English-layout key mapping (T-066) becomes truly consistent with the
  active layout.

## Verification

- Unit test `模式对应系统键盘布局`: layout constants and the mode mapping
  (English → 0x04090409, Chinese → 0x08040804).
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace` (ime lib 249, tsf 44, core 352,
  host_e2e 170), `git diff --check`.
- Docs updated: FR-004 (需求规格说明书), acceptance rows (系统英文指示 /
  英文布局符号 under FR-004 in 验收标准), design §5.3, CHANGELOG
  [Unreleased], todos T-168.
- Deployed (2026-10-18, PR #211 → develop f74619e): DLL
  `zhu_ye_ime_A4277E6B.dll` (2,260,480 B) landed with pointers switched and
  ctfmon restarted (PID 18940); verify-tsf-dll/identity passed. Real-host
  acceptance passed (2026-10-18, user confirmed): grey "A/" appears on
  English mode and disappears on Chinese, `@`/`:`/`?` correct in English
  mode; the task is closed (todos T-168 done).
