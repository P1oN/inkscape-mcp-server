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

## Task lifecycle

Before heavy builds or tests, at long-task stage boundaries and at task completion,
check filesystem free space and the size of the selected build and cache directories.
Set a cache budget and leave enough free space for the expected workload before
starting another build or test. Coordinate cleanup of shared roots with their users.
Check active processes and open files, including inside test VMs, and leave busy
paths in place. Use the guarded cleanup below and reuse compatible caches between checks.

Create disposable output under a task-owned directory. Record its owner, purpose,
source revision, how to recreate it and whether it is temporary output, a cache,
evidence or a rollback copy. Remove task-owned disposable output after successful
steps. After a failure or cancellation, retain the diagnostics and reproducer needed
for investigation. Preserve configuration, original artwork, uncommitted work, reports
and required screenshots and provenance. Schedule background cleanup only at the
user's request.

## Source setup

The local package builder removes its own staging directory on failure or handled
interrupt, and skips duplicate distribution archive creation. Successful
packages carry a versioned ownership marker. Successful source setup automatically
runs bounded package cleanup: retain the newest three marked builds, all builds younger
than seven days, the package selected by `setup.conf`, the configured workspace and any
package mentioned by a running process. Unmarked older builds, symlinks, staging roots
with user additions and packages with added or removed paths are preserved. Independent-update
runtime profiles follow their own retention and rollback policy.

Before rerunning setup or applying package cleanup, inspect eligible builds for packages
and symbols needed for rollback or an ongoing investigation. Keep required copies outside
the managed build directories and leave selected or running packages in place. Defer setup
or package cleanup if required copies cannot be preserved. The automatic age and count
rules apply only to marked disposable builds.
The package inventory checks paths, so preserve any in-place changes you need to keep
outside the managed build directory before running setup or cleanup.

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
For manual deduplication of recoverable copies, keep reports separate from the installed
packages or staging directories you plan to remove.

## Explicit deduplication of retained artifacts

Inspect ignored backups and test profiles for unique configuration, source fixes, artwork
and evidence before selecting paths. Retain compact reports and logs, commands, source
and artifact hashes, and required screenshots. Confirm each selected profile is unused by
configured clients and workspaces, running processes and open files.

Before removing an extracted package, verify that its retained archive reproduces the
complete tree, including file bytes, file types, symlink targets and significant metadata.
Keep the archive, checksum and provenance. Preserve any unique configuration or reports.
Remove explicitly selected synthetic disk images once they are unmounted and unused.
Keep their diagnostic reports.

Compare complete `.dSYM` bundles by relative path, file type, file bytes, symlink target
and significant metadata. Keep one complete copy of each distinct bundle outside every
directory selected for deletion, even when two distinct bundles share a UUID. Record
fingerprints and original and retained paths in a local manifest. Revalidate the retained
copies before deletion and verify the retained bundles, configuration and reports afterward.
Keep active and selected packages complete. Restore removed components of historical test
packages from the recorded copies before validating their manifests or rerunning them.
Passing reports apply to the bytes tested in the original run.

## Explicit cleanup

Preview package retention and check the candidates against the preservation rules above
before applying it:

```sh
scripts/cleanup-development.sh
scripts/cleanup-development.sh --apply
```

Before selecting `--caches`, retain release symbols and their matching executables outside
both default Cargo target directories. Keep each unique macOS `.dSYM` bundle and the matching
Linux release `.dwp` files for builds with packed debug information. If any required files
cannot be preserved, leave the targets in place. Preview or remove both default targets
while development tools are idle:

```sh
scripts/cleanup-development.sh --caches
scripts/cleanup-development.sh --apply --caches
```

Cache removal refuses active Cargo/rustc, running executables under the targets, a configured
runtime/workspace under a target, symlinked parents/targets and directories without Cargo's
cache marker. It leaves custom target directories alone. Keep builds idle throughout
cleanup because the script does not acquire Cargo's build lock. The script removes
eligible directories in full. Store drawings, configuration and evidence outside them.
Cleanup leaves Inkscape and MCP processes alone.
Record selected paths, retained originals and deferred busy paths. Measure filesystem
free space before and after cleanup. Hard links and APFS clones make summed file sizes
unreliable as a measure of physical savings. Concurrent builds and installations can
also change free space during cleanup. Rebuild the cache when the next check needs it.

`scripts/test-development-cleanup.sh` validates deletion/retention, process/symlink protection,
successful/failed acceptance output and build failure cleanup with isolated fixtures; native
GUI acceptance is separate. Quick CI runs this guard suite.
