# Agent Note: Dictionary update trust chain and the updater process

Status: implemented

[中文](2026-09-29-dictionary-update-trust-chain.zh.md) | English

## Problem

M6-P shipped a manifest and M6-R made several packs loadable at runtime, but
nothing could deliver a pack to a user's machine. FR-020 asks for discovery,
download, integrity verification, atomic replacement, and rollback, under a
hard constraint: online updates are an online feature and must be **off by
default** (P-03, NFR-010).

Two properties made this more than a file-copy problem:

1. **The dictionary packages are the attack surface.** A pack is mmap'd and
   queried on every keystroke. Accepting a tampered pack means an attacker
   controls candidate text. Content hashes alone are not a trust anchor —
   whoever can rewrite a pack can rewrite the hash next to it.
2. **The TSF DLL must never touch the network.** It runs inside host processes
   (notepad, browsers, IDEs). A blocking or failing TLS handshake there can hang
   the host, trip antivirus heuristics, or stall candidate generation.

## Decision

### Trust anchor: a compiled-in public key

The manifest carries an ed25519 signature. Crucially, verification uses a public
key **compiled into the client**, not the `public_key` field embedded in the
manifest:

- `verify_signature_with_key(manifest, trusted_key)` is the verification entry
  point used by both the updater and `zhu-ye-dict verify-signature`.
- `verify_signature(manifest)` (self-contained, uses the embedded key) exists for
  auditing and diagnostics only.

The embedded key is still written into the manifest so a publisher can rotate
keys and so operators can compare what was published against what the client
trusts. A test signs a manifest with an attacker key, confirms the self-contained
check passes (the forgery is internally consistent), and asserts the compiled-in
key rejects it. Without that test the design would be one line away from trusting
attacker-supplied key material.

### Signature covers canonical JSON, not file bytes

`canonical_bytes` serializes the manifest with `signature: None` and compact
formatting. Signing raw file bytes would break on any reformatting, and signing
the manifest *including* its signature field is self-referential. Excluding the
signature field makes signing idempotent and format-insensitive; a test asserts
the canonical bytes are byte-identical before and after attaching a signature.

### Failure atomicity

`update.rs` implements: verify signature → verify each pack's SHA-256 and size →
write to `packs/staging/` → replace each target atomically (write a sibling
`*.zyct.new`, then `rename`) → remove staging. The previous pack is copied to
`.bak` first, and `rollback_pack` restores it.

`rename` within one volume is atomic, so a crash mid-apply leaves either the old
pack or the new one — never a truncated file. Any failure leaves the old pack in
place and reports the reason; packs are applied independently, so one bad pack
does not block the others. A pack whose installed hash already matches is skipped
without a download or a backup.

### Only distributable packs participate

`DISTRIBUTABLE_PACK_IDS = ["it", "med", "slang"]` gates what the updater will
fetch. The base pack ships read-only with the installer, and `real`/`seed` are
development artifacts that are never published. This was not a theoretical
concern: the first end-to-end run against a real manifest containing all six packs
tried to download `base.zyct` and failed with HTTP 404.

Consequently `verify_release` validates **only the packs actually present in the
download set**, not every pack listed in the manifest. The signature is still
verified against the complete, unmodified manifest — an earlier attempt to filter
the manifest before calling `apply_release` broke verification, because trimming
the pack list changes the canonical bytes.

### Separate process, default off

`zhu-ye-updater` is a standalone binary and the only component that performs
network I/O (S-4). It exits after one operation; nothing resident remains.

The default-off guarantee is enforced at the config layer: with
`online_update: false`, `check` and `apply` return before constructing any
request. Verified by pointing the manifest URL at an unreachable address — the
command still exits successfully without a network attempt, which would be
impossible if a connection were being made.

Downloads shell out to the system `curl.exe` (present on Windows 10+) rather than
linking an HTTP/TLS stack. This keeps the updater's dependency surface to
`ed25519-dalek` alone.

### Key handling

The signing tool reads the private key from `ZHU_YE_RELEASE_SECRET_KEY` and never
accepts it as a command-line argument, which would expose it in the process list
and shell history. The client's public key is injected at build time via
`ZHU_YE_RELEASE_PUBLIC_KEY`. When no key is compiled in, the updater refuses to
apply anything rather than falling back to an unverified path.

## Alternatives considered

**Trust the manifest's embedded public key.** Rejected: an attacker who can serve
a manifest can serve their own key and signature, making the signature decorative.
The compiled-in key is what makes tampering detectable.

**Sign the manifest file bytes.** Rejected: it couples signature validity to JSON
formatting, so re-serializing the manifest during any tooling step would
invalidate a legitimate signature.

**Verify content hashes only, no signature.** Rejected: hashes in the same
document as the payload provide no protection against an attacker who controls
the download.

**Download inside the TSF DLL.** Rejected explicitly by S-4: blocking network work
in a host process risks hanging the host and stalling input, and the DLL has no
business holding TLS state.

**Vendor an HTTP client into the updater.** Rejected: `curl.exe` is already present
on every supported Windows version, and avoiding a TLS stack keeps the updater's
dependency and audit surface minimal.

**Apply packs one at a time and stop at the first failure.** Rejected: packs are
independently versioned and independently useful; a failure on one should not
block the others, and each failure already leaves its own pack intact.

**Write the target file directly instead of staging + rename.** Rejected: a crash
or power loss mid-write would leave a truncated `.zyct`, which the loader would
then reject at startup — the user would lose a working pack to a failed update.

## Consequences

- Updates are authenticated end to end: signature over canonical JSON plus a
  per-pack content hash. Both must pass before a byte reaches the packs directory.
- Cost: rotating the release key requires shipping a new engine build, since the
  trust anchor is compiled in. The manifest carries `min_engine_version` so an old
  client can be told to skip packs it cannot verify.
- Cost: pack distribution is a hardcoded list. Adding a new distributable pack
  requires a code change and a test update rather than only a manifest edit —
  deliberate, so publishing a new pack is an explicit decision.
- Cost: the updater depends on `curl.exe`. On a system where it is missing or
  blocked, updates fail loudly; input is unaffected.
- `zhu-ye-dict`'s manifest types now come from `zhu-ye-core`, so the build side and
  the runtime side cannot drift apart on the wire format.
- `ed25519-dalek` (BSD-3-Clause) is registered in `docs/licenses.md` under a new
  code-dependency section, since it is the first cryptographic dependency.
- Tests: 13 manifest and 12 update unit tests, plus two updater default-off
  assertions; workspace total is 317. Nine acceptance scenarios were exercised
  end to end against a local static server and an isolated `APPDATA` sandbox:
  normal update, repeat-apply no-op, tampered installed pack recovery, signature
  tampering rejected, truncated pack rejected, unreachable manifest, version gate,
  default-off with no network, and no resident process.
