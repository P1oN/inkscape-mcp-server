# Native installer development checkpoint — 2026-10-08

This is uncommitted local evidence, not a signed release or completed Priority 1.
Apple Developer credentials are unavailable. The other Mac is available for final
acceptance. The user authorized isolated Manager GUI acceptance only; no Inkscape
launch, user installation switch, real client restart or publication occurred.

The working tree adds the typed installer transaction, per-user Manager relocation,
Runtime app layout adapter, native first-install UI and candidate-to-distribution
packager. See the [contract](../../install/macos-manager-distribution.md).

Earlier build `70cdee8c233c64d1`, revision
`8762cfe238a0074a58c63c062b8f703c481a2ee7` (dirty working tree):

| Evidence | Observed scope |
| --- | --- |
| `migration/results/priority1-installer2/report.json` | Legacy layout, isolated both-client configs, actual doctor/STDIO, 18 file boundaries recovered twice, retry and removed/customized skill preservation |
| `migration/results/priority1-bundle-installer2/report.json` | Same transaction matrix on relocated Runtime app layout |
| `migration/results/priority1-bundle-package2/` | Headless per-call package acceptance, 452 files, native INX/private bus/CLI/STDIO |
| `.inkscape-mcp-local/priority1-distribution2/distribution-evidence.json` | 14 native code items relocated and individually signature-verified; ad-hoc only |
| `migration/results/priority1-bundle-package/packaged-bus.log` | Fully hardened ad-hoc GLib failed dyld Team-ID library validation; failure retained |

These artifacts predate later Manager relocation/signing/identity edits. Do not transfer
acceptance to a different build. Ignored local artifacts may be absent on another checkout.

Unit tests exercise first-install recovery and Manager directory-move recovery,
changed destinations/backups, forged journals and link/path refusals. A full
parallel runtime test run reproduced the previously recorded native lock test failure
at `live_launch::tests::private_session_and_locks_refuse_link_escape_and_parallel_launch`.
No concurrency repair is claimed. Final sequential checks and native GUI observations
are recorded below when complete.

At that earlier checkpoint, outstanding gates included the expanded refusal/native wizard matrix and final
bootstrap upgrade qualification, credential-backed protected four-stage CI signing
and publication, distinct finalized signed-runtime update qualification,
real Developer ID/notary/Gatekeeper execution, real Claude, another-Mac acceptance and
broader release review. The active backlog remains authoritative.

New local runtime build `1c77318bb953d393` was assembled into the third ad-hoc
distribution. Actual Cocoa window acceptance is in
`migration/results/priority1-native-manager/report.json`: canonical isolated profile,
fresh state, missing-workspace error, native workspace picker, real archive preparation
and doctor/STDIO, review/activation, completion identities and preserved synthetic
Codex preferences/original SVG. Native acceptance found an integer-vs-boolean checkbox
encoding error; the source was repaired and the fixture Manager recompiled. Its Info
build label predates that fixture recompile, so this is source/executable-hash scoped
evidence, not final DMG acceptance. A Documents fixture stalled on directory open; the
owned helper was terminated and interruption cleared loading. A UI-tool read after
closing caused a default-profile relaunch: cancelled before activation, generated
Manager staging copy/plan removed, no runtime/client binding changed. Test windows closed.

The sequential full runtime test run passed after the recorded parallel lock failure.
The added signed-release workflow separates candidate validation, protected preparation,
credential-free native verification, and minimal publication. Synthetic guards reject
candidate/signer/code/ticket/final-byte/acceptance failures and verify draft/upload
failure boundaries. Existing publication fixtures and actionlint passed locally. No
workflow was dispatched; protected environments/secrets are not configured. Signing,
notarization and finalized signed probes remain unexecuted. Signed referenced-runtime
reuse now has implementation and synthetic guard coverage; its native credential-backed
branch and the expanded native-state/refusal matrices remain unqualified.

Build `fa2ee7b66f2f5f9d` (sixth local distribution) passed headless package acceptance
(452 files), isolated installer acceptance with **19** publication boundaries recovered
twice, unknown-request/symlink-workspace refusals, read-only byte-bound bootstrap
identity, and distinct-runtime component activation/rollback with actual permanent
launcher STDIO probes. Evidence is in `migration/results/priority1-bundle-package6/`,
`priority1-bundle-installer6/` and `priority1-bundle-updates6/`. The earlier fifth
distribution passed all 16 real STDIO discovery profiles. No MCP schema/instruction
surface changed.

