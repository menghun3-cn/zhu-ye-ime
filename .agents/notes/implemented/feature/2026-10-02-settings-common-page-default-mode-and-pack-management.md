# Agent Note: Settings window — default IME mode and dictionary pack management (T-075)

Status: implemented

[中文](2026-10-02-settings-common-page-default-mode-and-pack-management.zh.md) | English

## Problem

The common-settings page (T-075) ships two remaining items. First, the
Chinese/English mode entry was originally planned as a read-only "current mode"
display plus immediate toggling — a session-state view — but the settings window is
a separate process and has no way to read the host IME's in-session mode, so "the
current mode" is meaningless across processes (D-32). Second, "add dictionary
packs" (FR-022/FR-042) needs a list of installed domain packs with enable/disable
toggles and a local-`.zyct` import flow, with the security semantics of imported
files decided by D-38 (no signature verification, no place in the online-update
trust chain).

## Decision

The **Chinese/English default mode** is an assembly item, not session state. The
settings window writes `config.json`'s `default_mode` (`"chinese"` | `"english"`,
parsed leniently by `deserialize_default_mode`, defaulting to Chinese; the choice
does not bump the config format version). The TSF DLL reads it on its next
assembly (`Activate`) and applies it as the starting mode of new sessions
(`zhu-ye-ime/src/tsf.rs` `configured_default_mode`). The page entry shows two
choice chips (中文 / 英文) with the stored default selected, and tapping a chip
saves immediately and hints "重启输入法后新会话生效" — the same save-on-click
pattern as the theme entry (S-8 re-read before write). The in-session Shift
toggle remains session state, is not persisted, and is intentionally not exposed
anywhere in the window; the decision record is in the
[settings-window process/registration note](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md).

The **add-dictionary sub-view** replaces the page content area (no inline row
expansion; the 12-page list already exceeds the 482 px content column). A
top note states "导入包以本地文件为准，不参与在线更新的签名信任链（不验签）"
(D-38). Each row shows name, one-line summary, "N 词条 · M KB" and version
(imported packs render "本地导入 · 未签名"), plus an enable toggle. Toggling
writes `enabled_packs` and saves `config.json` (assembly item, takes effect on
next IME assembly); the base pack renders "基础包" and is not toggleable.
Rows come from `inventory::list_packs`: known pack metadata first, then any
`packs/*.zyct` found on disk, in UTF-8 byte order; unknown packs use their file
stem as the display name. The toolbar pins a "← 返回常用设置" button and the
primary "导入本地 .zyct…" button to the bottom of the content area; rows beyond
the visible height are clipped, never compressed.

The **import flow** (`window::import_zyct`) is: `GetOpenFileNameW` system dialog
(`OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR`,
filter `"词典包 (*.zyct)\0*.zyct\0全部文件 (*.*)\0*.*\0"`) → full-format
validation via `zhu_ye_core::DictionaryFile::open` (magic, version, header,
content SHA-256, section layout); any failure rejects the file without touching
disk ("导入失败：…（文件未改动）") → copy into `packs/{stem}.zyct`
(overwrite on same id) → `sha256_file` → `installed.json` upsert with
`source: Import`, no version → refresh the list ("已导入「…」（本地导入，不参与
签名校验），勾选启用后重启生效"). A list-write failure after a successful copy
is surfaced honestly ("导入完成但清单记录失败…") instead of pretending the
import rolled back.

The window keeps the state split of the rest of the app: `SettingsState`
(`packs_view` flag, `default_mode`) carries the transient UI state, while
`WindowState.packs` holds the re-scan of disk; `list_packs_now` re-reads
`config.json` and `installed.json` before every render trigger (enter,
toggle, import) following S-8. `--packs` (with `--shot`) captures the sub-view
for acceptance screenshots.

## Alternatives considered

- **In-line expansion of an "add dictionary" row**: rejected — the common page
  already has 12 entries, more than the ~482 px content column can show without
  scrolling; a full-content-area sub-view reuses the existing page machinery
  (navigation and the back button both exit it).
- **Read-only current-mode display + immediate Shift-style toggle** (the original
  session-state plan): rejected under D-32 — a cross-process settings window
  cannot read the host IME's current mode, so the only meaningful semantics is the
  default mode for new sessions; the runtime toggle stays in the IME and stays
  unpersisted.
- **Explicit "save" button for mode and pack toggles**: rejected — matches
  neither the theme entry (save-on-click) nor the S-8 re-read discipline; an
  explicit-save boundary would be a third interaction pattern for no user value.
- **Scanning upstream-dict directories or the exe directory for packs**: rejected —
  only `packs/` is scanned; the base pack `dictionary.zyct` lives beside the exe
  (`shell::exe_dir`) and never collides with the import namespace `packs/`.
- **Verifying imports against the online signature chain**: rejected (D-38) — the
  UI states explicitly that imported files are not signature-checked; the
  installed record carries `source: Import` so the trust chain remains auditable.

## Consequences

- `default_mode` is persisted in `config.json` without a format-version bump;
  unknown values fall back to Chinese without damaging other fields.
- Imported packs are marked "本地导入" in the list and "未签名" in the meta line;
  they participate in enable/disable but never in the online update signature
  chain ([trust chain note](../../implemented/architecture/2026-09-29-dictionary-update-trust-chain.md)).
- `installed.json` records authored by the importer are compatible with the
  update-writer records; the record is treated as non-authoritative, and
  `packs/` re-scans catch missing or corrupted manifests
  ([deployment note](../../implemented/architecture/2026-09-21-installed-dictionary-deployment.md)).
- The import dialog and the packs sub-view are host-interactive; automated
  acceptance in the VM (T-080) covers the dialog, while in-host screenshots cover
  layout for the mode page and the packs sub-view.
