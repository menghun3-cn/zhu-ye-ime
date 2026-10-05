# Agent Note: Contact search index extended to company/address/email (T-102, D-20 revision)

Status: proposed

[中文](2026-10-05-contact-index-extra-fields.zh.md) | English

## Problem

Contact search currently indexes names only (decision D-20, confirmed
2026-10-02, note 2026-10-02-contacts-scenario9): typing `zhangsan`/`zs` reaches
the contact, but typing the company name, postal address, or an email username
never does. The user's accepted batch-three list explicitly asks to "extend
contact search to company/address/email", which revises D-20 and also aligns
with the scenario-9 requirement text itself (§16.1: "地址/邮箱/公司名也可联想").

## Proposal

1. **vCard extraction expands to three more fields.** `VCardContact` gains
   `org`/`email`/`address` (plus a `new(name)` constructor for tests/lone-name
   contacts). `extract_contact` collects `ORG`/`EMAIL`/`ADR` non-empty values in
   occurrence order joined with `"; "` (no truncation of multiple values), is
   order-independent (FN may appear before or after them — assembly happens at
   card end, FN wins over N), and keeps group-prefix/escape handling. `TEL` and
   other fields stay ignored (they are not part of the approved scope).
2. **Index builds keys from four sources.** `build_contact_index` now delegates
   to `contact_keys`, which merges `annotate_name` output of name + org +
   address + email and dedups (a key produced by two sources indexes the person
   once). Chinese company names/addresses get pinyin keys (`竹叶科技` →
   `zhuyekeji`, `北京市` → `beijingshi`); emails become one continuous lowercase
   key with symbols skipped (`bob@acme.com` → `bobacmecom`) so typing the
   username prefix reaches the contact, while the `@` domain part never becomes
   an independent key. Hits from any source still surface the **name** as the
   candidate text. Abbreviation-key derivation (FR-037) applies to every merged
   key unchanged.
3. **Scope discipline.** No new runtime state, no config, no TSF-layer change:
   the parse output and the index contract are the only changed surfaces;
   `contact_candidates` is untouched. T-057 eval cannot regress (contact
   candidates appear only on hits).

## Alternatives considered

- **Extra key source "rollup" per contact (single `annotate_name(name + ' ' +
  org + ...)` concatenation)**: cheaper, but mixes pinyin romanization across
  fields and loses the clean dedup boundary → rejected; per-field key sets with
  merge are clearer and deterministic.
- **Structured `Vec<String>` for multiple values per field**: faithful to
  vCard multiplicity, but the index only needs text for key derivation and the
  demo/tests only assert join semantics → single `String` joined with `"; "`
  keeps the surface minimal.
- **Index emails as exact-match only (no prefix)**: less useful for the
  typical "type the username" flow → rejected; prefix over the continuous key
  covers `bob`, `bob@`, `bob@acme` uniformly.
- **Make TEL a fifth key source**: user scope names company/address/email only;
  TEL numbers add little search value and widen the change → deferred,
  documented in the acceptance standard as still-ignored.

## Acceptance criteria

- `cargo test --workspace` green: `vcard` 22 (incl. 6 field-extraction tests),
  `contacts` 20 (incl. 6 four-source key tests); ime and host-e2e construct
  contacts via `VCardContact::new` and still pass.
- host-e2e `--m12` 9/9: new cases 7 (company pinyin `zhuye` → 张三), 8 (email
  username prefix `bob` → 张三), 9 (address pinyin `beijing` → 张三) all PASS;
  existing cases 1-6 unchanged.
- Docs: requirement §16 (D-20 row with 2026-10-05 revision note), acceptance
  §12.1 (four-source rows), contact-design doc §1/§3/§4/§5/§7 consistent with
  the implementation.
- Gates: fmt --check / clippy -D warnings / git diff --check /
  verify-agent-notes / verify-translation-pairs all clean.

## Risks

- **Key pollution**: extra sources add keys, so more pinyin prefixes could hit
  a contact — that is the feature, but the dedup keeps candidate count bounded
  and order deterministic (BTreeMap + stable push).
- **Email mis-keying**: a continuous `bobacmecom` key cannot be split at query
  time, so only username-prefix queries are supported; a future search-by-
  domain would need separate indexing (deliberate scope cap).
- **Multi-value growth**: `"; "`-joined values inflate key lengths; anchored by
  the per-source 64-key cap and CONTACT_INDEX_CAP, no unbounded growth.
- **vCard variance**: some exporters write `ADR;TYPE=HOME:...` with parameters —
  covered by existing param handling; non-UTF-8 CHARSET cards still skip
  (unchanged).
