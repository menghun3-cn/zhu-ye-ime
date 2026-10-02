# Agent Note: zhu-ye-ui crate extraction and rlib removal

Status: implemented

[中文](2026-10-02-zhu-ye-ui-crate-and-rlib-removal.zh.md) | English

## Problem

The settings window reuses the candidate window's pure-logic theme and layout
primitives. The mechanism chosen in T-073 was to add `crate-type = ["cdylib",
"rlib"]` to `zhu-ye-ime` and let `zhu-ye-settings` depend on the whole IME crate.
That couples the settings binary to the TSF side (`input`/`tsf`/dictionary
loading) at build and link time — S-10 recorded the extraction as a deferred
task (T-081).

Two concerns make the extraction non-trivial:

- The primitives themselves (`UiColor`, `UiThemeKind`, `SystemColors`,
  `UiRect`, `BASE_DPI`, `estimate_text_width`, `fit_text`) live inside
  `candidate_ui`, which also carries candidate-window view logic; they must move
  without touching any existing call path or test.
- The TSF identity constants (CLSID, profile GUID, keyboard TFCAT, language ID
  hex, dictionary file name, install-dir segment) are imported by
  `zhu-ye-settings` from `zhu_ye_ime::tsf`. Removing the `rlib` dependency
  orphans them in the IME crate, so their Rust-side single source of truth must
  move somewhere both consumers can reach.

## Decision

### 1. New crate `crates/zhu-ye-ui` (pure std, zero dependencies)

Owns the seven UI primitives plus their tests. `candidate_ui` re-exports them
with `pub use`, keeping every existing call path (`candidate_window`, settings
imports, and the 165 IME tests) untouched. `fit_text` is deliberately NOT
re-exported: it has no production consumer (tests only), and re-exporting it
produced an `unused_imports` warning under the `#[path]`-compiled demo/e2e
bins; the API now lives only at `zhu_ye_ui::fit_text`.

### 2. `zhu_ye_core::identity` becomes the Rust-side single source of truth

The identity constants move to a new pure module in `zhu-ye-core` (the only
crate both consumers already share). GUIDs are written as `u128` literals —
optically the big-endian text form, exactly what `windows::core::GUID::from_u128`
consumes — so `zhu-ye-core` keeps its zero-Windows-dependency contract. The
module also provides `guid_text(u128)` and tests that pin the values against
`scripts/ime-identity.ps1`.

Consumers convert as needed:

- `zhu_ye_ime::tsf` rebuilds its `windows::core::GUID` constants via
  `windows::core::GUID::from_u128(identity::...)` (a const fn).
- `zhu-ye-settings::registry` takes `u128` and converts in its local
  `guid_text`; its GUID-based tests import from `zhu_ye_core::identity`.
- String constants are re-exported (`pub use`) by `zhu_ye_ime::tsf`.

`scripts/verify-tsf-identity.ps1` now reads `crates/zhu-ye-core/src/identity.rs`
(the `Get-RustGuidConst` regex was adapted to the `: u128 = 0x...` form).

### 3. Settings high-contrast mapping is localized

`settings_theme_from_system_colors` previously delegated the BGR→RGB swap and
the high-contrast mapping to `candidate_ui::theme_from_system_colors`. With the
decoupling that call is re-implemented locally with the identical mapping, so
the rendered result is unchanged while the settings crate no longer reaches
into the IME crate.

### 4. `zhu-ye-ime` returns to `cdylib`-only

`crate-type` drops `rlib`; `zhu-ye-settings/Cargo.toml` drops the
`zhu-ye-ime` dependency and takes `zhu-ye-ui` instead. `cargo tree` confirms
`zhu-ye-settings` depends only on `zhu-ye-core` + `zhu-ye-ui`. The
`candidate-demo` and `host-e2e` bins are unaffected: they compile the candidate
modules via `#[path]` and never reference the package crate by name.

## Alternatives considered

**Keep the identity constants in `tsf.rs` and copy them into settings.**
Rejected: the copy would create a third drifting source of truth, and
`verify-tsf-identity.ps1` would have to maintain two Rust source paths.

**Make `zhu-ye-core` depend on `windows-core` and store real GUIDs.**
Rejected: although `windows::core::GUID` is pure data, the core crate's
contract is "no Windows dependency" and staying free of it keeps core
buildable/testable everywhere; a `u128` literal carries zero representation
cost and conversion happens once at each consumer.

**Localize only the constants and keep using
`candidate_ui::theme_from_system_colors` through a `pub use`.**
Rejected: that forces settings to keep depending on the IME crate, defeating
the whole point of the extraction; the localized mapping is six lines and is
pinned by the existing four theme tests.

**Also re-export `fit_text` from `candidate_ui`.**
Rejected: it has no production consumer, and re-exporting it trips
`unused_imports` in the `#[path]`-compiled bins; its home is now `zhu_ye_ui`.

**Create a separate identity crate.**
Rejected: six constants do not justify another crate; `zhu-ye-core` already
hosts the other shared pure modules.

## Consequences

- Dependency graph: `zhu-ye-settings` → `zhu-ye-core` + `zhu-ye-ui`;
  `zhu-ye-ime` → `zhu-ye-core` + `zhu-ye-ui`. No `rlib` for the IME crate.
- Editing `zhu-ye-ime` no longer rebuilds the settings binary; TSF symbols are
  out of the settings link surface (exe size is nearly unchanged because LTO
  already stripped them — the win is the dependency graph, not bytes).
- Future UI primitives evolve in `zhu-ye-ui`; future identity constants in
  `zhu_ye_core::identity`, with `verify-tsf-identity.ps1` as the cross-check
  against `ime-identity.ps1` (D-42).
- Evidence: workspace tests green — core 222 (incl. 2 new identity), zhu-ye-ui
  7 (new), ime 165 (unchanged), settings 97 (unchanged), dict 45, cli 117,
  updater 20, host-e2e 4, demo 2; fmt, clippy `-D warnings`, all three verify
  scripts and `git diff --check` clean; `Cargo.lock` free of GUI frameworks;
  host screenshots (candidate dark, candidate light at 192 DPI, settings
  toolbox page, settings update page) render normally.

Tracked in [docs/todos-list.md](../../../../docs/todos-list.md) (T-081) with
its S-10 record in [docs/设置窗口设计.md](../../../../docs/设置窗口设计.md).
