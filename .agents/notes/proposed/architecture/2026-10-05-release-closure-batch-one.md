# Agent Note: Release closure batch one — exe installer, update source, Pages (phase 12)

Status: proposed

[中文](2026-10-05-release-closure-batch-one.zh.md) | English

## Problem

Three release-closure gaps remain open after M14, all recorded in the phase-10 website
requirements (D-69 download form, P-01 mirror deferral, D-68 real Pages deployment) and in
the site roadmap as `流程收尾中`:

1. **No exe installer.** The download page offers only the portable zip (内测期产物); the
   exe row is marked `规划中`. The only install entry is an administrator PowerShell run of
   `scripts/install.ps1` — scriptable and rollback-safe, but not a product surface, and
   there is no native uninstaller entry point.
2. **The update source has never actually worked.** Verified on 2026-10-05 (T-096
   investigation): the shipped default URL
   `https://github.com/menghun3-cn/zhu-ye-ime/releases/latest/download/manifest.json`
   redirects **302 → 404**. Root cause is a GitHub Releases semantic:
   `releases/latest` resolves only to the latest **non-prerelease** release, while every CI
   publication (`release.yml`, `--prerelease` when the version contains `-`) is marked
   prerelease and — verified via `gh api releases/latest` — GitHub refuses to make a
   prerelease the latest even with the explicit `--latest` flag. `latest` therefore stays
   on v0.1.1-alpha (published by hand without the flag; its only asset is a zip, **no
   manifest.json**), while v0.1.2-alpha carries all six assets but is invisible to the
   `latest` semantic. The updater client hard-codes that URL at compile time
   (`crates/zhu-ye-updater/src/main.rs:30-31`), so the shipped client can never fetch a
   manifest. Corollary: **as long as releases keep the `-alpha` (prerelease) pattern, a
   `latest`-based URL is permanently broken** until a non-prerelease release exists.
3. **Pages has never deployed.** `pages.yml` exists, the repository Pages config is
   `build_type=workflow`, but no deployment has ever succeeded (`status=null`); the site is
   browsable only in the repository.

## Proposal

Recommended scope for phase 12 (D-74 to D-78, **pending user confirmation**; batch one
covers the three release-closure items as FR-061/FR-062/FR-063):

- **D-74 exe installer tech: Inno Setup.** `ai-zhu-ye-ime-setup-<ver>.exe`, single x64
  file, UAC manifest, native uninstaller. The exe is a **product shell over the existing
  scripts**: the payload is the same package staging tree as the portable zip
  (`bin/scripts/packs/docs`), released to `{tmp}\package`, and the install step invokes
  `install.ps1` (with `-SkipBuild -PackageRoot`) so versioned-DLL copying, PE export
  validation, TSF registration with rollback, pre-placed domain packs, the Start-menu
  shortcut, and delayed old-DLL cleanup are all reused verbatim — no behaviour rewrite, no
  second registration implementation. Uninstall calls the uninstall semantics
  (`Remove-TsfRegistration` + directory/residue cleanup) and explicitly spares
  `%APPDATA%\ai-zhu-ye-ime` user data.
- **D-75 no Authenticode signature.** Accept the SmartScreen `更多信息 → 仍要运行` step;
  `build-setup.ps1 -SignCommand` leaves a hook for the day a certificate exists.
- **D-76 update source = separate mirror distribution feed.** Create
  `menghun3-cn/zhu-ye-updates`; the release workflow atomically pushes
  `manifest.json + it/med/slang.zyct` there (≈3 MB). The client's `DEFAULT_MANIFEST_URL`
  moves to that feed's stable URL (Releases-`latest` of a repo that never publishes
  prereleases, or its Pages URL — final choice in M15-D). This bypasses the latest defect,
  delivers the P-01 "镜像配置留待后续" item, and keeps the site and the update feed
  **decoupled** so a site redeploy can never overwrite the feed.
- **D-77 `online_update` stays default-off (P-03 re-affirmed).** 开通 means "the link works
  once enabled", not "enabled by default"; zero-network-while-off stays an audited invariant.
- **D-78 keep the `vX.Y.Z-alpha` release cadence.** The mirror no longer depends on the
  `latest` semantic, so the prerelease marker stops blocking the update source; a
  non-prerelease release can wait for the usual formal-release moment.
- **FR-063 Pages deployment** = trigger the existing `pages.yml`, then a live-verification
  script (`scripts/verify-site-live.ps1`) over the six pages, AI entries, and download
  links; update the download page (exe available + direct link), hero-note/footer status
  lines, and the roadmap item to `已开通`.

## Alternatives considered

