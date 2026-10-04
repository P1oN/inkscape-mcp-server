# Private live runtime components

Native components support the Rust server; retained Python sources are historical development fixtures.

- helper_extension/: fixed Rust one-shot and socket INX manifests. The `.py` socket source is
  retained historical implementation, omitted from ready packages.
- insert_payload.py / edit_errors.py and the one-shot `.py` sources: retained development
  fixtures/historical implementation; no longer shipped or invoked for native edits.
- native/context.m: Objective-C document/window context bridge.
- ../rust/src/bin/inkscape-mcp-supervisor.rs: separate native managed-session supervisor.
- ../rust/src/bin/inkscape-mcp-inx.rs: fixed one-shot extension consumer.
- ../rust/src/bin/inkscape-mcp-live.rs: bounded v5 loopback snapshot bridge.
- tests/: retained helper refusal and serialization regressions.

The package builder copies these into a private runtime. The source installer prepares
pinned dependencies from rust/package/helper-requirements.txt; end users of a ready archive
do not install Python. Helpers never expose a second MCP server.

The shared Rust kernels in `../rust/src/helper_svg/` validate, plan and apply one-shot
SVG edits on owned candidates; see [live-helper-kernels.md](../docs/live-helper-kernels.md).
The supervisor installs the three fixed INX manifests and a safely quoted wrapper that
executes the packaged Rust helper. No Python interpreter or inkex participates in
one-shot insertion/editing. The socket live helper also executes a fixed Rust binary through its own quoted wrapper.
Ready packages contain no CPython, wheels or Python helper assets. Doctor checks native binaries, bridge and private bus without importing Python or requiring vendor inkex.
