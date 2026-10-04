# Install the Rust Inkscape MCP server

Install Inkscape 1.4 or newer and unpack a ready archive; see
[Release downloads and checksum verification](github-builds.md).
Run `./setup.sh` in the unpacked package. It detects Inkscape, asks for an existing
SVG workspace and saves private, data-only configuration files. No manual environment
editing is needed. It does not execute the server, private Python or Inkscape.
`./run-mcp.sh` starts the Rust STDIO server using the saved configuration.

Diagnostics are explicit: `./setup.sh --check` checks before saving settings, or
`./run-mcp.sh --doctor` checks an existing configuration. Doctor executes Inkscape CLI
and imports private Python/inkex/native modules, so macOS may assess those components.
Successful config-only setup does not certify runtime readiness or macOS approval.

From a current source checkout on Apple Silicon macOS 15+, `./setup.sh` automatically
prepares missing tools and builds on first use; see [local bootstrap](local-bootstrap.md).
Use `./setup.sh --local-tools` to rebuild using existing developer prerequisites and
cached dependencies only, without downloads. If something is missing, setup stops
with an explanation. `--build` aliases this mode; `--bootstrap` explicitly repeats
automatic tool preparation/build. Build modes cannot be combined with `--package`.
The already published v0.1.1 source archive still needs `--bootstrap` on first use.
Source packaging requires Rust, native build dependencies and a pinned helper environment
or uv. Subsequent setup runs preserve settings and reuse the configured package when
recorded source revisions match; a changed committed revision triggers a rebuild.
Unknown revisions/uncommitted edits require an explicit `--rebuild`.
Ready archive users need only Inkscape.

The v0.1.0 Release can show “Apple cannot check … for malicious software” for
private runtime components. This is Gatekeeper, not missing administrator privileges;
`sudo` does not provide Developer ID signing or notarization. The config-only launcher
changes are available in current sources and not included in that historical Release. A later runtime operation
can still trigger assessment of an unsigned/unnotarized component.

macOS security approval depends on the downloaded package's signatures/notarization and
privacy permissions. Saving a config or using Terminal cannot guarantee at most one
prompt for the current ad-hoc signed package. Setup does not remove quarantine attributes
or change macOS security/privacy settings. See
[Apple's app security explanation](https://support.apple.com/en-ie/102445).

Python package installation/uvx entry points are retired. Python remaining inside the
ready package serves bounded live helpers and the supervisor, not an MCP server.
Windows is backlog; native GUI acceptance is separate from CLI/package checks.

Current sources also support client registration, installed build identity, managed skill
updates and clean reinstall. See [client management](client-management.md).
