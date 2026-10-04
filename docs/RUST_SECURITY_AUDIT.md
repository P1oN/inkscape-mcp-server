# Historical scoped security review — stage38

This page preserves the named checkpoint and its evidence. Its package paths, counts,
validation and publication statements are historical, not the status of PR #8. For current
work use [the handoff](AGENT_HANDOFF.md), [the plan](RUST_NEXT_PLAN.md) and
[installation](install/install.md). Native results remain bound to the recorded binaries.

Candidate **stage38**, source Rust 215 passed / 1 ignored, fmt and clippy pass.
Actual package evidence is indexed by
[package-stage38-build-comparison.json](../migration/package-stage38-build-comparison.json).
This is a finite regression review, not exhaustive adversarial or whole-process memory proof.

| Boundary | Current evidence | Limit |
|---|---|---|
| XML and typed arguments | 35 package security checks, safe-parser Rust regressions | Selected malformed/namespace/depth/number/container cases |
| Files and resources | 12 special-file refusals; POSIX descriptor/no-follow/race/recovery tests; static/document/artifact resource reads | Windows reparse/handles remain backlog |
| Edit transaction | Both installed engines: originals, approval, atomic batch rollback, snapshots/restore, byte/mtime no-op | Finite fixtures; filesystem crash guarantees remain scoped |
| Engine dependencies | 17 linked-asset and 8 route checks on actual38, both engines | Selected routes and dependency forms |
| Error serialization | Three oversized invalid inputs yield responses below 4096 bytes, no writes | Specific validation errors, not all possible diagnostic output |
| Pair publication | Unsafe second source leaves inventory unchanged; successful blue/red PNG resource readback; rollback unit fixtures | Two file publications are not crash-atomic; recovery may be needed |
| Output caps |5 real input/artifact/render byte and pixel boundary cases |Not every JSON output or memory allocation; byte refusal may create empty directories |
| STDIO ingress | Configured 4096-byte cap rejects unfinished 5KB input without EOF | Configurable per-line limit, not a global RSS limit |
| Live guards | Actual38:122 native checks +5 two-window guards +closed-session reconnect | Owned synthetic documents; selected operations/failure paths |

## Confirmed defects fixed in the current package

- Namespace collision and rejected XML open could write working files: common preflight
  was repaired, with refusal inventories retained.
- CLI rendering could read outside-workspace linked raster assets. Bounded owned staging
  now resolves dependencies through the protected workspace, including supported SVG/CSS
  chains. Engine mutations restore original references; export removes private paths.
- capture_frame created output directories before unsafe dependency refusal: preparation
  now precedes destination planning.
- Large invalid arguments inflated diagnostic responses and intermediate strings:
  streaming excerpts bound key/tag/input representations while retaining ordinary errors.
- compare_region published the first PNG before discovering an unsafe second source:
  both sources render privately before publication, with guarded cleanup on write failure.
- Same-second automatic export names collided: full UUID suffixes now preserve repeated
  exports and earlier artifact bytes;30 real exports pass.
- SDK read_until accumulated unlimited unfinished STDIO frames: the AsyncRead wrapper
  bounds the line before accumulation/deserialization and closes oversized connections.

File input/output defaults are 50/100 MiB. Request-line default is 6 times input cap plus
1 MiB, overridable by positive INKSCAPE_MCP_MAX_REQUEST_BYTES. Base64/JSON overhead is
additional to artifact bytes. Dependency limits share the input byte budget, depth 8,
count 128, CSS block depth 32. Unsafe URI/symlink escapes, DTD/entities, script/foreignObject,
PI/xml:base and unsupported dependency syntax refuse before publication. The MCP surface
has no arbitrary execution, network fetching or bitmap tracing.

## Remaining acceptance and known scope

The unchanged38 retry passed current native Undo/Redo, binding/approval guards and reconnect.
Continuous STDIO two-window switching refused mutation and preserved the first tree/RGBA.
After the owned GUI closed, reconnect refused without launching it. Fresh current live
measurements retain15 stable observations. Original no-monitor failures remain historical;
no guard was bypassed. CUA's post-closure state read reopened a private welcome window;
it was left untouched (separate from MCP reconnect). These fixtures do not exhaust live
failure/timeout behavior. Historical disappearing-group investigation is deferred unless
recurrence. See [final checklist](RUST_COMPLETION_CHECKLIST.md).

Crate/runtime/wheel/native attribution checks and source kit are recorded in
[packaging](RUST_PACKAGING.md). Their limits are explicit; no credentials approval is needed.
Earlier review, failures and raw evidence remain in
[historical security log](history/RUST_SECURITY_AUDIT_LOG.md), with old checkpoints frozen.
