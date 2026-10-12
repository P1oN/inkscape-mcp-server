# History and acceptance evidence

This is the single index for narrative history. Records retain their checkpoint scope:
“current”, “next”, “uncommitted”, counts, commands and package paths describe their original
builds. Use [current status](../AGENT_HANDOFF.md), [active backlog](../RUST_NEXT_PLAN.md) and
[architecture decisions](../adr/README.md) for new work.

## Delivered milestones

| Milestone | Recorded outcome | Evidence |
| --- | --- | --- |
| Managed macOS workflow, 2026-09-30–10-01 | Document context, everyday edits, live search/previews; scoped native Undo/Redo pilot | [Original product roadmap](plans/macos-product-roadmap-2026-09-30.md), [live prototype log](MACOS_LIVE_PROTOTYPE_LOG.md) |
| Six working-copy improvements, 2026-10-02 | Workspace artifacts, groups/layers, editability, focused preview, stable-ID replacement, repetition | [Checkpoint chronology](AGENT_HANDOFF_LOG.md) |
| Rust migration through stage38, 2026-10-03 | Rust MCP checkpoint with historical Python helpers; scoped package/native/performance review | [Report](reports/RUST_MIGRATION_REPORT.md), [checklist](reports/RUST_COMPLETION_CHECKLIST.md), [original next plan](rust-stage38-plan.md) |
| Installation/responsiveness, PR #8 | Source setup, clients/skill updates, build identity, uninstall, responsiveness/cancellation | [Checkpoints through 2026-10-04](agent-checkpoints-through-2026-10-04.md), [later handoff snapshot](agent-checkpoints-through-pr9.md) |
| Python removal stages 1–7, PR #9 | Five native executables and Rust/Bash tooling; historical Python archived | [Completed plan](plans/python-removal-and-authoring.md), [stage-7 ledger](reports/stage7-tooling.md), [implementation checkpoints](agent-checkpoints-through-pr9.md) |
| Editable vector authoring, PR #9 | Shared guidance and bounded advisory analysis; real CLI synthetic acceptance | [Authoring ledger](reports/editable-vector-authoring.md), [completed design](plans/python-removal-and-authoring.md#editable-vector-authoring-quality-requested-2026-10-04) |
| PR #9 delivery, 2026-10-05 | Candidate `b7caa67` CI passed; merged as `df3272b`. Separate main run failed in managed effect-loss test; no release/installed runtime replacement | [Delivery record](pr9-delivery.md) |
| Managed effect CI regression, 2026-10-05 | Delayed preflight reproduces invalid short test budget; repair pushed as `c66583b`, full fresh native CI passed | [Diagnosis and evidence](reports/ci-effect-timeout-fix.md) |

## Reports and logs

- [Manager window refresh](reports/manager-ui-refresh-2026-10-12.md): native setup
  hierarchy, contextual actions, automated AppKit state/layout checks and scoped
  real-window evidence; local and unpublished.
- [Manager first install and shared Codex registration](reports/manager-first-install-2026-10-12.md): other-Mac failure report, app/CLI registration without executable discovery, full local installer acceptance and isolated native UI check.
- [Manager DMG presentation](reports/manager-dmg-design-2026-10-12.md): isolated
  worktree implementation, editable artwork, native metadata/remount checks and
  local ad-hoc preview; explicitly scoped Finder evidence and release limitations.

- [v0.1.7 signed release](reports/v0.1.7-signed-release.md): current-main publication,
  exact final asset verification, real Claude discovery on signed bytes and retained
  evidence from the initial unpublished candidate; remaining host/client gates.

- [v0.1.6 signed release](reports/v0.1.6-signed-release.md): successful main/protected CI,
  Accepted notarizations, final-byte acceptance and publication; remaining host gates.

- [Remaining Python source and release-script removal](reports/python-scripts-removal-2026-10-10.md): portable Rust release guards, Bash fixtures and removal of the retired source archive.

