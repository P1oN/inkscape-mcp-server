# Example MCP host configs

These examples launch the current Rust server through its absolute `run-mcp.sh` path.
First run setup from a permanent source/package location; the launcher reads saved
workspace, engine and monitoring settings. Replace the placeholder launcher path.

| File | Host |
| --- | --- |
| [claude_desktop_config.json](claude_desktop_config.json) | Claude Desktop JSON `mcpServers` configuration |
| [mcp.json](mcp.json) | Claude Code project `.mcp.json` or a compatible JSON-based STDIO host |

For native registration and a handshake, use `./setup.sh --connect-client codex|claude`
with the selected client installed. Default setup does not register a client. Print
manual TOML/JSON after setup with `./scripts/mcp-client.sh --client codex|claude config`.

See [host configuration](../docs/install/host-configs.md),
[client management](../docs/install/client-management.md) and [installation](../docs/install/install.md).
Python/uvx/pipx launch entry points are retired. Startup/reconnect never launches Inkscape.
