# Regression contracts and historical evidence

The Rust implementation is current; the Python MCP server and paired parity workflow are retired.
Active development/acceptance commands are in [CONTRIBUTING](../CONTRIBUTING.md).

- `contracts/`: frozen discovery/prompt/error fixtures and the shared authoring policy used
  by the current Rust server. These are active inputs, not historical prose.
- Tracked comparison/provenance/evidence-binding JSON: historical reports bound to named
  source/artifact hashes. Their original paths and checkpoint labels are retained.
- `results/`: Git-ignored local raw traces, synthetic SVG/PNGs, packages and acceptance logs.
  An old success belongs to its recorded binary; these artifacts are not current release assets.
- `vendor-notices/`: retained dependency/provenance material with its recorded scope.

All narrative history is indexed in [docs/history](../docs/history/README.md). The previous
long migration README is [archived there](../docs/history/migration-evidence-log.md).
Python source recovery/history is documented [beside its code](../scripts/history/python/README.md).
Use [current status](../docs/AGENT_HANDOFF.md) and [active backlog](../docs/RUST_NEXT_PLAN.md)
for new work; do not execute historical parity commands as a development requirement.
