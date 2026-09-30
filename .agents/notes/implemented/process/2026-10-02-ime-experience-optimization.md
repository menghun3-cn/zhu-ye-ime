# Agent Note: IME experience optimization — initials, fuzzy correction, sentence beam search

Status: implemented

[中文](2026-10-02-ime-experience-optimization.zh.md) | English

## Problem

The M7 scope (docs/输入法体验优化.md) targets the acceptance metrics that were
holding the IME back: Top1 ≥ 85–90%, Top3 ≥ 95%, whole-sentence accuracy 90%+,
first-screen latency < 50 ms, personalization, and an experience close to 搜狗.
The user picked all three proposed scenarios, with hard constraints: everything
must stay offline, self-built, deterministic, reusing the existing syllable
table + bigram + user-word infrastructure, and must not change the v2 dictionary
format, its build pipeline, or its update chain (decisions O-01..O-06).

Three concrete gaps were identified:

1. **No initials/short pinyin.** `nh` for 你好 or `wsm` for 为什么 was impossible:
   the composition is not segmentable, so the normal pipeline produced nothing.
2. **No fuzzy correction.** `zongguo` (zh↔z) or `niha` (missing trailing `o`)
   produced nothing plausible, and there was no mechanism to propose the
   corrected form without polluting ordinary ranking.
3. **No whole-sentence search.** A long string like
   `woxiangmingtianqubeijing` was assembled syllable by syllable (first word per
   syllable), ignoring bigram coherence and cross-syllable whole words.

## Decision

### 1. Initials (简拼, FR-023)

`pinyin::INITIAL_SYLLABLE_TABLE` is a static, self-maintained table mapping 22
initials (`b c d e f g h j k l m n o p q r s t w x y z`) to 4–6 common syllables
each, ordered by linguistic frequency (language facts, no license burden).
`initial_syllables(initial)` returns the slice; unknown letters return empty.

`candidate::initial_candidates(dictionary, initials)`:

- length 2..=4, all ASCII lowercase only — digits and single letters are
  rejected up front (single-letter fan-out is refused, and digits stay owned by
  the T-049 digit-abbreviation semantics);
- the cartesian product over the per-letter syllable sets is expanded
  front-order, and every combination is looked up as a **whole word** in the
  dictionary;
- if any letter has no table entry, or the combination count exceeds 128, the
  whole function returns empty (anti-explosion: 4 letters × 6 syllables =
  1296 combinations would otherwise be enumerated);
- results are capped at `INITIAL_COMPLETION_CAP = 32`, order is the table
  expansion order (deterministic).

Engine integration is deliberately narrow (anti-pollution, O-02): the initials
path runs **only** when the main candidate list is empty, the composition is
2–4 lowercase ASCII letters, and `segment_all` cannot segment it. Legal pinyin
like `wo` or `nih` never enters the path, so existing behavior is untouched.

### 2. Fuzzy correction (模糊音与纠错, FR-024)

`pinyin::FUZZY_GROUPS` holds the seven fuzzy pairs from the design doc: zh↔z,
ch↔c, sh↔s, n↔l, f↔h, an↔ang, en↔eng, in↔ing. `fuzzy_variants(syllable)`
applies exactly **one** substitution per call, deduplicates through a HashSet,
never includes the original syllable, and is deterministic.

`candidate::corrected_candidates(table, dictionary, pinyin)` triggers only when
the string **is** fully segmentable and has **no whole-word hit** (exact lookup
empty):

- **A — fuzzy substitution**: for every syllable of the first segmentation,
  `fuzzy_variants` replaces it in place, the re-concatenated string is looked
  up as a whole word;
- **B — missing-letter completion**: for the **last** syllable only,
  `complete_syllables_with_prefix` extends it (e.g. `ha` → `hao`), again
  whole-word lookup.

Hits are tagged `CandidateSource::Corrected` and returned as a separate group
(UI adds no new label). The engine appends them with `append_group` **after**
the main candidates and **before** the slang abbreviation group; on text
collision the main candidate wins. Volume is capped at `CORRECTION_VARIANT_CAP
= 24`.

