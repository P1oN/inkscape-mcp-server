# Agent handoff — current status

As of **2026-10-07**, the v0.1.2 prerelease surface includes installation/responsiveness
(PR #8), complete native Rust runtime and vector authoring (PR #9), computed live inspection
and reviewed packages (PR #10), and vector-quality guards/fragment dry-run (PR #11).
The [v0.1.2 tag](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.2) identifies
the release source; each ready archive records its actual source revision/build ID and
has a separate SHA-256 checksum. Historical v0.1.0/v0.1.1 archives remain unchanged.

Read [README](../README.md), [CONTRIBUTING](../CONTRIBUTING.md) and
[agent usage](agent-usage-guide.md). The [backlog](RUST_NEXT_PLAN.md) contains unfinished
work, and [history](history/README.md) retains earlier checkpoint/acceptance evidence.
Check current code and Git status before relying on recorded results.

## Release follow-up — 2026-10-09

PR #30 is merged as `6499d29ea3a067963efa3ecb07134c2fb7b668d1`.
[Main native CI 37851864135](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37851864135)
passes and supplies build `f80f0fd90eef4ad9`. Local signing/notarization of those
exact CI inputs succeeds, but final installer/update qualification exposed missing
per-actor runtime receipts; concurrent launcher startup also exposed lock starvation.
That candidate remains unqualified and unpublished.

The follow-up verifies and records the installed bootstrap's exact bytes and signer
when a management helper stages a runtime, and independently verifies probe callers.
Launch releases the recovery lock after reading the selector/settings snapshot, before
hashing immutable runtime/text trees. The signed release gate now requires 128 successful
sessions with four workers against the final installed launcher; installer success is
recorded only after its last independent probe.

Local validation includes 37 library tests, nine serial update-manager tests, tooling's
20 tests, Clippy/format and release guard fixtures. Modified signed tooling passes the
installer/update matrices against the original signed CI Runtime. A freshly compiled,
Developer ID signed helper passes isolated installation and 128/four-worker startup
(16.65 seconds); it is not a newly notarized Manager or fresh CI artifact.
PR #31 is now merged as `97707eac7f60723affd083dd72f5c407cd653a99`; its quick/native
CI and review status passed. Fresh main CI `37970554352` passed. GitHub now has
`release-signing` and `release-publication`, restricted to `main` with P1oN review.
Repository Team ID is `DN263AX69U`; all three signing secret names and required
notarization variables are present. Secret contents were not retrieved. First protected run `37973923331` passed
candidate validation but stopped before import because the installed Rust 1.99.0
toolchain was not selected. Preparation now pins its tooling-build environment;
the regression fixture starts with inherited `stable`. A protected rerun and final
signed qualification remain required. No release has been published. See the [scoped ledger](history/reports/signed-runtime-receipts-2026-10-09.md).

## Priority 1 development checkpoint — 2026-10-08

This branch implements typed first-install preparation/activation/recovery,
per-user Manager relocation, Runtime app layout, native installer UI and protected
signed-release preparation/publication jobs. The latest complete **local** signed candidate
is Runtime/helper build `1a52d477e623f8d5` (dirty source revision
`8762cfe238a0074a58c63c062b8f703c481a2ee7`) in
`.inkscape-mcp-local/priority1-signed3-complete2`. Developer ID Team `DN263AX69U`
is verified. Its Runtime, Manager and DMG are Apple Accepted and stapled; all tickets
validate. Final DMG SHA-256 is
`f84aed5a88c6decaa55209903554168845d393ca1c36d2358e52b6e258e58aa8`.

The local notarization profile is restored. Recovery reuses original Runtime submission
`1609561d-6737-44f7-9453-40098fd160da` and Manager submission
`39afddcd-d7a9-444b-911c-daea6e68004b`; no resigning or duplicate submission occurs.
The fixed local assembly creates one DMG submission,
`bd3fb7e8-fbd5-48b6-890f-4b1d5d64cc82`. Retained input/log/CodeDirectory identities
are verified before stapling. This is recorded local recovery, **not** a successful
main-CI candidate or protected whole-job interruption qualification.

`priority1-signed-final3/report.json` independently verifies 25 signed paths, Developer ID
Team/authority, secure timestamps and applicable hardening; all three tickets; current-host
app and DMG Gatekeeper; actual runtime/launcher extraction; and the Manager in a read-only
mounted DMG without launching it. The mount is detached. Runtime archive/offline payload,
code inventory and Runtime bundle remain unchanged through reuse. Final update manifests
and checksum sidecars bind actual assets. `LOCAL-PREPARATION.json` and the report remain
publication_eligible=false. No synthetic CI/publication receipt is created.

The finalized Runtime package passes relocation/rendering/approval/no-op/preservation
acceptance (453 files). The **stapled** Manager passes the 19-boundary installer matrix,
including custom-engine preservation, missing-engine picker recovery, foreign/ownership
refusals, legacy/custom-root migration, skill and component recovery. Signed-payload
failed-probe mutation is intentionally skipped; the separately scoped ad-hoc fixture
covers it. Distinct-runtime instructions/runtime/combined updates and rollback pass,
with real native doctor/STDIO/workspace probes and synthetic client profiles.
Reports: `priority1-signed-package3`, `priority1-signed-installer3-final`,
`priority1-signed-updates3` and `priority1-signed-final3` under `migration/results/`.

PR #30 review repairs isolate unsafe/invalid/oversized client configurations, report
client diagnostics in the Manager, and skip unmanaged/foreign-owned skills without
adoption. Staging retains signer verification bound to the runtime inventory and exact
helper bytes; normal launch checks that stored result without invoking `codesign`.
Signed-release preparation limits credential inheritance and supports empty optional
arguments on Bash 3.2. Publication verifies and publishes drafts by numeric release ID.
These source changes postdate the signed candidate above; its notarization/native
acceptance evidence does not qualify the repaired helper or Manager bytes.

Review-repair validation: Rust test targets pass, with the nine update-manager tests
run serially after a parallel run exceeded the startup initialization deadline.
Tooling's 20 tests, both Clippy/format checks, release publication/refusal/recovery
fixtures, Bash 3.2 empty/populated arguments and credential-inheritance fixtures,
actionlint and the Manager Objective-C syntax check pass. Native GUI and fresh
signed-artifact qualification were not repeated for these source changes.

Before these review repairs, Rust validation recorded 337 passing tests/two opt-in
ignored, tooling 20, both Clippy graphs and format checks. The engine repair preserves executable saved
paths and repairs only missing/non-executable saved paths without changing other
settings. The compiled adapter fixture exercises the actual approved headless engine;
copied-app first execution timed out with quarantine retained and remains unqualified.
No MCP surface/instruction changes require an installed-server restart in this iteration.

Earlier complete build `7ad414e1314fd62b` retains separately bound native Cocoa
relocation/workspace/first-install/no-op evidence and signed instructions-only reuse
with exact Runtime bytes/submission. Those GUI observations are not transferred to
new helper bytes. Fixed `resume-notarization` recovery of retained containers and
changed-ZIP/same-Team-changed-payload refusals remains separately recorded.

The four-job release flow now checks public repository `SIGNING_TEAM_ID` before
retrieving a candidate and passes that same value through preparation, verification
and publication. Six actual validation-shell cases, actionlint, signed-release guard
fixtures and existing publication fixtures pass. The setup guide distinguishes repository
Team configuration from protected signing secrets/variables. No GitHub configuration
or credentials were exported/written. Last read-only inspection finds no release
environments, signing secrets or variables; Claude Code is absent on this host.

Priority 1 remains open: protected CI setup/execution, a successful committed main-CI
candidate, complete signing-job interruption qualification, browser-quarantined
other-Mac online/offline/client acceptance, clean source bootstrap, real Claude and
broader release review are unqualified. The user has authorized committing/pushing this
candidate and opening a draft PR for CI/review. No release is published.
See the [distribution contract](install/macos-manager-distribution.md) and
[scoped ledger](history/reports/macos-installer-2026-10-08.md).
The user's installed runtime/client settings and Inkscape windows remain untouched.

## Implemented

Current unpublished sources implement independent text/runtime updates: a permanent native
launcher, typed verified assets, guarded skill merging, transaction recovery/rollback and a
macOS management app. Native packages now contain six Rust executables. Migration remains
opt-in; the user's installed runtime and clients were not changed. See the
[guide](install/independent-updates.md) and [acceptance ledger](history/reports/independent-updates.md).
Local CLI/package and explicitly authorized isolated management-window checks passed;
published GUI downloads, live Inkscape reconnect and full media-failure qualification remain
in the backlog. An earlier default-parallel run reproduced the native launch/lock regression. The
2026-10-08 checkpoint above repairs and verifies it; historical failures remain in the ledger.
PR #26 review repairs add bounded startup lock waiting, absent-client/skill preservation,
window-close protection and verified distinct-runtime CI acceptance. Final local checks:
326 runtime tests/two opt-in ignored, 17 tooling tests and both fmt/Clippy graphs passed;
see the ledger for build-bound package and automated Cocoa delegate evidence.

Current sources (PR #12): setup replaces existing MCP registrations and skill trees
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

## Dependency review

Earlier coordinated patches from PR #15 are incorporated. A follow-up replacement for
Dependabot PRs #21–25 aligns base64 0.23.1, toml 1.1.6 and toml_edit 0.25.15 across both Cargo
graphs, refreshing both locks and explicitly parsing TOML documents in package-notice readers.
Local runtime/tooling tests, both fmt/Clippy graphs and release/package notice checks passed;
hosted required CI remains pending on the replacement PR. Historical Python and manifest-only
sha2 0.11 updates were declined; see the [review ledger](history/reports/dependency-pr-review.md).

## Repository maintenance

Local development storage now uses small development/test profiles (no debug symbols or
incremental caches; release symbols unchanged). Failed private builds clean their staging;
successful builds skip duplicate archives. Successful source setup sweeps marked old packages
while retaining recent/selected/running/workspace paths. Automated acceptance can use
`dev-tools.sh --temporary-output` to delete passing scratch output and retain failures.
See [policy and explicit cache cleanup](install/development-storage.md). Historical evidence,
unmarked builds and independent-update profiles are preserved; no MCP surface change.
Validation: isolated cleanup/build/output guard fixtures, 17 tooling tests, tooling Clippy,
both format checks, Cargo manifest parsing, Bash syntax, release guard fixtures and
actionlint v1.7.7 passed. Tooling target rebuilt to approximately 663 MiB locally (previously
5.2 GiB; this is a measured checkout, not a future size guarantee). The older installed
package doctor probe refused linked assets before publication and retained its scratch
output; no new native GUI acceptance or installed-runtime change is claimed.

The repository was detached from its fork network on 2026-10-06; GitHub reports `fork=false`.
Git history and exported PR/release metadata were backed up beforehand. Existing PRs and releases
remained available after detachment. Issues, Dependabot alerts/security updates and private
vulnerability reporting are enabled; Wiki is disabled. Main requires PRs, resolved threads and
quick/native CI with no force-push/deletion or bypass. Release tags are protected and new releases
are immutable. The maintenance workflow changes provide weekly dependency updates, issue forms,
separate early checks and publication from verified main CI artifacts; see
[release operations](install/github-builds.md#publishing-a-new-distribution).
Local release guard fixtures validate refusal and staged publication without publishing a release.

## Next work and rules

Use the [single backlog](RUST_NEXT_PLAN.md). The [archived handoff](history/agent-checkpoints-through-pr11.md)
retains previous checkpoint counts and detailed delivery history.

2026-10-08: another-Mac installation feedback identified missing first-install UX in
Manager, stale loading/error state and repeated macOS component approvals. The user
requested an implementation plan and has requested Apple Developer access; membership and local signing/notarization credentials are now verified; protected CI
credential setup and signed artifact qualification remain pending. The [active installation/signing/release plan](RUST_NEXT_PLAN.md#smooth-macos-installation-and-signed-releases--implementation-plan)
scopes the offline DMG/native wizard, preservation/recovery, explicit bootstrap upgrade,
Developer ID/notarization and evidence-bound release preparation/publication. v0.1.4
published independent updates and v0.1.5 retained them; earlier “unpublished” statements
above retain their original checkpoint context. Planning changes do not implement the
wizard/signing or authorize publication/installed-runtime changes. Validation for this
documentation update: link/path consistency and `git diff --check`; no native GUI or
signed-package acceptance was performed.

Follow [AGENTS.md](../AGENTS.md) and CONTRIBUTING: typed bounded tools, existing approval
gates, snapshots/records/no-op handling, original/reference/appearance preservation, no
arbitrary execution or tracing. Startup/reconnect must not launch Inkscape. Preserve user
windows; native acceptance uses explicitly authorized owned synthetic documents.
Approval tokens are client-supplied markers, not authenticated consent. Commit, push,
release or installation requires user authorization; the user authorized documentation,
PR #11 merge and release publication on 2026-10-06, then this independent-update
documentation/PR on 2026-10-07. The latter authorizes committing/pushing this branch and
opening a PR into `main`; it does not authorize merging, publishing a release or changing
the installed runtime. Restart/reconnect the selected MCP to load changed instructions while
preserving GUI; existing GUI sessions retain their existing native helpers.

Cleanup PR review follow-up: relative/bare MCP commands now refuse cleanup conservatively.
Temporary acceptance cancellation supervises only its owned child, bounds shutdown to five
seconds and retains output. Isolated relative-process and stalled-child fixtures pass.
