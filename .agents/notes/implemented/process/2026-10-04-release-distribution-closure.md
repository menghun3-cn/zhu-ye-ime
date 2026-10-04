# Agent Note: Release distribution closure and tooling (P-01, T-094)

Status: implemented

[English](2026-10-04-release-distribution-closure.md) | 中文

## Problem

M6-U (see [dictionary update trust chain](../architecture/2026-09-29-dictionary-update-trust-chain.md))
delivered the trust mechanism -- signed manifest plus a compile-time built-in public
key -- but no formal channel from source code to user machines existed. Four gaps
became acute after v0.1.1-alpha (2026-10-04, the first engine shipping the updater)
was released:

1. **No key generation tool**: `sign-manifest` requires `ZHU_YE_RELEASE_SECRET_KEY`,
   yet the repository had no tool or manual for producing that key pair -- a first
   release could not even prepare key material.
2. **Release builds never injected the public key**: the updater's key comes from
   `option_env!(ZHU_YE_RELEASE_PUBLIC_KEY)`, but no build step set it. Without a
   built-in key the updater **refuses every update**, so the v0.1.1-alpha updater
   was effectively unusable.
3. **No release-asset assembly or end-to-end verification**: building the
   distributable packs, signed manifest, and installer zip was unstructured, with no
   repeatable full-chain preflight.
4. **No release runbook**: key management, Git release steps, asset upload, and
   update-source verification were scattered and could not be executed step by step.

## Decision

### `zhu-ye-dict keygen`: release key pair generation

`zhu_ye_core::generate_keypair()` (in `manifest.rs`) uses `getrandom` (system
entropy source; cross-platform, no rand/Windows-only API) to produce a 32-byte
seed and derives the ed25519 public key; it returns `(secret hex, public hex)`.
The secret lives only in the release environment (see `docs/发布流程.md`) and
**never enters the repository**; the public key may. New CLI subcommand plus unit
tests (derived key consistency, distinct pairs across calls).

### `scripts/assemble-release.ps1`: release asset assembly

Reads the version from the root `Cargo.toml`, then forces
`cargo clean -p zhu-ye-updater` before building -- the `option_env!` key change is
**not observed by cargo's incremental cache**, so skipping the clean can bake a
stale key into the shipping updater (the first manual run of this pipeline hit
exactly that). Assembles `target/release-assets/<v>/`:

- `packs/{it,med,slang}.zyct`: only distributable packs enter the manifest
  (`DISTRIBUTABLE_PACK_IDS`); `base.zyct` ships read-only with the installer and
  `en.zyen` is an engine asset -- neither is in the update source;
- `manifest.json`: `build-manifest` (pack version = publish date, `min_engine
  0.1.1`) -> `sign-manifest` -> `verify-manifest` (the per-pack hash/size recheck
  item 7.4 of the release checklist);
- `ai-zhu-ye-ime-<v>.zip` (renamed copy of the `package-portable.ps1` output);
- `SHA256SUMS.txt` (upload checklist for `gh release upload`).

### `scripts/verify-release-e2e.ps1`: full-chain preflight

Runs the whole update path over curl's `file://` scheme without real network (TLS
transport is upstream's concern; locally we verify "fetch -> built-in-key verify ->
download -> hash/size check -> staging+rename atomic landing -> idempotence"):
1) `apply` lands every distributable pack; 2) landed packs match manifest
hash/size; 3) a tampered landed pack is detected and re-fetched; 4) a tampered
manifest is rejected by the built-in key.

### `docs/发布流程.md`: release runbook

Steps 1-6 (git-publish -> build/assemble with injected key -> upload Releases ->
update-source verification -> release-checklist close-out -> sync back to
develop) plus key management (generate/store/rotate/leak plan) and a failure-plan
table.

### Version-gate fix (critical defect found by e2e)

`version_at_least` used to parse each segment with `parse::<u64>()`; `2-alpha` in
`0.1.2-alpha` failed and counted as 0, so the client engine version became `0.1.0`
-- always below `min_engine 0.1.1`, meaning **a release tagged v0.1.2-alpha would
be rejected by its own updater**. Fixed to take the leading digits of each segment
(`2-alpha` -> 2), keeping the existing "non-numeric = 0, missing segment = 0"
semantics; added six pre-release assertions.

## Alternatives considered

- **keygen in PowerShell/openssl**: rejected -- Rust adds no dependency
  (`getrandom` is a portable entropy standard), is unit-testable, and stays on the
  `zhu-ye-dict` command surface.
- **e2e over a local HTTP server**: rejected -- `file://` exercises the same
  download abstraction through `curl.exe` while avoiding port/URL-ACL/antivirus
  noise; the real HTTPS path is provided by GitHub Releases and verified in a live
  environment after publishing (pending VM recovery).
- **Shipping the `package-portable.ps1` `-test` zip name as-is**: rejected --
  release assets use the formal name; the test name stays for internal builds.

## Consequences

- A release is: load/generate keys -> build with injected public key ->
  `assemble-release` -> `gh release create/upload` -> `verify-release-e2e`; the
  updater only becomes genuinely usable **from v0.1.2-alpha on** (earlier artifacts
  carry an empty built-in key and refuse updates).
- Key rotation is expensive (compile-time trust anchor, needs an engine release);
  recorded in the runbook's failure plan.
- This note is process-level and does not replace the trust-chain mechanism note
  (`2026-09-29-dictionary-update-trust-chain`, kept active, cross-referenced).
- The test key pair is for local pipeline verification only; **generate a fresh
  pair before the actual release** and do not reuse it.
- Evidence: `assemble-release` full run passed (ed25519 signature + verify 3/3);
  e2e script 4/4 green; `cargo test` 291 items including the new keygen and
  version-gate assertions.
