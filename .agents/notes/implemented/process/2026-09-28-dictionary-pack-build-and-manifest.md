# Agent Note: Domain pack build and manifest pipeline (M6-P)

Status: implemented

English | [中文](2026-09-28-dictionary-pack-build-and-manifest.zh.md)

## Problem

M6 ships multiple dictionary packs (base, domain packs, slang) that the
runtime later composes. THUOCL-style word lists carry only words plus a
document frequency — no pinyin — so a build step must annotate pinyin before a
pack can be compiled into the ZYDT v2 binary format. The pipeline also needs a
machine-checkable way to publish a set of packs (manifest) and to prove cache
inputs are still the pinned versions before a build (source-check). Nothing in
phase one covered this: `zhu-ye-dict` had `build/import/inspect/verify` only,
and there was no per-pack reproducibility gate.

## Decision

`zhu-ye-dict` gained four M6 subcommands, all implemented in `m6.rs`:

- `source-check` — reads every `data/pins/*.json`, hashes the referenced cache
  or snapshot file, and fails the build unless every locked hash matches.
  Unlocked pins (missing `sha256`) are reported but do not fail. `build-pack`
  runs `source-check` first, so a drifting source can never silently feed a
  build.
- `build-pack <it|med>` — parses `THUOCL_*.txt` (`词<TAB>DF`), keeps only pure
  CJK words, annotates pinyin, dedups on (word, pinyin), compiles with
  `build_v2` and writes `data/artifacts/<id>.zyct`. Frequency = THUOCL DF
  capped at u32::MAX; domain packs carry no translations (the existing
  pipeline already filters empty translations).
- `build-manifest [dir] [--version V] [--min-engine V]` — scans `*.zyct` in
  the artifacts directory and writes `manifest.json` (JSON schema 1) with per
  pack `id/name/version/file/sha256/size/min_engine_version` plus top-level
  `schema` and `published_at`. **Unsigned at this milestone; ed25519 signing
  lands in M6-U.** Manifest default `version` is the UTC build date and
  `min_engine_version` defaults to "0.1.0".
- `verify-manifest <manifest.json>` — per package: file exists, content
  SHA-256 and byte size match the manifest; any mismatch fails.

### Pinyin annotation strategy

Annotating uses two tiers, both validated against the engine's standard
syllable table (`zhu_ye_core::pinyin::SyllableTable`):

1. Word tier: CC-CEDICT (D-001) word → pinyin map built with the existing
   import cleaning rules (pure CJK, complete standard syllables, syllable
   count == char count). For words with multiple readings, **the first entry
   wins** (CC-CEDICT ordering is frequency-ordered, so the first reading is
   the common one).
2. Char tier: Unihan `kTGHZ2013.txt` (D-014, new pin, MIT) maps each of the
   8,105 standard hanzi to its first pinyin (tone-marked input normalized to
   plain ASCII, `ü` → `v`). Words not in the word tier are annotated
   char-by-char and validated again.

Observed on 2026-09-28: IT pack 16,000 rows → 12,853 entries (1 annotation
failure, 1 duplicate), 0.94 MB; medical pack 18,749 rows → 18,675 entries (74
annotation failures), 1.37 MB. Annotation failures come from characters
outside the standard 8,105 set and are intentionally dropped.

## Alternatives considered

**Ship THUOCL words without pinyin and let the engine segment the chars.**
Rejected: the runtime composes whole-word entries; char-level fallback there
would duplicate annotation work at runtime and costs the standard-syllable
validation that keeps candidate quality.

**Use a single-char pinyin file from another (unpinned) source.**
Rejected for reproducibility: kTGHZ2013 is pinned, MIT-licensed, and covers
exactly the official 8,105-character set, which is the same baseline the base
pack uses — one char set, one source of truth.

**Generate the manifest inside M6-U only.**
Rejected: verifying reproducibility at the *build* milestone (hash check right
after pack creation) needs the manifest output already; signing is then a
purely additive step in M6-U.

## Consequences

- Benefits: every build is hash-gated (pins → annotation → v2 → manifest →
  verify); domain packs are reproducible and size-bounded today; the manifest
  schema is stable input for M6-U's signed manifest and M6-R's runtime
  composite dictionary.
- Costs: a new dependency (`serde`, `serde_json`, `sha2`) in the build-tool
  crate only — the runtime crate is unaffected; annotation failures must be
  watched per-source (74 in medical on first build; documented in
  `docs/数据清单.md` D-014 note).
- Remaining M6-P work is split into follow-up tasks: `build-base` (XDHCY PDF
  text extraction, wordfreq 3.1.1 data extraction, frequency merge, S-1 size /
  coverage calibration) and `build-slang` plus MDN glossary page fetches.
- Supersedes nothing; extends the pins/fetch mechanism documented in
  [dictionary source pins and fetch script](2026-09-28-dictionary-source-pins-and-fetch-script.md).
  The v2 binary format is unchanged (see
  [dictionary binary format v1](../../architecture/2026-09-19-dictionary-binary-format-v1.md)).
