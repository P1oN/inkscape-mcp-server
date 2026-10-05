# Native Rust runtime and tooling

Status: accepted. Recorded retrospectively 2026-10-05; Python removal stages 1–7 merged in
[PR #9](https://github.com/P1oN/inkscape-mcp-server/pull/9), merge `df3272b`.

## Context

The Rust MCP server still depended on project-supplied Python for client management, managed
GUI supervision, native effect/socket helpers and build/acceptance tooling. The user retired
the legacy Python MCP/parity workflow and requested removal of these remaining dependencies.

## Decision

Use five fixed Rust runtime executables, shared bounded SVG kernels and a separate locked
Rust development-tooling crate. Retain thin Bash entry points, the Objective-C context bridge,
YAML CI and required native bus dependencies. Archive retired Python sources outside active
paths; keep historical evidence without requiring repeated Python comparisons.

## Consequences and validation

Ready packages need no project-supplied Python/inkex/wheels. Source builds need Rust and native
build prerequisites, while consumers of ready packages do not need a compiler. Shared kernels
retain conservative refusals and typed execution rather than expanding arbitrary execution.

Regression, STDIO, package and owned-native evidence must be scoped to the actual binaries.
Clean-machine provisioning and foreign platforms remain separate. Source merge does not
replace release assets or installed runtimes. See [helper limits](../live/live-helper-kernels.md),
[stage-7 ledger](../history/reports/stage7-tooling.md) and
[archived implementation plan](../history/plans/python-removal-and-authoring.md).