### 3. Whole-sentence beam search (整句 Beam Search, FR-025)

`candidate::sentence_candidates` triggers only when the string is fully
segmentable, has at least 3 syllables, and has no whole-word hit. The key
implementation insight: **cross-syllable whole-word matching**. A naive
per-syllable beam can never select `明天` (pinyin `mingtian`, two syllables)
because each step queries one syllable only. Instead the search enumerates all
separable substrings (1 to `SENTENCE_MAX_WORD_CHARS = 12` chars, i.e. up to 4
syllables) at each position and looks each substring up as a whole word.

Scoring models **word-to-word transitions** rather than summing bare
frequencies; three facts were tuned against the real corpus during M7-A before
the acceptance case passed:

1. **unigram cap** (`SENTENCE_UNIGRAM_CAP = 100_000`): without a cap a
   high-frequency function character (e.g. 被) dwarfs a whole word («北京» has
   unigram 1088 in the real base pack) and the beam degrades to per-character
   concatenation.
2. **bigram-miss penalty equals the cap** (`SENTENCE_BIGRAM_MISS_PENALTY =
   100_000`): with any penalty < cap a capped high-frequency character still
   keeps a positive residue, and two residues together beat a low-frequency
   whole word. Penalty = cap makes an unsupported transition contribute ≤ 0, so
   a random concatenation «去被敬» (去→被 has no bigram evidence) is always
   deeper than a supported «去北京».
3. **completed paths survive across rounds**: a whole-word step («明天» = 8
   chars) lets the winning path finish early while per-character paths still
   advance; without a dedicated `completed` set the slow paths keep replacing
   finished ones on later rounds and the top candidate regresses.

Score = min(freq, unigram cap) × unigram_weight, plus min(bigram, cap) ×
bigram_weight when the previous→word transition has evidence (> 0); otherwise
the penalty is subtracted. All in saturating i64 (deterministic). Beam state is
(word sequence, consumed char count, score); `BEAM_WIDTH = 8` paths survive
each step, each substring contributes at most `BEAM_WORD_CAP = 4` words,
tie-break is by word text ascending. Consuming at least one character per step
guarantees termination in ≤ len steps. Completed paths are deduplicated by
sentence text (max score) and the top `SENTENCE_TOP_N = 5` are returned with
`pinyin` = the original string.

The engine places the sentence group **first** via `prepend_group` (main
candidates yield on text collision). If all beams die, the function returns
empty and the caller keeps the ordinary per-syllable candidates, guaranteeing
≥ 1 candidate.

### Engine assembly

`InputEngine` gains a `bigram: Arc<dyn BigramModel>` field (the beam needs it;
`StaticRankingModel` keeps its own). `new` uses `EmptyBigramModel`, and
`with_bigram` / `with_user_store_and_bigram` / the file-based constructors feed
the same instance the ranking model uses. `refresh_candidates` now runs the
flow: prefix groups (T-029, unchanged) → main candidates → sentence group
(first, if any) → corrected group (after main) → initials (only when main is
still empty and the string is unsegmentable) → slang abbreviation tail
(M6-R, unchanged). Two tiny helpers `prepend_group` / `append_group` keep the
group order fixed while deduplicating by text.

## Alternatives considered

**Broader initials triggers (single letter, or letters+digits).** Rejected by
O-02: a single initial would fan out across syllables and pollute; digits
belong to the T-049 abbreviation semantics. The narrow gate (empty main +
unsegmentable + 2–4 lowercase letters) is what keeps `wo`/`nih`/`u1s1` out.

**Per-syllable beam.** Rejected: it cannot select cross-syllable whole words
(`明天` = `mingtian`), so the headline acceptance case
`woxiangmingtianqubeijing` → 我想明天去北京 would rely on single-character
frequencies and often degrade. Substring whole-word lookup costs a bounded
enumeration (≤ len × 12 per state) and stays well under the latency budget.

