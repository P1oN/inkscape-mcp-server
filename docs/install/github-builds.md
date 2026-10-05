# Download a GitHub build for Apple Silicon

As of 2026-10-05, published previews remain v0.1.0 and v0.1.1. Current `main` includes
installation/client/skill/build identity improvements and complete Python removal/authoring
from PRs #8/#9; these changes are not in existing Release assets.

The [main run for `c66583b`](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37310025578)
passed after the effect-regression test repair. The earlier failed post-merge run remains
historical. Download installable artifacts only from successful runs and inspect
source/build identity; the newest run is not automatically an accepted package.
[Current installation](install.md) distinguishes each distribution.

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
temporary directory and removes those tools afterward. The historical v0.1.1 package retained private Python for runtime use. If Apple Command Line Tools are missing, complete their installation
dialog and rerun setup. See [local bootstrap](local-bootstrap.md) for prerequisites and limits.
The v0.1.1 bootstrap verification belongs to its original archive; its detailed evidence
is [historical](../history/reports/RUST_MIGRATION_REPORT.md). It does not establish acceptance
of current source or clean-machine Apple tool installation.

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

Install Inkscape first. Current packages include five Rust executables,
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
