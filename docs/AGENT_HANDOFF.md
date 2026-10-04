# Agent handoff — current status

Read README.md, CONTRIBUTING.md and docs/agent-usage-guide.md. Check code and Git status
and preserve any uncommitted work. The improvements were committed as `a6f2180` and pushed
to `codex/install-client-responsiveness` in [PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8),
targeting `main`. Follow its checks/review for remote status; local results below are separate.
The published v0.1.0/v0.1.1 assets and the user's installed runtime were not replaced. Historical checkpoint counts
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

Local review follow-up on `a4dc71f` (uncommitted): client onboarding now enforces a fixed
request deadline even during unrelated notifications. Uninstall ignores recorded clients
that have switched to a different launcher, while still refusing removal when another client
uses this installation. Skill ownership behavior is intentionally unchanged.
Validation: Python runtime tests: 21 passed; Ruff check/format: 48 files; `git diff --check`;
isolated real Codex and synthetic Claude client management acceptance:
`migration/results/client-review-fixes-20261004-1/comparison.json`.
Rust code and MCP contracts/instructions are unchanged; no new native GUI acceptance claimed.

Local CI follow-up (uncommitted): SHA-pinned checkout/upload-artifact are updated to
v7.0.1 and setup-uv to v10.2.0, all declaring `runs.using: node24`. The uv 0.12.22 pin,
disabled cache and artifact upload settings remain intact. Workflow YAML, exact-SHA upstream
action metadata and configured input compatibility were checked locally; remote CI has not
been rerun for these changes.

CI on `499a81d` timed out in final discovery; `3649425` passed discovery but hung on
its first `create_document` in shell responsiveness acceptance. Concurrent local startup
stress reproduced the hang: queued blocking work ran only after stdin closed. This matches
[Tokio's confirmed 1.52.0 blocking-pool regression](https://github.com/tokio-rs/tokio/issues/8056).
Tokio is now pinned to the upstream fix in 1.52.1, with Cargo.lock updated.

Patched local validation: Rust: 224 passed, 1 ignored; fmt/clippy; Python: 16 passed;
Ruff: 48 files; release discovery: 16 exact matches; 512 fresh sessions with 8 concurrent
servers; per-call/shell responsiveness and cancellation. Evidence:
migration/results/pr8-ci-startup-patched-1/comparison.json,
pr8-ci-responsiveness-patched-1/comparison.json and pr8-ci-discovery-patched-1/comparison.json.
The native CI now includes bounded startup stress (128 sessions, 4 concurrent servers).
Discovery uses pinned Python 3.12.14 and retains completed/pending traces on failure;
reader errors and timeouts report their cause. Follow the latest PR check for remote evidence.

Before the Tokio patch, PR #8 review fixes were validated locally on 2026-10-04: Rust: 224 passed, 1 ignored;
fmt/clippy; Python: 11 passed (6 helper tests and 5 installation regressions); Ruff: 46 files;
shell syntax; setup: 51 checks; bootstrap: 9 checks; discovery: 16 exact matches.
The new tests cover normal/linked Git build watches, committed versus working-tree archive
identity, skill-move failure and settings-archive rollback. Cancellation cannot publish partial
runtime capabilities or replace a successful cached probe.

The new Git-free working-tree archive passed native build/setup/skill/client handshake,
create/edit/Inkscape CLI render/save, uninstall and reinstall on this Mac. Its compiled
revision is `unknown`, with matching binary/package build identity. Conflicting skill merge
proposals remain outside client discovery; customized skill bytes and drawings survive removal.
Codex registration uses a real isolated profile; Claude CLI routing remains synthetic.
Owned per-call/shell cancellation and follow-up edits pass. Ready package checks:
launcher: 11; doctor: 10; notices: 17 (184 crates).

Review evidence (ignored local artifacts): migration/results/pr8-review-install-2/comparison.json,
pr8-review-source-2.tar.gz, pr8-review-setup-1/comparison.json,
pr8-review-bootstrap-1/comparison.json, pr8-review-clients-1/comparison.json,
pr8-review-cancellation-1/comparison.json, pr8-review-discovery-1/comparison.json,
pr8-review-launcher-1/comparison.json, pr8-review-doctor-1.json and pr8-review-notices-1.json.
The archive is explicitly unpublished; existing pinned host tools/caches were reused.

Before review fixes, relocated ready archives also passed both engines with empty PATH,
private Python/bus, original preservation, transaction/rollback/no-op and render/export checks;
see migration/results/improvements-package-{per_call,shell}.json and
improvements-final-evidence.json. Those results describe the earlier build.
Tool/schema counts remain unchanged; approval descriptions and llms manifests are corrected.
No new native GUI acceptance, clean-machine installation or foreign-target result is claimed.
Existing published Release assets and the user's installed runtime/config remain unchanged.

## Boundaries

Keep bounded typed tools, edit pipeline, snapshots, Operation Records, approval/no-op rules,
originals, workspace/symlink protections, IDs/references and appearance checks. Author vectors;
no bitmap tracing or embedded raster substitute. Use semantic named groups and scene layers.
Do not close/kill user Inkscape windows. No commits, PRs, publication or messages unless asked.
