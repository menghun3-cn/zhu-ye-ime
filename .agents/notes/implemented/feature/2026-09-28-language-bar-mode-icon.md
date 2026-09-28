# Agent Note: Language bar icon shows 中/英 per IME mode (T-046)

Status: implemented

[中文](2026-09-28-language-bar-mode-icon.zh.md) | English

## Problem

The user wants the Windows language-bar icon (Language Bar Icon) to reflect
the IME's actual activation state: **"中" in Chinese mode, "英" in English
mode**, switching live with Shift. Windows has no registry-only way to make a
TSF text service's language-bar icon change dynamically — the icon shown for a
tip comes from its registry `IconFile` entry or from a language bar item, and
only a **language bar item** can be refreshed at runtime.

## Decision

Register a **custom TSF language bar item** per activated thread — the
canonical mechanism MSDN documents for dynamic language-bar UI:

- Item: `ITfLangBarItemButton` (with `ITfLangBarItem`) plus a hand-written
  `ITfSource`. `ITfTextInputProcessor::Activate` registers it through
  `ITfThreadMgr → ITfLangBarItemMgr::AddItem`; `Deactivate` removes it with
  `RemoveItem`. Each activated thread gets its own item, so the icon always
  follows that thread's actual engine mode.
- Notification: MSDN states that objects implementing `ITfLangBarItem` can
  expose `ITfSource`, through which the language bar manager advises
  `ITfLangBarItemSink`. When the engine mode changes (the `ToggleMode` arm of
  `sync_engine` in `tsf.rs`), the item calls its stored sink's
  `OnUpdate(TF_LBI_ICON)` **outside the engine lock**; the language bar then
  re-queries `GetIcon`/`GetInfo` and repaints. `windows` 0.61 does not bind
  `ITfSource`, so it is defined in `lang_bar.rs` with
  `define_interface!` + a hand-written Vtbl; the IID
  `4EA48A35-60AE-446F-8FD6-E6A8D82459F7` comes from `msctf.h`
  (`MIDL_INTERFACE`), not from guesswork.
- Icons are rendered at runtime with GDI, 16×16: a 32bpp top-down DIB
  (`CreateDIBSection`, `biHeight = -16`, BI_RGB) filled with the background
  color, then a white bold SimSun glyph (`CreateFontIndirectW` + `DrawTextW`,
  `DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX`); the mask is a
  1bpp **all-zero** bitmap of the **same dimensions** — `CreateIconIndirect`
  rejects mismatched mask/color sizes (E_INVALIDARG), and for 32bpp icons
  transparency comes from the color bitmap's alpha channel. Handles are kept
  for the process lifetime in a `OnceLock` and never deleted, because
  `CreateIconIndirect` references the bitmaps (deleting them is a use-after-
  free risk).
- Look: Chinese = brand blue `#1E88E5`, English = neutral gray `#757575`,
  white glyph; item style `TF_LBI_STYLE_BTN_BUTTON | TF_LBI_STYLE_SHOWNINTRAY`
  (tray/language-bar display on Win10; Win11 ignores third-party items in the
  default taskbar indicator — see Consequences).
- Registration failure is non-fatal (logged, no `Activate` abort): input keeps
  working if ctfmon/language bar is unavailable.
- **Click-to-toggle is intentionally deferred**: `OnClick` returns success but
  does nothing. The engine state is `Rc<Mutex<EngineState>>` (not `Send`), and
  ctfmon may enter item methods on arbitrary RPC threads; toggling would need
  `Arc<Mutex<… Send>>` engine state. Recorded as future work.

## Alternatives considered

**Registry `IconFile` / class icon only.** Rejected: the icon is resolved per
TIP, not per thread or per mode; there is no supported runtime refresh path,
and poking HKCU while ctfmon caches values is fragile.

**Item without `ITfSource` (no sink).** Rejected: without
`ITfLangBarItemSink` the item cannot tell the language bar to repaint, so the
bar would keep showing the initial icon — exactly the bug this task exists to
avoid.

**Click-to-toggle via `OnClick` calling the engine directly.** Deferred (see
Decision): the `Rc`-based engine state is not `Send`, and entering the engine
mutex from arbitrary ctfmon callback threads would break the current thread-
model safety assumptions.

## Consequences

- The icon tracks the real mode only where the language bar renders the item:
  the desktop language bar and the Win10 tray work; Win11's default taskbar
  input indicator does not render third-party language bar items — the
  installation guide tells users to enable 桌面语言栏 (use the desktop language
  bar).
- `lang_bar.rs` owns the item, icons, `ITfSource` glue and unit tests; `tsf.rs`
  only wires Activate/Deactivate and the `ToggleMode` notification.
- Verification: 10 new unit tests drive the item directly without ctfmon
  (icon/text follow mode, `OnUpdate(TF_LBI_ICON)` on mode switch, riid
  filtering, re-advise replacement, unadvise silence, `GetInfo` identity,
  icon handle validity via `GetIconInfo`, `OnceLock` singleton); full crate
  lib suite 79 tests and host-e2e (19 seed + 6 real-dictionary checks) green;
  `clippy -D warnings` and rustfmt clean. OS-level display verification
  (desktop language bar screenshot on the VM) remains a manual acceptance step.
- Cross-references: mode-switching key wiring —
  [2026-09-20-candidate-window-tsf-integration](2026-09-20-candidate-window-tsf-integration.md);
  Shift gating outside composition —
  [2026-09-25-page-keys-minus-plus](2026-09-25-page-keys-minus-plus.md);
  MSDN: [ITfLangBarItemButton](https://learn.microsoft.com/en-us/windows/win32/api/ctfutb/nn-ctfutb-itflangbaritembutton),
  [ITfLangBarItemSink::OnUpdate](https://learn.microsoft.com/en-us/windows/win32/api/ctfutb/nf-ctfutb-itflangbaritemsink-onupdate),
  [ITfSource](https://learn.microsoft.com/en-us/windows/win32/api/msctf/nn-msctf-itfsource).