The sixth Manager (`c7b156cc75e9842c`) was explicitly upgraded through its Cocoa
“Install and reopen” control into the isolated per-user Applications directory.
The reopened window retained runtime/text `1c77318bb953d393`, workspace, Inkscape and
channel. It correctly showed an unknown launcher identity for a pre-metadata install.
Its installation upgrade then refused “occupied instruction identity” before activation:
identical instruction bytes had a different version label. Source now stores new bundles
under a full manifest identity while retaining compatible legacy lookup and immutable
rollback bundles. A dedicated regression test and fresh package evidence are required.
Test windows were closed without reading closed app bindings again.

A second UI-tool default-profile relaunch had also prepared a fixture Manager staging
copy. It was cancelled before activation and its exact fixture plan/copy removed; no
runtime selector or client binding changed. These tool-induced relaunches are separate
from the explicitly observed isolated Manager upgrade above.

Streamed bounded progress, safe-point cancellation, saved bootstrap byte identity,
quarantine-preserving guarded copies, signed immutable runtime reuse and separate
Manager/runtime identities are implemented. Rust/library tests, both Clippy graphs,
strict Cocoa compilation, publication fixtures, archive traversal/link/normalized-name
refusals and workflow actionlint passed at the sixth checkpoint. Full sequential runtime
tests passed before the later instruction-identity repair. Developer ID, secure timestamp,
notary/ticket/Gatekeeper and protected workflow execution have no native acceptance yet.

The seventh checkpoint, runtime `90e694fb1628dfae`, validates that repair:
`migration/results/priority1-bundle-installer7/report.json` passes 19 interruption
boundaries, equal-content/new-version retry and actual doctor/STDIO. Distinct-runtime
updates and rollback pass in `priority1-bundle-updates7/`. Full sequential all-target
Rust tests, both Clippy graphs and formatting pass after the repair. The parallel lock
failure above is still a separate unresolved observation.

Actual Cocoa evidence in `priority1-native-manager7/report.json` observes explicit
Manager “Install and reopen”, retained old runtime during relocation, successful
preparation, cancellation back to Ready, retry and activation. The completion window
shows runtime/text/bootstrap `90e694fb1628dfae`; both permanent launcher byte hashes
match saved bootstrap metadata and the previous runtime remains `1c77318bb953d393`.
The final test window is closed. This proves the scoped local upgrade, not real client
reconnect or browser-downloaded signed distribution acceptance.

The ninth runtime checkpoint (`9a65235f1facbccd`) expands native installer acceptance
in `migration/results/priority1-installer-matrix11/report.json`: missing-client and
foreign-binding refusals, conflicting skill preservation, multiple legacy candidates,
legacy migration into a custom root, real failed version-probe preservation and
component recovery through the Manager CLI. All 19 installer publication boundaries
still recover twice. The deliberately mismatched development package is a scoped
negative fixture; no Developer ID artifact is modified to manufacture that check.
Headless package acceptance (452 files), distinct-runtime update/rollback, and all 16
STDIO discovery configurations pass for this runtime in the corresponding `package9`,
`updates9` and `discovery9` result directories.

`priority1-native-manager9/report.json` observes the real bundled helper through Cocoa:
Manager relocation, an actual interrupted component selector detected/recovered, real
doctor/STDIO preparation and activation into the isolated profile. The old runtime
`90e694fb1628dfae` remains the preceding pair. The window is closed.
`priority1-native-ui-states/report.json` separately observes native presentation using
synthetic fixed helper replies: empty Stable, explicit prerelease choice, no-update
without resetting the selected channel, conflict, damaged installation, multiple
choices and recovery refresh. These are UI checks, not native transaction qualification.

The previously recorded parallel launch-lock failure is repaired: scope guards now
explicitly unlock rather than relying solely on close. A deterministic owned fork
fixture retains the descriptor while the parent releases/reacquires its lock; it passes.
The full default-parallel all-target runtime suite then passes (336 passed, two opt-in
ignored). Both Clippy graphs and tooling tests pass at this checkpoint. No Inkscape
launch was involved in this lock test.

