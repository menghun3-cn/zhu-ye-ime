# Agent Note: Abbreviation/Fuzzy toggles (T-103, O-05 revision)

Status: proposed

[中文](2026-10-05-abbreviation-fuzzy-switches.zh.md) | English

## Problem

Decision O-05 (confirmed 2026-10-02, note 2026-10-02-ime-experience-optimization)
said "abbreviation and fuzzy pinyin stay on by default, no config switch". The
user's accepted batch-three list explicitly asks for a "简拼/模糊音关闭开关"
(abbreviation/fuzzy off switches), which revises O-05: users who find
abbreviation expansion or fuzzy correction too aggressive need a way to turn
either off without abandoning the input method.

## Proposal

1. **Config** (`zhu-ye-core/pack_config.rs`, `ConfigFile`): two new booleans
   `enable_abbreviation` and `enable_fuzzy`, default `true` via serde
   `default = "default_..."` fns (same permissive pattern as
   `enable_domain_boost`). Old configs without the fields load as on;
   `CONFIG_FORMAT_VERSION` is NOT bumped (pure additive optional fields,
   T-073 rationale).
2. **Engine** (`zhu-ye-ime/input.rs`): `InputEngine` gains the two fields plus
   `with_abbreviation(bool)` / `with_fuzzy(bool)` builders (mirroring
   `with_domain_boost`). Two candidate-path gates:
   - abbreviation off → the FR-023 first-initial expansion block (empty main +
     2-4 lowercase + unsegmentable) never runs;
   - fuzzy off → the FR-024 corrected group (fuzzy substitution + missing-
     letter completion, both classes) is never generated.
3. **Assembly** (`zhu-ye-ime/tsf.rs`): engine creation chains
   `with_abbreviation(config.enable_abbreviation)` /
   `with_fuzzy(config.enable_fuzzy)` next to `domain_engine`.
4. **Deliberate boundaries** (documented, not configurable):
   - slang abbreviation path (FR-016/FR-017, M6-R) is unchanged;
   - contact-index native abbreviation keys (FR-037) are unchanged — they are
     first-class index keys of an explicit high-confidence store, not the
     dictionary abbreviation decoder;
   - no settings-window GUI this batch (same tier as `enable_domain_boost`:
     manual `config.json` editing; GUI stays a future enhancement).

## Alternatives considered

- **One combined "简拼/模糊音" switch**: coarser than user intent ("关闭开关"
   implies both exist separately); two independent booleans cost nothing.
- **Route fuzzy off through a variant cap of 0**: works but hides the intent
   and couples to an internal constant; an explicit `enable_fuzzy` condition is
   self-documenting.
- **Settings GUI entry**: user item names a switch, not a GUI; consistent with
   `enable_domain_boost` (config-only) and avoids widening this batch (lsg) —
   deferred, noted in settings design §5.1.

## Acceptance criteria

- Workspace green: core `pack_config` 27 (incl. new round-trip/legacy/solo
  switch test), ime lib 191 (incl. 5 toggle tests: nh off, whole-word on,
  zongguo off, niha off, contact abbreviation key unaffected).
- host-e2e: `--m7` real dictionary 27/27 (incl. "switches off → nh/zongguo
  untouched"), `--m12` 9/9.
- Docs: requirement §12 (O-05 row with 2026-10-05 revision, FR-023/FR-024
  switch lines), design §12.1 note + new §12.3.4, acceptance §8 (3 rows),
  architecture §10.1, settings design §5.1 (notes no GUI).
- Gates: fmt --check / clippy -D warnings / git diff --check /
  verify-agent-notes / verify-translation-pairs clean.

## Risks

- **Switch semantics mismatch**: users might expect the switches to also turn
  off contact abbreviation or `[网络]` abbreviation. Mitigated by explicit
  boundary documentation in requirement §12, design §12.3.4/§12.2, and this
  note; the default stays on so behavior is unchanged unless opted out.
- **Config stale read**: `enable_domain_boost` is read once at engine creation
  (restart applies); the new switches share that exact lifecycle — documented
  via the existing "重启输入法后生效" convention.
- **Fuzzy semantics**: `enable_fuzzy=false` disables both correction classes
  (A substitution + B completion). A user wanting only substitution would need
  finer granularity; judged unnecessary now (both are "打错/打快" corrections),
  noted in the design doc as the current scope.
