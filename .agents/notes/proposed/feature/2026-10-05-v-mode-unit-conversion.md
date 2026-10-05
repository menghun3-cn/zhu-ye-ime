# Agent Note: v-mode unit conversion table (T-104, D-02/13.3 revision)

Status: proposed

[中文](2026-10-05-v-mode-unit-conversion.zh.md) | English

## Problem

Requirement 13.3 originally declared "no unit-conversion table; v mode ships
with one page of common symbols only". The user's accepted batch-three list
explicitly asks for a full unit-conversion table in v mode, which revises that
non-goal: `v` + unit key should produce the unit's equivalents among its family.

## Proposal

1. **Core table** (`zhu-ye-core/units.rs`, new — no Windows API):
   - `UNIT_CONVERSIONS`: 23 keys × ≤9 equivalence strings each, covering 9
     families — length (mi/chi/li), area (mu/gongqing), volume (sheng/fang),
     mass (kg/jin/liang/dun), time (miao/shi/tian), temperature (she/far),
     speed (kmh), data (kb/mb/gb), angle (du/rad).
   - `unit_key_prefix(key)` and `unit_candidates(key)` for the keyed lookup.
   - Data policy: SI multiples exact; 市制 by common conversion (1 米=3 市尺,
     1 市斤=0.5 千克, 1 市里=0.5 千米); imperial/temperature as fixed common
     approximations (4-7 decimals); temperature rows include the fixed-point
     equivalence (0°C=32°F=273.15K) and a delta row. All static, reviewable,
     deterministic — not an external data source.
2. **IME v-mode unit state** (`zhu-ye-ime/input.rs`):
   - `v_code`/`v_accepts` accept unit-key prefix letters (`vm` waits for
     `vmi`/`vmiao`); full key → candidates, unfinished prefix → no candidates.
   - Symbol type codes (`1-9`/`x`/`h`) are only accepted while "waiting for a
     type code" (`v` alone); inside a unit code, letters are judged as units
     first (`vs`+`h` forms `sh`, not the punctuation group).
   - `v_consume` keeps accrued unit letters when falling back to pinyin
     (`vm`+`x` → `vmx`); a settled symbol code (`v1`+`i`) still falls back to
     `vi` as before.
   - Candidate source stays `Symbol` (v-mode candidates share one semantics; no
     new UI tag).
3. **TSF routing** (`zhu-ye-ime/tsf.rs`): letter keys while v is active now go
   through `engine.v_accepts(c)` instead of the hard-coded `x|h` check.
4. **Boundaries** (deliberate, documented):
   - Unit keys never start with `x`/`h` (symbol first-key codes keep priority);
   - after a settled symbol code (`v1`) unit letters are rejected (symbol
     selection semantics unchanged);
   - original fallbacks kept: `vi`/`vx`-style exits, `v` alone shows nothing.

## Alternatives considered

- **Runtime conversion from coefficients**: compute strings at query time.
  Rejected — static strings are reviewable, precise per family (temperature
  fixed points, odd imperial ratios), and keep determinism trivially.
- **New `CandidateSource::VUnit`**: rejected — v-mode candidates already own
  one UI semantic (no label), Symbol covers both groups; a new variant would
  touch every match site for no user-visible gain.
- **Long family keys only (e.g. `vpingfangmi`)**: 8+ letter keys are
  unfriendly; short keys with prefix continuation (`vm`→`vmi`, `vmiao`) keep
  the table "full" while staying fast to type.

## Acceptance criteria

- Workspace green: core units 5 tests; ime lib 197 (6 new toggle tests);
  fmt/clippy(-D warnings)/diff-check clean.
- host-e2e: `--m9` real dictionary 19/19 (added vmi conversion, vjin, select
  commit, `v1` boundary); seed regression + m7 27/27 + m10 12/12 + m12 9/9.
- Docs: requirement §13 (goal 2, non-goal line with revision note, D-02 row,
  FR-028 + acceptance direction), design §4.1.1 full table + §4.2 restored,
  acceptance §9 FR-028 +6 rows.

## Risks

- **`vi` fallback scope change**: letters that are unit-key prefixes now stay
  in v mode (e.g. `vm`). Practically no valid pinyin starts with `v`+these
  consonants, and complete v-mode control (Esc/backspace) is available; the
  fallback for non-prefix letters (`vi`) is unchanged.
- **vmi/vmiao ambiguity**: `mi` is a prefix of `miao`; typing `vmi` yields 米
  (full key wins at that point), continuing yields 秒. Documented in the unit
  table and covered by a prefix-continuation test.
- **Static approximation drift**: imperial/temperature constants are fixed
  common approximations; any future need for higher precision is a data-only
  change to units.rs.
