# Sentry monitoring

The Rust server supports optional panic/fatal capture, bounded subsystem failure events and sampled
MCP tool transactions. Set `SENTRY_DSN` on the process to enable it. With no DSN (or
an invalid DSN), no Sentry client is started. `--doctor` does not initialize telemetry.
Expected validation errors, approval/context refusals, missing live sessions and user input
errors are not reported. Ordinary nonzero CLI exits are not automatically bugs.

The project is `boryslav/inkscape-mcp-server`:
https://boryslav.sentry.io/settings/projects/inkscape-mcp-server/

## Setup without editing environment variables

Run `./setup.sh`. It asks whether to enable error reporting, accepts the DSN with hidden
input, and asks for an environment label such as `wife`. Use the same project DSN on
another computer to send its events to the same dashboard. No Sentry account or API
management token is needed on that computer. A fresh interactive setup defaults to off;
a blank answer preserves an existing saved setting.

Settings are data-only in `.inkscape-mcp-local/sentry.conf`, written atomically with mode600.
This directory is ignored by the repository and has its own `*` ignore file to protect an
unpacked package placed in another Git checkout. The DSN is not built into public code or
archives and is never printed by setup. `run-mcp.sh` reads the file without evaluating shell
code; saved enabled settings override client DSN/environment values. Saved opt-out clears
`SENTRY_DSN` and disables tracing even if the client supplied a DSN. Other existing client
options (release/sample rate) are preserved when enabled. Without a local Sentry file,
legacy process environment configuration below continues to work.

Unattended setup can use `--sentry true --sentry-dsn-file /private/path/to/dsn.txt
--sentry-environment wife`. The input file must contain exactly one HTTPS ingestion DSN;
keep it outside Git with mode600. Do not put the DSN on the command line or in shell history.
Use `./setup.sh --sentry false --package ... --workspace ...` to explicitly disable it.
Unattended setup without Sentry options preserves saved monitoring configuration.
Rerunning interactive setup can replace the DSN or environment; blank hidden DSN reuses
its saved value. Management auth tokens are neither accepted nor needed.

Configure the rebuilt server process with:

```json
{
  "SENTRY_DSN": "https://PUBLIC_KEY@INGEST_HOST/PROJECT_ID",
  "SENTRY_ENVIRONMENT": "production",
  "SENTRY_RELEASE": "inkscape-mcp-rust@YOUR_BUILD_VERSION",
  "SENTRY_TRACES_SAMPLE_RATE": "0.1"
}
```

These are process environment values, such as the `env` map in an MCP client
configuration. Defaults are `development`, a compiled version/revision/build release and 10% sampling.
Sampling accepts finite numbers from 0 through 1. Use a unique release per shipped
build. The DSN is a client ingestion identifier, not a management auth token.

Rebuild/package the modified source before using these values with `run-mcp.sh`:
that launcher runs the package selected by setup, not `rust/target/debug`.
Restart/reconnect the client after selecting a rebuilt package, preserving Inkscape windows.
`--version` emits JSON with version, source revision and a deterministic content/build-options
fingerprint (build ID, not a signing/provenance attestation). Git-free source archives use
SOURCE_REVISION; unknown origins are explicitly labeled unknown. Working-tree edits affect
the fingerprint. Error payloads and transactions carry fixed compiled `revision`/`build_id`
tags even when `SENTRY_RELEASE` is overridden. The default release includes all three values.
Package metadata captures the binary's actual identity, rather than inferring it from a tag.

## Privacy and diagnostics

Telemetry intentionally omits MCP arguments/results, SVG contents, document identifiers,
workspace paths and breadcrumbs. Panic messages are replaced with fixed text before
sending; full details remain on local stderr. Only known tool names are used as trace
names; unknown names are reduced to `unknown_tool`. Errors retain source basenames,
functions and line numbers. Error payloads use an allowlist: SDK contexts, tags, logentry,
threads, stack locals, source context and binary package paths are removed. Only fixed
failure category, OS and architecture tags are retained. Hostnames are not reported. SDK default PII collection is
disabled. Tool timing includes validation and error returns and does not currently
record success/error status or link a panic to its tool transaction.

## Stack traces and builds

Release builds now retain full debug information (`debug = true`, `strip = false`, `split-debuginfo = "packed"`),
so the Rust backtrace integration can resolve functions and line numbers locally.
On macOS, Cargo consolidates symbols into a matching `.dSYM` before deleting temporary
LTO object files. The package builder copies that bundle beside the executable so Rust
can resolve source lines without the development build directory.
The SDK `debug-images` feature is deliberately disabled for these unstripped binaries,
as recommended by Sentry. This increases executable/package size. Do not strip these
binaries after building without preparing matching debug artifact upload instead.
Source files and surrounding source context are not uploaded by this setup.

