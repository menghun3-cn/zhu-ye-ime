# Agent Note: Settings-window five upgrades (T-088)

Status: implemented

[中文](2026-10-04-settings-window-five-upgrades.zh.md) | English

## Problem

FR-048 (M13) delivers five settings-window enhancements: user-word list
import/export, symbol-panel expansion, theme files (custom themes), an async
update check at startup, and GUI .vcf import for contacts (D-53, user-confirmed
2026-10-02; plan §14.5). Constraints: the settings window and the IME process
stay zero-network (D-44 / S-4), the online-update default stays off (zero
spawn, zero outbound), `config.json` format version stays stable (S-7),
imports validate before writing (FR-042), and system high-contrast mode still
overrides hand-picked themes (D-31).

## Decision

Five subsystems, each self-contained with unit tests:

1. **User-word list exchange** (`zhu_ye_core::user_words_exchange`): v1 schema
   `{format:"zhu-ye-user-words", version:1, items:[{pinyin, word, freq}],
   exported_at}`. Export via GetSaveFileNameW with a default
   `user_words_YYYYMMDD.json`; import via GetOpenFileNameW → entire-file parse
   (any item error or `version > 1` rejects the whole file) → merge by
   `(pinyin, word)` taking the max freq → atomic write; failure never touches
   the on-disk file. Merging composes on `UserDictionary::merge_external`,
   which ignores empty word/pinyin and zero freq, keeps `last_used` for
   existing keys and seeds 0 for new ones. The on-disk `user_words.json`
   format (v1 `FileFormat`) is unchanged.
2. **Symbol-panel expansion** (D-33): core static tables go from the FR-028
   three groups × 9 (27 chars) to **230 chars** (9 extension groups
   `EXTRA_SYMBOL_GROUPS`, 203 chars; the v-mode quick symbols stay untouched).
   The settings panel enumerates `all_panel_groups`; the SendInput + UNICODE
   commit path is unchanged.
3. **Theme files / custom themes**: `themes\*.json` schema v1 with two
   sections — `candidate` (7 keys = the candidate-window palette) and
   `settings` (15 keys = the settings-window palette). Both windows resolve
   the same `config.theme` `Custom(name)` → same file (S-2). Name safety:
   `is_safe_theme_name` allows only `[A-Za-z0-9_-]` (no directory escape);
   missing file / invalid JSON / `version > 1` → rejected (return `None`);
   unknown keys ignored; missing or unparsable color keys → per-window
   fallback (settings window → current system light/dark preset
   `settings_theme(kind)`; candidate window → Light base). The settings theme
   subview lists `themes\*.json` sorted by display name; selecting one writes
   `config.theme`, applies immediately in-session (`SettingsTheme::with_theme_file`
   reports how many keys were covered), and a broken row shows a hint while
   keeping the current theme. System high-contrast (D-31) takes over and
   disables manual selection. No online theme distribution.
4. **Async update check at startup**: the updater gains a `check-once`
   subcommand — fully silent, no stdout. Gates: `online_update` enabled AND
   `last_check` older than 7 days (`check_due`). When due it fetches the
   manifest, finds outdated distributable packs, writes `update_status.json`
   atomically (tmp + rename), and rewrites `config.last_check` (S-8
   read-modify-write). IME side: `DllGetClassObject` spawns
   `zhu-ye-updater.exe check-once` at most once per host process (static
   `AtomicBool` switch), detached with `CREATE_NO_WINDOW` and
   `BELOW_NORMAL_PRIORITY_CLASS` (<10ms); `online_update` off → zero spawn.
   The settings About page shows the manual-check result first, else the
   `update_status.json` contents, else "尚未执行检查。".
