# Inkscape MCP server — Rust

A native Rust STDIO MCP server for inspecting, authoring and editing SVG, rendering/exporting
with Inkscape, and controlling bounded live operations. The full surface has 110 tools,
7 prompts and 18 resources. Original files, workspace boundaries, snapshots, approvals,
Operation Records, atomic rollback and genuine no-op behavior remain part of the design.

## Install and run

For M-chip Macs running macOS 15+, download `inkscape-mcp-source-bootstrap.tar.gz`
and its checksum from [Release v0.1.1](https://github.com/P1oN/inkscape-mcp-server/releases/tag/v0.1.1).
See [download and checksum instructions](docs/install/github-builds.md).

Install Inkscape 1.4 or newer. From the unpacked source archive:

```sh
./setup.sh --bootstrap
./run-mcp.sh
```

The published v0.1.1 archive requires `--bootstrap` on first use. In current sources,
simply run `./setup.sh`: a fresh source checkout automatically prepares missing tools
and builds; subsequent runs preserve settings and reuse a complete saved package when
recorded source revisions match. A changed committed revision triggers rebuilding;
unknown revisions or uncommitted edits require an explicit `--rebuild`.
A ready package only saves
configuration. Setup detects Inkscape and asks for an existing SVG workspace.
Use `./setup.sh --check` or
`./run-mcp.sh --doctor` for an explicit runtime diagnosis. Rerunning setup reuses the
configured package. On Apple Silicon macOS 15+, automatic setup downloads missing
tools into a private temporary directory,
build locally and remove build tools afterward. Existing tool installations stay intact;
private Python required by live helpers stays inside the package. Apple Command Line Tools
are a system prerequisite; if missing, complete their Apple installation dialog and rerun.
Use `./setup.sh --local-tools` to rebuild with existing tools and cached dependencies
only, without downloads. `--build` remains an alias; `--bootstrap` explicitly rebuilds
with automatic preparation for compatibility.
No manual env editing is required. See [local bootstrap](docs/install/local-bootstrap.md). A source build needs Rust and native build dependencies;
the private helper runtime is prepared from pinned dependencies. A ready macOS arm64 archive
needs only Inkscape. Exact prerequisites, archive paths and limits are documented in
[installation instructions](docs/RUST_MIGRATION_REPORT.md#install-and-check-the-current-local-candidate).
Clean-machine installation and Windows remain pending; see the [current status](docs/AGENT_HANDOFF.md).

For current sources, configure the client and optional skill directly:

```sh
./setup.sh --install-skill codex --connect-client codex
# Claude Code: use claude for both options.
./setup.sh --version
./setup.sh --rebuild
```

Connection verifies MCP initialization, required tools and a first read-only workspace
request before registering through the client CLI. It preserves other server entries and
refuses a differing existing `inkscape` entry. Both CLIs use user scope; Claude project/local
entries may take precedence. Restart/reconnect the client after installation.
See [client connection and removal](docs/install/client-management.md) for config snippets,
standalone checks and clean reinstall instructions. Published v0.1.1 remains unchanged;
these options require a package built from current sources.

Startup/reconnect/doctor never launch Inkscape. Explicit launch requires a user request.
Live operations change the selected open document; headless operations use working copies.

## Optional agent skill

The repository includes [inkscape-mcp](skills/inkscape-mcp/SKILL.md), a portable skill
for tool discovery, SVG authoring, live drawing edits, preview/refinement and export.
Install it alongside server configuration:

```sh
./setup.sh --install-skill codex
```

Use `--install-skill claude` for Claude Code. Installation is optional and preserves
different existing skills. Add `--update-skill` to merge an existing managed skill;
conflicts leave installed content unchanged. Skill installation alone does not register MCP.
For an already configured server, run `./scripts/install-skill.sh --client codex`
without rebuilding. See [skill installation](docs/install/agent-skill.md) for paths
and other clients. These additions are in the current source tree; the already
published v0.1.1 archive does not include them.

## Optional monitoring

Sentry error capture and sampled tool tracing are available in rebuilt Rust binaries.
`setup.sh` asks whether to enable reporting, reads the DSN without echo and saves a private
Git-ignored local file. Choose an environment label such as `wife` to distinguish computers.
See [configuration and release guidance](docs/sentry.md).
Revision and build IDs are compiled automatically into telemetry and package metadata;
`--version` displays the installed values. Existing published archives remain unchanged.

## Development

The legacy Python MCP server, its tests and paired Python/Rust comparison scripts are retired.
Development now uses Rust regression/invariant tests, true STDIO and package/native acceptance.
Python remains only for live helper extensions, the managed supervisor and development/package
scripts. See [CONTRIBUTING.md](CONTRIBUTING.md), [runtime components](runtime/README.md),
[current plan](docs/RUST_NEXT_PLAN.md) and [handoff](docs/AGENT_HANDOFF.md).
Historical migration reports and raw evidence are retained as history, not an active Python oracle.

Use editable vector geometry and ordinary named groups for semantic objects; layers organize
the scene. No bitmap tracing or embedded raster substitute. See
[agent usage](docs/agent-usage-guide.md). Generated [llms.txt](llms.txt) and
[llms-full.txt](llms-full.txt) describe the actual Rust MCP surface.

## Status and license

Current status and validation limits are maintained in [the handoff](docs/AGENT_HANDOFF.md);
next work is in [the plan](docs/RUST_NEXT_PLAN.md). Checkpoint reports remain historical evidence.
The project is MIT licensed; bundled dependency licenses are recorded separately.
