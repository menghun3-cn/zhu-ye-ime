# Agent Note: Personalized candidate ranking — user words as an independent candidate group

Status: implemented

[中文](2026-10-20-personalized-candidate-ranking.zh.md) | English

## Problem

The user reported that Sogou places a word they have already used first in
the candidate list on the next input of the same pinyin, and framed the
underlying pain as a probability statement rather than a single feature
request: 「用户同样的输入拼音，是输入相同词汇的概率较大，每次都翻页体验不好」.

The shipped ranking was a single global model. `StaticRankingModel::rank`
computed `total = 词典频次 × unigram_weight(1) + 用户频次 × user_weight(48)`
(`crates/zhu-ye-core/src/candidate.rs`), so the personal signal competed with
the global signal inside one addition. Measured against the deployed
dictionary and the real 235-entry user dictionary on the development
machine, the global side dominates completely:

| pinyin | candidate | dictionary frequency | user frequency | user bonus | total | rank |
| --- | --- | --- | --- | --- | --- | --- |
| `nishi` | 你是 | 5010 | 0 | 0 | 5010 | 1 |
| | 逆势 | 2850 | 6 | +288 | 3138 | **17** |
| `niyou` | 你有 | 5300 | 1 | +48 | 5348 | 1 |
| | 昵友 | 2477 | 4 | +192 | 2669 | **6** |
| `nihao` | 你好 | 5377 | 27 | +1296 | 6673 | 1 |

A user word only reaches the first position when
`用户频次 × 48` exceeds the dictionary frequency gap — roughly 100 uses for
the most frequent everyday words. A separate gap existed for words the
dictionary does not contain: `generate_candidates` draws every candidate from
`dictionary.lookup()` (`candidate.rs`), and `UserDictionary` was consulted in
exactly one place in the whole core crate (the frequency lookup above), so a
word such as 「你觉得」 (`nijuede`) or 「你角色如何」 (`nijueseruhe`) — both
present in the real user dictionary, both absent from the dictionary — could
never appear as a candidate at all. A scan of all 235 real user words found
11 such words.

## Decision

Personal signals no longer compete with global word frequency inside one
addition. They form a separate candidate group that is placed in front of the
global candidates and is ordered by personal signals only.

### Layer 1 — user words as an independent candidate group

`zhu_ye_core::candidate::user_group_candidates` returns the user words whose
pinyin equals the composition, scored by
`用户频次 × USER_GROUP_FREQUENCY_WEIGHT(48) + 转移频次 × USER_GROUP_TRANSITION_WEIGHT(64)`.
The dictionary frequency is deliberately absent from this score, so a word used
six times outranks 「你是」 with its frequency of 5010.

The group is prepended in `InputEngine::refresh_candidates` after the manual
separator group (T-128's hard syllable boundary is the strongest expression
of user intent) and before the appended groups. `prepend_group` gives the
user group priority when the same text also appears in the global candidates.

### Layer 2 — user-level binary grammar

`UserDictionary` now records `(previous, word, pinyin) → frequency` transfers
when a candidate is committed while a previous word exists. The transfer table
is consulted in the same group score with a higher weight than the
unconditional frequency, so a preference under a specific context outranks a
preference without one. The table is capped at `USER_TRANSITION_CAP` (2000)
and evicted by lowest frequency, then oldest use.

### Layer 3 — dictionary-external user words

When the composition has no whole-word hit in the dictionary, user words
whose pinyin matches are injected into the group even though the dictionary
does not contain them. When the dictionary does have a hit for that pinyin,
dictionary-external user words are not injected, because the global candidates
already cover the pinyin and injecting them would only displace normal
candidates.

### On-disk format

`user_words.json` moves to version 2 with a `transitions` section. A v1 file
is migrated losslessly on read — the word entries are preserved, the transfer
table starts empty, and the file is atomically rewritten as v2. `v0` (no
version field) remains a corrupt file, handled by the existing backup-and-
rebuild path. The read-only probe `probe_user_words_file` treats v1 as
healthy for the same reason: classifying a migratable file as corrupt would
make the settings window's level-1 repair offer to rebuild the file and throw
away every word the user has learned.

## Alternatives considered

