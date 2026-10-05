# Platform compatibility and validation scope

v0.1.2 includes installation/responsiveness, complete Python removal, editable-vector
guidance, computed live inspection/reviewed packages and vector-quality guards (PRs #8–11).
[PR #11 CI](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37372260969)
passed on macOS arm64, including default-parallel native tests and package/CLI acceptance.
Its first attempt could not acquire a hosted runner and ran no steps; the second passed.
Earlier local concurrency failures remain recorded, rather than erased by this pass.
Local acceptance uses official Inkscape 1.4.3 and pinned Rust 1.99.0.
See [current status](../AGENT_HANDOFF.md) for release-specific source/build identity.

| Area | Evidence and remaining scope |
|---|---|
| Apple Silicon source provisioning | macOS 15+ recipe; existing-host archive acceptance, not a clean machine |
| Codex | Real CLI registration in an isolated profile plus MCP handshake; no model session claim |
| Claude Code | Synthetic CLI/config guards; real installed client acceptance remains pending |
| Native GUI | Owned synthetic live style/text package Undo/Redo evidence is scoped to its recorded build; real artwork/artist/human-review race acceptance remains deferred |
| macOS Intel/Linux | Native CI jobs are prepared; execution/compatibility requires their own successful runs |
| Windows | Native filesystem/process/runtime port remains backlog |
| Signing | Ad-hoc macOS signatures; no Developer ID/notarization or security-setting bypass |

Recorded archives and their GUI/performance evidence are historical and remain bound to
those bytes. Later headless passes do not transfer native results to a new binary.
See [current status](../AGENT_HANDOFF.md), [installation](install.md) and
[historical migration report](../history/reports/RUST_MIGRATION_REPORT.md).
