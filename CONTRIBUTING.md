# Contributing to the Rust Inkscape MCP server

Read AGENTS.md, docs/AGENT_HANDOFF.md, README.md and docs/agent-usage-guide.md first.
Preserve uncommitted work. The Python MCP implementation and paired comparison workflow
are retired; improve Rust against explicit contracts, invariants and regression scenarios.

## Development and checks

Use Rust 1.99.0, pinned by the package builder/CI, and Cargo.lock.
A rust-toolchain.toml is not currently supplied.
Keep Tokio at the pinned 1.52.1 patch or a verified newer version: 1.52.0 has a
[blocking-pool hang regression](https://github.com/tokio-rs/tokio/issues/8056) affecting
STDIO and `spawn_blocking` workers.
Native libxml/clang development dependencies are required; on macOS point LIBXML2 at
$(xcrun --show-sdk-path)/usr/lib/libxml2.tbd. Inkscape is required for CLI/native acceptance.

```sh
cargo fmt --check --manifest-path rust/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/Cargo.toml
```

Python is development/packaging tooling and a private live runtime, not the MCP server.
Before Rust tests, create `.venv` with the pinned helper interpreter and dependencies
(the synthetic process/bus fixtures use it), for example `uv venv --managed-python
--python 3.12.14 .venv` then `uv pip install --python .venv/bin/python
-r rust/package/helper-requirements.txt`. Use `uv sync --group dev` for the additional
development tools:

```sh
python -m pytest runtime/tests
ruff check scripts runtime
ruff format --check scripts runtime
python scripts/gen_llms_txt.py --binary /absolute/path/to/current/rust/binary
```

Regenerate llms.txt and llms-full.txt whenever the exposed surface or instructions change.
The generator queries the actual Rust STDIO server without launching a GUI.
Frozen discovery JSON under migration/contracts is the current schema/instruction source;
CI checks that contract without starting a Python server. No automatic parity work is required.
Use the pinned `.venv/bin/python` for discovery checks. Each configuration retains a
`.trace.json` with completed requests and the pending request if the probe fails, alongside
server stderr. Do not hide response timeouts by retrying or weakening contract comparison.

Run package/doctor/launcher checks for packaging changes and real Inkscape render/export
for engine changes. Native acceptance is separate from automated tests. Only explicitly
owned synthetic GUI sessions may be changed or closed. Startup/reconnect must never launch GUI.

## Required invariants

Keep tools typed and bounded. Reuse the edit pipeline, snapshots, Operation Records,
no-op handling and approval gates. Preserve originals, workspace/symlink protections,
SVG IDs/references and appearance; reject unsafe structural edits before mutation.
Use argument-list subprocesses, safe XML parsing and no arbitrary shell/code/extensions.
Do not add bitmap tracing. Do not commit, publish or send messages without user authorization.
Windows is backlog. Prepared CI jobs are not evidence of real target validation.

## Installation and responsiveness regressions

Run `scripts/rust_responsiveness_acceptance.py --binary PATH --output DIRECTORY`
with `.venv/bin/python` for real STDIO discovery/workspace responsiveness and cancellation
against owned synthetic per-call/shell processes. `scripts/rust_stdio_startup_acceptance.py --binary PATH
--output DIRECTORY` exercises 128 fresh sessions with four concurrent owned servers,
including an immediate first mutation and discovery requests, without retries or a GUI.
Run launcher/package/doctor/notices acceptance for install changes. `scripts/install_path_acceptance.py --archive ARCHIVE
--output DIRECTORY` exercises extracted-source setup, real isolated Codex registration,
skill merge/conflicts, Inkscape CLI rendering, uninstall and reinstall on the current Mac.
It reuses existing pinned host tools/caches; it does not claim clean-machine or GUI acceptance.
Use `build_source_archive.py --working-tree` only for explicitly unpublished local snapshots;
default source export continues to contain committed files only.
Working-tree archives omit SOURCE_REVISION and report an unknown revision. The install
regressions in `runtime/tests/test_install_management.py` cover archive identity and Git
worktree build watches; run them with the helper tests above. Client management regressions,
including uninstall rollback and request deadlines, are Rust tests in
`rust/src/bin/inkscape-mcp-client.rs`. Cargo builds the server, client manager and supervisor;
package construction requires all three binaries in the same target directory. Run
`scripts/client_management_acceptance.py --binary PATH --output DIRECTORY` and
`scripts/client_package_acceptance.py --package DIRECTORY --output DIRECTORY` for client
CLI guards and relocated ready-package management without Python on client PATH and with
a damaged helper runtime. Both use isolated profiles; Claude remains synthetic.

Documentation-only changes need link/command consistency and `git diff --check`; do not
rerun runtime or native GUI acceptance unless code, MCP schemas or initialization guidance
also changes. Keep release instructions separate from PR sources, and label historical
checkpoint reports. The handoff records the evidence source/revision and validation limits;
the active plan records remaining work, without transferring old GUI results to new builds.

## Managed supervisor regressions

Cargo builds the separate `inkscape-mcp-supervisor` executable alongside the server/client
manager; the package builder requires all three in the same target directory. The supervisor
is invoked only by explicit `live_launch`, with the fixed session root and detected vendor
binary. Startup, client checks, reconnect and doctor must never invoke it.

`cargo test` includes supervisor filesystem/preparation and child lifecycle tests plus a
macOS integration test running the real supervisor with synthetic native processes, empty
PATH and no Python runtime. The fixture compiler is test tooling only, never runtime code.
Helpers still require the bundled interpreter; do not confuse supervisor independence with
removing that interpreter from ready packages.

For explicitly authorized native acceptance, use
`scripts/migration_native_gui_acceptance.py --package DIRECTORY --output DIRECTORY --close-owned`.
It isolates HOME/profile/workspace/session, checks launch/connect/context and the unchanged
vendor executable, then gracefully quits only the blank GUI whose unique private-bus owner,
PID and supervisor ancestry were verified. It checks manifest removal and owned-bus shutdown.
Without the flag the historical runner retains its owned window for follow-up acceptance.
Never use process-name termination or operate on pre-existing user windows.

## Shared SVG helper regressions

Stage 3 exposes `inkscape_mcp_rust::helper_svg` and the hardened XML parser as a
library for future native helpers. `cargo test` includes `rust/tests/helper_svg.rs`:
frozen fingerprint wire values, safe fragment preparation, typed selection/edit
plans, reference/size/work bounds, affine compensation and genuine planned no-ops.
The server already reuses fingerprint/insertion preflight; headless reparenting
shares affine arithmetic. See [live helper kernels](docs/live-helper-kernels.md)
for live semantics, conservative refusals and consumer obligations.

The planner returns semantic steps, not a replacement native extension. Python/inkex
helpers still apply live edits. Stage 4 must separately validate application, root
and mixed-content preservation, native Undo and stale-state refusal on owned
synthetic drawings before changing that route. No GUI acceptance is implied by
library tests. Exposed MCP contracts/instructions are unchanged by extraction;
manifest regeneration is required only when those surfaces change.
