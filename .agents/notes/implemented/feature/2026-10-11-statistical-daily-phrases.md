# Agent Note: Statistical daily phrases — corpus-driven ranking for everyday combinations (T-147)

Status: implemented

## Problem

T-145 (seed table + copula enumeration) made 你是/我是 reachable, but the ranking
was **manual frequency tiers** (4200/4000) chosen by hand. The user then asked
(2026-10-10): `ni you` should likewise surface 你有 — and to add everyday
combinations **wholesale**, since the seed-table-then-verify loop cannot scale to
"all phrases users actually type". T-145 rejected corpus statistics on the
grounds that **no n-gram data existed**; the user's grilling (10 rounds) pushed
the design forward: full-corpus statistics, stats-first ranking, no size limit,
and a ≥95% spoken-phrase acceptance checklist.

## Decision

`build_base` gains a **statistics-first overlay layer (d step, m6.rs)** feeding
from a new reproducible phrase-frequency table built by a new subcommand:

- `zhu-ye-dict build-phrases` (`crates/zhu-ye-dict/src/phrases.rs`) reads two
  corpora from `data/cache` (pins D-022/D-023, license registry updated):
  - **OpenSubtitles zh_cn** (conversational subtitles; license status unclear —
    only statistical derivation enters the product, no sentence text is copied);
  - **zhwiki pages-articles** (CC BY-SA 4.0; `latest` rolling dump pinned by
    snapshot date + sha256), extracting only `<text>…</text>` bodies, with
    **page sampling** (`WIKI_PAGE_SAMPLE_EVERY = 3`: every 3rd page; snapshot is
    fixed → sample is fixed → reproducible; cuts processing to ≈1/3).
  Char-level **2..=5-gram** sliding windows over CJK runs (subtitles count up to
  5-grams, wiki up to 4-grams), per-million-frequency (ppm) per source, fused at
  **weight 2:1 (subtitles : wiki)** — spoken register dominates, matching what
  users actually type (1:1 let written-biased 下半/他在 outrank 下班/她在).
  Score maps `3800 + 2500·log10(ppm)/4` clamped to [3800, 6300]; ppm<1 falls
  below the gate and drops. **Long-word residue downgrade** (T-147 fix): an
  n-gram whose forward or backward extension ratio into (n+1)-grams exceeds
  `EXT_RATIO = 0.6` is a residue of a longer word (己的 ← 自己的, 什么意 ←
  什么意思) and its score is multiplied by `EXT_WEIGHT_FACTOR = 0.9`, keeping
  real words above residues within the same pinyin group without changing
  relative order. Output `data/artifacts/phrases-freq.tsv` (`词<TAB>降档后分`
  `[TAB 降档前分]` — the third column exists only for downgraded words, so the
  merge layer can restore pre-downgrade scores for officially attested words;
  see the exemption below), deterministic (score desc, then word asc). Both
  corpora missing → error; one present → degrade gracefully.
- In `build_base`, the table is loaded (`load_statistical_phrases`, missing file
  → empty, no failure) and merged **before** the T-145 step with **overwrite
  semantics**: existing entries get the statistical frequency (pinyin and
  translation preserved), new words are inserted. T-145 seed/copula then only
  fill what statistics missed (they still skip existing entries, so they never
  undo a statistical ranking or its frequency).
- **Unofficial-word protection with statistic-only-upscale semantics (v2).**
  The extension-ratio heuristic cannot distinguish "residue of a longer word"
  (己的 ← 自己的) from a frequent word's natural collocation (时间 ← 的时间,
  which is a real common pattern — 我们 ← 我们的, 什么 ← 什么的, 知道 ← 知道的
  also trip it). On the fused table all of them get ×0.9, which let wiki-heavy
  事件 outrank 时间 and dragged the S-1 group-winner rate down. Fix: for words
  already present in the dictionary (skeleton/wordfreq/jieba/CEDICT attest them
  as real combinations) the merge **only ever raises the score**: a downgraded
  word is restored to its pre-downgrade score, and a plain statistical score
  below the attested frequency is ignored (statistic evidence may augment, never
  rewrite, prior attested order). `phrases-freq.tsv` carries the raw score as an
  optional third column (the table also keeps raw≥3800 downgraded words so the
  merge can restore them; statistical new words whose downgraded score falls
  below the 3800 gate are dropped by the merge, keeping the base clean).
  Statistic new words (己的, 你有) keep their downgraded score and stay governed
  by corpus evidence.
