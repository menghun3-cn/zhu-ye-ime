# Agent Note: context suggestion engine integration

Status: implemented

[中文](2026-09-30-context-suggestion-engine-integration.zh.md) | English

Related notes:
[context-suggestion-bigram-successors](../process/2026-09-30-context-suggestion-bigram-successors.md)
(the retrieval layer this note's engine state consumes: same `suggestion_candidates`
contract, same list shape — words first, phrases after) and
[ime-experience-optimization](../process/2026-10-02-ime-experience-optimization.md)
(the M7 feature family; interaction keeps the candidate-window conventions).

## Problem

The retrieval layer (T-058) can produce successor suggestions, but the engine and
TSF had no *idle candidate window* state: the window is shown only while the
composition string is non-empty and is hidden the moment a commit empties it.
The user scenario #5 (context completion) requires: after committing a word, the
candidate window stays on screen showing the bigram successors; pressing a digit
commits the picked suggestion directly; typing any letter leaves the suggestion
state and returns to the normal pinyin path.

## Decision

Add a `suggestion` state inside `InputEngine` and let the existing window/path
machinery render it as a normal (empty-composition) view:

1. **Engine state**: `InputEngine.suggestion: Vec<String>`. `commit_candidate`
   refreshes it right after updating `previous_word` — every commit recomputes
   (the "must recompute after each commit" requirement from the retrieval note).
   `commit_raw` and `handle_enter` clear it (`previous_word = None`).
2. **`suggestion_active()`**: composition empty **and** the list non-empty.
   Any letter input (`push_composing`) clears the list, so the suggestion state
   exits naturally on the first character of the next word.
3. **Commit from the idle window**: `commit_suggestion(text)` commits the picked
   word **as the new previous word** (continuous suggestion chains 明天→早上→…)
   and recomputes the list. It deliberately does **not** call `record_user_word`:
   suggestions carry no reliable syllable mapping and recording them would
   pollute the user dictionary.
4. **View/TSF**: `candidate_ui_view` returns composition `""` with items = the
   suggestion list (`CandidateSource::Suggestion`, no new UI label);
   `refresh_candidate_window` hides only when composition **and** items are both
   empty, so the idle window stays visible; `candidate_window.update` relaxes the
   same condition. Placement for the empty composition comes from
   `selection_placement` (document insertion point via `GetSelection` + top of the
   range) instead of the composition range.
5. **Key semantics in the suggestion state**: digits 1..N → `Select` the
   suggestion at that index, out-of-range digits pass through to the host (no
   key-swallow, no empty commit); Space commits the currently selected row
   (default row 0); Esc clears the list and closes the window (keeps
   `previous_word`); Enter / Backspace / page-up-down / arrow-up-down are passed
   to the host — the suggestion window is a passive helper and must not take over
   host semantics like newline or backspace-delete.

## Alternatives considered

- **Fake composition mode for the idle window** (placeholder composition string).
  Rejected: it drags the suggestion into the composition lifecycle — TSF would
  run `SetText`/`EndComposition` and edit sessions against a phantom range, and
  suggestion semantics (digit select, exit-on-letter) differ from composition
  (digit appends or selects candidates); the two must stay separate.
- **Route suggestion commits through `commit_raw`** (which clears
  `previous_word`). Rejected: continuous suggestion chains require the committed
  suggestion to become the next previous word; clearing it would kill the chain.
- **Suggestion state swallows Enter/Backspace**. Rejected: host semantics
  (newline / delete-before-cursor) take priority; the window only reacts to
  digits, Space, Esc and letter input.
- **Carry the idle window as a TSF temporary composition**. Rejected: extra
  composition lifecycle for zero benefit — the engine state + empty-composition
  view covers rendering, placement and commits without a phantom composition.

## Consequences

- `zhu-ye-ime` 104 tests green (+5 suggestion: commit-then-suggest with digit
  select continuing the chain, letter exits, out-of-range digit + Enter pass
  through + Esc closes, Space picks the selected row, no-bigram engine degrades
  to no suggestion). Workspace 387 green; fmt / clippy `-D warnings` / diff check
  clean.
- `host-e2e --m8` on the real dictionary 7/7: 今天 → 的 first, words before
  phrases, digit select continues with 的, Esc closes, letter exits.
- The hit-rate baseline contract is still untouched: suggestion never
  participates in ranking (pinyin is empty while it shows), so T-057 eval does
  not need a re-run.
- VM interactive acceptance (acceptance standard 8.1 same flow: commit → window
  shows successors → digit picks one → next letter exits) runs after the PR lands.

## Supersession check

No active note is superseded: this note is the engine layer consuming the
retrieval note's contract (cross-linked above); hit-rate-eval-baseline and
ime-experience-optimization stay active. The retrieval note's consequence
("engine may cache the idle window but must recompute after each commit") is
realized here exactly as stated.
