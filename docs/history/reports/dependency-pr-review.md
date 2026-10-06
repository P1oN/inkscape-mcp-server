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
