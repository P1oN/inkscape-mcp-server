# MCP client configuration

After setup, the client runs the absolute `run-mcp.sh` path. The launcher loads saved local
settings noninteractively; retain the source/package directory in a permanent location.

For current sources/packages:

```sh
./setup.sh --connect-client codex
./setup.sh --connect-client claude
```

Choose the installed client. Setup verifies MCP initialization, tool discovery and a first
workspace request, then registers through that client's CLI. Codex respects CODEX_HOME;
Claude uses user scope. Existing different entries are refused, other server settings
are preserved, and client restart/reconnect is needed afterward. Connection is explicit;
default setup does not edit client configuration.

To print the correct manual snippet after setup:

```sh
./scripts/mcp-client.sh --client codex config
./scripts/mcp-client.sh --client claude config
```

Published v0.1.1 lacks these options; configure its launcher manually.
For Codex, add this to its TOML configuration:

```toml
[mcp_servers.inkscape]
command = "/absolute/path/to/inkscape-mcp-server/run-mcp.sh"
args = []
```

For a JSON-based MCP host, use its documented format, for example:

```json
{"mcpServers":{"inkscape":{"command":"/absolute/path/to/inkscape-mcp-server/run-mcp.sh","args":[]}}}
```

The registered name is `inkscape`. See [client management](client-management.md) for
checks, scope precedence, ownership refusals, disconnect and uninstall. Skill installation
is separate unless `--install-skill` is also supplied. No Python MCP/uvx entry point is used.
Startup/reconnect never launches Inkscape GUI; explicit launch requires a user request.
