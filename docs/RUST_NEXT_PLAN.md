# Active backlog

Updated **2026-10-09** with the requested macOS installation and signed-release plan.
This file contains only
unfinished work. Implemented Python removal and authoring plans are in
[history](history/plans/python-removal-and-authoring.md); completed product milestones are
in [the milestone index](history/README.md). Current behavior/validation is in
[the handoff](AGENT_HANDOFF.md).

## Priority 1 — validation and delivery

### Smooth macOS installation and signed releases — implementation plan

Requested on 2026-10-08 after installation on another Mac exposed repeated macOS
component approvals and a Manager stuck at “Reading installed versions…” with
“No launcher found”. Apple Developer Program membership and App Store Connect access
are confirmed active on 2026-10-08. A usable Developer ID Application identity for
Team `DN263AX69U` is now installed, and the local `inkscape-release` notarization
profile successfully authenticates. Local signed/notarized qualification passes;
protected CI credential setup and clean-host delivery remain pending. This section is
the active implementation backlog. The installer/layout/release development checkpoint is
recorded in the [ledger](history/reports/macos-installer-2026-10-08.md). Latest complete
local Runtime/helper build `1a52d477e623f8d5` has Accepted Runtime/Manager/DMG,
validated tickets, 453-file package acceptance, stapled-Manager 19-boundary installer
acceptance and distinct-runtime update/rollback checks. It repairs custom/missing engine
path recovery. Local recovery preserves original Runtime/Manager submissions and records
one new DMG submission; all final assets are explicitly non-CI/local evidence.
Earlier build `7ad414e1314fd62b` retains separately bound native Cocoa first-install/no-op
and signed instructions-only reuse evidence. The local notarization profile is restored.
The complete delivery flow has local implementation and guard coverage, but protected
CI execution, whole-job interruption qualification and browser-downloaded/another-Mac
acceptance remain unqualified. No release has been published.

Release follow-up: PR #30 is merged and main CI `37851864135` passes. Local signed
build `f80f0fd90eef4ad9` was notarized, but failed final installer/update and concurrent
startup qualification. Receipt handoff and launch-lock fixes pass scoped local checks;
the fixes are merged through PR #31 and fresh main CI `37970554352` is running.
Protected environments and signing credential names/variables are configured; actual
protected signing and fresh distribution qualification remain unverified. No release
is published. See the
[2026-10-09 ledger](history/reports/signed-runtime-receipts-2026-10-09.md).

**Outcome:** one Apple Silicon DMG, a native installation/upgrade wizard, preserved
settings/artwork, and Developer ID signed/notarized executable distributions. Normal
users do not run Terminal commands or choose individual component archives. Standard
macOS first-open approval may remain; acceptance must establish that helper/library
launches do not require individual unidentified-developer overrides.

**Implementation baseline at the start of this plan:** `runtime/manager/main.m` only calls an already
installed launcher. `rust/src/update/` implements component verification, migration,
selection, skill merging and journal recovery. `rust/src/client_management.rs` preserves
client configuration. `rust/tooling/src/package.rs` builds six executables, the context
bridge, private D-Bus/GLib libraries and an ad-hoc signed Manager. Native CI in
`.github/workflows/rust-migration.yml` produces verified archives; the Ubuntu publication
job in `.github/workflows/release.yml` and `scripts/publish-verified-release.sh` currently
publishes those archives unchanged. Signing changes bytes, so this last invariant needs
an explicit, evidence-bound transformation stage.

#### 1. Distribution layout and compatibility decision

- Define a stable Manager application installed per user (default
  `~/Applications/Inkscape MCP Manager.app`) and retain the permanent MCP command and
  private state under `~/Library/Application Support/inkscape-mcp`. No administrator
  privileges are required for the default installation. DMG opening offers a single
  obvious application entry point; first run copies/verifies the application into its
  permanent location and relaunches it there before installation. Handle already
  installed apps and running Managers explicitly; never replace an app in use blindly.
- Bundle a complete matching runtime and instruction set for offline first installation;
  online release discovery is optional. No downloaded setup script is executed. Keep
  developer/source installation available as a separate advanced route.
