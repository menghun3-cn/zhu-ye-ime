# Agent Note: Dynamic word-compose candidates

Status: implemented

## Problem

On 2026-10-06 the user reported that typing `niyou` only suggests `昵友`/`腻友`/
`拟游隼` (low-utility matches) and never `你有`. Measurements showed neither
CC-CEDICT nor jieba contain the word 你有 (`你` r/234587, `有` v/423765), and the
existing candidate path had no fallback that combines single syllables when a
whole-word hit is short of a full page — the syllable-combination branch in
`generate_candidates` only runs when there is **no** direct whole-word hit at all.
The user confirmed the recommended "mechanism + data" dual track.

## Decision

When a whole-word hit exists but is **shorter than one page** (fewer than
`CANDIDATE_PAGE_SIZE`, including the zero-hit case) and the composing string is a
**complete syllabified string** (`segment_all` yields exactly one segmentation),
`zhu_ye_core::dynamic_compose_candidates` composes a single candidate by greedy
**longest-word matching from the head**: each segment prefers the longest word
available in the dictionary (multi-syllable words beat concatenated single
syllables), picks the highest-frequency entry per pinyin key, and the whole input
itself is skipped (direct whole-word hits belong to the main candidate group).
The resulting text (e.g. `你`+`有` → `你有`, `我们`+`的` → `我们的`) is appended
as one `CandidateSource::DynamicCompose` candidate after the main group (same
insertion point semantics as the FR-058/FR-059 expand group), deduplicated
against earlier texts, and capped to fill the page.

Not produced: non-complete strings (still typable, e.g. `ni`/`nish`), ambiguous
segmentations (`xian` → `xian` or `xi`+`an` — never guess), fewer than two
syllables, or any segment with no word. DynamicCompose candidates do not
participate in domain/contact boosting or user-word learning.

Data track: a few hundred high-frequency spoken collocations (`你有`/`你去`/
`我来`…) will be added to the dictionary via the T-115 rebuild pipeline so common
collocations become whole-word hits long-term; the mechanism is the immediate
fallback.

## Alternatives considered

- **Collocation table only.** Rejected: it cannot cover the arbitrarily many
  common combinations; the mechanism is the guarantee and the table is the
  long-term stability layer.
- **Surface the combination ahead of whole-word hits (position 1).** Rejected:
  D-83 pins the position after the whole-word group so existing short-hit
  candidates keep their rank; user-visible utility was the presence, not the
  order.
- **Statistical / model-based inlining (n-gram, learned pair scoring).** Rejected:
  the hot path must stay offline and deterministic; the dictionary-backed greedy
  match is cheap and reproducible.
- **Hard-coded exceptions for known pairs.** Rejected: no generalization; the
  dictionary-backed rule covers every complete string.

## Related

- [Prefix word expansion](../../../implemented/feature/2026-10-04-prefix-word-expansion.md) — the FR-058/059 expand group that shares this insertion point (after the whole-word group, before boosting).

## Consequences

- New `CandidateSource::DynamicCompose` (deduplicated, not boosted, not learned);
  `generate_candidates` behavior is unchanged.
- Per refreshing: one `segment_all` plus up to `syllables.len()` dictionary
  lookups — bounded, offline, deterministic.
- `niyou` now yields `你有` after the whole-word group; covered by unit tests in
  core and an engine-level test; spec FR-068 (§22).
