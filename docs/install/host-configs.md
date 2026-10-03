# MCP host configuration

Configure your host's STDIO MCP integration to execute the absolute path to `run-mcp.sh`
after running setup. The launcher loads saved local settings noninteractively.
Use the host's own configuration format; a typical MCP JSON configuration is:

```json
{"mcpServers":{"inkscape":{"command":"/absolute/path/to/inkscape-mcp-server/run-mcp.sh"}}}
```

No uvx, Python server module or Python console entry point is required.
Do not launch Inkscape as a side effect of server startup/reconnect. GUI launch remains
an explicit operation requested by the user. See [install](install.md) and
[agent usage](../agent-usage-guide.md). User host configuration is not modified automatically.
