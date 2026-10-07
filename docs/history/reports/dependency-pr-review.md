# Dependency PR review — 2026-10-06

Reviewed open PRs #13 and #15–20 using their diffs, existing bot comments, required CI failures,
upstream changelogs and repository call sites. CodeRabbit skipped these bot-authored updates;
its successful status was not treated as a completed review.

- #13: declined. It changes the read-only retired Python environment; current packages do not
  read or ship those historical inputs. No Python install/parity validation was performed.
- #19/#20: declined. sha2 0.11 changes digest output types and removes LowerHex support;
  CI reports E0277 in the runtime and tooling hash call sites. A coordinated migration with
  hexadecimal format invariants belongs to the unfinished-work backlog, not an automatic bump.
- #15: libc/serde/serde_json patch changes are suitable, but libc/serde_json pins conflict with
  the tooling graph. Align both manifests and refresh both locks.
- #16/#17/#18: Tokio, regex and plist changes are suitable after updating the tooling lock
  (Tokio/plist) or duplicate direct pin (regex). Incorporated into #15 so their final combined
  dependency graph is tested and merged together; original standalone CI failures are retained.

Final selected versions: libc 0.2.190, serde 1.0.229, serde_json 1.0.151, Tokio 1.53.2,
regex 1.13.1, plist 1.10.1. sha2 remains 0.10.9. Plist keeps the crate's declared Rust 1.88
compatibility; no MCP tools, schemas or instructions change, so catalogs need no regeneration.

Tokio 1.53.2 supersedes the proposed 1.53.1 update with upstream fixes for blocking-pool
shutdown and synchronization lock handling. The final patch version is validated below.

Review sources: [Tokio changelog](https://github.com/tokio-rs/tokio/blob/tokio-1.53.2/tokio/CHANGELOG.md),
[regex changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md),
[plist changelog](https://github.com/ebarnard/rust-plist/blob/master/CHANGELOG.md),
[sha2 changelog](https://github.com/RustCrypto/hashes/blob/master/sha2/CHANGELOG.md).

Both locked Cargo graphs passed fmt and Clippy. Default-parallel runtime tests passed
310 tests with two opt-in tests ignored; tooling passed 17 tests. Required hosted quick/native
CI will validate the final commit before merge. No GUI or installed-user runtime change
is part of this review, and historical native GUI evidence is not transferred to a new binary.

## Coordinated follow-up — 2026-10-07

The user requested one replacement PR for Dependabot PRs #21–25 and closure of the standalone
updates. Both Cargo graphs now select base64 0.23.1, toml 1.1.6+spec-1.1.0 and toml_edit
0.25.15+spec-1.1.0. Both direct shared pins and lockfiles are aligned; unrelated dependency
updates and the separate uncommitted development-cleanup work are excluded.

Standalone runtime bumps failed locked tooling checks because the tooling graph includes
the runtime as a path dependency. The standalone tooling base64 bump passed CI but split
the direct pins. The tooling TOML bump failed the valid crate-notice test: TOML's
`Value::from_str` parses an individual value, not a document, in newer versions.
Both crate-manifest and Cargo.lock readers now use explicit `toml::from_str`; bounded reads,
archive/hash checks and notice extraction guards remain in place. See the
[upstream TOML migration notes](https://github.com/toml-rs/toml/blob/toml-v1.1.6/crates/toml/CHANGELOG.md)
and [base64 release notes](https://github.com/marshallpierce/rust-base64/blob/v0.23.1/RELEASE-NOTES.md).

Local validation: 326 sequential runtime tests passed/two opt-in ignored; 17 tooling tests,
both fmt/Clippy graphs, locked release build and a fresh six-executable native package passed.
The package notice probe passed 11 checks across 199 crates, including the updated Cargo.lock
and actual registry crate manifests. Doctor passed all 12 profiles; isolated client lifecycle
and source-management guards passed with the new package. No MCP schema/instruction changes
require catalog regeneration.
Hosted required CI is pending on the replacement PR; local checks do not establish a new
native GUI result, clean-machine acceptance or installation into the user's runtime.

Local builds used `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` to limit scratch storage,
without including development-profile changes in this dependency PR. Release symbols remain
enabled. The user's existing checkout, settings, running MCP and Inkscape windows are preserved.
