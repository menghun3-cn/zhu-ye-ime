# Agent Note: Adjacent-transposition fuzzy correction and candidate pinyin display

Status: implemented

## Problem

On 2026-10-06 the user reported that when typing fast, adjacent characters swap
(`xiangzhe` becomes `xiagnzhe`, `zhegnq` should read `zhengq`, `shegnc` should
read `shengc`, `zhagnh` intends `zhangh` — all `gn`↔`ng` / `agn`↔`ang` style
transpositions), and the IME should still recognize the intended expression and
show the **corrected pinyin** next to each candidate (e.g. candidate `生`
labeled `sheng`, `正确` labeled `zhengque`, `账` labeled `zhang`).

The pre-existing fuzzy path (`corrected_candidates`, M7/FR-024) only handled
**fully segmentable** inputs; a transposed string like `zhegnq` is not
segmentable at all, so it never reached any correction path.

## Decision

`zhu_ye_core::corrected_candidates` now also handles **unsegmentable** input
(FR-069): when the input has no whole-word hit and cannot be fully segmented,
enumerate **one adjacent-character transposition** (`transposition_variants`,
ASCII-lowercase only, deduplicated) and run each variant through three routes in
order:

1. **Whole-word hit** (`lookup` non-empty) → emit directly;
2. **Fully segmentable** variant → the regular candidate pipeline
   (`generate_candidates`; `xiagnzhe`→`xiangzhe`→想着);
3. **Prefix path** → `generate_prefix_candidates` (completed + completion
   groups; `zhegnq`→`zhengq`→`zhengque`→正确, `shegnc`→`shengc`→
   `shengcheng`→生成, `zhagnh`→`zhangh`→`zhanghu`→账户).

All emitted candidates carry `CandidateSource::Corrected` (the existing
correction group, gated by `enable_fuzzy` per O-05/T-103) and their `pinyin` is
set to the **corrected** pinyin. Deduplicated by text, capped by
`CORRECTION_VARIANT_CAP` (24).

Candidate-window display (FR-069 second half): `CandidateUiItem` gains a
`pinyin` field (sourced from `Candidate.pinyin`), and
`display_main_text` renders Chinese candidates as `中文（拼音）`（`生（sheng）`、
`正确（zhengque）`). This applies to every candidate that carries pinyin — not
only corrected ones. Slang items keep the `[网络]` label without pinyin,
pinyin-less candidates (emoji, English forms) render unchanged, and the
translation layer still shows the translation as the main text. The demo data
(`candidate-demo`) and e2e assertions were extended accordingly.

## Alternatives considered

- **Iterate multiple transpositions / full edit distance.** Rejected: one
  transposition covers the reported `gn↔ng`/`agn↔ang` class; deeper edits
  explode the search space and raise false-positive risk. The mechanism is
  deliberately bounded; a dictionary-backed collocation is the long-term layer
  for whole-word hits.
- **Fuzzy replacement of syllables (existing path) only.** Rejected: it cannot
  reach unsegmentable strings; the whole point is strings that fail
  segmentation.
- **Pinyin in a separate aligned column.** Rejected for now: the `（拼音）`
  inline form matches the user's example (`1 生（sheng）`) and avoids another
  layout axis before the T-116 word-class/alignment layout lands; the inline
  form composes into the existing row-width estimation unchanged.
- **Show pinyin only on corrected candidates.** Rejected: the user asked for
  pinyin on candidates generally (`显示正确的拼音`), and whole-word/prefix
  candidates also benefit.

## Related

- [Prefix candidates for incomplete segmentation](../../../implemented/feature/2026-09-24-prefix-candidates-incomplete-segmentation.md) — route 3 reuses `generate_prefix_candidates` (completions + completed).
- [Full pinyin segmentation core](../../../implemented/feature/2026-09-19-full-pinyin-segmentation-core.md) — `segment_all`, the segmentability gate for routes 2 vs 3.

## Consequences

- The correction group now covers unsegmentable inputs at the cost of ≤
  `len-1` dictionary lookups per refresh (bounded, offline, deterministic).
- Candidates get visually wider (`中文（拼音）`); the dynamic column split
  (T-037) absorbs this, and demo/e2e data assert the new main text.
- Behavior change is user-visible and spec'd as FR-069 (§22, D-84); gated by
  the same `enable_fuzzy` switch as M7 fuzzy correction, so users can turn it
  off.
