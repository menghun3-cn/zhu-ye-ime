# Agent Note: Slang pack and build-time content gate (M6-P)

Status: implemented

English | [中文](2026-09-28-slang-pack-and-content-gate.zh.md)

## Problem

M6 ships an internet-slang pack (FR-016/017). It must hold two kinds of
entries: pure-Chinese slang typed as normal pinyin (内卷, 破防), and letter or
digit abbreviations (yyds → 永远的神). No license-clean, pinned slang corpus
exists; open candidate pools are unvetted crowd text. FR-021 also demands a
content gate: zero offensive, political, or advertising entries may leak, false
kills must stay at or below 5%, and every build must record the gate version
and its hit data. The blocklist itself must never ship. Abbreviations are
risky too: a key such as `wo` or `emo` can also be split into pinyin
syllables, so letting it into the abbreviation path would pollute ordinary
typing.

## Decision

`zhu-ye-dict build-slang` (`crates/zhu-ye-dict/src/slang.rs`) builds
`data/artifacts/slang.zyct` from three committed, project-owned inputs under
`data/slang/`:

- `seed.tsv` — `词<TAB>键<TAB>类型<TAB>来源`. Type `词` is pure-Chinese slang.
  An empty key means annotate via CC-CEDICT word tier then kTGHZ char tier. An
  explicit key uses apostrophe-separated full pinyin (for polyphones such as
  `长草 zhang'cao`) and is checked syllable by syllable. Type `缩写` stores
  the key as the v2 entry's pinyin field. Every row needs a source note.
- `blocklist.tsv` — `词 模式 类别` with a mandatory `# version:` line. It has
  four modes: `包含` (substring), `精确` (exact word), `放行` (an allow-string
  that exempts a substring hit it fully covers, e.g. 大麻烦 over 大麻), and
  `缩写键` (exact abbreviation key). Comparison first normalizes the text:
  full-width ASCII becomes half-width, letters are lowercased, and whitespace
  is removed.
- `gate-samples.tsv` — `标签<TAB>类型<TAB>文本` negatives and positives.

Build order is audit first, then filter. The audit runs every sample through
the gate, and the build fails when negatives or positives number fewer than
200, when any negative leaks, or when the false-kill rate exceeds 5%. Only
then does it filter the seed rows. Rows the gate hits are reported and
dropped. Abbreviation keys must be at least two lowercase letters or digits.
A pure-letter key must not segment fully into standard syllables
(`segment_all` returns empty). Keys that fail go to the excluded list; they
are never "fixed" into the pinyin path. Every entry gets a flat frequency of
5000 (a mid zipf×1000 value on base's scale). The report (blocklist version,
audit counts, killed positives with the matched pattern, gate-blocked rows,
excluded rows, size, SHA-256) is written to `data/artifacts/slang.gate.json`.
It is a build artifact next to the pack, not part of the manifest.

Measured on 2026-09-28: blocklist 151 rows (version 2026.09.28-1). Samples:
310 negatives (the original terms plus context, upper-case, full-width, and
spaced variants), all blocked. 453 positives (every seed row plus benign
look-alikes), 1 killed (0.2%: `屌丝` hits `屌`, kept as an auditable known
kill). Seed 413 rows → 319 words + 92 abbreviations; `wa` and `emo` were
excluded as segmentable; 411 entries, 26,248 bytes. Two builds are
byte-identical. The pack is in manifest 2026.09.28-p3.

## Alternatives considered

**Import `social-media-chinese-words` or a similar crowd pool now.** Deferred:
its quality and licensing need per-row review. The seed table plus the gate
give a reviewed baseline, and a pool can later feed candidates into
`seed.tsv` through the same gate.

**Filter only, without a sample audit.** Rejected: a blocklist that "passes"
the seed proves nothing about leaks or over-blocking. The audit makes the
FR-021 thresholds an executable build gate.

**Treat segmentable abbreviations (`emo`, `wa`) as abbreviations anyway.**
Rejected: 方案 11.4 forbids separable strings on the abbreviation path.
Otherwise typing `emo`/`wa` would surface slang in the ordinary candidate
stream.

**Ship the blocklist so the runtime can filter user input.** Rejected: the
gate is a build-time quality control, and shipping the list would publish an
offensive-term catalog in the installer.

## Consequences

- Benefits: every slang build proves FR-021 on its own inputs; the report
  gives a versioned audit trail; the abbreviation rule is enforced at build
  time, so the runtime path (M6-R) can trust every letter key in the pack.
- Costs: the seed and blocklist need manual maintenance per release; the
  sample set must grow with the blocklist to keep ≥200 per side meaningful.
  Digit keys (996, 520) are packed; the engine reaches them since T-049
  resolved the digit-key semantics (see the
  [digit abbreviation key note](../bug-fix/2026-09-29-digit-abbreviation-key.md)).
- Runtime lookup, the tail-group placement, and the `[网络]` label belong to
  M6-R. This note covers only the build side.
- Supersedes nothing. It extends
  [domain pack build and manifest pipeline](2026-09-28-dictionary-pack-build-and-manifest.md),
  and the task is tracked as T-045 in [todos-list](../../../../docs/todos-list.md).
