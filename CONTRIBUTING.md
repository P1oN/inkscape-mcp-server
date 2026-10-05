# Contributing to the Rust Inkscape MCP server

Read AGENTS.md, docs/AGENT_HANDOFF.md, README.md and docs/agent-usage-guide.md first.
Preserve uncommitted work. The Python MCP implementation and paired comparison workflow
are retired; improve Rust against explicit contracts, invariants and regression scenarios.

## Development and checks

Use Rust 1.99.0 and the two locked Cargo graphs. No Python environment is required.
Native libxml/clang development dependencies are required; on macOS set
`LIBXML2="$(xcrun --show-sdk-path)/usr/lib/libxml2.tbd"`. Inkscape 1.4+ is required
for CLI/native acceptance. `scripts/dev-tools.sh` locates pinned Cargo and the macOS
SDK, builds the development-only CLI in `rust/tooling`, and invokes a fixed command.
The development executable and its dependencies are never copied into ready packages.
Keep Tokio at the pinned 1.52.1 patch or a verified newer version: 1.52.0 has a
[blocking-pool hang regression](https://github.com/tokio-rs/tokio/issues/8056).

```sh
cargo fmt --check --manifest-path rust/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/Cargo.toml
cargo fmt --check --manifest-path rust/tooling/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/tooling/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/tooling/Cargo.toml
cargo build --locked --release --manifest-path rust/Cargo.toml
scripts/dev-tools.sh discovery --binary rust/target/release/inkscape-mcp-rust --matrix --output migration/results/discovery
scripts/dev-tools.sh manifests --binary rust/target/release/inkscape-mcp-rust --output .
```

After editing `migration/contracts/authoring-guidance.txt`, run
`scripts/dev-tools.sh sync-authoring-guidance` to refresh the generated initialization and
compose snapshots. The policy has one source; regression checks enforce snapshot consistency.

Regenerate llms.txt and llms-full.txt whenever the exposed surface or instructions change.
The generator queries the actual Rust STDIO server without launching a GUI. Frozen
migration/contracts remain the schema/instruction source. Failed discovery retains
completed requests, the pending request and stderr. Never conceal timeouts with retries.
Cargo tests compile native synthetic process/bus/engine fixtures with the same Rust toolchain.
The development crate covers archive traversal/links/checksums, owned ELF relocation,
bootstrap cleanup, Git-free source identity and Git worktree build watches. Historical
Python sources/tests and their former dependency files are retained under
[scripts/history/python](scripts/history/python/README.md) as evidence, outside active paths.

## Required invariants

Keep tools typed and bounded. Reuse the edit pipeline, snapshots, Operation Records,
no-op handling and approval gates. Preserve originals, workspace/symlink protections,
SVG IDs/references and appearance; reject unsafe structural edits before mutation.
Use argument-list subprocesses, safe XML parsing and no arbitrary shell/code/extensions.
Do not add bitmap tracing. Do not commit, publish or send messages without authorization.
Windows is backlog. Prepared CI jobs are not evidence of real target validation.
Startup/reconnect/doctor/client checks must never launch Inkscape. Native acceptance is
separate from automated tests and requires authorization to launch a synthetic GUI.

## Package and regression acceptance

The builder requires all five native executables in one Cargo target directory:
server, client manager, supervisor, INX helper and socket helper. Ready packages retain
fixed Bash interfaces, native context bridge and private D-Bus/GLib dependencies.

```sh
scripts/dev-tools.sh build-package --output dist/candidate --archive dist/candidate.tar.gz
scripts/dev-tools.sh package-acceptance --archive dist/candidate.tar.gz --output migration/results/package-cold
scripts/dev-tools.sh package-acceptance --archive dist/candidate.tar.gz --engine-mode shell --output migration/results/package-shell
scripts/dev-tools.sh socket-acceptance --binary dist/candidate/bin/inkscape-mcp-live --output migration/results/socket
scripts/dev-tools.sh doctor-acceptance --package dist/candidate --output migration/results/doctor
scripts/dev-tools.sh notices-acceptance --package dist/candidate --output migration/results/notices
scripts/dev-tools.sh launcher-acceptance --package dist/candidate --output migration/results/launcher
scripts/dev-tools.sh sentry-setup-acceptance --output migration/results/sentry
scripts/dev-tools.sh client-acceptance --package dist/candidate --output migration/results/clients
```

Package acceptance verifies FILES, relocates a real archive, uses empty PATH, executes
actual native INX/private-bus exchanges and checks headless edits, approvals, rollback,
records, no-ops, snapshots, resources and real CLI pixels. Socket acceptance separately
checks authentication, framing, geometry, render/export, tokens, input preservation and
rendezvous cleanup. Notices acceptance binds crate/native/Rust provenance to package
hashes and rejects damaged provenance. Legal audit gaps remain explicit.
Client acceptance uses real Codex in an isolated home and synthetic Claude; it checks
ownership, failed handshakes, symlinks, uninstall/skill archival and preserved drawings.
No installed user runtime or configuration is changed.

For server changes run the following bounded real-STDIO suites with
`--binary dist/candidate/bin/inkscape-mcp --output migration/results/NAME`:
`security-acceptance`, `frame-acceptance`, `startup-acceptance`,
`responsiveness-acceptance`, `defects-acceptance`, `diagnostic-acceptance`,
`compare-acceptance`, `special-file-acceptance`, `renderer-acceptance` and
`engine-routes-acceptance`. For authoring changes also run:

```sh
scripts/dev-tools.sh authoring-acceptance --binary rust/target/release/inkscape-mcp-rust --output migration/results/authoring
scripts/dev-tools.sh authoring-acceptance --binary rust/target/release/inkscape-mcp-rust --engine-mode shell --output migration/results/authoring-shell
```

These probes use deliberate editable flower/fold/snowball and overlapping-shape fixtures,
real CLI pixels, read-only report checks, approved deletion/no-op/restore and reference
refusal. Inspect the retained SVG and PNGs separately for silhouette quality.

Startup defaults to 128 fresh sessions/four concurrent
servers with immediate mutation/discovery and no retries. Responsiveness checks both
engine modes and owned process cancellation. Renderer checks require isolation.

```sh
scripts/dev-tools.sh source-archive --working-tree --output migration/results/source.tar.gz
scripts/dev-tools.sh install-acceptance --archive migration/results/source.tar.gz --output migration/results/source-install
```

Source archive export defaults to committed files and SOURCE_REVISION. Explicit
`--working-tree` snapshots omit that marker and report unknown revision. Source install
acceptance requires real Codex and existing pinned native tools/caches. It builds from
Git-free unpublished sources, registers an isolated client, merges/conflicts a customized
skill, renders through real CLI, uninstalls and reinstalls while preserving drawings.
It does not establish clean-machine installation or native GUI acceptance.

## Native helpers and explicitly authorized GUI acceptance

`rust/tests/helper_svg.rs`, `inx.rs` and `socket_helper.rs` cover frozen fingerprints,
reference remapping, local affine compensation, all ten INX operations, parser bounds,
input preservation and planned no-ops. The explicit native CLI gate is:

```sh
INKSCAPE_MCP_ACCEPTANCE_INKSCAPE=/absolute/path/to/inkscape \
  cargo test --locked --manifest-path rust/Cargo.toml --test inx real_inkscape_render -- --ignored
```

The Rust development CLI provides these opt-in native phases:

```sh
scripts/dev-tools.sh native-gui --package dist/candidate --output migration/results/native --phase setup
scripts/dev-tools.sh native-inx --output migration/results/native --phase capture --label before
scripts/dev-tools.sh native-socket --package dist/candidate --output migration/results/native-socket --phase setup
```

`native-gui setup` launches one isolated blank GUI, records its manifest, private bus,
context UUIDs and vendor hash, then verifies reconnect without another launch. Failures
retain the synthetic session. Every subsequent phase checks that exact manifest and
single window/document context. Evidence labels must be fresh ASCII alphanumeric/hyphen
names. No phase guesses a selection or operates on a pre-existing user window.

`native-inx` supports `insert`, `insert-text`, `style`, `noop`, `text`, `duplicate`,
`delete`, `group`, `ungroup`, `raise`, `lower`, `front`, `back`, `capture`, `stale-content`,
`stale-ids` and `stale-selection`. Perform selection and native menu Undo/Redo in the
recorded private app. Capture independent before/after/Undo/Redo SVGs with fresh labels;
operation phases require applied audit records and refusals/no-ops preserve artwork.
Restore the blank original, then run `native-gui --output DIRECTORY --phase close-owned`.
Graceful close requires the unchanged blank capture, unique bus owner, peer PID (or
Darwin context fallback), exact supervisor ancestry/private executable path, manifest
removal, bus shutdown and owned process exit. It never kills Inkscape by name.

`native-socket setup` creates a baseline rectangle/text. Select its rectangle and run
`--phase style`; select its text and run `text`. Both check changed output and repeated
no-op. Use the exact owned app's native Undo, then `verify-text-undo`, another Undo,
`verify-style-undo`, Undo baseline insertion and `finish`. `package-style` checks the
selected baseline root group without another selection; after native Undo run
`verify-style-undo`, Undo baseline and `close-owned`. Finish requires the blank original
and independent Undo/change/no-op evidence before graceful close. Full text/Redo remains
separate; CLI success or session cleanup never proves native Undo/Redo. See
[helper semantics](docs/live-helper-kernels.md) for timing and supported limits.

Documentation-only changes need command/link consistency and `git diff --check`;
rerun native GUI acceptance only when live behavior changes. Record fresh evidence and
limitations in the handoff. Never transfer historical GUI results to a new build.
