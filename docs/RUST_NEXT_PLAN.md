# Current Rust improvement plan

Current work is unpublished; inspect Git status and [AGENT_HANDOFF.md](AGENT_HANDOFF.md).
Historical stage38 completion notes are in [history](history/rust-stage38-plan.md).

The requested installation, client onboarding, skill update, build identity, responsiveness,
archive acceptance, documentation separation and uninstall improvements are implemented locally.
Validation and its limitations are recorded in the handoff and local acceptance reports.

Remaining follow-up scope:

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

No publication or personal runtime replacement is implied by this local implementation.
