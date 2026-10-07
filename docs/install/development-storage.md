# Development disk retention

Source builds and acceptance runs previously accumulated Cargo incremental/debug caches,
private package directories, duplicate archives and fully extracted test installations.
Both Cargo graphs now disable development debug symbols and incremental compilation by
default. Release debug symbols remain unchanged for Sentry. This trades incremental
recompilation speed and source-level development debugging for smaller local caches;
Cargo still reuses compiled dependencies. Opt in to debugging when needed:

```sh
CARGO_PROFILE_DEV_DEBUG=2 CARGO_PROFILE_DEV_INCREMENTAL=true cargo test --locked --manifest-path rust/Cargo.toml
```

## Source setup

The local package builder removes its own staging directory on failure or handled
interrupt, and skips duplicate distribution archive creation. Successful
packages carry a versioned ownership marker. Successful source setup automatically
runs bounded package cleanup: retain the newest three marked builds, all builds younger
than seven days, the package selected by `setup.conf`, the configured workspace and any
package mentioned by a running process. Unmarked older builds, symlinks and staging roots
with user additions, or packages with added/removed paths, are preserved. Independent-update runtime profiles keep their existing
retention/rollback behavior; this cleaner does not operate on them.

Process inspection/configuration errors refuse cleanup. A runtime command using a relative
or bare executable name also refuses the whole cleanup because its owning package cannot
be identified safely. SIGKILL or power loss cannot run
exit traps; unmarked interrupted staging is retained for explicit inspection. Automatic
retention is an age/count policy, not a hard byte quota. A stopped client registered
directly against an old marked package should select the current setup runtime before
the old package reaches retention age. Ordinary clients use the stable `run-mcp.sh` path.

## Acceptance output

For supported automated acceptance checks whose output does not need long-term retention:

```sh
scripts/dev-tools.sh --temporary-output authoring-acceptance --binary rust/target/release/inkscape-mcp-rust
scripts/dev-tools.sh --temporary-output doctor-acceptance --package dist/candidate
```

The wrapper creates a private synthetic output directory, deletes it only after a passing
command, and prints its location on failure or interruption. Cancellation forwards the
signal to the owned acceptance process and allows five seconds to stop before terminating
that process; cancelled evidence is retained. Do not use this option when
the output is needed for visual inspection or build-bound evidence. An explicit `--output`
retains the existing behavior. Native GUI phases and update/installer/client profiles are
excluded because they deliberately retain sessions, profiles or ownership evidence.
Failed output is intentionally retained; inspect it and remove it explicitly after diagnosis.
Historical `migration/results` and the retained-evidence archive are never swept automatically.

## Explicit cleanup

Preview or apply package retention without rebuilding anything:

```sh
scripts/cleanup-development.sh
scripts/cleanup-development.sh --apply
```

To also remove both default Cargo target directories, while development tools are idle:

```sh
scripts/cleanup-development.sh --caches
scripts/cleanup-development.sh --apply --caches
```

Cache removal refuses active Cargo/rustc, running executables under the targets, a configured
runtime/workspace under a target, symlinked parents/targets and directories without Cargo's
cache marker. It leaves custom target directories alone. It does not acquire Cargo's build
lock: keep builds idle throughout cleanup. Subsequent checks rebuild dependencies. No cleanup
command launches, restarts or closes Inkscape/MCP, or removes drawings/configuration/evidence.

`scripts/test-development-cleanup.sh` validates deletion/retention, process/symlink protection,
successful/failed acceptance output and build failure cleanup with isolated fixtures; native
GUI acceptance is separate. Quick CI runs this guard suite.
