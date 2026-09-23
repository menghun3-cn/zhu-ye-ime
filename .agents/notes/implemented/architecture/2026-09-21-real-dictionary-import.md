# Agent Note: Real dictionary import pipeline from CC-CEDICT and word frequency data

Status: implemented

English | [中文](2026-09-21-real-dictionary-import.zh.md)

## Problem

T-006 defines the dictionary data pipeline, but the only usable dictionary data
was the 20-entry self-built demo seed. FR-002 needs real unigram frequencies
for candidate ranking, and FR-007 needs real bilingual entries with pinyin so
the input engine can suggest translations. The raw data had to enter through a
reproducible import stage instead of being hand-copied into source, because
the derived dictionary must stay deterministic, license-compliant, and easy to
rebuild.

## Decision

`zhu-ye-dict import` consumes CC-CEDICT plus FrequencyWords Chinese word
frequency, optionally adds an OPUS GlobalVoices tokenized Chinese corpus for
bigram statistics, and compiles a real v2 dictionary. The parser accepts the
official space-separated `简体词 [pin1 yin1] /译文/` format and the older
tab-separated form by locating the `[pinyin]` brackets instead of relying on
field count. Each line passes through five cleaning rules before compilation:

1. The simplified word must be pure CJK.
2. Pinyin is split on spaces/apostrophes, normalized to unaccented ASCII
   (`ü`/`u:` become `v`), and validated against the same 410-syllable standard
   table used by the engine.
3. The syllable count must equal the character count.
4. Duplicate word-plus-pinyin entries keep only the first.
5. FrequencyWords counts attach to matching words; words without a frequency
   entry get 1.

The import never converts multi-syllable pinyin into one joined string before
validation, because that would lose syllable boundaries. Joining happens only
after validation so the dictionary key stays compatible with engine lookups
such as `nihao`.

The real run on 2026-09-21 accepted 120,028 entries from 125,113 CC-CEDICT
lines, hit 28,130 frequency entries, validated 318,015 syllables, and rejected
894 non-standard syllables. The same run consumed 397,039 corpus lines from
OPUS GlobalVoices v2018q4 (`mono/zhs.tok.gz`) and produced 820,368 unique
bigram pairs. Bigram cleaning treats each corpus line as an independent
context, keeps only pure-CJK whitespace tokens, and splits each token by
longest dictionary match (up to 8 characters); unmatched characters break the
context so punctuation and unknown spans do not create cross-sentence
co-occurrences. Counts saturate at the `u32` limit of the v2 format. Source
URLs, licenses, SHA-256 values, cleaning rules, reproduction command, and
result hashes are recorded in `docs/数据清单.md` and `docs/licenses.md`; raw
files and generated artifacts stay out of git. The imported dictionary now
carries real bigram records, so T-008's ranking verification can run against
`data/artifacts/real.zyct`.

## Alternatives considered

**Split pinyin fields by counting fields.** Rejected: the official CC-CEDICT
export uses spaces and the pinyin inside `[...]` also contains spaces, so a
bare delimiter count cannot identify the simplified word, pinyin, and
translations reliably. Locating the bracket delimiters works for both current
space-separated and legacy tab-separated exports.

**Use a prebuilt third-party dictionary binary.** Rejected: the project's
principle is to keep the data pipeline self-controlled, deterministic, and
compatible with the custom v2 mmap format. Importing from open text sources
keeps licensing and the build path auditable.

**Keep duplicate entries for the same word and normalized pinyin.** Rejected:
a tone-less key cannot distinguish pronunciations that normalize to the same
string, so keeping both would only keep the last one in the pinyin index and
make the build input inconsistent. One entry per word-plus-normalized-pinyin
keeps `build_v2` deterministic.

**Count raw whitespace tokens without dictionary segmentation.** Rejected:
Chinese OPUS tokens often span several dictionary words or include
punctuation, so raw token adjacency would introduce noisy and rare pairs.
Segmenting only against the import vocabulary keeps the bigram statistics
consistent with the entries shipped in the same dictionary.

**Use an external segmentation engine (for example jieba).** Rejected: it
would add a large dependency and a second vocabulary with different
segmentation conventions. Longest dictionary match is deterministic,
license-light, and sufficient for static co-occurrence statistics; richer
segmentation belongs to the user-side AI features, not the offline import.

## Consequences

The pipeline now produces a real dictionary with 120,028 bilingual entries,
unigram frequencies, and 820,368 real bigram records, and the
unknown-syllable sample list still gives a concrete signal for extending the
engine's syllable table. The generated artifact is not committed, so the
runtime seed dictionary remains the default until real data is packaged
through the release process. Because CC-CEDICT and FrequencyWords content are
CC BY-SA 4.0 and GlobalVoices content is CC BY 3.0, any future distribution
of derived dictionary packages must carry attribution and share-alike
compliance for the applicable sources; raw data and local artifacts are not
part of the repository today.
