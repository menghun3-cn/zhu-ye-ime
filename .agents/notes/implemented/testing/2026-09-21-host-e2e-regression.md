# Agent Note: Host-side end-to-end regression

Status: implemented

English | [中文](2026-09-21-host-e2e-regression.zh.md)

## Problem

FR-001 through FR-007 core input behavior had unit tests and ad hoc CLI checks, but no single repeatable host-side command could verify the dictionary-driven input loop before VM acceptance. Regressions between changes could slip through while the VM environment was still being prepared.

## Decision

`crates/zhu-ye-ime` now ships a `host-e2e` binary that loads a v2 dictionary file and drives `InputEngine` through 17 seeded checks: dictionary loading, `nihao` candidates with inline translation, `xian` ambiguity, deterministic ordering, numeric selection, candidate pagination and wrap-around, space/Enter/Esc, Shift mode, user-word promotion and persistence, deletion/reset, bigram context ranking, translation-layer switching, the no-translation guard, and forward/reverse translation. `scripts/e2e.ps1` ensures `data/artifacts/seed.zyct` exists, builds the checker, runs it, and then runs a real-dictionary smoke (`jiao` keeps 叫, `xian` has no 洗按, and 你好/good translations resolve) when `data/artifacts/real.zyct` is present. Any failed check makes the binary exit non-zero, so the script can be used as a gate.

## Alternatives considered

**Extend cargo unit tests only.** Rejected: unit tests are not an acceptance command. The PowerShell script turns the executable into a repeatable, machine-visible gate without losing readable per-check output.

**Parse `demo`/`rank` CLI text.** Rejected: text snapshots are brittle, and `demo` does not cover submission flow, user-word promotion, or the translation layer.

**Wait for VM acceptance.** Rejected: host-side checks can cover the full core algorithm now; VM acceptance stays focused on TSF registration, candidate-window rendering, and system-level interaction.

## Consequences

Every dictionary/input change can be gated by `scripts/e2e.ps1` on the developer machine. The seed dictionary keeps checks deterministic; the real-data smoke is optional and only runs when the artifact exists. This does not replace VM end-to-end acceptance for TSF and GUI behavior.
