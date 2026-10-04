# Local source installation on Apple Silicon

For the sources in [PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8),
on Apple Silicon macOS 15+, run:

```sh
./setup.sh
./run-mcp.sh --doctor
```

Install Inkscape 1.4+ first. The installer asks for the SVG workspace and optional
Sentry settings, builds a local Rust server and saves private data-only configuration.
MCP clients use the absolute path to the checkout's `run-mcp.sh`.
Keep the checkout and its `.inkscape-mcp-local` folder in a permanent location.
Subsequent runs reuse a complete saved package when source revisions match; revision
changes trigger rebuilding. Explicit build/check/skill/client options perform their requested work.
Diagnostics are explicit; `--check` runs doctor before saving configuration.

Reruns preserve the saved workspace, Inkscape path, live setting and engine unless an
explicit option replaces them. Invalid settings/configuration paths are refused before
building. In a source checkout, setup compares the current committed revision (or source
archive marker) with the installed package's recorded revision. A mismatch triggers an
automatic rebuild; a failed build preserves saved runtime settings. An explicit `--package`
selects that package even if revisions differ. Unknown revisions or uncommitted source
edits produce a rebuild instruction; use `--rebuild` to include those changes explicitly.

`./setup.sh --local-tools` explicitly rebuilds using installed tools and cached Cargo
dependencies only. It does not download Rust/Python/uv/native inputs or dependencies;
missing prerequisites cause failure. A pinned `.venv` or `.packaging-venv` helper
environment must already exist. `--build` remains a compatibility alias for this mode.
`--rebuild` explicitly repeats automatic preparation and build, even with an existing
configuration; `--bootstrap` is its compatibility alias. Build modes cannot be combined
with each other or with `--package`.

For a Git-free installation, download `inkscape-mcp-source-bootstrap.tar.gz` from the
Release, verify its checksum and unpack it into a permanent location. The published
v0.1.1 archive predates automatic setup by default: use `./setup.sh --bootstrap` on first
use with that version. The current source tree contains the updated default behavior.

Automatic bootstrap currently supports **Apple Silicon, macOS 15 or newer**.
The published v0.1.0 ready-binary archive does not contain this installer. Use the source
bootstrap archive for its original installer, or the PR sources for the updated workflow.
Committed source exports include SOURCE_REVISION metadata. Explicit `--working-tree`
acceptance snapshots omit it and record an unknown revision; their build ID still identifies
the source content. Ordinary GitHub source archives without this metadata also build,
recording an unknown source commit rather than failing.

The build uses Apple Xcode or Command Line Tools. If these are missing, the installer
opens Apple's installation dialog and stops with a rerun instruction; it does not accept
Apple's license or administrator authorization on the user's behalf. These shared system
tools remain installed. Inkscape also remains installed.

An exact existing Rust 1.99.0 toolchain or pinned helper environment can be reused without
upgrading or uninstalling it. Otherwise, rustup 1.28.2 and uv 0.12.22 are downloaded with
pinned SHA-256 checksums. Rustup installs a minimal Rust 1.99.0 toolchain with private
CARGO_HOME/RUSTUP_HOME and `--no-modify-path`. uv prepares Python 3.12.14 and the six
pinned helper wheels under private install/cache directories, without global Python links.
No shell profile or system package manager is installed or changed.

Six native Homebrew bottles are pinned by URL/hash in
`rust/package/bootstrap-native-macos-arm64.json`. They are extracted into the temporary
build tree; Homebrew itself is not installed. The bridge is compiled locally. The package
builder resolves bottle placeholders against that tree, relocates the required D-Bus/GLib
closure into the final package and records native input hashes/SBOMs/license texts.
Temporary roots cannot remain as linked runtime dependencies.

Only the installer's own `bootstrap.*` mktemp directory is removed on success, failure
or handled interruption. This removes downloaded development Rust/uv/Python, native inputs,
Cargo/uv caches and compilation intermediates. Existing tools and prior installed packages
are preserved. Failed builds may leave a partial `build.*` package for diagnosis; saved
configuration is changed only after a successful build. A hard kill or power loss can leave
a bootstrap directory; never delete unrelated tool installations to clean it up.

**Private Python inside the installed package remains necessary** for bounded live
helpers and supervision. It is copied before cleanup and works without the development
runtime. Neither Python nor Rust needs to be registered in the user's shell PATH.

This is local source installation, not Developer ID signing/notarization. Bootstrap does
not remove quarantine attributes or disable Gatekeeper/TCC. It downloads pinned build
inputs over HTTPS using ordinary command-line tooling. Actual OS prompts depend on the
host's security/privacy settings; at most one prompt is not guaranteed. Native GUI, Undo/Redo
and clean-machine Apple tool installation are separate acceptance checks.

Upstream mechanisms: [rustup isolated homes](https://rust-lang.github.io/rustup/installation/index.html),
[uv storage](https://docs.astral.sh/uv/reference/storage/), and
[Homebrew formula metadata](https://formulae.brew.sh/api/formula/glib.json).

Current sources also support client registration, installed build identity, managed skill
updates and clean reinstall. See [client management](client-management.md).
