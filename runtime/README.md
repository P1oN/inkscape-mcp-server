# Private native live runtime assets

- `helper_extension/`: fixed INX descriptors for the Rust one-shot/socket consumers.
- `native/context.m`: Objective-C window/document context bridge.
- `../rust/src/bin/`: native supervisor, INX and socket helper implementations.

The Rust package builder copies these assets into a private runtime. Ready packages
contain no Python/wheels/inkex. The supervisor installs fixed quoted wrappers that
execute packaged Rust binaries; no second MCP server or arbitrary extension is exposed.
Doctor checks native binaries, context bridge and private bus.

Shared bounded SVG kernels and their regressions are in `rust/src/helper_svg` and
`rust/tests`. Historical Python fixtures/tests moved to
[scripts/history/python](../scripts/history/python/README.md). Use current Rust/Bash
checks in [CONTRIBUTING](../CONTRIBUTING.md) and read
[helper semantics](../docs/live-helper-kernels.md).