- First technical checkpoint: prove the signed bundle layout and relocation on macOS.
  Place executable nested code in Apple's supported bundle locations, sign dependencies
  inside out after relocation, and verify preservation through copy/archive/extraction.
  Exercise the dynamically loaded `context.so`, private D-Bus/GLib and all six binaries
  under hardened runtime. Derive minimal entitlements from actual failures; do not
  blanket-disable library validation or assume signing Manager covers its payload.
- Decide whether the runtime needs its own app bundle to preserve notarization tickets
  and offline assessment. A stapled DMG alone is not proof that extracted standalone
  helpers work offline. Record the exact layout, ticket coverage and update container
  decision before implementing the packaging contract. The installed filesystem layout
  must not be mutated inside signed code bundles; settings/skills/selectors stay outside.
- Preserve old published releases and format-1 update assets. The current manifests use
  `deny_unknown_fields`: new required fields/layouts need explicit version negotiation or
  a separate versioned manifest. Existing launchers must receive a useful upgrade route,
  rather than an unexpected-field error. Keep compatible legacy assets where feasible;
  do not silently serve a new layout to an old launcher.

Acceptance: a documented layout/compatibility decision, complete code inventory, and a
relocated candidate passing signature checks and existing package acceptance. Final
Apple notarization/Gatekeeper evidence waits for developer credentials.

#### 2. Native installation backend and preservation

- Add fixed typed native CLI operations for installation inspection, preparation,
  activation and recovery, usable by the bundled Manager before a permanent launcher
  exists. Use bounded JSON requests/results and structured progress; GUI remains a thin
  client of Rust. Do not introduce arbitrary command/script execution or invoke an old
  unverified launcher during discovery.
- Discover permanent installations and Codex/Claude user bindings/settings read-only.
  Support legacy `run-mcp.sh` installations, current independent-update installations,
  missing/broken paths and custom roots. Multiple candidates require an explicit choice;
  never adopt a foreign binding based only on a server name. Manual directory selection
  is an advanced recovery fallback, not the first-install instruction.
- Collect client choice and SVG workspace through native controls. Detect Inkscape and
  offer a picker when absent; explain unsupported OS/architecture and missing clients.
  Reuse saved workspace, engine/live and private monitoring settings on upgrade. New
  telemetry remains optional/off by default; never expose a saved DSN in progress/logs.
- Use existing archive/path/ownership guards, lock, immutable runtime/text staging,
  doctor and real bounded STDIO/workspace checks before switching client bindings.
  Extend the durable transaction to cover first installation, client config changes and
  bootstrap/Manager replacement; compensate partial multi-client changes on failure.
  Do not advertise all-or-nothing migration until interruption tests prove it.
- Merge managed skills against their baselines, preserving user customizations and
  reporting conflicts before activation. Respect removed skills; distinguish a user
  choosing a new skill installation from updating an existing destination.
- Allow retry after interruption without duplicate registration or lost backups. Preserve
  legacy sources/packages, unrelated client settings, drawings and running Inkscape.
  Never automatically launch Inkscape or restart/terminate Codex/Claude.

Acceptance: isolated fresh/legacy/permanent/custom-root fixtures, both clients,
missing-client/removed-skill/conflict cases, symlink/ownership/overlap refusals,
failed probe and interruption at each mutation boundary, retry and rollback.

#### 3. Manager installation and update UX

- Replace the current implicit state with explicit loading, not installed, legacy found,
  installed, damaged installation, busy, failed and completed states. Always end loading
  on an error. Show “Install” or “Upgrade existing installation” with detected versions,
  client/workspace choices and a concise review of intended changes.
- Show concrete stages: checking prerequisites, preparing files, checking MCP connection,
  updating client configuration/skills and completing. Report download bytes when known;
  do not invent progress percentages. Offer bounded diagnostics and a retry/recovery
  action. Define cancellation only at safe points; preserve current close protection
  during non-interruptible activation/recovery.
- After success, display installed distribution/runtime/instruction/Manager/launcher
  identities and “Restart Codex/Claude to load the update”. A reopened Manager reads
  actual installed status. MCP startup remains offline and silent except JSON-RPC.
- Label channels in user terms. A prerelease DMG proposes prerelease with an explanation;
  upgrading preserves an existing channel. If stable has no eligible releases, explain
  that fact and offer an explicit switch; never silently opt users into prereleases.