- [Manager helper deadlines](reports/manager-helper-deadline-2026-10-11.md): bounded pipe I/O and termination escalation; automated subprocess acceptance.

- [Actual storage/process-failure acceptance](reports/storage-failure-2026-10-11.md): APFS ENOSPC, installed startup, retry and one real journal interruption; explicit remaining limits.

- [Signed runtime receipts and launcher concurrency](reports/signed-runtime-receipts-2026-10-09.md): PR #30 merge, main CI, failed final candidate and scoped release fixes.

- [Native macOS installer checkpoint](reports/macos-installer-2026-10-08.md): unfinished Priority 1 implementation and scoped local evidence.

- [Independent updates](reports/independent-updates.md): unpublished launcher, component updates, recovery and scoped CLI/native management acceptance; [original user plan](plans/independent-updates.md).
- [Dependency PR review](reports/dependency-pr-review.md): coordinated Rust graph updates and declined historical/API-breaking updates.

- [Setup upgrade follow-up](reports/setup-upgrade.md): unpublished replacement/config-preservation policy and local regression acceptance.

- [v0.1.2 release follow-up](reports/v0.1.2-release.md): documentation reconciliation, source archive regression and distribution identity/evidence.

- [Handoff through PR #11 validation](agent-checkpoints-through-pr11.md), archived before the v0.1.2 release follow-up.

- Local vector-quality guards, 2026-10-05: [closure, duplicate path nodes, vector-only save and fragment dry-run](reports/vector-quality-guards.md); uncommitted candidate and concurrency limitation.

- Live drawing workflow candidate: [computed styles, resources and reviewed packages](reports/live-drawing-workflow.md); automated/CLI evidence and scoped native GUI Undo/Redo are separate from the user-deferred pilot.

- Rust migration: [report](reports/RUST_MIGRATION_REPORT.md), [long log](RUST_MIGRATION_LOG.md),
  [former migration README](migration-evidence-log.md), [helper investigation](reports/RUST_HELPER_INVESTIGATION.md).
- Packaging/provenance: [checkpoint report](reports/RUST_PACKAGING.md), [packaging log](RUST_PACKAGING_LOG.md).
- Review/security: [review](reports/RUST_REVIEW.md), [review log](RUST_REVIEW_LOG.md),
  [security audit](reports/RUST_SECURITY_AUDIT.md), [audit log](RUST_SECURITY_AUDIT_LOG.md).
- Measurements: [phase measurements](reports/RUST_PHASE_MEASUREMENTS.md).
- Original guides with historical native results: [managed macOS](reports/macos-live-prototype-through-pr9.md),
  [document context](reports/document-context-through-pr9.md), [everyday edits](reports/everyday-edits-through-pr9.md),
  [real illustration discovery pilot](reports/live-discovery-through-pr9.md), [Sentry verification](reports/sentry-through-pr9.md).
- Agent chronology: [early log](AGENT_HANDOFF_LOG.md), [through 2026-10-04](agent-checkpoints-through-2026-10-04.md),
  [through PR #9 preparation](agent-checkpoints-through-pr9.md).
- Release history: [current changelog](../../CHANGELOG.md), [inherited changelog](original-changelog.md).

- [Real Claude Code protocol acceptance](reports/real-claude-protocol-2026-10-11.md): reproduced v0.1.6 discovery failure, modern response envelopes and legacy wire invariants.

## Evidence storage

Tracked JSON comparisons, provenance and hash-bound snapshots remain in [`migration/`](../../migration/README.md).
Local raw packages/traces/SVG/PNGs remain Git-ignored under `migration/results/`; availability on
another checkout is not guaranteed. Retired Python sources were removed on 2026-10-10;
their original bytes remain in Git history. The archive index centralizes navigation
without rewriting recorded JSON paths/hashes or duplicating binary artifacts.

Markdown moves normalize links for the new location; record content and limitations are preserved.
Exact original file bytes remain accessible at their original Git revisions. Historical commands
may reference removed scripts and should not be executed as current development requirements.