Explicit partial-draft recovery now validates draft/source/channel and every existing
asset digest, uploads only missing assets without clobbering, and verifies unique names
and the full remote inventory before publication. Mock fixtures cover partial/complete
recovery and corrupted/foreign/duplicate/published/wrong-source refusals. Final gates
now check required executable/bridge/dependency entries and checksum sidecars; the
native receipt writer refuses missing negative/migration/recovery installer gates.
No real GitHub release was created or modified.

Later status-display and no-op edits require their own evidence: the Manager exposes its
sealed distribution label and actual helper Developer ID/ad-hoc/unverified status.
Notarization is explicitly reported as not checked on this host, not inferred from a
build flag. Empty activation plans retain the preceding rollback record and avoid a
client restart recommendation. Credential-backed signing, timestamp/hardening, ticket
assessment, another-Mac/quarantined installation, real Claude and broader release
review remain pending. Signing timeout records are retained; automatic resumption of
a partly prepared signing job has not been qualified.

## Final local checkpoint — build c80fa89e152d3a28

This checkpoint supersedes the earlier local matrix/lock/status gaps above; it does
not supersede credential-backed and external-host gates. Source revision remains
`8762cfe238a0074a58c63c062b8f703c481a2ee7` with uncommitted working-tree changes.
The ready legacy package is `.inkscape-mcp-local/build.U9c5YD/package`; relocated
Runtime app, Manager and development DMG are in `priority1-distribution12`.

`migration/results/priority1-installer-matrix13/report.json` passes all 19 publication
boundaries and the expanded refusal/migration/recovery matrix. New coverage proves
inspection displays the first saved workspace without mutating multiple-root settings,
and no-op activation preserves rollback records without requesting a reconnect.
The real legacy Manager acceptance in `priority1-native-legacy12/report.json` observes
explicit selection among two candidates, selection retained after inspection, saved
workspace reuse, verified per-user Manager replacement/reopen and successful activation.
The original drawing, Codex preferences, unselected Claude binding and old ready runtime
`70cdee8c233c64d1` remain intact. The completion view visibly shows runtime/bootstrap
`c80fa89e152d3a28`, Manager `9f3b38ab7224158d`, Development (ad-hoc) helper signing and
“Notarization: not checked on this Mac”. The test window is closed. Client profiles are
synthetic; these are actual Cocoa/helper transactions, not real client reconnect tests.

The relocated Runtime app matrix also passes in
`migration/results/priority1-installer-layout12/report.json`, with the same refusal,
preservation and interruption gates. Headless package acceptance passes in
`priority1-bundle-package12/acceptance.json` (per_call, 452 files).

Final default-parallel all-target runtime tests pass (336 passed, two opt-in ignored).
Both Clippy graphs, both formatting checks and 18 tooling tests pass. Release guard
fixtures/Bash/actionlint passed earlier at their unchanged script checkpoint.
No exposed MCP instructions/tools changed; no llms regeneration or installed server
restart was needed. No Inkscape GUI, real installed runtime/client changes, commit,
GitHub publication or outgoing message was performed.

Priority 1 is not complete. Credential-backed Developer ID signing/hardening,
notarization acceptance/tickets, protected-environment execution, another-Mac browser
quarantine/online/offline tests, finalized signed update/rollback, real Claude and broader
release review remain open. Signing timeout evidence is retained, but automatic
resumption of a partly prepared signing job is not qualified. Real publication requires
the user's separate authorization. Local ad-hoc and mocked guards establish neither
Gatekeeper acceptance nor a stable-release qualification.

## Continued local audit — secure timestamp verification

The signing command requested secure timestamps, but its verification previously checked
only authority/Team and native hardening. The tooling now explicitly refuses absent or
ordinary signing timestamps. Final native acceptance repeats the checks for the signed
packaging tool, runtime code inventory, Runtime/Manager main executables, Manager helper
and DMG, alongside cryptographic signature/Team verification. Native code must also show
hardened runtime; the DMG is assessed as a container. Rust and Python fixtures refuse
missing timestamps, `Timestamp=none`, wrong authority and absent native hardening.
This is a local verification repair, not evidence of credential-backed signing.
The previously accepted runtime build is unchanged; this edit affects development
packaging tooling and signed-release acceptance scripts only.

