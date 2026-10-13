# Agent Note: User dictionary persistence and recovery for T-009

Status: implemented

[中文](2026-09-19-user-dict-persistence.zh.md) | English

## Problem

FR-003 requires locally persisted user-word learning with delete and reset,
and NFR-002 requires automatic backup recovery when the user-word file is
corrupt. Before this task, `UserDictionary` only held in-memory state:
selecting a candidate never wrote anything to disk, so learned words were lost
on process exit and there was no durable storage contract (path, format, or
failure semantics) for the IME and CLI to share.

## Decision

`zhu-ye-core::user_store::UserDictStore` owns the on-disk contract. Callers
inject the JSON file path: the TSF host uses
`%APPDATA%\zhu-ye-ime\user_words.json`, while CLI and tests use their own
paths, keeping the core platform-independent.

The file is versioned JSON: `{"version": 1, "entries": [{"word", "pinyin",
"frequency", "last_used"}]}`. `UserWord` derives serde; `UserDictionary`
gained `from_entries`, which filters empty text, empty pinyin, and
zero-frequency entries. `words_sorted` now sorts by frequency descending, then
word and pinyin, so saved files are deterministic.

Writes are atomic: `save` creates the parent directory, writes a `.tmp` file,
calls `sync_all`, then renames over the target, and removes the temp file on
failure. `load` treats a missing file as an empty library. Parse failures and
unsupported low versions rename the corrupt file to `.bak` and save an empty
library; versions newer than the supported one are refused without touching
the file.

`InputEngine` gained `with_user_store`, `Candidate` carries the pinyin used to
learn, and commit (space or numeric selection of a real candidate) records the
selection then saves through the store. Enter and raw-pinyin fallback do not
learn. `delete_user_word` and `reset_user_words` update memory and disk
together. The CLI exposes `user list / delete / reset` for observable
verification.

## Alternatives considered

**Save every selection asynchronously.** Rejected: the file is small and the
sync path is fast; synchronous saves keep the durability contract easy to test
and review. Deferred to T-017 observation if latency appears on slow disks.

**Store user words inside the dictionary binary.** Rejected: dictionary files
are read-only mmap data while user words are mutable per-user state; mixing
them breaks the T-006 replacement boundary and makes backup/reset unsafe.

**Use a binary or SQLite format.** Rejected: the data is small and
user-owned; human-readable versioned JSON is debuggable and serde_json
matches the existing design choice for user-state persistence.

**Overwrite the corrupt file in place.** Rejected: an archive is mandatory
for NFR-002 recovery and diagnostics; `.bak` keeps at least the last bad
file available.

## Consequences

Selection now survives process exit and feeds ranking on the next start. The
`Candidate` type carries pinyin, which gives commit a durable key for learning
and later sync/AI features. TSF host wiring means the DLL writes only under
`%APPDATA%`; tests and CLI never touch that path unless configured to.

T-009 stays in progress until VM end-to-end verification (install, type,
select, restart, then observe ranking and the JSON file) passes. The ranking
interface from
[2026-09-19-candidate-ranking-static-model.md](../feature/2026-09-19-candidate-ranking-static-model.md)
consumes the persisted dictionary without structural change.
