> Historical checkpoint. Counts, commands, package paths and “current” labels
> apply only to the recorded build. See the [history index](README.md).

# Agent handoff — current Rust candidate

Current local defect fixes (2026-10-04, not committed/published): delete_object and
apply_edits refuse deletion of externally referenced subtree IDs (href/XLink/CSS URL,
encoded fragments, timing/accessibility references); stylesheet ambiguity refuses.
Registry publication now follows durable index persistence, with old-index rollback
on reported write/sync failure; failed candidates never enter active in-memory entries.
Unreadable/nonregular registry destinations refuse before candidate copies. Ambiguous
late persistence retains candidate bytes for diagnosis. Test-only late-write hook is
absent from release builds. Originals and existing user GUI remain untouched.

User selected the client-confirmation model for approval: documentation, skill and
all 16 MCP initialization contracts explicitly say the server checks only nonempty
strings and does not authenticate/bind/expire tokens. Client must obtain user confirmation
for each operation. Do not claim server-verified one-time authorization.

Setup fixes: saved workspace/Inkscape/live/engine preserved with explicit-option
precedence; parameter/Sentry/config-file validation before builds; config-directory
destinations fail without false success. Recorded committed source revision mismatch
triggers rebuild; explicit --package wins. Unknown revisions/uncommitted edits warn
and require explicit --bootstrap. Failed updates retain saved config. CI triggers now
include skills/** and run real-STDIO defect acceptance. No client-config integration,
new Sentry release tracking or concurrency redesign in this scope.

Validation: Rust223 passed/1ignored; fmt/clippy; Ruff41 files, shell syntax/actionlint,
skill validation; setup49, bootstrap guards9; real STDIO defects5 and discovery16 exact
matches; ready package notices17/184 crates and launcher11. Empty-PATH private runtime,
bus, real CLI render/export and transaction acceptance passed. llms manifests regenerated.
Final evidence binding: migration/results/defects-final-validation.json; package
migration/results/defects-ready-final-package. No new native GUI acceptance. Existing
Release assets and configured user runtime have not been replaced.

The setup/skill entries below record preceding checkpoints, not additional pending fixes.

Current local setup UX: fresh source ./setup.sh automatically provisions/builds;
subsequent runs reuse a saved complete package. --local-tools forces a rebuild with
existing tools only (no uv/Python preparation; Cargo --offline). --package selects a
ready runtime. Compatibility aliases --build (local) and --bootstrap (explicit automatic
rebuild) remain; conflicting modes/package options fail before mutation. CI source
setup now exercises the default, with --local-tools on other prepared targets. Previously
published v0.1.1 still needs --bootstrap; its archive was not changed.
Automated launcher acceptance passes 37 checks (default first build/reuse, offline
local mode routing, explicit ready-package selection, conflicting options, aliases,
failed-build config preservation, skill/Sentry paths). Evidence:
migration/results/default-setup-final-acceptance/comparison.json. Bootstrap guards9,
Ruff, shell syntax, actionlint and diff checks pass. Build routing uses isolated inert
builders; this is not a new full native package build or GUI acceptance.

Local addition: portable skills/inkscape-mcp with Codex UI metadata and optional
setup.sh --install-skill codex|claude. scripts/install-skill.sh supports a custom
skills destination, honors CODEX_HOME, avoids runtime execution and preserves differing
existing skills. Ready package builder includes skill/installer; source archives include
committed files. Existing Release assets are unchanged. No MCP surface/instructions
change, so llms manifests are unchanged. See docs/install/agent-skill.md.
Skill frontmatter validation, shell syntax and Ruff pass. Earlier skill-only launcher
acceptance passed 28 checks, including installation from the package bundling helper, opt-in,
idempotence, customized-skill preservation, symlink refusal and isolated client paths:
migration/results/skill-setup-acceptance/comparison.json. No new native GUI acceptance.

User requested a Release source archive for local bootstrap. build_source_archive.py
exports committed sources with SOURCE_REVISION, excluding Git metadata/local settings.
Package builder handles source downloads without .git and does not inherit parent-repo
identity. Source archive acceptance must use a truly extracted, Git-free copy.

Published source preview [v0.1.1](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.1)
at db190b9c8ac106e7686d0aa18f492e058a632313. Uploaded source archive SHA-256:
a8696c1a71d828a8be01a618aed5323017ba591f68c264f000153bec61485bc3.
Exact extracted archive passed --fresh-tools compilation and cleanup, notices17 and
empty-PATH per_call MCP/bus/render/export/transaction acceptance. Evidence:
migration/results/source-bootstrap-notices.json, source-bootstrap-cold.json and
source-archive-guards (nine synthetic guards). No new native GUI acceptance.

User-authorized source bootstrap publication to main (2026-10-04): setup.sh --bootstrap on Apple Silicon
macOS15+ provisions missing Rust1.99/private Python3.12.14 with pinned rustup/uv inputs,
and six SHA-256-pinned temporary bottles without installing Homebrew. Bridge/server compile
locally; owned temporary tools/cache/target/native directory is removed on exit. Existing
installations and prior packages stay intact; packaged helper Python remains required.
Apple CLT/Xcode is a shared prerequisite: missing SDK starts Apple's installer dialog and
asks the user to rerun, without accepting OS authorization or uninstalling Apple tools.

Forced fresh-tools acceptance (no reused Rust/Python): temporary root removed, package
.inkscape-mcp-local/build.mecp7C/package survives; launcher11, notices17, per_call and shell
empty-PATH archive installs/private Python/bus/real CLI/transaction checks pass. Raw evidence:
migration/results/bootstrap-fresh-tools.json, bootstrap-fresh-cold.json, bootstrap-fresh-shell.json.
Eight synthetic download/archive/path/failure-cleanup guards and setup20 pass; workflow linted.
Native placeholder relocation preserves SBOMs/build provenance. PCRE2 10.49 source notices
include LICENCE.md/sljit license; equivalent gettext source URLs were content-hash verified.
See docs/install/local-bootstrap.md. No native GUI or truly clean Apple-tool installation
acceptance; no quarantine removal, notarization or one-prompt guarantee. Current Release
v0.1.0 is unchanged. CI bootstrap step is configured; its remote status must be checked separately.


Pending config-only onboarding fix: default setup does not execute server/doctor,
Python imports or Inkscape. --check explicitly diagnoses before saving; --build is
required for a fresh source build; reruns reuse the saved package path. No quarantine
removal or TCC changes. 20 inert setup/privacy/TTY checks; hidden-input harness now
waits for terminal ECHO-off. User confirmed v0.1.0 Release shows Gatekeeper “cannot check for malicious software”
during setup. Config-only setup avoids diagnostic imports but is not a Gatekeeper fix.
No one-prompt guarantee is asserted for ad-hoc signatures. v0.1.0 unchanged.
Real rebuilt archive passes launcher11 and empty-PATH per_call CLI/MCP/runtime/bus
acceptance (migration/results/config-only-cold.json); no new GUI/signing acceptance.

User-authorized Release v0.1.0 published as an Apple Silicon preview, tag at verified
source 5345f619add78004d3ecc9ca83abd5a6205788db. Archive and SHA-256 attached;
GitHub asset digest matches 48af10da30278ae1f4131d1b74c707be15551b3dc15160a75c83aa2ac91d024d.
https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.0
No additional platforms or native GUI validation claimed.

Apple Silicon GitHub build verified: run 37154851232 succeeded for source
5345f619add78004d3ecc9ca83abd5a6205788db. Downloadable artifact 11285725976
contains archive/checksum; SHA-256
48af10da30278ae1f4131d1b74c707be15551b3dc15160a75c83aa2ac91d024d.
CI passed Rust220/1ignored, helper6, exact discovery, packaging/notices, security,
source setup, Sentry wizard, launcher/doctor and archive per_call/shell acceptance.
Downloaded archive additionally passed local empty-PATH per_call acceptance,
private runtime/bus, real CLI rendering/export and transaction invariants:
migration/results/github-build-5345f61-cold.json. No local settings in archive.
This is CLI/package acceptance, not new native GUI evidence.
Run: https://github.com/P1oN/inkscape-mcp-server/actions/runs/37154851232


GitHub builds: rust-migration.yml now defaults to macos-15 Apple Silicon on relevant
main pushes/PRs and manual dispatch. Manual all-posix retains the other prepared targets.
Verified installable archives/checksums upload only on success, separately from always-retained
build evidence; DSN disabled in CI. Includes hidden-input Sentry setup acceptance. See
docs/install/github-builds.md. Runner GLib 2.88.3 has an exact SHA-256-verified GNOME notice supplement alongside
local 2.90.0. Notice acceptance selects SBOM versions while retaining URL/hash/path
refusals and mandatory runtime license checks.
Remote packaging also exposed Homebrew D-Bus @rpath inputs; builder now resolves
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
