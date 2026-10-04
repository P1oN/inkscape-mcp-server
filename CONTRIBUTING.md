# Contributing to the Rust Inkscape MCP server

Read AGENTS.md, docs/AGENT_HANDOFF.md, README.md and docs/agent-usage-guide.md first.
Preserve uncommitted work. The Python MCP implementation and paired comparison workflow
are retired; improve Rust against explicit contracts, invariants and regression scenarios.

## Development and checks

Use Rust 1.99.0, pinned by the package builder/CI, and Cargo.lock.
A rust-toolchain.toml is not currently supplied.
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
against owned synthetic per-call/shell processes. Run launcher/package/doctor/notices
acceptance for install changes. `scripts/install_path_acceptance.py --archive ARCHIVE
--output DIRECTORY` exercises extracted-source setup, real isolated Codex registration,
skill merge/conflicts, Inkscape CLI rendering, uninstall and reinstall on the current Mac.
It reuses existing pinned host tools/caches; it does not claim clean-machine or GUI acceptance.
Use `build_source_archive.py --working-tree` only for explicitly unpublished local snapshots;
default source export continues to contain committed files only.
Working-tree archives omit SOURCE_REVISION and report an unknown revision. The install
regressions in `runtime/tests/test_install_management.py` cover archive identity, Git worktree
build watches and uninstall rollback; run them with the helper tests above.

Documentation-only changes need link/command consistency and `git diff --check`; do not
rerun runtime or native GUI acceptance unless code, MCP schemas or initialization guidance
also changes. Keep release instructions separate from PR sources, and label historical
checkpoint reports. The handoff records the evidence source/revision and validation limits;
the active plan records remaining work, without transferring old GUI results to new builds.
