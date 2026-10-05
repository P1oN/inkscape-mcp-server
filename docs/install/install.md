# Install the Rust Inkscape MCP server

Install Inkscape 1.4 or newer. Choose the instructions for your source/package version:

| Distribution | First setup | Client and skill management |
|---|---|---|
| v0.1.2 source archive / current `main` | `./setup.sh` builds automatically on Apple Silicon macOS 15+ | `--connect-client`, skill install/update and `uninstall.sh` are available |
| Published v0.1.1 source archive | `./setup.sh --bootstrap` | Configure the launcher manually; new management options are absent |
| v0.1.2 ready runtime / current-source package | `./setup.sh` saves settings using its bundled runtime | New management options are available; no build tools required |
| Published v0.1.0 ready runtime | Follow its included setup; doctor runs during setup | Manual client configuration; no new management options |

See [Release downloads and checksums](github-builds.md) and
[source prerequisites](local-bootstrap.md). Ready runtimes need no user-installed Python,
Rust, uv or compiler. Source automatic provisioning currently supports Apple Silicon macOS 15+.

For v0.1.2 sources/ready runtime or current sources/packages:

```sh
./setup.sh --install-skill codex --connect-client codex
./run-mcp.sh --doctor
```

Use `claude` for Claude Code; the selected CLI must be installed and available on PATH.
Setup detects Inkscape, asks for an existing SVG workspace and saves private data-only
configuration. `run-mcp.sh` starts the Rust STDIO server using these saved settings.
Connection is optional and includes a bounded server handshake before client registration.
See [client management](client-management.md) for commands and scope.

Without build, `--check`, or client connection, ready-package setup only saves configuration.
`--check` runs doctor before saving; `--connect-client` executes the native Rust client CLI and starts
a separate MCP process for the handshake. Source first setup compiles/packages the runtime.
None of these paths launches Inkscape GUI. Doctor executes Inkscape CLI and checks packaged native binaries/bridge/private bus,
so macOS may assess those components. Saving settings alone
does not certify runtime readiness or macOS approval.

Reruns preserve saved choices and reuse a complete runtime when recorded revisions match.
A changed committed revision triggers rebuilding; unknown revisions or uncommitted source
edits need `--rebuild`. This option repeats automatic preparation/build. `--local-tools`
uses existing pinned prerequisites and cached dependencies offline; `--build` aliases it,
and `--bootstrap` aliases automatic rebuilding. Build modes and `--package` are exclusive.
Use `--version` for installed metadata, and the [update/removal guide](client-management.md)
for skill merging, backups and clean reinstall.

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

Python package installation/uvx entry points are retired. Ready packages, source bootstrap and active development tooling use Rust/Bash.
Windows is backlog; native GUI acceptance is separate from CLI/package checks.

Current sources also support client registration, installed build identity, managed skill
updates and clean reinstall. See [client management](client-management.md).
