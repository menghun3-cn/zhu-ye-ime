# Agent Note: Composite dictionary and the abbreviation input path

Status: implemented

[中文](2026-09-29-composite-dictionary-and-abbreviation-path.zh.md) | English

## Problem

M6-P shipped four separate dictionary packs (`base`, `it`, `med`, `slang`), but
the runtime still loaded exactly one file: `dictionary.zyct` next to the DLL, with
`%APPDATA%` as a fallback and `ZHU_YE_DICT_PATH` as a test override. Everything the
packs promise — FR-015 (enable a domain pack and its words become reachable),
FR-016/FR-017 (slang words and letter-string abbreviations), FR-022 (config-driven
pack management) — was unreachable for a real user. VM acceptance in T-048 had to
switch packs by rewriting an environment variable and restarting, which is a test
harness, not a product.

Two distinct gaps:

1. **No composition.** `InputEngine` takes a single `Arc<dyn Dictionary>` plus a
   single `Arc<dyn BigramModel>`. There was no way to query several `DictionaryFile`
   instances as one, and no configuration file to say which ones to load.
2. **No abbreviation path.** Slang keys like `yyds` are stored as the pinyin field
   of an entry, but the normal pipeline feeds the composition string through
   `generate_candidates`, which segments it into syllables. `yyds` is not
   segmentable, so it never produced the slang candidate. T-049 fixed the adjacent
   problem of digits entering the composition, but only so that the *key* could be
   typed — the lookup path itself was still missing.

## Decision

### Composition

`zhu_ye_core::CompositeDictionary` holds `Vec<Arc<DictionaryFile>>` and implements
`Dictionary`, `BigramModel`, and `Translator`. Merge rules follow the design doc:
same `(pinyin, word)` is deduplicated with `frequency` taken as **max** (S-6),
bigrams take max, and translations take the first non-empty value.

Equivalence is the regression floor: with a single pack the composite must return
exactly what the `DictionaryFile` returns. The merge preserves first-seen order and
then sorts by descending frequency with a stable sort, which is the same ordering
`DictionaryFile::lookup` already produces.

`from_paths` never fails as a whole: a pack that will not open is skipped and
reported in a diagnostics vector. NFR-009 forbids half-loading, and a single
corrupt pack must not make the IME unusable.

**Hot-path fast channel.** `collect_merging` returns the single non-empty result
directly instead of allocating a hash map. The common configuration — base pack
only — therefore costs nothing extra. Measured with `zhu-ye-cli bench`: multi-pack
lookup went from 9.69 µs to 2.30 µs per query after adding this path, against
2.26 µs for a single pack.

### Configuration

`zhu_ye_core::pack_config` parses `%APPDATA%\ai-zhu-ye-ime\config.json`:

```json
{ "version": 1, "enabled_packs": ["it","med","slang"],
  "online_update": false, "last_check": null }
```

`online_update` defaults to `false`, which is what keeps the default-off guarantee
(P-03) testable at the config layer rather than only in the updater. Unknown pack
ids are filtered into a `unknown` list and logged; a corrupt or unreadable file
falls back to the default (base only) and returns a diagnostic. `plan_packs`
resolves `<packs_dir>/<id>.zyct` and reports configured-but-missing packs
separately from unknown ones, so the two failure modes are distinguishable in logs.

### Abbreviation path

`is_abbreviation_input` gates the path on three conditions, all required:

1. length ≥ `ABBREVIATION_MIN_LEN` (2, per S-2) — a single letter must not fan out;
2. only ASCII lowercase letters or digits;
3. `segment_all` returns **empty** — the string is not a legal pinyin combination.

The third condition is the anti-pollution rule from design 11.4: `wo` and `emo`
segment into syllables, so they must never enter the abbreviation path. This is the
same predicate the build pipeline uses to reject such keys, so the two agree.

