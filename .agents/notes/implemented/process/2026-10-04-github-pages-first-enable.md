# Agent Note: GitHub Pages first enablement and environment branch policy (v0.1.1-alpha release)

Status: implemented

English | [中文](2026-10-04-github-pages-first-enable.zh.md)

## Problem

The v0.1.1-alpha release batch carries `site/` (T-084 official website) — the
first real deployment of the site. After the `release/vX.Y.Z` PR merged into
`main`, the `pages.yml` workflow fired but failed: the first run reported "Get
Pages site failed... repository has Pages enabled and configured to build using
GitHub Actions" from `actions/configure-pages@v5` (the GitHub REST Pages API
returned `404 Not Found` — the repository had never had a Pages site created);
after creating the site with
`gh api -X POST /repos/{o}/{r}/pages -f build_type=workflow`, every subsequent
run finished **completed|failure with no job steps (failed in ~2 seconds)**,
with missing log blobs and `error: null` — the job was rejected before a
runner was allocated.

## Decision

- Automated first-enablement path: `gh api -X POST repos/{owner}/{repo}/pages
  -f build_type=workflow` (equivalent to Settings → Pages → Source: GitHub
  Actions).
- Creating the site auto-creates a **`github-pages` environment** with a
  `branch_policy` protection rule (`deployment_branch_policy.custom_branch_policies:
  true`) that by default lists only **develop** (the default when POST omits
  `source`). `pages.yml` builds and deploys from `main`, so the job is rejected
  by the environment protection.
- Fix: add `main` to the environment's deployment branch policy:
  `gh api -X POST repos/{owner}/{repo}/environments/github-pages/deployment-branch-policies
  -f name=main -f type=branch` (or Settings → Environments → github-pages → add
  a deployment branch).
- Re-run: `gh workflow run pages.yml --ref main` (pages.yml has
  `workflow_dispatch`) or push to `main` again.

## Alternatives considered

- **Deploy from develop in `pages.yml`**: rejected — Pages must be fed by the
  production branch (`main`); develop content has not passed the release gate.
- **Switch environment protection to `protected_branches: true`**: workable but
  too broad (opens every protected branch); adding branches on demand is more
  precise.

## Consequences

- Diagnostic cheat sheet: a workflow run that is **completed|failure with
  `steps: []`, finishing in ~2 seconds, with missing logs** ("log not found") =
  the runner never started = environment protection rejected it; check
  `GET /environments/github-pages/deployment-branch-policies` first. If
  `configure-pages` reports "Get Pages site failed" = Pages not enabled; POST
  the site first.
- `site/README.md` deployment section now carries the first-enablement notes
  ([site/README.md](../../../../site/README.md), from L101), pinning the two
  API calls.
- The `github-pages` environment branch policy is repository state, not in git;
  it must be reconfigured after repo re-creation/migration. `pages.yml` itself
  declares no policy beyond environment permissions, so deployments do not fail
  on that account on clones.
- Editing `pages.yml`, changing a custom domain, or rolling back still uses the
  same environment policy.

Verification (2026-10-04): after adding `main` to the branch policy,
`gh workflow run pages.yml --ref main` succeeded (deploy-pages
completed|success); `https://menghun3-cn.github.io/zhu-ye-ime/` returns 200 with
hero-note and download page showing v0.1.1-alpha, `llms.txt` consistent;
Release v0.1.1-alpha carries `zhu-ye-ime-0.1.1-alpha-test.zip` (18.56 MB).

Related records (kept active, cross-linked):
[official-website-and-brand-assets
(architecture/2026-10-03-official-website-and-brand-assets.zh.md)](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.zh.md)
— source of the site/ deployment config and S-18 wording;
[work-branch-pr-flow
(process/2026-09-29-work-branch-pr-flow.zh.md)](../../implemented/process/2026-09-29-work-branch-pr-flow.zh.md)
— the develop→release→main→tag chain.
