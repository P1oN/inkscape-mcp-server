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

Setup detects Inkscape, asks for an existing SVG workspace and saves local configuration
without executing the server, Python imports or Inkscape. Use `./setup.sh --check` or
`./run-mcp.sh --doctor` for an explicit runtime diagnosis. Rerunning setup reuses the
configured package. From a source checkout on Apple Silicon macOS 15+, use
`./setup.sh --bootstrap` to download missing tools into a private temporary directory,
build locally and remove build tools afterward. Existing tool installations stay intact;
private Python required by live helpers stays inside the package. Apple Command Line Tools
are a system prerequisite; if missing, complete their Apple installation dialog and rerun.
Use `./setup.sh --build` when all developer prerequisites are already installed.
No manual env editing is required. See [local bootstrap](docs/install/local-bootstrap.md). A source build needs Rust and native build dependencies;
the private helper runtime is prepared from pinned dependencies. A ready macOS arm64 archive
needs only Inkscape. Exact prerequisites, archive paths and limits are documented in
[installation instructions](docs/RUST_MIGRATION_REPORT.md#install-and-check-the-current-local-candidate).
The historical migration checkpoint archive stage38 includes protected SVG/CSS asset staging and bounded STDIO
requests; cold/warm installation, launcher and doctor pass. Current native checks and Rust-only headless/live measurements pass; see the report for scope.
Clean-machine installation will be checked later by the user. Windows is backlog.

MCP clients use the absolute launcher path:

```json
{"mcpServers":{"inkscape":{"command":"/absolute/path/to/inkscape-mcp-server/run-mcp.sh"}}}
```

Startup/reconnect/doctor never launch Inkscape. Explicit launch requires a user request.
Live operations change the selected open document; headless operations use working copies.

## Optional monitoring

Sentry error capture and sampled tool tracing are available in rebuilt Rust binaries.
`setup.sh` asks whether to enable reporting, reads the DSN without echo and saves a private
Git-ignored local file. Choose an environment label such as `wife` to distinguish computers.
See [configuration and release guidance](docs/sentry.md).
A ready archive with this wizard is
`migration/results/packages/inkscape-mcp-macos-arm64-sentry-setup.tar.gz`.
Existing stage38 archives are unchanged and do not include Sentry.

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

Current local development targets macOS arm64; prepared native POSIX CI jobs are not remote
validation results. Historical live incident investigation is deferred unless it recurs.
Current native acceptance, Rust-only headless/live measurements, dependency attribution and
scoped security checks are recorded in the migration report. The current-Mac migration
scope is complete; clean-machine and foreign-target checks remain explicitly deferred.
The project is MIT licensed; bundled dependency licenses are recorded separately.
