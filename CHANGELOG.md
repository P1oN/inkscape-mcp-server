# Changelog

Distribution tags describe published previews; source/build identity remains recorded in each package.
Detailed checkpoint evidence is indexed in [history](docs/history/README.md).

## v0.1.2 — 2026-10-06, prerelease

Source archive and ready macOS Apple Silicon package, with checksums.
[Release](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.2).

- Omit Git global PAX metadata from committed source exports, retaining explicit source
  identity and compatibility with strict archive extraction.
- Add bounded path parsing, explicit closure/zero-length guards, read-only node/closure
  findings, whole-document vector inventory and vector-only save refusal (PR #11).
- Add structural fragment dry-run candidates; application retains existing approval/reference gates (PR #11).
- Add computed live inspection and reviewed style/transform/text packages with content/selection guards (PR #10).
- Preserve scoped synthetic GUI evidence, explicit unknowns and conservative per-call Undo verification.


- Replace project-supplied Python runtime/helper routes with five Rust executables;
  ready packages omit CPython/wheels/inkex helpers and active tooling uses Rust/Bash (PR #9).
- Add shared editable-vector authoring guidance and bounded, read-only explicit stroke-role,
  CSS/subpath and hidden-geometry advice; no automatic cleanup (PR #9).
- Improve source installation, client registration, managed skill updates, build identity,
  uninstall/reinstall, responsiveness and owned cancellation (PR #8 and deadline follow-up).
- Preserve existing working-copy edit guarantees and bounded managed live operations.

## v0.1.1 — 2026-10-03, prerelease

Published source-bootstrap preview for automatic local installation on Apple Silicon.
It predates the current default setup/client management and complete Python removal.
[Release](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.1).

## v0.1.0 — 2026-10-03, prerelease

Published Rust MCP preview with a ready Apple Silicon package.
Its runtime includes historical Python helper dependencies.
[Release](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.0).

The inherited changelog and original feature list are
[archived](docs/history/original-changelog.md); they do not describe these release assets.
