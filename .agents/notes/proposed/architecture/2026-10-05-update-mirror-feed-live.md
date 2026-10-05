# Agent Note: Update mirror feed live — zhu-ye-updates (phase 12, T-098)

Status: proposed

[中文](2026-10-05-update-mirror-feed-live.zh.md) | English

## Problem

The shipped client's default manifest URL pointed at the main repo's
`releases/latest`, which is permanently 404 while every release carries the
prerelease flag (see `2026-10-05-release-closure-batch-one.md` §Problem 2, verified
2026-10-05). The phase-12 recommendation (D-76) was an independent mirror feed so the
update source no longer depends on the `latest` semantic of a prerelease-only repo. T-098
executes that decision.

## Proposal

1. **Feed repo `menghun3-cn/zhu-ye-updates`.** The release workflow (or a manual publish)
   pushes `manifest.json + it/med/slang.zyct` there as a GitHub Release.
2. **Feed releases carry no `--prerelease` flag**, so `releases/latest` resolves to the
   newest feed. Key insight recorded here: *prerelease is an explicit flag on a release,
   not derived from the tag shape* — a tag like `feed-v0.1.2-alpha` stays a
   non-prerelease release unless the flag is passed. The earlier assumption that
   `-` in a tag forces prerelease was wrong; only `release.yml`'s own convention adds
   `--prerelease` for `-alpha` versions (and that convention is **not** applied to feed
   releases).
3. **URL structure needs no client logic change.** `releases/latest/download/<asset>`
   serves any asset name under the latest release, and the updater derives
   `base_url` = directory of the manifest URL, so
   `https://github.com/menghun3-cn/zhu-ye-updates/releases/latest/download/manifest.json`
   → `.../download/it.zyct` etc. just work.
4. **`DEFAULT_MANIFEST_URL` switched to the mirror** in
   `crates/zhu-ye-updater/src/main.rs`; `ZHU_YE_MANIFEST_URL` override retained for
   tests/private feeds.
5. **CI automation (`release.yml` publish-updates step)**: `GITHUB_TOKEN` cannot push to
   another repository, so the step uses the `ZHU_YE_UPDATES_TOKEN` secret (fine-grained
   PAT, contents:write on zhu-ye-updates). When the secret is absent the step warns and
   skips — a try-run / pre-PAT publish never blocks the main release. Feed release
   `--target` pins the publishing commit for provenance.

## Alternatives considered

- **Feed repo Pages URL.** Rejected: adds a deploy step and eventual-consistency wait;
  the Releases-`latest` path is immediately served by the asset CDN.
- **Keep the main-repo `latest` URL.** Rejected: proven 404 (batch-one note).
- **Publish feed releases with `--prerelease`.** Rejected: would recreate the latest
  defect in the feed repo.
- **Rewrite the updater to accept a URL in the manifest.** Rejected: manifest schema
  change for no need — `base_url` derivation already covers it.

## Verification (2026-10-05, host)

- Created `menghun3-cn/zhu-ye-updates`; manually published feed
  `feed-v0.1.2-alpha` (manifest + it/med/slang.zyct), `--target d8685a4…` (mirror main).
- `curl`: `releases/latest/download/manifest.json` → **200**, 1108 B, schema=1, 3 packs
  (was 302→404 on the main repo); `releases/latest/download/it.zyct` → **200**,
  956,228 B, size matches the manifest entry.
- Updater built with the public key injected: `check` fetches the mirror manifest,
  **signature verified with the built-in key**, outdated = it/med/slang.
- Default `online_update:false` stays zero-network (config restored after the probe).

## Consequences

- The trust chain is untouched — only the URL moved; the private key still lives only in
  the Actions secret.
- Feed repo hygiene matters: no prerelease-flagged releases, no key material, protected
  branches.
- Until `ZHU_YE_UPDATES_TOKEN` is configured, feeds are published manually (as above);
  after that, the release workflow publishes automatically after each release.
- `verify-release-e2e.ps1 -Live` mode and VM acceptance (apply / `.bak` rollback /
  tamper refusal / feed-hash parity) remain open for the acceptance phase (§17.2).

## Acceptance criteria

> Mapped to acceptance standard §17.2 (update source).

- Default URL fetches a manifest without 404 — **done** (verified above).
- `apply` verifies signature, lands atomically, `.bak` rollback works; a tampered pack is
  refused; feed hashes match the Release assets — pending live/VM e2e.
- `online_update:false` = zero network — **done** (default check).
- `ZHU_YE_MANIFEST_URL` override intact — override path unchanged in code.

## Risks

- **CDN transient reset** (observed once as curl exit 56 on the first probe; immediate
  retry succeeded). Mitigated by the updater's bounded `--max-time` and retry loops in
  live e2e.
- **PAT absence/rotation** → manual publish fallback; step already degrades to warn+skip.
- **Feed tag collision on re-publish** — `gh release create` failure is ignored and
  `upload --clobber` overwrites assets.

## Supersession

- Partial executor of `2026-10-05-release-closure-batch-one.md` D-76 ("final choice in
  M15-D"): M15-D now executed as Releases-`latest` + explicit non-prerelease feed
  releases. The batch-one note stays active (broader phase-12 scope: exe, Pages, docs);
  cross-referenced.
