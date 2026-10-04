Historical phase measurements. Current status and build evidence are recorded in
[AGENT_HANDOFF.md](AGENT_HANDOFF.md). Python/parity profiler scripts were retired by user decision;
old reproduction commands below describe retained experiments, not current workflows.
No historical diagnostic timings are automatically transferred to the current binary.

# Rust diagnostic phase measurements

Checkpoint 2026-10-03. These measurements advance phase attribution but do not complete
the migration or prove pure IPC/kernel IO timings. The distributable candidate remains
stage23. Its source, executable and archive were not modified by this experiment.

`scripts/migration_prepare_phase_profile.py` copies the current Rust sources into a new
private directory and inserts a diagnostic module from `scripts/migration_phase_profile.rs`.
No profiler is linked into the normal server. The copy instruments synchronous headless
`call_tool`, 19 named logical-file API functions and bounded child-process execution.
It refuses nested/concurrent handler attribution and thread migration. Live, async wait,
warm shell and Python handler phases are outside this diagnostic's scope.

The first five-pair run wrote nested spans to stderr immediately. Inspection showed this
put diagnostic output inside enclosing file spans. That experiment and exact sources are
preserved in `migration/results/phase-profile-stage24/`; its timings are not the corrected
phase result. The second copy accumulates at most 512 nested events per request and writes
one batch after the handler's end timestamp. The profiler still has clock/collection overhead;
batch serialization/output is part of the outside-handler remainder.

The corrected five alternating Python/Rust pairs made 160 real STDIO requests. All 50 Rust
tool handlers have matched request order/name, same-thread nested events, bounded intervals
and exact partitions. Full inspection envelopes for the one-object and 100-object fixtures
match Python after only request-bound document IDs and JSON text decoding. Sampler errors: 0.
The diagnostic binary uses an unchanged copy of stage23 helper/native dependencies and real
official Inkscape CLI. Its synthetic SVG originals remain unchanged. No GUI was launched.

| Corrected diagnostic median, ms | Handler remainder | Logical-file API | Inkscape process envelope | Outside handler, mixed |
|---|---:|---:|---:|---:|
| Open small | 0.095 | 18.293 | 0 | 7.286 |
| Inspect small | 0.511 | 0.557 | 0 | 6.111 |
| Single edit with preview | 1.528 | 65.512 | 649.430 | 19.720 |
| Atomic batch with preview | 1.475 | 63.808 | 504.697 | 20.484 |
| Render | 0.511 | 8.612 | 252.987 | 5.729 |
| Export | 0.545 | 7.912 | 248.929 | 6.314 |
| Save | 0.310 | 19.890 | 0 | 12.166 |

These are medians of individual components; independently computed medians need not sum.
The raw per-request partitions do sum exactly. `migration_verify_phase_profile.py` unions
nested intervals rather than adding them. Overlapping category priority is Inkscape process,
other process, then logical files. It refuses out-of-bounds, reversed, unknown and cross-thread
events; regression cases also cover nested file intervals and category overlap.

Logical-file intervals include secure-path validation, byte copying, fsync and publication,
not just kernel IO. Unguarded direct filesystem calls and hash computation remain in the
handler remainder. The Inkscape envelope includes spawn, polling, pipe drain and reaping;
the earlier fixed vendor wrapper separately measures vendor CLI wall time. The outside-handler
remainder includes client JSON, protocol serialization, pipes, scheduling and diagnostic output.
It must not be labeled pure IPC. The diagnostic is not a production speed comparison.

Reproduction (use a fresh output directory; never overwrite prior evidence):

```sh
.venv/bin/python scripts/migration_prepare_phase_profile.py --output .inkscape-mcp-local/new-phase-copy
LIBXML2="$(xcrun --show-sdk-path)/usr/lib/libxml2.tbd" cargo build --locked --release \
  --manifest-path .inkscape-mcp-local/new-phase-copy/rust/Cargo.toml \
  --target-dir .inkscape-mcp-local/new-phase-copy/target
```

For the recorded run, a private diagnostic package copy replaced only the Rust executable,
removed its now-stale FILES index and retained the original stage23 package separately.
This copy is not an installable artifact. `migration_process_benchmark.py --repeats 5 --counts 100`
then ran against it; `migration_verify_phase_profile.py` validated the Rust spans and
`migration_verify_large_response.py` compared the complete recorded inspections.
The exact invocations, source copies, hashes, raw responses and spans are retained in the
stage24 result directories and evidence binding.

