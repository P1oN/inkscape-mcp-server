# Independent updates — original implementation plan

Archived on **2026-10-07** after implementation. The text below preserves the user’s
original design; proposed commands describe that historical plan. See the
[current guide](../../install/independent-updates.md) and
[acceptance ledger](../reports/independent-updates.md) for implemented behavior and limits.

## Independent instructions and runtime updates — implementation plan

Requested on **2026-10-07**. This section is a design and unfinished implementation plan;
commands and layouts below are proposed, not available features. It does not authorize
changing the user's installed runtime or publishing a release.

### Outcome and current constraints

After one migration, users update the skill, MCP instructions/prompts or ready runtime
without repeating setup, client registration, source compilation or manual archive extraction.
The initial supported downloadable target remains Apple Silicon macOS 15+.

- `scripts/install-skill.sh --update` already performs a local three-way merge, but downloads
  nothing. Setup currently replaces skills and archives the old tree. The new updater must
  preserve customizations through the merge path; replacement remains an explicit choice.
- `rust/src/authoring.rs`, `contract.rs` and `prompts.rs` embed guidance, initialization
  instructions and prompt templates with `include_str!`. Text-only updates need a new loader.
- `run-mcp.sh` reads repository-local configuration and executes an absolute package binary.
  Runtime packages include five Rust executables, bridge/native dependencies and metadata;
  replacing only the server binary would leave an inconsistent installation.
- Package/archive/bootstrap validation exists in development-only `rust/tooling`. Reuse the
  validated logic through a narrowly scoped shared module/crate where needed; do not ship
  the development CLI or invoke arbitrary downloaded scripts.
- Existing GUI sessions retain their original helpers. Reconnecting the MCP must preserve
  those sessions and explicitly check helper protocol compatibility.

### Architecture and proposed interface

Use a permanent per-user launcher/configuration directory, independent of the checkout and
downloaded versions. On macOS, use `~/Library/Application Support/inkscape-mcp` by default,
with a validated explicit installation-directory override for isolated profiles/tests.
Its proposed contents are `bin/inkscape-mcp-launcher`, private configuration,
`runtime/<build-id>/`, `instructions/<content-id>/`, staging, backups and an atomic
`active.json` selector. Store runtime and instruction identities together in this selector;
launch resolves one immutable pair. Protect managed paths against symlink/path traversal.

The launcher is a native Rust executable separate from the replaceable runtime. With no
management arguments it starts MCP with inherited STDIO and existing settings; only MCP
JSON-RPC reaches stdout. Management output is separate. Initial management commands:

```text
inkscape-mcp-launcher update --check
inkscape-mcp-launcher update --instructions
inkscape-mcp-launcher update --runtime
inkscape-mcp-launcher update
inkscape-mcp-launcher rollback
inkscape-mcp-launcher --version
```

Instruction updates include the managed skill and server text. `update --runtime` keeps the
current instructions only if compatible; otherwise refuse and recommend the combined update.
`update` chooses a compatible release pair. Checking is read-only. Rollback restores the
previous compatible pair and owned skill state, refusing to overwrite later user edits.
Report runtime build/revision, instruction content/version, launcher version, channel and
whether a client reconnect/skill reload is needed.

Start with explicit updates and offline-capable startup; MCP startup never performs network
checks or installs an update. Support stable and explicitly selected prerelease channels:
GitHub's latest-release endpoint alone is insufficient for prereleases. The launcher itself
has a separate version; this first iteration reports an unsupported manifest/required launcher
upgrade with a manual recovery path rather than attempting self-replacement.

Deliver a usable CLI first, then a graphical management interface. Install a documented
`inkscape-mcp` CLI entry point accessible from any directory, with explicit PATH setup or an
absolute-path fallback; keep the client's registered launcher path permanent. Provide concise
`--help`, current/available versions, readable download/check/activation progress, actionable
errors, consistent exit codes and a structured output mode for the later GUI. Keep management
commands distinct from MCP STDIO startup. Noninteractive use must never wait for input.

### Ordered implementation stages

1. **Define manifests and compatibility contracts.** Add typed, versioned release/runtime and
   instruction manifests: distribution tag, source/build/content identity, OS/architecture and
   minimum OS, asset names/bytes/SHA-256, instruction format, text interface compatibility,
   launcher minimum version and native-helper protocol compatibility. Instruction artifacts
   contain skill files, initialization text and prompt templates, never tool implementations
   or tool schemas. Keep prompt names/arguments and approval enforcement in compiled code.
   Derive shared authoring text from `migration/contracts/authoring-guidance.txt`; do not add
   a second editable policy source. **Done:** fixtures reject unknown formats, incompatible
   pairs and unsupported targets before installation; canonical content hashes are deterministic.

2. **Load an instruction bundle without rebuilding.** Add a bounded read-only loader around
   `authoring.rs`, `contract.rs` and `prompts.rs`, selected by the launcher and loaded once per
   MCP process. Validate ownership/path, total/file sizes, hashes, required templates, approved
   placeholders and exact compiled prompt interfaces. Retain embedded defaults when no external
   bundle is configured; an explicitly configured corrupt/incompatible bundle fails clearly,
   rather than silently hiding an update failure. Atomically select instructions on reconnect;
   running sessions retain their original text. **Done:** initialization and `compose_artwork`
   share the same guidance, prompt substitution stays bounded, and text-only changes affect
   discovery without recompilation or weakening executable approval/workspace checks.

