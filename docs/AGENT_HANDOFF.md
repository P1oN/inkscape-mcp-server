# Agent handoff — current status

Read README.md, CONTRIBUTING.md and docs/agent-usage-guide.md. Check code and Git status
and preserve any uncommitted work. Installation/responsiveness improvements from
[PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8) are merged locally into `main`
as `a4dc71f`; MCP deadline/client ownership fixes are committed as `ccd1b0d`.
Python removal stages 1–4 are implemented in this branch. Local validation is separate from remote CI.
The published v0.1.0/v0.1.1 assets and the user's installed runtime were not replaced. Historical checkpoint counts
and package paths are in [the checkpoint archive](history/agent-checkpoints-through-2026-10-04.md),
not current validation claims. [RUST_NEXT_PLAN.md](RUST_NEXT_PLAN.md) is the active plan.

## Current implementation

- Rust STDIO server, client manager and separate managed GUI supervisor; legacy Python
  MCP/parity workflow retired. One-shot native insert/edit effects are Rust; Python
  remains for the socket live helper and development/packaging tooling. MCP startup/reconnect never launches GUI.
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

## PR #9 review follow-up (2026-10-04, local)

All three attached review findings were confirmed against PR head `5193631` and
fixed locally while preserving the existing uncommitted stage-4 changes. Both
supervisor fixture compilers honor `RUSTC` and otherwise resolve `rustc` through
PATH. Transform plans compare parsed matrices, retaining the original transform
for exact identity deltas without inverse/parent roundoff; context/invertibility
refusals and real small edits remain intact. The native GUI acceptance runner saves
`launch.trace.json` before replacing the first Wire; `mcp.trace.json` retains the
reconnect trace. This does not reconstruct missing traces from past runs.

Focused current-working-tree validation: 29 Rust tests passed, 1 explicitly ignored
native CLI rendering test (12 helper, 9 supervisor, 1 standalone supervisor, 7 INX).
Both supervisor suites passed with RUSTC unset, an isolated HOME without a Cargo
compiler and a compiler provided through PATH. New regression covers identity
deltas with absent/translate/matrix attributes, transformed parents, accompanying
style changes, singular-parent refusal and real 1e-9 translations. Fmt, all-target
clippy with warnings denied, Ruff on the changed acceptance runner, its trace
save/close/replacement ordering and `git diff --check` passed. No native GUI
acceptance, package rebuild, commit or push was performed for this follow-up.

## Local Python removal stage 4 (2026-10-04, uncommitted at validation)

Stage 4 is locally implemented. `rust/src/bin/inkscape-mcp-inx.rs` is the fixed
one-shot consumer; `helper_svg::oneshot` validates the typed private request and
captured SVG/IDs/native selection, and `helper_svg::apply` computes/applies a fresh
plan to an owned DOM. It preserves the root, namespace metadata, prolog/epilog,
comments/PIs/DTD, mixed text and element tails. Duplicate IDs/local references and
ungroup affine compensation use the shared kernels. Descendant CSS transforms and
nested viewports during ungroup refuse before publication. Text uses a real text
node so ampersands/angle brackets remain literal. Unchanged plans and refusals emit
zero SVG bytes, bypassing native document rebase and extra Undo steps. The input
file is never written. Reads remain bounded/no-follow; result publication uses the
existing descriptor-anchored atomic filesystem primitive and refuses linked results.

The supervisor installs the two INX manifests and a safely quoted wrapper executing
the relocated `bin/inkscape-mcp-inx`. Packages omit the four Python one-shot assets;
setup rejects saved packages missing the native helper and doctor checks its native
architecture without executing it. Socket/live Python and vendor inkex are still
required until stage 5; CPython/wheels/notices cleanup is stage 6. The retained Python
one-shot sources/tests are development/historical fixtures, not a fallback route.
No arbitrary code/shell/extension/environment route or MCP schema/instruction change.
Existing approvals, context UUIDs, snapshots, Operation Records and post-application
SVG/fingerprint confirmation remain in the server pipeline.