## Stage25 Python/Rust paired phase follow-up

The diagnostic Python entry point is `scripts/migration_python_phase_profile.py`.
It runs the unchanged reference server, wraps 13 fixed Path APIs and three workspace path
resolvers, and wraps the existing `subprocess.run` in that process only. The original
implementations, arguments and protections remain in use. ContextVars carry request identity
into FastMCP worker threads; event collection is bounded at 512 nested events. Parameters,
SVG paths, environment values and child output are never logged. One event batch is emitted
after the middleware end timestamp. This entry point is not the installed default server.

Five alternating pairs with that entry point and the same instrumented Rust v2 binary made
160 real STDIO requests. All 50 Python middleware requests and 50 Rust handlers bind to
actual wire calls and have exact interval-union partitions. The one-object/100-object full
inspection envelopes match and sampler errors are zero. Regression checks retain strict
cross-thread refusal for Rust and allow Python's explicitly context-bound workers; a foreign
request ID, invalid/reversed bounds or unknown category still refuses attribution.

| Diagnostic median, ms | Python handler remainder | Rust handler remainder | Python logical files | Rust logical files | Python Inkscape envelope | Rust Inkscape envelope |
|---|---:|---:|---:|---:|---:|---:|
| Single edit with preview | 6.125 | 1.787 | 1.041 | 64.510 | 691.375 | 703.636 |
| Atomic batch with preview | 4.771 | 1.801 | 1.171 | 65.217 | 523.256 | 550.404 |
| Render | 2.293 | 0.702 | 0.289 | 8.602 | 268.820 | 269.568 |
| Export | 2.555 | 0.625 | 0.394 | 9.021 | 265.016 | 269.647 |
| Save | 1.297 | 0.320 | 0.644 | 21.522 | 0 | 0 |

These are instrumented observations, not production speed comparisons. Python middleware
and Rust SDK handler boundaries differ; Python Path/resolver and Rust secure descriptor API
regions also differ. Python direct IO through other APIs remains in its handler remainder.
Do not treat the categories as identical pure kernel-IO scopes or the remainder as pure IPC.
Outside-handler medians still include diagnostic batch output and framework/client work.
All per-row partitions are preserved; the component medians need not sum.

The larger Rust logical-file region motivates a focused synchronization measurement.
Current Rust writes explicitly call file `sync_all`, and atomic publication also synchronizes
the parent directory. Those calls are an investigation target, not a proven sole cause; this
experiment neither measures their individual latency nor justifies weakening durability or
path protections. The next diagnostic should time them separately before any optimization.

Actual raw directory: `migration/results/phase-profile-stage25/`. Reports:
`migration/phase-profile-python-stage25-comparison.json`,
`migration/phase-profile-rust-stage25-comparison.json`,
`migration/phase-profile-stage25-envelope-comparison.json`. The benchmark's optional
`--python-entrypoint` selects the explicit diagnostic script; default invocations retain the
ordinary Python entry point. No production source or stage23 ready package changed.

Remaining work: lower transport boundaries for IPC and protocol/client serialization,
coverage of unguarded filesystem calls, focused sync latency, applicable warm/live paths
and final candidate attribution. Native stage23 acceptance still requires an available
primary monitor; this experiment does not establish a fix for earlier native history failures.


## Stage26 synchronization spans and no-op finding

A new isolated copy instruments the three existing file `sync_all` sites and two directory
`sync_all` sites individually, while still calling the exact original methods. Five alternating
Python/Rust pairs make 160 requests; both 50-handler partitions and complete inspection
envelopes pass, with zero sampler errors. The interval union excludes sync spans from their
parent logical-file region, preserving an exact per-request total.

| Diagnostic Rust median, ms | File sync | Directory sync | Remaining logical-file API |
|---|---:|---:|---:|
| Single edit with preview | 41.173 | 18.532 | 4.281 |
| Atomic batch with preview | 41.024 | 18.312 | 4.431 |
| Render | 7.658 | 0 | 0.870 |
| Export | 7.716 | 0 | 1.029 |
| Save | 11.365 | 6.838 | 1.831 |
| No-op, before source fix | 3.905 | 3.857 | 0.533 |

