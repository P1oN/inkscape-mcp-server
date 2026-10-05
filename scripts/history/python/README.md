# Historical Python tooling

These files preserve earlier bootstrap, packaging, fixture, probe and native acceptance
implementations and their Python dependency graph. They are read-only historical evidence,
not supported commands. Their former paths/imports may no longer execute. Do not install
this environment or use it as a Python MCP/parity oracle. Current Rust/Bash entry points,
required gates and native acceptance phases are documented in ../../../CONTRIBUTING.md.

Native runtime assets still in runtime/ are INX descriptors and the Objective-C context
bridge. Frozen migration/contracts and historical migration/results remain unchanged.
Python wheel/CPython provenance under rust/package is historical vendor provenance;
the active Rust package builder never reads or ships those inputs.
