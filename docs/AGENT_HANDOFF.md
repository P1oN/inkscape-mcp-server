# Agent handoff — current Rust candidate

GitHub builds: rust-migration.yml now defaults to macos-15 Apple Silicon on relevant
main pushes/PRs and manual dispatch. Manual all-posix retains the other prepared targets.
Verified installable archives/checksums upload only on success, separately from always-retained
build evidence; DSN disabled in CI. Includes hidden-input Sentry setup acceptance. See
docs/install/github-builds.md. Remote packaging also exposed Homebrew D-Bus @rpath inputs; builder now resolves
declared Mach-O loader paths, refuses missing/ambiguous inputs, then relocates and
rechecks the owned dependency closure. Synthetic resolver regressions run in CI.
First remote run exposed a missing test Python environment; CI now installs pinned
private Python/dependencies before Rust tests, and engine fixtures use a manifest-relative
interpreter path rather than the calling directory. Workflow linted with actionlint; actual run status is visible
in GitHub Actions and must not be inferred from local checks.


Publication authorized by the user on 2026-10-03: commit/push codex/rust-migration and
merge into main. Earlier no-publication checkpoint statements below describe prior scope.
Configured Sentry DSN/tokens, local settings, archives and raw results are excluded from Git.
Pre-publication checks: Rust220/1ignored,fmt/clippy; helper6; Ruff37 files.


Sentry onboarding: setup now asks opt-in, hidden DSN and environment label; separate
mode600 data-only .inkscape-mcp-local/sentry.conf. Explicit opt-out overrides ambient
DSN; unattended no-options rerun preserves settings. A local * ignore file also protects
unpacked packages inside other Git checkouts. Management tokens are not needed.
17 inert launcher/TTY/privacy checks pass; see migration/results/sentry-setup-review.
Current installed package: /Users/bm/Documents/repos/inkscape-mcp-server/.inkscape-mcp-local/build.KjfRBi/package
Ready archive: migration/results/packages/inkscape-mcp-macos-arm64-sentry-setup.tar.gz
(no local settings or DSN). Server code/contract unchanged from Sentry boundaries build.


Sentry subsystem extension: optional fixed-category errors now cover process spawn/deadline/
signal crash, typed live uncertain results and internal lock/serialization failures. Each
category emits at most once/minute/process; allowlisted payload drops contexts, threads,
locals, source snippets and paths. Existing panic/fatal and sampled timings remain.
Rust220 passed/1ignored,fmt/clippy and relocated package acceptance pass. Real packaged
MCP render fault injection confirmed process_crash and process_timeout in Sentry with
readable source lines; duplicate crash suppressed. Evidence: migration/results/sentry-boundaries-review.
Subsystem verification package: /Users/bm/Documents/repos/inkscape-mcp-server/.inkscape-mcp-local/build.Jlhkjk/package; launcher local settings updated, client config unchanged.
Already-running MCP needs reconnect to load it; no GUI restart/launch performed.


Sentry setup (2026-10-03): user authorized creating `boryslav/inkscape-mcp-server`,
SDK integration, private package installation and updating their Codex MCP configuration.
The Rust startup initializes optional Sentry before Tokio; missing/invalid DSN disables it.
Panic/fatal errors use fixed scrubbed messages; traces use known tool names without args
or SVG contents. Release debug information uses packed splitting; macOS packages carry the matched dSYM,
and debug-images is disabled. See docs/sentry.md. Real startup panic and packaged tool
trace confirmed through Sentry MCP; temporary panic trigger removed. Rust218 passed/1
ignored, fmt/clippy, doctor, notices17 and launcher11 pass. Manifests regenerated against
new package. Relocated package acceptance passes; its default mutable report
`migration/package-comparison.json` was refreshed and copied to the private Sentry report.
Stage archives and stage-specific evidence remain unchanged. Codex reload confirmed
the final local package `build.TcUUIV`; its dSYM UUID and application source line pass.
The prior migration package status below is historical and does not include Sentry.

Read README.md, CONTRIBUTING.md and docs/agent-usage-guide.md; preserve uncommitted work.
Work originated on codex/rust-migration; user now authorized commit/push/merge to main.
No PR, release publication or further user configuration edits were requested.
User retired the rewritten Python MCP and paired executable comparison workflow. Required
Python/inkex helpers and supervisor remain. Use fixed JSON contracts and Rust regressions.

Post-stage38 source fix: SVG export now URI-encodes asset filesystem paths, separates
real fragments while rebasing DOM hrefs, and preserves CSS URL/import links for filenames
containing #, %, ?, spaces and Unicode. Rust217 passed /1 ignored; renderer22 cases pass
in both per_call and shell modes, including export/reopen/render. Evidence is under
migration/results/uri-export-review-{per-call,shell}. Stage38 archives and their historical
acceptance indexes are unchanged; they do not contain this source fix.

Current package: stage38. Final index migration/package-stage38-final-comparison.json; immutable build index
migration/package-stage38-build-comparison.json,
mutable pointer migration/current-package.json. Actual cold/warm archive checks pass;
Rust 215 passed / 1 ignored, fmt/clippy, lint/format; discovery16/110tools/7prompts/18resources;
security35/special-files12, doctor10/launcher11/notices17 plus BSD2 binding,
asset17/routes8 both engines, diagnostics3/limits5, compare pair publication and STDIO frame checks pass.
Generated llms files match the actual package. Read RUST_MIGRATION_REPORT.md for scope.
Actual38 retry passed122 fixed native checks +5 two-window guards +closed-session reconnect.
Current headless and live measurements pass; no transfer of historical27/29 results.

Completed current stage38 measurements: 10 runs/250 timings, both engines, three SVG sizes,
30 repeat exports with pixels/resource bytes/prior-file preservation. Same-second automatic
export collision on37 fixed with full UUID suffix on stage38. Limits5 input/resource/PNG/pixel
checks pass. Read report for RSS/timing scope; no pure IPC or true peak claim.
Final requirement audit: migration/requirement-audit-stage38-final.json. Current-Mac migration
scope is complete. Native raw captures: migration/results/native-stage38-retry1; live raw:
migration/results/live-measurements-stage38-retry1 (5 reconnects/15 stable observations).
Final evidence binding: migration/package-stage38-final-evidence-binding.json.

Owned session /private/tmp/imcp-native-7ev3le9z is terminal, original GUI15336 and supervisor15332
absent, manifest removed. Closed-session reconnect refused without GUI launch. Before closing,
298 workspace files and final logs were preserved. CUA getAXState after closure incidentally
reopened the private copy as unmanaged PID21952 welcome/version window; left untouched.
Do not inspect cached CUA targets after closure without process checks; reads can open apps.
Failed shortcuts are preserved; successful native Undo/Redo used explicit Edit menu.
Original no-monitor failures and all prior evidence remain immutable history.

Next improvements are iterative Rust work, not missing required migration gates. Preserve
headless/live distinctions, finite acceptance scope, fixed JSON contracts, originals/IDs,
bounded tools, approval/no-op/edit pipeline and GUI ownership protections. Investigate the
historical disappearing-group incident only on recurrence; capture first divergent tree/PNG,
selection, document/window IDs, audit/wire/logs and Undo state before restart/closure.
Windows and clean-machine install are user-deferred. Timing phase refinement is secondary.
Foreign POSIX jobs are prepared, not executed. Signing/provenance limitations remain explicit.
The ready archive bytes did not change during acceptance/documentation updates; use the
final external report for current status. No commits/PR/publication/config edits/messages.
