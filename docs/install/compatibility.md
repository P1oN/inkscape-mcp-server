# Platform compatibility and validation scope

The current improvements are in [PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8).
Local validation on 2026-10-04 used macOS arm64, official Inkscape 1.4.3 and pinned
Rust 1.99.0 with native Rust/Bash build and runtime tooling. Extracted-source setup, isolated real Codex registration,
skill merge/conflicts, SVG edit/render/save, uninstall/reinstall and relocated ready archives
passed. Both `per_call` and `shell` engines passed with empty PATH/private runtime/bus;
owned process cancellation and responsive discovery/workspace reads were checked.

| Area | Evidence and remaining scope |
|---|---|
| Apple Silicon source provisioning | macOS 15+ recipe; existing-host archive acceptance, not a clean machine |
| Codex | Real CLI registration in an isolated profile plus MCP handshake; no model session claim |
| Claude Code | Synthetic CLI/config guards; real installed client acceptance remains pending |
| Native GUI | Historical stage38: 122 fixed checks, 5 two-window guards and closed-session reconnect; not rerun for this PR |
| macOS Intel/Linux | Native CI jobs are prepared; execution/compatibility requires their own successful runs |
| Windows | Native filesystem/process/runtime port remains backlog |
| Signing | Ad-hoc macOS signatures; no Developer ID/notarization or security-setting bypass |

Stage38 archives and their GUI/performance evidence are historical and remain bound to
those bytes. Later headless passes do not transfer native results to a new binary.
See [current status](../AGENT_HANDOFF.md), [installation](install.md) and
[historical migration report](../RUST_MIGRATION_REPORT.md).
