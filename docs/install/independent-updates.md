# Independent instructions and runtime updates

Current sources and the v0.1.7 signed prerelease include a permanent native launcher
and a macOS management application. These features are absent from historical v0.1.2
archives. Published distributions expose updates through `inkscape-mcp-update.json`. Historical
releases without that contract are skipped; an explicit request for one explains the
manual-installation requirement. Publishing a release does not migrate an existing
user installation automatically.

The [v0.1.7 signed prerelease](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.7)
includes the native installation wizard, which prepares and verifies offline runtime/text,
client/skill changes and explicit bootstrap upgrades. The per-user app location is
`~/Applications/Inkscape MCP Manager.app`; existing legacy migration remains available.
See the [macOS distribution contract](macos-manager-distribution.md) for compatibility 2,
signed-bundle layout, recovery and qualification limits. Compatibility format remains 1;
text-only assets continue to support compatibility 1.

## One-time migration

On Apple Silicon macOS 15+, select/build a current complete package using the existing
setup flow, then opt into independent updates:

```sh
./setup.sh --rebuild --independent-updates
```

Existing settings and unrelated client preferences survive. Migration validates the
complete staged package with doctor and real STDIO/workspace checks before redirecting
recorded Codex/Claude bindings. It backs up the original settings and client configuration.
With this option, setup merges existing managed skills instead of replacing customizations.
An old ready package needs a current package first; updating just its server executable is
not sufficient. New clients still need `--connect-client codex` or `claude` during setup.

The default installation is `~/Library/Application Support/inkscape-mcp`. For an isolated
profile, pass `--install-dir /absolute/separate/directory`; the installation must not overlap
the source/ready package or drawing workspace. The source checkout and its local rebuild
flow remain available. Moving the old source/ready directory after migration does not
change the permanent client command.

For an already configured current package, the equivalent migration command is:

```sh
/absolute/package/bin/inkscape-mcp-launcher migrate --source /absolute/configured/directory
```

Retry that same command if a client binding change failed after staging. It checks the
existing installation before retrying; it does not replace the permanent launcher.

## CLI workflow

Make the native entry point available in the current shell:

```sh
export PATH="$HOME/Library/Application Support/inkscape-mcp/bin:$PATH"
inkscape-mcp --help
inkscape-mcp --version
inkscape-mcp update --check
inkscape-mcp update --instructions
inkscape-mcp update --runtime
inkscape-mcp update
inkscape-mcp rollback
```

To keep this PATH in future shells, add the export to your shell profile. The absolute-path
fallback is `"$HOME/Library/Application Support/inkscape-mcp/bin/inkscape-mcp"` followed by
these same arguments; commands work from any directory. Client registration always uses
the permanent `bin/inkscape-mcp-launcher` path and does not depend on PATH.

The default update channel is stable. Explicitly opt into prereleases with
`inkscape-mcp update --channel prerelease`; pass `--release vX.Y.Z` to select an immutable
version. A successful explicit update saves the channel, including a component no-op.
`--check` is read-only and does not save a channel or recover interrupted transactions.

Instruction updates contain server initialization, shared authoring guidance, prompt text
and the managed skill. Prompt names/arguments, schemas, typed tool implementations,
workspace guards and approval enforcement remain compiled. Runtime-only updates preserve
the currently selected text when compatible; incompatible pairs refuse and explain how to
request a combined update. Startup loads a single validated bundle once; running servers
retain their original text. An explicitly selected corrupt bundle fails clearly.

Progress goes to stderr; management results go to stdout. `--json` returns structured
results for version/check/update/rollback and structured operation/argument errors.
Exit codes are 0 for success or no update, 1 for failed operations and 2 for invalid
arguments. Commands never wait for input. With no arguments, the launcher starts MCP
offline and only JSON-RPC reaches stdout; startup never checks GitHub, launches Inkscape,
or restarts a client.

After activation, reconnect MCP and reload its skill when the result requests it. Existing
Inkscape sessions retain their original native helpers. Protocol compatibility is checked
before activation and the existing live wire checks reject incompatible helpers. No version
is garbage-collected in this iteration, so retained servers/supervisors/helpers keep their
paths. The launcher has a separate version; unsupported formats/minimum launcher versions
require an explicit Manager/launcher upgrade. The new wizard stages that upgrade after
exit; automatic background replacement remains out of scope.

## Graphical management

Manager itself now has a separate **Review Manager update…** action followed by explicit
**Update Manager and restart**. Its asynchronous check works even when runtime discovery
requires a newer launcher or settings.json does not exist. The signed app/bootstrap plan,
profile-preserving restart and interruption recovery are described in
[reviewed Manager updates](manager-self-update.md). Historical manifests remain unchanged.


