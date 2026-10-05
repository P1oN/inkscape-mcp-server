> Historical checkpoint. Counts, commands, package paths and “current” labels
> apply only to the recorded build. See the [history index](../README.md).

# Rust helper migration investigation

Historical investigation checkpoint: 2026-10-02, stage13. The current stage38 package
still retains the Python/inkex helper; current native acceptance is recorded in
RUST_MIGRATION_REPORT.md. The historical stage13 package retains the fixed Python/inkex
one-shot helper. This investigation does not replace it or claim native Undo for a
future Rust executable.

The [official INX documentation](https://inkscape.gitlab.io/extensions/documentation/authors/inx-overview.html)
defines effect extensions as external programs consuming SVG and returning changed
SVG on stdout. The [script-extension guide](https://wiki.inkscape.org/wiki/Script_extensions)
also describes GNU-style parameters and selection IDs. A compiled Rust filter is
therefore a feasible candidate; this is an inference about architecture, not completed
integration or an ABI certificate. The installed Inkscape 1.4.3 inkex implementation is
hashed in the probe report; current online documentation may describe newer versions.

`migration/helper-protocol-stage13-comparison.json` records **20/20 offline checks**,
executing the actual packaged private CPython, fixed helper and installed vendor inkex.
Ten synthetic cases run through both a positional SVG file and stdin: style/text changes
and exact no-ops, selection/ID/fingerprint mismatch, duplication, insertion and unsafe
insertion refusal. Each observation retains exact command/environment, input/request,
stdout/stderr, nonce result, exit code, SHA-256 and elapsed time. Inputs and requests stay
unchanged; result publication is private mode 0600 without an abandoned temporary file.
No Inkscape GUI, bus effect dispatch or real document was used.

The measured boundary matters:

- Exact style/text no-ops emit **zero stdout bytes**, exit zero and publish successful
  nonce results. Vendor `SvgThroughMixin.has_changed` compares serialized complete
  trees. Returning an unchanged reserialized SVG instead could add an empty native
  Undo step. Keep document-level comments/processing instructions for no-op detection.
- Refused edits also emit no SVG and exit zero, with the helper's structured refusal
  side channel. This avoids the native blocking error dialog used for insertion refusal.
- Unsafe insertion emits no SVG, publishes an unsuccessful nonce result and exits
  **251** through `AbortExtension`. A replacement must deliberately preserve or explicitly
  validate any change to that failure boundary.
- Actual changes emit SVG, preserving existing IDs. Inserted IDs are nonce-bound and
  local references must use the same remapping. The server checks exact post-effect
  identities/fingerprint and reports uncertainty without retry if confirmation is lost.
- The public operation completes through Inkscape's one-shot effect transaction; writing
  the user's live SVG directly to disk would not implement this boundary or its Undo.

Native server code already implements insertion preflight/fingerprinting and guarded
request/result IPC in `rust/src/live_insert.rs` and `rust/src/live_managed.rs`. It does
not yet implement the helper's full copied-tree edit planner. The latter depends on
inkex style/cascade handling, parent affine transforms, SVG class semantics, reference
checks and native extension input selection. Reuse the bounded XML/identity kernels,
but verify serialization, stylesheet/refusal behavior and transformations against the
helper before substituting it. A whole-document namespace serializer can change
fingerprints or no-op detection even when geometry looks unchanged.

A next implementation should be a private, fixed executable with a strict allowlisted
extension-argument parser (`--id`, `--selected-nodes`, optional input/output), bounded
stdin/file input, and descriptor-protected request/result files. Preserve the existing
session lock and context/fingerprint checks. Reject unknown options and external SVG
execution. Do not expose arbitrary executable paths or parameters through MCP. Use a
separate owned acceptance profile, retain the current package, and verify one-step
Undo/Redo, no-op, refusal, stale selection, context switch and lost-result handling in
native UI before switching the shipped INX command. The probe accepts vendor file/stdin
forms; it does not capture every argument Inkscape actually supplies during native dispatch.

Replacing this helper alone would **not** remove all Python from the package: the fixed
managed supervisor remains Python, and the optional socket helper is also Python. The
Objective-C context bridge and private D-Bus/GLib remain separate native dependencies.
No package assets, INX descriptors or installed GUI profile were changed by this investigation.

Reproduce in a new output directory:

```sh
.venv/bin/python scripts/history/python/migration_helper_protocol_probe.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage13 \
  --output migration/results/helper-protocol-stage13-repeat \
  --report migration/helper-protocol-stage13-repeat-comparison.json
```

## Stage19 preservation check

The helper still uses Python/inkex. Changed edits now retain the document root,
prolog/epilog and DOCTYPE; child removal uses inkex callbacks to release existing IDs.
See the stage19 section in [the migration report](RUST_MIGRATION_REPORT.md) for
20 actual packaged-helper observations, the preserved failed all-epilog probe and
46+38 scoped native checks. This does not implement a Rust helper or resolve vendor
inkex no-op detection for multiple epilog siblings.
