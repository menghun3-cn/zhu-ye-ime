# Agent Note: Updater e2e via local mirror — network-independent acceptance channel (T-098)

Status: proposed

[中文](2026-10-05-updater-e2e-sandbox.zh.md) | English

## Problem

§17.2 (update source online e2e: check / apply / tamper rejection / rollback /
consistency) normally needs a live network to GitHub plus a real feed. On 2026-10-05 the
GitHub channel from this host was flapping (connection resets, 21s connect timeouts,
5/5 failures in a row), and the acceptance VM was unreachable (SMB refused on both the
old IP and the only same-subnet SMB host). The mechanism-level acceptance had to be
completed regardless, without touching the user's real configuration.

## Proposal

Adopt the local-mirror drill as the standard network-independent updater e2e channel:

1. **Isolated user profile**: run the updater with `APPDATA`/`LOCALAPPDATA` pointed at a
   scratch dir under `target/release-e2e/`, with a minimal `config.json`
   (`{"online_update":true}`). Nothing touches the user's real `%APPDATA%`.
2. **Local feed mirror**: serve the published snapshot
   (`target/release-assets/<ver>/packs/`) over a tiny `HttpListener` on 127.0.0.1;
   point the updater at it with `ZHU_YE_MANIFEST_URL=http://127.0.0.1:<port>/manifest.json`.
   This exercises the exact same code path as the real channel (the override mechanism
   is a documented feature, §17.2 "覆盖机制保留").
3. **Tamper + rollback assertions** (8 checks, all green on 2026-10-05):
   - tampered pack in the mirror → `apply` exits non-zero with
     `包内容校验失败: 包 it 内容哈希不一致` and nothing is placed in `packs/`;
   - restore the mirror → `apply` succeeds and the replaced pack hash matches the
     manifest;
   - corrupt an already-installed pack in `packs/` → `apply` treats it as outdated,
     backs it up to `<id>.zyct.bak` and atomically restores the original hash
     (`.bak` keeps the corrupt copy as evidence — this is the rollback reserve).
4. **Online consistency**: when GitHub responds, `curl -fsSL` the four feed files and
   compare SHA-256 against the local snapshot (4/4 matched on 2026-10-05: manifest
   7b7752f6…, it 57bc745f…, med 489c1e76…, slang d1eb2c1a…).
5. **Repo hygiene**: `gh api` on the mirror repo — `releases/latest` shows
   `"prerelease":false` (tag contains `-alpha` but the release is not marked
   prerelease, so `releases/latest` semantics hold); `main` branch protection was
   configured via `gh api -X PUT …/branches/main/protection` with
   `required_pull_request_reviews.required_approving_review_count=1` (HTTP 404
   confirmed it was unprotected before).

## Acceptance criteria

- The drill runs entirely inside `target/release-e2e/` with redirected
  `APPDATA`/`LOCALAPPDATA` (no writes to the real user profile).
- Tampered pack → non-zero exit, distinctive hash-mismatch message, nothing placed
  under `packs/`.
- Restored mirror → `apply` succeeds; installed pack hash equals the manifest value.
- Corrupted installed pack → re-`apply` re-downloads the original, keeps
  `<id>.zyct.bak` with the corrupt copy, restores the original hash.
- Online 4/4 SHA-256 comparison (manifest + three packs) passes when GitHub responds;
  otherwise the deferral to the release run is stated explicitly.
- The HTTP server job is killed after the drill.

## Alternatives considered

- **Waiting for the flaky GitHub channel to stabilize and running the real-network
  e2e directly**: rejected — five consecutive failures made the wait unbounded, and
  §17.6 mandates filling the acceptance table rather than deferring everything.
- **Running the drill against the real manifest URL with cached pack bytes**: rejected —
  tamper rehearsal requires serving modified bytes, which needs a controlled endpoint;
  the local mirror is the only deterministic way.
- **Replaying the updater's unit tests as e2e evidence**: insufficient — acceptance
  needs the real CLI binary end-to-end, including staging and atomic rename on disk.

## Consequences

- When GitHub/CDN or the VM is flaky, the mechanism-level updater e2e must not be
  silently skipped — the local-mirror drill above is the fallback, and the real-network
  full `apply` is re-run once at the v0.1.2 official release.
- Keep `target/release-e2e/` as the repeatable home for this drill (scripts
  `http-server.ps1`, `tamper-e2e.ps1`); it is outside git by the `target/` rule.
- The updater already guards both layers (manifest signature + per-pack SHA-256), which
  is what makes the drill pass; the failure message naming the exact hash is useful
  evidence for future acceptance records.

## Risks

- A local mirror that differs from the real feed would produce false PASSes; always
  pair the drill with the online 4/4 hash comparison (or state explicitly when the
  online check is deferred to the release run).
- HttpListener on an arbitrary port is local-only; kill the server job after the drill
  (managed background job), never leave it serving.
