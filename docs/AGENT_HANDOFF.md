# Agent handoff — current status

As of **2026-10-06**, the v0.1.2 prerelease surface includes installation/responsiveness
(PR #8), complete native Rust runtime and vector authoring (PR #9), computed live inspection
and reviewed packages (PR #10), and vector-quality guards/fragment dry-run (PR #11).
The [v0.1.2 tag](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.2) identifies
the release source; each ready archive records its actual source revision/build ID and
has a separate SHA-256 checksum. Historical v0.1.0/v0.1.1 archives remain unchanged.

Read [README](../README.md), [CONTRIBUTING](../CONTRIBUTING.md) and
[agent usage](agent-usage-guide.md). The [backlog](RUST_NEXT_PLAN.md) contains unfinished
work, and [history](history/README.md) retains earlier checkpoint/acceptance evidence.
Check current code and Git status before relying on recorded results.

## Implemented

Current unpublished follow-up: setup replaces existing MCP registrations and skill trees
while preserving configuration/preferences and supplementing missing optional settings.
Source edits/unknown revisions rebuild automatically. The published v0.1.2 archives retain
the previous preserve/refuse installation policy. See the [upgrade ledger](history/reports/setup-upgrade.md).

- Rust STDIO server, native client manager, separate GUI supervisor, one-shot INX helper
  and socket bridge. Active build/development/packaging use Rust/Bash. Ready runtime
  packages omit Python, wheels and inkex, retaining fixed Bash interfaces, the Objective-C
  context bridge and private D-Bus dependencies.
- Automatic source preparation/rebuild, offline local tooling, ready runtime packages,
  compiled source/build identity, bounded client registration, managed skill merging and
  uninstall archival. Existing settings/customizations/drawings are preserved.
- Bounded blocking workers and serialized edits/rendering, original-file preservation,
  workspace/symlink guards, snapshots, Operation Records, rollback and genuine no-ops.
  Owned process cancellation does not promise transaction undo.
- Workspace discovery/portable artifacts, named groups/layers, appearance-preserving
  reparenting within conservative limits, focused previews, stable-ID replacement and
  bounded declarative repetition. See [usage](agent-usage-guide.md).
- Shared editable-vector guidance, explicit stroke/closed-shape roles, bounded CSS/path
  and hidden-geometry advice. Creation can enforce explicit closure/zero-segment refusal;
  vector-only save refuses raster or unknown resource content before publication writes.
  Fragment dry-run returns a structural candidate without document/history writes.
  No automatic cleanup, general occlusion inference, bitmap tracing or raster substitution.
- Full profile: `live_inspect_objects` and `live_change_package` provide computed static
  paint/resource inspection and reviewed style/transform/text packages bound to document,
  selection, content and canonical edits. Apply uses the existing guarded native helper.
  No-op/refusal/recovery are explicit; runtime Undo verification remains conservative.
  See [reviewed workflow](live/reviewed-workflow.md).

The surface remains **112 tools, seven prompts and 18 resources**. Policy has one source:
`migration/contracts/authoring-guidance.txt`; schema/instruction snapshots and catalogs
are synchronized, not independent policy copies.

## Validation and limitations

| Evidence | Status and scope |
| --- | --- |
| PR #11 `a865fca` | [Native macOS arm64 CI passed](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37372260969), including default-parallel native tests, both Cargo graphs, release/discovery, package/STDIO/setup/CLI checks |
| First CI attempt | No hosted macOS arm64 runner was acquired; no steps ran. A fresh attempt acquired a runner and passed; cancellation was not a source/test failure |
| Vector guards local acceptance | 308 sequential runtime tests passed/two opt-in ignored, tooling 15, both fmt/Clippy graphs, 16 discovery profiles, both CLI authoring modes and ten STDIO suites; [ledger](history/reports/vector-quality-guards.md) |
| Earlier local parallel failure | Managed launch/lock test failed before publication. It remains recorded; a later CI pass does not erase the failure or establish that a suspected race is fixed |
| Native GUI | Owned synthetic live style/text pair, no-op, one Undo/Redo with equal SVG/PNG pairs and stale-request guards passed for the build in the [live ledger](history/reports/live-drawing-workflow.md); not transferred to a later binary |
| Review | Older CodeRabbit reviews were skipped due to file count. A successful status is not by itself a completed human/code review |
| Release vs installed runtime | Distribution publication does not replace the user's configured runtime. Rebuild/select the new package and reconnect MCP to activate it, preserving Inkscape GUI |

Release/source archive checksums and actual package identity belong to their published
assets and release notes. The [release follow-up](history/reports/v0.1.2-release.md) records
documentation reconciliation and the committed-source PAX metadata repair. Historical local build hashes remain in their original ledgers.
Clean-machine Apple Silicon, real Claude, real artwork/artist and broader human-review
race acceptance remain pending or user-deferred; Intel/Linux execution and Windows port
remain separately scoped. Ad-hoc macOS signing is not Developer ID/notarization.

## Next work and rules

Use the [single backlog](RUST_NEXT_PLAN.md). The [archived handoff](history/agent-checkpoints-through-pr11.md)
retains previous checkpoint counts and detailed delivery history.

Follow [AGENTS.md](../AGENTS.md) and CONTRIBUTING: typed bounded tools, existing approval
gates, snapshots/records/no-op handling, original/reference/appearance preservation, no
arbitrary execution or tracing. Startup/reconnect must not launch Inkscape. Preserve user
windows; native acceptance uses explicitly authorized owned synthetic documents.
Approval tokens are client-supplied markers, not authenticated consent. Commit, push,
release or installation requires user authorization; the user authorized documentation,
PR #11 merge and release publication on 2026-10-06. This does not authorize changing their
installed runtime. Restart/reconnect the selected MCP to load changed instructions while
preserving GUI; existing GUI sessions retain their existing native helpers.
