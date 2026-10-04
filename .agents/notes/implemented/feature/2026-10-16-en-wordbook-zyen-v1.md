# Agent Note: En wordbook file en.zyen (ZYEN v1) for the ECDICT-backed English dictionary (T-085)

Status: implemented

[中文](2026-10-16-en-wordbook-zyen-v1.zh.md) | English

## Problem

FR-046 (M13) requires the full ECDICT dataset (770,611 rows) as the English dictionary
instead of the ~15,534-entry compiled-in static table from [M9's English candidate
layer](../../implemented/feature/2026-10-02-english-candidates-and-email-url-formats.md):
users expect `pytho`→Python, `iphon`→iPhone, `ap`→API/Apple for the whole vocabulary,
not just the top slice. D-51a explicitly rejected frequency truncation ("the goal is
completeness") and turned load time (≤50 ms), query latency (≤0.5 ms median) and
artifact size (≤20 MB budget) into hard acceptance gates. A 770k-entry in-crate Rust
static slice is unmaintainable (tens of MB of source, long compile), and the v2
`base.zyct` dictionary format is organized around pinyin-keyed Chinese entries plus
translation indexes — the wrong shape for rank-ordered English prefix lookup.

## Decision

The English dictionary ships as an independent mmap file `en.zyen` (ZYEN v1), built at
release time, distributed inside the package next to `base.zyct` (T-078), and loaded by
the IME engine as a drop-in upgrade over the static table.

**Data pipeline** (`zhu-ye-dict en-build`, inputs pinned via `data/pins/ecdict.json` +
`data/cache/ecdict-full.csv`, SHA-256 locked):

- Primary: ECDICT rows cleaned to `[A-Za-z'\- ]` with ≥1 letter and length 2..=40
  (source rows carrying "word:gloss" pollution are rejected); 770,611 rows in →
  760,951 kept. Dedupe on lowercase key keeping the max `(frq, bnc, word)` triple.
- Sort/rank key: `(frq>0, frq, bnc)` descending, norm lexicographic for the binary
  search order; rank = index after sorting (smaller = more frequent). `frq` covers
  100% of kept rows, `bnc` 75.4% — no gaps, deterministic.
- Supplement: FrequencyWords (D-018) top 10,000 fills gaps not in ECDICT (79 rows,
  freq used as placeholder rank input); CC-CEDICT English side (D-001) adds pure
  single words (0 rows in practice — ECDICT already covers them).
- Patches: `data/patches/en-capitals.tsv` restores official forms / forces rank
  (92 applied); `data/patches/en-exclude.tsv` removes noise (51). Both were
  previously M9 lists, still authoritative.
- Result: 760,987 entries, final artifact 19,462,460 bytes (18.56 MiB ≤ 20 MB).

**ZYEN v1 binary layout** (fixed, validated on load):

- Header 96 B: magic `"ZYEN"`, version u32 = 1, count u64, reserved u64,
  records_off/pool_off/anchors_off u64, pool_len u64, anchor_stride u32 = 1024,
  anchor_count u32, content_hash [u8;32] = SHA-256 of everything after byte 96.
- Record 11 B per entry: `norm_off u24, word_off u24, norm_len u8, word_len u8,
  rank u24`; `word_len == 0` reuses the norm pool span (display form == norm).
  Records are byte-ascending by norm (binary search depends on it; compile asserts
  ASCII lowercase, non-empty, ≤255).
- Pool: one UTF-8 span holding all norm/word texts (deduplicated when equal).
- Anchors: one 12-B bucket (`key_pfx [u8;8]`, `idx u32`) every 1024 records for
  short-prefix queries without scanning from the head.
- Query: lowercase input → bucket start by binary search on `key_pfx` → record
  binary search for first norm ≥ prefix → scan prefix run → keep top `limit` ranks
  with a max-heap (avoids full sort of 100k+ hits for "a"/"b"/"c"); same result
  semantics as the M9 table (`sort_by rank → truncate`).

**Loading** (`crates/zhu-ye-core/src/en_lexicon.rs`):

- `EnLexicon::open/from_bytes`: mmap (or heap) + full SHA-256 of the body +
  `validate_layout` (bounds, ASCII-lowercase norms, strict ascending, rank < count,
  anchor monotonicity and bucket-head agreement). Layout validation runs in
  `std::thread::scope` chunks (no rayon, no Windows API — core stays portable); the
  cross-chunk ascending chain is re-checked sequentially.
- Resolution order at engine start (`tsf.rs`): `en.zyen` next to the DLL, else
  `%APPDATA%\ai-zhu-ye-ime\en.zyen`; load failure logs via `debug_log` and falls
  back to the M9 static table; neither present → `None` silently (FR-030 behaves
  exactly as before T-085).
- `input.rs` FR-030 branch: `en_word_candidates_from(lexicon, ...)` when the lexicon
  is loaded, `en_word_candidates(...)` (static table) otherwise — same mapping
  (`score = -(rank as i64)`, `pinyin = None`, appended after main candidates,
  capped like the static table).

**Tooling**: `zhu-ye-dict` subcommands `en-build` / `en-bench` / `en-inspect`;
`scripts/build-en-wordbook.ps1` (PS 5.1) verifies the cached CSV against the pin,
builds, benchmarks, and fails the run when load > 50 ms, median query > 500 µs, or
artifact > 20 MB. `package-portable.ps1` / `install.ps1` copy `bin/en.zyen` next to
the DLL; a missing file degrades to the static-table fallback instead of failing
installation.

**Measured (2026-10-16, release build)**: load+verify 28.08 ms; 200,000-query
benchmark median 43.2 µs, P99 2.2 ms; artifact 19,462,460 B; checksum stable.

## Alternatives considered

- **Grow the in-crate static table to 770k**: tens of MB of generated Rust source —
  unmaintainable compile cost and table bloat; rejected.
- **Extend the v2 `.zyct` format with an English section**: couples English prefix
  lookup to the pinyin-keyed Chinese layout and translation indexes; versioning and
  per-frequency updates would drag the base dictionary along. Rejected in favor of a
  standalone file (independent mmap pages, per-word frequency updates, version
  decoupled from `base.zyct`).
- **Frequency truncation to 100–200k**: rejected by D-51a — the goal is the full
  dataset; performance became hard acceptance gates instead.
- **No integrity check at load (trust the package manifest)**: the distribution
  manifest already hashes the artifact, but a full SHA-256 on load costs only
  ~12.7 ms (hardware-accelerated sha2) and catches tampering/corruption before any
  query; kept, parallelized where possible.
- **Full sort of the hit interval**: 100k+ hits for short prefixes made P99
  7.9 ms; a max-heap keeping `limit` smallest ranks (limit is typically ≤ 6) brings
  the median to 43 µs while preserving `sort_by rank → truncate` semantics.
- **A CSV crate for parsing ECDICT**: kept a hand-written parser (quote fields,
  `""` escape pairs, trailing-empty-field rule for the frequently empty `audio`
  column) — no new runtime dependency, matching the project's self-reliance stance.

## Consequences

- `en_words.rs` (15,534 entries) stays as the fallback when `en.zyen` is missing or
  corrupt; both paths share the same public semantics, so FR-030 behavior is
  unchanged for any deployment state.
- Artifact is built at release time (source tree stays free of large binaries);
  `data/cache/ecdict-full.csv` is gitignored, `data/pins/ecdict.json` is committed.
- `EN_WORDBOOK_FILE_NAME` is maintained in two places (`identity.rs` + 
  `scripts/ime-identity.ps1`), following the existing D-42 dual-maintenance pattern;
  `verify-tsf-identity.ps1` does not cover it (file name is not a registry identity).
- Words differing in case keep their display form as a separate pool span (86,783
  capitalized entries), costing ~10.9 MB of pool — the price for exact case-form
  display (D-09).
- Packaged deployments silently degrade to the smaller static table when the file is
  absent; the acceptance gates (≤50 ms load, ≤0.5 ms median query, ≤20 MB) are
  measured on the file-backed path.
