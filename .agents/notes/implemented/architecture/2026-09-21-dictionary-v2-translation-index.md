# Agent Note: Dictionary v2 translation index and reverse lookup

Status: implemented

English | [中文](2026-09-21-dictionary-v2-translation-index.zh.md)

## Problem

FR-005/FR-006/FR-007 require bilingual suggestions at input time: the IME
must show a translation next to a Chinese candidate and let the user commit
either the Chinese word or its translation. Keeping translations only in the
entry table cannot answer a standalone English-to-Chinese query, and an
in-memory translation table would split the data contract from the dictionary
artifact. A real word list also needs a deterministic reverse path from
normalized English back to the Chinese word.

## Decision

The dictionary format is upgraded from v1 to v2 (`DICT_VERSION = 2`). The
fixed header grows from 96 to 128 bytes, the content SHA-256 is stored at
header bytes 80..112, and it still covers every byte after the header. Between
the bigram table and the text pool, v2 adds two read-only sorted indexes:

```text
Word Translation Index  24 bytes/record: Chinese word text offset/length, translation text offset/length
Reverse Translation     24 bytes/record: normalized English text offset/length, Chinese word text offset/length
Index
```

`dict_builder::build_v2` emits deterministic bytes: entries are grouped by
pinyin, the text pool is deduplicated, bigrams are merged, and both translation
index counts and offsets are computed from the same source data. The reverse
key normalizes English text with lowercase plus whitespace folding, so
`HELLO` and `good morning` resolve to the same keys as their seed values.

`DictionaryFile` implements `Translator` over the mmap record views.
`zh_to_en` binary-searches the Chinese word index; `en_to_zh` normalizes the
query before binary-searching the reverse index. Both lookups return `None`
immediately when their index count is zero, so a valid dictionary without any
translations still loads and queries safely. `zhu-ye-dict` build, inspect, and
verify commands validate the translation and reverse translation counts,
bounds, and sort order on load; `zhu-ye-cli dict <file> -r <english>` exposes
the reverse lookup for development.

## Alternatives considered

**Keep translations only in the entry table.** Rejected: there is no way to
answer a standalone English-to-Chinese query without scanning the whole table
per lookup. A sorted reverse index keeps both directions in binary-search
time with the same mmap record views.

**Store translations in a separate JSON/SQLite table.** Rejected: a second
store would split the build/verify path, add a heavyweight dependency, and
break the single mmap byte contract that keeps loading cheap and deterministic.

**Forward translation only, no English reverse lookup.** Rejected: FR-007
requires reverse lookup with case and whitespace tolerant matching; the
normalized key is the deterministic form shared by builder, loader, and CLI.

## Consequences

The dictionary artifact now carries both directions of the bilingual layer,
and the input engine can drive its translation candidate layer from the same
mmap file used for pinyin and bigram lookups. The new indexes are validated
during load, and an empty translation set remains a legal dictionary state.
Tests cover file round trips, normalized reverse keys, tampered header counts,
and engine behavior with v2 files. Format consumers must expect
`DICT_VERSION = 2`; v1 files are no longer loadable.
