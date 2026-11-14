# Reviewed Manager updates

Current sources implement Issue [#63](https://github.com/P1oN/inkscape-mcp-server/issues/63).
The feature becomes available in a separately authorized signed distribution; historical
release assets remain unchanged. An older Manager that lacks this feature still needs a
manual distribution upgrade once. No release is published by implementing this change.

Manager checks its own updates independently of runtime/text. Both checks run asynchronously
with separate 20-second deadlines, coalesce requests, and reject replies after a change of
channel, installation, options, review or activation. Checking only reads metadata and local
identities. A runtime that requires a newer launcher cannot hide the Manager update needed
to upgrade that launcher. Fresh, prepared-only and damaged MCP roots do not need settings.json.
The channel remains the user's stable/prerelease choice; opening Manager never installs an
online update. MCP startup/reconnect stays offline and never opens Manager or Inkscape.

Choose **Review Manager update…** to download and verify the candidate. Review lists the
installed and target Manager/bootstrap identities, exact destination, changed bootstrap files
and hashes, publisher, and restart requirement. Only **Update Manager and restart** authorizes
publication. Cancelling keeps the verified stage and leaves installed files unchanged. The
separate runtime/text update actions retain their own review and confirmation.

The destination is the running installed Manager in `~/Applications` or `/Applications`.
A system installation requires existing write permission; the updater reports a permissions
failure and never escalates, changes system permissions or creates a competing installation.
The app is staged beside its destination, including when the selected MCP root is on another
volume. Application-wide and installation locks serialize different Manager instances and
installation roots. Another process using the destination refuses activation before the
confirming window exits; it is checked again after that instance exits. Nothing kills another
Manager, client, Inkscape, or an existing MCP session.

## Separate immutable release contract

`inkscape-mcp-manager-update-v1.json` is a separate format-1 asset. Do not add Manager fields
to `inkscape-mcp-update.json`: old launchers use `deny_unknown_fields`. The legacy launcher
archive remains `inkscape-mcp-launcher.tar.gz` with its existing top-level
`inkscape-mcp-launcher/{bin/inkscape-mcp-launcher,Inkscape MCP Manager.app}` layout.

The Manager contract binds the immutable release tag/channel, macOS/aarch64 target and minimum
OS, bundle ID `org.inkscape-mcp.manager`, trusted Developer ID Team `DN263AX69U`, native Manager
build/source revision, separately measured bootstrap build/revision/length/SHA-256/compatibility,
supported text/helper interfaces, archive identity and complete app file inventory. Native Manager build identity comes from
its sealed Info.plist; it is not the Rust runtime or bootstrap build ID. The bootstrap's
`bootstrap-info` command reads its own compiled identity without requiring settings or
starting MCP. Release tooling verifies the publisher before invoking it with a cleared
environment. A publisher change requires a reviewed code/contract change.

Discovery uses the fixed project repository, published immutable releases, explicit preview
selection and numeric version eligibility. Same/older distributions and identical Manager/helper
identities are not offered. Unversioned development Managers retain manual installation.
Downloads use bounded HTTPS, approved GitHub redirects, complete archive hashes, traversal/link/
entry/decompression guards, exact inventories and bundle/helper/Developer ID verification.
Ad-hoc development apps cannot opt into production trust. The complete prepared plan is
hashed for Review; changed metadata, app bytes, bootstrap, profile or installation selection
requires another review. The verified target bootstrap must support both retained current
and rollback component contracts before readiness/publication; incompatible candidates refuse
without selecting other runtime/text. Activation uses the retained **current** helper, never an unchecked
new helper. Signature verification precedes any downloaded bootstrap identity probe.

The signed preparation path builds the separate manifest after final signing/stapling and
archive creation. `verify-manager-update` re-extracts and verifies those final bytes before
recording signed acceptance. Its report is a required gate. Runtime-only historical references
remain distinct from the current Manager/bootstrap and the legacy manifest remains unchanged.

## Publication, restart and recovery

Manager-only updates leave an identical permanent bootstrap unchanged. When a permanent
bootstrap exists and its bytes/identity change, Review lists both executable copies and
bootstrap.json explicitly. No bootstrap is manufactured for a fresh/runtime-only root.
Neither mode changes the selected runtime/text pair, workspaces, complete skills, client
configuration/registrations/secrets, settings or monitoring preferences. Saved app preferences
remain in the same app domain. Only HOME, CODEX_HOME and CFFIXED_USER_HOME profile paths,
selected installation and selected channel are carried through Launch Services; monitoring
credentials are not placed in arguments, progress or reports.

The retained activation helper first acquires both locks, validates Review and checks competing
processes. Its readiness handshake allows only the confirming Manager to exit. Under the
installation lock it journals the complete app/bootstrap plan, backs up the old app, publishes
the new app, then publishes the two bootstrap executables and identity using the installer's
existing durable file/backup machinery. This is a compensating transaction across locations,
**not** an atomic multi-location rename. Existing sessions keep their original runtime/helper
paths. A healthy retained runtime receives signer-bound verification and a real MCP probe.

Launch Services success alone does not commit activation. The new Manager's bundled helper
verifies the published app and bootstrap, repairs a partially published bootstrap only when
all identities still match old/new journal values, and acknowledges actual startup. Only then
is `manager-last.json` written and `manager-journal.json` retired. A failed restart restores the
old app/bootstrap when the target is idle; a running unacknowledged Manager is preserved with
concrete instructions. `manager-outcome.json` carries failures back into the next inspection.
Backups, stages and helper paths remain available; no live executable is garbage-collected.

With no Manager using the target, **Recover / retry** (`install-recover`) or offline launcher
recovery restores the previous app/bootstrap. It preflights all destinations and backups
before mutation. Later bytes, foreign files or dangling symlinks refuse recovery and retain
the journal. Interrupted recovery is repeatable; another recovery after success is a no-op.
Do not delete the journal, retained app or `backups/<id>/activation-helper` to force recovery.
If a process still uses the app, close only your own Manager and retry. No recovery launches
an application from MCP startup.

## Validation scope

Ordinary Cargo tests cover discovery/channel/version/target/network failures, independent
runtime compatibility, strict contract separation, content/publisher refusals, complete Review
guards, app-wide lock contention, publication/recovery boundaries, later edits and dangling
symlink preservation. Tooling tests compile the native UI strictly and exercise separate
Manager offers, stale replies, errors and action routing without executing an update.

The opt-in native fixture uses only owned synthetic apps and a separate HOME/CODEX_HOME. It
requires an explicitly selected Developer ID signing identity and prebuilt current binaries:

```sh
INKSCAPE_MCP_ACCEPTANCE_SIGNING_IDENTITY='Developer ID Application: … (DN263AX69U)' \
INKSCAPE_MCP_ACCEPTANCE_LAUNCHER=/absolute/current/inkscape-mcp-launcher \
INKSCAPE_MCP_ACCEPTANCE_SERVER=/absolute/current/inkscape-mcp-rust \
  cargo test --locked --manifest-path rust/Cargo.toml --lib \
  update::manager_app::native_tests::native_signed_manager_replacement_restart_and_recovery \
  -- --ignored --exact --nocapture
```

Only the signing subprocess accesses the explicitly selected identity in the existing
keychain; application/restart/profile operations use synthetic paths. The fixture checks
verified staging, Manager-only/coordinated bootstrap updates, fresh/prepared/damaged roots,
publication and idempotent recovery, failed restart/retry, real Launch Services restart and
profile/channel/installation preservation. It keeps a pre-update MCP session and verifies
initialize → tools/list exactly against the current contract → get_workspace_info through
the permanent launcher after update/recovery. Failed native profiles are retained for diagnosis.

This automated AppKit/Launch Services evidence is separate from interactive product GUI
acceptance, notarized final-package/Gatekeeper/browser-download qualification, native
`/Applications` permissions prompts and another-Mac acceptance. Those remain release gates;
a Developer ID synthetic fixture does not replace them. Follow
[development storage retention](development-storage.md) for builds, evidence and cleanup.
