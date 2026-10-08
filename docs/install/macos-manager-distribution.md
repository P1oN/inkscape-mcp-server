# macOS Manager distribution contract

The 2026-10-08 working tree has a local Developer ID signed/notarized native installer
candidate (`1a52d477e623f8d5`, Team `DN263AX69U`). Apple Accepted all three containers,
and isolated package/installer/update checks pass. Earlier build `7ad414e1314fd62b`
has separately bound native first-install checks. These candidates remain
unpublished and unqualified on a browser-quarantined clean host. Use the existing
[source installation](../../README.md) until the signed release gates in
[Priority 1](../RUST_NEXT_PLAN.md) pass.

PR #30 review repairs add per-client configuration diagnostics and preserve unmanaged
or foreign-owned skills without adopting them. Runtime staging verifies the signer and
stores a private, guarded receipt outside the bundle, bound to `FILES.json`, build/revision
and the exact helper bytes. Startup rechecks the inventory and receipt without running
`codesign`. Helper upgrades authorize retained rollback runtimes and keep receipts for
older helpers. A missing or changed receipt refuses launch and requires installation
preparation again. These source changes require fresh signed-artifact qualification;
the local candidate evidence above remains bound to its earlier bytes.

The default application location is `~/Applications/Inkscape MCP Manager.app`.
The Manager stages a verified copy and asks to install/reopen it. A separate bundled
helper waits for that Manager to exit before replacing the destination. A running
destination app causes refusal; the installer never kills it. An inventory-bound
journal restores interrupted directory moves and preserves later edits. Runtime state
remains in `~/Library/Application Support/inkscape-mcp`.

The DMG contains one Manager application. Its Resources contain matching instructions
and an offline runtime archive. The runtime archive has a fixed
`inkscape-mcp-macos-arm64/` top directory, an external `FILES.json`, and an
`Inkscape MCP Runtime.app`. Rust executables and private D-Bus tools reside in
`Contents/MacOS`; context bridge and private libraries reside in
`Contents/Frameworks`; ordinary runtime data reside in `Contents/Resources`.
Settings, selectors, skill baselines, installation journals and inventories remain
outside sealed code bundles. Native dependency paths are relocated before signing.

Tickets cover the Runtime app and Manager app individually, then the final DMG.
A tarball cannot be stapled. The runtime archive is made from the finalized Runtime
app; its external inventory is computed after stapling. Signing and notarization
records bind candidate inventory, source/final code hashes, separate candidate/runtime build identities and final
container hashes. Sealed package metadata describes the signing intent; the external
evidence records actual notarization acceptance. Assessment of extracted code on a
quarantined clean host remains a separate gate.

Format-1 manifests retain their existing strict schema. Bundle runtime manifests use
`launcher_minimum: 2`, so a format-1 launcher checks compatibility before extraction
and can require a Manager upgrade. Text-only assets retain minimum 1. Historical
legacy layout downloads and launchers remain supported; the new layout adapter selects
bundle paths only when bundle metadata exists. Never redirect old runtime assets to
new bytes or claim notarization for unsigned historical runtime reuse.

The Manager invokes fixed Rust operations: `install-inspect`, `install-prepare`,
`install-activate`, `install-recover`, `manager-prepare`, and `manager-activate`.
Preparation takes bounded JSON through stdin and performs archive verification,
doctor and a real STDIO/workspace probe. Activation uses a preparation identity,
rechecks destinations, journals client/settings/skill/bootstrap changes, and publishes
the selector last. Startup recovers an interrupted installation before serving MCP.
Discovery reads configuration; it never executes an old discovered launcher. Saved multiple workspace roots remain intact; the native picker displays the first
root during inspection. Explicit legacy choices remain selected when settings are read.
Empty activation plans preserve rollback history and do not request a client restart.
Inspection prefers an executable saved custom Inkscape path. If that path is missing
or no longer executable, the user's valid picker selection repairs PATH while preserving
other saved settings and workspace roots. A usable saved engine is retained on upgrade.
The completion view shows the sealed Manager distribution/build, actual helper signature
status, runtime/instruction/bootstrap identities and compatibility integer. Notarization
is reported as not checked on the current host rather than inferred from build metadata.


The local ad-hoc checkpoint hardens Rust and Manager code. Hardened ad-hoc private
GLib executables cannot load libraries without a common Team ID; the checkpoint leaves
those two executables unhardened and adds no library-validation bypass entitlement.
The Developer ID mode hardens every inventoried code item and verifies the configured
Team ID, Application authority and a secure `Timestamp=` from codesign display output.
Final native acceptance repeats authority/timestamp checks for the packaging tool, every
inventoried runtime code item, both app main executables, the Manager helper and DMG;
Mach-O items must also show hardened runtime. Ordinary signing time is insufficient.
Real Developer ID execution now passes for the local signed candidate, including private
GLib hardening, ticket preservation through copies/extraction/installation and local app
Gatekeeper assessments. Clean-host online/offline quarantine acceptance remains open.
Accepted Apple logs are retained and matched to each submission ID/input digest; future
preparation embeds them in distribution evidence and publication refuses mismatches.

