# Agent Note: Repo-local AI development workflow adaptation

Status: implemented

[中文](2026-09-18-project-ai-workflow-adaptation.zh.md) | English

## Problem

The repository adopted `.agents/notes` and `.agents/skills` snapshots from a TypeScript/pnpm-oriented repository. Those conventions assume ignored agent files, a pnpm-based toolchain, Conventional Commits, and a Python-project git-publish default. This project is a Rust/Cargo workspace published through git, so blindly following those assumptions would block development, commit, and push workflows with missing commands and conflicting rules. The workflow rules need one authoritative project-level adapter.

## Decision

`.agents/` is versioned with the repository so Agent Notes and skills evolve with the codebase; `.gitignore` no longer excludes it. `AGENTS.md` is the binding adapter: it states the Agent Note/todos division, the commit format `<type>(<scope>): <中文描述> (T-xxx)`, minimal diff-based pre-push verification, and release rules anchored to the root `Cargo.toml` workspace version and a Chinese `CHANGELOG.md`.

Every pnpm, Vitest, `./invariant`, ACP/Loader, HMR, or doc-sync step from the dsh skills is rewritten to Rust/Cargo/PowerShell/git equivalents; concepts with no counterpart in this repository (documentation-site build, web tooling) are marked out of scope and point to the repository's own gates. `git-publish` uses the root `Cargo.toml`, `CHANGELOG.md`, and the decided branch model: integration on `develop`, production on `main`, release branches `release/vX.Y.Z`, and tags `vX.Y.Z` (T-019).

This note records the adaptation under `implemented/process/`, and todos record the branch-model decision as T-019, linked from [docs/todos-list.md](../../../../docs/todos-list.md).

## Alternatives considered

**Copy the skill defaults as-is.** Rejected: pnpm commands and Python version files do not exist here, and protected-branch defaults could conflict with the repository's actual branches once they are created.

**Keep `.agents/` ignored.** Rejected: Agent Notes must version with the decisions they record; ignoring them would hide workflow rules and block review.

**Rewrite every skill from scratch.** Rejected: the dsh skills and git-publish still contain useful patterns for review, pre-push evidence, and release sequencing; adapting them costs less than reinventing equivalents.

## Consequences

Development, commit, and push steps now share one Chinese, Rust-aware source of truth in `AGENTS.md`, removing contradictory instructions. Agent Notes can be reviewed with the code and archived through the standard lifecycle.

The PowerShell verifiers in `scripts/verify-agent-notes.ps1` and `scripts/verify-translation-pairs.ps1` now automate note-tree, archive-seal, and pairing gates; semantic supersession judgment still requires review by hand. The git-publish branch model is fixed as `develop` (integration) → `release/vX.Y.Z` → `main` (production) → tag `vX.Y.Z`, recorded in `AGENTS.md` and the skill configuration (T-019); publishing follows that model. Day-to-day changes reach `develop` only through work-branch PRs (T-047, see [work-branch PR flow into develop](2026-09-29-work-branch-pr-flow.md)).