Validation for this repair: all 19 tooling tests, tooling Clippy with warnings denied,
Rust/tooling formatting, signed-distribution guard fixtures, Bash syntax and
`git diff --check` pass. The display verifier also rejects the actual ad-hoc bundled
helper from distribution12. No signing credential or Apple notarization call was used.

## Enrollment status correction — 2026-10-08

Apple Developer Program and App Store Connect welcome confirmations establish that
membership/access are active. Earlier unavailable-enrollment statements retain their
historical context. A fresh local `security find-identity -v -p codesigning` check finds
zero usable Developer ID Application identities. Enrollment therefore no longer blocks
certificate setup; signing certificate/private-key and notarization credential setup
still precede real signed-package qualification. No credential was created, exported,
installed or placed in repository files by this check.

## Local credentials verified — 2026-10-08

`security find-identity -v -p codesigning` now reports a valid Developer ID Application
identity for Team `DN263AX69U`. The previously suggested `P488V8X523` came from an
Apple Development identity and was incorrect for this distribution signer.
`xcrun notarytool history --keychain-profile inkscape-release --output-format json`
succeeds with empty submission history, proving that the named local profile can
authenticate. No artifact has been submitted or signed by these checks. No password,
private key or credential payload was read or stored in the repository. Actual signed
artifact execution/notarization acceptance and protected CI setup remain open.

## First real signing attempt — requirement syntax repair

The local `priority1-signed1` preparation used the verified c80fa89e152d3a28 candidate
and Team DN263AX69U. After the user authorized codesign in macOS, context.so acquired
a Developer ID Application signature, secure timestamp and hardened runtime. The
next verification failed because `codesign -R` takes a bare argument as a filename;
literal requirement source must begin with `=` (confirmed against the installed
codesign manual). This attempt stopped before any notarization submission.
The partial output and local provenance are retained; it is not a distributable release.

The requirement prefix is repaired in the builder, client signature check, runtime
bundle verification and final native acceptance script. The actual signed library
passes the corrected DN263AX69U requirement and refuses a different Team ID. A new
runtime candidate is required because the installer runtime contains the same check;
older packaged binaries are not relabeled as repaired.

## Signed local candidate — build 7ad414e1314fd62b

The repaired ready package is `.inkscape-mcp-local/build.JhpFWb/package`. Its exact
bytes were transformed into `priority1-signed2` with Developer ID Application Team
DN263AX69U, secure timestamps and hardened runtime for all 16 runtime code items,
including private GLib tools. No library-validation bypass entitlement was added.
The Runtime app, Manager app and DMG each have Accepted Apple submissions:

| Container | Submission ID |
| --- | --- |
| Runtime app | 31c8eeca-65c4-45d2-a4c1-ec66f2e3b688 |
| Manager app | 660b5f03-5b5e-44c0-91f8-ee5d656de8fa |
| DMG | e4f79b2c-eee5-48ce-accd-8b2e72b01260 |

All tickets are stapled and validate; relocated/copied app signatures and tickets
remain valid. Apple logs are retained in the output directory, with matching Accepted
status, job ID and submitted-container SHA-256; all three report no issues. Final DMG
SHA-256 is `b1f2b51b0e12f71a265a2a10947fe05ee179c9d92ada5f11b3c32aaa1ad80f47`.
`LOCAL-PREPARATION.json` records the candidate inventory, packaging tool hash, dirty
source revision and local-only scope. This is not a successful main-CI candidate and
is not publication eligible.

`migration/results/priority1-signed-validation2/report.json` independently verifies
21 signed paths (runtime inventory, apps, helper, DMG and acceptance tool), correct
Team, secure timestamps/native hardening, all three tickets and local Gatekeeper app
assessments. Signed per_call package acceptance passes (452 files), and the signed
installer matrix passes all 19 publication boundaries with preservation/refusal/
legacy/custom-root/component recovery and no-op gates. The distinct previous runtime
70cdee8c233c64d1 activates/rolls back with actual permanent-launcher STDIO checks;
`priority1-signed-updates2/report.json` reports runtime_exercised=true.

