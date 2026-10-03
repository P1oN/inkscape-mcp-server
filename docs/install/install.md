# Install the Rust Inkscape MCP server

Install Inkscape 1.4 or newer, clone this repository and run `./setup.sh`.
The script detects Inkscape and asks for an existing SVG workspace; then `./run-mcp.sh`
starts the Rust STDIO server. No manual environment editing is needed.

Source packaging requires Rust and native build dependencies, plus a pinned private
helper environment or uv to prepare it. Ready archive users need only Inkscape.
The current local macOS arm64 archive is stage38; see
[exact commands and current artifact](../RUST_MIGRATION_REPORT.md#install-and-check-the-current-local-candidate).
Clean-machine installation will be checked later by the user; Windows is backlog.

Python package installation/uvx entry points are retired. Python remaining inside the
ready package serves bounded live helpers and the supervisor, not an MCP server.