These timings apply to the pre-fix diagnostic source, not the subsequently fixed executable.
Independent medians need not sum. The no-op sync observations revealed a transient proposed
Operation Record: the pipeline wrote it before DOM staging, then removed it when serialized
SVG did not change. Previous final-tree/history checks did not observe that transient write.
A deterministic regression fixes the owned operations directory timestamp to Unix second 1
and asserts that a real no-op does not change it. It failed before the fix and passes afterward.

The normal source now exposes an internal `apply_dom` path for 22 explicitly reviewed
memory-only DOM families. It validates the operations directory through anchored descriptors,
stages in memory and returns a genuine no-op before creating any new audit file. For changed
SVG, durable intent still precedes snapshot, preview, recovery preparation and working-copy
publication. Failed DOM staging retains discarded audit handling. The existing `apply` path
remains for CLI-staging actions, paths and fit, preserving their pre-dispatch audit refusal.
No fsync, policy gate, original-file guard or publication/rollback guarantee was removed.

Fresh normal release validation compares the complete synthetic workspace tree, bytes,
file sizes and nanosecond mtimes before/after a true MCP no-op without normalization.
The before/after inventories match. The Rust suite passes 198 tests with 1 ignored;
clippy passes with warnings denied. An initial edit harness invocation used an existing,
unbound debug executable and is explicitly excluded from fix validation. The subsequent
fresh release edit comparison is recorded separately. The stage23 ready archive still has
its historical pre-fix binary; it is not silently relabeled as fixed. A new final package and
its affected validation remain pending. Native/live and pure IPC gates are still open.

### Stage27 current production measurements and prompt review (2026-10-03)

Five alternating Python/Rust pairs each for normal per-call, normal warm-shell and diagnostic
CLI capture complete 520 real STDIO requests. All80 full inspection envelopes agree after
only request-bound document IDs/JSON-text decoding; zero sampler errors. The production
binary SHA is the stage27 package's 62a75763d517af691c0cc1c9026d8963dc9bb33394d9dfdd76aa8fa9d5819f18;
no diagnostic handler profiler is linked. Native GUI was not exercised.

| Normal per-call median | Python | Rust |
|---|---:|---:|
| Startup to initialize |926.689ms|21.454ms|
| Inspect100k objects |16176.299ms|2107.127ms|
| No-op |3.179ms|1.203ms|
| Single edit |760.856ms|797.639ms|
| Atomic batch |562.117ms|628.107ms|
| Render |278.705ms|289.001ms|
| Export |276.896ms|291.465ms|
| Inspect-small sampled tree RSS |102.38MiB|18.73MiB|
| Inspect100k sampled tree RSS |1160.02MiB|1199.12MiB|

Normal shell100-object fixture medians Python/Rust: startup702.065/6.605ms,
single-edit432.459/480.899ms (first shell render starts the engine), batch8.281/70.904ms,
render4.529/12.325ms, export5.532/12.730ms. These are distinct engine modes on a warm local
host, not cold-OS or GUI performance. No overall edit/export speedup or large-output memory
reduction is supported. Per-call sampler median gaps25.03/24.64ms, maxima349.29/261.85ms;
zero errors does not establish exact peak memory or coverage of every short-lived child.

Diagnostic CLI vendor render medians263.880/271.891ms and export266.673/267.018ms;
outside-CLI remainders include server/files/IPC/wrapper bootstrap. Pure IPC and complete
exclusive phase attribution remain open. This current evidence supersedes historical
stage23 speed claims only for its named stage27 fixtures; historical reports remain untouched.

Reports: migration/process-benchmark-{stage27,warm-stage27,cli-stage27}-comparison.json,
migration/process-benchmark-stage27-followup.json. Raw runs, environment/commands, harness and
verifier snapshots are under the corresponding migration/results/process-benchmark-* dirs.

Prompt review found the harness used nonexistent INKSCAPE_MCP_ENABLE_LIVE. It now uses the
actual LIVE_ENABLED flag and asserts visible live tools for full profile; core profile has no
live tools. Current actual package passes100 exact prompt/resource-index scenarios in all
four profile/live combinations. Prior prompt runs did not establish live-flag switching.
Report: migration/prompt-stage27-comparison.json; exact fixed harness and raw replies retained
in migration/results/prompt-stage27. No normal server or package source changed this cycle.


