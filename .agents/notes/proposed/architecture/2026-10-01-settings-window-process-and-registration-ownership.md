# Agent Note: Settings window process and registry-write ownership (phase 8)

Status: proposed

[中文](2026-10-01-settings-window-process-and-registration-ownership.zh.md) | English

## Problem

The repository has no settings UI at all. `zhu-ye-ime` is a TSF `cdylib` whose only
window is the candidate window, and `Cargo.lock` contains no GUI toolkit — no egui,
winit, tao, wry, WebView2, Slint, or Direct2D binding. There is no settings window, no
tray menu, and no property page anywhere in the source tree.

Three accepted requirements therefore have no way to be satisfied:

- **P-11** deferred the settings GUI (`本期不做设置 GUI；领域包启停走配置文件，GUI 设置窗下期交付`),
  so the configuration file has stood in for a UI since M6-R.
- **FR-015** and **FR-022** require the user to enable and disable domain packs **in a
  settings UI**, and FR-022 fixes the fields that UI must present: name, description,
  entry count, size, version, and enable switch, with the enable state persisted across
  restarts.
- **P-02** requires a manual "check for updates" entry **in the settings UI**. The
  updater binary exists and is verified, but its only caller is `zhu-ye-cli dict update`.

Two blockers stand behind those gaps:

- A one-click **repair** cannot be built while registration ownership is fixed to the
  installer scripts. [The TSF registration note](../../implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)
  states that registration is owned by `scripts/install.ps1` and `scripts/uninstall.ps1`,
  and `docs/architecture.md` repeats the invariant (`注册表只由 scripts/install.ps1 /
  scripts/uninstall.ps1 管理`). No shipped code writes the registry, and the only
  implemented registration check is the PowerShell function `Test-TsfRegistration`.
- The packaging path cannot deliver a UI. `install.ps1` takes the DLL from the source
  tree, copies exactly one base dictionary, and creates no `packs` directory, no manifest,
  no domain pack, and no shortcut. `package-portable.ps1` assembles the same shape.

The requested scope is explicit that entries without a shipped backend must still appear,
marked as planned rather than omitted.

## Proposal

### Process shape

Add a `zhu-ye-settings` crate that builds a standalone `zhu-ye-settings.exe`: a native
Win32 top-level window with GDI painting, reusing the pure view-model and theme logic
`candidate_ui` already exposes, and introducing **no** GUI dependency. The input-method
DLL gains no window and no UI code, so nothing new runs inside a host process.

The first batch (T-073) reaches that shared logic by adding `rlib` to `zhu-ye-ime`'s
`[lib]` and depending on it, rather than by extracting a shared primitives crate in the
same change: extraction would touch the shipped candidate window and its tests, putting a
refactor and a new window in one task. The cost is that `zhu-ye-settings.exe` links the
TSF-side code; extracting `zhu-ye-ui` (colors, theme kind, rects, DPI, text measurement,
re-exported from `candidate_ui` so existing paths stay valid) removes the `rlib` and is
tracked as T-081.

### Window shape and settings taxonomy

One window with left-hand navigation over three pages: 工具箱 (Toolbox), 常用设置
(Common settings), 关于与更新 (About and updates). Entries whose backend is unbuilt stay
clickable and expand an inline `正在规划中` notice; they are neither greyed out nor
answered with a modal dialog.

Settings divide into three kinds, and the window will render the difference instead of
flattening everything into a switch:

- **Runtime toggle** — takes effect immediately and is persisted.
- **Assembly item** — written to configuration and applied when the input method
  reassembles on its next start.
- **Session state** — takes effect immediately and is deliberately not persisted; the
  Chinese/English mode is the first member of this kind.

Runtime toggles are written to `config.json` as they change; assembly items are committed
by an explicit save. The window is single-instance — a second launch activates the
existing window — and it exits when closed, so the "zero background processes" acceptance
metric stays true.

### Entry points

Four entrances, so that each habitual route reaches the window:

- **`ITfFnConfigure`** on the text-service object. Windows reaches it through
  `CoCreateInstance` with the CLSID already registered via
  `ITfInputProcessorProfiles::Register`, so the 属性 button in Windows Settings → Language
  options opens the window with **no new registry keys**.
- **A right-click menu item** on the existing language bar button. `InitMenu` and
  `OnMenuSelect` return `E_NOTIMPL` today; a menu entry that only spawns a process never
  touches engine state, so it sits outside the non-`Send` boundary recorded in
  [the language bar note](../../implemented/feature/2026-09-28-language-bar-mode-icon.md).
- **A Start-menu shortcut** created by the installer.
- **`zhu-ye-cli settings`**, forwarding to the executable the way `zhu-ye-cli dict update`
  already forwards to the updater.

