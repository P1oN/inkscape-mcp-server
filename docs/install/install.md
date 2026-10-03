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

From a source checkout on Apple Silicon macOS 15+ use `./setup.sh --bootstrap`
for automatic private tool provisioning; see [local bootstrap](local-bootstrap.md).
Use `./setup.sh --build` to build with existing developer prerequisites.
Source packaging requires Rust, native build dependencies and a pinned helper environment
or uv. Subsequent setup runs reuse the configured package instead of rebuilding it.
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
