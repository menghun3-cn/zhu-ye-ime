# Agent Note: Mixed pinyin-English whole-sentence decoding `python代码` (T-086)

Status: implemented

[中文](2026-10-04-mixed-pinyin-english-decode.zh.md) | English

## Problem

FR-050 (M13) covers scenario 6 "mixed strings" in which users type code words,
abbreviations or product names together with pinyin in one composition, e.g.
`python代码`/`API接口`/`iPhone价格`. On the TSF path the composition is always
pure ASCII (`pythondaima`, `APIjiekou`), because only letter/digit/format keys
enter the composition; the Chinese in the acceptance examples is the *decoded
shape*, not the input. Existing exclusive paths (full pinyin T-007, abbreviations
FR-017, pure-English FR-030, prefix candidates T-029) each assume the whole
composition belongs to one layer, so `pythondaima` today degrades to prefix
candidates around `py…`. At the same time the discriminator must not hijack
`nihao` (pure pinyin), `yyds` (abbreviation), `python` (pure English) or
`xie`-style double-segmentable strings (risk table §14.8).

## Decision

A deterministic, offline, statistic-free gate plus decoder in
`crates/zhu-ye-core/src/mixed.rs`, wired into `input.rs refresh_candidates`
*after* the `detect_format` (FR-031 `@`/`www.`/http) return and *before*
`generate_prefix_candidates` (T-029). D-56 holds: no statistical or language
model.

**Trigger** (`is_mixed_input`, pure function over segments = `[A-Za-z]+` runs
and other runs):

- English/abbreviation component: a letter run of length ≥ 2 that contains an
  uppercase letter (cased; `API`/`iPhone`), or is fully non-segmentable and
  matches a complete English word (`en_is_word`: static-table prefix-query top
  hit lowercased == probe; `python`).
- Pinyin component: a non-ASCII segment (Chinese body) or a letter run that
  fully segments (`daima`).
- Same-run mix: `en_prefix_then_pinyin` — a longest English-word prefix followed
  by a segmentable suffix (`pythondaima`).
- Fires iff `strong_mixed || (pinyin_body && (cased || english)) ||
  (cased && pinyin_letters)`. Pure `nihao`, `yyds`, `python`, `代码`, `xie`,
  `xiedaima`, `python123` (digits are not a pinyin segment) and
  lowercase-segmentable `api接口` do not fire; `daimapython` (pinyin before
  English in the same run) is an accepted non-goal (§14.3.5 + risk table).

**Decode** (`mixed_candidates`, per letter run, in order): ① whole run is a
complete English word → that word (canonical case form); ② longest English-word
prefix + segmentable suffix → English candidate + pinyin top-1 (`generate_candidates`,
T-007); ③ longest pinyin prefix (optionally followed by a complete English word);
④ English-word prefix + recursive decode of the remainder (`pythonxyz` →
`python` + `xyz` literal); ⑤ literal fallback — a run never loses input text.
"Hit" always means *complete English word*: the §14.3.2 "drop one prefix char
and retry" loop targets words, so `pyx` does not fall back to non-word prefix
`py`. English candidates come from `en.zyen` when loaded (T-085) else the M9
static table (same dual-source rule as FR-030).

**Output**: `[whole-sentence candidate] + per-segment best candidates`, in
segment order. The whole-sentence candidate is the *concatenation of the best
segment decodes* (`pythondaima` → `python代码`; `APIjiekou` → `API接口`) with
`source = Mixed`, `score = i64::MAX`, first; segment candidates follow
(`python` / `代码`), each segment capped at its top-1 ("best-combination",
§14.3.3). Deterministic: no randomness, no model, same input → same output.

**Engine integration** (`input.rs`): the mixed branch clears
`cached_translation_candidates` / `selected_on_page`, clamps the page and
returns, mirroring the format branch; committing selects by candidate `text`,
so an unhit whole-sentence candidate types exactly what was typed. FR-031 keeps
priority because `detect_format` returns first.

**Tooling**: `zhu-ye-dict mixed-bench` (22 samples × 100k iterations, warmup
excluded) measured 2026-10-04 (release): median 3.9 µs, P99 40.7 µs, max
329.7 µs — acceptance gate ≤1 ms (§14.2), filled back.

## Alternatives considered

- **Treat any letter run containing uppercase as fully English**: loses the
  pinyin suffix (`nihaoAPI` → `API` only) and mis-decodes `XPdiannao`-style
  runs; rejected in favor of the ordered ①–⑤ strategy.
- **Symmetrically support pinyin-before-English (`daimapython`)**: doubles the
  search space and is not in the acceptance set; documented as a non-goal.
- **Multi-candidate English segments per run**: §14.3.3 explicitly says "the
  best-combination of each segment" (one top candidate per segment); top-1 keeps
  the candidate row quiet and deterministic.
- **Statistical/language-model reordering of the whole sentence**: rejected by
  D-56 (offline, deterministic, no model); order is segment order.
- **Non-ASCII trigger input**: accepted at the core level (the Chinese body is a
  pinyin segment) but unreachable from TSF; the ASCII form is the real path.

## Consequences

- `CandidateSource` gains a `Mixed` variant; no exhaustive matches exist in the
  codebase (only `==` comparisons in host-e2e), so the UI renders candidates in
  vector order with no new label.
- Case matters by design: `api接口` (lowercase, segmentable) stays on the normal
  path while `API接口` triggers; this asymmetry is documented in the acceptance
  tests and the §14.3 risk table.
- `daimapython` and `pythonxyz`-style tails resolve via literal fallback rather
  than being dropped; input is never lost, but those forms do not produce a
  joined Chinese segment.
- The T-029 prefix path and all other layers are untouched for non-firing input
  (eval re-run unchanged: Top1 84.7% / Top3 97.2% / sentence 21.0%,
  2026-10-04).
- Property tests for determinism/full-coverage of segments are deferred to
  T-089 (proptest), cross-linked below.

Relevant notes: [M9 English candidates + email/URL formats (FR-031 priority
and static-table fallback)](../../implemented/feature/2026-10-02-english-candidates-and-email-url-formats.md),
[T-085 en.zyen file (dual source)](../../implemented/feature/2026-10-04-en-wordbook-zyen-v1.md),
[full pinyin segmentation (T-007, reused for pinyin tails)](../../implemented/feature/2026-09-19-full-pinyin-segmentation-core.md),
[prefix candidates (T-029, ordered after the mixed branch)](../../implemented/feature/2026-09-24-prefix-candidates-incomplete-segmentation.md).
