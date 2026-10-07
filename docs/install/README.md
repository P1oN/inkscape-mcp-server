# Install and operate Inkscape MCP

Use the Rust/Bash installer with Inkscape 1.4+:

```sh
./setup.sh --workspace /absolute/path/to/your/svgs
./run-mcp.sh --doctor
```

Point the MCP host at the absolute path to `run-mcp.sh`; the server speaks MCP over
STDIO. Startup never launches Inkscape. Ready packages require no Python or compiler;
source setup prepares the native toolchain on supported Apple Silicon hosts.

- [Installation](install.md) and [source bootstrap](local-bootstrap.md)
- [Host configuration](host-configs.md) and [client management](client-management.md)
- [Independent instruction/runtime updates and management app](independent-updates.md)
  (newer source feature; migration is opt-in)
- [Compatibility](compatibility.md)
- [Troubleshooting](troubleshooting.md)

Read/edit/validate work without Inkscape. Render/export/path geometry and explicitly
requested live sessions require Inkscape. `diagnose_runtime` reports available support.
