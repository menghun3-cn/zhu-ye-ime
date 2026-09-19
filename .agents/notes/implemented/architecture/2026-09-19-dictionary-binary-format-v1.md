# Agent Note: Dictionary binary format v1 and mmap loading

Status: implemented

English | [中文](2026-09-19-dictionary-binary-format-v1.zh.md)

## Problem

The IME needs a dictionary pipeline that can grow from a tiny demo seed to
real word lists without scattering ad hoc loaders across crates. Loading a
large dictionary into heap memory would be unacceptable for a desktop IME,
and a versionless file format would make upgrades and corruption detection
ambiguous. T-007 needs a verified syllable table and T-008 needs real bigram
statistics; both consumers should receive data through the same normalized
path.

## Decision

A versioned v1 binary dictionary format is defined once in
`zhu-ye-core::dict_format` and shared by the builder, the loader, and the CLI.
The file starts with four bytes `ZYDT`, version `1`, and a fixed 96-byte
header; the content SHA-256 is stored at header bytes 56..88 and covers every
byte after the header. The layout is:

```text
Header        96 bytes: magic, version, counts and offsets, SHA-256
Pinyin index  24 bytes per record: pinyin text offset/length, entry start/count, hit count
Entry table   24 bytes per record: pinyin index, frequency, translation offset/length
Bigram table  16 bytes per record: previous/next entry index, frequency
Text pool     UTF-8 variable-length text; offsets start at 1, 0 means no translation
```

`dict_builder::build_v1` produces deterministic bytes: entries are sorted by
pinyin group, text is deduplicated in the pool, bigrams are merged, and all
counts and offsets are computed from the same source data. `DictionaryFile`
in `zhu-ye-core::dict_loader` opens the file read-only with mmap, validates
magic, version, length, hash, UTF-8, partition, and sort invariants on load,
and implements both `Dictionary` and `BigramModel` over record views without
copying the file into the heap.

`zhu-ye-dict` owns the CLI: `build` writes `data/artifacts/seed.zyct` by
default, `inspect` prints header statistics, and `verify` loads the file once
and checks every visible entry and bigram. `zhu-ye-cli demo` and the TSF
adapter try the app-data or artifact file first and fall back to the in-memory
demo dictionary when the file is missing or invalid.

## Alternatives considered

**JSON or text word lists.** Rejected: they are easy to debug but require
full parsing when loading, take more disk space, and do not support mmap
entry lookups safely.

**SQLite.** Rejected: it adds a heavyweight third-party dependency and
runtime complexity that the project deliberately avoids; the dictionary is a
read-only derived artifact, not an online transaction database.

**Compress the whole file.** Rejected for v1: a compressed package prevents
direct mmap access and is better as a separate release transport format
later; v1 keeps raw records so the loader stays minimal.

**Use CC-CEDICT or another third-party source for the seed.** Rejected: the
license position is still being reviewed, so v1 ships a self-built demo seed
and keeps third-party data registration separate in `docs/licenses.md`.

## Consequences

The dictionary pipeline now has a single byte contract and one verified
loading path: build a file, hash it, mmap it, query it. Tests cover buffer
round-trips, deterministic builds, tamper rejection, stable ordering, and
file-driven candidate generation in the input engine. T-007 and T-008 can
consume the same loader once real syllable and bigram data are introduced.

Mmap keeps memory flat while the OS pages in only the records touched by a
query. Future format changes must bump `DICT_VERSION` and keep the header and
hash contract stable; migration is deliberately out of scope while the format
is still internal.