If distribution later requires stripped binaries, follow the platform procedure at
https://docs.sentry.io/platforms/rust/source-context/ to build and upload matching
native symbols before distribution. Never embed a Sentry management token in a client
package. Symbolicated file/function/line frames and uploaded source context are separate
features; this setup targets the former.

## Verification on 2026-10-03

A temporary panic through real server startup reached Sentry as
[INKSCAPE-MCP-SERVER-1](https://boryslav.sentry.io/issues/INKSCAPE-MCP-SERVER-1),
with readable Rust functions, file basenames and line numbers. The trigger was removed.
A read-only `get_workspace_info` call through the packaged release produced a confirmed
`mcp.tool` trace in Sentry. After a Codex reload, the actual connected Rust MCP server
answered the same tool successfully.

Rust formatting/Clippy and 218 tests passed (one ignored); package doctor, 17 dependency
notice checks and 11 launcher checks passed. The packaged macOS dSYM matches the executable UUID and resolves an application
address to `main.rs` with a source line. Relocated package acceptance also passed
CLI rendering/export, private runtime/bus, STDIO discovery and mutation invariants.
Codex was reloaded again and confirmed running the corrected package. These are automated and headless checks;
no native GUI acceptance was run for this change.

## Subsystem failure monitoring

Additional error boundaries cover private process spawn failure, process deadline,
signal termination, a typed uncertain live-operation result, and internal registry/live
lock or protocol serialization failure. The shell worker also reports spawn failures,
deadlines and observed signal exits. Categories are fixed (`process_start`,
`process_timeout`, `process_crash`, `live_uncertain`, `internal`) and group separately.
No dynamic error text, command line, stdout/stderr, SVG or tool arguments are uploaded.
Each category emits at most once per minute per server process, using fixed in-memory
storage; contention drops telemetry rather than delaying the operation. SDK transport
is asynchronous, with its bounded shutdown flush. No persistent upload queue is added.
Panic/fatal capture and the pre-existing sampled tool timing remain enabled as before.

A GUI/helper crash is visible here when it causes an observed IPC/process failure or
uncertain live result; this is not a native crash reporter inside the external Inkscape
application. A visually incorrect but successful edit still needs a manually captured
local diagnostic report. This extension does not launch GUI or change the tool contract.

Verification evidence: `migration/results/sentry-boundaries-review`. A real packaged
MCP render request used a synthetic headless executable that was signal-terminated twice,
then exceeded its deadline. The server returned bounded failures and stayed responsive;
Sentry delivery and category grouping are recorded alongside the raw STDIO trace.
Current source checks:220 tests pass,1 ignored; format and all-target Clippy pass.
Native GUI acceptance was not repeated for telemetry-only changes.

Confirmed events: [signal crash](https://boryslav.sentry.io/issues/INKSCAPE-MCP-SERVER-2)
and [deadline](https://boryslav.sentry.io/issues/INKSCAPE-MCP-SERVER-3), both with readable
`process.rs`/`main.rs`/`telemetry.rs` locations. Two crashes produced one event.
These are deliberate verification issues in environment `verification`; left unresolved
for inspection. Event details are retained locally. Sentry inferred geographic location
from the ingestion connection, despite the SDK not attaching user data; transport IP
is visible to the hosted service. The privacy policy above concerns the client payload.

Subsystem verification package: `/Users/bm/Documents/repos/inkscape-mcp-server/.inkscape-mcp-local/build.Jlhkjk/package`.
The checkout launcher now selects it; existing client Sentry environment settings are
preserved. Reconnect/reload the MCP client to replace an already-running server; this
must preserve the existing GUI. No client configuration was edited by this extension.

## Ready package with the setup wizard

Use `migration/results/packages/inkscape-mcp-macos-arm64-sentry-setup.tar.gz` on a
compatible Mac arm64 with Inkscape installed. Unpack it, run `./setup.sh`, enable
reporting, paste the project DSN at the hidden prompt and enter `wife` as the environment.
Then configure the MCP client to run that folder's absolute `run-mcp.sh` path. No DSN
is needed in the client configuration. The archive contains no saved local configuration.
Old stage38 archives do not contain the wizard or Sentry SDK; use this new package.
17 isolated TTY/privacy/refusal checks and real launcher/package acceptance are recorded
under `migration/results/sentry-setup-review`. No GUI actions are needed for these checks.

Current installed wizard package: `/Users/bm/Documents/repos/inkscape-mcp-server/.inkscape-mcp-local/build.KjfRBi/package`. The launcher now selects it.
