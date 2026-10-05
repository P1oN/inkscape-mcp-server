# Platform compatibility and validation scope

Current `main` includes PR #8 installation/responsiveness and PR #9 Python removal/authoring.
The [PR #9 candidate CI](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37296446838)
passed on macOS arm64. Its separate post-merge
[main CI](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37298327949) failed in a
managed effect-loss regression; the test budget is [repaired locally](../history/reports/ci-effect-timeout-fix.md),
with fresh remote confirmation pending. Local acceptance uses official
Inkscape 1.4.3 and pinned Rust 1.99.0. See [current status](../AGENT_HANDOFF.md).

| Area | Evidence and remaining scope |
|---|---|
| Apple Silicon source provisioning | macOS 15+ recipe; existing-host archive acceptance, not a clean machine |
| Codex | Real CLI registration in an isolated profile plus MCP handshake; no model session claim |
| Claude Code | Synthetic CLI/config guards; real installed client acceptance remains pending |
| Native GUI | Historical stage38 and Python-removal owned native checks are scoped to their builds; authoring does not repeat GUI acceptance |
| macOS Intel/Linux | Native CI jobs are prepared; execution/compatibility requires their own successful runs |
| Windows | Native filesystem/process/runtime port remains backlog |
| Signing | Ad-hoc macOS signatures; no Developer ID/notarization or security-setting bypass |

Recorded archives and their GUI/performance evidence are historical and remain bound to
those bytes. Later headless passes do not transfer native results to a new binary.
See [current status](../AGENT_HANDOFF.md), [installation](install.md) and
[historical migration report](../history/reports/RUST_MIGRATION_REPORT.md).
