# Current Rust improvement plan

The improvements are committed on `codex/install-client-responsiveness` in
[PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8), targeting `main`.
Inspect Git status and [AGENT_HANDOFF.md](AGENT_HANDOFF.md) before continuing.
Historical stage38 completion notes are in [history](history/rust-stage38-plan.md).

The requested installation, client onboarding, skill update, build identity, responsiveness,
archive acceptance, documentation separation and uninstall improvements are implemented in that PR.
Validation and its limitations are recorded in the handoff and local acceptance reports.

Remaining follow-up scope after review:

1. Run the source bootstrap on a truly clean Apple Silicon machine, including Apple's SDK
   installation, downloads, quarantine and client installation. Existing-host archive
   acceptance does not establish this.
2. Run Claude Code registration on a real installed client; synthetic CLI/config regression
   checks establish argument routing only. Codex registration is exercised in an isolated home.
3. Measure representative workload latency before finer concurrency changes. Headless
   edits/rendering remain serialized intentionally; do not weaken snapshot/record ordering.
4. Repeat native GUI acceptance only when changes affect live behavior. Preserve current
   user windows; use owned synthetic documents and distinguish GUI results from headless tests.
5. Windows and foreign-target executions remain deferred. Investigate the historical live
   group incident only on recurrence; capture first divergent tree/PNG, selection, document
   and window IDs, audit/wire/logs and Undo state before any restart or closure.

Review and merge/release remain separate actions. PR publication has not replaced the
published v0.1.1 archive or the user's configured runtime.