**Always run beam even on whole-word hits.** Rejected: `nihao` must keep
producing 你好/尼好 exactly as before; the no-drift regression floor forbids
changing short-string behavior.

**Let corrected candidates compete in normal ranking.** Rejected: a fuzzy hit
with high static frequency could outrank the user's intended pinyin candidates;
a separate labeled group after the main candidates is the anti-pollution
position agreed in O-03.

**Use only the first segmentation for correction.** Rejected: the DP order is
not guaranteed to be longest-word-first; the final implementation does not pick
a segmentation at all for the beam — it enumerates separable substrings by
position and keeps the fewest-syllables guard for the trigger only.

## Consequences

- `CandidateSource` gains a `Corrected` variant (no UI label). It is appended
  as a separate group; `corrected_candidates` and `sentence_candidates` are
  pure functions in `zhu-ye-core` with no Windows dependency.
- Existing behavior changes in exactly two observable ways, both intended:
  `niha` now offers 你好/尼好 (corrected), and long unsegmentable strings gain
  initials results; the page-clamp test was updated accordingly (退格到 `ni` 时
  页码收敛归零的断言签名改为退格三次).
- Determinism: all three paths are order-deterministic (table order, HashMap
  only for dedup with explicit sort afterwards); no randomness, no wall clock.
- Performance: each path is capped (≤128 combos, ≤24 corrected, ≤8×4 beam×5);
  the whole sentence path adds a bounded substring enumeration per position.
  VM acceptance for first-screen latency (<50 ms) is part of M7-A.
- Tests: core +13 (pinyin 4: table coverage / lookup-miss / one-substitution /
  no-mapping-empty; candidate 9: initials 5, correction 3, sentence 3 —
  including the headline `woxiangmingtianqubeijing` → 我想明天去北京, bigram
  steering, a high-frequency-character adversarial case added during M7-A, and
  no-drift guards); zhu-ye-ime +10 engine-level (nh/wsm, zongguo, niha,
  whole-word no-trigger, sentence-first, short-string no-drift). The full gate
  is green: fmt, clippy `-D warnings`, `cargo test --workspace` (core 132,
  ime 99), `git diff --check`, host-e2e seed 19/19, multi-pack 7/7, and the new
  `--m7` assertion group 22/22 against the real base pack.
- Performance (measured, release + real base pack, benchmark 8.2):
  `zhu-ye-cli bench` gained the three-path scenario; initials 13.6 µs,
  corrected 5.7 µs, sentence 1630 µs (~1.6 ms) per refresh — all inside the
  ≤30 ms acceptance (≤15 ms target). Zero network by construction (O-05).
- Related active notes (keep active, cross-link): the segmentation
  infrastructure (feature/2026-09-19-full-pinyin-segmentation-core), prefix
  candidates (feature/2026-09-24-prefix-candidates-incomplete-segmentation,
  initials complement it for unsegmentable strings), and the ranking weights
  (feature/2026-09-19-candidate-ranking-static-model, beam reuses them).
  Nothing in the M7 batch supersedes those; the language-fact tables
  (initials, fuzzy pairs) are self-maintained and need no license entry.
- VM interactive acceptance of the visible candidate window (验收标准 8.1)
  passed end-to-end on the acceptance VM (Windows Server 2019 zh-CN, real TSF
  stack, four packs + dictionary): 6/6 checks green — `nh` first=你好,
  `wsm` first=为什么, `zongguo` window contains 中国 (select key 3 commits it),
  `niha` window contains 你好 (select key 3), whole-sentence case first=
  我想明天去北京, and `nh` + digit 1 commits 你好. Evidence: TSF debug log
  `cand-show first=`/`commit`, six screenshots with distinct MD5 (画面变化真实),
  pixel forensics (候选窗主题色包围盒: s1–s5 可见, s5 最宽与 items=9 一致,
  s6 上屏后隐藏). Deployment note: the dictionary file is mmap-locked by TSF
  inproc hosts, so install into a fresh directory (`tsf-m7`) instead of
  overwriting; later cleanup then removes the legacy directory.
