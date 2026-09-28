# Agent Note: Dictionary data source pins and fetch script

Status: implemented

English | [中文](2026-09-28-dictionary-source-pins-and-fetch-script.zh.md)

## Problem

M6 (dictionary system) adds more than ten data sources — jieba, THUOCL,
official word lists, wordfreq, MDN glossary — on top of the three first-phase
sources. Each source needs a locked URL, SHA-256, license and fetch procedure
so that builds stay reproducible and license records stay accurate. Until now
the sources were registered only by hand inside `docs/数据清单.md`; there was
no machine-readable record and no script that could download and verify a
source. Hand-maintained records drift (URLs, versions, hashes), and R4's data
source decisions made the three-source manual approach untenable.

## Decision

`data/pins/*.json` is the machine-readable lock for every data source entering
the dictionary pipeline. Each pin carries `id`, `name`, `role`, `license`,
`license_note`, and either:

- `kind: "url"` — a downloadable file: `url`, `cache_file` (or `cache_rel` for
  files that already live under `data/raw/`), plus `sha256`/`size`/`fetched_at`
  that `scripts/fetch-sources.ps1 -WritePins` backfills on first fetch; or
- `kind: "snapshot"` — a small committed artifact whose hash locks the build
  input, used for unstable or paginated sources. The MDN zh-CN glossary slug
  list is committed as `data/pins/mdn-glossary-zh.snapshot.json` (601 slugs,
  captured 2026-09-28 from the GitHub contents API of `mdn/translated-content`);
  M6-P fetches the individual page bodies per slug.

`scripts/fetch-sources.ps1` downloads missing files into `data/cache/`
(gitignored), verifies each against its pinned SHA-256, and treats any hash
mismatch as a hard failure that keeps the previous cache — never a silent
degradation. `-WritePins` backfills `sha256`/`size`/`fetched_at` on first run
so the lock is established from the actual download; afterwards the hash pins
the version and any source-side change fails loudly until a human reviews and
re-locks the pin. `-Force` re-downloads even when the cache matches. `-DryRun`
reports the planned action for every pin without touching the network. The
script targets Windows PowerShell 5.1 and carries the UTF-8 BOM required by
the portable-scripts convention.

Legacy pins (`cedict`, `frequencywords-zh`, `globalvoices-zhs`) reference the
existing `data/raw/` files with their recorded hashes, so the script verifies
them without re-downloading; M6-P migrates new fetches to `data/cache/`.

## Alternatives considered

**Keep registering sources by hand in `docs/数据清单.md`.** Rejected: it is
already drifting with three sources; ten-plus sources with per-version hashes
cannot stay accurate by hand, and there is no executable way to prove a
recorded hash still describes the downloaded file.

**Commit all raw data into git.** Rejected: the repo rule "large files and
build artifacts never enter git" exists precisely because dictionary sources
are tens of megabytes (the wordfreq wheel alone is 57 MB); only pins plus a
tiny committed snapshot enter the tree.

**Fetch sources in CI (GitHub Actions) instead of a local script.** Rejected:
the project has no CI-facing fetch path so far and builds run locally; a
local PowerShell script keeps the workflow identical for maintainers and for
the VM acceptance path (skills are adapted to Rust/PowerShell, no Node tooling).

**Pin the MDN glossary via the site sitemap.** Rejected: `https://developer
.mozilla.org/zh-CN/sitemap.xml` returns 404. The GitHub contents API of
`mdn/translated-content/files/zh-cn/glossary` (601 items, two paginated
requests) provides the same slug set; the slug list is small enough to commit.

## Consequences

- Benefits: one-command reproducible download and verification; pinned
  versions; explicit failure on hash drift; snapshot commits make unstable
  paginated sources build-stable.
- Costs: refreshing a pin after the upstream changes is a manual,
  hash-reviewable step; snapshot kind needs a manual refresh (low frequency,
  MDN glossary pages change rarely).
- Relationship to the first-phase pipeline: `zhu-ye-dict import` (see
  [real dictionary import](../../architecture/2026-09-21-real-dictionary-import.md))
  stays unchanged; M6-P's `zhu-ye-dict` build subcommands consume
  `data/cache/` and extend the pipeline with pins as the source of truth for
  `docs/数据清单.md` registration.
- The BOM requirement follows the
  [portable-scripts PS 5.1 encoding fix](../../bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md).