Developer tooling (new output directories are required):

```sh
scripts/dev-tools.sh build-distribution --package VERIFIED_PACKAGE --output NEW_DIRECTORY \
  --channel prerelease --tag LOCAL_TAG
scripts/dev-tools.sh installer-acceptance --package VERIFIED_PACKAGE --output NEW_EVIDENCE_DIRECTORY
```

For Developer ID preparation, `build-distribution` additionally requires all of
`--identity`, `--team-id`, and `--notary-profile`. Never place credential values in
repository files or logs. The protected CI flow is implemented in `.github/workflows/signed-release.yml`:
validate successful main CI inputs, prepare with an ephemeral keychain, verify native
probes without credentials, then upload/check remote digests/publish. It has not run.
Configure protected `release-signing` and `release-publication` environments before use.
Set the public `SIGNING_TEAM_ID` **repository Actions variable** to the verified Team
(`DN263AX69U` for this setup). Validation rejects an absent/malformed value before
candidate retrieval and passes that same checked value to preparation, verification
and publication. A signing-environment-only Team variable is insufficient.
In `release-signing`, set secrets `SIGNING_CERTIFICATE_BASE64`,
`SIGNING_CERTIFICATE_PASSWORD`, `NOTARY_KEY_BASE64`, and variables `SIGNING_IDENTITY`,
`NOTARY_KEY_ID`, `NOTARY_ISSUER_ID`. The current workflow uses a Team App Store Connect
API key; its issuer ID is required. Keep private credential values out of repository
files, local acceptance reports and logs. For `runtime_tag` releases, validation follows the original immutable runtime release,
checks its signed/notarized acceptance evidence, and binds its archive digest. Preparation
copies the original archive and Runtime app without resigning or rebuilding them, while
signing the current Manager/compatible bootstrap and including current instructions.
The published manifest retains the original runtime reference. For earlier local build
`7ad414e1314fd62b`, real Developer ID reuse passes: Runtime bytes, inventory, offline
payload and submission remain exact;
the new Manager/DMG have Accepted submissions. Isolated native helper upgrade, MCP and
rollback preserve Runtime files, custom skills, settings and artwork. This is local
development evidence, not protected CI or a published immutable release.
Publication stops on an existing draft. After separately authorizing publication,
an operator may recover the exact retained candidate with
`scripts/publish-signed-distribution.sh VERIFIED_ASSET_DIRECTORY --recover-draft RELEASE_ID`.
Recovery requires the draft tag, source SHA, channel and every existing asset digest
to match, uploads only missing assets without replacement, and verifies the complete
remote set before publishing. Wrong, foreign, duplicate or published assets/releases
are refused without mutation. No automatic draft deletion or tag replacement is offered.
Local signed distinct-runtime update/rollback and immutable-runtime reuse pass.
Protected CI delivery, signing-job retry qualification and clean-host final acceptance
remain pending. macOS release tar steps use `COPYFILE_DISABLE=1` so AppleDouble entries
cannot violate the fixed archive root; extracted Manager signatures/tickets still validate.

For an interrupted notarization poll, retain the exact prepared containers, submitted
ZIPs and `notary-*.json` records. The protected job preserves these and its validated
candidate identity in the failure artifact. Maintainers can resume a fixed container:

```sh
scripts/dev-tools.sh resume-notarization --output ABSOLUTE_RETAINED_DIRECTORY \
  --label runtime --identity 'Developer ID Application: NAME (TEAM)' \
  --team-id TEAM --notary-profile KEYCHAIN_PROFILE
```

Labels are only `runtime`, `manager` or `dmg`; paths are fixed within the directory.
Recovery verifies ownership, signatures/Team, retained ZIP digest and any recorded
code identity, then polls the original submission. Before stapling it binds the live
Accepted log to that job, submitted digest and target CodeDirectory hash. This also
handles a DMG already changed by legitimate stapling. It never signs, rebuilds or
submits code, and a recovered container does not bypass the final inventory/package/
CI gates. Real local recovery of all three already Accepted/stapled containers passes
with unchanged Apple history and payload bytes; changed archives and differently
signed payloads from the same Team are refused before credential access. Whole-job
recovery after an actual protected-CI interruption remains unqualified.

A read-only GitHub API check on 2026-10-08 finds no repository release environments.
Both environments and protected credentials still need setup before CI delivery;
local Keychain signing does not populate GitHub secrets.

Application tickets are copied as ordinary `Contents/CodeResources` files (see
[Apple’s ticket lookup implementation](https://github.com/apple-oss-distributions/Security/blob/main/OSX/libsecurity_codesigning/lib/notarization.cpp)).
The guarded tree copier also preserves the bounded `com.apple.quarantine` attribute on
app roots/files; relocation does not clear the browser gate. Native notarization checks
validate tickets after extraction and app copy. These automated checks remain separate
from observing real quarantined first-open prompts on the other Mac.

Preparation can be cancelled at its completed review point. Active verification,
activation and recovery protect the window from closing. Errors end loading; stage
messages and bounded download byte counts are reported without invented percentages.
