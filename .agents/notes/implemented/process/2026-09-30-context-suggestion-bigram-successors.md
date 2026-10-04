# Agent Note: context suggestion via bigram successors

Status: implemented

[中文](2026-09-30-context-suggestion-bigram-successors.zh.md) | English

Related notes: [hit-rate-eval-baseline](../process/2026-09-30-hit-rate-eval-baseline.md)
(the accuracy baseline the user's four enhancements must hold against; this note
adds the context-completion capability that enhancement #2 needs) and
[ime-experience-optimization](../process/2026-10-02-ime-experience-optimization.md)
(the M7 feature family this joins: interaction semantics follow the same
candidate-window conventions).

## Problem

User scenario #5 ("context completion"): after typing "今天天气", the candidate
window should offer "怎么样 / 不错 / 很好" — the engine should look at the
preceding committed text, not just the current pinyin in isolation. The engine
already had `frequency(previous, word)` point queries for context-aware ranking,
but there was no way to ask "what are the frequent successors of this word" for
an idle candidate window, and any naive implementation ("enumerate the whole
bigram table") would have needed a new dictionary format or section.

## Decision

Add read-only successor lookup, zero dictionary format change:

1. **`BigramModel::successors(previous, limit) -> Vec<(String, u64)>`** — trait
   method with a **default empty implementation**: models without a successor
   index degrade to "no suggestion" without touching existing ranking semantics.
   `InMemoryBigramModel` returns top-`limit` by frequency desc, tie-break by
   word form; `EmptyBigramModel` stays empty.
2. **`DictionaryFile` uses the existing bigram table layout** — the table is
   built sorted by `(previous, word, frequency)` (see `dict_builder`), so all
   records for one previous word are contiguous. `successors` binary-searches
   the lower bound of `(previous, "")`, scans the contiguous run, sorts by
   frequency desc, truncates to `limit`. No byte-layout change, no new section,
   backward compatible with every existing `.zyct` file.
3. **`CompositeDictionary` merges per-source successors** taking `max` per word
   (same policy as `frequency`), then truncates — pack aggregation stays
   consistent.
4. **`suggestion_candidates(model, previous) -> Vec<String>`** — the user-facing
   list: Top 5 successor words plus up to 3 two-word phrases of the form
   **previous + successor** (明天见 → 明天见你), phrases after words, total ≤ 8;
   empty previous → empty list (suggestion only fires right after a word was
   committed). Phrases with successor == previous (no information) are skipped.
5. **CLI handle**: `zhu-ye-cli suggest <dict> <previous>` uses the exact same
   function the engine will call (T-059), giving a debug/VM probe now.

### Verified behavior on the real dictionary (2026-09-30 build)

| previous | successors (top 5) | phrases |
| --- | --- | --- |
| 明天 | 的 / 早上 / 会 / 我们 / 又 | 明天的 / 明天早上 / 明天会 |
| 天气 | 型态 / 很 / 也 / 变得 / 里 | 天气型态 / 天气很 / 天气也 |
| 今天 | 的 / 是 / 早上 / 我 / 在 | 今天的 / 今天是 / 今天早上 |

The mechanism works; the *register* of the results is bounded by the bigram
corpus (GlobalVoices news), so colloquial successors like 怎么样/不错 are not in
the high-frequency band. That is a data property, not a mechanism defect —
swapping in a spoken-register corpus is a data item, tracked in the T-059 work.

## Alternatives considered

- **Add a dedicated "successor index" section to the dictionary format.**
  Rejected: the contiguous-run property of the existing sorted bigram table
  makes a section unnecessary; the section would duplicate data and force a
  format bump plus a rebuild of every pack. The scan is not a hot path (idle
  window only) so O(run length) is fine.
- **Reuse the existing `RankingContext` previous-word ranking only.** Rejected:
  ranking influences the order of *pinyin candidates while typing*; it cannot
  produce an idle-window list of successors, which is exactly what the user
  picked (idle candidate window after commit).
- **Phrase = successor + its best successor** (明天见 → 你吧). Rejected during
  review: the user's example is explicitly previous + successor (明天见 → 明天见你),
  so phrases concatenate the committed word with the next word.
- **Commit suggestion data as a precomputed static list.** Rejected: it would
  duplicate bigram state, drift from the dictionary, and needs the same
  traversal anyway.

## Consequences

- `zhu-ye-core` tests 139 green (bigram +2: successor order/empty; suggestion
  +4: word order, empty guards, phrase composition incl. same-form skip; loader
  regression assertions inside the roundtrip test). `zhu-ye-cli` builds with the
  new command and its usage line.
- Determinism: successor lists come straight from the (deterministic) bigram
  table — same dictionary, same list; the engine may cache the idle window but
  must recompute after each commit.
- The baseline contract from hit-rate-eval-baseline is **untouched**: suggestion
  only appears when the pinyin is empty, so it never participates in candidate
  ranking; future ranking changes must still not regress Top1/Top3.
- T-059 (engine integration, not in this change): idle candidate window state,
  commit-from-window, exit on any input, host-e2e `--m8` assertions, VM UI
  acceptance.

## Supersession check

No active note is superseded: this note adds a retrieval capability on top of
the bigram table; hit-rate-eval-baseline (accuracy), ime-experience-optimization
(M7 semantics) and cli-benchmark-metrics-and-acceptance (latency) stay active
and are cross-linked above.