- Keep instructions-only/runtime-only/combined updates and rollback, while making
  “Update” the primary action. Show which components will change and compatibility
  refusals before activation. Instructions-only updates do not require resigning code.
- Provide an explicit signed Manager/launcher upgrade route from the same application,
  with staged verification, recovery and deferred app replacement after exit. Separate
  executable build/version identity from the current launcher compatibility integer.
  Retain old helper/runtime paths while running sessions use them. Automatic background
  self-update remains out of scope; user-requested bootstrap upgrade is in scope.

Acceptance: native isolated management-window checks for every major state, fresh
installation and upgrade without Terminal, failed/retried installation, no-update and
channel-empty states, skill conflict, interrupted operation and bootstrap replacement.

#### 4. Developer ID signing and notarization

- After Apple activation, confirm Team ID, issue/export Developer ID Application signing
  credentials and configure notarization credentials. A DMG/app flow does not require
  Developer ID Installer; add that certificate only if a future PKG is chosen.
- Configure a protected GitHub release environment with certificate/P12 password and
  appropriate App Store Connect API notarization credentials (or a supported Apple
  credential alternative). Use an ephemeral macOS keychain with guaranteed cleanup,
  pinned Actions, least privileges and redacted logs. PR/fork jobs get no signing secrets.
- Sign the full executable inventory with Developer ID, secure timestamps and hardened
  runtime where applicable, verifying expected Team ID/designated requirements and
  minimal entitlements. Fail if any bundled executable/library is missed or remains
  ad-hoc. Include the Manager and bootstrap assets as well as runtime downloads.
- Submit the final supported container using `notarytool`, bound polling/timeouts and
  retain submission ID/status/logs. Staple tickets to supported app/DMG containers and
  validate them; do not try to staple a tarball. Build updater archives from the finalized
  signed/notarized payload and prove assessment after extraction in the decided layout.
- Recompute `FILES.json`, bridge/dependency output hashes, package signing metadata,
  archive SHA-256/lengths and update manifests after all relevant transformations.
  Respect signing order: final inventories stored outside sealed bundles must account
  for stapling; any sealed resource changes require a fresh signature/notarization.
  No self-referential final-container hash inside its signed payload.

Acceptance: verified Team ID and complete inventory, Accepted notarization, validated
tickets, final-byte checksums, and real execution/probes on the finalized package.

#### 5. GitHub release flow

Keep ordinary quick/native CI as the build/regression gate. Refactor publication into:

1. **Validate candidate** (no signing secrets): require a successful `main` push run,
   source SHA and package identity, exact input asset inventory/checksums, new tag and
   immutable reference checks. Refuse expired/missing artifacts or mismatched revisions.
2. **Prepare distribution** (protected macOS signing job): download that exact candidate;
   assemble installer layout, sign/notarize/staple and generate final runtime/launcher
   archives, DMG, checksums and versioned update contract. Do not rebuild application
   binaries. Packaging/signing tools must come from the verified candidate revision or
   an explicitly recorded packaging revision, never unrecorded newer `main` tooling.
   Bind candidate and output digests, build IDs, signing Team ID, notarization IDs and
   packaging revision in machine-readable distribution evidence. Validate signatures
   before executing fixed final-package acceptance; keep this runner isolated from the
   user's installation and GUI. Delete signing credentials before package probes.
3. **Verify and stage**: verify final assets against distribution evidence, run final
   package/installation/update checks including distinct-runtime activation, and create
   a draft only after all required gates pass. No publication on failed notarization,
   incomplete inventories, wrong Team ID or absent final-package evidence.
4. **Publish** (minimal GitHub write permissions): upload the exact verified assets and
   metadata, check remote asset digests/counts, then publish the immutable release.
   Keep serialization and existing no-overwrite/tag guards. Signing retries reuse the
   recorded candidate/submission where possible; partial drafts require explicit recovery
   rather than replacement of published assets.

- For `runtime_tag` instructions-only releases, preserve original immutable runtime
  bytes/references and validate signing/notarization evidence. A user-facing offline DMG
  combines that referenced runtime with current instructions/Manager and records their
  distinct source/build identities. Do not imply all components share the new release
  SHA. Refuse unsigned historical runtime reuse for the smooth signed-install channel;
  historical manual downloads remain available.