Clicking the language bar button's left mouse button will **not** open the window; that
gesture stays reserved for the mode toggle the language bar note records as future work.

### Theme selection

The window offers 浅色 and 深色 only. High contrast stays under system control and is not
selectable. "Follow the system" is deliberately absent for the candidate window: the
fixed-light default is an accepted decision
([the light-default note](../../implemented/feature/2026-09-25-candidate-window-light-default-and-empty-panel.md)),
which already anticipates a future user-visible setting as the only route to the dark
palette. There are no theme files, no user-defined palettes, and no online distribution —
the `不做皮肤商城` non-goal stands.

The settings window itself **does** follow the system light/dark setting. That does not
disturb the candidate-window decision, which binds the candidate window only.

### Pack management and the installed-pack record

The 添加词库 page fulfils FR-022 and adds local `.zyct` import. Import validates the magic
`ZYDT`, the format version, the header layout, and the content SHA-256 through the
existing loader path, and **does not verify a release signature** — a user-supplied pack
carries none. The UI must say so, because imported packs sit outside the trust chain
recorded in [the update trust-chain note](../../implemented/architecture/2026-09-29-dictionary-update-trust-chain.md).

FR-022's fields have no single current source, so they will be satisfied as follows:

| Field | Source |
| --- | --- |
| Name, description | A static `id → (name, description)` table in `zhu-ye-core` |
| Entry count, size | Read from the `.zyct` product — header `entry_count`, file metadata |
| Version | A new on-disk record, `packs/installed.json` |
| Enable switch | `config.json` `enabled_packs` |

`packs/installed.json` is written by the online updater and by local `.zyct` import, and
records `id`, `version`, `sha256`, `source`, and time. The split is deliberate:
`config.json` holds **user intent** — which packs to enable, whether networking is allowed
— while `installed.json` holds **machine fact** — what is actually on disk, at which
version. Hand-editing configuration therefore cannot silently falsify installed state.

User word-list import and export are out of scope for this work.

### Repair: two privilege tiers

修复输入法 reports first and acts only when the user presses the repair button. It never
deletes: a damaged file is renamed to a `.bak` sibling and rebuilt, matching the recovery
convention `UserDictStore` already uses.

- **Tier one, no elevation.** Detection takes the three criteria `Test-TsfRegistration`
  encodes as its base (the language-profile key exists with `Enable = 1`; `InProcServer32`
  exists, its default value names the DLL path, and `ThreadingModel` is `Apartment`) and
  **adds one strengthening: the registered DLL path must point to a file that actually
  exists** — the installer script only compares against a caller-supplied path, which is
  exactly what lets uninstall assert absence with a dummy path. Detection also validates
  every `.zyct` header and content hash and confirms that `config.json` and
  `user_words.json` parse. Repair recreates a missing `packs` directory and rebuilds
  damaged configuration.
- **Tier two, elevated.** Re-register both HKLM trees by removing and recreating them,
  idempotently. The DLL path comes from the current `InProcServer32` value and falls back
  to the newest `zhu-ye-ime*.dll` in the install directory, because deployments use a
  versioned file name. When the DLL itself is gone the window reports
  `输入法文件缺失，请重新安装` rather than pretending to repair.

Elevation is requested when the repair button is pressed, not when the window opens.
After a successful registration repair the window offers to restart ctfmon — the step that
makes a registration change take effect — behind a confirmation, because restarting
ctfmon interrupts input in every running application.

管理输入法 shows the same registration verdict as read-only status, plus a button that
opens the Windows input-method settings. That keeps "manage this input method" and
"manage input methods in general" distinguishable, and it reuses the detection code the
repair path needs anyway.

### Update check

The page displays state and offers an explicit button. The window **spawns
`zhu-ye-updater`** to do the network work and never opens a connection itself, preserving
the single-networked-component invariant. `online_update` stays default-off and must be
ticked explicitly; applying an update asks for a second confirmation.

P-02's other half — an asynchronous check at input-method startup — is **not** part of
this work. It would introduce process creation into the input method's startup path, where
cold-start latency, host-process privileges, and anti-malware behavior all apply, and the
acceptance VM is unreachable, so it cannot be verified. It becomes its own task.

### Packaging

`install.ps1` will take its files from the distribution itself instead of the source tree,
install `zhu-ye-settings.exe` and `zhu-ye-updater.exe`, pre-place the three domain packs,
and create the Start-menu shortcut; `package-portable.ps1` follows. The three domain packs
total roughly 2.25 MB against a 23 MB base, so pre-placing them is cheap and makes FR-022
auditable offline instead of costing a network round trip per acceptance run. Both scripts
must keep the UTF-8-with-BOM property recorded in
[the portable-scripts note](../../implemented/bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md).

### Toolbox contents