`priority1-native-signed2/report.json` observes real signed Cocoa/helper relocation,
native workspace selection, bundled first-install preparation/activation and genuine
no-op completion (“No client restart is needed”). Runtime/bootstrap 7ad414e1314fd62b
and Developer ID verified DN263AX69U are visible. Original SVG and preferences remain
intact; no-op selector and installation-last hashes remain unchanged. Installed
Manager and Runtime app tickets/signatures validate after copying. The test window
is closed. Profiles are synthetic; network was not disabled and quarantine was not
a browser download. No real client reconnect or clean-host/offline claim follows.

Default-parallel runtime tests (336 passed, two opt-in ignored), both Clippy graphs,
19 tooling tests, formats, Bash syntax, signed-release fixtures and diff checks pass.
The continued accepted-log gate adds status/job/input mismatch refusals; preparation
now embeds accepted logs and their hashes in future distribution evidence. This
tooling-only edit follows the actual signed2 preparation; its logs were independently
fetched/verified and retained in the scoped local validation report.

Protected CI credential setup/execution, signed immutable-runtime reuse and signing
job retry qualification, browser-quarantined another-Mac online/offline/client tests,
real Claude and broader release review remain open. No user installation, Inkscape
GUI, commit, PR, GitHub publication or outgoing message was performed.

## Real signed immutable Runtime reuse — local instructions-version refresh

`priority1-signed-reuse2` uses the exact previously Accepted Runtime archive
SHA-256 `7ccae2fdda391de00b00ff53464b56ee6686ebd0324eb0e7e3d538e3195e340c`,
all 16 original code identities and original Runtime submission
`31c8eeca-65c4-45d2-a4c1-ec66f2e3b688`. No Runtime submission or resigning occurred.
The offline payload is identical. A legitimate locally generated format-1 manifest
retains the original `local-signed-checkpoint` Runtime reference under the new local
`local-signed-text-refresh` distribution. The instruction version changes to
`local-text-refresh` while content remains identical, exercising version-specific staging.
The local reference evidence incorporates the independently fetched original Accepted
Runtime log after verifying its job ID and input digest; no CI receipt was fabricated.

The refreshed Manager submission `30d3b37d-b482-4a7b-8f03-df5b45f8f0ff` and DMG
submission `1264db1c-37df-441c-ae48-c1ce3aa67c5c` are Accepted. The current builder
retrieved and verified both Accepted logs before stapling. All three container tickets
validate. Final refreshed DMG SHA-256 is
`f7d14a7f56a3d21ff19baea09a77acb4f775dd7b7d3e8ec723c00b672796560e`.
All 20 runtime/Manager/helper/container paths pass cryptographic Team DN263AX69U,
Developer ID authority, secure timestamp and applicable native hardening checks.

The actual signed helper upgrades the previously isolated signed-native profile.
A synthetic custom skill note, Codex preferences/binding, private settings and SVG
remain byte-identical. The installed Runtime inventory and Runtime selector are
unchanged. Actual permanent-launcher initialization/tools/workspace requests succeed
(100 tools visible in this default environment; the full registry remains 112).
Rollback restores the preceding instruction pair and preserves those files again.
The scoped report is `migration/results/priority1-signed-reuse2/report.json`.
This iteration uses the native helper/STDIO, not another Cocoa window or real client.

The first fixture archive preparation was refused because macOS tar wrote AppleDouble
metadata outside the fixed package root. Release launcher/baseline/job-transfer tar
steps now set `COPYFILE_DISABLE=1`; the corrected launcher archive extracts with valid
Manager signature and stapled ticket. Signed-release refusal fixtures, existing
publication fixtures, Bash syntax and actionlint pass after this repair. Failed local
preparation output is retained; it had no notarization submission.

The refreshed assets and `LOCAL-PREPARATION.json` are explicitly local development,
publication_eligible=false. Protected CI credential setup/execution, signing-job retry
qualification, browser-quarantined another-Mac online/offline/client acceptance, real
Claude and broader release review remain open. No user installation, Inkscape GUI,
commit, PR, release publication, credential export or outgoing message was performed.

## Fixed-container notarization recovery and remaining delivery audit

