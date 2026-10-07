# Inkscape MCP server — Rust

[![M8ven status](https://m8ven.ai/badge/mcp/p1on/inkscape-mcp-server?variant=verified)](https://m8ven.ai/mcp/p1on/inkscape-mcp-server?s=readme)

A native Rust STDIO MCP server for inspecting, authoring and editing SVG, rendering/exporting
with Inkscape, and controlling bounded live operations. The full surface has 112 tools,
7 prompts and 18 resources. Original files, workspace boundaries, snapshots, approvals,
Operation Records, atomic rollback and genuine no-op behavior remain part of the design.

[Documentation index](docs/README.md) · [Current status](docs/AGENT_HANDOFF.md) ·
[Backlog](docs/RUST_NEXT_PLAN.md) · [History](docs/history/README.md) · [ADRs](docs/adr/README.md)

## Install and run

[v0.1.2](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.2) provides current
source installation and a ready Apple Silicon package. It includes installation/client
management, complete Python removal, reviewed live packages and vector-quality guards
from PRs #8–11. Older v0.1.0/v0.1.1 previews retain their original behavior.

For v0.1.2 sources or current sources on Apple Silicon macOS 15+, install Inkscape 1.4+
and the client (v0.1.2 also needs its CLI),
then run from the checkout:

```sh
./setup.sh --install-skill codex --connect-client codex
./run-mcp.sh --doctor
```

Use `claude` for both client options to connect Claude Code. Setup asks for an existing
SVG workspace and optional monitoring, prepares missing build tools privately and builds
on first use. Keep the checkout in its permanent location: the client stores its absolute
`run-mcp.sh` path. Apple Command Line Tools are required; if missing, complete Apple's
installation dialog and rerun setup. Existing developer tools are preserved; source build/development tools use Rust and Bash; ready packages contain no Python runtime.

Connection checks MCP initialization, required tools and a first workspace request before
registering through the client CLI. In current sources, setup replaces the existing `inkscape` transport binding while
preserving other client settings. v0.1.2 archives still refuse a differing entry. Restart/reconnect the client after installation. Default setup refreshes existing registrations and skills;
`--connect-client` is required to register a new client.

For the published v0.1.1 source archive, use its original commands instead:

```sh
./setup.sh --bootstrap
./run-mcp.sh --doctor
```

See [downloads and checksums](docs/install/github-builds.md),
[installation](docs/install/install.md) and [source build prerequisites](docs/install/local-bootstrap.md).
Ready runtime packages need Inkscape and no user-installed compiler or Python.

## Update and remove

Current-source setup refreshes an existing installation automatically: runtime selection,
MCP registration and installed skill. Saved workspace/Inkscape/live/engine/Sentry choices
and unrelated Codex/Claude configuration survive; missing optional settings receive defaults.
This upgrade policy is not included in the already published v0.1.2 archives.

```sh
./setup.sh --version
./setup.sh --rebuild
./scripts/install-skill.sh --client codex --update
./uninstall.sh --client codex
```

Setup reruns preserve workspace/Inkscape/live/engine and monitoring settings, and reuse a
complete saved runtime when source revisions match. A changed committed revision triggers
rebuilding; unknown revisions or local edits also trigger a rebuild in current sources.
`--local-tools` builds offline using existing pinned prerequisites; `--build` is its alias.
`--bootstrap` is the compatibility alias for automatic preparation/rebuild.
Build options cannot be combined with `--package DIRECTORY`.

Current setup replaces installed skill contents and archives the old tree outside skill discovery.
The standalone `install-skill.sh --update` still merges customizations against the upstream baseline. Uninstall disconnects the matching client entry and archives local settings,
builds and an owned default-location skill; drawings remain in the workspace.
Client `config`, `check`, `connect`, `disconnect` and `uninstall` use the packaged native
Rust CLI through `scripts/mcp-client.sh`. Management does not depend on Python;
disconnect/uninstall remain available with a damaged helper runtime.
See [client management and clean reinstall](docs/install/client-management.md) for
manual configuration, standalone checks, legacy skill handling and recovery.

Startup/reconnect/doctor never launch Inkscape. Explicit GUI launch requires a user request.
Live operations use the selected open drawing; headless operations use working copies.
Clean-machine installation, real Claude Code client acceptance and broader native GUI checks
remain follow-up work; [current status](docs/AGENT_HANDOFF.md) records validation scope.

## Optional agent skill

The repository includes [inkscape-mcp](skills/inkscape-mcp/SKILL.md), a portable skill
for tool discovery, SVG authoring, live drawing edits, preview/refinement and export.
Install it alongside server configuration:

```sh
./setup.sh --install-skill codex
```

Use `--install-skill claude` for Claude Code. New skill installation is optional. Current setup replaces an existing installed skill,
archiving its complete previous tree; `--update-skill` remains a compatibility alias.
Use standalone `install-skill.sh --update` to merge customizations instead. Skill installation alone does not register MCP.
For an already configured server, run `./scripts/install-skill.sh --client codex`
without rebuilding. See [skill installation](docs/install/agent-skill.md) for paths
and other clients. These additions are included in v0.1.2 source and ready packages; the historical
v0.1.1 archive does not include them.

## Optional monitoring

Sentry error capture and sampled tool tracing are available in rebuilt Rust binaries.
`setup.sh` asks whether to enable reporting, reads the DSN without echo and saves a private
Git-ignored local file. Choose an environment label such as `wife` to distinguish computers.
See [configuration and release guidance](docs/operations/sentry.md).
Revision and build IDs are compiled automatically into telemetry and package metadata;
`--version` displays the installed values. Existing published archives remain unchanged.

## Development

The legacy Python MCP server, its tests and paired Python/Rust comparison scripts are retired.
Development now uses Rust regression/invariant tests, true STDIO and package/native acceptance.
The managed GUI session now runs through a separate native Rust supervisor. The socket snapshot bridge now uses the Rust `inkscape-mcp-live` executable. Development and package tools now use Rust/Bash; ready packages contain no CPython or helper wheels. One-shot native insertion
and the ten fixed selection edits run through the Rust `inkscape-mcp-inx` executable;
shared SVG kernels prepare and apply bounded candidates. See
[their semantics and limits](docs/live/live-helper-kernels.md). See [CONTRIBUTING.md](CONTRIBUTING.md), [runtime components](runtime/README.md),
[remaining work](docs/RUST_NEXT_PLAN.md) and [handoff](docs/AGENT_HANDOFF.md).
Historical migration reports and raw evidence are retained as history, not an active Python oracle.

Use editable vector geometry and ordinary named groups for semantic objects; layers organize
the scene. No bitmap tracing or embedded raster substitute. See
[agent usage](docs/agent-usage-guide.md). Generated [llms.txt](llms.txt) and
[llms-full.txt](llms-full.txt) describe the actual Rust MCP surface.

## Status and license

Current status and validation limits are maintained in [the handoff](docs/AGENT_HANDOFF.md);
next work is in [the plan](docs/RUST_NEXT_PLAN.md). Checkpoint reports remain historical evidence.
The project is MIT licensed; bundled dependency licenses are recorded separately.

Editable vector authoring guidance is shared by initialization and `compose_artwork`.
`quality_report` accepts explicit stroke/group roles and reports bounded read-only
structural and hidden-geometry advice with uncertainty, independently of validity/score.
See [the authoring guide](docs/agent-usage-guide.md#editable-vector-authoring-quality) and
[the acceptance evidence](docs/history/reports/editable-vector-authoring.md); silhouettes and occlusion
remain rendered/manual reviews, with existing approval gates for repairs.

The full live profile adds bounded static computed paint/resource inspection and reviewed
style/transform/text packages. See [reviewed live workflow](docs/live/reviewed-workflow.md)
for guards, preview/refusal behavior and scoped native GUI Undo/Redo evidence and deferred artist acceptance.

Explicit path-quality guards support closed silhouettes and exact zero-length segment
refusal. `quality_report` adds closure/node diagnostics and conservative whole-document
vector status; `save_document_as(vector_only=true)` refuses raster or unknown resource
content. `replace_svg_fragment(dry_run=true)` returns a structural candidate without
changing the drawing. See [usage and limits](docs/agent-usage-guide.md#closure-duplicate-path-nodes-and-vector-only-delivery).

Current sources also support [independent instructions/runtime updates](docs/install/independent-updates.md) through a
permanent native launcher and macOS management app. Opt in once with
`./setup.sh --independent-updates`; published v0.1.2 archives retain their original behavior.
