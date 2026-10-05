# PR #9 delivery — 2026-10-05

Python removal stages 1–7 and editable vector authoring merged through
[PR #9](https://github.com/P1oN/inkscape-mcp-server/pull/9) at 10:42:52 UTC.
Authoring commit: `b7caa677a5449efc4b8b91f6861759a9640c35d2`.
Merge commit: `df3272bf5a7029f4f3c57e2c5ee97e5932b1f6c9`.

The [candidate run 37296446838](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37296446838)
passed at 10:42:24 UTC before merge. Both Cargo graphs, native tests, release/discovery,
package, notices, STDIO, setup, launcher, relocated CLI/socket/doctor checks passed.

The separate [main run 37298327949](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37298327949)
failed in Native tests. At `rust/src/live_managed.rs:1064`,
`effect_loss_stale_missing_mismatch_and_context_switch_stay_uncertain_without_retry`
asserted the `effect-stale` case: actual `Err(Connection("DBus request timed out"))`,
expected `Err(Uncertain)`. The binary cohort recorded 223 passed, one failed, one ignored.
This is not the whole-runtime test count; later cohorts/package checks did not execute.
Cause remains unresolved; investigation is carried into the [active backlog](../RUST_NEXT_PLAN.md).
The pre-merge pass must not conceal the later failure or establish a release-ready main build.

CodeRabbit skipped requested full/incremental reviews exceeding its 100-file limit.
Its successful status is not completed review evidence. No paid review or scope exclusions
were introduced. Merge did not publish new release assets, install a runtime, update client
configuration or restart/close Inkscape. Local `main` was fast-forwarded with a clean tree.
