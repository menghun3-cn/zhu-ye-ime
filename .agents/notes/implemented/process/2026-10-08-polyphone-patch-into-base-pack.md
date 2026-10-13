# Agent Note: polyphone gap patches now applied to the shipped base pack (shui→谁 and 30 more)

Status: implemented

[中文](2026-10-08-polyphone-patch-into-base-pack.zh.md) | English

## Problem

User report (2026-10-08): "typing `shui` must also yield 谁, and similar
polyphone/口语音 characters must all appear." Investigation found the T-056 patch
mechanism only plugged into the **import pipeline** (`build_real_dictionary` →
`data/artifacts/real.zyct`), while the shipping product chain's
`data/artifacts/base.zyct` (`m6::build_base`: xdhyc skeleton + wordfreq +
jieba expansion + CEDICT fallback) **never applied the patch table** — the
released `bin/dictionary.zyct` is a copy of base, so on the user machine 谁 had
only the `shei` reading and 熟 only `shu`; the `shui`/`shou` keys were
unreachable. T-056 validated real.zyct on the VM but not the base chain — a
pipeline coverage blind spot.

## Decision

1. **Shared apply function**: `polyphone::apply_patch_entries(entries, patches,
   table)` extracted from the import-inline logic (add-only, frequency inherits the
   character's current max, character must exist in the word table, syllable must
   pass the standard table, skip existing combos / in-table duplicates; returns
   (applied, skipped)). `import` and `build_base` now share one codepath, so the
   pipelines cannot diverge again.
2. **build_base integration**: after entries are assembled and before
   `build_v2`, unconditionally load `data/patches/polyphone.tsv` and apply;
   `BaseStats` gains `polyphone_applied` / `polyphone_skipped`, printed in the
   build log.
3. **base-side gap audit**: `audit-polyphone` gains `--base <zyct>` — enumerates
   all standard syllables (`SyllableTable::complete_syllables_with_prefix("")`),
   `lookup`s each, collects single-character reading sets, and uses each
   character's max entry frequency (no external frequency file needed). Audits the
   shipped base rather than CC-CEDICT.
4. **Patch table expanded by 29** (31 total, see `data/patches/polyphone.tsv`):
   from the 460-entry base-side gap diff, human-curated **口语音/日常 single-char
   readings** — 的di 了liao 都du 还huan 着zhao/zhuo 地di 得dei 长zhang 觉jiao 绿lu
   行hang 血xie 调diao 藏zang 乐yue 吓xia 朝zhao 角jue 系ji 塞se 便pian 佛fu 似shi
   俩liang 色shai 露lou 重chong 陆lu. **Excluded**: surname/classical/place-name/
   disputed readings (区ou 万mo 大dai 说shui 会kuai 单chan/shan …) — compound-word
   keys already hit them, and single-char patch readings would pollute the key.
5. **Acceptance subcommand**: `zhu-ye-dict lookup <file> <pinyin>` lists entries
   for a full pinyin (frequency descending) for build/deploy verification.

## Alternatives considered

- **Engine special-case `shui`→谁**: fixes one symptom, bypasses ranking
  determinism, pollutes the hot path; already rejected in T-056.
- **Apply all 460 gaps**: many are surname/classical/place readings (区ou 龟qiu
  万mo 单chan/shan) that would pollute keys (e.g. `qiu` suddenly yielding 龟), and
  frequency inheritance would let them top their own primary readings; rejected.
- **Edit the CC-CEDICT source**: sources are read-only (pinned); CEDICT pinyin
  fields describe the entry's reading, not a place for colloquial single-char
  patches; the patch table stays.
- **Burning the gap list into the repo**: the audit is a process, not an
  artifact; the table still needs human curation; the audit only narrows the
  selection.

## Consequences

- base rebuilt at 287,233 entries (+29), `多音补丁 31/跳过 0`; `shui` now leads
  with 谁 (5840 > 水 5290) — the user's core complaint is fixed; verified also
  `liao`→了, `zhuo`→着, `yue`→乐, etc.
- 谁 rising to the top of `shui` is an intentional trade-off (frequency inherits
  the character's max, 5840, above 水 5290): 谁 first, 水 still second on the
  first screen. If ranking ever feels wrong, the alternative would be
  "patched-reading frequency = max frequency of the *key's* existing readings
  rather than the character's max" — not implemented.
- Future patch-table growth must rebuild base (release dictionary hash changes);
  real.zyct and base now apply the same table with identical semantics.
- Partially supersedes [2026-09-30-polyphone-gap-patch](2026-09-30-polyphone-gap-patch.md):
  its "import --polyphone at build time" scope extends to "import and build-base
  share `apply_patch_entries`"; the table mechanism/audit/curation principles are
  unchanged; the original note stays active and is cross-linked.

## Verification

- New `polyphone.rs` unit tests: shared apply function copies readings and
  inherits the char's max frequency; skips existing combos / in-table duplicates /
  out-of-table characters; rejects non-standard syllables.
- `audit-polyphone --base` actually reports 460 gaps against base readings.
- After the base rebuild, `zhu-ye-dict lookup` confirms **all 30 checkable keys
  hit** (shui→谁 #1, liao→了 #1, zhuo→着 #1, yue→乐 #4 …).
- Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --workspace` (core 327/ime 215/dict 62/settings 107/ui 35 green),
  `git diff --check` — all pass.
- Deploy: `copy-data` (dictionary.zyct rename-swap + .tones) to the product dir +
  Roaming copy sync; `lookup` re-verified against the deployed artifacts — see
  T-129 todos row and the deployment record.