## Stage28 transport-boundary diagnostic

The isolated source preparer now optionally instruments STDIO with --transport-spans.
It wraps the existing Tokio read/write/flush APIs, retains at most1024 frame metadata
records and8MiB per frame, and dumps after service shutdown. No request parameters, SVG
contents, paths or environment values are logged by this collector. The module is absent
from normal rust/src and the distributable stage27 binary. It is not a new MCP capability.

A fresh normal-source copy with file/sync/process spans and transport wrappers completes
five alternating pairs/160STDIO requests on100-object fixtures. All50 Rust tool handlers
bind to exact JSON-RPC IDs and incoming frame byte counts; all50 child interval partitions
and nested file/process partitions validate. Twenty complete inspection envelopes match
Python, sampler errors0; diagnostic release clippy-Dwarnings passes. This is diagnostic
boundary evidence, not production speed evidence or native GUI acceptance.

Child clocks use CLOCK_MONOTONIC; Python perf_counter has a different epoch on this Mac.
Only durations cross that boundary. Incoming frame observation <= handler <= first write
attempt <= final write acceptance <= successful flush is validated within child timestamps.
Read-to-first-write + the residual sum to client roundtrip. Write/flush duration is retained
separately and overlaps client IO; it must never be added as an exclusive phase. Initial
capture/validator subtracted a complete flush envelope and correctly refused negative
residuals. Logs/failed validator preserved in transport-profile-stage28; corrected v2 uses
first attempt before poll_write (including a pending attempt), with a single read stamp.
The corrected data includes flush completion0.036459ms beyond a client roundtrip, confirming
the overlap is observable. No clock offset or negative sample is silently normalized away.

Diagnostic medians (ms, Rust):

| Operation | Child read to first write | Residual outside that interval | Write/flush, overlapping |
|---|---:|---:|---:|
| Inspect100 |8.570|0.380|0.385|
| No-op |2.924|0.065|0.028|
| Single edit |813.827|0.110|0.031|
| Atomic batch |666.370|0.105|0.028|
| Render |304.542|0.099|0.029|
| Export |307.993|0.124|0.031|

These independent medians need not sum. Raw rows contain exact additive child intervals.
Read stamps observe bytes returned by Tokio, not kernel arrival. JSON decoding in the
collector and handler diagnostic stderr dumps add overhead. The residual includes client
JSON/parsing, pipes and scheduling, rather than pure IPC. Complete kernel/file and exclusive
IPC attribution, Python frame counterparts, warm/live diagnostic boundaries remain pending.

Reports: migration/transport-profile-stage28-v2-comparison.json,
phase-profile-stage28-v2-comparison.json and transport-profile-stage28-v2-envelope-comparison.json.
Raw initial/v2 results, exact original/instrumented Rust copies, commands/build/clippy logs
and used harnesses are bound in migration/transport-profile-stage28-evidence-binding.json.


## Stage29 normal current-package native/live measurements

Actual stage27 package passes122 fixed native checks on a fresh owned synthetic managed
session, including native Undo/Redo and saved-artifact revalidation. Separate sequential
read-only five-pair Python/Rust live runs make80STDIOrequests and pass40exact scene,
selection and PNG checks. No sampler errors; seven objects throughout. Startup medians
506.854/6.882ms; connect1108.579/1106.663ms;scene201.468/206.803ms;render106.122/125.651ms.
MCP-tree sampled scene RSS102.17/26.70MiB; warm GUI/supervisor about282.09MiB for both.
Gaps median/max Python24.991/25.458ms, Rust24.349/25.358ms. RSS includes shared pages;
GUI/bus stay warm. These are normal executable measurements, without diagnostic spans.
Roundtrip remains mixed JSON/pipes/files/native processing, not exclusive IPC. No general
live or mutation speedup claim. Raw results/live-process-benchmark-stage29 and
live-process-benchmark-stage29-verified.json preserve actual wire/commands/env/samples.
Historical stage21 disappearance cause remains unknown despite no repeat in current sequence.
