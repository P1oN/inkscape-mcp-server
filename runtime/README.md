# Private live runtime components

These files are required by the Rust server and are not the retired Python MCP server.

- helper_extension/: fixed inkex effects/socket helper and INX manifests.
- insert_payload.py / edit_errors.py: bounded one-shot helper validation.
- native/context.m: Objective-C document/window context bridge.
- ../rust/package/supervise.py: private managed-session supervisor.
- tests/: retained helper refusal and serialization regressions.

The package builder copies these into a private runtime. The source installer prepares
pinned dependencies from rust/package/helper-requirements.txt; end users of a ready archive
do not install Python. Helpers never expose a second MCP server.