The development command `resume-notarization` accepts only fixed Runtime app, Manager
app or DMG labels under an existing owned directory. It never signs, rebuilds or
submits code. It bounds retained JSON, verifies ownership/Team/deep app signatures,
secure timestamps and hardening, checks retained ZIP bytes and recorded code identity,
polls the original ID and matches Apple's live Accepted log to the job/input digest
and exact target CodeDirectory hash before stapling. App tickets specify arm64; Apple's
DMG ticket has no architecture field. Records, retained logs and recovery receipts use
the existing guarded atomic write/fsync pipeline. Concurrent recovery uses its lock.

`migration/results/priority1-notary-resume1/report.json` proves real service recovery
of all three previously Accepted/stapled containers in owned copies. Apple's history
remains the same five IDs; original submitted ZIPs, DMG and app executables remain
byte-identical. Original signed candidates were not modified. Separate corrupted-ZIP
and different-signed-payload/same-Team fixtures are refused before credential lookup.
The first DMG check refused its missing architecture field; correcting the container
rule then passed. This is real Accepted-container recovery, not an injected server
poll timeout or complete protected-CI job recovery. The workflow now retains complete
prepared containers plus validated candidate identity in its failure artifact.

Tooling tests pass (16 unit and four source tests), as do tooling Clippy, both format
checks, signed-release refusal fixtures, actionlint, Bash syntax and diff checks.
No runtime/MCP behavior or authoring instruction changed in this iteration.

The current requirement audit preserves the complete Priority 1 scope:

| Requirement | Current authoritative evidence | Remaining qualification |
| --- | --- | --- |
| 1. Layout and compatibility | Runtime app v2 adapter, format-1/minimum-launcher contract; signed2 signature/inventory/package report and copied tickets; signed native relocation | Real browser-quarantined and offline clean-host assessment |
| 2. Installation and preservation | Typed inspector/preparer/activator/recovery; signed 19-boundary installer report, legacy/custom-root/foreign/missing-client/skill/no-op gates; native first installation | Real client and other-host delivery matrix; historical ad-hoc failed-probe fixture remains separately scoped |
| 3. Manager UX | Explicit native states and native acceptance reports; latest signed first-install/no-op; actual signed helper refresh/STDIO/rollback | Published GUI component downloads and other-host/bootstrap/client observations tied to final release bytes |
| 4. Signing and notarization | Team DN263AX69U; real Accepted Runtime/Manager/DMG logs, complete hardening/signature/ticket checks and final-byte hashes | Protected CI credentials; clean-host helper prompts/offline behavior |
| 5. Release flow | Four evidence-bound jobs, immutable-reference and publication/refusal/recovery fixtures; real signed Runtime reuse; fixed notarization recovery | Successful committed main-CI candidate, protected jobs, complete CI-interruption recovery and separately authorized real publication |
| 6. Final delivery and existing qualification | Local package/helper/STDIO/native evidence linked to actual artifacts | Other-Mac/browser online/offline matrix, real Codex/Claude reconnect, clean source bootstrap and broader completed release review |

A read-only GitHub environments API request returns an empty list for this repository;
release-signing secrets/variables endpoints return 404. No environments or credentials
were created or exported. `command -v claude` finds no client on this host. The API-key
availability question is pending; local Keychain credentials do not imply protected CI
credentials. AGENTS.md still prohibits committing/publishing without the user's request.
Priority 1 remains active and incomplete; no CI or clean-host result is inferred from
local fixtures, and the final release review is not declared complete by this audit.

## Installer audit repair — custom and moved Inkscape paths

The audit found that permanent inspection ignored saved custom engine paths, while
preparation reused an unusable saved PATH even after a valid replacement was chosen.
Inspection now prefers an executable saved path; preparation prepends the selected
engine directory only when the saved engine cannot be used. Other path entries and
workspace/live/monitoring/client/artwork state are preserved. Non-executable prerequisite
files are refused; unrepresentable PATH directories fail before settings mutation.

Build `1a52d477e623f8d5` is `.inkscape-mcp-local/build.9gWD4d/package`. Default-parallel
all-target runtime tests pass (337, two opt-in ignored), tooling tests pass (20), and
both Clippy graphs, formats, signed-release fixtures, actionlint and diff checks pass.
`priority1-engine-installer17/report.json` passes 19 interruption boundaries plus new
custom-engine/no-op preservation and missing-engine picker repair gates. The native
receipt recorder now requires those two gates and refusal fixtures exercise their absence.