- Put the DMG first in generated release notes: prerequisites, one-click install/upgrade,
  preserved settings, reconnect step, channel and component changes. Clearly label
  advanced source/component downloads. Keep archive names/contracts for older clients
  where supported; don't ask normal users to choose among ten technical assets.
- Expose genuine signing/notarization status and published distribution identity in
  metadata/UI; signing success does not make a prerelease stable. Update README,
  installation/compatibility/independent-update guides and handoff when implemented.

Acceptance: release guard fixtures for candidate/output mismatch, wrong signer,
notarization failure/timeout, final hash changes, unsigned referenced runtime, failed
acceptance, expired artifacts, upload failure and draft recovery; Bash syntax,
actionlint and the existing publication fixtures. Fixture publication never uploads a
real release. An actual signed release needs separate user publication authorization.

#### 6. Final acceptance and delivery order

| Increment | Deliverable | Exit gate |
| --- | --- | --- |
| A — layout/backend | Distribution contract and typed installation/recovery backend | Isolated migration/preservation/interruption tests; supported relocation proven |
| B — native wizard | Fresh/upgrade/error/channel/progress UX and bootstrap upgrade | GUI acceptance in a separate profile; no Terminal in the ordinary flow |
| C — release preparation | Developer ID, notarization, final inventories and DMG production | Apple credentials available; finalized signed package passes automated acceptance |
| D — publication | Evidence-bound jobs, compatible update assets and useful release notes | Publication guard fixtures/actionlint; draft asset set verified |
| E — another Mac | Actual browser-downloaded signed release installed over the old version | Gatekeeper/clean-host/client/update acceptance tied to final artifact hashes |

A/B and unsigned C/D fixtures can proceed while Apple access is pending. Do not claim
signed acceptance until real credentials and an Accepted submission exist. Run Rust
tests/fmt/Clippy on affected graphs, existing package/client/update acceptance and
discovery if runtime behavior changes. Documentation-only planning needs link/command
consistency and `git diff --check`; this plan changes no MCP tools or authoring policy.

Final another-Mac matrix: fresh install; upgrade from legacy and current permanent
installations; first online and offline launch from a browser-downloaded quarantined
DMG; extracted runtime/helper execution; real Codex and Claude where available;
instructions-only/runtime-only/combined updates; explicit bootstrap upgrade; rollback;
network failure and interrupted installation; customized/conflicting/removed skills.
Record actual system prompts, Gatekeeper/signature results, OS/client versions,
notarization IDs and artifact/build identities. Never clear quarantine or disable
Gatekeeper to make acceptance pass. User Inkscape windows remain untouched; drawing
GUI checks require separately authorized owned synthetic documents.

