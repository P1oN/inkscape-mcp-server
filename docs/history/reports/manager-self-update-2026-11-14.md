# Reviewed Manager self-update — 2026-11-14

Issue: [#63](https://github.com/P1oN/inkscape-mcp-server/issues/63).
Implementation starts at current `origin/main` `1b1b8f3b65f5378ac0df75dd8218e7d2c8a2fe82`
in the isolated `codex/manager-self-update` worktree. The original checkout's uncommitted
files remain untouched. No release or tag is created.

## Implementation

- Independent asynchronous read-only Manager discovery uses immutable fixed-repository
  releases and a separate `inkscape-mcp-manager-update-v1.json` contract. The strict legacy
  runtime manifest/archive layout remains compatible. Stable/prerelease, OS/architecture,
  numeric version eligibility, actual native Manager and separately measured bootstrap identities
  drive eligibility; an older latest publication cannot hide another eligible build.
- Verified staging checks complete archive/inventory, bundle identity and trusted Developer
  ID Team `DN263AX69U` before a downloaded helper's isolated identity probe. Target text,
  helper and launcher compatibility must preserve both current and rollback pairs before
  readiness/publication. Missing settings is not a dependency.
- Review shows installed/target identities, destination, exact bootstrap changes and hashes,
  publisher, channel, compatibility and restart requirement. Explicit confirmation is
  followed by a retained-current-helper readiness handshake; competitors refuse before
  the confirming Manager exits and are checked again afterward.
- Existing Manager backups/journal and installer file publication implement a compensating
  app/bootstrap transaction. Adjacent stages are fully synced, including the `/Applications`
  copy path and roots on another volume. Actual new-Manager startup acknowledges activation;
  failed relaunch restores the previous idle app/bootstrap. Later changes/links refuse
  recovery before mutation. Outcomes return to the next inspection.
- Runtime/text selection, complete skills/configuration/secrets, preferences, HOME/CODEX_HOME,
  installation and channel remain preserved. Existing MCP sessions keep their helpers.
  MCP startup/reconnect remains offline and never opens an app. No MCP schemas/instructions
  changed, so catalogs and an existing user server were not regenerated/restarted.

See [the operational contract](../../install/manager-self-update.md).

## Local automated validation

Rust 1.99.0, current macOS Apple Silicon host, existing Inkscape CLI 1.4.3. Checks use
synthetic profiles and owned output only:

- Both locked Cargo graphs: 378 runtime tests passed (three opt-in ignored), 30 tooling tests
  passed (one opt-in ignored), formatting and all-target Clippy with warnings denied.
  Runtime tests run serially locally because short subprocess deadlines are sensitive to
  concurrent compilation. The earlier complete suite passed. One final attempt alongside
  tooling validation failed the unchanged 500-ms `each_request_keeps_its_own_fixed_deadline`
  test at initialization; that failure log is retained. The isolated final run passes
  completely; its log is retained alongside all other results.
- Manager regressions cover independent/incompatible-runtime discovery, channels, older/
  identical builds, mutable releases, target/network/timeout errors, read-only missing-settings
  checks, exact Review identity, publisher/content refusals, shared locks and later edits.
- Ten owned subprocesses exit abruptly with both locks held: after app backup/publication,
  each of three bootstrap publications, and the five corresponding recovery boundaries.
  Lock reacquisition, old pair restoration, retained candidate and repeated recovery pass.
  Separate cases reject dangling target/stage/backup links, missing backups and subsequent
  app/bootstrap edits without changing any other destination.
- Strict native Objective-C compilation and automated AppKit/task-I/O tests exercise separate
  Manager offers, runtime refusal, damaged roots, generation/root/channel/review guards,
  failures/deadlines and action routing without launching a management helper.
- Package headless acceptance passes on the ready archive and finalized ad-hoc Runtime layout.
  Native installer acceptance passes all 19 publication boundaries, real doctor/MCP, legacy/
  custom roots, bootstrap identities, customized/conflicting skills and client preservation.
- Independent component update acceptance passes with `runtime_exercised=true`: current build
  `be2265bb2a3dc3f7`, separately compiled Git-free baseline `29f4d26a55709fb7` (`unknown` revision).
  This is a fixture identity difference, not an older published distribution.
- Ad-hoc `build-distribution` passes its relocation/signature/DMG checks. Its Runtime/Manager/DMG
  is not notarized or publication eligible. Release publication and signed-distribution guard
  fixtures, Bash syntax and actionlint 1.7.12 pass; fixtures never publish a release.

The locally qualified release package records source base `1b1b8f3` plus the unpublished
working-tree build ID `be2265bb2a3dc3f7`; its DMG SHA-256 is
`04b7f5305f634e2887ce8bfa716dc7d18766783f71bd73863ca09a5c15b0119b`.
These local artifacts are not the subsequently committed/hosted CI final bytes.

## Owned Developer ID native replacement

The opt-in `native_signed_manager_replacement_restart_and_recovery` fixture passed in
81.67 seconds with real Developer ID signatures from the trusted publisher. It signs only
owned synthetic apps/helpers, not a new distribution. The signing subprocess uses the
explicitly selected existing identity; app/profile operations use a separate HOME,
CFFIXED_USER_HOME and nonstandard CODEX_HOME.

The fixture exercises verified staging, coordinated bootstrap and Manager-only replacement,
fresh/prepared-runtime/damaged roots without settings, competing running Managers, a failed
restart followed by complete rollback and retry, actual Launch Services reopen and actual
startup acknowledgment. It checks restored HOME/CODEX_HOME, installation and selected channel;
complete synthetic secrets/configuration/custom conflicting skill bytes and runtime/selector/
settings remain identical. A pre-update MCP process remains connected throughout.

After each of five isolated publication/recovery paths and successful replacement/retry,
real permanent-launcher sessions complete initialize, exact tools/list comparison against
`migration/contracts/live-false_raw-false_full_full.json`, and get_workspace_info. The retained
pre-update session also serves those requests afterward. Tool counts are not hard-coded.

An earlier native fixture exposed Launch Services spelling the system `/var` alias. Resolving
only the running helper's executable to its canonical `/private/var` location fixes that
restart refusal without relaxing untrusted destination/symlink guards. The failed startup
record and subsequent passing logs remain retained. Native stream/process waits are bounded.

## Independent review and boundaries

A separate agent, uninvolved in writing the changes, reviewed the complete implementation
and new fixtures repeatedly. Justified findings were corrected: later dangling links,
pre-exit competitor checks, exact bootstrap Review, cross-volume staging, target compatibility,
full stage durability, a publisher test that reaches verification and bounded native harness
reads. Final re-review found no remaining actionable findings. This is static agent review;
hosted required checks/review-thread resolution and merge status belong to the linked PR.

Still unverified: interactive product Manager Review/confirmation GUI; a browser-downloaded,
notarized final distribution and real Gatekeeper/offline first-open behavior; native
`/Applications` administrator/permission interaction and cross-volume acceptance on another
mounted filesystem; another Mac; app/bootstrap power-loss/media/ENOSPC qualification; a real
published Manager manifest served through GitHub (no release was authorized). Developer ID
synthetic fixtures do not qualify those release gates. Existing user installations, profiles,
skills, artwork, application sessions and system protection settings are unchanged.

## Retention

Task-owned output/evidence lives under `.inkscape-mcp-local/manager-63`; its ownership marker
records the base, budget and recreation route. Compact logs, reports, archive identities,
matching executables and complete unique dSYM bundles are retained. Temporary accepted
packages/profiles are removed only after preserving/verifying their recoverable copies and
checking processes/open files. Warm task Cargo caches remain reusable; unrelated outputs and
the original checkout are outside cleanup scope. No scheduled cleanup is created.