3. **Add the permanent launcher and migrate configuration.** Build/package the Rust launcher;
   reuse existing client-manager configuration parsing, bounded handshake and ownership guards.
   Setup can migrate a selected complete package and private workspace/Inkscape/live/engine/Sentry
   settings, backing up the old binding/config before a one-time registration change. Validate
   the staged installation before redirecting Codex/Claude. Keep legacy source setup and local
   rebuild available; downloaded updates never rewrite a developer checkout or its local edits.
   Coordinate one installation shared by multiple clients with their recorded skill destinations.
   Preserve disconnect/uninstall recovery even if runtime helpers are damaged.
   **Done:** relocated runtime starts through the unchanged launcher path, source and old ready
   installations migrate, settings/unrelated client config survive, and offline launch works.

4. **Implement release discovery and staged download.** Add fixed-repository GitHub release
   discovery for `P1oN/inkscape-mcp-server`, explicit channel/version selection and bounded HTTPS
   downloads. Handle API limits, missing assets, timeouts and network loss without changing the
   active installation. Validate redirect destinations, manifest/asset identities, byte limits
   and SHA-256 before extraction. Checksums from the same release detect corruption; do not
   claim independent publisher authentication. Reject traversal, unsafe links, duplicate paths
   and decompression limits using archive invariants. Never execute release-provided install
   scripts. **Done:** deterministic local HTTP/API fixtures cover discovery/download failures,
   malformed archives and checksum mismatch; current installation still starts after each refusal.

5. **Activate updates and implement recovery.** Serialize updates with an installation lock;
   stage immutable directories, run candidate doctor/STDIO initialization/discovery/workspace
   checks without GUI launch or artwork mutation, then atomically replace `active.json`. Reuse
   skill three-way merging, ownership markers, baselines and backups. Prepare all requested
   clients' skill merges before activation; a conflict leaves active runtime/text/skills intact
   and retains proposals outside skill discovery. Since skills and the selector span locations,
   use a durable transaction journal and recovery before launch/update to finish or restore a
   partial transaction after a crash. A skill write failure rolls back the requested update.
   Retain the preceding pair and any version referenced by a running server/supervisor/helper;
   defer garbage collection and avoid automatic restart of active clients. Check old native
   helper compatibility and refuse incompatible live writes with a recovery explanation.
   **Done:** concurrent updates, interrupted writes, disk exhaustion, merge conflicts, no-op
   updates and rollback preserve a launchable pair; user drawings and Inkscape processes survive.

6. **Produce update assets and complete acceptance.** Extend `rust/tooling/src/package.rs`,
   native CI, `.github/workflows/release.yml` and `scripts/publish-verified-release.sh` to produce
   instruction assets/manifests and launcher distributions from the same verified CI evidence.
   Publish text-only changes as new immutable releases referencing an already verified compatible
   runtime; never replace historical assets. Publication verifies both referenced runtime identity
   and instruction evidence without rebuilding or executing archive contents. Exercise migration,
   instructions-only/runtime-only/combined update and rollback in isolated Codex/Claude profiles;
   distinguish real Codex, synthetic Claude and real Claude evidence. Update install/skill/client/
   release docs, handoff and generated `llms.txt`/`llms-full.txt` to reflect actual loaded defaults.
   **Done:** downloaded update works without a user compiler or manual archive handling, client
   registration remains unchanged after migration, and evidence records every selected identity.

7. **Qualify the CLI user experience before GUI work.** Exercise the public `inkscape-mcp`
   entry point from an unrelated directory in an isolated user profile. Document first setup,
   update checks, component selection, channel selection, update, reconnect and rollback with
   copyable examples. Verify help, progress, errors, exit codes and structured output for success,
   offline use, unavailable updates, conflicts and interrupted updates. **Done:** a user can
   complete the supported update/recovery workflow through the CLI without editing configuration
   or manually handling archives; the management interface is stable enough for GUI reuse.

8. **Add a graphical management interface on the qualified CLI/backend.** Provide a macOS
   application that opens from Finder and shows installed/available versions, update channel,
   update status and whether a client reconnect is required. Include actions to check updates,
   update instructions, update runtime, update both and roll back, with progress and readable
   failure/recovery messages. Call the same typed update backend or fixed CLI commands using
   structured results; do not duplicate compatibility, download, merge or transaction logic.
   Choose the UI framework during this stage based on packaging and maintenance constraints.
   Opening the management window must not launch Inkscape, restart clients or apply updates.
   **Done:** users perform the same supported workflows without Terminal; GUI acceptance covers
   component updates, no-update/offline/conflict/failure states and rollback, preserving settings,
   custom skills and open Inkscape drawings. Record native GUI evidence separately from CLI tests.

### Validation and delivery boundaries

Run both locked Cargo graphs' fmt/Clippy/tests as applicable, plus discovery across all profiles,
true STDIO and relocated-package/setup/client acceptance for changed paths. Add regression cases
for manifest compatibility, instruction parsing/identity, archive/path guards, channel selection,
old-release handling, durable activation/recovery and preservation of settings/custom skills.
For publication changes run Bash syntax, actionlint and `scripts/test-publish-verified-release.sh`
with additional instruction/launcher assets and reused-runtime fixtures. Mock network failures;
automated tests must not depend on GitHub availability or publish a release.

Native GUI acceptance is separate: when explicitly authorized, use an owned synthetic session
to check that MCP update/reconnect retains the open drawing and old helper paths, and that
incompatible helpers refuse edits. Clean-machine and real Claude acceptance remain existing
backlog items; do not infer them from isolated automated fixtures.

Deliver in stage order: contracts/loader, launcher/migration, downloader/activation/recovery,
then CI/publication/docs and CLI qualification, followed by the graphical management interface
and its acceptance. Differential per-file runtime downloads, automatic scheduling,
launcher self-update, Windows and Developer ID/
notarization are follow-up scope. Runtime archives remain an internal transfer format in the
first version; users do not download or extract them manually. No new MCP update tool or
arbitrary execution capability is required.