**Rewrite install logic in Inno Pascal.** Rejected: a second implementation of TSF
registration and DLL deployment would recreate the drift problem the TSF
registration-ownership note closed, and it retests rollback semantics that are already
accepted. The shell approach keeps one authoritative implementation.

**Host the update feed inside the site (`site/packs/`) and deploy both in one artifact.**
Rejected after adversarial analysis: Pages in workflow mode replaces the **entire** site on
every deploy, so the next `pages.yml` run on any `site/**` change would silently drop the
feed until a release re-uploaded it — a periodic disappearing update source.
(Alternative: teach `pages.yml` to read the latest release's assets and merge them — too
complex and error-prone for the benefit.)

**Pin `DEFAULT_MANIFEST_URL` to a concrete version tag.** Rejected: the client constant
would freeze the client at one version forever; a pinned URL never discovers newer packs.

**Wait for a non-prerelease formal release to "open" the source.** Rejected: it couples
update-source verification to a release date and leaves the shipped client broken in the
meantime (today it 404s before it even verifies).

**Drop the prerelease marker in `release.yml`.** Rejected: it pollutes the `latest`
semantic with in-test builds and reverses the project's alpha-means-prerelease convention.

**Reproduce `install.ps1` behaviour in the installer entirely.** Rejected for the same
reason as the Pascal rewrite: one authoritative implementation (D-74).

## Consequences

- The update trust chain (ed25519 signature in `manifest.json`, public key compiled in via
  `ZHU_YE_RELEASE_PUBLIC_KEY`) is unchanged — only the URL moves, so mirroring does not
  change the trust anchor; the private key still lives only in the Actions secret.
- The updater keeps `ZHU_YE_MANIFEST_URL` overriding support (test/private feeds).
- The distribution repo must stay hygienic: protected branch, no prerelease-flagged
  releases (no workflow publishes there), no key material pushed.
- Installation and uninstallation keep their accepted rollback behaviour; overwrite installs
  are idempotent; user data is never touched by uninstall.
- The zip install path remains as a fallback download (parity with the exe payload is
  asserted by hashing the shared staging).
- Release CI gains one asset (setup exe, 7th) and one or two steps (build setup, publish
  feed); assemble/verify gates stay in place before each.

## Acceptance criteria

> Mapped to acceptance standard §17: 17.1 exe installer, 17.2 update source, 17.3 Pages,
> 17.4 regression (eval zero-regression Top1 84.7% / Top3 97.2% / sentence 21.0% MISS 385,
> full host-e2e, bench no-degradation, four gateways green).

- 17.1: asset present; clean-machine install → TSF registered → typing works; overwrite
  idempotent; uninstall leaves no registration/registry residue and spares `%APPDATA%`;
  e2e adds a silent install into a temp dir with parameter-connectivity assertion; zip/exe
  payload hashes identical.
- 17.2: default URL fetches a manifest (no 404); apply verifies signature, lands atomically,
  and the `.bak` rollback works; a tampered pack is refused; feed hashes match the Release
  assets; `online_update: false` stays zero-network; `ZHU_YE_MANIFEST_URL` override intact.
- 17.3: `verify-site-live.ps1` green over six pages + AI entries + download links;
  `check-site-structure.ps1` updated and green; status lines and roadmap live.
- 17.4: eval zero-regression (Top1 84.7% / Top3 97.2% / sentence 21.0%, MISS 385), full
  host-e2e green, bench no-degradation, four gateways green.

## Risks

**GitHub delivery flakiness (feed push, CDN latency).** Mitigated by reusing the existing
retry loops and by bounded CDN-delay retries in the live e2e.

**Installer/script parameter drift breaks the shell design.** Mitigated by the silent-install
e2e step that asserts parameter connectivity before assets are uploaded.

**The distribution repo could acquire a prerelease or key material.** Mitigated by
protected-branch hygiene and a hygiene check in the live e2e.

**Unsigned exe suppressed by SmartScreen.** Accepted (D-75), documented on the download
page, and reversible via the SignTool hook.

**Uninstall deletes user data.** Explicitly excluded by the uninstall path and asserted in
e2e (`%APPDATA%\ai-zhu-ye-ime` preserved).

**Mirror becomes the new single point for updates.** Accepted: a single mirror now, with
multi-mirror deferred (the manifest schema has no URL fields, so a later multi-mirror design
stays client-side).

**Batch one partially depends on user confirmation.** Scope and the three feed choices
(D-74~D-78, plus the Releases-vs-Pages feed URL) are written as recommended drafts; the
docs batch (T-096) lands as a draft PR pending confirmation, and implementation tasks
T-097/T-098/T-099 start only on confirmation.
