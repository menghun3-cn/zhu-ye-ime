# Agent Note: polyphone gap patch — `谁` shui, `熟` shou, and the reverse-lookup boundary fix

Status: implemented

[中文](2026-09-30-polyphone-gap-patch.zh.md) | English

## Problem

User report: typing `shui` produced no `谁` in the candidate window. `rank
data/artifacts/real.zyct shui` returned 说 413852 / 睡 17644 / 水 10119 / 税 414
… but no 谁, while `dict … shei` showed 谁 at frequency 127180. Root cause:
CC-CEDICT's line `誰 谁 [shei2] /who/also pr. [shui2]/` only carries the `shei`
reading in its pinyin field; the import pipeline never saw `shui`, so the real
dictionary had no (谁, shui) entry and 谁 was unreachable under `shui`.

A second latent defect surfaced while reproducing this: `dict real.zyct -r 谁`
(a Chinese key whose UTF-8 bytes sort after every English reverse key) panicked
with an out-of-range slice in `dict_loader::find_en_to_zh` — the binary search
can end with `low == count`, and the code sliced before checking the boundary.

## Decision

**Build-time patches, no engine special-casing, additive only** (consistent with
the M7 constraint set: deterministic, offline, self-maintained, must not pollute
the existing ranking path):

1. **Authoritative reading source — kTGHZ2013** (`data/cache/kTGHZ2013.txt`,
   D-014, already pinned): the official 通用规范汉字表 readings in
   `U+8C01: shéi,shuí  # 谁` form — precomposed tone letters and accent-free
   characters, one per line. Parsed by a new `zhu-ye-dict::polyphone` module:
   `load_standard_readings` splits on `#`/`,` and normalizes with the same
   conventions as the CEDICT side (precomposed tone letter → base letter,
   `ü`/`nǚ` → `v`, combining U+0300–U+030F stripped), so
   `strip_tone_letters("shuí") == normalize_pinyin("shui2")`.
2. **`polyphone_gaps` audit**: compare the CEDICT single-character reading set
   (same split/normalize pipeline as the builder) against the standard table.
   Full run: 10,789 CEDICT single chars vs 8,105 standard chars → 144 missing
   readings; filtered at frequency ≥ 50,000 only two remain: `嗯→ng` (nasal,
   not a legal syllable in `SyllableTable`, rejected by design) and `谁→shui`.
   The rest are low-frequency variant readings (熟→shou at 1619 is real and
   included; 嘘→shi, 臂→bei, 巷→hang, 杉→sha … recorded in the audit report,
   deferred for later triage).
3. **Patch table `data/patches/polyphone.tsv`** — human-curated TSV,
   `字<TAB>读音<TAB>备注`, `#` comments; only 谁→shui and 熟→shou this round.
   `load_patch_table` rejects non-standard syllables at parse time with
   per-line reasons (a `ng` line fails loading, which is a deliberate guard).
4. **`import --polyphone`** (defaults to the repo patch table when it exists):
   applied after truncation, per entry — syllable must pass
   `SyllableTable::standard().is_complete_syllable`; the character must already
   exist in the vocabulary (no introducing out-of-table glyphs); a matching
   (字, 读音) entry already present is skipped; duplicates inside the table are
   skipped; frequency inherits the character's current highest frequency;
   existing entries are never modified (reinforce-don't-pollute).
5. **`audit-polyphone <CC-CEDICT> <kTGHZ> [--freq] [--min-freq]` CLI command**:
   re-runs the whole-word comparison (the reproducible evidence trail behind any
   future patch batch).

The reverse-lookup fix is a one-line boundary check (return `None` when
`low >= count`) plus a regression test (Chinese keys and over-range keys return
`None` without panicking).

## Alternatives considered

- **Engine special-case `shui`→谁.** Rejected: fixes one symptom, bypasses the
  ranking pipeline's determinism and pollutes the hot path; any other missing
  reading would need its own hack.
- **Maintain a private fork/variant of CC-CEDICT with the readings amended.**
  Rejected: D-001 is hash-pinned (NFR-007/008); a private variant would fork
  the source of truth, break reproducible imports, and bury the amendment in a
  giant file instead of a reviewable diff.
- **Auto-apply every gap from the standard table.** Rejected: the 144 gaps
  include nasals outside the syllable table (`嗯→ng`), low-frequency variants,
  and formal-register distinctions; automatic application would inject
  unreviewed glyphs/readings and silently desync from the engine's segmentable
  set. A human-curated TSV plus the `audit-polyphone` report keeps every patch
  enumerable and reproducible.
- **Store patched readings with tones.** Rejected: the whole dictionary key
  space is tone-less pinyin (same normalization as CEDICT); tone-marked patch
  keys could not match engine queries.

## Consequences

- `real.zyct` rebuild: 120,028 → 120,030 entries (`多音补丁: 读取 2 条，应用 2`),
  content SHA-256 changes; `rank shui` → 说 413852 first, **谁 127180 second
  (first screen)**, 睡/水 follow; `rank shei` still yields 谁 (original reading
  untouched); `rank shou` yields 熟 at position 4. Both readings of 谁 now
  share the same inherited frequency, so ordering stays deterministic.
- Patch words carry no translation (补丁 entries have empty 译文); the TSF
  candidate window never shows translations, and the CLI `rank` output is a
  dev-only view — acceptable.
- The 嗯→ng reading is deliberately not patched: `ng` is absent from the
  standard syllable table, the engine cannot segment it, and a patch would
  silently desync. Recorded in the audit output instead.
- This mechanism **adds** readings on top of the import pipeline's
  "keep the first reading" rule (see
  [real dictionary import](../../architecture/2026-09-21-real-dictionary-import.md));
  it never replaces or reorders existing entries, so both rules coexist.
- host-e2e `--m7` gains four assertions (shui→谁, shei still→谁, shou→熟,
  reverse-lookup Chinese key no panic): 26/26. Workspace tests 363 green
  (core +1, dict +9); fmt/clippy/diff-check clean.
- Verification on the acceptance VM (real TSF stack, fresh `tsf-m8` directory
  to dodge the dictionary mmap lock): shui candidate window contains 谁 and
  digit 2 commits it; shei still shows 谁; shou shows 熟. Evidence in
  `target/t056-accept-deploy` (not committed).

- **T-129 extension (2026-10-08)**: the same patch table now also applies to the
  shipped base chain (build_base) via the shared \pply_patch_entries\ codepath, and the
  table grew to 31 readings. See [polyphone-patch-into-base-pack](2026-10-08-polyphone-patch-into-base-pack.md).
