# Agent Note: Work-branch PR flow into develop

Status: implemented

English | [中文](2026-09-29-work-branch-pr-flow.zh.md)

## Problem

The branch model (T-019) covered the release leg: `develop` →
`release/vX.Y.Z` → `main` → tag. It left open how day-to-day work reaches
`develop`. In practice, features, fixes, and docs were committed straight onto
local `develop`, and unreviewed commits piled up there (nine unpushed commits
by 2026-09-29). That gives no review point before integration, and pushing
them would violate the "no direct push to protected branches" rule anyway.

## Decision

Every change — feature, bug fix, optimization, refactor, docs, or process —
starts on a work branch cut from the latest `develop`, and it enters
`develop` only through a PR (`AGENTS.md` 5.2):

- Branch name: `<type>/T-xxx-<short-english-desc>`, where `type` matches the
  commit type (`feat/T-045-slang-pack`, `fix/T-043-candidate-border`).
- One branch per task or milestone batch. A branch may hold several commits,
  and each one passes the section 5 gates on its own.
- When the work is done, pass the gates, push the branch, and open a PR
  targeting `develop`. The PR title follows the commit format, and the body
  lists changes, verification evidence, and linked T numbers. Merge it into
  `develop`.
- Only after that does the release leg run, unchanged: `develop` →
  `release/vX.Y.Z` → PR into `main` → tag → sync back to `develop`
  (git-publish). A work branch never opens a PR against `main`.
- After a merge, delete the remote and local work branch, then switch back to
  and update `develop`.

## Alternatives considered

**Keep committing on `develop` and push directly.** Rejected: it bypasses
review and contradicts the protected-branch rule already in `AGENTS.md` 5.1.

**Open work-branch PRs directly against `main`.** Rejected: `main` is
production and receives only release PRs, so `develop` stays the single
integration point where changes combine before a release.

**Enforce with a local pre-push hook now.** Deferred: a server-side rule on
`develop` works better and can be enabled later. Until then the rule lives
in `AGENTS.md`, and agents follow it.

## Consequences

- Every change gets a PR review point and a CI run before integration, and
  `develop` history groups work by task.
- Costs: each task needs extra branch, push, and PR steps, which in turn need
  network access and an authenticated `gh`.
- Extends, without superseding,
  [repo-local AI workflow adaptation](2026-09-18-project-ai-workflow-adaptation.md),
  which still owns the release-leg model. Tracked as T-047 in
  [todos-list](../../../../docs/todos-list.md).
