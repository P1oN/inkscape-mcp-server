> Historical checkpoint. Counts, commands, package paths and “current” labels
> apply only to the recorded build. See the [history index](../README.md).

# Historical Rust migration report — stage38

This page preserves the named checkpoint and its evidence. Its package paths, counts,
validation and publication statements are historical, not the status of PR #8. For current
work use [the handoff](../../AGENT_HANDOFF.md), [the plan](../../RUST_NEXT_PLAN.md) and
[installation](../../install/install.md). Native results remain bound to the recorded binaries.

Recorded candidate: **stage38, macOS arm64**, tested on this Mac with official
Inkscape 1.4.3 and Rust 1.99. The MCP server is native Rust using pinned rmcp 3.5.0.
Full discovery has 110 tools, 7 prompts and 18 resources; 16 gate/profile/description
configurations match the frozen JSON contracts exactly. The rewritten Python MCP server,
entry points and paired comparison scripts were removed at the user's request.

The migration is **complete within the user-confirmed current Mac scope**. Current
headless/package checks, 122 fixed native checks, five two-window guard checks and
closed-session reconnect pass on the unchanged stage38 binary. Fresh headless and live
measurements are recorded below. Historical failures/results remain preserved separately.
At that checkpoint, no commits, PRs, release publication or user MCP configuration
changes had been made.

## Install and check the historical stage38 candidate

From the checkout, run `./setup.sh`, then `./run-mcp.sh`. Setup detects Inkscape, asks for
an existing SVG workspace and saves local configuration without manual env editing.
Source builds require Rust/native development tools and prepare a pinned private helper
runtime. The ready archive requires Inkscape; it does not require user-installed Python,
uv/pip, Homebrew, Rust or a compiler.

```sh
mkdir -p "$HOME/Applications/inkscape-mcp-stage38"
tar -xzf /Users/bm/Documents/repos/inkscape-mcp-server/migration/results/packages/inkscape-mcp-macos-arm64-stage38.tar.gz \
  -C "$HOME/Applications/inkscape-mcp-stage38"
cd "$HOME/Applications/inkscape-mcp-stage38/inkscape-mcp-macos-arm64-stage38"
./setup.sh
./run-mcp.sh --doctor
```