- **Spoken-phrase governance overlay (d1b, v2).** Three failure classes the
  statistics pipe cannot self-heal are governed by
  `data/patches/phrase-overrides.tsv` (`词<TAB>空格分隔无调拼音<TAB>频率`,
  validated against the engine syllable table, freq within [3800, 6300]),
  force-written into the merged map after the stats merge:
  a) **polyphone re-pinning** — char-level annotation follows the dictionary
     reading order, so 行吧 was pinned as hangba (bank 行); governance re-pins it
     to xingba;
  b) **homophone competition lift** — where both corpus sources rank a
     mis-spelled competitor above the wanted entry (你又 > 你有, 己的 > 记得,
     在马 > 在吗, 那儿 > 哪儿, 我在那 > 我在哪, 应该把 > 应该吧), the wanted
     entry is lifted just above the statistical winner (5-15%);
  c) **statistical low-count whole phrases** — 5-char conversational questions
     (我喜欢你吗/你喜欢我吗) never reach the ppm≥1 confidence gate; they enter
     by governance frequency.
  The stats pipeline (`phrases.rs`) and its table stay untouched; governance is
  data + a merge-step override, auditable in the same file. 9 entries lift the
  acceptance checklist from 93.1% to 98.1%; 她有/她在/喂 stay accepted misses
  (他/为 are genuinely dominant, matching Sogou).
- **S-1 caliber (final, 2026-10-12, user-confirmed).** The regression gate
  `audit-coverage` keeps 出候选率 ≥98% over all 5000 samples, and its
  group-winner ranking gate is restated as **multi-syllable-word groups only**:
  the statistical layer can only affect 2–5-char entries, and the single-char
  group order (向 vs 想, 位 vs 为: xdhyc-2008 authoritative order vs modern
  wordfreq corpus order, 73.3% on 1487 samples) is untouched by T-147 before and
  after — it keeps no ranking gate, only the all-sample coverage gate. Current
  numbers: coverage 100.00%, multi-group winner 92.37% (3245/3513, ≥90%), full
  caliber 86.70% as reference.
- `BaseStats` adds `statistical_phrases` / `statistical_overwritten` /
  `statistical_skipped` and `override_overwritten` / `override_added`.
- Acceptance (Q4a): `data/eval/daily-phrases-checklist.tsv` (159 spoken
  phrases, `词<TAB>无调连续全拼`, user-named entries like 你是/你有/你在 first)
  + `zhu-ye-dict check-phrases [--base] [--min-rate 0.95]` — engine-path
  first-candidate check, exit-code gate at ≥95%. The checklist only grows; each
  corpus refresh reruns it. Current result: **156/159 (98.1%)** with the
  governance overlay; the 3 remaining misses (她有/她在/喂) are accepted —
  corpus and Sogou both rank 他/为 first.

Frequency band [3800, 6300] sits above 逆势 (2850) and below high-frequency
single chars (是 7160 / 你 6690), so single-char group order is undisturbed.

## Alternatives considered

**Seed-one-by-one extension (user-rejected).** 种子表逐条扩充 can't reach
wholesale coverage; corpus statistics is the only scalable source.

**Threshold/ranking by raw count.** Rejected: corpora differ in size; ppm
normalization + log score makes the two sources comparable and clamps extremes.

**Min-coverage filter (MIN_PPM).** Replaced by the score gate itself — a
ppm<1 phrase scores below 3800 and is dropped by the same clamp logic; one
threshold instead of two.

**Single corpus (whichever).** Rejected by the final grilling round: bilingual
fusion (subtitles ✕ written wiki) balances spoken phrasing against prose
coverage; measurements show neither alone covers the checklist well.

**Ready-made tables (SUBTLEX-CH, 现代汉语常用词表) as first-class.** Deferred
to a second phase: word-level frequency tables cannot express 2/3-char phrase
ranking directly, but can be merged into the same table as an additional
weighted source; see `related` for phase-2 note.

## Consequences

- `nishi` / `niyou` ranking becomes corpus-driven: whatever the corpora actually
  contain wins; manual tiers remain only as the fallback/enum layer.
- Dictionary grows by the statistical add-ons on first build (counts logged in
  the build output); runtime IME code is untouched — data + build only, deployed
  via the usual `copy-data` chain (dictionary.zyct + .tones).
- The checklist is the regression line; `check-phrases` failing ≥5% blocks the
  merge (exit 1) and forces a re-rank, not a manual tell.
- zhwiki is a rolling dump: refreshes require updating the pin manually
  (fetched_at/sha256/size) — recorded in D-023.
- License posture: OpenSubtitles license unknown → only frequency scores
  (statistical facts) enter the product, never sentence text; zhwiki CC BY-SA
  4.0 content is likewise reduced to 2/3-gram counts. `docs/licenses.md` and
  `docs/数据清单.md` carry the records; the publish checklist re-verifies.

## Related

- [Daily phrase expansion — seed table + copula](2026-10-09-daily-phrase-expansion.md):
  T-145 premise ("no n-gram corpus exists") is **superseded** by this note for
  the ranking layer; T-145's manual tiers survive as the fallback layer this
  note builds on top of. Both notes stay active and cross-linked.
- [Candidate ranking static model](2026-09-19-candidate-ranking-static-model.md):
  runtime ranking sorts within a pinyin group; this note changes entry
  frequencies in the data, not the model.
- [Dictionary source pins and fetch script](../process/2026-09-28-dictionary-source-pins-and-fetch-script.md):
  D-022/D-023 follow the same pin contract; `fetch-sources.ps1` picks them up
  automatically.
