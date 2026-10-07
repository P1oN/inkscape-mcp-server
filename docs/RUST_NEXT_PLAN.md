# Active backlog

Updated **2026-10-07** after independent-update implementation and scoped acceptance.
This file contains only
unfinished work. Implemented Python removal and authoring plans are in
[history](history/plans/python-removal-and-authoring.md); completed product milestones are
in [the milestone index](history/README.md). Current behavior/validation is in
[the handoff](AGENT_HANDOFF.md).

## Priority 1 — validation and delivery

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Clean-machine Apple Silicon installation | Exercise source bootstrap on a separate clean macOS 15+ host: Apple SDK installation, pinned downloads, quarantine, first setup, client registration, render and reinstall. | Host/tool versions, commands, actual prompts/failures and preserved settings/drawings. Existing-host acceptance is insufficient. |
| Real Claude Code acceptance | Use an installed Claude client in an isolated profile where supported; check registration scope, handshake, first workspace request, reconnect and disconnect. | Actual client version/config scope and results. Synthetic CLI routing remains separate. |
| Complete broader review and release qualification | Resolve the previously skipped CodeRabbit review through a bounded review process and complete the host/client acceptance above before a stable release. v0.1.2 remains a prerelease; successful status alone is not proof of completed review. | Reviewed scope and unresolved findings are explicit; source/build identity and distribution checks stay bound to each archive. |

## Independent updates — remaining release qualification

The requested implementation is available in current unpublished sources. See the
[installation guide](install/independent-updates.md), [original plan](history/plans/independent-updates.md)
and [acceptance ledger](history/reports/independent-updates.md). No installed runtime was
changed and no release was published.

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Published management GUI downloads | After an authorized release, exercise instructions-only, runtime-only and combined downloads and the no-update state in an isolated native app profile. | Native observations tied to actual published manifests and package identities; current component-success evidence is backend/CLI acceptance. |
| Live reconnect and retained helpers | With explicit GUI authorization, reconnect an isolated client after updating and exercise owned synthetic Inkscape documents while an older helper path is retained. | Actual client/runtime identities, unchanged user windows and native results. No Inkscape GUI was launched for update acceptance. |
| Storage/media failure qualification | Extend automated journal/move-boundary and post-selector write-failure checks with real ENOSPC, media failure and process termination. | Old or new pair remains launchable, recovery is idempotent, and skill edits are preserved. |

Clean-machine installation, real Claude and broader release review remain in Priority 1.
Automatic scheduling, differential downloads, launcher self-update, Windows and Developer ID
signing/notarization remain future scope rather than implemented capabilities.

## Priority 2 — live drawing workflow

The implementation from these items now provides bounded computed live paint/resource
inspection and reviewed style/transform/text packages in the full profile. Synthetic
resource identity/bounds/CLI previews and automated guards/publication checks are in the
[acceptance ledger](history/reports/live-drawing-workflow.md); supported scope is in the
[workflow guide](live/reviewed-workflow.md). Remaining acceptance is listed below.

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Real-artwork computed styles and mask/pattern/clone acceptance | The user deferred real artwork on 2026-10-05 until copies are supplied. Use the pilot protocol; include unsupported CSS, clone instance paint, linked resources and renderer refusals. Existing synthetic CLI results do not establish realistic-drawing acceptance. | Correct identity/bounds/render and explicit uncertainty on copies of actual drawings, with original hashes preserved; headless and GUI evidence distinguished. |
| Human-review race acceptance for live packages | Owned two-member style/text GUI apply/no-op/Undo/Redo and deliberately stale native request guards passed on 2026-10-05. Still observe actual intervening human edits, selection and window/document changes after a retained review; include transform and broader-selection cases. | Build-bound retained reviews and before/after captures proving refusal without mutation; scoped Undo/Redo for the added cases. Runtime verification fields remain conservative. |
| Artist pilot and installation usability | User-deferred until copies of real work and a separate Mac are available. Observe installation, selection, preview/refinement and recovery; derive a terminal-free setup/update proposal from those observations. | Separate host/tool/build identity, observed problems and scoped fixes, preserved originals and user sessions. The documented protocol and current CLI setup are preparation, not completed usability evidence. |

## Research and deferred work

- **Latency:** measure representative workloads before changing concurrency. Separate startup,
  inspection, edit, rendering and live transport costs. Preserve serialized snapshot/record ordering;
  historical Python/Rust benchmarks are not a current performance baseline.
- **General occlusion:** demonstrate a bounded read-only design with supported scope, reference
  protection and explicit uncertainty. Bounding-box overlap is not proof of complete coverage.
  No automatic deletion or silhouette conversion is planned.
- **Platforms/signing:** execute prepared Intel macOS/Linux jobs before claiming compatibility.
  Windows needs native filesystem/process support. Developer ID/notarization require a separate
  distribution decision; existing ad-hoc signatures do not establish them.
- **Historical live group incident:** investigate only on recurrence. Preserve the first divergent
  SVG/PNG, selection/window/document IDs, audit/wire logs and Undo state before restart or closure.

## Rules for taking a task

Use CONTRIBUTING checks appropriate to changed code and update the handoff with fresh evidence.
Regenerate manifests when the MCP surface or instructions change. Repeat native GUI acceptance
only for changed live behavior, on explicitly authorized owned synthetic documents; preserve user
windows. Keep original files, IDs/references, approvals, snapshots, records and no-op behavior.
Do not revive the retired Python MCP/parity workflow. A plan is not authorization to commit,
publish, launch a GUI, change an installed runtime or send messages.
