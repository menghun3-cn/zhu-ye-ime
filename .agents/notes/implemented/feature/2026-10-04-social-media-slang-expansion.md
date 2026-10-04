# Agent Note: Slang expansion from social-media-chinese-words (T-087)

Status: implemented

[中文](2026-10-04-social-media-slang-expansion.zh.md) | English

## Problem

FR-047 (M13) expands the internet-slang pack. The slang pack shipped by M6 was
built entirely from a hand-maintained seed table (413 rows, T-045), which
under-covered niche/circle slang despite the corpus
[deferral note](../../implemented/process/2026-09-28-slang-pack-and-content-gate.md)
having already named `social-media-chinese-words` (MIT, ~1.077M lines) as the
future candidate pool. D-52 (user-confirmed 2026-10-02) commits to cleaning a
high-frequency subset of ~10k entries into the slang pack plus a continuously
grown hand seed table. Constraints: fully offline and deterministic, input
SHA-pinned (A-1/NFR-008), reproducible, the abbreviation path (FR-017) and
user-word learning must not regress, and the pack must stay within the same
size/load budget as the existing slang pack.

## Decision

A locked download + deterministic clean + merged build pipeline, with the
*cleaned subset committed to the repo* so every build-slang input stays
repo-resident (the same rule as the T-045 seed table):

- **Fetch** (`scripts/fetch-social-media.ps1`, PS 5.1): downloads the 7
  category txt files (`词 词频` per line), merges them UTF-8 no-BOM with a
  `# CATEGORY:` comment before each category, and fails on SHA-256/size drift
  against pin `data/pins/social-media-zh.json` (D-020; LICENSE/README archived
  as `data/cache/social-license.txt` / `social-readme.txt`). The generic
  `fetch-sources.ps1` is single-URL and does not own the merge step.
- **Clean** (`zhu-ye-dict social-clean`, new module `social.rs`; runs
  `source-check` first): parse `词 词频` → filter (shape = Han + ASCII
  alphanumerics with ≥1 Han char; length 2–12 chars) → blocklist gate
  (identical `Gate` as build-slang) → drop exact duplicates of seed-table
  words (weight already lives in the seed entry; no new entry) → dedup by
  ASCII-lowercased key keeping first occurrence ("same shape, same meaning") →
  stable sort by corpus frequency descending → truncate to 10,000. Since the
  source *has* a frequency column, the §14.4 "frequency or fallback" branch
  resolves to corpus frequency. Output `data/slang/social-words.tsv`
  (D-021, 4 columns, same shape as `seed.tsv`, source column =
  `social-media-chinese-words`), header comment recording rule version "v1"
  and the input SHA; two runs are byte-identical (verified).
- **Build** (`build-slang` in `slang.rs`): `parse_seed` both files, seed rows
  first then social rows, one unified gate + annotation + dedup + v2 compile
  (single frequency 5000 as before). Social rows are all `词` (never
  abbreviations), so the FR-017 abbreviation-key rule is untouched. Missing
  social file degrades to `social_rows = 0` (report field added) instead of
  failing, so old checkouts still build the classic pack.
- **Seed table expansion**: 19 rows appended (2026-10 block) with per-row
  meme-source notes (`维护者整理 2026-10（…）`), e.g. 瑞思拜/泼天的富贵/
  雪糕刺客/已老实/修勾/吗喽/双向奔赴, all previously absent; duplicates were
  checked against the existing 413 rows, characters verified against the
  kTGHZ2013 backplate via the normal annotation path.

**Measured (2026-10-04, release)**: clean input 1,075,792 rows → shape
88,075 / gate 513 / seed-dup 922 / dedup 270,253 → output 10,000; repeated
runs are byte-identical (verified 3×, SHA `65697A83…`); build →
slang.zyct 9,900 entries / 661,803 B (26 KB → 646 KB), gate negative
310/310, false-kill 1/453 (0.2%) unchanged; eval re-run identical to the
T-057 baseline (Top1 84.7% / Top3 97.2% / sentence 21.0%); `source-check`
15/15 sources locked (16 pin JSONs incl. 1 snapshot data file, not a source).

## Alternatives considered

- **Standalone pack (e.g. `slang-social.zyct`)**: rejected — D-46 keeps the
  three-domain-pack preset; slang semantics already overlap, a separate pack
  only adds assembly complexity.
- **build-slang reading the raw cached corpus directly, no committed subset**:
  rejected — build inputs would then live outside the repo, breaking the
  "inputs repo-resident" rule and diff/audit of what actually shipped; the
  intermediate tsv is small (~200 KB) and is the audit artefact.
- **CJK-symbol forms (·、&…), pure English, or length 1 / >12**: rejected —
  §14.4's shape rule is "pure CJK or CJK+alphanumeric"; anything else amplifies
  the known corpus noise (brands, person names, non-Han raw entries).
- **Gate-sample enrichment**: not done — false-kill rate is already 0.2% way
  below the 5% gate; widening samples would shift the gate, needs a separate
  decision, and §14.1.3 only requires "not regress".
- **Keeping traditional/domain-specific forms**: accepted — the CJK range
  includes traditional forms; build-time annotation failure excludes unreadable
  entries anyway (532 social rows), which is the same policy as T-045.

## Consequences

- `build-slang` gains a second input file and a `social_rows` report field;
  `social-clean` adds a new CLI subcommand; `source-check` count 15 → 16.
- Growth is bounded: 5,900% in entries is far larger than base/domain packs in
  proportion, but 646 KB is ~2.2% of base.zyct and load is mmap + layout
  checks, so the "same order of magnitude" budget holds; update size for the
  slang pack grows accordingly (manifest/hash regenerated per release, no
  contract change).
- Quality: the subset is algorithmically extracted (brands/person names/
  traditional variants pass the shape rule by design); rows the annotation
  backplate cannot read are dropped at build time; the blocklist gate runs both
  in the clean step and again during build (belt and braces).
- Future expansion is a three-command pipeline
  (`fetch-social-media.ps1` → `social-clean` → `build-slang`), all hashes pinned.

Relevant notes: [slang pack and content gate (T-045 — the pool was deferred
there and is lifted here; gate mechanics unchanged)](../../implemented/process/2026-09-28-slang-pack-and-content-gate.md),
[composite dictionary and abbreviation path (FR-017 untouched)](../../implemented/architecture/2026-09-29-composite-dictionary-and-abbreviation-path.md),
[en.zyen dual-source (sibling data-pin pattern)](../../implemented/feature/2026-10-04-en-wordbook-zyen-v1.md).
