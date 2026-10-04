# Download a GitHub build for Apple Silicon

As of 2026-10-04, published previews are v0.1.0 and v0.1.1. New installation/client/skill/
build identity/removal features are in [PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8),
not these Release assets. Follow the PR checks for its remote build status and download
only a successful run. [Current installation](install.md) distinguishes each distribution.

For local installation on Apple Silicon macOS 15+, open
[Release v0.1.1](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.1)
and download `inkscape-mcp-source-bootstrap.tar.gz` and its `.sha256` file from **Assets**.
With both files in the same directory:

```sh
shasum -a 256 -c inkscape-mcp-source-bootstrap.tar.gz.sha256
tar -xzf inkscape-mcp-source-bootstrap.tar.gz
cd inkscape-mcp-source-bootstrap
./setup.sh --bootstrap
./run-mcp.sh --doctor
```

Install Inkscape first. Setup builds locally, downloads missing build tools into a private
temporary directory and removes those tools afterward. The packaged helper Python stays
available for runtime use. If Apple Command Line Tools are missing, complete their installation
dialog and rerun setup. See [local bootstrap](local-bootstrap.md) for prerequisites and limits.
The exact Git-free release archive passed a fresh-tools build, temporary-tool cleanup,
license-notice checks and empty-PATH MCP/D-Bus/render/export/transaction acceptance on the
development Mac. This is local automated acceptance; native GUI and clean-machine Apple
tool installation were not revalidated.

The command above is for published v0.1.1. Current sources instead use `./setup.sh`
for automatic preparation/build on first use and reuse the saved package on subsequent
runs. `--local-tools` selects a rebuild with existing tools/cached dependencies only;
`--rebuild` explicitly repeats automatic preparation/build; `--bootstrap` remains compatible. These later changes are not in the v0.1.1 asset.

The earlier [Release v0.1.0](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.0)
keeps the prebuilt archive described below. Both releases are marked prerelease.

The [Rust checks and native packages workflow](https://github.com/P1oN/inkscape-mcp-server/actions/workflows/rust-migration.yml)
automatically builds **macOS arm64 (M chips)** after relevant pushes to `main` and for
pull requests. It can also be started manually with **Run workflow → macos-arm64**.
Manual `all-posix` additionally selects the prepared Intel Mac and Linux jobs; their
existence does not prove compatibility before they have passed on their own runners.

First verified Apple Silicon build: [successful run 37154851232](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37154851232),
source `5345f619add78004d3ecc9ca83abd5a6205788db`. Its downloaded archive also
passed installation and real CLI acceptance on the development Mac.

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
The historical v0.1.0 archive still runs doctor
during setup; these launcher changes are not in that published archive.

Install Inkscape first. The package includes Rust server, private Python/inkex helpers,
prebuilt bridge, D-Bus dependencies and matching debug symbols. Users do not install a
compiler, Python, uv/pip or Homebrew. Packages built from PR #8 can register a client with `--connect-client codex|claude`;
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
