# Agent Note: Multi-sense translations (per-sense rows, POS stripped on commit, Ctrl+digit quick commit) (T-131)

Status: implemented

## Problem

User requests (2026-10-09), three together:

1. Committing English with Tab must not bring the part-of-speech prefix along (e.g. `v. suspicious` must commit `suspicious`, not `v. suspicious`).
2. 你好 must show **both** `hello` and `hi` as candidates. Today the translation layer shows only one sense: the m6 build truncates every translation at the first `;` (`clean_translation`), and the patch table overrides 你好's `hello; hi` with a single `hello`.
3. Ctrl+digit must directly commit the translation (no Tab toggle first).

Confirmed product decisions: split **all** multi-sense translations (`a; b`) into separate rows, not just 你好; Ctrl+digit works on **both** layers — on the Chinese layer it commits the Nth candidate's first-sense translation, on the translation layer it commits the Nth row; out-of-range or no-translation cases pass through to the host.

## Decision

The v2 dictionary format is **unchanged**: translations stay a single string field, multi-sense data is stored as one `a; b` string (m6 stops truncating). Splitting happens at display/commit time, not in the data.

- **m6 build** (`crates/zhu-ye-dict/src/m6.rs`): `clean_translation` keeps the whole sense string (`hello; hi`) instead of the first sense, cleaning per-sense trailing punctuation only through core `split_translations`. The 你好 patch becomes `hello; hi`. The POS-prefix loop is now per-sense (`pos_label_each`): `hello; hi` + `int.` → `int. hello; int. hi`.
- **Reverse index** (`crates/zhu-ye-core/src/dict_builder.rs`): keys are emitted per sense (`split_translations` → `strip_pos_prefix` per sense), with a per-source `seen_keys` guard; the loader's strictly-increasing (key, word) sort requirement still holds because multiple keys for the same word sort fine.
- **IME translation layer** (`crates/zhu-ye-ime/src/input.rs`): `build_translation_candidates` expands each translatable Chinese-layer candidate into one row per sense (text = word, translation = single sense, pinyin/source carried, score decremented per extra row to keep order). The Chinese layer's candidate-window secondary text shows only the first sense (avoids `int. hello; int. hi` overflow).
- **POS stripping on commit**: display keeps the prefix (`v. suspicious` shown); the committed text is stripped everywhere it leaves the engine — `commit_candidate` (translation layer + space preview via `display_text`) and the new direct-commit path.
- **Ctrl+digit** (`crates/zhu-ye-ime/src/tsf.rs`): new `KeyAction::CommitTranslation(usize)`. `plan_action`'s modifier branch calls `plan_modifier_key(wparam, lparam, shift, ctrl_held, engine)`; it yields `CommitTranslation` only when Ctrl is held alone (Alt excluded), the engine is composing, and `engine.can_translate_by_index(index)` (Chinese layer: visible candidate's sense list non-empty; translation layer: the split row exists). Everything else under Ctrl/Alt still returns `None` → host passthrough. `commit_text` pre-fetches via `engine.preview_translation_by_index` and `sync_engine` advances state via `engine.commit_translation_by_index`, which shares the commit-candidate teardown (record user word + pinyin, clear composition, set previous word, refresh suggestion).

Split-then-strip ordering matters: stripping `int. hello; int. hi` as a whole would wrongly leave `int. hi`. Sense splitting always happens before POS stripping.

## Alternatives considered

**Split multi-sense entries at the data level (separate dictionary records per sense).** Rejected: candidate generation deduplicates by candidate text, so same-word multi-sense entries would collapse back into one row; it would change the v2 format, loader sort keys, and forward/reverse mapping semantics for no user-visible gain.

**Commit the full string and strip POS at the very end only.** Rejected for the translation layer: the commit candidate carries the whole `int. hello; int. hi` string, and a whole-string prefix strip leaves a stray `int. ` on the second sense. Requires split-then-strip anyway (implemented).

**Reuse `commit_candidate` for Ctrl+digit by fabricating a selection.** Rejected: `commit_candidate` decides text vs. translation by the current layer, which mis-models the Chinese layer case (commit the first sense *as text* while staying on the Chinese layer). A dedicated `commit_translation_by_index` keeps both layers' semantics explicit.

**Ctrl+digit anywhere, consuming even when not translatable.** Rejected per user decision: no-translation/out-of-range must pass through to the host (e.g. Ctrl+1 in an English app context).

**Alt+digit or Ctrl+letter also routed through the modifier branch.** Rejected: only Ctrl+digit with a translatable candidate is consumed; Alt combinations (and other Ctrl shortcuts) were already documented as host passthrough and stay that way.

## Consequences

- Translation-layer row counts grow to sense counts (你好 → two rows, `hello` and `hi`); long multi-sense tail words get many rows. Score decrement keeps within-word sense order stable.
- The dictionary format is untouched, so loaders, `zh_to_en`, and `en_to_zh` keep working; reverse keys become more (per sense), at a small dictionary-size cost.
- POS prefixes stay visible user-facing (translation layer display), matching the pre-existing T-115 decision; only committed text is stripped. `previous_word` stores the stripped text (bigram context unaffected).
- `base.zyct` changes (translations longer, per-sense POS), so the whole chain must be rebuilt and redeployed together with the IME DLL.
- Ctrl+number now has IME meaning only during an active composition with a translatable candidate; everywhere else the key still reaches the host.

## Related

- [Acceptance fix batch 3 — POS labels in translations (T-112 follow-up)](2026-10-07-acceptance-fix-batch-3.md): m6 POS-prefix decision; this note refines it to per-sense application, keeping batch-3 as the origin of the label mapping and display format.
- [Candidate window TSF integration](2026-09-20-candidate-window-tsf-integration.md): translation-layer mechanics (`CandidateLayer`, Tab toggle, commit returns to Chinese layer).
- [Polyphone patch into the base pack (T-129)](../process/2026-10-08-polyphone-patch-into-base-pack.md): the build/deploy chain that `base.zyct` ship rides on.
