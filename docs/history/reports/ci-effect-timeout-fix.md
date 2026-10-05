# Managed effect regression timeout — 2026-10-05

Status: fixed, committed and pushed to `main` as `c66583b`; the fresh
[CI run 37310025578](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37310025578)
passed on that exact revision. Local validation preceded the commit. The failed
[main run 37298327949](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37298327949)
remains a failed historical run; the earlier PR candidate passed separately.

## Failure and diagnosis

`effect_loss_stale_missing_mismatch_and_context_switch_stay_uncertain_without_retry`
failed in `effect-stale`: actual `Connection("DBus request timed out")`, expected `Uncertain`.
An initial isolated local run passed. The regression had changed the shared bus timeout
to 300 ms before calling `set_text`, so read-only selection/export preflight inherited
the same short budget. A read-only timeout before dispatch correctly returns `Connection`;
it cannot exercise the intended post-dispatch stale-reply path.

A bounded test-only fixture delay of 600 ms on `query-x` reproduces the identical
assertion failure with the old 300 ms budget. This establishes the faulty test mechanism;
the original CI log does not identify which preflight RPC was delayed or prove the
runner's precise scheduling cause. No new production transport defect is inferred.

## Repair and retained guarantees

The matrix now uses a bounded five-second test budget for preflight and reply completion.
Its stale case retains the 600 ms delayed read, preventing reintroduction of the invalid
300 ms shortcut. All six cases still assert their exact error classification, exactly one
effect dispatch, one selection fence, no retries, released operation depth/lock and removed
request/result files. Missing and mismatched replies still reach the real completion deadline.
The fixture delay is capped at two seconds and is compiled only for tests.

Production deadlines, preflight/activation error classification, approval gates and live
mutation code are unchanged. No MCP surface/instruction change or GUI acceptance is involved.

## Fresh evidence

- Deterministic before: one failed test, exact original error (`effect-stale`).
- After: the delayed six-case matrix passes, including lost, stale, missing, mismatched,
  context-switch and lost-but-confirmed-refusal outcomes.
- Full default-concurrency runtime suite: **295 passed, zero failed, two existing opt-in ignored**.
- Tooling suite: **15 passed, zero failed**.
- Both Cargo graphs: fmt check and locked all-target Clippy with `-D warnings` pass.
- `git diff --check` passes. Documentation links and anchors checked after status updates.

Raw logs are retained under Git-ignored `migration/results/ci-effect-timeout-fix/`:
`before.log`, `after.log`, `runtime-tests.log`, `runtime-clippy.log`,
`tooling-tests.log` and `tooling-clippy.log`.

Reproduce the repaired regression without retries:

```sh
cargo test --locked --manifest-path rust/Cargo.toml --bin inkscape-mcp-rust \
  effect_loss_stale_missing_mismatch_and_context_switch_stay_uncertain_without_retry -- --nocapture
```

Use the Cargo/macOS SDK environment from [CONTRIBUTING](../../../CONTRIBUTING.md).
Fresh remote CI confirmation is complete: both Cargo graphs, the delayed effect regression,
release/discovery, package/notices/STDIO, source setup, Sentry/launcher and relocated
per-call/shell CLI/socket/doctor checks passed. The subsequent documentation-only update
records that evidence; it changes no runtime, fixture, contract or workflow inputs.
