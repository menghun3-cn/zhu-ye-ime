# Agent Note: Base-pack coverage tuning and audit metric (T-044)

Status: implemented

English | [中文](2026-09-29-base-pack-coverage-tuning.zh.md)

## Problem

S-1 required the base pack to pass two gates: coverage ≥98% and first-candidate
accuracy ≥90% on a 5000-word sample. The initial `audit-coverage` tool reported
100% coverage but only 70.5% first-candidate accuracy — far below the 90% gate.
Two questions had to be answered before the task could close: is the metric
itself fair, and are there real bugs inflating rare-word frequencies above
common words?

## Decision

Two build-time bugs were found and fixed, and the audit metric was split into
two tiers to separate language facts from ranking quality.

### Bug 1: polyphone overwrite in the skeleton

`load_xdhyc` returned one entry per skeleton row, including duplicates for
polyphones (`了` has `le` rank 590 and `liao3` rank 2542). `build_base`
inserted these into a `HashMap` by word, so the later row silently overwrote
the earlier one. The common reading was lost: `了` landed under `liao`, and
the `le` group contained only rare characters (`叻`, `鳓`, `泐`).

Fix: `load_xdhyc_ranked` now returns `(word, pinyin, rank)`. `build_base` keeps
the entry with the lowest rank (most frequent reading) via a `skeleton_rank`
map. The old `load_xdhyc` signature is kept under `#[cfg(test)]` for existing
unit tests.

### Bug 2: jieba expansion bypassed the frequency_of priority

The jieba expansion stage stored raw `jieba_score(word)` instead of
`frequency_of(word)`. The `frequency_of` closure encodes "wordfreq first, fall
back to jieba" — but the expansion stage ignored it. For words that exist in
both wordfreq and jieba (most CJK words), this meant the higher jieba scale
won. Example: `垸` has wordfreq 2450 but was stored at jieba scale 5690,
outranking the skeleton word `元` at 5600.

Fix: the jieba expansion loop now calls `frequency_of(word)` for both the
`--min-score` gate and the stored frequency value, matching the skeleton and
CEDICT stages.

### Audit metric: target-word vs group-winner

5000 common words map to only 3661 distinct pinyins. Homophones (`是/时/使`)
can only have one first candidate, so the target-word-first-candidate rate has
a mathematical ceiling of ~73.22%. Using it against the 90% gate would guarantee
failure regardless of ranking quality.

Fix: `audit-coverage` now reports two metrics:

- **Target-word first-candidate rate** — raw "is the queried word itself first?"
  Has a homophone ceiling (~73%), shown as a reference value only.
- **Group-winner rate** — "is the first candidate the skeleton's most-frequent
  word for that pinyin?" This is the ranking-quality metric gated against 90%.
  It isolates fixable frequency-assignment errors from irreducible homophone
  collisions.

The `CoverageReport` struct gained `distinct_pinyins`, `group_winner_hit`, and
`group_winner_pct` fields. The CLI gate uses `group_winner_pct ≥ 90%`.

### min-score scan

`--min-score` was scanned across 1000–3500. Both common-word metrics
(target-word 72.06% and group-winner 90.20%) were identical at every value —
`--min-score` only affects long-tail jieba expansion volume, not common-word
ranking. The default 2000 was retained as a balance between coverage (94.8%
char-set) and pack size (23.1 MB).

## Verification

After both fixes, rebuilt `base.zyct` (287,152 entries, 23.1 MB, SHA
`7f389214…`, deterministic across two builds):

- Coverage: 5000/5000 = 100.00% (gate ≥98% ✓)
- Target-word first: 3603/5000 = 72.06% (ceiling 73.22%, reference)
- Group-winner: 4510/5000 = 90.20% (gate ≥90% ✓)
- `了` correctly under `le` (rank 590), not `liao`
- host-e2e real-dictionary smoke: 6/6 PASS
- 319 workspace tests pass, clippy clean
- 2 new unit tests: `骨架多音词保留官方排名最优读音`, `jieba扩充词频取主源优先而非jieba标定值`

## Alternatives considered

**Keep the target-word metric and lower the gate to 72%.** Rejected: the gate
exists to catch ranking regressions, not to measure homophone density. Lowering
it would hide real bugs (the `垸`-over-`元` case passed at 70.5% under the old
metric). The group-winner metric catches exactly the fixable errors while
respecting the language fact.

**Filter out homophones from the sample.** Rejected: removing words that share
a pinyin would bias the sample toward rare pinyins and hide the most common
words (`是/时/使/十/事` are all top-50). The group-winner approach keeps the
full sample and separates the two concerns analytically.

**Raise `--min-score` to shrink the pack.** The scan showed 2500 cuts the pack
to 15.7 MB / 162,692 entries with no change to common-word metrics. Not chosen
because the extra 124,490 jieba-expansion entries provide long-tail coverage
(char-set 94.8% vs 94.3%) at 7.4 MB — well within the 60 MB budget. If pack size
becomes a constraint later, 2500 is a viable fallback with no quality cost.

## Consequences

The base pack is 5 MB smaller (28.2 → 23.1 MB) because jieba-expansion words now
get wordfreq-scale frequencies (lower than jieba scale), so more fall below the
`--min-score 2000` gate. This is correct behavior — the old artifact had
inflated frequencies. The `audit-coverage` subcommand is now the canonical S-1
verification tool; its two-metric output is referenced in 验收标准 7.5 and FR-018.