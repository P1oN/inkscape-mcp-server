# Agent handoff — current status

Read README.md, CONTRIBUTING.md and docs/agent-usage-guide.md. Check code and Git status;
this tree includes unpublished local changes. Preserve them. Historical checkpoint counts
and package paths are in [the checkpoint archive](history/agent-checkpoints-through-2026-10-04.md),
not current validation claims. [RUST_NEXT_PLAN.md](RUST_NEXT_PLAN.md) is the active plan.

## Current implementation

- Rust STDIO server; legacy Python MCP/parity workflow retired. Python remains private
  helper runtime and development/packaging tooling. MCP startup/reconnect never launches GUI.
- Current setup supports automatic first builds, `--rebuild`, offline `--local-tools`,
  ready packages, installed build metadata and preserved saved options/Sentry settings.
- Optional `--connect-client codex|claude`: bounded handshake, required tool discovery,
  first workspace request, then native client CLI registration. Config snippets and checks
  are available through scripts/mcp-client.sh. Tests use isolated client profiles.
- Skill updates store an upstream baseline, merge customizations with `--update`, preserve
  prior versions and leave installed files unchanged on conflicts. Legacy customized
  installs without a baseline require manual merging.
- `uninstall.sh --client codex|claude` disconnects the matching user entry and archives
  local settings/builds and an owned default-location skill; drawings survive. Other
  connected clients must first be disconnected. Custom-destination/shared skills remain.
- Compiled revision/build ID appear in `--version`, package metadata, Sentry release and
  allowlisted error/transaction tags. No drawings, arguments or paths are added to telemetry.
- Synchronous tool/resource kernels run in bounded blocking workers; headless filesystem
  work stays serialized by an async gate. Registry metadata is cloned/released for ordinary
  operations; registration keeps its publication lock. Discovery/workspace stay responsive.
  Cancellation stops owned per-call/shell headless work; it is not transaction undo.
- Reference-safe deletion and durable registry publication fixes from the preceding local
  work are retained. Server approval tokens remain nonempty strings; the client must
  obtain confirmation for each operation. Do not claim server-authenticated authorization.

## Validation

Validated 2026-10-04: Rust223 passed/1ignored, fmt/clippy; helper6; Ruff45 files;
shell syntax and workflow YAML parsing; setup50 and bootstrap guards9; client ownership/
config guards (real isolated Codex, synthetic Claude). Current source archive passed real
build/setup/skill/handshake/create/edit/render/save/uninstall/reinstall on this Mac.
Relocated ready archive passed both engines with empty PATH/private Python/bus, original
preservation, transaction/rollback/no-op and render/export checks. Discovery16 exact matches, doctor10, launcher11 and notices17
(184 crates) pass; per_call/shell cancellation reaps the owned process and leaves no PNG.

Evidence: migration/results/improvements-install-final/comparison.json,
improvements-cancellation-final/comparison.json, improvements-client-guards-1/comparison.json,
improvements-package-{per_call,shell}.json, improvements-setup-final/comparison.json,
improvements-notices-final.json and improvements-discovery-final/comparison.json.
Final build/evidence binding: migration/results/improvements-final-evidence.json.
These are ignored local artifacts; the source archive is an explicitly unpublished working-tree
snapshot. The MCP surface remains unchanged; llms manifests were regenerated from this package.
No new native GUI acceptance, clean-machine installation or foreign-target result is claimed.
Existing published Release assets and the user's installed runtime/config remain unchanged.

## Boundaries

Keep bounded typed tools, edit pipeline, snapshots, Operation Records, approval/no-op rules,
originals, workspace/symlink protections, IDs/references and appearance checks. Author vectors;
no bitmap tracing or embedded raster substitute. Use semantic named groups and scene layers.
Do not close/kill user Inkscape windows. No commits, PRs, publication or messages unless asked.
