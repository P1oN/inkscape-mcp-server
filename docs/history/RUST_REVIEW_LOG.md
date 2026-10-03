# Local Rust migration review

Reviewed 2026-10-03 on `codex/rust-migration`, reference HEAD `c50a924`.
The migration is uncommitted, including new Rust sources; review therefore includes
working-tree/untracked files, not only a committed branch diff. No commits, PRs,
publication, user MCP configuration changes or GUI launches were performed.

## Namespace collision follow-up (2026-10-03)

Confirmed P2: libxml's `get_property` and `remove_property` match local names across
namespaces, unlike Python's `element.get("fill")`. A `q:fill="red"` could produce a
false `set_fill(red)` no-op; setting blue removed the foreign attribute. A `q:id`
could also select or shadow an ordinary SVG ID. Both new Rust regressions failed
before the fix and pass afterward (initial failures are in this chat's tool output).

All ordinary-attribute reads/removals in 37 Rust DOM modules now use the explicit
`*_no_ns` methods. Qualified XLink/Inkscape/Sodipodi operations retain their namespace.
The parser module documents this distinction. No tool/schema/instruction changes
were made; all 16 frozen discovery configurations still match exactly.

Current validation: Rust **201 passed / 1 ignored**, fmt and all-target clippy pass;
Python **1211 passed / 90 skipped** with the current default PATH, Ruff check/format
and mypy pass. Real STDIO tests compare **15 namespace scenarios**, **71 existing
read scenarios**, and **122 existing edit scenarios**, with zero differences. The
edit suite uses official Inkscape CLI rendering/exports on isolated drawings;
none of these checks is native GUI acceptance. The namespace suite checks foreign
attributes, target shadowing/refusal, batch rollback, original preservation, XLink
rewrites, no-op retained files and Rust's stricter directory-mtime invariant.

`scripts/migration_namespace_acceptance.py` retains full replies and SVG bytes;
it normalizes only the existing validated ID/timestamp bindings. The final fresh
release result, executable/source hashes and raw traces are indexed by
`migration/namespace-review-comparison.json` and stored under
`migration/results/namespace-review-release-v2/`. Other runs are in
`migration/results/namespace-review-{read,edit,discovery}/`. The initial new harness
refused Python's known transient no-op directory-mtime change; that failed trace
remains in `namespace-review-debug/`. The corrected harness requires unchanged
retained files for both servers and unchanged directories additionally for Rust.

Existing stage27 packaged/native/performance results describe the pre-fix package.
No package installation, user MCP configuration change, server restart or GUI
mutation was performed in this follow-up. Build/setup from current source includes
the fix; a ready archive must be rebuilt and separately validated before carrying
this evidence to it. This finding does not establish a cause for historical native
history incidents.

## Confirmed findings and fixes

1. **P1: lost audit writes could hide a live effect's uncertain completion.**
   `rust/src/live_mutation.rs` converted a failed post-dispatch operation-record update
   into a generic storage refusal. This hid an original uncertain transport outcome;
   even a completed effect whose final audit failed looked like a clean refusal.
   The error path now preserves its original transport error, and failed final applied
   persistence returns the inspect-before-retry uncertainty message. It never retries
   the effect. Pre-dispatch approval/record failures retain their refusal behavior.
   A real no-follow audit-directory symlink replacement test exercises both successful
   effects and uncertain transport replies, verifies exactly one dispatch, retained
   proposed history and an untouched external sentinel. Before the fix the focused test
   failed with `could not store live operation record`; afterward it passes. That initial
   failure was observed in tool output, not saved as a raw log file.

2. **P2: continuously readable inherited pipes could defeat the process deadline.**
   `rust/src/process.rs` formerly stopped completed-process readers only on EOF or
   `WouldBlock`. A descendant continuously supplying bytes could keep `join()` waiting
   indefinitely. Readers now have a 100 ms drain grace after leader completion and
   retain already buffered output within that bound. A synthetic continuously readable
   reader verifies termination/output caps; another verifies buffered output is preserved.
   This finding follows the original loop's control flow; no hanging baseline test is claimed.

3. **P2: failed nonblocking setup was ignored.**
   The reader previously ignored `fcntl` failures and could enter a blocking read.
   Failed `F_GETFL`/`F_SETFL` now returns a diagnostic without reading. A failed-descriptor
   regression panics if reading is attempted and verifies the explicit error instead.

These are concrete process/audit defects. They are **not a demonstrated cause or fix**
for stage21's native duplicate disappearance. That owned drawing and its earlier raw
evidence remain preserved; no repair, reinsertion, structural retry or restart was attempted.

## Review cycles and validation

Cycle 1 inspected high-risk bounded process and post-dispatch live audit paths, added
regressions and fixed the findings. Cycle 2 reread the fixes, joined-reader cleanup,
pre-dispatch gates and error propagation; no further defect was found in those paths.
It ran the applicable full gates and fresh release/STDIO/CLI comparisons. This is bounded
review, not a claim of exhaustive review of every migration module or possible failure.

Final applicable checks:

- Rust: **197 passed / 1 ignored**, format and all-target clippy with warnings denied.
- Python with real official Inkscape 1.4.3 on PATH: **1289 passed / 12 skipped**;
  Ruff check/format (**325 files**) and mypy (**122 source files**) pass.
- Fresh release executable: **16/16 exact discovery configurations**, **122 edit scenarios**
  with no differences, **62 synthetic IPC live observations** with no unexpected differences.
  The existing two explicit stronger lost-reply uncertainty deltas remain documented;
  the new post-dispatch audit-loss behavior is also intentionally stronger than the reference.
- Initial clippy failed for a test module preceding runtime items; it was moved to the
  end and clippy passed. Initial launcher harness assumed an incorrect default tool count;
  it now compares the exact frozen default contract. Failed logs are retained.

After review, installation work added `setup.sh` and `run-mcp.sh`, then rereviewed their
configuration boundaries. Settings are data, never `eval`/`source`; setup errors preserve
saved settings and launcher stdout remains JSON-RPC only. **11 actual-package checks** cover
literal shell metacharacters/spaces, private file permissions, missing/truncated/extra/invalid
settings, separator/newline refusal, symlink refusal, STDIO discovery/open/inspect and original
preservation. Shell syntax checks pass. Automatic source package building remains pending.

Stage22 was rebuilt from these fixes and includes both executable launchers. Installation
from its actual archive with empty PATH passes cold and warm modes, real CLI render/export,
private helper/D-Bus, snapshots/restore, approval/no-op/rollback and 110/7/18 discovery.
Ten doctor profiles and nine notice checks pass. **No native GUI acceptance or updated
performance measurements are transferred to stage22.**

Raw commands/results are under `migration/results/review-cycle1/`;
`migration/review-cycle1-comparison.json` and its evidence binding identify source, executable,
archive, reports and logs. Source/report changes after a run must be distinguished from
the exact versions captured in that binding. Full migration remains incomplete; remaining
work is enumerated in [the completion checklist](RUST_COMPLETION_CHECKLIST.md).

## Subsequent goal work

After the review checkpoint, a separate explicitly authorized fresh stage22 native session
passes 46 fixed effect/history, 38 structure, 22 transform and 16 lower/order checks; five
Python/Rust read-only pairs pass 40 comparisons. Stage21's disappearance did not reproduce
in this sequence; no causal fix is claimed. This follow-up is bound separately in
`migration/native-stage22-evidence-binding.json` and the migration report. The review-cycle1
binding/snapshots and its no-GUI checkpoint remain immutable historical evidence.

### Stage27 remaining headless review (2026-10-03)

Current byte-identical release location passes 16 headless families (1715 reported
scenarios/observations); actual package location passes 11 families (1462). Counts overlap,
and neither is a uniform test count. Real CLI action/path/render/export/profile comparisons
retain SVG/pixel/artifact/rollback assertions. Fault suites retain six action/seven path
explicit stronger discarded-audit differences, with zero unexpected differences.

Initial runs are preserved: missing vendor PATH prevented reference CLI scenarios;
package auto-discovery of /Applications intentionally invalidates historical absent-PATH
fixtures. Those fixtures use the byte-identical unbundled release; package cold/warm checks
separately prove auto-discovery. A later quality fixture lacked fc-list on PATH; only failed
subsets were rerun with the correct environment. The runner now handles the exact fault
report format and records terminal process state before parsing reports.

Coverage review fixed omission of gzip raw traces. The corrected union records 4145 actual
tools/call requests, 84/110 names with successful replies. Remaining 26 names are live tools;
this does not establish complete arguments/error/native/appearance coverage. Prior coverage
reports remain historical; use migration/current-headless-stage27-coverage-with-package-v2.json.
Selected passing attempts reference their actual original reports/raw directories in
migration/results/current-headless-{release,package}-stage27-verified/comparison.json.
Failed attempts are preserved; no normal server/package source changed in this cycle.
Native GUI, current production measurements, Windows/foreign-platform and licensing gates
remain pending. Review snapshots: migration/results/current-headless-stage27-review/.