The mandatory small prototype passed before switching the route: native insertion,
selection input, one style Undo after a repeated no-op and stale fingerprint refusal.
Inkscape supplies `--id=svg1` on a blank drawing; insertion is selection-independent,
while edit calls retain strict native-selection comparison.

Fresh automated validation: 268 Rust tests passed, 2 ignored in the standard run
(the existing ignored server case and the separate real-CLI render gate). The render
gate was then explicitly run and passed: before/grouped/affine-ungrouped PNG pixels
match. Fmt/clippy; 14 retained Python development tests; Ruff check/format (49 files).
The seven new INX regressions cover all ten semantic operations, complete XML,
references/namespaced metadata IDs/tails, literal text, no-op output, stale guards,
strict fields/cross-mode requests, bounded growth and linked result refusal.
Release discovery matches all 16 frozen configurations exactly: 110 tools, 7 prompts,
18 resources. The exposed surface/instructions are unchanged; llms manifests were
not regenerated. Documentation links, shell syntax and `git diff --check` passed.

The rebuilt relocated ready archive passed both per-call and shell acceptance with
empty PATH: real CLI render/export, approvals/batches/snapshots/rollback, preserved
originals and headless no-ops without audit/transient writes. Acceptance also disables
the copied private interpreter and runs the Rust INX change/no-op CLI against the
frozen fingerprint fixture. Launcher: 11 checks; doctor: 12 profiles without GUI
launch; bootstrap: 9; notices: 18 checks / 192 crates. Native helper Mach-O dependencies
are system libxml2/libiconv/libSystem only. These are fresh stage-4 package results.

Native acceptance uses a separate copied package with its private Python executable
disabled and only an owned HOME/profile/workspace/session. All ten edits and insertion
passed through the actual INX route. Each changed edit was reversed by exactly one
native menu Undo and the SVG compared with its captured pre-edit state (excluding
only namedview UI data). Repeated style/text no-ops and stale IDs/content/selection
added no Undo step; all three stale calls left the SVG unchanged. Duplicate and text
Redo restore their captured after states. Native group/ungroup SVGs render with
identical PNG pixels. Both prototype and final synthetic sessions were restored to
their original blank state and gracefully closed after verifying exact recorded
manifest/context/PIDs; their supervisors/private buses exited and manifests vanished.
A temporary computer-use timeout interrupted selection only; native mutation was
resumed after UI access returned, and the final text/Undo/Redo/shutdown gate passed.
No existing user window/process was closed or signalled.

Evidence (Git-ignored): `migration/results/rust-inx-prototype-native/`,
`rust-inx-{tests,clippy,release,render}-final.log`, `rust-inx-stage4-package-final/`,
`rust-inx-stage4-package-final.tar.gz`, `rust-inx-package-final-build.log`,
`rust-inx-stage4-discovery-final/` and `.json`,
`rust-inx-stage4-{per-call,shell,doctor,notices}-final.json`,
`rust-inx-stage4-{per-call,shell}-with-inx/`,
`rust-inx-stage4-{launcher,bootstrap,doctor}-final/`, and
`rust-inx-stage4-native-final/{acceptance,native-one-step-undo,native-text-undo,native-structural-render,shutdown-acceptance}.json`.
The native record includes the exact helper SHA-256; its copied helper equals the
final ready-package binary. Package/native claims refer to those immutable binaries;
validation-only harness additions afterward do not install or replace them. The user's
installed runtime/client settings, source drawings and vendor executable are unchanged.
No commits/publication. Next pending stage: 5, the socket bridge/perception/geometry.
Linux/Windows native GUI and clean-machine acceptance remain unverified.

## Local Python removal stage 3 (2026-10-04, historical checkpoint)

