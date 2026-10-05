# Rust server troubleshooting

Run `./setup.sh` again to review Inkscape/workspace paths, then start `./run-mcp.sh`.
Source builds require the pinned Rust toolchain and native libxml/clang/GLib dependencies;
a ready archive avoids user compilation and development-tool installation.
The launcher's errors go to stderr; MCP STDIO stdout is reserved for protocol messages.
Use the configured Rust executable's `--doctor` for read-only dependency diagnostics.
Doctor does not launch GUI or repair the user's system.

If live reconnect succeeds but editing is unavailable, explicitly select the intended
window/document. Reconnect clears edit readiness. An uncertain operation result requires
inspection before any retry. Preserve trace, Operation Records, document IDs and artwork
if a historical native incident recurs; see [the next plan](../RUST_NEXT_PLAN.md).
Do not use retired Python server entry points. Do not close user windows or kill Inkscape.