MCP client configuration (the launcher reads setup's saved configuration):

```json
{"mcpServers":{"inkscape":{"command":"/Users/bm/Applications/inkscape-mcp-stage38/inkscape-mcp-macos-arm64-stage38/run-mcp.sh"}}}
```

Archive: `/Users/bm/Documents/repos/inkscape-mcp-server/migration/results/packages/inkscape-mcp-macos-arm64-stage38.tar.gz`. Size: 46,748,658 bytes.
SHA-256: `94aab2652babf39c440a8a88ebd52efd70f036ea35f254a65dbe26307405bd6e`.
Binary SHA-256: `d6614579efa32d8912e8bc73c552cf0597dcd74e4e6b1ae1448c167415a4f311`.
Initial build index: [package-stage38-build-comparison.json](../../../migration/package-stage38-build-comparison.json).
Final acceptance overlay: [package-stage38-final-comparison.json](../../../migration/package-stage38-final-comparison.json).
The build-time index and archive remain unchanged; this external report records later acceptance.

Characteristic checks: open a synthetic SVG; inspect/find its objects; dry-run an edit;
apply an approved change and inspect its Operation Record; restore a snapshot; export PNG
and read its artifact URI. Confirm the original SVG remains unchanged. For live testing,
explicitly request launch of a separate managed synthetic drawing, connect/select it,
insert a named group and test native Undo/Redo. Startup/reconnect/doctor never request GUI
launch. Do not use user drawings for acceptance.

## Architecture and remaining runtime

Rust owns discovery, typed argument validation, SVG inspection/editing, transactions,
workspace/resource access, CLI rendering, IPC, live guards and audit records. XML uses
libxml2 with bounded safe parsing, encoding/mixed-content/namespace/reference tests.
The common edit pipeline keeps immutable originals and working copies, validates atomic
batches, journals recovery, snapshots changes, preserves true no-ops and requires existing
approvals. Semantic objects use ordinary named groups; layers organize the scene. Raster
tracing and arbitrary shell/code/extension/network execution are not exposed.

`runtime/helper_extension` retains fixed Python/inkex effect helpers;
`runtime/insert_payload.py` and `runtime/edit_errors.py` support their private protocol.
`rust/src/bin/inkscape-mcp-supervisor.rs` supervises an explicitly launched owned GUI/bus. These are not
another MCP server. The archive carries private CPython 3.12.14 and six pinned wheels.
`runtime/native/context.m` supplies the Objective-C/GTK context bridge; prebuilt bridge,
D-Bus/gdbus and their library closure are included. The private effect transaction
preserves Inkscape native Undo; writing SVG to disk is not a replacement.
[Helper investigation](RUST_HELPER_INVESTIGATION.md) records 20 actual packaged-helper
observations and the limits of a future Rust helper. Full helper rewrite is not required.

## Current validation and compatibility

| Requirement | Actual evidence | Scope/limit |
|---|---|---|
| Rust checks |215 passed / 1 ignored, fmt, all-target clippy |Local source; see export-name-review logs |
| Retained helper/tooling |6 helper regressions; current Ruff lint/format |Helper tests are automated, not GUI acceptance |
| MCP surface |16 exact configurations;110/7/18; generated llms unchanged |Frozen JSON, no executable Python oracle |
| Ready archive |Actual cold/per_call and warm/shell installs, empty PATH |Temporary local install; clean-machine check user-owned |
| Core edit behavior |Both archive runs: approval, atomic batch/rollback, snapshot/restore, original preservation, byte/mtime no-op |Selected synthetic fixtures plus Rust regression suite |
| Render/export/resources |Actual CLI pixels, export artifact readback, static/document resources and prompts |Headless; no GUI inference |
| XML/filesystem |35security and12special-file refusals; descriptor/race/recovery tests |POSIX; finite fixtures, not exhaustive adversarial proof |
| Linked assets |17 asset cases and8 engine-route cases on stage38 |Protected bounded SVG/raster/CSS/import staging; both engines tested on current stage38 |
| Diagnostics/publication |3 large-input refusals; compare pair refusal leaves inventory unchanged and successful PNGs read back |Pair publication is not crash-atomic; cleanup can require recovery |
| Output limits |5 selected input/resource/render byte and pixel checks |Raw data limits, not every JSON output; output-byte failure may create empty planned directories |
| Repeated export |30 successful exports over 10 runs; unique names, exact resource pixels, prior bytes preserved |Automatic names now include a full UUID; originals remain untouched |
| STDIO bound |Configured4096 cap refuses unfinished5KB frame without EOF; ordinary requests pass |Request cap, not a whole-process RSS cap |
| Doctor/launcher |10 doctor profiles,11 launcher checks |Read-only; GUI not launched by these checks |
| Notices/provenance |102 crate inventories,17 notice checks plus explicit Homebrew BSD2 binding;2533 FILES entries verified |Source/notice attribution, not legal certification or binary reproducibility |
| Native GUI |Actual38 retry:122 fixed checks,5 two-window guard checks and closed-session reconnect |Owned synthetic documents; bounded scenarios, not all live operations/failures |

`INKSCAPE_MCP_MAX_INPUT_BYTES` defaults to50 MiB and `MAX_OUTPUT_BYTES` to100 MiB for
bounded file/artifact data. Base64/JSON adds wire overhead. STDIO request lines, including
CR/newline, default to six times input cap plus 1 MiB; a positive
`INKSCAPE_MCP_MAX_REQUEST_BYTES` overrides this. Overlong input closes the MCP connection
before deserialization. These limits do not certify global peak memory or every error
serialization cost. Unsupported engine dependencies (PI/xml:base, unsafe URI/functions,
non-UTF-8 CSS) refuse explicitly before publication; originals remain preserved.

## Provenance and native source kit

Crate archives match Cargo.lock; standard-library notices are copied from the selected
compiler. rmcp's missing archive notice is supplied from its exact recorded upstream commit.
The CPython executable and 946 regular runtime members match the SHA-verified Astral release;
libpython install-name and all sysconfig changes are separately reproduced, not broadly
ignored. 19 distribution licenses and PYTHON.json are included. Six exact PyPI wheels have
1259 matching members; original RECORD rows remain, with verified uv metadata and omitted
NumPy CLI entries. Wheel/license metadata remains in the package.

Native GLib/D-Bus/gettext/PCRE2 source archives match installed SBOM URL/version/hash.
GLib's installed recipe matches its exact cached bottle; its sole historical patch applies
to the exact source. Recipe/patch and Homebrew BSD2 notice are included. Source kit:
`migration/results/homebrew-source-audit/native-source-kit-stage37.tar.gz`;
hash/file index:[source-kit-stage37.json](../../../migration/results/homebrew-source-audit/source-kit-stage37.json).
It includes four upstream archives, recipes/receipts/SBOMs, patch and the recipe's
introspection resource. Libraries remain separate replaceable dylibs. Source/build inputs
are supplied; a reproduced whole binary and full redistribution clearance are not claimed.
`LICENSE-INVENTORY.json` retains explicit broader/foreign-target gaps.

## Measurements and supported scope

Historical stage29 on stage27: five alternating pairs, 80 requests, matching 40 scene/
selection/PNG observations and zero sampler errors. Python/Rust startup medians were
506.854/6.882 ms; MCP RSS 102.17/26.70 MiB. GUI/bus RSS was 282.09 MiB for both and includes
shared pages. Connect 1108.579/1106.663 ms, scene 201.468/206.803 ms, render 106.122/125.651 ms.
These prove selected historical startup/RSS improvements, not a general live/mutation
speedup or current stage38 performance. Raw/verified evidence is in
`migration/live-process-benchmark-stage29-verified.json`.

[Phase measurements](RUST_PHASE_MEASUREMENTS.md) distinguish handler, logical-file and
Inkscape process envelopes from mixed transport/serialization/scheduling remainder.
Those historical diagnostic binaries are not the production server; timing refinement
is secondary by user decision. Fresh current-candidate headless measurements follow.

Current stage38 Rust-only benchmark: five alternating runs of each engine, 250 measured
requests/startups. SVG fixtures contain 100/3000/15000 rectangles (not realistic-illustration
coverage). Command:

```sh
.venv/bin/python scripts/history/python/rust_current_measurements.py \
  --binary migration/results/packages/inkscape-mcp-macos-arm64-stage38/bin/inkscape-mcp \
  --output migration/results/rust-measurements-stage38-repeat --repeats 5
```

The output path must be new. Use a development environment with Pillow. Actual raw commands,
isolated HOME/PATH/settings, timing samples, RSS samples and wire traces are preserved in
[migration/results/rust-measurements-stage38](../../../migration/results/rust-measurements-stage38).
Roundtrip medians in milliseconds:

| Operation | per_call | shell | Samples per engine |
|---|---:|---:|---:|
| Startup to initialize response |6.984|6.604|5|
| Open 100 / 15000 rectangles |19.259 /27.530|17.485 /27.168|5 per size|
| Inspect 100 / 15000 |3.365 /302.264|3.139 /306.055|5 per size|
| Single edit 100 / 15000 |721.401 /1129.146|454.415 /907.005|5 per size|
| Atomic batch of 8, 100 / 15000 |583.652 /1127.915|75.096 /663.648|5 per size|
| First standalone render |269.546|269.815|5|
| Reused render |266.096|10.379|10|
| Export |264.946|11.528|15|

Edits include snapshots, durable files/audit and automatic before/after Inkscape previews;
these are not pure DOM timings. First standalone render follows earlier edits, so it is
not a cold engine start. Sampling ps every 50 ms plus sampler runtime observed maximum server
RSS 263.02/263.55 MiB and child sums 270.25/494.31 MiB. First post-initialize samples were about
12 MiB; after the full workload, idle server medians were 257.17/254.34 MiB. Shared pages count
in each process RSS; short peaks may be missed. Shell retained two headless workers after
this workload. These absolute results demonstrate selected engine reuse savings; they do
not establish a new Python comparison, general speedup or peak-memory guarantee.

The first stage37 measurement attempt reproduced same-second export-name collisions. Current stage38
adds a full UUID to automatic export names; 30 repeated exports pass and earlier artifact
bytes remain intact. Explicit safe publication/no-overwrite checks remain in place.

Only current macOS arm64 has local package execution evidence. Four POSIX native-target
CI jobs are prepared; they were not remotely run. Windows is backlog. Foreign ABI/older
OS compatibility is unverified. Local ad-hoc signing is used; DeveloperID/notarization
is not available and no security setting was bypassed.

## Current native acceptance

With the user present and an active display, the unchanged stage38 package passed:
- 46 insertion/style/repeat/duplicate/approval/reconnect checks;
- 38 group/ungroup/delete checks;
- 22 compound transform checks;
- 16 lower/paint-order checks.

Each changed effect was tested with native Edit-menu Undo/Redo, captured SVG trees, PNG
RGBA and immediate audit records. Repeated style adds no extra Undo step. Transform matrix
comparison uses 5e-6 tolerance for Inkscape's six-digit serialization; drawing comparison
excludes namedview only. Lower used two identical blue copies: XML order and preserved
pixels were checked, not differing-color occlusion. No disappearing-group recurrence.
Reports: `migration/native-{,structure-,transform-,order-}stage38-retry1-comparison.json`.

Five additional checks used continuous Rust STDIO with two owned synthetic windows:
switching the active document refused mutation, identities differed, the second drawing
was blank, the first tree/RGBA stayed exact and explicit rebinding returned ready.
See `migration/results/native-window-guard-stage38-retry1/comparison.json`.

Fresh live measurements used five Rust reconnects and 15 identical selection/tree/RGBA
observations on the owned drawing. Mixed roundtrip medians:

| Operation | Median ms | Samples |
|---|---:|---:|
| Startup to initialize response |23.670|5|
| live_connect |1306.051|5|
| live_get_selection |51.592|15|
| live_get_scene |270.203|15|
| live_render_view |155.734|15|

Point RSS medians: server18.17 MiB; GUI/bus/supervisor sum272.25 MiB (shared pages counted,
not peaks). Startup used an existing persisted workspace and retention, unlike the empty
headless roots. Native mutation traces retain individual timings, not a repeated mutation
benchmark. These are absolute current measurements, with no new Python comparison or
pure IPC/general speedup claim. Commands/environment/raw/wire/PNGs are retained in
`migration/results/live-measurements-stage38-retry1`; development harness:
`scripts/history/python/rust_current_live_measurements.py` (requires an explicitly owned live session).

The original stage36/38 no-primary-monitor/CVDisplayLink failures remain historical;
no bridge guard was bypassed and no server change was needed for this retry. The first
keyboard-shortcut attempt did not invoke GTK Undo; failed captures are retained under
`native-stage38-retry1/failed-ui-shortcuts`, followed by explicit menu acceptance.
Before closure, 298 workspace files and final session logs were copied and hashed. Owned
GUI15336/supervisor15332 are terminal and the manifest is removed. Real reconnect refused
without recreating the manifest or launching GUI. After closure, CUA getAXState incidentally
reopened the private application copy as unmanaged PID21952 with a welcome/version window;
it was left untouched. This UI observation behavior is separate from MCP startup/reconnect.
See `migration/results/native-stage38-retry1/closure.json`; do not inspect a cached CUA
application after closing it without checking process state first.

## Backlog and user decisions

The final scoped audit is `migration/requirement-audit-stage38-final.json`; no required
current-Mac migration gate remains. Finite fixtures do not certify all SVG/live/error cases.
Historical disappearing-group cause is deferred unless it recurs (one recorded incident,
not the separate stage18 export/crash). Clean-machine installation will be checked by the
user later; Windows is deferred. No credentials or signing approval is required to continue
local independent work. [Current requirement checklist](RUST_COMPLETION_CHECKLIST.md) and
[handoff](../../AGENT_HANDOFF.md) record the completed scope and backlog.

All prior checkpoints, failures, raw results and comparison details are preserved in
[historical migration log](../RUST_MIGRATION_LOG.md) and hash-bound result snapshots.
Old Python/parity reproduction commands there are retired and should not be rerun.