The toolbox page carries an emoji panel over the existing 381-entry alias table and a
symbol panel over the existing three groups of nine symbols, structured by category so a
larger symbol set can fill it later. Symbol-table expansion is its own task. 图片表情
stays a placeholder: TSF inserts text only, so an image cannot be committed into an
arbitrary host; a clipboard-copy action is the most a future version could offer.

## Alternatives considered

**Build the window inside the TSF DLL.** Rejected: a TSF DLL is loaded into arbitrary host
processes, where creating a settings window is indistinguishable from suspicious host
behavior and invites anti-malware interference. The DLL is also called by ctfmon on
arbitrary RPC threads, where the engine state's `Rc<Mutex<…>>` is not `Send` — the same
boundary that [the language bar note](../../implemented/feature/2026-09-28-language-bar-mode-icon.md)
already documents for `OnClick`.

**Render with WebView2.** Rejected: it depends on the Edge runtime rather than a Windows
guarantee, adds a browser process to a deliberately lightweight input method, and buys
nothing that GDI cannot draw for this window.

**Adopt a third-party Rust GUI (egui/eframe and friends).** Rejected: the project's first
principles are self-controlled code and no bundled runtime behavior. A window with a
navigation rail, a few checkboxes, and a list does not justify a new GUI stack, and the
candidate window already proves the in-house GDI approach.

**Open the window from the language bar button's left click.** Rejected: it spends the most
prominent gesture on the least valuable action and closes off the mode-toggle-on-click
direction the language bar note records as future work.

**Keep `install.ps1` as the repair backend.** Rejected: it reads the DLL from the source
tree and can invoke `cargo`, so it cannot run on a user machine that has no checkout. It
also installs one base dictionary and never restarts ctfmon, so it is a developer
installer, not a repair path.

**Add `DllRegisterServer` and call `regsvr32`.** Rejected: it would be a third
registration implementation alongside the scripts and the settings window, making "which
one is authoritative" a new source of drift, and it re-opens the registration path the
[TSF registration note](../../implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)
closed on purpose.

**Require administrator rights for the whole settings window.** Rejected: routine changes
such as picking a theme or enabling a pack would raise a UAC prompt, and elevation would
be granted far more often than the one operation that needs it.

**Let the settings window perform the network request itself.** Rejected: it would break
the single-networked-component invariant that makes "zero outbound connections while
online features are off" auditable in one binary.

**Offer "follow the system" as a candidate-window theme.** Rejected: it reverses the
fixed-light default decided for the candidate window. Users who want the dark palette
select 深色 explicitly.

**Persist the Chinese/English mode as a setting.** Rejected: the mode is session state.
Making it persist produces "I left it in Chinese and it came back English", and it would
also mean a restart changes what the next keystroke does.

**Resolve FR-022's name, description, and version from a new manifest shipped alongside
the packs.** Rejected: the published manifest is a remote release artifact and is never
written to the local packs directory, so reading packs from it would require inventing a
second local manifest — the very duplication `installed.json` avoids by carrying only
update-relevant facts.

**Extend the `.zyct` header to format v3 to carry name, description, and version.**
Rejected: only 20 bytes of the 128-byte header are unwritten (offsets 76..80 and
112..128), which cannot hold a Chinese name and description, so this is a format change
rather than a field addition — a large cost for metadata that belongs to the distribution,
not to the dictionary content.

**Store the installed-pack record in `config.json`.** Rejected: it mixes machine fact into
the file users are expected to hand-edit, which is how "I edited my config and my state
disappeared" happens; the configuration loader already treats damage as a reason to fall
back to defaults.

**Add user word-list import and export.** Rejected for this work: it needs a new text
format contract (field separation, pinyin annotation, polyphone handling) designed before
it is coded, and that contract is a project of its own.

**Grey out placeholder entries, or answer them with a modal dialog.** Rejected: greying out
hides what the entry will eventually do, and a modal interrupts the user once per
placeholder. An inline notice keeps the entry discoverable and still explains it.

**Expand the symbol table as part of this work.** Rejected: a symbol corpus is data work
with its own sourcing, licensing, and review trail. Separating it lets the panel ship and
be accepted without waiting for the corpus.

**Implement P-02's startup check here.** Rejected: it puts process creation into the input
method's startup path, and the acceptance VM is unreachable, so the behavior that matters
most — a real host loading the DLL — could not be observed.

## Acceptance criteria

- The window opens from all four entrances on a machine with the input method registered:
  the 属性 button in Windows Settings, the language bar right-click menu, the Start-menu
  shortcut, and `zhu-ye-cli settings`.
- Each page lists exactly the agreed entries. The placeholder entries (图片表情, 简繁切换,
  全半角切换, 生僻字输入) show the inline `正在规划中` notice and trigger no other action.