Apple references: [Developer ID](https://developer.apple.com/developer-id/),
[distribution signatures](https://developer.apple.com/documentation/xcode/creating-distribution-signed-code-for-the-mac),
[notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow),
[distribution packaging](https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution).

The following existing delivery qualification remains open:

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Clean-machine Apple Silicon installation | Exercise source bootstrap on a separate clean macOS 15+ host: Apple SDK installation, pinned downloads, quarantine, first setup, client registration, render and reinstall. | Host/tool versions, commands, actual prompts/failures and preserved settings/drawings. Existing-host acceptance is insufficient. |
| Real Claude Code acceptance | Use an installed Claude client in an isolated profile where supported; check registration scope, handshake, first workspace request, reconnect and disconnect. | Actual client version/config scope and results. Synthetic CLI routing remains separate. |
| Complete broader review and release qualification | Resolve the previously skipped CodeRabbit review through a bounded review process and complete the host/client acceptance above before a stable release. v0.1.2 remains a prerelease; successful status alone is not proof of completed review. | Reviewed scope and unresolved findings are explicit; source/build identity and distribution checks stay bound to each archive. |

## Independent updates — remaining release qualification

The independent-update implementation was published in v0.1.4 and retained in v0.1.5.
See the
[installation guide](install/independent-updates.md), [original plan](history/plans/independent-updates.md)
and [acceptance ledger](history/reports/independent-updates.md). Historical local
acceptance there does not establish another-Mac signed installation acceptance.

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Published management GUI downloads | After an authorized release, exercise instructions-only, runtime-only and combined downloads and the no-update state in an isolated native app profile. | Native observations tied to actual published manifests and package identities; current component-success evidence is backend/CLI acceptance. |
| Live reconnect and retained helpers | With explicit GUI authorization, reconnect an isolated client after updating and exercise owned synthetic Inkscape documents while an older helper path is retained. | Actual client/runtime identities, unchanged user windows and native results. No Inkscape GUI was launched for update acceptance. |
| Storage/media failure qualification | Extend automated journal/move-boundary and post-selector write-failure checks with real ENOSPC, media failure and process termination. | Old or new pair remains launchable, recovery is idempotent, and skill edits are preserved. |

Clean-machine installation, real Claude and broader release review remain in Priority 1.
Automatic scheduling, differential downloads, unattended launcher self-update and Windows
remain future scope. User-requested Manager/launcher upgrade and Developer ID
signing/notarization are now scoped in the implementation plan above.

## Priority 2 — live drawing workflow

The implementation from these items now provides bounded computed live paint/resource
inspection and reviewed style/transform/text packages in the full profile. Synthetic
resource identity/bounds/CLI previews and automated guards/publication checks are in the
[acceptance ledger](history/reports/live-drawing-workflow.md); supported scope is in the
[workflow guide](live/reviewed-workflow.md). Remaining acceptance is listed below.

| Task | Next action | Completion evidence |
| --- | --- | --- |
| Real-artwork computed styles and mask/pattern/clone acceptance | The user deferred real artwork on 2026-10-05 until copies are supplied. Use the pilot protocol; include unsupported CSS, clone instance paint, linked resources and renderer refusals. Existing synthetic CLI results do not establish realistic-drawing acceptance. | Correct identity/bounds/render and explicit uncertainty on copies of actual drawings, with original hashes preserved; headless and GUI evidence distinguished. |
| Human-review race acceptance for live packages | Owned two-member style/text GUI apply/no-op/Undo/Redo and deliberately stale native request guards passed on 2026-10-05. Still observe actual intervening human edits, selection and window/document changes after a retained review; include transform and broader-selection cases. | Build-bound retained reviews and before/after captures proving refusal without mutation; scoped Undo/Redo for the added cases. Runtime verification fields remain conservative. |
| Artist pilot and installation usability | User-deferred until copies of real work and a separate Mac are available. Observe installation, selection, preview/refinement and recovery; derive a terminal-free setup/update proposal from those observations. | Separate host/tool/build identity, observed problems and scoped fixes, preserved originals and user sessions. The documented protocol and current CLI setup are preparation, not completed usability evidence. |

## Research and deferred work

- **SHA-2 API migration:** PRs #19/#20 were declined because `sha2 0.11` removes the digest
  output's `LowerHex` implementation used by runtime/tooling hashes. If adopting it, migrate
  both graphs together and prove byte-for-byte stable hexadecimal build, package, transaction
  and recovery hashes before updating the pin. Existing `sha2 0.10.9` remains supported.


- **Latency:** measure representative workloads before changing concurrency. Separate startup,
  inspection, edit, rendering and live transport costs. Preserve serialized snapshot/record ordering;
  historical Python/Rust benchmarks are not a current performance baseline.
- **General occlusion:** demonstrate a bounded read-only design with supported scope, reference
  protection and explicit uncertainty. Bounding-box overlap is not proof of complete coverage.
  No automatic deletion or silhouette conversion is planned.
- **Platforms/signing:** execute prepared Intel macOS/Linux jobs before claiming compatibility.
  Windows needs native filesystem/process support. Developer ID/notarization are scoped above;
  existing ad-hoc signatures do not establish them.
- **Historical live group incident:** investigate only on recurrence. Preserve the first divergent
  SVG/PNG, selection/window/document IDs, audit/wire logs and Undo state before restart or closure.

## Rules for taking a task

Use CONTRIBUTING checks appropriate to changed code and update the handoff with fresh evidence.
Regenerate manifests when the MCP surface or instructions change. Repeat native GUI acceptance
only for changed live behavior, on explicitly authorized owned synthetic documents; preserve user
windows. Keep original files, IDs/references, approvals, snapshots, records and no-op behavior.
Do not revive the retired Python MCP/parity workflow. A plan is not authorization to commit,
publish, launch a GUI, change an installed runtime or send messages.
