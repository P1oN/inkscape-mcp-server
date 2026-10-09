# Download a GitHub build for Apple Silicon

[Release v0.1.2](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.2)
contains `inkscape-mcp-macos-arm64.tar.gz` (ready runtime) and
`inkscape-mcp-source-bootstrap.tar.gz` (source installer), each with a `.sha256` file.
Both include the current installation/client/skill management, five Rust executables,
reviewed live tools and vector-quality guards. This remains a prerelease for Apple Silicon
macOS 15+ with Inkscape 1.4+; Intel/Linux/Windows and clean-machine/native artist acceptance
retain their separate scope.

For the ready runtime, download its archive and checksum into one directory:

```sh
shasum -a 256 -c inkscape-mcp-macos-arm64.tar.gz.sha256
tar -xzf inkscape-mcp-macos-arm64.tar.gz
cd inkscape-mcp-macos-arm64
./setup.sh --install-skill codex --connect-client codex
./run-mcp.sh --doctor
```

Use `claude` for an installed Claude Code CLI. Keep the extracted directory in a permanent
location; the client registers its absolute launcher. No compiler, Python or Homebrew is
needed for the ready runtime. For the source archive:

```sh
shasum -a 256 -c inkscape-mcp-source-bootstrap.tar.gz.sha256
tar -xzf inkscape-mcp-source-bootstrap.tar.gz
cd inkscape-mcp-source-bootstrap
./setup.sh --install-skill codex --connect-client codex
./run-mcp.sh --doctor
```

Source setup builds automatically and prepares missing pinned build inputs privately.
Apple Command Line Tools are required; complete Apple's installation dialog if absent.
`--rebuild` explicitly repeats preparation/build, while `--local-tools` uses existing tools
and cached inputs offline. See [source prerequisites](local-bootstrap.md).

Inspect `./setup.sh --version` for actual compiled revision/build identity; a distribution
tag does not replace those fields. Ready-package setup defaults to saving configuration;
`--connect-client` checks/registers the client and `--check` runs diagnostics. Startup and
reconnect never launch Inkscape GUI. The release does not change your configured runtime
until you install/select it and reconnect.

Historical [v0.1.1](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.1)
is a source preview requiring `./setup.sh --bootstrap`; it retained private Python and lacks
current client/skill management. Historical [v0.1.0](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.0)
contains its original ready package. These archives and their acceptance evidence remain unchanged.