- Selecting 深色 changes the candidate window palette after the input method restarts, and
  the choice survives a restart; 高对比度 remains under system control.
- Enabling a domain pack in the window, saving, and restarting changes which packs load,
  and the enable state survives a restart.
- Every pack row shows name, description, entry count, size, and version; the name and
  description come from the built-in table, the counts from the product, and the version
  from `packs/installed.json`.
- Importing a valid local `.zyct` makes it appear with its entry count, size, and an
  explicit statement that it is unsigned and outside the update trust chain; importing a
  file with a wrong magic, an unsupported format version, or a broken content hash is
  rejected with the reason shown.
- Configuration written through the window is atomic; a start with a deliberately
  corrupted `config.json` falls back to defaults and reports the problem.
- Repair tier one, on a deliberately corrupted user word store, declares the damage and
  rebuilds it after an explicit press, leaving a `.bak` sibling and no lost entries.
- Repair tier two, run elevated with a registration key deleted by hand, restores both
  HKLM trees, and the offered ctfmon restart restores input in a running application.
  With the DLL removed, the window reports that a reinstall is required instead of
  claiming a successful repair.
- With `online_update` off, pressing "check for updates" produces zero outbound
  connections; with it on, the window reports the updater's result and asks for a second
  confirmation before applying.
- `install.ps1` on a clean machine installs both executables, the three domain packs, and
  the Start-menu shortcut; `uninstall.ps1` leaves no registry residue and no packs
  directory.
- `Cargo.lock` gains no GUI dependency, and `zhu-ye-ime.dll` exports nothing beyond what
  it exports today.
- Host-side evidence exists: unit tests for the three-way settings behavior, registration
  verdict detection, repair decisions, and pack metadata assembly, plus BMP screenshots of
  all three pages.
- Interactive acceptance on the VM — entrances, real registry writes, ctfmon reload — is
  written as criteria and **backfilled once the VM is reachable**. Until then this phase
  reports implementation complete with acceptance outstanding, and no task in it is marked
  complete.

## Risks

**Registration responsibility spreads across two implementations.** The identity constants
now live in `crates/zhu-ye-ime/src/tsf.rs` and `scripts/ime-identity.ps1`, kept in sync by
comment alone. A second writer in Rust doubles the chance of drift. Mitigation: a
cross-consistency test that compares the Rust constants against the values
`ime-identity.ps1` exports, run as part of the pre-push gate; without it this proposal
should not land.

**A second writer makes partial repair possible.** Tier two writes two HKLM trees; a
failure between them leaves registration half-applied. Mitigation: the repair must remain
idempotent and re-checkable, and the post-repair verdict comes from the same detection
path the read-only status uses, so a partial result is reported as failure rather than
success.

**`installed.json` is a new durable format with two writers.** The updater and the local
import path must agree on its schema, and the file needs a version field plus a rule for
what happens when the version is too high. Deleting or corrupting it must degrade to "no
version shown", never to a failed start.

**Concurrent writes to the application data directory.** The settings window, the updater,
and the input method's read path all touch `%APPDATA%\ai-zhu-ye-ime`. Two processes writing
`config.json` can clobber each other — most concretely, a window save can discard the
updater's `last_check`. Atomic writes alone do not close this window; whatever re-read or
single-writer rule the design adopts must be honoured by both writers.

**A repair button that writes HKLM is a privilege boundary in a UI.** A defect here can
damage another text service's registration. Mitigation: the write set is bounded to the
two trees the installer already owns, the operation is remove-then-recreate like the
installer's, and it runs only on an explicit press.

**The packaging change invalidates existing install acceptance.** Install and uninstall
were accepted through a VM drill that is currently unavailable, so a regression in the
installer could stay unnoticed until the VM returns.

**VM unavailability leaves acceptance debt.** This phase cannot be closed while the VM is
unreachable, exactly as T-061, T-062, and T-067 stand today. The debt must be recorded, not
papered over.

**`ITfFnConfigure::Show` runs on a thread we do not choose.** It must follow the same rule
as `OnClick`: do no work beyond spawning the settings process, and never touch engine
state.

**Placeholders are user-visible promises.** Four entries ship as `正在规划中`; each one
represents a real project (image stickers, simplified/traditional conversion, full/half
width conversion, rare-character input). Leaving them indefinitely erodes trust in the
window.

**The settings window follows the system theme while the candidate window does not.**
Deliberate, and still a possible source of "this looks like a bug" reports.

**Knowingly given up in this work:** image stickers committed into the host, symbol-table
expansion, user word-list import and export, theme files and user-defined palettes, the
startup update check, tray residency, and the click-to-toggle language bar gesture. The
window also means a greenfield GUI surface to maintain, with its own DPI, layout, and
accessibility questions.
