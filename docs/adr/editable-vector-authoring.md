# Shared vector guidance and explicit advisory roles

Status: accepted. Recorded retrospectively 2026-10-05; implemented in `b7caa67`, merged via
[PR #9](https://github.com/P1oN/inkscape-mcp-server/pull/9).

## Context

A plausible render can conceal geometry that is difficult to edit: fragmented silhouettes,
filled stroke-only lines, independent strokes combined as subpaths or redundant construction.
Names, counts, open paths and overlapping bounds cannot reliably establish artistic semantics.

## Decision

Deliver one shared authoring policy from `migration/contracts/authoring-guidance.txt` through
MCP initialization and compose. Use named semantic groups, deliberate editable vector geometry
and subject-appropriate silhouettes. Extend the existing read-only quality report with bounded
explicit stroke roles, CSS/subpath checks and conservative hidden/transparent findings.

## Consequences and validation

Advice remains separate from validity/score. Unsupported CSS, ambiguous references or exhausted
limits produce uncertainty. General occlusion and automatic silhouette/cleanup conversions are
deferred. Reviewed repairs reuse existing approvals, reference guards, snapshots and records;
compare renders after structural changes and preserve artwork outside the request.

Regression and real CLI fixtures exercise guidance consistency, structure, read-only reporting,
refusals, no-op/restore and pixel-preserving approved cleanup. This does not prove general model
compliance or native GUI Undo/Redo. See [agent usage](../agent-usage-guide.md) and the
[acceptance ledger](../history/reports/editable-vector-authoring.md).