The [Rust checks and native packages workflow](https://github.com/P1oN/inkscape-mcp-server/actions/workflows/rust-migration.yml)
automatically builds **macOS arm64 (M chips)** after every push to `main` and for
pull requests. It can also be started manually with **Run workflow → macos-arm64**.
Manual `all-posix` additionally selects the prepared Intel Mac and Linux jobs; their
existence does not prove compatibility before they have passed on their own runners.

Open a successful run and download **Artifacts → inkscape-mcp-macos-arm64**. Unzip the
artifact: it contains `inkscape-mcp-macos-arm64.tar.gz` and its `.sha256` file. With both
files in the same directory:

```sh
shasum -a 256 -c inkscape-mcp-macos-arm64.tar.gz.sha256
tar -xzf inkscape-mcp-macos-arm64.tar.gz
cd inkscape-mcp-macos-arm64
./setup.sh
./run-mcp.sh --doctor
```

For ready packages using the updated launcher, setup without build/client options saves
private configuration; diagnostics are explicit with `./setup.sh --check` or `./run-mcp.sh --doctor`. Source rebuilding
with installed tools/cached dependencies is explicit with `./setup.sh --local-tools`.
Fresh current source checkouts automatically prepare tools and build with `./setup.sh`.
The historical v0.1.0 archive still runs doctor during setup; use its included instructions.

Install Inkscape first. Published v0.1.2 packages include five Rust executables,
prebuilt bridge, D-Bus dependencies and matching debug symbols. Users do not install a
compiler, Python, uv/pip or Homebrew. Current packages can register a client with `--connect-client codex|claude`;
older assets require manual configuration of the absolute `run-mcp.sh` path. Setup can save an optional Sentry DSN/environment privately; CI receives
no DSN or management token, and local settings are not packaged.

Downloadable installable artifacts are uploaded only after all required checks succeed.
A separate `build-evidence-aarch64-apple-darwin` artifact retains diagnostic results even
when a run fails. Successful packages have a 30-day retention period; rerun the workflow
if a download expires. Actions artifacts are separate from the published Release assets above and are not
Git-tracked binaries. A GitHub login may be required to download an Actions artifact.

The Apple Silicon job runs on `macos-15`, verifies the native compiler architecture,
uses Cargo.lock and pinned Rust/private runtime dependencies, and checks Rust, helpers,
exact MCP discovery, security/transactions, hidden DSN setup, doctor, real CLI rendering
and cold/warm installs from the archive. It also exercises the source setup recipe.
CI does **not** perform native GUI Undo/Redo acceptance. Libraries are built on the CI
runner; older OS/foreign ABI compatibility is not inferred. macOS packages use ad-hoc
signing; Developer ID/notarization is not supplied. Existing local native evidence remains
scoped to its recorded binary, and no macOS security setting is bypassed.

## Publishing a new distribution

The `Publish verified release` workflow is dispatched from `main` with a new tag, a successful
`main` push run ID of `Rust checks and native packages`, and the prerelease choice. It downloads
the existing `inkscape-mcp-macos-arm64` artifact, verifies both archive checksums and their exact
source revision, and publishes ready/source archives with `RELEASE-METADATA.json`. No archive
code is executed and no binaries are rebuilt during publication. Older CI artifacts lacking the
source archive cannot be published with this workflow; run current main CI first.

A draft is created first, all ten current assets are uploaded, then publication makes the new release
immutable. On publication failure, inspect the retained draft before retrying; the workflow refuses
an existing tag/release rather than replacing it. Existing historical releases remain unchanged.
Release tags matching `v*` cannot be force-updated or deleted. Distribution tags remain separate
from the runtime crate version. Choose a new distribution tag for each set of release assets.

The native workflow runs on every PR and main push so required checks never wait forever because
of path filters. `Quick Rust checks` performs syntax, fmt/Clippy, tooling and release-guard tests
before `Native candidate (aarch64-apple-darwin)` starts runtime/package acceptance. Both checks
are required by the main ruleset, which also requires a PR, resolved review threads, and current
base-branch validation. Required approving reviews remain zero for the current solo maintainer;
there is no administrator bypass. Dependabot opens weekly Cargo and Actions updates through this
same PR/CI process. Issues use structured bug/feature forms; security reports remain private.

Current sources also support [independent instructions/runtime updates](independent-updates.md) through a
permanent native launcher and macOS management app. Opt in once with
`./setup.sh --independent-updates`; published v0.1.2 archives retain their original behavior.

## Signed Manager preparation (unpublished checkpoint)

The four-stage workflow in `.github/workflows/signed-release.yml` validates exact successful
main CI artifacts, transforms them under protected `release-signing`, verifies final native
package/installer/distinct-update receipts without credentials, and publishes exact assets
under protected `release-publication`. The checkout used for packaging is the candidate
revision. Pinned Actions, explicit ephemeral keychain cleanup, Accepted tickets and remote
asset digest/count checks gate publication. A failed existing draft requires explicit
inspection/recovery; it is never overwritten. Legacy publication remains available for
advanced historical assets. Signed immutable runtime reuse preserves original archive
bytes and reference/build identities. See [the contract](macos-manager-distribution.md)
and [scoped evidence](../history/reports/macos-installer-2026-10-08.md). No protected
environments or signing secrets had been configured at that checkpoint. As of
2026-10-09, both environments restrict deployment to `main` with P1oN review, and the
required secret names/variables are present. A real protected signing run is still
needed to validate those credentials and qualify the distribution.
