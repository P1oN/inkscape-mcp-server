# Private live runtime components

These files are required by the Rust server and are not the retired Python MCP server.

- helper_extension/: fixed Rust one-shot INX manifests and the remaining inkex socket helper.
- insert_payload.py / edit_errors.py and the one-shot `.py` sources: retained development
  fixtures/historical implementation; no longer shipped or invoked for native edits.
- native/context.m: Objective-C document/window context bridge.
- ../rust/src/bin/inkscape-mcp-supervisor.rs: separate native managed-session supervisor.
- ../rust/src/bin/inkscape-mcp-inx.rs: fixed one-shot extension consumer.
- tests/: retained helper refusal and serialization regressions.

The package builder copies these into a private runtime. The source installer prepares
pinned dependencies from rust/package/helper-requirements.txt; end users of a ready archive
do not install Python. Helpers never expose a second MCP server.

The shared Rust kernels in `../rust/src/helper_svg/` validate, plan and apply one-shot
SVG edits on owned candidates; see [live-helper-kernels.md](../docs/live-helper-kernels.md).
The supervisor installs the two fixed INX manifests and a safely quoted wrapper that
executes the packaged Rust helper. No Python interpreter or inkex participates in
one-shot insertion/editing. The socket live helper still needs the private interpreter
until stage 5; removing the bundled runtime is stage 6.
