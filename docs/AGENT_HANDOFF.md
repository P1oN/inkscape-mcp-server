# Agent handoff — current status

As of **2026-10-05**, the tested `main` source is `c66583b` (documentation cleanup and
CI regression repair); the following documentation-only update records its successful CI.
[PR #9](https://github.com/P1oN/inkscape-mcp-server/pull/9)
merged Python removal stages 1–7 and editable vector authoring quality. Its authoring
commit is `b7caa67`; installation/responsiveness from PR #8 and the `ccd1b0d` follow-up
are already included. Check Git status and preserve uncommitted work before continuing.

Read [README](../README.md), [CONTRIBUTING](../CONTRIBUTING.md) and
[agent usage](agent-usage-guide.md). [Documentation index](README.md) maps the guides;
[active backlog](RUST_NEXT_PLAN.md) is the single list of unfinished work.

## Live workflow branch checkpoint

`codex/live-drawing-workflow` starts from `main` `44e0e30` on 2026-10-05. The
candidate adds `live_inspect_objects` and `live_change_package` to the full profile (112 tools).
Computed paint uses bounded static CSS with explicit unknowns and local resource links;
packages review page previews, bind document/selection/content/canonical edits, then publish
one native candidate through the existing guarded helper and Live Operation Records.
No-op/refusal/recovery boundaries remain explicit. See the [workflow guide](live/reviewed-workflow.md)
and [fresh acceptance ledger](history/reports/live-drawing-workflow.md).
Owned native GUI acceptance passed for the final package: a two-member style/text package,
no-op history, one native Undo/Redo with equal SVG/PNG pairs, and stale native request guards.
The isolated session was restored to blank and gracefully closed. General human-review race
acceptance remains unverified; artist/real-work/separate-Mac acceptance is user-deferred.
Runtime Undo fields remain conservative. Acceptance used an uncommitted candidate; the user
subsequently authorized committing, pushing and opening a PR. No installed-runtime change occurred.

PR #10 review follow-up: the published package schema now conditionally requires non-null
review guards and a nonempty approval token only when `dry_run=false`. Omitted/true dry-run
requests retain their preview contract. Tool descriptions and runtime dry-run notes now
state the scoped synthetic Undo/Redo evidence consistently; per-call verification remains
conservative. The manifests are regenerated from fresh STDIO discovery. This follow-up
changes schema/documentation only, not native publication behavior or installed helpers.
Validation: 64 JSON Schema cases across all four full/live contracts, 16 exact discovery
configurations, regenerated catalogs, release server build, fmt and Clippy passed. The
default-parallel Rust run failed the existing immediate post-release lock assertion in
`live_launch::tests::private_session_and_locks_refuse_link_escape_and_parallel_launch`;
that test passed in isolation. The suspected concurrent fork/lock lifetime race is outside
this schema repair and remains recorded rather than hidden by a retry. A diagnostic full
sequential run passed: 301 tests, zero failures, two opt-in ignored; its retained log is
`migration/results/pr10-review-checks/runtime-serial.log`.

## Implemented

- Rust STDIO server, native client manager, separate GUI supervisor, one-shot INX helper
  and socket snapshot bridge. Active bootstrap, development, packaging and CI use Rust/Bash.
  Ready packages omit project-supplied Python, wheels and inkex helpers. Fixed Bash wrappers,
  the Objective-C context bridge and private D-Bus dependencies remain.
- Source first-use build/rebuild, offline local tools, ready packages and compiled revision/build
  identity. Client registration uses bounded handshakes and native Codex/Claude CLIs. Managed
  skill updates preserve customizations; uninstall archives owned settings/builds/skills.
- Bounded blocking workers preserve discovery responsiveness. Headless edits/rendering remain
  serialized; owned cancellation does not promise transaction undo. The edit pipeline preserves
  originals, workspace/symlink protections, references, snapshots, records and no-ops.
- Working-copy workspace discovery, groups/layers/reparenting, editability advice, focused previews,
  stable-ID subtree replacement and declarative repetition. See [agent usage](agent-usage-guide.md).
- Shared editable-vector guidance in initialization/compose; explicit stroke roles, effective CSS
  fill/subpath checks and conservative hidden/transparent geometry findings. Advice is read-only,
  separate from validity/score, and reports uncertainty. No automatic cleanup, general occlusion
  detector or semantic silhouette inference. The policy source is
  `migration/contracts/authoring-guidance.txt`.

## Validation and delivery

| Evidence | Status and scope |
| --- | --- |
| PR #9 candidate `b7caa67` | [CI passed](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37296446838): both Cargo graphs, native tests, discovery, package/STDIO/setup/CLI checks |
| Current tested source `c66583b` | [CI passed](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37310025578): repaired effect regression, both Cargo graphs, release/discovery, package/notices/STDIO/setup/launcher/CLI/socket/doctor checks |
| Historical post-merge `df3272b` | [CI failed](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37298327949) before the test-budget repair; the original failure remains recorded |
| Local CI regression repair | Delayed stale-reply regression and full default-concurrency runtime suite pass (295 / two ignored), tooling 15, both fmt/Clippy graphs. Only test code changed; [diagnosis/evidence](history/reports/ci-effect-timeout-fix.md) |
| Local authoring candidate | 295 runtime tests passed / two opt-in ignored; 15 tooling tests; fmt/Clippy; 16 discovery configurations; ten STDIO suites; relocated per-call/shell package and real CLI authoring acceptance |
| Native GUI | Earlier owned synthetic GUI evidence belongs to its recorded builds; authoring did not change live mutations or repeat GUI Undo/Redo acceptance |
| CodeRabbit | Requested reviews were skipped because file counts exceeded 100; its successful status is not a completed review |
| Releases / installed runtime | Published previews remain v0.1.0/v0.1.1; merge did not replace release assets or the user's configured runtime |

The local authoring package's hashes, complete commands and limits are in the
[authoring acceptance ledger](history/reports/editable-vector-authoring.md).
Stage-7 evidence is in [its ledger](history/reports/stage7-tooling.md).
Do not relabel the failed historical run or old packages as current acceptance.

## Next work and boundaries

The CI regression repair and remote confirmation are complete. Proceed to clean-machine
Apple Silicon and real Claude acceptance. Feature research and platform work remain in the backlog.
The complete checkpoint chronology is [archived](history/README.md), not a pending task list.

Follow [AGENTS.md](../AGENTS.md) and CONTRIBUTING: typed bounded tools, existing approvals,
snapshots/records/no-op handling, original/reference/appearance preservation and no arbitrary
execution. Author editable vectors, use named semantic groups and scene layers, and follow
the shared policy. Approval tokens are client-supplied nonempty markers, not authenticated consent.
Startup/reconnect must not launch Inkscape. Preserve user windows; native acceptance requires
explicitly authorized owned synthetic documents. No commit, push, release, installation or
messages without user authorization. Deploy changed guidance by restarting/reconnecting the
selected MCP server while preserving the GUI; a running GUI keeps its existing native helpers.