Stage 3 is locally implemented. `rust/src/lib.rs` exposes the hardened XML parser
and `helper_svg` kernels for future native helpers, without executable, filesystem,
process or MCP entry points. The server calls the extracted fingerprint and fixed
insertion preflight; headless reparenting and the live planner share the existing
finite affine arithmetic/formatting. Iterative element traversal is shared too.
The extracted production kernels retain their existing behavior and error mapping.

Fragment preparation now produces an owned ordinary group with deterministic IDs,
remapped local href/CSS references and retained element mixed content/tails. Fixed
allowlists, 1 MiB/10,000-element bounds and safe parser depth limits remain. The pure
live edit planner covers the ten existing semantic operations, collapsing parent/child
selections and returning typed steps/affected IDs without applying a document. It
checks IDs/collisions, definitions/layers, locks, references, text constraints,
parent/stacking and transforms; guards compare captured IDs/fingerprint and native
selection. Request/output/reference-work limits prevent multiplied style or reference
work from growing without bounds. Matching style/text and unchanged stacking plans
produce no steps. The wire fingerprint v1 representation and consumer obligations
are documented in [live-helper-kernels.md](live-helper-kernels.md).

The planner intentionally refuses complex inline CSS, stylesheet transforms,
singular/ill-conditioned parents and minted descendant collisions. It is not wired
to native publication; At that checkpoint, Python/inkex helpers and the bundled Python runtime remained
active. Stage 4 above supersedes that status and implements plan application and confirm full SVG preservation,
selection/context guards, native Undo, unchanged-result no-op and stale-state
refusal on owned synthetic drawings. This stage does not claim native equivalence.

Fresh automated validation: 257 Rust tests passed, 1 ignored (217 server, 7 shared XML,
12 client, 9 supervisor, 11 shared helper regressions and 1 standalone supervisor
integration); fmt/clippy; 14 retained Python tests; Ruff check/format (48 files).
Release discovery matched all 16 frozen configurations exactly: unchanged 110 tools,
7 prompts, 18 resources. Exposed instructions/schemas are unchanged, so llms manifests
were not regenerated. Documentation local links and `git diff --check` passed.

A newly rebuilt local ready archive passed relocated per-call and shell acceptance
with empty PATH: real Inkscape CLI render/export, approvals, batch/snapshot/rollback,
original preservation, no-op without audit/transient writes, private helper imports
and helper CLIs. Launcher: 11 checks; doctor: 11 profiles without GUI launch;
notices: 18 checks / 192 crates. Those are current-stage package results, not reused
stage-2 acceptance. GUI acceptance was not repeated: this stage extracts the existing
live preflight/fingerprint and adds an unapplied planner; no native helper route changed.

Evidence (Git-ignored): `migration/results/rust-helper-stage3-{tests,clippy,build}.log`,
`rust-helper-stage3-package-final/`, `rust-helper-stage3-package-final.tar.gz`,
`rust-helper-stage3-package-final-build.log`, `rust-helper-stage3-discovery/` and `rust-helper-stage3-discovery.json`,
`rust-helper-stage3-{per-call,shell,doctor,notices}-final.json`, and
`rust-helper-stage3-{per-call,shell,doctor,launcher}-final/`. The user's installed runtime,
client settings, source drawings and existing Inkscape GUI were not replaced or
restarted. No commits/publication at that checkpoint. Stage 4 was the next pending stage then;
its completed prototype/native acceptance is recorded above.

## Local Python removal stage 2 (2026-10-04, uncommitted at validation)

