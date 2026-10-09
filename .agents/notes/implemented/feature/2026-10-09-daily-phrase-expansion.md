# Agent Note: Daily phrase expansion — seed table + copula enumeration (T-145)

Status: implemented

## Problem

User asked (2026-10-09): typing `ni shi` should rank everyday high-frequency
phrases like 你是 first (aligned with Sogou), not 逆势 — and asked whether daily
typed combinations need their own vocabulary aggregation.

Field evidence from the shipped dictionary (`C:\Program Files\zhu-ye-ime\tsf\dictionary.zyct`):

- `rank nishi` yields only five whole words: 逆势 (2850), 泥湿 (2477), 睨视
  (2477), 逆事 (2477), 逆施 (2477). 你是 is **not in the base dictionary** at all.
- The self-learning user dictionary (`user_words.json`) cannot help: it only
  records words the user actually commits as one candidate, and a word that is
  never a candidate can never be learned (chicken-and-egg).

Why is 你是 missing? All three base sources treat such grammar-level phrases as
separate tokens: CEDICT is a dictionary, jieba tokenizes 你是 into 你/是, and
xdhyc is a character table. There is no n-gram corpus anywhere in the pipeline.

## Decision

`build_base` gains a **daily-frequent-phrase expansion step (T-145, m6.rs)
right after the jieba expansion**, with two candidate sources:

1. **Seed table** `data/patches/daily-phrases.tsv` — manually maintained pure-CJK
   phrases (2–5 chars), one per line, `#` comments and blank lines ignored,
   duplicates keep first occurrence. Covers conversational formulas that no
   corpus would ever produce from our data (你是/我是/你是谁/你是不是/怎么办/
   什么时候/没错/好吧/…). Frequency: `DAILY_PHRASE_FREQ` = 4200.
2. **Copula enumeration** — `COPULA_PREFIXES` whitelist (personal/demonstrative/
   interrogative pronouns and common adverbs) × 是, generating 你是/我是/其实是/
   反正是/… up to 4 chars. Frequency: `COPULA_PHRASE_FREQ` = 4000.

Both sources are annotated by the existing `tables.annotate` (kTGHZ char-level,
no hand-written pinyin, no spelling drift). Words already in `merged` are
**skipped, keeping their original frequency** (e.g. 是不是 keeps 5320) —
never overwrite existing entries. Frequency tiers sit above 逆势 (2850) and the
current first candidates of common phrase pinyin groups, and below absolute
high-frequency single chars (是 7160 / 你 6690), so `shi` is unaffected.

`BaseStats` adds `daily_seed` / `daily_copula` / `daily_skipped`; the new
`zhu-ye-dict freq <词...>` subcommand probes wordfreq for data-source debugging.

## Alternatives considered

**Pure wordfreq subset.** Rejected: `freq 你是` shows 你是 is absent from the
wordfreq zh corpus entirely (it is space-tokenized into 你/是), as is 我是 and
most grammatical phrases. There is nothing to take.

**Corpus bigram statistics (globalvoices / social media).** Rejected after
measuring: `globalvoices_zhs.tok.gz` is word-tokenized news text — adjacent
single CJK chars across the whole 8 MB corpus number only 97 pairs, and neither
你是 nor 我是 occurs once. The social-media source is a word list, not running
text. No n-gram data exists to derive phrase frequencies with.

**Seed table only, no enumeration.** Rejected: coverage stays at manual breadth;
enumeration makes the "X是" family systematic (90+ copula phrases for free,
consistent 4000 tier) while the seed table stays authoritative for irregular
formulas.

**Aggregate combinations at runtime instead of build time.** Rejected: that is
what `user_words.json` already does, and it is exactly the chicken-and-egg
mechanism that cannot fix the first appearance; the base dictionary must carry
the phrases so the user can commit them once, after which self-learning takes
over per-user personalization.

## Consequences

- `nishi` now ranks 你是 (4200) first, 逆势 (2850) second; `woshi` ranks 我是
  (4200) above 卧室 (4020); single-char groups are unchanged (是 7160 still the
  `shi` head).
- Base dictionary grows by the added phrases (287,233 → 287,342 entries on the
  first build; seed 22 + copula 87 added, 43 already-present skipped).
- S-1 `audit-coverage` metrics are identical to the pre-change baseline
  (candidate rate 100.00%, first-candidate-correct 89.60%) — the expansion does
  not perturb single-char ranking.
- Runtime IME code is untouched: the whole fix is data + build. Deployment is
  the usual `copy-data` dictionary chain (dictionary.zyct + .tones), no DLL swap.
- The seed table is the maintenance surface for further "daily phrase" reports;
  growing it only reruns `build-base` + `copy-data`.

## Related

- [Candidate ranking static model](2026-09-19-candidate-ranking-static-model.md):
  runtime ranking sorts by per-entry frequency within a pinyin group; this note
  supplies missing high-frequency entries rather than changing the model.
- [User dict persistence](2026-09-19-user-dict-persistence.md): per-user
  whole-word learning + promotion (D-70/D-71); the expansion makes such phrases
  reachable so learning can start.
- [Real dictionary import](../../architecture/2026-09-21-real-dictionary-import.md)
  and [Dictionary source pins and fetch script](../process/2026-09-28-dictionary-source-pins-and-fetch-script.md):
  the build pipeline this step joins; the phrase table is a patches-level input,
  no new pinned source.
