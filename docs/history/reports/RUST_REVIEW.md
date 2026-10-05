> Historical checkpoint. Counts, commands, package paths and “current” labels
> apply only to the recorded build. See the [history index](../README.md).

# Historical Rust review record — stage38 and URI export follow-up

This page preserves the named checkpoint and its evidence. Its package paths, counts,
validation and publication statements are historical, not the status of PR #8. For current
work use [the handoff](../../AGENT_HANDOFF.md), [the plan](../../RUST_NEXT_PLAN.md) and
[installation](../../install/install.md). Native results remain bound to the recorded binaries.

The recorded candidate was stage38. Earlier review cycles are retained in
[historical review log](../RUST_REVIEW_LOG.md); their old current-package statements
and retired Python comparison commands are historical.

The subsequent local source review found an SVG export regression: decoded asset filenames
were restored as raw paths, making literal #, % and ? become URI syntax. The source fix
encodes filesystem paths for staged/restored SVG and CSS links and rebases hrefs through
the XML DOM with fragments kept separate. Regression coverage includes alternate XLink
prefixes, CSS URLs/imports, reserved characters in output directories, and export/reopen/render.
Current Rust checks pass217 tests /1 ignored, fmt/clippy and Python lint/format. Real
Inkscape renderer acceptance passes22 cases in each of per_call and shell modes, with
no outside-workspace reads; evidence is in migration/results/uri-export-review-per-call
and migration/results/uri-export-review-shell. These are headless checks, not GUI acceptance.
The stage38 archives have not been rebuilt and their acceptance evidence remains immutable.

Recent review found and fixed rejected-open writes/namespace collisions, outside-workspace
engine dependency reads, capture preflight side effects, oversized validation diagnostics,
partial compare pair publication and unbounded unfinished STDIO input. The stage38 source baseline has
215 passing Rust tests and one ignored test, with format/clippy passing. Each defect has
preserved before/after evidence; actual38 package checks are indexed in
[build/check index](../../../migration/package-stage38-build-comparison.json).

Both cold and warm temporary archive installs pass, alongside exact discovery, helper/runtime,
launcher/doctor/notices, security/special-file and selected renderer/resource checks. These
checks are scoped in [security review](RUST_SECURITY_AUDIT.md) and
[migration report](RUST_MIGRATION_REPORT.md); no suite is presented as exhaustive proof.
Current Rust-only headless measurements pass (10 runs/250 timings), including 30 repeated
exports after fixing same-second automatic filename collisions. Current native GUI acceptance
passes122 fixed checks +5 two-window guards, closed-session reconnect, and live measurements
(5 reconnects/15 stable observations) on unchanged38. Original no-monitor failures are preserved
as history. UI keyboard-shortcut failures were corrected by explicit Edit-menu Undo/Redo;
this required no server fix. CUA reopened the private welcome window after closure during a
state read; left untouched and documented. Final source/artifact hashes are bound in the
[final index](../../../migration/package-stage38-final-comparison.json).
No user document edits are part of acceptance. No commits/PR/publication are authorized.