Open `Inkscape MCP Manager.app` inside the permanent installation from Finder. It shows
installed runtime/revision, instruction and launcher versions; choose an update channel and
check available versions, update instructions/runtime/both, or roll back. Use **Choose
installation** for an isolated/custom directory. Current-source Manager also checks
GitHub asynchronously on each launch, with a 20-second overall deadline. Local setup
stays available offline. This check reads release metadata only; it never installs,
activates or recovers anything. MCP startup/reconnect remains offline.
The app calls fixed CLI commands with JSON results; all download, merge, compatibility,
transaction and recovery logic stays in Rust. Update failures and reconnect/reload requests
are displayed in the window. It does not launch Inkscape or restart clients.

Checks use the selected stable/prerelease channel, validated release contracts and actual
installed component identities. Before installation is complete, they use sealed offline
runtime/text metadata without requiring settings.json. Numeric release ordering prevents
an older release being offered over a newer installed distribution. A different unpublished
build can still be offered when its distribution has no comparable version.
Manual **Check updates** remains available before installation. A first-launch offer
explains that the offline installation must be completed before applying an online update.
**Review update**, **Instructions only** and **Runtime only** show the verified component
plan before **Update now**. Activation pins the release and rejects a changed review
identity. Checks coalesce; changing choices/root/channel or starting review/activation
invalidates an outstanding reply.

Published v0.1.7 is Developer ID signed/notarized; local development apps use ad-hoc
signatures. Native management-window acceptance is separate from automated AppKit checks,
Inkscape drawing/Undo acceptance and clean-machine use.

The client card displays the effective Codex profile and config.toml, honoring CODEX_HOME.
An existing other installation is offered as an explicit source selection. A removed legacy
source is eligible for recovery only when its exact registration, matching skill owner and
known upstream baseline agree. Known baselines are compiled project skill bytes or a fully
validated staged instruction bundle. A similar path name alone, an unknown baseline or an
unverified skill is preserved; deselect that client or reconcile its configuration manually.
Discovered launchers are never executed to establish ownership.

**Review installation** lists a confirmed stale-registration repair and managed-skill
transfer with its private backup directory. Only **Install now** publishes them. The
transaction retains the full original config and skill (including baseline/additions), uses
the existing three-way merge/conflict refusal, and rechecks config/whole-skill identities
and source absence before activation. Later edits or a reappearing source require a new
review. Interrupted publication restores the old config and skill owner together; unrelated
settings, environment values, registrations and artwork stay preserved.

## Recovery and preservation

MCP startup waits up to two seconds for a brief installation-lock conflict, then reports
a retryable error if it is still busy; update commands retain immediate lock refusal.
Updates serialize under an installation lock and download only from the fixed GitHub
repository through bounded HTTPS and approved redirect hosts. Lengths/SHA-256, archive
paths/links/entry/decompression limits, identities, target and interface compatibility are
checked before selection. Checksums from the same release detect corruption; they are not
independent publisher authentication. Downloaded setup scripts are never executed.

`runtime/<build-id>` and `instructions/<content-id>` are retained immutable selections.
`active.json` binds one pair; settings live in private `settings.json`. Managed paths refuse
symlinks, foreign ownership and writable-by-others paths. Skill merges use the existing
three-way baseline semantics and preserve extra user files. Conflicts retain proposals
outside skill discovery and leave the selected pair and installed skills unchanged.
A removed skill destination is skipped and reported in CLI/GUI results; updates and rollback
do not recreate it. Rollback also preserves destinations that were not changed by the
selected update, including a skill recreated by the user after a skipped update.

A durable journal covers skill-directory moves and the atomic selector change. Launch or
update recovers an interrupted transaction to the preceding pair before continuing. If
someone edited a skill after interruption, recovery preserves it and reports its backup
path. Rollback also refuses to overwrite later skill edits. Versions and transaction
backups are retained; automatic cleanup is deferred. Source/configuration/drawings and
Inkscape processes are preserved.

`inkscape-mcp disconnect --client codex` (or `claude`) removes an owned binding without
requiring runtime helpers. `inkscape-mcp uninstall` disconnects recorded owned clients and
archives the permanent installation and owned skills beside it. It preserves drawings,
other bindings and the original source installation. Recovery needs the permanent launcher;
if it is damaged, restore that executable manually from a matching verified launcher asset.

## Asset production and publication

Native CI produces instruction and launcher archives/checksums plus an update-manifest
template alongside the complete runtime/source assets. The launcher archive includes the
management app. `build-instructions` derives shared guidance from
`migration/contracts/authoring-guidance.txt`; there is no second editable policy source.
`build-update-manifest` validates instruction and complete runtime identities without
executing archive contents. Publication binds the verified template to the new distribution
tag and refuses existing tags/releases.

The release workflow's optional `runtime_tag` references a previously published immutable
compatible runtime. Publication verifies its archive identity and retained publication
evidence, then references the original distribution rather than uploading/replacing it.
Publication itself builds nothing and executes no archive content. Instruction evidence
still comes from the selected successful main CI run. Real Claude CLI discovery passes
on v0.1.7 signed bytes; authenticated workspace interaction, clean-machine and broader
native drawing acceptance remain separate gates.

See [CONTRIBUTING](../../CONTRIBUTING.md) and the
[acceptance ledger](../history/reports/independent-updates.md) for the automated and native
checks performed for this implementation.
