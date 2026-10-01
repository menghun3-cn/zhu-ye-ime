# Agent Note: Contact-index candidates (scenario 9, M11)

Status: implemented

[中文](2026-10-02-contacts-scenario9.zh.md) | English

## Problem

Scenario 9 (contacts, FR-036/FR-037/FR-038) needs local, privacy-minimal
contact names reachable from pinyin composition: the user exports a vCard 3.0
file, names it in `config.json`, and the full/initial pinyin of a contact's
name appears as a promoted candidate (full-word and abbreviation reachable,
FR-037). Confirmed constraints (2026-10-02, D-18~D-22): import from vCard
files, never the system address book (D-18); configured vcf paths trigger
import (D-19); index only the name (D-20); promote in the same layer as domain
boost (D-21, after base candidates and before the appended groups, contacts
after domain in that layer); polyphonic characters build keys for **every**
reading (D-22).

The ranking baseline (T-050) must not drift: with no contact configured the
candidate list must stay bit-for-bit identical to the pre-scenario-9 build.

## Decision

### Core: `zhu-ye-core/src/vcard.rs` (T-071-1)

`parse_vcard(input: &str) -> Result<Vec<VCardContact>, VCardError>` implements
a vCard 3.0 subset: `BEGIN:VCARD` section parsing, `FN` preferred over `N`
component join, other fields skipped, line folding and escapes handled,
malformed input reported with line numbers. 16 unit tests.

### Core: runtime pinyin table `zhu-ye-core/src/char_pinyin.rs` (T-071-2)

The core crates previously had no character→pinyin table, and the runtime must
annotate contact names offline. Following the `en_words` pattern (generator
script + checked-in generated file + pinned source), `scripts/build-char-pinyin.ps1`
generates `CHAR_PINYIN` from the pinned `kTGHZ2013.txt` cache (8,105 chars,
sha256 pinned), normalized to tone-less ASCII (`ü`→`v`), ordered by char for
binary search. This is the **runtime** companionship to the build-time
annotation pipeline in the dictionary-pack-build note: dict crates annotate at
build time; contact names annotate at index-build time, never on the hot path.

### Core: `zhu-ye-core/src/contacts.rs` (T-071-2/T-071-3)

- `annotate_name(name) -> Vec<String>`: per-character reading cartesian product
  (polyphonic full forms, D-22), ASCII letters/digits lowercased verbatim,
  spaces/symbols skipped, keys sorted+deduped; `CONTACT_KEYS_CAP` (64) bounds
  the product.
- `abbreviation_key(key, table) -> Option<String>`: derived **at index build
  time** from each full-pinyin key by syllable segmentation (first
  `segment_all` result on the standard syllable table) → first letter of each
  syllable. Non-pinyin keys (English names, digits) and single-syllable keys
  derive nothing. This gives FR-037 abbreviations without touching the FR-023
  dictionary initial path: `zs → 张三` hits the index directly. — Deviation
  from the draft design §3.2 which said "reuse FR-023": behavior is identical,
  mechanism is index-native.
- `build_contact_index(contacts: &[VCardContact]) -> ContactIndex`: BTreeMap
  over keys (full-pinyin + derived abbreviation), names de-duplicated per key,
  `CONTACT_INDEX_CAP` (10k) truncates by appearance order. — Deviation from
  the draft design §4 signature
  `build_contact_index(contacts, annotate: &dyn Fn(...))`: the built-in table
  removed the annotation callback (same rationale as `en_words`: self-owned,
  offline, deterministic; no trait object).
- `contact_candidates(index, pinyin_prefix, cap) -> Vec<Candidate>`:
  `partition_point` prefix range (`upper = prefix + "\u{10FFFF}"`),
  `Candidate::new(name, 0).with_source(CandidateSource::Contact)`, name
  seen-dedup keeping order.
- `CandidateSource::Contact` is a distinct variant: separate from user-word
  learning (FR-003 semantics — contact selections never enter the learning
  pipeline).

### Config (T-071-3)

`ConfigFile.contact_vcards: Vec<PathBuf>` (`#[serde(default)]`, missing =
empty = no-contact baseline) persisted to `config.json` (D-19; `config.json`
remains user-editable, FR-038 clear = empty list).

