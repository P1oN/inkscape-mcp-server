# Independent instructions and runtime updates

Current sources add a permanent native launcher and a macOS management application.
These features are not in the published v0.1.2 archives. Update downloads become available
when an immutable distribution with `inkscape-mcp-update.json` is published. Historical
releases without that contract are skipped; an explicit request for one explains the
manual-installation requirement. This change does not publish a release or migrate an
existing user installation automatically.

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
require a manual launcher upgrade rather than automatic self-replacement.

## Graphical management

Open `Inkscape MCP Manager.app` inside the permanent installation from Finder. It shows
installed runtime/revision, instruction and launcher versions; choose an update channel and
check available versions, update instructions/runtime/both, or roll back. Use **Choose
installation** for an isolated/custom directory. Opening it only reads offline status.
The app calls fixed CLI commands with JSON results; all download, merge, compatibility,
transaction and recovery logic stays in Rust. Update failures and reconnect/reload requests
are displayed in the window. It does not launch Inkscape or restart clients.

The application is ad-hoc signed, not Developer ID signed or notarized. Native management
window acceptance is separate from Inkscape drawing/Undo acceptance and clean-machine use.

## Recovery and preservation

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
still comes from the selected successful main CI run; clean-machine, real Claude and
broader native drawing acceptance remain separately scoped.

See [CONTRIBUTING](../../CONTRIBUTING.md) and the acceptance ledger for the automated and
native checks performed for this implementation.