5. **Contacts .vcf GUI import**: there is no separate contact word-list
   persistence — the import copies the chosen .vcf into the data dir
   `contacts\` (appending `(N)` to avoid name collisions) and appends the path
   to `config.contact_vcards` (deduped by normalized path, config re-read
   before saving, S-8). The subview lists current paths; the import dialog
   confirms the N parsed contacts first; parsing reuses the existing
   `parse_vcard`.

## Alternatives considered

- **Persistent own contact dictionary**: rejected — the FR-036 data layer
  already feeds the engine from `contact_vcards`; a second persistence would
  duplicate state and drift.
- **Settings window as the startup-check host**: rejected — the settings
  process is not resident (FR-039, exits on close); the check belongs to the
  updater process (D-44: window/engine never touch the network).
- **Spawn on every engine creation**: rejected — repeated spawns per
  app/process; one spawn per host process with the config gate.
- **Applying a theme by inlining all colors into `config.json` (key
  expansion)**: rejected — S-7 keeps the config format version stable;
  `config.theme` stores the theme *name* and the file holds the colors.
- **Online theme distribution**: rejected (existing §17.3 boundary).
- **Strict theme parsing (any unknown/missing key rejects)**: rejected —
  lenient parsing (unknown keys ignored, missing keys → presets) is what keeps
  old theme files usable when the palette grows; strictness is reserved for
  the version field.

## Consequences

- `config.json` gains an optional `contact_vcards` array (absent = empty list)
  and can store a custom theme name in `theme`; format version unchanged (S-7).
- `update_status.json` is a new data-dir file:
  `format`/`version`/`last_checked`/`available`/`outdated_packs`/`latest_version`/`error`.
- `CandidateWindow` grows an optional custom `ThemeFile`;
  `resolve_with_custom` keeps the light base and the high-contrast override.
- No new network paths; no new runtime dependencies; the settings window
  gains three subviews plus `--user-words`/`--contacts`/`--themes` shot modes
  for host forensics.

## Verification

Shipped (2026-10-04): core tests — user_words_exchange 5, theme_file 7,
update_status 6, time 5, symbols budget — settings 101, ime 170 (including the
`theme_with_candidate` overlay test); workspace 15 groups green;
fmt/clippy `-D warnings` clean. `check-once` host test: offline / not-due →
exit 0 with no network and no file writes; due → status file written plus
`last_check` rewritten. Host shots `data/artifacts/t088-shots`: the three
subview screenshots are pairwise distinct (pixel-diff ≥890 sample points) and
the About→update screenshot renders with a status file present.
VM-interactive items (real TSF assembly, packet capture for 0 outbound,
cold-start timing) are explicitly deferred until the VM is reachable.

Related notes (partial overlaps kept active and cross-linked): [candidate
window light default and empty panel (the light base and high-contrast order
stand; custom themes overlay colors only)](../../implemented/feature/2026-09-25-candidate-window-light-default-and-empty-panel.md),
[settings about and update page (now also reads update_status.json)](../../implemented/feature/2026-10-02-settings-about-and-update-page.md),
[user-dict persistence (user_words.json format unchanged; exchange format
added beside it)](../../implemented/feature/2026-09-19-user-dict-persistence.md),
[contacts scenario 9 (FR-036 data layer; GUI import added on config
registration)](../../implemented/feature/2026-10-02-contacts-scenario9.md),
[format/symbol/emoji candidates (FR-028 v-mode untouched)](../../implemented/feature/2026-09-30-format-symbol-emoji-candidates.md),
[dictionary update trust chain (check-once only reports outdated packs; the
chain itself is unchanged)](../../implemented/process/2026-09-29-dictionary-update-trust-chain.md).

## Window title "竹叶输入法" and bamboo LOGO icon (T-142)

The window title was `竹叶输入法 设置`; the window class had no icon
(`WNDCLASSW.hIcon` defaulted), so the title bar and taskbar showed no logo.
Also latent: `wnd_proc` handles `WM_NCCREATE` by stashing the state pointer and
returns `LRESULT(1)` **without forwarding to `DefWindowProcW`** (a
custom-draw staple), which means the window's internal title buffer was never
initialized from the create parameters — `GetWindowTextW` returned empty for
the settings window (cross-process reads confirmed empty; the title bar was in
fact blank before this change).

Decision (T-142, user: "设置GUI的标题改为竹叶输入法，并且加上LOGO"):
- `WINDOW_TITLE` → `竹叶输入法`; right after `CreateWindowExW` the window
  calls `SetWindowTextW(hwnd, WINDOW_TITLE)` — the explicit write works
  regardless of the `WM_NCCREATE` short-circuit, with zero change to its
  semantics (smallest diff; we deliberately do not start forwarding
  `WM_NCCREATE` to `DefWindowProc` for an unknown-risk habit change).
- Logo: new `crates/zhu-ye-settings/build.rs` embeds
  `zhu-ye-ime/assets/zhu.ico` via `winresource` 0.1 (build-dependencies,
  matching the TSF DLL mechanism from T-112b; cross-crate relative path so the
  one asset never drifts). `winresource::set_icon` writes `1 ICON …`, so the
  window class loads it with `LoadIconW(hInstance, MAKEINTRESOURCEW(1))` and
  assigns `WNDCLASSW.hIcon`. Make-int-resource takes `#[allow(clippy::manual_dangling_ptr)]`
  deliberately: clippy's suggested `ptr::dangling` yields value 2, which would
  look up icon id 2 instead of 1. Load failure falls back to a null icon and
  does not block startup (same as before).
- Verification channel: `--shot` mode now self-reports the window title and
  class icon (`GetWindowTextW` + `GetClassLongPtrW(GCLP_HICON)` in-process,
  printed with the screenshot path) — cross-process `GetWindowText`/
  `ExtractAssociatedIcon` are unreliable under desktop/session isolation.
  Self-proof passes: title `[竹叶输入法]`, class icon `0x…` non-zero; EXE
  resource extraction yields the 32×32 竹 icon.

Related notes (partial overlaps kept active and cross-linked):
[taskbar 竹 icon via TSF DLL resource injection (T-112b — same winresource
mechanism and the same zhu.ico asset)](../../implemented/feature/2026-10-06-taskbar-icon-zhu.md),
[settings about and update page (T-142 adds no new page content)](../../implemented/feature/2026-10-02-settings-about-and-update-page.md).