### Engine (`crates/zhu-ye-ime/src/input.rs`)

`InputEngine` gains `contacts: Option<ContactIndex>` set via
`with_contacts(index)` (None when the index is empty) and cleared via
`clear_contacts()`. In `refresh_candidates`, after the domain boost block and
before the FR-030 appended groups (English / abbreviation / emoji), a contact
step queries `contact_candidates(contacts, &self.composing, 8)` for any
non-empty composing string and appends through `append_group` — the same
layer as D-13, contacts after domain when both hit, contacts before the
appended groups (design §3.3 order). No index / empty input ⇒ step skipped
(T-050).

The TSF layer (`crates/zhu-ye-ime/src/tsf.rs`) gets `contact_engine(...)`: reads
`config.contact_vcards`, opens each file, `parse_vcard`, aggregates names,
`build_contact_index`, attaches via `with_contacts`; missing/unreadable/
unparsable files log `debug_log` diagnostics and are skipped; if nothing
parses the engine stays at the no-contact baseline. **No TSF interaction code
changes.**

### Acceptance: `host-e2e --m12 [<vcf file>]`

The in-memory assertion group (deterministic base table + built contact
index) covers: full-pinyin prefix reachability with base-after ordering,
`zs` abbreviation, polyphonic abbreviation (`cz`→曾子), English-name verbatim
key (`al`→Alice), no-hit bit-identical baseline, and `clear_contacts`
restoring baseline. With a real vcf file the full import chain
(parse → index → reachability) is re-checked. Result: 7/7 including the real
import.

## Alternatives considered

- **Windows system address book (WinRT `Windows.Contacts`)**: rejected (D-18) —
  COM/WinRT FFI would violate the core-crates zero-.NET constraint, and
  privacy-minimality favors an explicit exported file.
- **Annotation callback injected by the IME**: rejected — the built-in
  kTGHZ2013 table is self-owned, offline, deterministic, and matches the
  `en_words` precedent; no `dyn Fn` plumbing.
- **Abbreviations through the FR-023 dictionary initial path**: rejected — the
  FR-023 path (and its O-02 narrow triggers) stays untouched; index-native
  abbreviation keys deliver the same `zs` semantics with zero hot-path or
  dictionary changes.
- **Runtime per-character fallback annotation on the hot path**: rejected —
  same reason as the dictionary-pack-build note: annotation happens once at
  index build, never per refresh.
- **GBK encoding support in the parser**: deferred — vCards are accepted in
  UTF-8; non-UTF-8 files are reported and skipped (documented in the design
  §8 risk list).

## Consequences

- No contact configured / all files invalid ⇒ bit-for-bit baseline (T-050),
  verified by m12 case 5 and the workspace regression suite.
- Contact candidates carry `source = Contact`; the UI adds no label.
- Determinism (FR-002): same vcf + same input ⇒ same list; keys are Ordinal-
  sorted, multi-key same-name entries de-duplicated.
- Contact selections never enter the user-word learning pipeline (FR-003
  remains untouched; independent source variant).
- Privacy: contact content only lives in the user's imported file and the
  in-memory index; never in dictionary artifacts, logs, or commits
  (FR-038 clear deletes index; runtime file handling outside this batch).
- New acceptance command `host-e2e --m12 [<contacts.vcf>]`;
  `--m11`/`--m7`/`--m8`/`--m9`/`--m10` unchanged (workspace + e2e regression
  green after this batch).

## Related notes

- [2026-10-02-domain-boost-scenario8](2026-10-02-domain-boost-scenario8.md):
  D-13 layer; contacts share its insertion point (D-21, after domain).
- [2026-10-02-ime-experience-optimization](2026-10-02-ime-experience-optimization.md):
  FR-023 dictionary initial path — unchanged; contact abbreviations are
  index-native.
- [2026-09-28-dictionary-pack-build-and-manifest](2026-09-28-dictionary-pack-build-and-manifest.zh.md):
  build-time annotation pipeline; the runtime table here is its parallel for
  contact-name annotation.