`abbreviation_candidates` looks up the exact key and its prefix completions, tags
them `CandidateSource::Slang`, and `append_abbreviation_group` appends them after
the normal candidates so they do not compete in default ranking.

`CandidateSource::Slang` drives the `[网络]` label through
`candidate_ui::display_main_text`. The label is appended to the main text rather
than occupying its own column, so `row_split`'s width estimate accounts for it
automatically and it cannot overlap the translation column.

### Assembly

`create_engine` reads the config, plans the packs, builds the composite, and mounts
the slang pack separately via `InputEngine::with_slang`. Slang is mounted only when
`slang` appears in the enabled list, so the path is off unless the user asks for it.
Each step logs (`config-warn`, `config-unknown-pack`, `config-missing-pack`,
`pack-skipped`, `pack-ok`, `composite-ok`, `slang-path enabled`) so a VM run can be
diagnosed from the TSF log alone.

`resolve_base_dir` replaces the old single-file `resolve_dictionary_path`. It keeps
the T-022 priority order (env override > DLL directory > `%APPDATA%`) but yields a
*directory*, because composition needs one.

## Alternatives considered

**Merge into a single on-disk pack at install time.** Rejected: it would make pack
enable/disable require a rebuild or re-download, defeat mmap-on-demand for packs a
user never enables, and contradict the independent pack versioning that FR-020
depends on.

**Concatenate entries into one in-memory dictionary at startup.** Rejected: it
abandons mmap and would load every enabled pack's full text into RAM, breaking the
`<= 100 MB` resident-memory acceptance row.

**Always run the merge, with no single-source fast channel.** Rejected on
measurement: it made the base-only configuration four times slower than a single
`DictionaryFile` for no benefit, since a one-source merge is a no-op by definition.

**Put the abbreviation lookup inside `generate_candidates`.** Rejected: that
function's contract is pinyin segmentation, and every caller (CLI `rank`, host-e2e,
tests) would silently acquire slang behaviour. Keeping it a separate function makes
the trigger condition explicit and testable in isolation.

**Make the abbreviation group compete in normal ranking.** Rejected by design 11.4:
a slang word with an inflated build-time frequency would outrank ordinary pinyin
candidates for the same string, which is exactly the pollution the anti-pollution
rule exists to prevent.

**Let the main group win deduplication.** Rejected after the multi-pack regression
caught it: when the slang pack is enabled it also participates in the normal pinyin
path, so `yyds` produced a `Static` candidate before the abbreviation group ran. The
dedup then dropped the slang version and the `[网络]` label disappeared. The
abbreviation group now wins, so the label and the tail placement both hold while the
candidate still appears exactly once.

## Consequences

- Packs become user-controllable through a config file; the enabled set is honoured
  at startup and takes effect on restart (P-12, no hot switching).
- Cost: pack resolution now reads a config file at engine creation. This happens
  once per activation, on the same path that already opens and validates the
  dictionary, so it does not touch the per-keystroke path.
- Cost: the base-only fast channel is a branch whose correctness rests on the claim
  that one source needs no merge. The equivalence test asserts this against a real
  `DictionaryFile` rather than trusting the argument.
- `DictionaryFile` gained `path()` and `entry_count()`. `path` is stored only for
  diagnostics and is `None` for in-memory instances.
- `CandidateSource` gained a `Slang` variant. `display_main_text` is the single
  place that turns it into the visible label.
- Tests: core gains 12 composite, 13 config, and 7 abbreviation tests; `zhu-ye-ime`
  gains engine-level slang tests and three label tests; `host-e2e` gains a
  `--multi-pack` mode whose seven checks include the equivalence floor, the
  no-drift check, dedup, and the pollution guard. `zhu-ye-cli bench` gains a
  `ZYDT_PACKS` multi-pack scenario. Workspace total is 288 tests; the full gate
  (fmt, clippy `-D warnings`, workspace tests, `git diff --check`) is green, and
  host-e2e passes 19/19 seed plus 7/7 multi-pack.