The first custom engine fixture lacked vendor GTK layout and doctor refused it. A full
owned copy of Inkscape retained quarantine; its first version query timed out, so it is
not claimed as relocated-app acceptance and no quarantine was cleared. The deterministic
path fixture uses a fixed compiled adapter to the actual approved headless engine and
an actual vendor GTK binary for the existing doctor metadata check. It verifies actual
helper preparation/activation/MCP probes and moved-path persistence; another app's first
open remains a separate Gatekeeper/host qualification. Original Inkscape was untouched.

The latest Runtime app is Developer ID signed in `priority1-signed3`. The builder stopped
before receiving a submission ID with a default profile lookup error, although independent
history/recovery requests still authenticated. The exact retained ZIP was submitted once
through the system notarytool, recording ID/input SHA/CodeDirectory atomically. Submission
`1609561d-6737-44f7-9453-40098fd160da` is Accepted; the fixed recovery command fetched and
matched the live log, stapled and validated the Runtime ticket without resigning it.
`LOCAL-RECOVERY.json` links the pre-build local candidate/tool provenance and exact bytes.
This is a recovered Runtime checkpoint; Manager/DMG assembly and finalized-package checks
for build 1a52 remain pending. The prior complete signed DMG/evidence remain bound to 7ad.

Additional read-only GitHub queries find no repository signing secret names or variables,
in addition to the absent release environments. API-key availability is still pending.
No credential payload was read/exported; no commit, PR or release was published. Priority 1
remains active with its full CI, host/client and review requirements intact.


## Recovered Runtime reuse and Manager submission — local checkpoint

The recovered build `1a52d477e623f8d5` now has a local Runtime-only reference in
`priority1-signed3-reference`. Its recorder validates the original candidate inventory
and pre-build tool identity, actual compiled build/revision, all 16 code signatures,
hardening/timestamps and the corresponding Accepted log CodeDirectory hashes. The
copied Runtime bundle and debug payload remain byte-identical; only an outer `FILES.json`
is added after stapling. Native update-manifest generation and Runtime verification pass.
The reference is explicitly local and publication_eligible=false, with no CI receipt.

The existing immutable Runtime reuse path starts a new local assembly in
`priority1-signed3-complete2`, preserving the original Runtime submission and bytes.
The first attempt supplied a relative reference path and was refused by the absolute-path
storage guard before any submission; its empty output is retained. The absolute-path
attempt signs/submits the Manager as `39afddcd-d7a9-444b-911c-daea6e68004b`.
It stops during polling when the default `inkscape-release` profile becomes unavailable.
A direct system notarytool status query also returns “No Keychain password item found”.
The login Keychain is the current default/search path and has no timeout; the cause of
the credential lookup failure remains unconfirmed. No credentials were read/exported,
restored or changed. The user has been asked to confirm/restore the local profile.
The exact Manager ZIP and submission ID are retained; no duplicate submission is made.

`priority1-signed-package3/acceptance.json` passes for the finalized Runtime: 453
relocated files, real headless rendering, rollback, approval refusal, no-op and original
preservation. `priority1-signed-installer3/report.json` also passes all 19 publication boundaries,
including custom-engine/no-op preservation and missing-engine picker repair, foreign
binding/ownership refusals, skill preservation, legacy/custom-root and component recovery.
Client profiles remain synthetic; doctor/STDIO/workspace probes use the real signed
native package. The signed-payload failed-probe mutation fixture is intentionally skipped
(`failed_probe_exercised=false`); the separately recorded ad-hoc source fixture covers it.
The Manager is signed but its final notarization/ticket is still pending, so this result
qualifies the current helper/backend rather than final browser/installer delivery.
A fixed local recovery script is prepared to resume the recorded Manager, verify the
unchanged Runtime reference/offline payload, create/sign the DMG, record a single DMG
submission atomically and use fixed native recovery to finish it. An uncertain submit
outcome is explicitly retained rather than retried automatically. The script is only
syntax checked at this checkpoint; neither a finalized DMG nor a completed recovery
is claimed. Entire Priority 1, protected CI and the other-host/client matrix remain open.


