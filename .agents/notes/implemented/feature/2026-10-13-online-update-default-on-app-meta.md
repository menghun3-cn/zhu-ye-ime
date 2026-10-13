# Agent Note: Online updates on by default; IME version + changelog via signed app.json (T-154)

Status: implemented

## Problem

The user gave three verbatim requests on 2026-10-13:

1. "在线更新，去掉文字说明中的（P-03），改为默认开启" — drop the (P-03) mention
   in the online-update text and make the feature on by default;
2. "自动检查更新，如果有新版本则显示最新的版本及更新日志" — the startup
   auto-check should show the newest IME version and its changelog when a newer
   version exists;
3. "联机更新的文案不要出现（D-44）之类的说明" — user-visible copy must not show
   internal decision IDs like (D-44).

Clarification answers (ask_user_question): the check target is the **IME app
version + changelog** (not dictionary packs); the display lives permanently on
the settings window's About & Updates page; older configs upgrade with a
**missing `online_update` key treated as enabled** (explicit `false` stays
off).

This reverses the P-03 "off by default" decision (shipped since the update
trust chain note, 2026-09-29) and adds a new signed metadata file to the
release pipeline.

## Decision

- **Default-on reversal**: `ConfigFile.online_update` gets
  `#[serde(default = "default_online_update")]` returning `true`, and the
  `Default` impl flips too — old configs missing the key load as enabled
  (upgrade path), explicit `false` is still honored, and the updater's
  "network request" gate (`online_update` false ⇒ zero spawn / zero network)
  is unchanged. UI copy for the switch no longer says "default off, enable
  explicitly".
- **IME app version lives in a separate signed `app.json`** — NOT a new field
  on `manifest.json`. The manifest signature is computed over the *canonical*
  JSON (`serde_json::to_vec`: field order = struct declaration order, signature
  field excluded). If we added a field to the manifest, an old binary would
  drop the unknown field while parsing, re-canonicalize differently than the
  publisher, and fail verification. A brand-new asset that old updaters simply
  never read is naturally backward compatible. `app.json` sits next to
  `manifest.json` on the release endpoint and reuses the trust model of P-04:
  ed25519 signature over canonical JSON, verified client-side with the
  **built-in** public key.
- **`AppMeta::verify` uses only the trusted public key passed in** (`&VerifyingKey`),
  never the public key embedded in the file (that key is audit-only). The
  initial implementation copied the manifest `verify_signature` pattern and
  verified against the file-embedded key — a security bug caught by the
  wrong-key unit test.
- **Version comparison is bidirectional**:
  `app_update_available(app, core) = version_at_least(app, core) && !version_at_least(core, app)`
  — reports "newer" only when the app version is strictly greater (leading
  digit of each segment, pre-release suffix ignored, same semantics as
  `min_engine_version`). A single-direction check would have inverted the
  meaning.
- **`UpdateStatus` stays `version: 1`** with two new optional fields
  (`app_version`, `release_notes`, `#[serde(default, skip_serializing_if)]`)
  and a new constructor `available_app(last_checked, app_version,
  release_notes, outdated_packs, latest_version)`; old v1 files read as
  `None`. check-once composes it when the app has a newer version (pack info
  carried along), so the About page reports both.
- **Degradation, not failure**: in `fetch_app_meta`, an HTTP fetch miss
  (404 / no asset / network) returns `Ok(None)` ("release source provides no
  app version info"); a parse/verify failure returns `Err` (visible one-line
  hint) — neither blocks dictionary-pack checks or applies, and check-once
  never writes a `failed` status because of app.json.
- **Result area grows 3 → 6 rows** (`LOGICAL_RESULT_ROWS`), and
  `update_status_lines` prints "发现输入法新版本 vX（当前 vY）" + "更新日志："
  + one line per changelog entry, with pack info appended when both exist.
- **No internal decision IDs in user-visible strings** (P-03 / D-44 / P-12 …);
  code comments keep them for traceability. This is the dsh-prose-standard
  rule applied to UI copy.
- **Release tooling**: `zhu-ye-dict` gains `build-app` / `sign-app` /
  `verify-app`; `assemble-release.ps1` builds `app.json` (app_version = root
  Cargo.toml version, release_notes = latest CHANGELOG section plain-texted —
  Markdown headings/links/backticks stripped by `plain_text_notes`),
  signs and includes it in the upload list and SHA256SUMS.txt;
  `verify-release-e2e.ps1` grows two steps: benign app.json display and
  tampered-app.json degradation.

## Alternatives considered

- **Add the version to `manifest.json`**: breaks old-binary verification
  (canonicalization drift) — rejected; a separate asset is the only
  backward-compatible path.
- **Reuse the manifest `version` field as the app version**: pack version is a
  dated release string independent of the engine — the IME version cannot be
  derived from it — rejected.
- **Keep default off and only change wording**: the user explicitly asked to
  flip the default — rejected.
- **Treat app.json failure like check failure (write `failed`)**: an optional
  display enhancement must not block pack updates or stale the status
  incorrectly — rejected; degrade instead.
- **Read the embedded public key for verification**: makes a tampered file able
  to vouch for itself — rejected; built-in key only, embedded key is audit
  data.

## Consequences

- Old updaters (already shipped) ignore `app.json` entirely — no signature
  break, no server-side compatibility shim.
- Every release now ships four artifacts: zip, packs, signed `manifest.json`,
  signed `app.json`. The e2e script covers both signatures (steps 4-6).
- Settings, updater, and core all change; workspace tests green (core 347,
  settings 118, updater 2, dict 87) with new tests locking the default-on
  flip, missing-key upgrade, bidirectional compare, v1-compatible status, and
  note plain-texting.
- The About page shows "已是最新版本" or "（发布源未提供应用版本信息）" until a
  release ships `app.json` — correct degradation, not a bug.
- Because the local machine has no release private key, full signing is
  verified via `-SkipSign` dry-run only; "find new version + changelog" needs
  a real v0.1.3 release to observe end-to-end (recorded in acceptance §22).

## Related

- Reverses the default-off decision of
  [2026-09-29-dictionary-update-trust-chain](../../implemented/architecture/2026-09-29-dictionary-update-trust-chain.md)
  (P-03 onboarding); trust chain mechanism itself (P-04) is unchanged.
- Partially supersedes
  [2026-10-02-settings-about-and-update-page](../../implemented/feature/2026-10-02-settings-about-and-update-page.md)
  (UI gating stays, default flips) and
  [2026-10-04-settings-window-five-upgrades](../../implemented/feature/2026-10-04-settings-window-five-upgrades.md)
  (check-once gating stays, default flips).
- Neighboring update-path change: [2026-10-13-updater-spawn-no-console-window](../../implemented/bug-fix/2026-10-13-updater-spawn-no-console-window.md)
  (T-153, same check/apply subprocess plumbing).
- Acceptance: docs/验收标准.md §22; todos-list T-154.
