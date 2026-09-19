# Agent Note: Full-pinyin segmentation core for M2

Status: implemented

[中文](2026-09-19-full-pinyin-segmentation-core.zh.md) | English

## Problem

The M1 scaffold shipped a hand-picked syllable table and linear recursive
segmentation. The table omitted common syllables such as `cha`, `duo`, and
`guan`, so arbitrary full pinyin could not be segmented reliably; the recursive
walker did not match the design's dynamic-programming plan; and there was no
extension point for double pinyin.

## Decision

`zhu-ye-core::pinyin` now provides the M2 full-pinyin core.

`STANDARD_SYLLABLES` contains 410 standard unstressed Mandarin full-pinyin
syllables in lowercase ASCII (`ü` is written as `v`), grouped by initial for
manual review. Syllables are language facts rather than implementation
copyright; the source is registered as D-003 in `docs/licenses.md`, and T-006
will regenerate and validate the table from authoritative data.

`SyllableTable::standard()` keeps syllables in a sorted `Vec<String>`.
`is_complete_syllable` uses binary search; prefix queries use
`partition_point` plus a `starts_with` scan instead of walking the whole table.
The old `basic()` constructor is gone; the input engine and CLI now use
`standard()`.

`segment_all` builds the segmentation bottom-up with dynamic programming:
`dp[end]` holds every segmentation of the first `end` bytes, and the final
ordering is deterministic by end/start position. Non-ASCII input and
un-segmentable ASCII strings return an empty list.

`PinyinScheme` is the reserved input-scheme interface. `FullPinyinScheme`
normalizes keys to lowercase; a double-pinyin implementation can be added later
without changing the segmentation core.

## Alternatives considered

**Keep the linear recursive walker.** Rejected: the design specifies
dynamic-programming segmentation, and prefix queries benefited from the sorted
array while documenting a clear target for the future mmap index.

**Introduce an existing pinyin/rime engine.** Rejected: the project commits to
self-developed, controllable core algorithms; adopting a third-party engine
would replace the problem instead of solving it.

**Implement double pinyin now.** Deferred: FR-001 deliberately excludes
non-full-pinyin schemes; the trait reserves the interface without expanding M2.

**Build a full trie now.** Rejected for this milestone: 410 short syllables
already fit compactly in a sorted array, and T-006 can add a trie or compact
index when the mmap dictionary format is defined.

## Consequences

All standard table entries segment to themselves, `xian` still yields both
`xi-an` and `xian`, and `xiange` yields all four pinyin-valid segmentations.
The CLI self-check reports 410 syllables; `demo cha` now segments where the
old scaffold failed. The workspace test suite grows to 38 passing tests and
clippy passes with `-D warnings`.

T-007 remains in progress until T-006 regenerates the table from authoritative
data and the M2 acceptance check passes. The segmentation order is stable but
not scored; candidate ranking is owned by T-008.
