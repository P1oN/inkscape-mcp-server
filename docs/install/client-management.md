# Client connection, updates and clean reinstall

These options are committed in [PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8).
Use that branch or a package built from it; published v0.1.1 assets lack these options.
Install Inkscape first, then configure an existing SVG workspace with setup.

```sh
./setup.sh --connect-client codex --install-skill codex
./setup.sh --connect-client claude --install-skill claude
./setup.sh --version
./setup.sh --rebuild
```

Choose one client, or connect each separately. `--rebuild` explicitly repeats automatic
provisioning/build; `--local-tools` rebuilds offline with existing pinned developer tools.
Both retain saved workspace/Inkscape/live/engine and monitoring choices. Ready packages use
`--package DIRECTORY`. Version output reads installed package metadata without starting MCP
or Inkscape. Older metadata may have only source_head; rebuild for full build identity.

The client must be installed and its CLI available on PATH. Registration uses
[Codex's CLI](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) or
[Claude Code's CLI](https://code.claude.com/docs/en/mcp), never arbitrary config-file rewrites.
Codex respects CODEX_HOME; Claude registers in user scope. Other servers are preserved.
A differing existing inkscape entry is refused; explicitly rename/remove it first.
Claude local/project configuration may shadow a user entry; check `/mcp` in the client.

```sh
./scripts/mcp-client.sh --client codex config
./scripts/mcp-client.sh --client claude config
./scripts/mcp-client.sh --client codex check
./scripts/mcp-client.sh --client codex connect
./scripts/mcp-client.sh --client codex disconnect
```

`config` prints a ready TOML/JSON snippet for manual use. `check` launches only the configured
MCP process, with bounded initialization/discovery/workspace requests and shutdown. `connect`
runs the same check before registration and verifies saved user configuration afterward.
This verifies server availability; it does not run a model session or prove a client's
permission choices. Restart/reconnect the client to load the changed server and skill.
No path launches Inkscape GUI. The management helper uses packaged private Python.

For [skill updates](agent-skill.md), use `--update-skill` with `--install-skill`, or the
standalone installer with `--update`. Customizations merge against the saved baseline;
conflicts preserve installed content and leave a proposed merge for review.

## Remove and reinstall

```sh
./uninstall.sh --client codex
# Or: ./uninstall.sh --client claude
./setup.sh --install-skill codex --connect-client codex
```

Uninstall removes only a matching user-scoped client entry and moves `.inkscape-mcp-local`
into `.inkscape-mcp-backup-TIMESTAMP` beside the source. It also archives an owned skill
at that client's default location, including user changes. Other recorded clients must be
disconnected first. Shared/custom-destination skills remain and can be moved aside manually.
Drawings and workspace `.inkscape-mcp` working copies/snapshots remain intact. Source files,
external ready packages, system Inkscape, Rust and Apple developer tools are preserved.
Existing client/server processes are not killed; close/restart the client before reinstall.

The next source setup builds afresh. A ready archive can reuse its bundled runtime after
local settings are archived. The backup contains private settings (including a saved DSN);
it is Git-ignored in this repository. Keep it private or delete that explicitly identified
backup directory manually once you no longer need recovery. Moving it preserves old package
paths in setup.conf for diagnosis; rerun setup with the relocated package path to recover.

## Acceptance scope

The extracted-source acceptance script covers unpacking, native setup with existing pinned
host tools, skill installation/update/conflicts, isolated real Codex CLI registration,
handshake/discovery, a first workspace request, real SVG edit/render/save, uninstall and
reinstall. It does not establish clean-machine installation, native GUI behavior or real
Claude Code client acceptance. Synthetic Claude command/config checks are separate.

If the private runtime is damaged and the management helper cannot run, remove the client
entry manually with `codex mcp remove inkscape` or `claude mcp remove inkscape --scope user`
after checking it belongs to this installation. Move `.inkscape-mcp-local` to a private
backup directory, and move an old skill aside before rerunning setup. This recovery path
requires no execution of the broken MCP binary.