- **Raise `user_weight` so the personal signal out-scales the dictionary**:
  the arithmetic works but the knob is global. A weight large enough to beat
  「你是」 (5010) would also make a single accidental selection dominate
  permanently, and it still leaves dictionary-external words invisible. The
  user's phrasing — a probability over *repeated* inputs — is a statement
  about ranking structure, not about a constant.
- **Keep one addition and add a separate user-word boost term**: the same
  arithmetic problem remains; any finite boost competes against a dictionary
  frequency of 5000+, and the same boost applies to every user word rather
  than to the user's actual preference ordering.
- **Time-decayed frequency (exponential decay over selection timestamps)**:
  designed first, then rejected on the real data. 161 of the 235 real user
  words have been used exactly once and the whole dictionary records 496
  selections, so decay has nothing to decay for most entries — a word used
  once has one timestamp and no history. The binary-grammar layer carries
  strictly more signal on this data, which is also closer to the user's
  probability framing.
- **Store a full `(previous, word)` probability table**: quadratic growth in
  the vocabulary. The current table stores only transfers the user actually
  produced, and the cap bounds it.
- **Store the transfer table in a separate file**: two files would need
  coordinated atomicity and a second corruption-recovery path. The transfers
  are part of the same learned state as the words, and one atomic write
  covers both.
- **Inject dictionary-external user words even when the dictionary has a
  hit for that pinyin**: this would put a rarely used made-up word ahead of
  the normal candidates for a common pinyin, displacing what the user most
  likely wants. Injection is restricted to the case where the global path
  cannot produce the pinyin at all.

## Consequences

- A word the user has used once is now the first candidate for that pinyin.
  Verified against the deployed dictionary and the real user dictionary:
  `nishi` → 「逆势」 (was rank 17), `niyou` → 「昵友」 (was rank 6),
  `nijuede` → 「你觉得」 (was absent), `nijueseruhe` → 「你角色如何」 (was
  absent), `nihao` → 「你好」 and `shenme` → 「什么」 unchanged.
- Because the group is placed in front, the candidate list changes order once
  a word enters the user dictionary. Selecting by number therefore selects
  from the new order — the first candidate is the one the user wants, and
  space is enough. This is inherent to the placement and matches Sogou.
- The existing `user_part` weighting inside `StaticRankingModel::rank` is
  kept. It still affects candidates outside the user group, so the previous
  behavior remains observable there and the existing test baselines hold.
- `user_words.json` grows a `transitions` section. The file stays local; no
  network access is introduced (NFR-010 unchanged).
- The user dictionary is now written on every commit that has a previous word,
  so the file is touched slightly more often. Writes remain atomic
  (temporary file plus rename) and failures stay non-fatal.
- `InputEngine::user_dictionary_mut` and `InputEngine::set_previous_word` are
  public so a host can seed personal preferences and a context explicitly.

## Verification

- Core unit tests: user words ordered by personal frequency only; dictionary
  external words injected; not injected when the dictionary has a hit;
  context transfer raises the in-group score; empty inputs return no group.
- Core unit tests: transfer recording, empty previous word is not a
  transfer, cap eviction drops the lowest-frequency oldest entry, rebuild
  filters invalid transfers, reset clears the transfer table.
- Core unit tests: v1 file migrates losslessly to v2 and the file on disk is
  rewritten as v2; v2 transfer section round-trips; v0 is still treated as
  corrupt; the probe reports v1 as healthy and v0 as corrupt.
- IME unit tests: a word used once is immediately the first candidate with
  source `User`; a dictionary-external word enters the candidates; the
  transfer table records only commits that have a previous word; a context
  transfer reorders the user group.
- Real-data end-to-end (ignored test, run against the deployed dictionary and
  the real user dictionary): `nishi` → 「逆势」, `niyou` → 「昵友」,
  `nihao` → 「你好」, `nijuede` → 「你觉得」, `nijueseruhe` → 「你角色如何」,
  `shenme` → 「什么」, all with source `User`.
- Full gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --workspace`, `git diff --check`.

## Related

- [Candidate ranking with the static model](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
  — introduced `frequency_by_word` and the single-addition scoring this
  decision replaces for the user group.
- [User dictionary persistence](../../implemented/feature/2026-09-19-user-dict-persistence.md)
  — the `user_words.json` contract whose version this decision advances.
- [Settings window input management and repair](../../implemented/feature/2026-10-02-settings-input-management-and-repair.md)
  — the level-1 repair path whose probe semantics this decision preserves for
  migratable files.
