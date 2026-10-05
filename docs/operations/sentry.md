# Sentry monitoring

Compiled revision/build identity is implemented in current `main`. Package identity and
saved setup configuration determine the running build; merging source does not replace it.
Dated verification events/packages are [archived](../history/reports/sentry-through-pr9.md).

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
fingerprint (build ID, not a signing/provenance attestation). Git-free committed source archives use
SOURCE_REVISION; working-tree archives and unknown origins are explicitly labeled unknown.
Working-tree edits affect the fingerprint. Error payloads and transactions carry fixed compiled `revision`/`build_id`
tags even when `SENTRY_RELEASE` is overridden. The default release includes all three values.
Package metadata captures the binary's actual identity, rather than inferring it from a tag.

## Privacy and diagnostics

Telemetry intentionally omits MCP arguments/results, SVG contents, document identifiers,
workspace paths and breadcrumbs. Panic messages are replaced with fixed text before
sending; full details remain on local stderr. Only known tool names are used as trace
names; unknown names are reduced to `unknown_tool`. Errors retain source basenames,
functions and line numbers. Error payloads use an allowlist: SDK contexts, tags, logentry,
threads, stack locals, source context and binary package paths are removed. Only fixed
failure category, OS, architecture, compiled `revision` and `build_id` tags are retained.
Hostnames are not reported. SDK default PII collection is disabled. Tool timing includes validation and error returns and does not currently
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

## Validation scope

Use the Rust checks and `sentry-setup-acceptance` command in
[CONTRIBUTING](../../CONTRIBUTING.md). Synthetic telemetry failures and their real delivery
results remain scoped to the [recorded packages/events](../history/reports/sentry-through-pr9.md).
No historical installed-package path establishes today's selected runtime. This monitoring
does not provide a native crash reporter inside Inkscape or detect visually incorrect edits.
