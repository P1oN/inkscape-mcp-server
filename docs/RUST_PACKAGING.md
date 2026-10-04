# Historical packaging checkpoint — stage38

This page preserves the named checkpoint and its evidence. Its package paths, counts,
validation and publication statements are historical, not the status of PR #8. For current
work use [the handoff](AGENT_HANDOFF.md), [the plan](RUST_NEXT_PLAN.md) and
[installation](install/install.md). Native results remain bound to the recorded binaries.

Recorded candidate: **stage38, macOS arm64**, tested locally on this Mac. See
[Rust migration report](RUST_MIGRATION_REPORT.md) for exact install/config commands,
archive SHA-256 and compatibility limits. The authoritative build/check index is
[migration/package-stage38-build-comparison.json](../migration/package-stage38-build-comparison.json).

The ready archive includes the Rust MCP executable, prebuilt Objective-C context bridge,
private CPython/inkex helper runtime, fixed supervisor, D-Bus/gdbus and their native library
closure. It requires Inkscape; user-installed Python, uv/pip, Homebrew, compiler or Rust
are unnecessary. The checkout workflow is `./setup.sh` then `./run-mcp.sh`; a source build
requires development tools. Setup asks for configuration rather than requiring env edits.

Actual archive installations in fresh temporary directories pass with empty PATH in both
per_call and shell engine modes. Checks include private runtime imports, both effect helper
CLIs, private bus exchange, STDIO discovery, transactions, original preservation, snapshots,
rollback, strict no-op, real PNG pixels, export and artifact resource readback. Doctor has
10 tested profiles, launcher 11 checks, notices 17 checks; all 2533 FILES entries match.
Current native acceptance and headless/live measurements pass on the unchanged archive.
The immutable build-time index records earlier status; the later
[final acceptance overlay](../migration/package-stage38-final-comparison.json) binds the
current results. Final documentation is external to the unchanged archive.

## Attribution and sources

The package retains crate notices (102 crates), standard-library notices, 19 distribution
license texts from the exact CPython release and original wheel license metadata. Provenance
checks match 946 regular runtime files plus explicitly verified installation transformations,
and 1259 members of six exact wheels. GLib/D-Bus/gettext/PCRE2 sources match installed SBOM
URL/version/hash. Exact GLib recipe/patch and Homebrew BSD2 license are supplied.

Companion source kit: `migration/results/homebrew-source-audit/native-source-kit-stage37.tar.gz`.
Its SHA-256 is `5e8d6f0da837a3990553cd880f7e4d12cf6f7b9d1176de1fbd60ec6952d00ec2`;
[source-kit-stage37.json](../migration/results/homebrew-source-audit/source-kit-stage37.json)
binds its 24 inputs. It supplies native source/build inputs; it does not establish a
reproduced whole binary. LICENSE-INVENTORY preserves broader redistribution/foreign-target
questions. No legal certification is claimed.

Only macOS arm64 has actual local execution evidence. Prepared POSIX target jobs are not
completed remote checks. Windows is backlog; clean-machine installation is user-deferred.
Local ad-hoc signatures are supplied. Developer ID/notarization is unavailable; no macOS
security setting was bypassed and signing credentials are not a blocker to this workflow.

Earlier packaging checkpoints and failures are retained in
[historical packaging log](history/RUST_PACKAGING_LOG.md); obsolete current labels there
are historical, and retired Python oracle commands must not be rerun.
