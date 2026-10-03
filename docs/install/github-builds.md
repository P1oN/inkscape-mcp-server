# Download a GitHub build for Apple Silicon

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

Install Inkscape first. The package includes Rust server, private Python/inkex helpers,
prebuilt bridge, D-Bus dependencies and matching debug symbols. Users do not install a
compiler, Python, uv/pip or Homebrew. Configure the MCP client with the absolute path to
`run-mcp.sh`. Setup can save an optional Sentry DSN/environment privately; CI receives
no DSN or management token, and local settings are not packaged.

Downloadable installable artifacts are uploaded only after all required checks succeed.
A separate `build-evidence-aarch64-apple-darwin` artifact retains diagnostic results even
when a run fails. Successful packages have a 30-day retention period; rerun the workflow
if a download expires. These are Actions artifacts, not Git-tracked binaries or published
GitHub Releases. A GitHub login may be required to download an artifact.

The Apple Silicon job runs on `macos-15`, verifies the native compiler architecture,
uses Cargo.lock and pinned Rust/private runtime dependencies, and checks Rust, helpers,
exact MCP discovery, security/transactions, hidden DSN setup, doctor, real CLI rendering
and cold/warm installs from the archive. It also exercises the source setup recipe.
CI does **not** perform native GUI Undo/Redo acceptance. Libraries are built on the CI
runner; older OS/foreign ABI compatibility is not inferred. macOS packages use ad-hoc
signing; Developer ID/notarization is not supplied. Existing local native evidence remains
scoped to its recorded binary, and no macOS security setting is bypassed.