The seven-stage [Python removal roadmap](RUST_NEXT_PLAN.md#python-removal-roadmap) records
stages 1–2 as locally implemented at that checkpoint. Stage 3 above supersedes its next-stage status.
Local review of the stage 1–2 working-tree changes covered the Rust client manager,
supervisor/preparation/process lifecycle, launch/doctor integration, wrappers/setup,
packaging and acceptance harnesses. No confirmed actionable code defects were found.
Fresh automated verification: 246 Rust tests passed, 1 ignored; fmt/clippy; 14 Python
tests; Ruff check/format (48 files); documentation links and `git diff --check`.
This review did not rebuild the ready archive or repeat native GUI acceptance; package/native
claims below remain the prior stage-2 evidence, not new review results.

`rust/package/supervise.py` is removed. Explicit `live_launch` now starts the separate
`bin/inkscape-mcp-supervisor` Rust executable, with the same fixed root/vendor arguments,
setsid, 15-second server launch deadline and surviving-session ownership. Ready packages
include all three Rust binaries; setup rejects/rebuilds saved packages without the supervisor,
and doctor checks its native architecture without executing it. No startup/reconnect/client
check path starts the supervisor or Inkscape GUI. Historical Python process-path checks in
native development scripts now recognize the Rust supervisor.

The supervisor preserves the four-field session manifest, private D-Bus/socket, held
supervisor lock, 0700 session directories, bounded no-follow reads, descriptor-anchored
atomic/fsynced publication and owned-process cleanup. Lock losers do not remove an existing
manifest (the Python finally block could do this). Preparation still installs the same six
fixed helper assets and safely quoted private-Python wrapper, copies only the private app
executable/context bridge, links validated vendor Resources, updates the private plist bundle
identifier and codesigns only the private executable. Source/vendor app and security settings
are unchanged. GUI exit triggers owned-bus terminate/reap and manifest removal; GUI startup
or preparation errors cannot terminate existing Inkscape processes. If publication/wait fails
after a GUI has started, a possibly unsaved drawing is retained rather than killed.

Automated validation: 246 Rust tests passed, 1 ignored (224 server, 12 client, 9 supervisor
including four reused filesystem invariants, one standalone supervisor integration).
The standalone integration runs the actual Rust supervisor with native synthetic bus/GUI
processes, empty PATH and no Python runtime. Cases cover lock competition/manifest retention,
readiness timeout, bus exit, missing GUI, partial preparation/recovery, codesign/missing assets,
plist/helper preservation, permissions, symlink/hardlink refusal, publication and GUI/bus exit.
Fmt/clippy; 14 Python helper/development tests; Ruff (48 files); shell syntax/diff checks passed.
Setup: 51; bootstrap: 9; relocated launcher: 11; doctor: 11; notices: 18 (192 crates).
The new plist crate uses `LICENCE`; notice collection now recognizes that spelling, with a
synthetic archive regression, while retaining hash/path/type/size checks.
The relocated ready archive passed per-call render/export, original/rollback/no-op/approval
checks and private Python helper imports/CLIs. Client management with damaged runtime and
supervisor still passed. Supervisor Mach-O dependencies are system libSystem/libiconv only.
The manifests were regenerated: unchanged 110 tools, 7 prompts and 18 resources.

Native acceptance on this Mac passed using only new isolated HOME/profile/workspace/session:
explicit launch, managed-D-Bus connect, context/document binding, STDIO close/reconnect without
launch, unchanged vendor executable, graceful blank-GUI quit, both owned processes exiting,
owned-bus shutdown and manifest removal. Darwin D-Bus returned UnixProcessIdUnknown for the
peer PID query: the acceptance verifies exact recorded GUI ancestry/private executable and
matching context UUIDs through the pinned unique owner before quit. The first session was
safely closed after repairing that development check; the final acceptance passed end to end.
No pre-existing user window/process was closed or signalled.

Evidence (Git-ignored): `migration/results/rust-supervisor-tests-final.log`,
`rust-supervisor-package-2.tar.gz`, `rust-supervisor-package-final.json`,
`rust-supervisor-native-final/{launch,shutdown}-acceptance.json`,
`rust-supervisor-launcher-final/`, `rust-supervisor-doctor-audit-2/`, `rust-supervisor-notices-final.json`,
`rust-supervisor-{setup,bootstrap}-1/`, `rust-supervisor-client-final/`. The final native evidence records exact PIDs,
root, profile, source executable hash and reconnect/context results.
Python remains necessary for Inkscape SVG/socket helpers and development/package tools;
those have not been migrated. This supervisor still targets the official GTK3 macOS bundle;
Linux/Windows managed GUI and clean-machine acceptance are not established. Installed user
configuration/runtime were not replaced; nothing was committed or published.

## Local Python removal stage 1 (2026-10-04, uncommitted at validation)

Client management has moved from `scripts/mcp_client.py` (removed) to the separate
`rust/src/bin/inkscape-mcp-client.rs` binary. All five actions (`config`, `check`,
`connect`, `disconnect`, `uninstall`) use Rust through the existing Bash interfaces.
Ready packages bundle `bin/inkscape-mcp-client`; setup requires it and rebuilds an
older saved source package lacking it. There is no Python fallback. Client configuration
is inspected as TOML/JSON and modified only through Codex/Claude CLI; CODEX_HOME, Claude
user scope, unrelated/foreign entry guards, recorded-client launcher checks, atomic
records, archive rollback and default-location skill ownership remain intact.

The manager never loads the supervisor/helper runtime. Config/disconnect/uninstall work
with damaged helpers; check/connect require the server to pass the same bounded read-only
handshake/discovery/workspace requests. Fixed 30-second request deadlines include continuous
notifications; response lines/queues and owned-server shutdown remain bounded. No Inkscape
startup, GUI changes, tool/schema/instruction changes or new execution surface were added.
At this stage-1 checkpoint, supervisor and helpers were still Python; stage 2 above supersedes
that supervisor status. Source
builds, packaging and development acceptance runners still require Python.

Validation on this Mac: 12 new Rust client regressions plus 224 server tests passed,
1 existing ignored test; fmt/clippy, 14 Python helper/development tests, Ruff (48 files),
shell syntax and diff checks. The first full Rust run had a transient failure in the
existing managed-session lock test; that test passed independently and the complete
subsequent run passed. No server code was changed to hide it.
Setup: 51 checks; bootstrap: 9; ready launcher: 11; doctor: 10; notices: 17 (190 crates).
The Git-ignored ready archive contains both binaries and passed relocated per-call
package acceptance with real Inkscape CLI rendering/export, original preservation,
snapshots/rollback, approvals and no-ops. POSIX builder synthetic logic also passed.
The package manager has only system Mach-O dependencies (no Python/native helper libraries).

Isolated real Codex and synthetic Claude client guards passed. Relocated ready-package
acceptance exercises all five actions for both clients with no Python on client PATH and
a deliberately damaged private Python runtime/supervisor. It verifies blocked uninstall,
switched-client preservation, owned customized skill/settings archive, restoration and
Claude user-scope uninstall; existing drawings survive. The initial acceptance fixture
used an uncanonicalized macOS owner path; corrected `/var`/`/private/var` fixture identity
passed without a production ownership-policy change.

Evidence: `migration/results/rust-client-tests-final.log`, `rust-client-cli-tests-final.log`,
`rust-client-management-2/`, `rust-client-no-python-final/`, `rust-client-package-2.tar.gz`,
`rust-client-package-final.json`, `rust-client-{launcher,doctor}-final/`,
`rust-client-{setup,bootstrap}-1/`, `rust-client-notices-final.json`,
`rust-client-posix-builder-1.json`. The final rebuilt archive repeated package, manager,
launcher, doctor and notice checks successfully after the last client validation change.
The manifests were regenerated against the rebuilt Rust server: unchanged 110 tools,
7 prompts and 18 resources. Real Claude, native GUI, clean-machine and foreign-target
acceptance remain unverified. Nothing was committed, published or installed into the
user's client profiles/runtime. Earlier validation below is historical context.

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
