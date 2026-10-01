# Agent Note: Domain-boost candidate layer (scenario 8, M10)

Status: implemented

[中文](2026-10-02-domain-boost-scenario8.zh.md) | English

## Problem

Scenario 8 (automatic domain suggestions, FR-033/FR-034/FR-035) needs activated
domain packs (P-12: only packs the user enabled contribute) to promote their
whole-word entries when the composition matches exactly. The confirmed
restrictions are: promotion only for full-word hits (D-15), only domain
candidates take effect — user words, associative memory, and other layers are
untouched (D-17), multiple packs pick the first by package id in lexicographic
order (D-16), the insertion point sits after the base candidate list and before
the appended groups (D-13), and the switch `enable_domain_boost` defaults to on
(D-14).

The ranking baseline (T-050) must not drift: when no pack hits, the candidate
list must be bit-for-bit identical to the pre-scenario-8 build.

## Decision

### Core: `zhu-ye-core/src/domain_boost.rs`

`domain_boost_candidates(packs, composing) -> Option<Vec<Candidate>>` queries
each pack in caller-supplied order (the assembler guarantees id-ascending, D-16)
with a full-word `lookup(composing)`; the first non-empty pack wins (a prefix
naturally misses because `lookup` is exact). The returned candidates carry
`source = Domain`, keep the entry's pinyin/translation, take
`score = i64::try_from(frequency).unwrap_or(i64::MAX)`, and the group is sorted
by frequency descending through the shared `CandidateSorter`. `DOMAIN_BOOST_CAP`
(8) caps the group. No hit / empty composing / unrelated string returns `None`.

### Config

`ConfigFile.enable_domain_boost: bool` with `#[serde(default = "default_domain_boost")]`
(default `true`, D-14) is persisted to `config.json`. `PackPlan` gains
`pack_ids: Vec<String>` — a per-entry id paired 1:1 with `packs` (configuration
order), which is what the assembler sorts.

### Engine (`crates/zhu-ye-ime/src/input.rs`)

`InputEngine` gains `domain_packs: Vec<(String, Arc<dyn Dictionary>)>` (only
enabled packs, id-ascending at assembly time) and `enable_domain_boost: bool`
(default true), set via `with_domain_packs` / `with_domain_boost`. In
`refresh_candidates`, right after `self.candidates = main;` and before the
FR-030 appended groups (English / abbreviation / emoji), the boost step builds a
borrowed `Vec<(String, &dyn Dictionary)>` and appends the domain group through
`append_group(main, boosted)` (D-13).

The TSF layer (`crates/zhu-ye-ime/src/tsf.rs`) gets `domain_engine(...)`, which
reads `plan.pack_ids.zip(plan.packs)`, sorts by id ascending, opens each
`DictionaryFile` (a failed open is silently skipped — the composite stage already
logged `pack-skipped`), logs a `debug_log` line with the enabled pack ids, and
attaches them. **No TSF interaction code changes**: boosting only affects
candidate content and order.

### Append-group de-duplication is the dominant real-world behavior

`append_group(main, extra)` keeps the earlier (main) candidate when texts match,
so a domain entry whose text already exists in the base path is absorbed and
produces no visible change. Real base is a 287k-entry general dictionary: it
covers almost every single-syllable character and a vast number of multi-word
combinations, so most domain-pack entries collide. The visible wins are domain
entries whose exact string the base path cannot produce (typically niche
technical transliterations). This is by design — T-050 never drifts because the
boost can only *add* non-colliding text or nothing.

Because of this, `host-e2e --m11` asserts with an **in-memory controllable
domain table** (deterministic hits/misses, prefix, switch-off, D-16 lexicographic
choice, base<domain<emoji ordering) and, when given a real dictionary path,
re-checks "no-hit ⇒ bit-identical" in the real 287k-entry context.

## Alternatives considered

- **Fused scoring into the main sorter**: rejected — would jitter positions and
  break the T-050 baseline; appending as a group keeps base ordering untouched.
- **Domain entries merged into the base dictionary**: rejected — packs must stay
  separable (enable/disable, replaceable), and merging would pollute the general
  dictionary with domain terms.
- **TSF-layer key interception for domains**: rejected — no key semantics differ,
  so the layer stays zero-change (same principle as scenarios 6/7).
- **Score-relaxed match (prefix/lower frequency)**: rejected — D-15 mandates
  full-word hits only; prefix promotion would displace ordinary pinyin input.

## Consequences

- On the real base the boost is quiet: most domain hits collide and are
  de-duplicated, which is exactly the no-drift guarantee; a handful of
  non-colliding technical entries become visible.
- `enable_domain_boost` defaults on; users who prefer the pure baseline can turn
  it off via `config.json` (D-14/D-17), restoring append semantics for the packs.
- Determinism: same input, same pack set ⇒ same candidate list (FR-002); pack
  choice is id-lexicographic, intra-group order is frequency descending.
- Domain candidates carry `source = Domain`; the UI does not add a label for it
  (no UI change beyond the switch already planned).
- New acceptance command `host-e2e --m11 [<dictionary.zyct>]`; T-057 eval
  baseline verified unchanged after this batch (Top1 84.7% / Top3 97.2% /
  sentence 21.0%).

## Related notes

- [2026-09-30-format-symbol-emoji-candidates](2026-09-30-format-symbol-emoji-candidates.md)
  and
  [2026-10-02-english-candidates-and-email-url-formats](2026-10-02-english-candidates-and-email-url-formats.md)
  define the appended groups that follow the domain group (D-13 order).
