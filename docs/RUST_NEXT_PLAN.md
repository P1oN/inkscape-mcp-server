# Active backlog

Updated **2026-10-05** after PR #9 merged and the CI repair passed on `c66583b`. This file contains only
unfinished work. Implemented Python removal and authoring plans are in
[history](history/plans/python-removal-and-authoring.md); completed product milestones are
in [the milestone index](history/README.md). Current behavior/validation is in
[the handoff](AGENT_HANDOFF.md).

## Priority 1 — validation and delivery

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Clean-machine Apple Silicon installation | Exercise source bootstrap on a separate clean macOS 15+ host: Apple SDK installation, pinned downloads, quarantine, first setup, client registration, render and reinstall. | Host/tool versions, commands, actual prompts/failures and preserved settings/drawings. Existing-host acceptance is insufficient. |
| Real Claude Code acceptance | Use an installed Claude client in an isolated profile where supported; check registration scope, handshake, first workspace request, reconnect and disconnect. | Actual client version/config scope and results. Synthetic CLI routing remains separate. |
| Complete review and prepare a release | Resolve the uncompleted CodeRabbit review through a suitable bounded review process; choose a release candidate after the validation above. | Reviewed scope and unresolved findings are explicit; candidate source/build identity, required package checks and distribution notes recorded. Publishing assets or replacing an installed runtime requires a separate request. |

## Priority 2 — live drawing workflow

These remaining items come from the original product roadmap. First check current code
against the requested behavior; do not reimplement the existing headless batch/edit pipeline.

| Task | Scope to design or validate | Completion evidence |
| --- | --- | --- |
| Computed live styles and relationships | Bounded effective paint and links to gradients, masks, clipping, patterns and clones; retain explicit unknowns for unsupported CSS. Headless quality-report CSS advice does not complete live computed-style support. | Representative synthetic and realistic drawing cases, preserved references and documented live limits. |
| Realistic mask/pattern/clone acceptance | Inspect and preview drawings using these resources; identify unsupported live paths before extending them. | Correct object identity/bounds/render results with unchanged originals; headless and GUI evidence distinguished. |
| Reviewed live change packages | Plan/preview before application, stale-document guard, explicit results/recovery; research one native Undo transaction for a bounded multi-operation package. Existing working-copy `apply_edits` is already atomic. | Defined supported operations/failure semantics, appropriate regression/CLI checks and owned GUI Undo/Redo evidence before promising live atomicity. |
| Artist pilot and installation usability | Test copies of real artwork on a separate Mac; observe selection, preview/refinement and recovery. Evaluate a setup/update flow without terminal use from those observations. Existing CLI setup and agent guidance are implemented. | Recorded usability problems and scoped follow-up fixes; originals and user sessions preserved. |

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