The latest `priority1-signed-updates3/report.json` passes with
runtime_exercised=true: real permanent-launcher STDIO/workspace, offline source removal,
private preferences/bindings/custom skills and instructions/runtime/combined updates
with rollback. The distinct baseline remains `70cdee8c233c64d1`, the latest signed
helper is `1a52d477e623f8d5`, and client profiles are synthetic. No native Cocoa, real
Claude or clean-host result is inferred. `priority1-signed-recovery3/report.json` binds
package/installer/update report hashes and exact retained Runtime/Manager identities,
explicitly leaving Manager Accepted/ticket and DMG completion false. All local tool
handles are now terminal. The local notarization profile restoration question remains
pending; protected credentials and the full Priority 1 delivery scope remain open.


## Release configuration scope review

A bounded review of the four-job release path found that the setup guide placed the
signing Team variable only in the protected environment, while validation/verification/
publication jobs read repository variables. That configuration would fail outside the
signing job, potentially after expensive preparation. The workflow now rejects an
absent/malformed repository `SIGNING_TEAM_ID` at initial validation, emits its checked
value and threads it through preparation, verification and publication. The guide lists
repository vs signing-environment scopes and the current Team API key/issuer contract.
No GitHub configuration or credentials are changed by this source repair. Broader
release review and protected execution remain unqualified.

Validation for the Team scope repair: the actual initial workflow shell prefix passes
six absent/malformed/valid Team cases, with no output on invalid input. Actionlint,
signed distribution refusal/publication/recovery fixtures, existing publication
fixtures and diff checks pass. `priority1-team-scope-review/report.json` records the
reviewed workflow digest and bounded scope. No shipped runtime/helper code changes,
credential writes, workflow dispatch, release or installed-runtime changes occurred.
The local profile lookup remains unavailable; its restoration question is still pending.


## Latest Manager and DMG recovery completed

The user restores the local notarization profile. Direct status verifies the existing
Manager submission is Accepted. The fixed local recovery procedure validates/staples
that same Manager, preserves the original Runtime archive/bundle/submission, creates
and signs one DMG and records its single submission atomically before polling. No
Runtime or Manager resigning/resubmission occurs. All three receipts match live
Accepted logs and signed CodeDirectory/input identities; tickets validate.

Latest complete local distribution: `priority1-signed3-complete2`, Runtime/helper
`1a52d477e623f8d5`, Team `DN263AX69U`, dirty revision
`8762cfe238a0074a58c63c062b8f703c481a2ee7`.
Runtime ID `1609561d-6737-44f7-9453-40098fd160da`;
Manager ID `39afddcd-d7a9-444b-911c-daea6e68004b`;
DMG ID `bd3fb7e8-fbd5-48b6-890f-4b1d5d64cc82`.
Final stapled DMG SHA-256:
`f84aed5a88c6decaa55209903554168845d393ca1c36d2358e52b6e258e58aa8`.

`priority1-signed-final3/report.json` verifies 25 signed paths (including extracted
Runtime, extracted launcher Manager and read-only mounted DMG Manager), expected Team,
Developer ID authority, secure timestamps and native hardening; all tickets and live-log
hashes; current-host app/DMG Gatekeeper; exact offline payload; final manifest asset
hashes/lengths. The mounted Manager is inspected without launching and the mount detached.
`priority1-signed-installer3-final/report.json` repeats the 19-boundary installer matrix
against the finalized stapled Manager, including both engine recovery gates. It passes;
clients remain synthetic and signed failed-probe mutation is skipped. Earlier package
and distinct-runtime update/rollback reports remain valid for the exact unchanged
Runtime/archive identities. Final sidecars are generated after stapling.

These are local current-host results, with publication_eligible=false. No main-CI receipt
is fabricated. The earlier recovery3 report retains its interrupted checkpoint context.
Protected CI credentials/environments/execution, actual main candidate, complete CI
interruption recovery, browser quarantine/other-Mac offline prompts/real clients,
clean source bootstrap and broader review remain open. No user installation/GUI,
commit, PR, release, credential export or external message changes occurred.
