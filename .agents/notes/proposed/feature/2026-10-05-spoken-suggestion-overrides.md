# Agent Note: spoken suggestion overrides (T-105, T-058 data item)

Status: proposed

[中文](2026-10-05-spoken-suggestion-overrides.zh.md) | English

## Problem

T-058 suggestion words come from the bigram corpus (OPUS GlobalVoices, news
register), so spoken successors like "怎么样/不错" are not in the high-frequency
band ("今天→的/是/早上/我/在"). The user's batch-three list asks to replace the
T-058 **data item** with spoken corpus. Rebuilding the whole bigram from a new
corpus would touch licenses, pipeline, and 23 MB artifacts for one behavioral
goal; a deterministic, reviewable data layer fits the project's static-table
tradition (emoji, units, polyphone patches).

## Proposal

1. **Core static spoken override table** (`zhu-ye-core/spoken.rs`):
   - `SPOKEN_SUGGESTIONS`: 31 high-frequency everyday previous words → spoken
     successors (≤5 words each, "天气→不错/怎么样/很好/挺好/很冷",
     "今天→天气/怎么样/下雨/忙/累", "你→好/在吗/知道/觉得/想", …).
   - `spoken_overrides(previous) -> Option<&'static [&'static str]>`: exact
     previous-word match, no segmentation.
2. **Merge rule** in `suggestion_candidates` (suggestion.rs):
   - Hit: the word zone is fully replaced by the spoken list (truncated at
     `SUGGESTION_WORD_CAP`); the phrase zone still merges bigram phrases
     (`今天早上`/`今天天气` stay useful); total ≤ `SUGGESTION_CAP` unchanged.
   - Miss: word zone keeps the original bigram statistics path, behavior and
     ordering unchanged (zero drift for uncovered previous words).
3. **Guarantees**:
   - Assertions in T-057 (Top1/Top3) unaffected: suggestions still appear only
     in the post-commit idle window; they never join pinyin candidate ranking.
   - Deterministic, offline, no new license obligations (hand-maintained neutral
     everyday words; no third-party content — same stance as abbreviation/fuzzy
     syllable tables). Table size and format are test-enforced.

## Alternatives considered

- **Import a spoken corpus and rebuild bigram artifacts**: rejected — new
  license registration, pipeline changes, and 23 MB artifact rebuild for a
  single behavioral goal; unreviewable bulk data.
- **Score mixing (spoken list ranked against corpus)**: rejected — nonlinear
  ranking changes uncovered-word behavior subtly; full replacement is a clean,
  reviewable contract.
- **Whole-previous-word-only table**: chosen over phrase keys — suggestion
  triggers on one committed word; phrase-y results already come from the bigram
  phrase zone.

## Acceptance criteria

- core: spoken.rs 3 tests (hit / miss / table format + size budget
  30..=512); suggestion.rs +3 (word-zone replacement, phrase-zone merge,
  uncovered stays statistical); workspace all green.
- host-e2e `--m8` real dictionary 8/8: "今天" first suggestion is spoken
  "天气"; continuing suggestion after committing "天气" is non-empty
  (spoken table + bigram phrases); uncovered previous words keep the
  statistics path; lifecycle (letters exit, Esc clears) unchanged.
- e2e.ps1 full regression green; docs updated (需求 §13 note, design §3.1 +
  sample-table note + §5 acceptance); todos T-105; bilingual Agent Note.

## Risks

- **"今天" contract migration**: m8 previously asserted first suggestion "的"
  (corpus); now the spoken table owns "今天" so the assertion moved to "天气".
  This is the intended data replacement; lifecycle assertions unchanged.
- **Table staleness**: 31 previous words is a curated seed; additions are plain
  data edits with the format test guarding budget.
- **Over-replacement risk**: covered previous words lose corpus signals
  entirely. Chosen deliberately — the covered set is small and the spoken
  wording matches the scenario-5 user expectation ("今天天气→怎么样/不错").
