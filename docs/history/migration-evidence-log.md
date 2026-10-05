> Historical snapshot archived on 2026-10-05 after PR #9 merged. Statements such as
> “current”, “next”, “local” and “uncommitted” describe their original checkpoint.
> Use [current status](../AGENT_HANDOFF.md) and [active backlog](../RUST_NEXT_PLAN.md).

User-approved Rust-only development update (2026-10-03): the legacy Python MCP
implementation and paired comparison harnesses are retired. Frozen JSON contracts and
historical evidence remain, but no executable Python oracle or repeated parity runs are
part of the current plan. Use Rust regression/invariant tests, STDIO, package and native
acceptance. Required Python helpers/supervisor remain in runtime/ and rust/package/.
Historical preservation/parity requirements below are superseded by this user decision.
Recovery location is recorded in migration/python-retirement.json. See CONTRIBUTING.md.

Stage29 current live benchmark complete:5alternating Python/Rust pairs/80read-only
requests/40scene-selection-PNG comparisons pass; sampler errors0, scene7objects throughout.
Startup medians506.854/6.882ms; scene201.468/206.803ms;render106.122/125.651ms.
No general live speedup; private GUI/bus warm, sampled shared-page RSS, mixed roundtrip.
Reports live-process-benchmark-stage29-verified.json and raw results. Current native122checks
complete within fixed scope; continue historical causes/attribution/platform/provenance/audit.

Stage29 transform/order: current stage27 package22+16native checks pass and repeat from
saved captures, bringing fixed native cohort to122. Original scene restored to7objects.
Current live read-only five-pair benchmark underway; historical stage21 cause remains unknown.
No new GUI launch or user window/document changes. Continue attribution/platform/provenance
and final requirement audit; current monitor is available, earlier no-monitor refusal historical.

Stage29 follow-up: current stage27 package also passes38 native group/ungroup/delete
checks, repeated from captured artifacts. Scene restored to7objects through native Undo.
Fixed verifier phase naming accepts recorded group-selection-apply without rewriting raw
artifacts; historical stage22 captured regression passes38. Native transform/order/current
benchmark and historical root-cause investigation remain pending. No user windows changed.

Current stage29 native checkpoint: actual stage27 package passes 46 fixed native checks
(insert/style repeat/duplicate, Undo/Redo, reconnect and guards), repeated from captured
artifacts. Owned session remains open. Extended structure/transform/order and native
benchmark pending; no cause/fix claim for historical stage21 disappearance. Review fixed
missing binary hash in native launch harness; initial selection/sync refusals retained.
See native-stage29-comparison.json and native-stage29-captured-comparison.json.

Stage28 isolated transport diagnostic:50 frame/handler+50file/process partitions pass,
5pairs160requests/20full inspection envelopes equal/sampler errors0. Normal stage27 unchanged.
Initial flush envelope subtraction refused: flush may overlap client roundtrip; v2 read-to-first
write + residual additive, write/flush separate. Clock epochs differ, duration comparison only.
No pureIPC/native/production claim. See RUST_PHASE_MEASUREMENTS/transport-profile-stage28 binding;
continue Python/warm/live boundaries, native-positive/package live, Windows/foreign/licensing.

Stage27 filesystem review: test-only FIFO regression detects injected lostNONBLOCK withouthang;
fullRust199/1ignored+clippy/fmt pass. Actual package12special-file/escape refusals preserve
original/zero managedwrites. Reviewed paths not exhaustive concurrent-race/foreign proof.

Stage27 current argument audit:12negative matrices/4351observations/24STDIOstarts pass actual
package; nine positive coercion suites547observations/316converted requests pass exact unbundled
release bytes. live_connect zero-conversion fixture explicitly excluded; initial wrong --cases
orchestration rejected before startup, retained. Resource map18of18 successful names is presence,
not fullcontent/error proof. Reference110tools/51models/36finite paths unchanged. Current reports
and snapshots:current-arguments-stage27-evidence-binding.json; no server source changed. Continue
native positive/package live, final compatibility/security/phase/foreign/licensing gates.

Stage27 current per-call/shell/diagnostic CLI: five pairs each/520STDIOrequests, all80 full
inspection envelopes match, sampler errors0. Normal startup926.689/21.454ms (Python/Rust),
inspect100k16176.299/2107.127ms; no general edit/export or large-RSS improvement. Warm shell
batch8.281/70.904ms. OutsideCLI not pureIPC; current production measurements now present,
native still pending. Fixed prompt live flag with actual tool-gate assertions;100exact scenarios
pass actual package. Reports process-benchmark-stage27-followup and prompt-stage27-comparison.
Next argument/current resource audit, precise IPC, native history and foreign/platform gates.

Stage27 synthetic live follow-up:15families/651 observations pass on exact release bytes.
No native/package-live inference. Named current union4619calls/110called/109success;
live_edit_selection only structural refusals. current-live-stage27-coverage.json and
results/current-live-synthetic-stage27-verified/comparison.json reference actual attempts.

Stage27 affected-family follow-up: 15 headless cohorts pass 1035 scenarios/profiles on
current package; raw results/current-family-stage27-v2 preserves path-only harness copies.
Coverage-with-package maps 1155 actual tool calls, 53/110 successful names; remaining57
explicit. No full/native/argument coverage claim; old results/bindings untouched.

Current candidate stage27 packages the DOM no-op fix. Cold/warm archive plus complete-tree
mtime no-op checks, 16 discovery, 10 doctor, 9 notice and 11 launcher cases pass. Build report
package-stage27-build-comparison.json; current-package.json indexes it. Current native and
performance acceptance still pending; no historical evidence transfer.

Stage26 sync diagnostic found transient proposed-record creation on DOM no-op. Current
source fixes memory-only DOM staging; CLI pre-dispatch audit gates and sync remain. Raw
phase-profile-stage26 contains pre-fix timings, failed regression and fixed Rust/STDIO logs.
Stage23 archive is pre-fix; replacement package is pending. Existing-debug edit attempt is
explicitly excluded; fresh-release edit evidence is recorded separately.

Stage25 paired Python/Rust phase diagnostics: 50+50 bounded handler partitions validated,
small/100-object full inspection envelopes match, sampler errors0. See phase-profile-{python,rust}-
stage25-comparison.json, phase-profile-stage25-envelope-comparison.json and
../docs/RUST_PHASE_MEASUREMENTS.md. Next investigate individual sync latency; pure IPC,
unguarded file APIs and warm/live still pending. Production source/archive unchanged.

Stage24 isolated diagnostic phase follow-up: see ../docs/RUST_PHASE_MEASUREMENTS.md,
phase-profile-stage24-v2-comparison.json and raw results/phase-profile-stage24-v2/.
Initial nested-log contamination retained in results/phase-profile-stage24/. No normal
server source/package changed; pure IPC/Python/warm/live phase attribution remains pending.

Stage23 live follow-up (2026-10-03): 134 synthetic connect/loop/mutation observations pass.
Isolated native launch refused without a primary monitor; terminal logs retained under
results/native-gui-stage23. No relaunch or native success claim. See
live-stage23-followup-comparison.json and native-stage23-launch-comparison.json.

# Rust migration evidence

Reference: clean `main` at `c50a9248b3bca9177103373ce66d6dd1307bedd3`.
Work branch: `codex/rust-migration`. Python remains the production/reference server.

Current package index: `current-package.json`. The legacy `package-build-comparison.json`
is frozen at its hash-bound stage22 native checkpoint; use per-stage reports for acceptance.

## Stage23 source-first onboarding (2026-10-03)

setup.sh now invokes the fixed native source recipe and saves launcher settings; ready
archives still need only Inkscape. source-onboarding-stage23-comparison.json and its binding
record clean-target source-copy acceptance with existing host dependencies, intentional Cargo
settings overrides, the empty-PATH launcher fix and cold/warm/doctor/notices/discovery/edit
checks. Stage23 headless direct/diagnostic five-pair follow-up is saved; no stage23 GUI, uv-download or foreign execution is claimed. Historical
stage22/review evidence remains immutable; full migration is incomplete.

## Review cycles and stage22 (2026-10-03)

See ../docs/RUST_REVIEW.md for the process/audit findings, their fixes and scope.
review-cycle1-comparison.json and review-cycle1-evidence-binding.json bind the fresh
release, source snapshots, raw logs and actual stage22 archive acceptance. Native history
remains unresolved; no new GUI acceptance is transferred from stage21. setup.sh/run-mcp.sh
now configure a ready package without manual env edits; automatic source package building
is pending. Full migration remains incomplete.

### Stage21 live benchmark and extended-native refusal (2026-10-03)

Five alternating read-only Python/Rust pairs on the existing owned stage21 GUI pass
**40 scene/selection/PNG/RGBA comparisons**. All 80 requests succeed; no GUI launch or
editing effect is requested. Actual parent/PID/full executable path and packaged
binary/bridge/helper FILES hashes are checked before and after each trial. The harness
accepts the explicit fixed-native-acceptance state, records the actual binary SHA and
preserves its exact pre-format source as `harness-used.py`.

Median ms (Python / Rust): initialized **581.46 / 7.45**, connect **1,282.17 / 1,287.35**,
documents **22.26 / 25.80**, bind **340.91 / 255.85**, status **174.89 / 191.55**,
selection **40.87 / 51.88**, scene **231.16 / 271.26**, render **115.69 / 153.70**.
Sampled scene MCP-tree RSS is **102.81 / 26.77 MiB**; the separately sampled supervisor/
GUI trees are **205.95 / 206.25 MiB**. Do not add independent maxima. Python has one
rusage ESRCH for transient child PID 22485 in trial 0, marking its MCP memory observation
incomplete; Rust and both GUI samplers have zero errors. The warm existing GUI/bus and
background retained sessions are not a cold-launch or exclusive-load benchmark. Server
CPU, GUI surviving-process CPU and physical IO are observations, not exclusive wall phases.
Raw evidence, commands, environments and exact comparisons are in
`results/live-process-benchmark-stage21/`, `live-process-benchmark-stage21-comparison.json`
and `live-process-benchmark-stage21-evidence-binding.json`.

**Extended native stage21 acceptance did not pass.** After the benchmark (whose final
scene still had seven objects), native Select All was invoked for the fixed group fixture.
The next guarded group preflight saw four objects and only the original wrapper instead
of the two expected wrappers. It refused before any group mutation request. A separate
read-only sync/scene capture still shows four objects; the native XML inspector likewise
showed a single wrapper. Native Edit showed Undo available and Redo disabled. These
observations do not establish Select All, export or a specific helper/native component as
the cause. The duplicate's disappearance/history mismatch requires investigation.

`native-structure-stage21-comparison.json` explicitly records `passed:false`,
`native_extended_readiness:false`, **zero group edit requests**, counts **7 → 4 → 4**
and missing root ID `mcp_fdcae944f9854451b8755106c8891883_0`.
`results/native-structure-stage21/` preserves a separate ownership record, raw refusal
trace, later context capture and copied SVG/PNG evidence. Its binding verifies those
files, not acceptance. The original native ownership record and earlier 46-check evidence
remain immutable and scoped; they do not prove broad live/history readiness. The XML
inspector was closed; the exact owned drawing, supervisor 19914 and GUI 19927 are retained
for investigation. No document repair, duplicate reinsertion, structural retry or restart
was attempted. The candidate remains experimental and full migration remains incomplete.

### Stage21 repeated current-package benchmarks (2026-10-03)

Five alternating Python/Rust pairs use the actual stage21 executable and real MCP STDIO,
with SVG fixtures containing 100, 10,000 and 100,000 rectangles. All **40 complete inspect
responses** match, including text/structured envelopes, object counts and full trees;
only request-bound document IDs and JSON text decoding are normalized. All 200 direct
requests succeed. Initial missing-CLI PATH failure is retained separately; the completed
v2 run prepends the vendor CLI path for both implementations.

Median direct per-call times (ms):

| Measurement | Python | Rust stage21 |
|---|---:|---:|
| Popen to initialized | 868.39 | 20.10 |
| Single edit | 777.12 | 812.21 |
| Atomic two-edit batch | 579.67 | 637.76 |
| Render preview | 293.50 | 294.77 |
| PNG export | 299.94 | 297.81 |
| Open 100 / 10,000 / 100,000 | 4.50 / 18.03 / 175.25 | 16.75 / 22.95 / 92.45 |
| Inspect 100 / 10,000 / 100,000 | 29.02 / 1,556.12 / 16,014.94 | 2.97 / 205.49 / 2,106.37 |

Median sampled render process-tree RSS is **234.72 / 151.06 MiB** (Python/Rust).
At inspect-100,000 it is **1,184.61 / 1,101.14 MiB**: large full replies still require
substantial memory. Samples target 20ms and can miss peaks/short-lived children; shared
pages are summed. Rust has **one rusage ESRCH** for child PID 21826 during trial 2;
Python has zero errors. Missing child memory is retained explicitly; memory observations
are marked incomplete. The initial strict verifier refused this sample error. Its new
explicit `--record-sampler-errors` mode records every error without discarding reply fields;
it still fails sampler-thread failures and verifies every result. Request counts now come
from each trace instead of a hard-coded count.

A separate five-pair diagnostic run (100-object fixture) records actual vendor CLI wall/CPU
through a fixed developer wrapper; all 160 requests and **20 complete inspect responses**
pass, with zero sampler errors. Median CLI wall Python/Rust is **677.17/689.15 ms** for
single edit, **522.03/532.47 ms** for batch, **261.18/263.55 ms** for preview and
**261.28/267.27 ms** for export. The corresponding median outside-CLI remainders are
**230.71/290.16**, **45.02/107.08**, **23.52/27.33**, **25.28/28.44 ms**.
Those are separately computed medians, not additive components of median total latency.
Wrapper bootstrap adds overhead, so diagnostic times are not direct comparison timings.
The remainder includes server, IPC, filesystem and wrapper work; CPU/disk counters and
ping are observations, not exclusive wall-phase attribution. Exclusive server/IPC/logical
file-I/O phases and current-package live mutation timings still require work.

Raw commands/environments/traces/Mach counters/samples are in
`results/process-benchmark-stage21-v2/` and `results/process-benchmark-cli-stage21/`.
Reports: `process-benchmark-stage21-comparison.json`,
`process-benchmark-cli-stage21-comparison.json` and
`process-benchmark-stage21-evidence-binding.json`. These are per-call measurements;
stage13's warm-shell and live results retain their own binary/environment scope. Caches
were not flushed; five retained synthetic GUI sessions remained in the background and
no exclusive machine-load isolation was claimed. Startup and inspection improve in these
fixtures; edits/batches do not show an improvement. No general speed or memory claim.

Reproduce separately, never concurrently:

```sh
PATH=/Applications/Inkscape.app/Contents/MacOS:$PATH .venv/bin/python \
  scripts/migration_process_benchmark.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage21 \
  --output migration/results/process-benchmark-stage21-new \
  --repeats 5 --counts 100 10000 100000
# Run the diagnostic after the direct run has ended, into a different new directory.
PATH=/Applications/Inkscape.app/Contents/MacOS:$PATH .venv/bin/python \
  scripts/migration_process_benchmark.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage21 \
  --output migration/results/process-benchmark-cli-stage21-new \
  --repeats 5 --counts 100 --diagnostic-cli
```

### Stage21 deterministic bridge and fresh native package acceptance (2026-10-02)

The macOS developer builder sets `LC_ID_DYLIB` to
`@rpath/inkscape-mcp-context.so` instead of embedding the output directory. Two actual
same-host builds in different directories have identical bytes and valid ad-hoc signatures;
`context-reproducibility-comparison.json` records compiler commands, source hash and
load-command/signature evidence. This proves those two bridge builds, not whole-archive
reproducibility or foreign-host ABI compatibility. Stage21's packaged bridge matches them.

Current native-tested archive: `results/packages/inkscape-mcp-macos-arm64-stage21.tar.gz`,
**46,493,873 bytes**, SHA-256
`28e90a3c15d55a385702bb8c8df2cd834b6b69290d8ad0f004ac933b289e459d`.
It contains **2451 regular files / 132,768,713 bytes**. Rust executable and helper remain
byte-identical to stage19; the bridge and package metadata changed. Both actual relocated
archive installations (per-call and warm shell) pass with empty PATH, real Inkscape CLI
render/export, private helper/runtime and bus, 110 tools / 7 prompts / 18 resources,
atomic rollback, snapshots, no-op and approval refusal. Doctor passes 10 profiles without
GUI launch; notice acceptance passes 9 checks with the existing explicit audit gaps.

Fresh owned stage21 native session `/private/tmp/imcp-native-h1xg7o7d` has supervisor
19914 / Inkscape 19927. Its fixed insertion/style-repeat/duplicate, native Undo/Redo,
reconnect/binding and approval guards pass **46 checks / 24 preserved SVG/PNG artifacts**.
The independent captured-artifact verifier also passes all 46. A duplicate preflight first
refused because native Redo had cleared selection; the trace was preserved as
`duplicate-selection-refused.trace.json`. No mutation request was sent in that attempt.
After native Select All, the fixed guard confirmed the single expected wrapper before
applying duplication. The owned window is retained; no user document/configuration changed.

Reports are `package-stage21-{build,cold,shell}-comparison.json`,
`doctor-stage21-comparison.json`, `notices-stage21-comparison.json` and
`native-stage21-{comparison,captured-comparison}.json`.
`package-stage21-evidence-binding.json` verifies all 2451 FILES entries and binds sources,
package metadata, raw native traces and copied artifacts. Stage19's extra structure,
compound-transform and lower checks remain historical evidence for stage19; they have
not been transferred to stage21. Current-package per-call benchmarks are recorded above; current warm-shell/live mutation timings remain pending.

Stage19 closure preflight exactly matched the restored synthetic scene. Its native save
confirmation did not close after coordinate/AX Raise actions; both recorded PIDs remained
alive. `results/native-gui-stage19/closure-attempt.json` records closure as unconfirmed;
the original ownership record remains unchanged. Seven synthetic sessions have now been
launched: stage5/6 verified closed, stage12 closure unconfirmed, stage13 retained, stage18
crash UI retained, stage19 closure dialog retained, stage21 retained. No process was killed.
Full migration, additional native effects/fault acceptance, other-host execution, phase
benchmarks and signing/ABI/license gaps remain pending. Historical stage20 notes below
were superseded by this stage21 install candidate.

### Stage20 crate notices and package inventory (2026-10-02)

The native developer builder now collects notices from Cargo's locked offline
normal/build dependency graph for the native host. Each crate archive is SHA-256
verified against Cargo.lock before bounded tar reads; paths/links/duplicate notices
and excessive input/member/file/total sizes refuse. No extraction or arbitrary code
execution is added to MCP. The archive contains **92 crates / 178 notice files**, plus
**13 Rust standard-library notice files**, `THIRD_PARTY_NOTICES.md` and a machine-readable
`LICENSE-INVENTORY.json`; Python wheel notices already remain alongside dist-info.
The graph conservatively includes build/proc-macro dependencies and does not assert
that every listed dependency is linked into the executable.

`rmcp` 3.5.0 has no license text in its Cargo archive. Its metadata declares Apache-2.0;
the complete upstream LICENSE was fetched from the exact commit recorded by the crate's
`.cargo_vcs_info.json` (`0cde3c5cf3e6aff0cc852ce6045f107e95991f48`). Source URL/hash
are preserved under `migration/vendor-notices/rmcp-3.5.0`, and the collector verifies
both before copying. That upstream text retains its Apache/MIT transition provisions;
no license alternatives or attribution were silently replaced.

Actual stage20 build: **2451 regular files / 132,767,882 bytes**. Archive:
`results/packages/inkscape-mcp-macos-arm64-stage20.tar.gz`, **46,493,840 bytes**, SHA-256
`711eb12708050df7e0a23f7e5fde97d9ea0bbc51f60db572d9d3ed784e30e276`.
Actual archive extraction with empty PATH passes cold/per-call and warm-shell private
runtime/helper CLIs/bus/110-7-18 STDIO/real CLI PNG-SVG/batch/snapshot/no-op/approval checks.
Ten read-only doctor profiles pass. Notice acceptance passes **9 checks**, including
checksum tampering, traversal, symlink, duplicate and oversize refusals, no extraction,
all real copied crate notices bound to FILES.json and explicit audit gaps.
`package-stage20-evidence-binding.json` hashes the build/acceptance/source/provenance.

The native builder's four prepared host jobs now run notice acceptance; YAML and the
existing four-target/two-refusal/three-case synthetic ELF checks pass. No remote CI
execution is claimed. Current Ruff check/format cover 324 files; no production Python
or Rust code changed, and the previous full suites are not claimed as a new run.

**Stage20 native GUI evidence is not transferred from stage19.** A component-byte
comparison found only context.so and package metadata differ among prior files; Rust,
private Python/helpers/bus bytes match. Mach-O load-command records show that bridge
recompilation embeds the new output path in LC_ID_DYLIB and changes LC_UUID. The failed
full runtime-equivalence check is explicit in `package-stage20-build-comparison.json`.
Source equality/load headers alone do not prove native behavior. Stage20 still needs
its own owned GUI acceptance (or a stronger controlled bridge equivalence proof).
Stage19 remains the current native-tested installation candidate; stage20 is a tested
notice-bearing CLI candidate. No new GUI launch, user config change or window action
occurred in this checkpoint. Existing retained sessions are preserved.

`redistribution_audit_complete` remains **false**. The actual bus/GLib/gettext/pcre2
SBOMs and copied paths are inventoried in `native-license-gaps-stage20.json`: GLib has
only its SBOM, while D-Bus COPYING references license texts not currently copied.
Private standalone CPython's bundled native dependencies, exact corresponding-source/
relink obligations, native-library license texts and foreign-host attribution remain
open. This notice collection is factual provenance, not legal clearance or a complete
release audit. The full migration remains active; native text/document-switch/lost-result
coverage, platform execution/Windows protection, deterministic bridge packaging and
current phase benchmarks remain independent next work.

### Stage19 native transform and paint order (2026-10-02)

Reused the independently revalidated owned stage19 GUI (12505, supervisor 12496),
without launch/restart, package/source changes or effects on the retained other windows.
The fixed structure harness now accepts compound `transform` and single-known-ID `lower`
phases, retaining its binary/manifest/exact-process/document checks and no-retry trace gate.

Compound translation (20,15), scale 1.25 and rotation 15 on the two synthetic groups
passes **22 checks / 10 SVG/PNG artifacts**: existing IDs/paint order, unchanged 7-object
count, only the selected matrices changed, expected six affine coefficients, applied
Operation Record and native AX Undo/Redo plus final restoration. Expected coefficients
use a 5e-6 absolute tolerance for Inkscape's six-digit matrix serialization; raw SVGs
are retained. Drawing C14N omits only `sodipodi:namedview` GUI metadata. Exact scene and
RGBA Undo/Redo/restoration are verified independently. `native-transform-stage19-comparison.json` and `native-transform-stage19-captured-comparison.json` pass.

Native CUA click on the owned canvas selected only the already known duplicate wrapper;
MCP preflight checked its exact ID before `lower`. The order change passes **16 checks /
10 artifacts**, including reversal of the two root sibling groups, byte-equivalent C14N
group subtrees, unchanged pixels/count/IDs, immediate applied audit and exact native
Undo/Redo plus final restoration. The fixture has identical overlapping blue copies:
this proves the XML paint order and preserved appearance, not differing-color occlusion.
`native-order-stage19-{comparison,captured-comparison}.json` pass. Both verifiers rerun
successfully on saved artifacts without GUI access; no uncertain mutation was retried.

`native-more-effects-stage19-binding-comparison.json` hashes the phase traces, ownership,
scene/SVG sync, verifier source and reports against the unchanged package binary/helper.
The official Inkscape executable remains unchanged. The original scene has been restored
and the owned window retained. The current archive remains stage19 with its existing
cold/warm/doctor evidence. Ruff check/format (322 files) and `git diff --check` pass;
production Python/Rust code was unchanged, so prior full gates are not claimed as rerun.
No MCP instructions/surface changed; generated llms files remain synchronized.

This adds two scoped native effect families to stage19's earlier 46+38 checks. It is
not broad native compatibility or a benchmark. Text, other order operations, parent-
transform fixtures, document switching/lost results, epilog no-op limits, cross-platform
execution/Windows protection and current phase benchmarks remain. The previous goal
turn was progress; the full goal remains active with independent work available.

### Stage19 document-preserving helper and native structure (2026-10-02)

The changed-edit helper now keeps the original SVG root and document instead of
`_setroot`: it retains document-level comments, processing instructions, their order,
and DOCTYPE metadata. After full clone validation it removes each original child
through inkex's callback before attaching validated children, releasing the ID cache
without reminting existing IDs. Exact no-ops retain the original tree untouched.
A root tag/namespace mismatch refuses before mutation. This remains the fixed private
Python/inkex helper; no arbitrary extension or execution surface was added.

Focused tests cover changed/no-op fill, root identity, prolog/epilog order and DOCTYPE.
The packaged stage19 Python/helper with actual Inkscape 1.4.3 inkex passes **20 offline
file/stdin observations**, including result fingerprints equal to emitted SVG, existing
IDs, text/style changes, duplicate, insertion and guard refusals. Changed fixtures include
DOCTYPE and two epilog siblings. This is separate from native GUI/Undo proof.
`helper-prolog-stage19-v2-comparison.json` retains commands, environments and input/source
hashes. Failed development observations remain under `helper-prolog-source`: an initial
`clear()` trial exposed inkex's stale ID cache; the subsequent all-epilog trial passed
10/20 because vendor inkex's baseline deepcopy reverses multiple epilog nodes and emits
SVG even on a no-op/refusal. The final probe explicitly separates changed epilog fixtures
from no-op fixtures; native no-op behavior with multiple epilog siblings remains pending.

A fresh owned stage19 GUI at `/private/tmp/imcp-native-i4iar794` (supervisor 12496,
Inkscape 12505) uses the actual private runtime/bus/prebuilt bridge, owned profile and
empty PATH. Startup creates no session; explicit synthetic launch/context binding pass.
The installed helper matches the archive/source bytes. The retained drawing UUID is
`fbedd775-7757-40a9-a6be-50944b45f81e`, window
`50e1b940-0042-4ab0-9c8c-22a6d84fc984`.

Native AX Edit-menu insertion/style-repeat/duplicate Undo/Redo plus reconnect/approval
guards pass **46 checks / 24 SVG/PNG artifacts**. Extended structural acceptance now passes
**38 checks / 26 artifacts**: group 7→8→7→8, ungroup 8→7→8→7, delete 7→1→7→1 and final
native Undo restoration to 7. Exact scene trees, IDs, plain group paint order, RGBA
appearance, Undo/Redo pixels and immediate applied Operation Records are checked.
Both verifiers repeat successfully using copied artifacts without GUI access.
Reports: `native-stage19-{comparison,captured-comparison,binding-comparison}.json` and
`native-structure-stage19-{comparison,captured-comparison}.json`.
This one fixed sequence passes the previously failing group-Redo/Select-All/preflight
point; it does **not** prove the stage18 crash's root cause or universal native readiness.

Stage18's read-only owned-PID sample identifies the stack through bridge method →
`export_do` → `do_export_vector` → `SPDocument::copy` → `SPDocument::createDoc` →
`crash_handler`. Matching official `INKSCAPE_1_4_3` sources and hashes, a valid earlier
10-ID autosave, the blank four-ID preflight SVG and sample are retained in
`results/native-gui-stage18/crash-evidence/copy-stack-investigation.json`.
The stack narrows the failure to live vector-export copying; preceding corruption remains
unproven. No ungroup request was sent in stage18. Its crash GUI remains untouched.

Stage19 archive: **46,347,048 bytes**, SHA-256
`d18c42a06bd76a7fcb2434298bcb632b1d5b91c83995ad69e2c3b01be424d4d2`.
Rust executable is unchanged from stage18 (`c30527cf…33030`); helper SHA-256 is
`30ce11b57011990fa726d2d86dc3c9718cdc48ec1ee805086927a5ac68dc5bf8`.
Actual cold/per-call and warm-shell archive extraction pass with empty PATH, private
imports/helper CLIs/bus, 110/7/18 STDIO, real PNG/SVG and snapshot/batch/approval/no-op
checks. Doctor passes ten read-only profiles without GUI launch. Six owned native
sessions have been launched historically; stage5/6 closed, stage12 uncertain, stage13
healthy retained, stage18 crash retained and stage19 healthy retained.

Current Python checks with real Inkscape: **1289 passed / 12 skipped**; without CLI:
1211 passed / 90 skipped. Ruff checks/format (321 files) and strict mypy (122 files) pass.
The unchanged Rust executable retains stage18's 193-test/clippy/discovery/argument
checks; they were not relabeled as a new run. Regenerated llms files are unchanged.
No commits, publication, user MCP configuration edits or user-window actions occurred.
Full migration remains active: additional native text/transform/order/document-switch
and uncertain-result coverage, the epilog no-op limitation, other-platform execution/
Windows protection, licenses/ABI/signing limits and current phase benchmarks remain.


Additional actual stage19 helper probes pass **24/24** file/stdin observations in
`helper-stylesheet-transform-stage19-comparison.json`, adding a stylesheet and a
stylesheet-plus-transform fixture with preserved stylesheet ID and output fingerprint.
A vendor stylesheet-removal cache concern from source inspection did not reproduce in
these fixed cases (22/22 stylesheet, then 24/24 including transform). A speculative
cache-clearing variant also passed offline and remains a recorded experiment only;
that workaround was removed. Current source helper again matches stage19 bytes. This
establishes those fixtures, not all inkex cache behavior. `qa-stage19-comparison.json`
records this checkpoint's automated gates; native GUI evidence remains separate.

### Stage18 extended native structure failure (2026-10-02)

Extended acceptance reused the recorded stage18 session (PID 8205/supervisor 8176),
without a new launch. The fixed developer harness `migration_native_structure.py`
checks exact native PID/parent/executable, supervisor path, private manifest, packaged
binary hash and document UUIDs before each phase; mutations additionally require the
exact known selectable top-level group IDs. One initial group attempt refused before
mutation because the harness included `namedview`/`defs` in its expected selection.
That raw trace remains preserved; the corrected fixed phase uses only ordinary groups.

Native group and AX Edit-menu Undo/Redo completed **7→8→7→8** visible objects.
Eight checks pass in `native-structure-stage18-comparison.json`: exact before/after
scene trees and RGBA, preserved appearance and applied Operation Record, plus proof
that no later ungroup mutation was sent. Eight group SVG/PNG artifacts are retained
under `results/native-gui-stage18/structure-captured-artifacts`.

**Broader native structure acceptance failed.** After group Redo and native Select All,
the read-only preflight for ungroup exported a blank document, then `live_get_scene`
timed out. Inkscape stderr recorded a crash and emergency save. The saved emergency
file has only the XML declaration/comment and fails XML parsing; its CLI exit code 0
with no query output is explicitly not success evidence. The preceding complete
`group-redone.svg` remains valid and produces nine geometry rows with real headless
Inkscape, with original bytes preserved. Crash logs, input hashes, owned emergency
save and separate owned-profile CLI query results are under
`results/native-gui-stage18/crash-evidence`.

Cause remains unproven: the trace contains **no `live_edit_selection` ungroup request**.
Ungroup/delete and their Undo/Redo have not run. The exact process remained live in
crash UI; no timeout-triggered relaunch, close, process kill or mutation retry occurred.
The session state is `owned-native-crash-dialog-retained`. Its earlier 46 fixed native
checks remain valid for that scope, but do not establish native structure readiness.
This failure needs a controlled reproduction/root-cause fix before broad live readiness
can be claimed; current archive/production code remain unchanged. Full migration is
incomplete; independent platform/security/benchmark work remains available.

### Stage18 current-package native GUI acceptance (2026-10-02)

The actual stage18 package now passes native launch/context binding on a new explicitly
owned synthetic managed session at `/private/tmp/imcp-native-sgy_a85u`. Its private
Python supervisor (PID 8176), private D-Bus and copied managed Inkscape (PID 8205) run
with owned HOME/profile/TMPDIR/workspace and empty PATH. Startup was observed before
`live_launch` and created no session. Vendor Inkscape 1.4.3 bytes remain unchanged.
The native document UUIDs and exact PIDs/paths are retained in
`results/native-gui-stage18/session.json` and `final-owned-processes.txt`.

Real CUA native AX Edit-menu Undo/Redo has been performed for insertion, style plus
repeat, and duplication. Captured STDIO scene/SVG/PNG evidence proves insertion 4→1→4
objects; repeated style adds no extra native Undo step; duplication 4→7→4→7 restores
exact IDs/tree/RGBA. The named semantic group remains ordinary, geometry stays
editable, the blue child fill correctly overrides its orange wrapper fill. Immediate
applied Operation Records, distinct approved-repeat audit, unbound reconnect refusal
and approval refusal are checked. Guards preserve exact final tree and pixels.

`native-stage18-comparison.json` passes **46 checks / 24 hashed SVG/PNG artifacts**;
`native-stage18-captured-comparison.json` repeats verification using saved artifacts
without a running GUI. `native-stage18-binding-comparison.json` binds these reports to
the current source/package executable and manifest hashes, lists native UI actions and
records the retained owned session. This is current-package native evidence, unlike
the preserved historical stage13 result. It covers these fixed effects/guards only,
not all live effects, lost results or document switching, and is not a benchmark.

After insertion Redo, native exported selection was empty and the first style harness
refused before mutation. Its trace/stderr remain as `preselection-style-repeat.*`.
Native AX Select All restored the exact known wrapper ID; the subsequent guarded
style/duplicate tests passed. Two CUA coordinate clicks reported `noWindowsAvailable`
while exact PID/private context remained live; AX menu actions worked. This was not
treated as a stopped session, no relaunch or process kill occurred, and no uncertain
mutation was retried. The test window remains open for further owned native effects.
Stage12/stage13 retained process identities were rechecked and their GUIs untouched.

The archive is unchanged (same stage18 SHA-256); candidate metadata now points to this
scoped native result. No production code, helper, instructions or user MCP configuration
changed. Previous 193-test/clippy, 4927 argument observations, discovery, cold/warm and
doctor evidence remains bound to this same binary. Full migration, broader native
coverage, other-platform execution/Windows port, ABI/license audit and phase benchmark
coverage remain incomplete.

### Stage18 finite-number model constraints (2026-10-02)

The trusted annotation audit now derives exact wildcard field paths for argument models
with `allow_inf_nan=False`, including tagged union branches and nested/list models.
`argument-schema-finite-audit.json` freezes 110 tools / 51 models, source/generator hashes
and **36 finite float paths**; `contracts/argument-finite-number-paths.json` is compiled
into Rust. These paths cover repeat placement/points/variation/anchor and render/compare
region models. The earlier stage17 audit remains historical and unchanged.

Native argument validation rejects nonfinite conversions on those model fields with
Pydantic's `finite_number` before numeric bounds, approvals, locks, IPC and edit mutation.
JSON integers too large for Python float conversion return `float_type`, preserving
original integer input types. This is model configuration absent from public JSON Schema;
public discovery and the private helper/supervisor behavior have not been changed.

A fresh actual Python STDIO oracle captures **576 cases / 36 paths**: NaN/infinity text
in different spellings, positive/negative exponent overflow, valid numeric strings,
invalid strings/null/dict, booleans and JSON integers above/below float overflow.
Unknown root arguments ensure even valid controls refuse before kernel execution.
The frozen native fixture and fresh source/packaged STDIO match every ordered error.
All previous negative matrices rerun on the new source: **4927 observations / 26 server
starts** in `migration/argument-finite-comparison.json`, with no normalization, differences
or workspace mutation. Nonstandard JSON NaN/Infinity literals are outside this matrix;
more scalar grammar/float formatting, nested integer extremes and bounded pathological
error representations remain to be checked. Full compatibility is not claimed.

Rust format/all-target clippy/locked release and **193 tests (one child helper ignored)**
pass, as do Ruff lint/format and mypy. LLM indexes regenerate unchanged; discovery is
still **16/16 exact**. Packaged repeat coercion with identical actual vendor CLI and owned
HOME/Inkscape/XDG profiles passes **19 scenarios** including SVG, records, restoration
and pixels (`repeat-finite-package-cli-comparison.json`).

The macOS arm64 stage18 candidate passes actual fresh empty-PATH archive extraction in
per-call and warm-shell modes, private helper imports/CLIs/bus exchange, 110/7/18 STDIO,
real blue PNG/export resource, atomic rollback, repeat fragment/restore, approvals,
no-op audit and preserved originals. All **10 doctor profiles** pass. Current pointer:
`package-stage18-build-comparison.json`; all prior candidate evidence stays scoped to
its original binaries. Stage18 native GUI/Undo and new benchmarks have not run;
stage13 native/benchmark and stage15 synthetic live evidence remain historical.
No existing GUI was touched. Full migration, native acceptance and other-platform
execution/Windows port remain incomplete.

### Stage17 integer boundary compatibility and schema audit (2026-10-02)

Native argument validation now distinguishes arbitrary-precision JSON integers from
floats without passing them through f64. It retains exact accepted integer values and
reports the original Python `int` input type for values beyond u64. Float-to-integer
conversion now follows the observed strict magnitude boundary at 2^63; fractional
values retain `int_from_float`, out-of-range integral floats return `int_parsing_size`.
Valid integer strings enforce Pydantic's 4300-character decimal conversion limit after
removing underscores, a positive sign, leading zeroes and a zero fractional tail;
the negative sign counts for a nonzero value. Invalid syntax still wins over the
size limit. Existing typed kernels and safety gates remain in place.

A new actual Python STDIO oracle covers **487 observations / 21 direct or nullable
integer parameters**, including large exact JSON integers, positive/negative floats
near 2^63, very large finite floats, 4299/4300/4301 digit strings, signs, underscores,
leading zeroes, integral decimal tails and malformed/fractional strings. Four additional
cases check big-integer error input types. Ordered error messages match the frozen
native fixture and fresh source/packaged STDIO; the first 257-case oracle is preserved.
All previous argument matrices rerun on the fresh source binary. The aggregate contains
**4262 observations / 24 STDIO starts**, no differences or normalization and no workspace
mutation: `migration/argument-integers-comparison.json`.

`migration_argument_schema_audit.py` inspects the actual registered trusted annotations,
including tagged union branches, and freezes source/generator/contract hashes at
`migration/argument-schema-audit.json`. It records **110 tools / 51 argument models**:
no multi-type `anyOf`, aliases or custom Pydantic field/model/root validators occur.
The earlier generic pending list overstated those gaps. The audit identifies real
model configuration beyond JSON Schema: repeat models use `allow_inf_nan=False`,
which still needs native argument-boundary/error coverage. Scalar grammar/float error
formatting, nested extremes and bounded pathological representations also need work;
this audit does not claim complete argument or kernel behavior compatibility.

Rust format, all-target clippy, locked release and **192 tests (one child helper ignored)**
pass. Ruff lint/format and mypy pass; LLM indexes regenerate unchanged. Discovery stays
**16/16 exact**. Fresh packaged repeat coercion acceptance with the same actual vendor
CLI and owned HOME/Inkscape/XDG profiles passes **19 scenarios**, including SVG/records,
restoration and pixel assertions (`repeat-integers-package-cli-comparison.json`).

The new macOS arm64 stage17 archive passes actual empty-PATH extraction/install in both
per-call and warm-shell modes: private runtime/helper/bus, 110/7/18 STDIO discovery,
real blue PNG/export resource, atomic rollback, repeat fragment/restore, approval,
no-op audit and preserved originals. All **10 doctor profiles** pass. The candidate
manifest is `package-stage17-build-comparison.json`; stage16 evidence stays unchanged.
No current native GUI/Undo or new benchmark was run. Stage13 native/benchmark and
stage15 synthetic live evidence retain their original hashes/scopes. No existing GUI
was touched; full migration remains incomplete.

### Stage16 container argument validation and archive (2026-10-02)

Native validation now checks the exposed fixed two-number tuples (shape, missing items,
item types and overlength), typed palette mapping values, repeat polyline point-list
limits and quality semantic-group list limits. It follows the observed error order:
maximum length precedes invalid elements; minimum length follows successful item
validation. Validation remains before approvals, locks, IPC and edit mutation, with
existing native array/error caps retained. Multi-type unions, custom model validators,
aliases, extreme integers and pathological error representations remain pending.

The new actual Python STDIO oracle has **78 cases**, including direct tools and tagged
batch operations, accepted numeric strings/bools, short/long tuples, typed mapping
values and invalid elements at both ends of oversized lists. All 78 ordered error
messages match a native fixture test and fresh source/packaged STDIO. Nine previous
negative matrices also pass on the new source binary: **3366 observations / 22 starts**
across all eleven comparisons, with no normalization or workspace mutation.
`migration/argument-containers-comparison.json` binds all reports and executable hashes.
The earlier 67-case oracle remains preserved alongside the expanded one.

The packaged repeat coercion comparison initially failed because only the packaged
server could locate vendor Inkscape through its fallback; Python lacked CLI on PATH.
The failed report/raw results remain at `migration/repeat-containers-package-comparison.json`.
A new run gives both implementations the same actual CLI and owned HOME/Inkscape/XDG
profiles: **19 scenarios pass**, including SVG/records/restoration and pixel assertions,
with transformed requests retained in `repeat-containers-package-cli-comparison.json`.
This rerun does not exclude the original differences or modify production Python.

Rust format/all-target clippy/locked release and **191 tests (one child helper ignored)**
pass; Ruff lint/format and mypy pass. LLM indexes were regenerated unchanged. Discovery
is still **16/16 exact**. Current macOS arm64 stage16 archive passes extraction into
fresh empty-PATH environments, cold/per-call and warm-shell STDIO/real CLI render/export,
private helper imports and bus exchange, no-op/approval/rollback/original checks, and
all **10 doctor profiles**. `package-stage16-build-comparison.json` is the candidate
manifest; stage15 evidence is preserved and has not been relabeled.

Stage16 native GUI/Undo and new performance measurements have **not** run. The scoped
native GUI and benchmarks remain stage13; stage15 synthetic live comparisons remain
stage15. Existing test windows were not touched. Full migration remains incomplete.

### Stage15 live boundary regression (2026-10-02)

Fresh release and packaged stage15 binaries have identical SHA-256. Real STDIO comparison
against authenticated owned loopback peers now covers **657 observations / 30 server starts**:
157 source observations (connect/events/mutation/loop), 263 packaged observations (those plus
viewport/render), and 237 packaged coercion observations (events/mutation/loop/viewport/render).
Every report binds the executable hash; raw requests/replies/stderr, fixed IPC parameters,
records/resources/history and pixel checks are retained where the underlying family asserts
those invariants. No broad normalization or exclusion is added.

Packaged/coercion harnesses explicitly use owned HOME/Inkscape/XDG profiles and remove inherited
D-Bus/managed/context bindings. They connect only to their authenticated synthetic peers;
no existing GUI is connected or changed. Coercion suites retain each original/transformed
wire request and hash their wrapper, exercising numeric strings/integer `.0` tails/boolean
strings. Numeric bools are converted to `1`/`0`, keeping their accepted scalar meaning.

There are **no unexpected differences** in the asserted scope. Each mutation matrix retains
its two explicitly asserted stronger lost-reply uncertainty differences from Python; each
loop matrix retains ten PNG encoding differences alongside semantic/pixel comparisons. These
are visible result differences, not UUID/timestamp normalization. The aggregate is
`migration/live-stage15-regressions-comparison.json`; all 15 individual reports are listed.
This is **synthetic IPC acceptance, not native GUI/Undo/Redo** and not a new live benchmark.

No production code or archive changed in this phase. Six existing harnesses now support
explicit `--rust/--output/--report` and hashes, preserving earlier debug defaults and evidence.
Ruff lint/format (318 files), mypy (122 sources), diff checks and regenerated unchanged LLM
indexes pass. Previous 190-test/clippy/release evidence remains scoped to this same unchanged
binary. Read-only PID inspection confirms both retained stage12/stage13 owned GUI/supervisor
pairs still run; stage12 closure remains unconfirmed. Full argument/native/other-platform
compatibility and the full migration goal remain incomplete.

### Native POSIX package jobs and stage15 (2026-10-02)

The native builder now selects macOS/Linux arm64/x86_64. macOS retains the prebuilt Cocoa
bridge/private bus; GNU/Linux packages add bounded non-glibc ELF dependency closure,
relative RPATH relocation, post-relocation loader checks and dependency license provenance.
Original host libraries are not patched. `migration_build_posix_package.py` is the common
entry point; the previous CLI remains compatible. Six helper versions are pinned separately.

Four CI jobs now build actual archives and perform empty-PATH cold/warm CLI/STDIO installation
and doctor checks, retaining archives/manifests/results. Runner labels/action pins and amd64/
arm64 PPA metadata were verified against primary sources. **No remote CI or actual Linux/
Intel execution has run.** Windows remains pending its native handle/runtime port. Source
workflow evidence is `migration/posix-package-ci-prepared-comparison.json`; the three
synthetic ELF closure/missing/collision cases and four target/two refusal cases are
`migration/posix-builder-logic-comparison.json`, explicitly not Linux ABI/loader acceptance.
Two initial fixture failures (canonical `/var` root and an insufficient parent traversal)
are retained under `results/posix-builder-logic*/`.

Doctor accepts matching supported host manifests and checks thin/fat Mach-O or ELF64 headers
with bounded fat tables. macOS now checks vendor GTK3 architecture; Linux omits Cocoa-only
prerequisites and reports managed GUI unsupported. Linux packaged discovery can locate
standard Inkscape paths with empty PATH; its ordinary D-Bus/socket support remains, while
`live_launch` stays macOS-only. No public tool/schema/resource/prompt change is introduced.

**190 Rust tests pass, one ignored crash-child entry point**, clippy/format/locked release,
Ruff (318 files), mypy (122 sources), diff checks and **16/16 fresh discovery configurations**
pass. New current-Mac stage15 archive passes actual cold/warm installation, private runtime/
helper/bus/110-7-18/real PNG/resources/rollback/history/original checks and ten doctor profiles.
**Stage15 native GUI/Undo remains unverified**; scoped stage13 GUI/benchmark evidence remains
historical, with both retained owned sessions preserved. No user configuration was changed.
Build instructions and exact platform/evidence limits are in `docs/RUST_PACKAGING.md`.
The latest package pointer is `migration/package-stage15-build-comparison.json`; full goal
completion and other-platform execution, ABI/license/signing audit remain pending.

### Current local archive: stage14 (2026-10-02)

The current macOS arm64 archive is
`migration/results/packages/inkscape-mcp-macos-arm64-stage14.tar.gz`, **46,351,540 bytes**,
SHA-256 `4a2887e78135297ef1be98e8a267d7ea542e33907f99f08aa3f3682f9bf4e0c4`.
It contains the exact current release binary, including the argument-validation and
materialization work below: **2258 regular files / 130,192,959 bytes** in the file manifest.
Cold/per-call and warm-shell acceptance both install from this archive into owned empty-PATH
environments, verify every file, execute private Python/helper CLIs/private D-Bus, enumerate
110/7/18, render/export real pixels/resources and check no-op, approvals, originals, atomic
rollback, repeat/fragment snapshots and restore. Doctor passes ten profiles without launching
GUI or bytecode/system modifications (`migration/package-stage14-cold-comparison.json`,
`package-stage14-shell-comparison.json`, `doctor-stage14-comparison.json`).

The first macOS tar attempt included an AppleDouble file as a second archive root; the
bounded installer refused it before extraction/runtime startup. That failed archive and
its root inventory are preserved. The corrected archive uses Python tarfile, and the
builder now offers `--archive` with exclusive creation and one root, avoiding those sidecars.
`migration/package-build-comparison.json` now points to stage14; the previous full pointer
is saved as `migration/package-stage13-build-historical.json`.

**Stage14 native GUI effect/Undo acceptance has not run.** The stage13 scoped GUI result and
its five-pair benchmark retain their original binary/runtime scope. Stage12 closure remains
unconfirmed and both owned GUI sessions remain preserved. Stage14 has ad-hoc assets, no
Developer ID/notarization; no security settings were changed. Full compatibility and other
platform packaging remain incomplete. The complete record is
`migration/package-stage14-build-comparison.json`; installation/MCP commands at the top of
`docs/RUST_MIGRATION_REPORT.md` now use stage14. No user MCP configuration was modified.

### Validated argument materialization (2026-10-02)

The native boundary now materializes frozen defaults, finite numeric/boolean coercions,
nullable fields, tuple items, typed mapping values and selected tagged model branches after
validation and before kernel dispatch. Integer strings with allowed separators/zero decimal
tails and JSON integers keep arbitrary precision; ignored BaseModel extras are removed.
Unspecified multi-type unions and nonfinite numbers retain their input representation pending
full validation/model work. Raw inputs still determine validation errors. No Python validator
runs in the Rust server.

Trusted reference annotations/signature defaults are captured without invoking tools in
`migration/contracts/argument-normalization-values.json`: **110 signature-default cases and
182 supplied-value cases** match Rust. Only equivalent IEEE float exponent spellings are
canonicalized in that unit comparison; integers remain exact. Two annotation-invalid samples
and four nonfinite samples are retained separately in
`migration/argument-normalization-reference-recapture.json`; the original nonfinite JSON
capture is preserved under `results/qa-arguments-normalization/`. Recapture is byte-exact.

Fresh release real STDIO passes **3210 negative argument observations** (nine matrices,
no normalization, unchanged workspaces/no managed session), plus **210 behavioral scenarios**
with schema-directed numeric strings, integer `.0` tails and boolean strings: repeat 19,
atomic batch members 24, find 45 and edit 122. Find also sends ignored bbox model fields.
Full envelopes, SVG/history/bytes and real CLI pixels compare where the underlying suite
asserts them (`migration/*-coercion-final-comparison.json`, `edit-coercion-verified-comparison.json`).
The wrapper retains original/transformed requests. Its first read-only suite correctly
failed its coverage guard because it changed no arguments; `migration/read-coercion-comparison.json`
retains that failure and is not claimed as coercion evidence. Its underlying 71 ordinary
read scenarios had matched before the final large-integer fix. Claimed final reports bind
one SHA-256; edit was repeated with pre/post binary identity checks to bind that evidence.

**189 Rust tests pass, one ignored crash-child entry point**; clippy/all targets, format,
locked release, Ruff lint/format (316 files), mypy (122 sources) and diff checks pass.
LLM indexes regenerate unchanged. Evidence and remaining scope are indexed in
`migration/argument-normalization-source-comparison.json`. The stage13 archive still predates
this source change. Both owned stage12/stage13 supervisor/GUI pairs remain running; no launch,
window interaction or document mutation occurs in this phase. Full migration remains active.

### Numeric argument text and schema bounds (2026-10-02)

A dedicated native argument parser now follows the captured Pydantic numeric-text rules,
separately from Python `float()` decoding used for SVG data. Integer strings are checked
without f64 rounding: exponent syntax and nonzero fractional tails refuse, while ASCII
integer digits, allowed separators and all-zero decimal tails pass. Numeric argument text
rejects Unicode digits; the SVG decoder keeps its existing behavior. Schema minimum,
maximum and exclusive bounds now produce ordered reference errors using the original input.
No Python validation subprocess, new tool or executable choice is introduced.

**2250 new numeric-text observations and 960 rerun argument observations pass through real
STDIO, zero differences and no normalization**, on the same fresh release binary. The new
matrix spans 90 numeric fields and 25 strings per field, with a deliberate unknown root
keyword guaranteeing refusal before kernel dispatch, including for valid numeric text.
It validates boundary acceptance/errors, not successful numeric coercion in kernels.
Workspaces remain unchanged and no managed session is created. Reference capture, raw
requests/replies/stderr and SHA-256 bindings are preserved in
`migration/arguments-numeric-comparison.json` and `arguments-*-numeric-rerun-comparison.json`;
all complete numeric reference responses are frozen in
`migration/contracts/argument-numeric-text-errors.json`.

**187 Rust tests pass, one ignored crash-child entry point**; all-target clippy, format,
locked release build, Ruff lint/format (314 files), mypy (122 sources) and diff checks pass.
Fresh ordinary read acceptance passes **71** and reversible edit acceptance **122** scenarios,
zero differences, including real CLI previews, snapshots, audit and SVG bytes where asserted
(`migration/read-arguments-numeric-comparison.json`, `edit-arguments-numeric-comparison.json`).
LLM indexes regenerate unchanged. These results extend the historical 186-test checkpoint
below. They do not establish defaults/coercion materialization, ignored nested-field
normalization, all custom/model validators, union/alias parity or pathological numeric input
handling (including very large integer limits). Generic bounds are implemented for nested
schemas too; the new wire matrix exercises top-level fields and does not claim exhaustive
nested-bound coverage. Current source evidence is indexed in
`migration/argument-numeric-source-comparison.json`.

The **stage13 archive predates this change** and retains its own previous acceptance scope.
No GUI interaction or new launch occurs in this phase; read-only process inspection confirms
both retained stage12/stage13 owned supervisor/GUI pairs still run. Stage12 closure remains
unconfirmed. The full migration goal remains active.

### Unknown fields and invalid discriminator tags (2026-10-02)

The native boundary now rejects unexpected keyword arguments for every registered signature,
including zero-argument live/lifecycle tools, before dispatch. Nested models reject extra
fields only where the frozen schema explicitly sets `additionalProperties: false`; models
that permit ignored fields are not blanket-forbidden. Errors retain signature order, followed
by unexpected keys in their input order, and use the reference's `unexpected_keyword_argument`
or `extra_forbidden` messages. The 4096-error guard also bounds extra-field aggregation.

Invalid discriminator values now produce `union_tag_invalid`, with the exact ordered allowed
tags, in `apply_edits`, `repeat_objects` and `transform_objects`. Eighteen reference cases
cover unexpected string, null, bool, integer, object and list tags. Matching known tags still
enter their existing typed branches. No tool names, schemas, approvals or executable choices
are added; the change invokes no Python validator.

Reference capture ran Python only first; an exhaustive native unit check proved all **110
unexpected-keyword signatures refuse** before exercising the fresh binary over STDIO. This
includes `live_launch` and does not intentionally execute an unfixed launch-capable request.
The three new real STDIO matrices pass **110 unknown-keyword, six forbidden nested-field and
18 invalid-tag observations**, with zero differences and no normalization
(`migration/arguments-unknown-comparison.json`, `arguments-nested-unknown-comparison.json`,
`arguments-invalid-tag-comparison.json`). Complete responses are frozen in the corresponding
`migration/contracts/argument-*-errors.json`; raw traces/stderr and inventory checks are retained.

The previous five matrices also pass again on this same binary, giving **960 observations /
zero differences** across eight matrices and sixteen real server starts. All owned workspace
inventories stay unchanged and no managed directory/session is created. Every report binds
the current release SHA-256. The summary is `migration/argument-validation-source-comparison.json`;
older 826-case reports remain historical, with their distinct executable hashes.

**186 Rust tests passed, one ignored crash-child entry point**, all-target clippy, format and
locked release build pass. Ruff lint/format (**314 files**), strict mypy (**122 sources**) and
diff checks pass. Fresh discovery remains **16/16 exact configurations**; repeat acceptance
passes **19**, read **71**, edit **122** and atomic-batch member **24** observations, all zero
differences (`*-arguments-unknown-comparison.json`), with real Inkscape previews/history/bytes
where the individual harness asserts them. LLM indexes regenerate unchanged. Read-only process
inspection still identifies the two owned stage12/stage13 GUI/supervisor pairs; no window was
closed or new GUI launched in this phase, and stage12 closure remains unconfirmed.

These observations do not complete argument compatibility. Defaults/coercion materialization,
ignored nested-field normalization, multi-type unions, all numeric/model validators and aliases,
pathological scalar/representation cases and reserved/injected-name behavior remain to verify.
The current **stage13 archive predates these source changes**, retaining its scoped 177-test
and native GUI evidence. A later package/archive/native acceptance pass is still required;
other-platform release work remains incomplete and the full goal is active.

### Ordered scalar/container argument validation (2026-10-02)

The native pre-dispatch boundary now traverses ordered frozen schemas, aggregating missing
fields and invalid scalar/container shapes before kernel calls, approvals, locks or IPC.
It handles strict strings, numeric/boolean parsing errors, list/item and object fields,
nullable single-type fields, literals, selected tagged-union branches and missing tags.
The reference's JSON Schema omits model/Enum identities, so a separate developer generator
captures **12 direct/nullable/nested/list BaseModel paths** and the `StepAction` Enum path
from the trusted reference annotations for all 110 tools. Rust consumes only the static JSON
metadata; no Python validation subprocess is introduced. The metadata recapture is byte-exact
(`migration/argument-model-names-comparison.json`). Multi-model union metadata is not claimed.

Five fresh real STDIO matrices pass **826 observations / zero differences, no normalization**:

| Matrix | Observations | Report in migration/ |
|---|---:|---|
| Empty argument objects | 87 | arguments-empty-shapes-comparison.json |
| Required values replaced by objects | 87 | arguments-required-objects-comparison.json |
| Required values replaced by null | 87 | arguments-required-null-comparison.json |
| Individual scalar/container fields with wrong types | 418 | arguments-type-fields-comparison.json |
| One required field missing among supplied fields | 147 | arguments-missing-one-comparison.json |

Complete response envelopes, ordered multi-error messages, shortened Python input representations,
commands, traces and workspace inventories remain under `migration/results/arguments-*/`.
All five final matrices leave both workspace inventories unchanged and create no managed
session. Their Python responses are frozen in `migration/contracts/required-object-errors.json`,
`required-null-errors.json`, `argument-type-field-errors.json` and
`argument-missing-one-errors.json`, alongside the earlier empty fixture. The baseline object
and null runs had 87 differences each; the first broader field run had one difference because
`StepAction` needs error type `enum` rather than `literal_error`. Failed reports remain retained.

Validation refuses lists over **10,000 items** and stops aggregation above **4096 errors**,
with dedicated bounded-work tests. These are native guard limits, not claimed reference error
parity for oversized input. The refusal uses a native limit message rather than an invented
Pydantic error URL. Other typed kernel and workspace bounds still apply.

**183 Rust tests passed, one ignored crash-child entry point**, all-target clippy, format and
locked release build pass. Ruff lint/format (**314 files**), mypy (**122 sources**) and diff
checks pass. Fresh discovery matches **16/16 configurations**. Ordinary parity passes again:
**71 read, 122 edit, 131 fragment, 19 repeat and 24 atomic-batch member observations**, all zero
differences, including actual Inkscape previews, histories and original/snapshot bytes where
those harnesses assert them. Reports use the `*-argument-shapes-comparison.json` suffix.
The repeat/fragment harnesses now accept explicit binary/output/report paths so the fresh
release is tested instead of a stale default debug executable. LLM indexes remain unchanged.

This does **not** prove all 110 argument schemas compatible. Unknown-field handling,
materialized defaults/coercions, multi-type unions, invalid tag values, all numerical bounds,
model validators/aliases and pathological scalar/representation cases remain pending.
The **stage13 archive still predates this boundary work**; the change is in source and the
fresh release binary. Package stage13 retains its scoped 177-test/native-GUI evidence. Further
archive/native acceptance must follow the completed boundary work; the full goal is active.

### Required-argument validation before dispatch (2026-10-02)

The native STDIO boundary now checks an empty argument object against every frozen
required signature **before kernel calls, approvals, state locks or IPC**. It aggregates
missing fields in signature order and returns the exact reference call-validation message.
This covers all **87 tools with required arguments**, including document-selection and
approved-edit tools. Optional signatures still dispatch normally; no startup/reconnect
launch behavior changes. No Python validator/server subprocess is used by Rust.

The baseline `migration/arguments-empty-baseline-comparison.json` has **87 differences**;
Rust also created an Action-map cache while processing a missing-argument request. The
fixed fresh release passes **87 observations / zero differences, no normalization**, with
unchanged workspace inventories and no managed session creation
(`migration/arguments-empty-comparison.json`, `migration/results/arguments-empty-final/`).
The reference error responses are frozen in `migration/contracts/required-empty-errors.json`;
raw Python/Rust STDIO traces and stderr remain available, including failed harness attempts.
The first harness assertions incorrectly attributed workspace changes to Python; retained
inventories show Python unchanged and the Rust baseline's `action-maps/unknown.json` creation.

**179 Rust tests passed, one ignored child entry point**, including two new argument-boundary
tests and the ten regular crash-child invocations. All-target clippy and format pass; fresh
release build passes. The initial clippy nested-if failure is preserved beside its successful
rerun. Discovery remains **16/16 exact configurations** and read acceptance remains **71
observations / zero differences** (`discovery-arguments-empty-comparison.json`,
`read-arguments-empty-comparison.json`). Actual Inkscape edit acceptance also passes **122
observations / zero differences** (`edit-arguments-empty-comparison.json`). Ruff lint/format
(**313 files**), strict mypy (**122 sources**) and diff checks pass. LLM indexes are
regenerated unchanged.

This is a targeted first validation slice: nonempty argument objects, wrong scalar/container
types, defaults/coercion, combined error ordering, unknown fields and nested/discriminated
models still require full compatibility work. It does not claim validation parity for all
110 schemas. The **stage13 archive predates this change** and retains its recorded 177-test
checkpoint; the new validation is in source and `rust/target/release/inkscape-mcp-rust`.
A later candidate must be rebuilt and archive-tested after the broader boundary work.
The current package pointer remains stage13 and the goal remains incomplete.

### Repeated native live read benchmark (2026-10-02)

Five alternating sequential Python/stage13 Rust pairs now exercise the **existing owned
stage13 GUI** through true MCP STDIO: initialize, reconnect, list/bind the known document,
status, selection, scene and render. No GUI launch, editing effect or window close occurs.
Each trial revalidates the private manifest, exact GUI/supervisor command paths and document
IDs. A new owned workspace isolates artifacts; the existing synthetic profile/bus stay warm.
Raw commands, environment, complete replies, separate server/descendant and supervisor/GUI
libproc samples and PNGs are in `migration/results/live-process-benchmark-stage13/`.

`migration/live-process-benchmark-stage13-comparison.json` passes **40 comparisons** across
all ten trials: full scene JSON hashes, selection replies, and exact scene/render PNG plus
RGBA hashes. Connect/status clocks and minted artifact paths remain raw; complete envelope
parity is not claimed for those responses. The exact executed harness source is retained
with its matching initial SHA-256; subsequent source changes only address lint formatting
and reliable cleanup of sampler threads/raw evidence.

| Operation | Python median ms | Rust median ms |
|---|---:|---:|
| Actual STDIO startup | 570.08 | 7.64 |
| Reconnect to existing private GUI | 1262.65 | 1326.33 |
| List documents | 23.02 | 26.06 |
| Bind owned document | 239.88 | 256.42 |
| Live status | 177.77 | 192.52 |
| Selection read | 40.50 | 51.76 |
| Scene including frame | 246.39 | 274.72 |
| Following whole-canvas render | 130.84 | 155.54 |

Median sampled MCP subtree RSS during scene is **102.48 MiB Python / 27.05 MiB Rust**;
separately sampled owned supervisor/GUI tree RSS is **189.05 / 189.02 MiB**. These trees are
sampled independently; adding their separate maxima is not a measured simultaneous peak.
Shared pages remain included. CPU/physical IO and surviving GUI-process CPU deltas are
recorded, not an exclusive server/IPC/filesystem/Inkscape wall-time decomposition.

Python has zero sampler errors; Rust has **two ESRCH rusage observations** for transient
MCP-child PIDs during one trial, with no sampler-thread failure. Those missing observations
can hide short-lived child peaks/CPU. Maximum sample gaps are about 25.61/25.39ms. No cache
flush, cold GUI launch or mutation performance is measured. The render follows a scene
capture in the same session and can reuse the frame cache. Rust is slower for these live
read/IPC operations; only startup and sampled MCP memory show an improvement in these runs.
Both owned GUI sessions remain retained, including unconfirmed stage12 closure. This adds
live read measurements without completing native effect or full migration compatibility.

### Separate helper-to-Rust investigation (2026-10-02)

[The helper investigation](reports/RUST_HELPER_INVESTIGATION.md) documents a feasible compiled
one-shot SVG filter boundary and its unimplemented requirements. The actual stage13
Python/helper/vendor inkex passes **20/20 offline file/stdin protocol checks**
(`migration/helper-protocol-stage13-comparison.json`), including zero stdout for exact
style/text no-op and refused edits, changed SVG output, selection/content guards and
insertion refusal exit 251. Raw commands, hashes and results are retained. This does not
prove native Undo for a future Rust helper; the fixed shipped helper remains unchanged.
A replacement still needs a typed argument parser, copied-tree planner, exact no-op
serialization, descriptor-protected side channel and separate native Undo/Redo acceptance.
The managed supervisor and optional socket helper would remain Python after this step.

### Recovery journal and current stage13 package (2026-10-02)

The shared edit/restore pipeline now publishes a bounded, private recovery journal after its
pre-snapshot and proposed audit record, before changing the working SVG. The journal binds the
exact document/operation/snapshot identity and before/after SHA-256. Startup recovers prior
uncommitted edits before retention; unresolved evidence prevents pruning that document.
Recovery restores verified pre-edit bytes only when the head still matches the journal's after
hash, or resolves an already unchanged head. A durable applied audit is the commit marker and
preserves a later legitimate head. Invalid identities, hashes, linked files, unrelated heads and
multiple journals refuse without guessing or replaying tool parameters. Historical operations
without journals are not retroactively rolled back. Recovery never launches Inkscape.

**177 Rust tests passed, one ignored child entry point**; a regular parent test invokes that
entry point in ten real child processes which exit immediately at publication boundaries.
Late edit/restore audit failures also exercise rollback and subsequent recovery. Format,
all-target clippy and locked release build pass. Ruff lint/format (**312 files**) and strict
mypy (**122 sources**) pass. These checks do not establish arbitrary power-loss safety,
every fsync failure or protection against concurrent external writers. Windows remains unported.

Real STDIO recovery acceptance passes **13 profiles / 26 starts** on both the fresh binary and
stage13 archive (`migration/recovery-comparison.json`,
`migration/recovery-packaged-stage13-comparison.json`). The startup fixtures are synthetic
durable states, distinct from the Rust child-exit tests of actual pipeline publication.
Complete raw file inventories, originals/external bytes, audit/journal state and repeated
startup are retained. Fresh ordinary parity also passes: **122 edit, 50 save and 24 atomic-batch
member observations, zero differences** (`edit-recovery-comparison.json`,
`save-recovery-comparison.json`, `batch-members-recovery-comparison.json`). All **16 discovery
configurations match exactly** on the packaged binary (`discovery-recovery-stage13-comparison.json`).

The current archive is `migration/results/packages/inkscape-mcp-macos-arm64-stage13.tar.gz`:
**46,301,370 bytes**, SHA-256
`e2c254776fb672bf28800d14e32f68d62d361b69632a182483901f1e71cb04a8`.
It includes startup retention and recovery, with 2258 regular files / 130,110,511 regular bytes.
Actual archive installation into an owned empty-PATH environment passes cold and warm CLI,
private Python/helper/bus and true STDIO checks (`package-stage13-cold-comparison.json`,
`package-stage13-shell-comparison.json`). Ten read-only doctor profiles pass again.
`package-build-comparison.json` now points to stage13; older reports remain historical.

The unmodified stage13 package passes **46 native GUI checks**, including insertion,
style/repeated-style and duplication with exact one-step Undo/Redo, anonymous native IDs,
reconnect binding reset and approval/unbound guards. Immediate live audit readbacks are captured
before reconnect, which clears live history in both implementations. Repeated style emits two
applied records with distinct IDs while adding no extra native Undo step. The style targets the
wrapper; an explicit child fill keeps its visible pixels unchanged, so this is attribute Undo
verification. `native-stage13-comparison.json` and its captured-only repeat retain 24 SVG/PNG
artifacts and raw replies. No uncertain mutation was retried.

Four owned synthetic sessions have been opened: stage5 and stage6 have verified native closure;
stage13 remains open. Stage12's close attempt is **unconfirmed**: the UI reported no windows,
but private readback still lists its original document/window IDs and both owned PIDs remain
live. Its corrected session metadata preserves the contradictory observations. Stage13 was
launched after a failed stage12 PID assertion because the command did not stop on that failure;
the two private sessions remain separate. Preserve both, and do not infer closure or target
another window by bundle ID alone. No process-name kill or user-window close is authorized.

Five alternating sequential Python/stage13 warm runs are saved in
`results/process-benchmark-shell-stage13/`; complete 1/100-object inspection envelopes match
in all five pairs. `process-benchmark-shell-stage13-comparison.json` retains environment,
commands, raw wire traces, CPU/physical IO and sampled server/descendant memory.

| Operation | Python stage13 ms | Rust stage13 ms |
|---|---:|---:|
| Actual STDIO startup | 792.11 | 7.42 |
| First edit including worker startup | 433.06 | 497.17 |
| Subsequent atomic batch | 8.84 | 70.87 |
| Subsequent preview | 4.33 | 12.10 |
| Subsequent PNG export | 5.55 | 13.07 |
| Inspect 100 objects | 26.37 | 4.33 |

Median sampled tree RSS during preview is **253.34 MiB Python / 169.58 MiB Rust**;
there are zero sampler errors and maximum observed gaps about 25.48/25.39ms. Summed RSS includes
shared pages and can miss peaks. No cache flush, controlled cold start or exclusive wall-phase
attribution was performed. The two retained GUIs are outside the sampled benchmark subtree.
Rust warm edit/render/export remain slower; journal IO cost is not causally isolated from run
conditions. Live mutation benchmarks, helper-to-Rust implementation/acceptance, full argument/error parity,
other-platform packages and signing/ABI/license work remain pending. The goal is incomplete.

### Prepared POSIX Rust CI validation (2026-10-02)

`.github/workflows/rust-migration.yml` now prepares four native validation jobs: Linux GNU
x86_64/arm64 and macOS x86_64/arm64. It verifies each runner's actual Rust host target, installs
Rust 1.99.0 with format/clippy, uses locked dependencies, runs all native tests, builds release,
and compares all 16 real STDIO discovery configurations to the frozen contract. It records
executable SHA-256 and dynamic dependencies, retaining an explicitly named validation executable,
Cargo.lock and discovery/build evidence. Repository permission is read-only. Checkout and artifact
actions are pinned to verified official tag commit SHAs; no workflow was pushed or triggered.
Runner labels are checked against [GitHub's hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

The YAML parses locally and the same current-Mac format/clippy/175-test/release checks pass;
the fresh release discovery capture matches **16/16 configurations exactly**
(`migration/discovery-startup-retention-comparison.json`). The discovery comparator now selects
the 16 exact gate/profile/description filenames, because the contract directory also contains
private `live-*.json` protocol fixtures. It still compares every response field unchanged.
**Actual remote CI runs: zero.** The local Docker daemon is unavailable, so no local Linux container
result is claimed. The prepared jobs produce validation executables, not complete installable
helper/runtime/bridge/bus packages. Linux packaging, Intel Mac runtime/ABI acceptance and the
Windows no-follow reparse/rename-handle port/build jobs remain pending. This does not satisfy the
full other-platform release requirement. `migration/rust-ci-prepared-comparison.json` records
this scope separately from executed acceptance.

### Startup retention in the current source (2026-10-02)

The native MCP entry point now runs the reference's explicit boot-time retention pass before
serving STDIO. It enumerates persisted document directories independently of the new empty
registry and reuses the bounded snapshot/record/live-frame retention kernels. Directory and
index discovery use descriptor descent with no-follow checks; child symlinks and nonregular
indexes are skipped. Roots without managed state stay untouched. A malformed or inaccessible
document is isolated from its siblings and other roots; failures are summarized on stderr only.
The doctor path returns before maintenance, and no GUI/engine is launched. This pass is limited
to startup or the existing explicit `prune_snapshots` tool; mutating tools do not trigger cleanup.

**175 Rust tests**, format and all-target clippy pass. Three new tests verify cleanup of prior
session state without registration, preservation of original/working bytes, idempotence, an
empty root remaining unmodified, malformed/link siblings, continuation across roots, and live
frame versus selection-export preservation. `migration/startup-retention-environment-comparison.json`
records **two profiles / eight real Python/Rust STDIO starts**, with fixed IDs and complete
before/after filename/hash inventories. Only valid snapshot-index JSON whitespace/key ordering
is canonicalized; all other bytes are compared exactly. Bounded and disabled policies match,
including linked/unlinked valid records, malformed records/indexes, protected live previews,
frame caps and repeated startup. Commands/settings and raw wire/stderr traces are retained under
`migration/results/startup-retention-environment/`. Native symlink protection is tested separately,
not by asking the Python reference to follow hostile fixtures.

The explicit-retention acceptance rerun on the fresh release binary passes **12 observations /
zero differences**, plus native tampered-basename/symlink guards
(`migration/retention-startup-comparison.json`, `migration/results/retention-startup-rerun/`).
At this historical 175-test checkpoint the stage12 archive predated startup retention.
The current stage13 archive above now includes retention and targeted recovery. Full
snapshot-index schema/error compatibility and other-platform protection remain pending;
the boot pass alone does not complete retention guarantees.

### Readiness polling, packaged native Undo/Redo and stage12 (2026-10-02)

Stage12 replaces the warm worker's fixed 5ms command/write polling interval with bounded
`poll(2)` readiness waits. EINTR retries keep the original deadline; invalid descriptors fail;
closed stderr is removed from subsequent watches to avoid spinning on EOF. Fragmented prompts,
closed stderr and deadline/error handling have regression coverage. The shutdown wait still has
its separate bounded child-exit polling. All **172 Rust tests**, all-target clippy and format pass.
Real cold/warm PNG/SVG equality, freshness, one-worker reuse, startup/open no-spawn and exact
owned-child cleanup pass again (`migration/engine-poll-comparison.json`); a failing shell startup
falls back to the fixed CLI. Real shell path parity passes **170 observations / zero differences**
(`migration/path-shell-poll-comparison.json`). Previous Python full QA remains **1289 passed,
12 skipped** with actual Inkscape; this change does not modify Python production code.

Five fresh alternating sequential Python/packaged stage12 warm runs are preserved in
`migration/results/process-benchmark-shell-stage12/`. Complete 1/100-object inspection envelopes
match in all five pairs, normalizing only bound document IDs and decoding JSON text content.
`migration/process-benchmark-shell-stage12-comparison.json` records raw environment/commands,
wire traces, CPU, physical IO, sampled server/descendant memory and prior stage11 summaries.

| Operation | Python stage12 ms | Rust stage12 ms | Earlier Rust stage11 ms |
|---|---:|---:|---:|
| Startup through actual STDIO initialize | 696.25 | 6.80 | 6.73 |
| First edit including warm worker startup | 399.43 | 442.00 | 471.89 |
| Subsequent atomic batch | 7.20 | 56.91 | 93.77 |
| Subsequent whole-document preview | 3.94 | 11.27 | 28.31 |
| Subsequent PNG export | 4.80 | 11.87 | 29.92 |
| Inspect 100 objects | 24.93 | 4.08 | 6.81 |

Median sampled tree RSS during preview is **252.56 MiB Python / 165.86 MiB Rust**. Both trees
include the persistent Inkscape worker and shared pages; these are summed sampled values, not
unique allocated pages or continuous peaks. These stage12 runs have zero sampler errors and
maximum actual sample gaps about 25.33ms; historical large-response runs have longer gaps.
The first Rust startup is 437.96ms versus the 6.80ms median, so these five runs do not characterize
cold startup. Binary versions and run conditions differ across checkpoints; there was no cache
flush or exclusive server/IPC/filesystem wall-time attribution. Rust remains slower than Python
for the warm edit/render/export cases. The observed reduction after readiness polling is scoped
to these retained runs; no general speedup or native live performance claim is made.

At this historical checkpoint the current-Mac archive was
`migration/results/packages/inkscape-mcp-macos-arm64-stage12.tar.gz`: **46,256,010 bytes**, SHA-256
`9b8dd952a78909ebf4864f50c8a2ed0be1e8c5785ee6c4d30f496ce57befdc42`.
2258 regular entries / 129,994,671 regular bytes. Clean archive installation with empty PATH
passes both per-call and shell acceptance (`migration/package-stage12-cold-comparison.json`,
`migration/package-stage12-shell-comparison.json`), including private runtime/helper CLIs/bus,
110/7/18 discovery, actual PNG/export resource, approval, no-op, mixed-batch rollback/restore and
source preservation. Ten read-only doctor profiles pass. `migration/package-build-comparison.json`
now points to stage13; the stage12 and prior stage11 reports are preserved separately. No user configuration changed.

**Fresh native acceptance now passes on the unmodified stage12 package**, official Inkscape
1.4.3, isolated profile/workspace/private bus, empty PATH and prebuilt context bridge. Native UI
Edit → Undo/Redo actions were used; disk-save substitutions were not used. The recorded document
and window IDs were checked before each mutation, with one known window on the owned bus.
`migration/native-stage12-comparison.json` verifies **41 checks**: anonymous inner group receives
Inkscape's `g1` ID while planned IDs/geometry survive confirmation; one Undo removes the entire
inserted group and Redo restores its exact scene tree/RGBA; a changed wrapper fill plus identical
repeat needs only one Undo; style Redo restores the exact tree; duplicate preserves the original,
creates three unique IDs and has exact one-step Undo/Redo; reconnect clears binding; absent
binding or approval refuses editing and preserves tree/RGBA; official vendor executable hash
is unchanged. The explicit blue child fill overrides the orange wrapper fill, so this style case
checks a real attribute/Undo change with unchanged pixels, not a visible recoloring claim.
The insertion itself has verified visible pixels. This is a scoped native proof, not coverage of
all live effects, uncertain outcomes or fault paths. Approved live calls create Operation Records; the frozen connect/disconnect flow clears live
history. This native report does not assert audit persistence across reconnect. The verified
no-op guarantee here is absence of an extra native Undo step.

Raw replies/traces and **24 captured SVG/PNG artifacts with hashes** are retained under
`migration/results/native-gui-stage12/`; read-only verification is reproducible without GUI:

```bash
.venv/bin/python scripts/migration_verify_native.py --output migration/results/native-gui-stage12 --report migration/native-stage12-captured-comparison.json --captured-only
migration/results/packages/inkscape-mcp-macos-arm64-stage12/bin/inkscape-mcp --doctor
```

At the stage12 checkpoint, three owned sessions had been opened: stage5 and stage6 were closed
through native UI; stage12 was retained. The current stage13 section records later closure uncertainty. Before closing stage6, new stage12 readback confirmed
its exact document/window IDs and saved tree/canvas. Its exact owned GUI/supervisor PIDs exited and
session manifest disappeared. The earlier stage6 uncertain insertion was never retried or
retroactively marked successful; its failed evidence remains. User windows were untouched.
Full migration remains active: other target ports/build jobs, crash retention/failure handling,
additional schema/error/security and native effects coverage, helper-port investigation,
signing/ABI/license audit and deeper phase/live benchmarking are still required.

### Native warm shell engine and stage11 (2026-10-02)

`rust/src/engine.rs` now implements the private headless `inkscape --shell` transport. It is
selected only by explicit operator `INKSCAPE_MCP_ENGINE_MODE=shell` (trimmed/case-insensitive);
default/unknown modes use the existing fixed CLI path. Startup/open/reconnect do not create a
worker. The pool serializes commands, bounds workers with the existing configurable capacity,
evicts LRU/idle/dead workers on access, invalidates faults, and shuts down its exact owned
children when MCP STDIO closes. It never kills processes by name or operates on a GUI window.
Only typed approved internal kernels reach this transport; no arbitrary execution capability
or new public tool/argument was added.

Whole-document PNG/SVG render/export, approved path/Action kernels and operation before/after
previews use the pool. Object/region/PDF exports retain the fixed per-call path, following the
reference's sticky-option limits. Every operation closes its previously loaded private document
and opens a new descriptor-validated operation-owned staged copy. This deliberately avoids
passing mutable workspace paths directly to the shell and avoids caching stale in-memory edits.
Export type, plain SVG, intrinsic/explicit width and page area are set explicitly. Paths with
semicolon/newline/carriage-return are refused for shell framing and use the safe argv fallback.
Framing drains stderr before accepting the bare prompt; unknown-action errors, crashes, missing
outputs and timeouts select CLI fallback on the private staged input. Command/pipe buffers are
bounded (1 MiB / 64 KiB). Bounded safe SVG parsing precedes accepting Action output. Existing
post-export size/pixel checks and descriptor-protected artifact adoption remain in effect.

Real shell comparisons: render/export **74 observations**, path tools **170 observations**,
zero differences after the reference bug fix below. `migration/engine-native-comparison.json`
records real cold/warm PNG dimensions/RGBA and SVG byte equality, sticky width reset, fresh pixels
after edit, one reused owned worker, no Inkscape spawn at initialize/open, and exact worker
cleanup when MCP closes. Its fixed shell-startup failure case proves CLI fallback with identical
artifacts. Rust fault tests cover unknown-action stderr, EOF/crash, output flood, timeout, newline
and separator refusal, reuse/LRU/idle eviction. All **171 Rust tests**, all-target clippy/format,
Ruff (**307 files**), mypy (**122 sources**) pass. Full Python QA with actual Inkscape on PATH:
**1289 passed, 12 skipped**; these are automated/CLI checks, not native GUI acceptance.

The initial cold/warm harness incorrectly counted two operation-preview CLI calls as public
export fallbacks; its failed evidence is retained in `migration/results/engine-native-diagnostic/`.
The corrected check identifies fixed public export staging, and operation previews subsequently
moved into the same warm pool. A separate first path comparison exposed a real Python reference
bug: `engine_run_actions` left an unadopted mutated document cached, so the pipeline's
`preview_before` sometimes depicted the mutation instead of the unchanged disk SVG. The shared
Python reference now invalidates `opened_path` in a `finally` block after the action/export.
The regression proves an actual file-open occurs before a preview even when disk mtime/size did
not change. Original failed differences remain in `migration/path-shell-comparison.json`;
`migration/path-shell-fixed-comparison.json` and the final operation-preview-routing rerun
`migration/path-shell-preview-comparison.json` each pass 170 observations. No pixel differences
were normalized away. This is an explicit correctness fix, not a claim that the faulty historical
reference previews matched Rust. The Python reference remains available.

Five alternating sequential warm-mode Python/packaged stage11 runs are in
`migration/results/process-benchmark-shell-stage11/`; summary and complete inspection-envelope
checks are in `migration/process-benchmark-shell-stage11-comparison.json`. These runs inspect
1/100 objects and measure edit/batch/CLI exports; they do not replace the historical larger
stage9 `per_call` traversal measurements. Median warm roundtrips:

| Operation | Python ms | Stage11 Rust ms |
|---|---:|---:|
| First single edit, including first warm worker start | 387.05 | 471.89 |
| Subsequent atomic batch | 7.01 | 93.77 |
| Subsequent whole-document preview | 3.79 | 28.31 |
| Subsequent PNG export | 4.69 | 29.92 |
| Inspect 100 objects | 24.65 | 6.81 |

During warm preview, median sampled descendant-tree RSS is **253.36 MiB Python / 167.16 MiB Rust**.
Startup median is **889.13 / 6.73 ms**, with a first Rust startup outlier around **985 ms**;
these five runs do not characterize a controlled cold-start distribution. The sampler now also
records CPU deltas for descendants still live at request boundaries, because root reaped-child
CPU counters exclude a running persistent worker. Raw Mach counters/timebase, timestamps,
commands/environment, replies and actual sample gaps are preserved. The existing sampling,
shared-page, cached IO and non-exclusive phase-attribution caveats apply. Warm Rust is slower
than warm Python in these edit/render cases; the 5ms nonblocking polling interval adds framing
latency and is a concrete optimization candidate. Historical per-call measurements differ in
binary/version/conditions and are not presented as a controlled same-binary speedup. No overall
speedup or GUI/live performance claim is made.

Stage11 archive: `migration/results/packages/inkscape-mcp-macos-arm64-stage11.tar.gz`,
**46,261,890 bytes**, SHA-256
`10f9e6c0d290159a3543565f14be54abc9d5e21a315e819831d69ddc6284c53a`.
2258 regular hash entries / 129,994,735 regular bytes. Archive-install with empty PATH passes
in both `per_call` and `shell` modes (`migration/package-stage11-cold-comparison.json` and
`migration/package-stage11-shell-comparison.json`), including private runtime/helpers/bus,
110/7/18 discovery, actual PNG/export resource readback, mixed batch rollback/restore, approvals,
no-op and original preservation. Ten doctor profiles pass. `migration/package-build-comparison.json`
now points to stage11 and preserves the stage9 record separately. Stage10 is a retained
intermediate build before operation-preview routing; it was not promoted as the latest candidate.
Run `migration/results/packages/inkscape-mcp-macos-arm64-stage11/bin/inkscape-mcp --doctor`.
No user MCP configuration changed. Fresh native GUI acceptance, other-platform ports/builds,
retention/crash handling, deeper phase attribution and remaining schema/error/security coverage
are still required; the full migration goal remains active.

### Direct native tool results and stage9 package (2026-10-02)

Native tool success/error replies now build the rmcp `CallToolResult` directly, moving the
structured JSON value instead of placing it in an intermediate JSON envelope and deserializing
that envelope again. Text content, explicit structured null, error flags, omitted result type
and the frozen boolean `fastmcp.wrap_result` metadata are preserved. The SDK's constructors are
used because its result struct is non-exhaustive; constructor defaults are adjusted to the
frozen shape. A focused scalar/array/object/null/boolean/error wire-shape regression passes.
**169 Rust tests**, all-target clippy and format pass. The freshly built release binary passes
**71 read comparisons with zero differences** (`migration/read-direct-result-release-comparison.json`).
An earlier comparison used the previous debug executable because `cargo test` rebuilds the test
binary; that report is explicitly labeled as prior-binary evidence, not validation of this change.

Five additional sequential, alternating Python/stage9 real STDIO repetitions are retained in
`migration/results/process-benchmark-stage9/`. At 100,000 objects, median sampled server RSS
is **1224.48 MiB Rust / 1096.53 MiB Python**, versus **1482.12 MiB** for stage8 Rust: about **17%**
lower sampled Rust peak. Median inspection roundtrip is **2059.01 ms Rust / 16196.84 ms Python**,
versus **2337.03 ms** for stage8 Rust. Stage9 startup median is **20.19 ms Rust / 936.75 ms Python**.
This improves a demonstrated allocation drawback, but Rust still peaks above Python for this
large full response. Sampling gaps, cached IO, background scheduling and client work limit
precision; no exclusive wall-time or overall speedup claim follows from these measurements.
Stage8's separate instrumented CLI phase observations remain historical and are not presented
as new stage9 diagnostic measurements. No native GUI/live timing is claimed.

`migration/process-benchmark-stage9-comparison.json` records the five paired runs, actual sample
gaps and SHA-256 of **complete inspection result envelopes** for the 1/100/10,000/100,000-object
fixtures. Every repeated Python/Rust result matches, including duplicated JSON text; object and
flat tree counts prove the large result was not truncated. Only actual `doc_id` fields bound by
requests are normalized and JSON text is decoded; no result fields are discarded. Reproduce
with `scripts/migration_verify_large_response.py --output migration/results/process-benchmark-stage9
--prior-summary migration/results/process-benchmark-stage8/summary.json
--report migration/process-benchmark-stage9-comparison.json`. The benchmark harness retains all
raw wire replies and samples. This check is fixture evidence, not full schema/error acceptance.

The new archive is `migration/results/packages/inkscape-mcp-macos-arm64-stage9.tar.gz`,
**46,253,475 bytes**, SHA-256
`4b3f4cef755718f6879604a6ade6bb72ece675afc64631fbdcdc6e74080a0da0`.
Its directory contains 2258 regular hash entries / 129,994,527 regular bytes. Archive-install
acceptance with empty PATH passes: private runtime/helpers/bus, 110/7/18 STDIO discovery,
actual CLI PNG pixels/export readback, approvals/no-op, mixed batch rollback/restore and original
preservation. All ten doctor profiles pass. `migration/package-build-comparison.json` points to
stage9; `migration/package-build-stage8-comparison.json` preserves the previous build record.
Run `migration/results/packages/inkscape-mcp-macos-arm64-stage9/bin/inkscape-mcp --doctor` for the
read-only prerequisite check. No user configuration was changed. Stage8 and earlier packages
remain available. Fresh native GUI confirmation, other platform ports/builds, warm engine,
retention/crash handling and remaining compatibility/security coverage still require work;
the full migration goal remains active.

### Stage8 real STDIO process and CLI measurements (2026-10-02)

`scripts/migration_process_benchmark.py` now benchmarks the packaged optimized stage8 binary
against the preserved Python console entry point, sequentially with alternating order across
**five repetitions**. Each run uses a fresh owned temporary HOME/profile/workspace, live disabled,
full descriptions/profile, raw Actions enabled and explicit `per_call` mode. No GUI is launched.
Raw JSON-RPC requests/replies, exact commands/environment, OS timestamps and per-process samples
are retained in `migration/results/process-benchmark-stage8/`. The comparison is
`migration/process-benchmark-comparison.json`; it is performance evidence, not a substitute for
contract/native acceptance. Original synthetic SVGs remain unchanged.

| Operation | Python median ms | Stage8 Rust median ms |
|---|---:|---:|
| Process start through real initialize | 1206.42 | 21.75 |
| Ping roundtrip | 0.41 | 0.04 |
| Inspect 100 objects | 27.29 | 8.76 |
| Inspect 10,000 objects | 1505.36 | 216.68 |
| Inspect 100,000 objects | 15486.26 | 2337.03 |
| Single edit | 698.04 | 753.57 |
| Atomic batch | 508.40 | 570.48 |
| CLI preview | 253.95 | 268.94 |
| CLI PNG export | 261.96 | 268.15 |
| Open small SVG | 5.74 | 21.80 |
| Save new SVG | 5.86 | 19.55 |

The measurements show faster startup/inspection, with slower small open/save/no-op and
CLI-backed mutations/render/export in this run. They do not establish an overall speedup.
Startup includes Popen, sampler setup and the initialize exchange; discovery has separate
measurements. Roundtrips include client JSON, pipes and scheduling. No cache flush or system
background workload control was used. Historical pilot timings remain historical; they are not
interchangeable with these full-surface packaged runs.

The sampler uses macOS `proc_pid_rusage(RUSAGE_INFO_V2)` and `proc_listchildpids`, observing only
its server and descendants, capped at 256 identities. It targets 20ms sampling and retains actual
timestamps/gaps, PID plus Mach start identity, RSS/physical footprint, server CPU, reaped-child
CPU and physical disk counters. Actual median gaps were about 25ms; maximum gaps were 338ms
(Python) / 256ms (Rust), so these observations must not be treated as continuous peak monitoring. CPU fields are converted using `mach_timebase_info` (125/3 on
this Mac), with raw ticks retained. A comparison against `time.process_time_ns` plus an exact
owned-child probe passes (`migration/process-sampler-validation.json`). The first smoke failed
because the Python package lacks a `__main__`; its stderr/partial trace is retained and the
console entry point is used in successful runs. An initial CPU-unit probe caught the Mach ticks
conversion error before the full benchmark.

Median sampled server RSS during initialization is **97.81 MiB Python / 11.72 MiB Rust**. During
single edits, median sampled total descendant-tree RSS is **251.22 / 165.88 MiB**; preview is
**234.28 / 150.66 MiB**. However, the large 100,000-object inspection peaks at **1184.39 MiB
Python / 1482.12 MiB Rust**. This is a demonstrated Rust memory drawback to investigate, despite
its faster traversal. These are sampled peaks, not exact absolute peaks: short children or peaks
between samples may be missed. Summed RSS includes shared pages and summed footprint is not a
unique-page measurement. Root child CPU counts can lag until reaping; physical disk counters
can be zero for cached IO and do not measure logical file IO.

A separate instrumented diagnostic run (`migration/results/process-benchmark-cli-stage8/`)
uses an owned fixed-vendor CLI timing wrapper. It observes six successful real Inkscape calls
per implementation: two for single edit, two for batch, one preview and one export. Vendor CLI
wall time in the single diagnostic trial is Python/Rust **667.29/664.10 ms** for single edit,
**502.80/510.57 ms** for batch, **248.78/256.23 ms** for preview and **249.01/250.53 ms** for export.
It records child CPU as well. Wrapper bootstrap adds overhead; these diagnostic numbers are
kept separate from direct performance trials. The outside-CLI remainder contains server,
filesystem, IPC and wrapper bootstrap; it is not an exclusive server wall-time measurement.
Ping, server CPU and OS physical IO counters provide additional observations without inventing
an exclusive wall-time split. Detailed filesystem/IPC attribution and live benchmarks remain
pending, along with large-response allocation reduction and the other full-goal requirements.

Reproduce direct measurements (use a new output directory):

```bash
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" .venv/bin/python \
  scripts/migration_process_benchmark.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage8 \
  --output migration/results/process-benchmark-stage8-new --repeats 5
# Separate diagnostic trial; never mix this with direct timing comparisons:
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" .venv/bin/python \
  scripts/migration_process_benchmark.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage8 \
  --output migration/results/process-benchmark-cli-stage8-new \
  --repeats 1 --counts 100 --diagnostic-cli
```

### Complete batch-member dispatch and shared kernels (2026-10-02)

`apply_edits` now dispatches all **33 typed variants** in the frozen/reference union. The two
remaining members, `repeat_objects` and `replace_svg_fragment`, reuse their standalone kernels;
fragment parsing/allowlisting/ID validation happens in the validate-all phase and its document
is retained for safe node lifetime. The ordered batch still uses the single transaction pipeline
with one snapshot/Operation Record, bounded member count and rollback. A fragment member raises
the whole batch to high risk, just like delete, and requires explicit approval even for a no-op.
`migration/batch-members-inventory.json` checks the frozen schema's names against current
reference/native dispatch; it proves inventory only, not complete malformed-argument parity.

`migration/batch-members-comparison.json`: **24 scenarios, zero differences** through real
Python/Rust MCP STDIO. Complete replies, raw SVG hashes (with narrowly validated minted copy
IDs only), snapshots/bytes, records/policy/approval, restore results and actual CLI PNG RGBA
pixels are compared. Cases cover linked/copy/tangent/seeded repetition; standalone and canonical
no-op fragment batches; ordering, targets created earlier in the same batch, approval refusal,
eager invalid-value refusal before audit, late missing targets/reference conflicts, stylesheets,
locked ancestors, singular parents and byte-exact rollback/restore. A first singular fixture
incorrectly tested a singular source instead of its parent; its failed trace is retained.

These fixtures also exposed a namespace-prefix serialization mismatch: a detached qualified
SVG group consumes `ns0` before its label receives `ns1` in the reference. The repeat kernel now
reuses an existing Inkscape prefix or starts new label prefixes at `ns1`. It avoids shadowing
existing namespace bindings, and a serialized/reparsed regression proves the group remains SVG
and the label remains Inkscape even when the root already uses `ns1` for SVG. No namespace
normalization conceals this difference. The attempted raw detached-node reconciliation did not
reuse ancestor prefixes correctly and failed checks; it was replaced by the validated bounded
prefix helper. Standalone comparisons rerun: repeat **19 scenarios**, fragment **131 observations**,
zero differences. All **168 Rust tests**, all-target clippy/format, Ruff (**304 files**) and mypy
(**122 sources**) pass. LLM indexes regenerated unchanged at 14121/149076 bytes.

Stage8 was built from the current optimized native binary, with the fixed private helper/runtime,
bridge and bus assets. Its archive is `migration/results/packages/inkscape-mcp-macos-arm64-stage8.tar.gz`
(**46,258,464 bytes**, SHA-256
`95ae48208521bb325620e4bfe090e7b2079d844a5dcfc4085156b9713a752cb6`),
2258 regular hash entries / 130,027,503 regular bytes. Archive-install and ten-profile doctor checks pass, including a new installed-artifact mixed
repeat/fragment batch, late failure rollback and snapshot restore. Run
`migration/results/packages/inkscape-mcp-macos-arm64-stage8/bin/inkscape-mcp --doctor`
for read-only diagnosis; normal MCP execution uses the same binary without that flag. Native GUI acceptance still awaits manual Mac unlock; retain/revalidate the
owned stage6 session. Full schema/coercion/error coverage, native effects/guards, warm-shell
execution, retention/crash recovery, Windows protections/platform builds and phase/peak-memory
benchmarks remain required. This is progress toward the full migration, not completion.

### Native anonymous IDs and stage7 confirmation fix (2026-10-02)

The stage5 synthetic document was closed through its native close/save prompt after private-bus
identity checks; its exact GUI/supervisor PIDs exited normally. No process was killed. A fresh
stage6 managed session then launched from the unmodified package with empty PATH/private
profile. Inserting a named semantic group without an explicit group ID applied successfully,
but Rust reported completion uncertain: Inkscape assigned `id="g1"` to the anonymous group,
which the exact-ID-only confirmation rejected. The mutation was never retried. Read-only SVG,
scene, wire and discarded/uncertain record evidence is retained in
`migration/results/native-gui-stage6/`; `migration/native-stage6-comparison.json` is **failed**.

Rust now confirms the exact original preorder/hierarchy and element namespaces/types, every
remapped source ID in its original position, a single direct root wrapper, and globally unique
IDs. It permits a native-assigned ID only at a position originally lacking an ID. Extra nodes,
reparenting, reordering, missing/wrong remapped IDs, duplicate IDs inside/outside the wrapper
and indirect wrappers refuse. Planning/allowlisting/context guards and uncertain/no-retry
handling remain in place. A regression covers native `g1`/`circle1` assignment and those
adversarial cases. All **166 Rust tests** and clippy/format pass. Managed public comparison again
passes **55 observations** with only seven explicit runtime-version identity differences.

The new candidate archive is `migration/results/packages/inkscape-mcp-macos-arm64-stage7.tar.gz`
(**46,261,127 bytes**, SHA-256
`84d7a1df4cbe04fb4e7bfba2cdad51163eb07cf656629b43ab785d15bd291d3e`),
with directory `migration/results/packages/inkscape-mcp-macos-arm64-stage7/` and 2258 hashed
regular files totaling 129,994,367 bytes. Clean archive-install acceptance passes runtime/helpers,
private bus, frozen 110/7/18 STDIO discovery, no-op, batch/rollback/approval, source preservation,
SVG/resource and real CLI PNG pixel checks. Ten-profile read-only doctor acceptance also passes. To check this candidate, run
`migration/results/packages/inkscape-mcp-macos-arm64-stage7/bin/inkscape-mcp --doctor`.
Fresh stage7 GUI confirmation remains **unverified**:
macOS locked before the stage6 window could be closed. The owned stage6 document/session is
retained; do not relaunch or kill it based on a stale record. Revalidate its manifest/context/PIDs
and use its exact application path (the old/new apps share a bundle ID). The user has been asked
to unlock manually; independent work may continue. The native harness now accepts `--output`
to preserve each session's evidence and still refuses reuse of an existing ownership record.

`contracts/` contains complete initialization and discovery wire results captured through
the installed `.venv/bin/inkscape-mcp` STDIO entry point for all 16 combinations of live,
advanced Actions, profile and description settings. These include instructions, defaults,
input/output schemas, annotations, prompt arguments and resource/template descriptors.
They contain metadata only; the Rust implementation does not invoke the Python server.

`discovery-comparison.json` compares every field and list position without normalization.
`read-comparison.json` compares native workspace/open/summary/read errors; its only
normalizations are actual discovery/open ID bindings and decoding JSON text payloads.
The index also binds its validated malformed-open document ID and concrete resource URIs.
No semantic result field, MIME type or error message is ignored. There are now 71 scenarios,
including all seven document resources, aggregate inspection and validation.

`find-comparison.json` covers 45 filtered-search observations through actual MCP STDIO:
authored ObjectRefs, AND filters, CSS/inherited paint matching, Unicode text, inclusive region
intersections and real CLI accurate bounds. Isolated absent/failing/timed-out/synthetic CSV
engines prove DOM fallback, exactly one query per accurate call, malformed/duplicate/negative/
zero/nonfinite CSV behavior, and unchanged source/working SVGs with no history writes.
`results/find-acceptance/native-output-cap.json` separately proves bounded stdout falls back
to identical DOM results; this is a Rust safety check, not Python parity. Raw responses/logs
are retained. One earlier repeat timed out at Rust initialization without stderr; its cause
is undetermined. Subsequent complete repeats passed. Read's 71 scenarios were rerun after
sharing ObjectRef extraction. More argument/error/security edges remain pending.

`contracts/bus-variant-cases.json` captures 25 exact guarded GVariant string/bool/double/empty
values/errors (`.venv/bin/python scripts/migration_live_bus_reference.py`). Private native Bus
uses fixed action/method/context/target types, bounded gdbus argv and stdout, UUID identity and
shared unique-owner parsing. Broker lookup precedes Actions.List; every application/context/
introspection call targets that pinned unique owner, never a well-known application name. Four
Rust tests inspect fake CLI traces: missing/bad owners stop after broker; all fixed methods and
window/context identity params stay bounded; timeout/nonzero/oversized mutation replies are
uncertain, quarantine the owner and never retry. ContextChanged keeps the owner and exposes only
fixed recovery text. Bounded stderr is retained internally to classify that fixed bridge error.
These stronger semantics are documented, not normalized into reference equivalence. Initial
200 ms fixture connect timed out (cause unproven); production-floor 1 s with 2 s mutation stall
passes. All 98 Action-discovery comparisons reran after shared stderr retention with no differences.
Fake CLI is not real D-Bus/native GUI acceptance. Public/transport/managed
helper/packaging/native Undo remain pending; public count stays 84/26.

`contracts/context-reply-cases.json` captures 54 exact document-context/list parsing replies and
errors (`.venv/bin/python scripts/migration_live_context_reference.py`). Private live_context
implements a data-only string/tuple/list parser with a 1 MiB byte cap, depth/node bounds and
10,000-row cap; it never evaluates expressions. Escapes, adjacent strings, title annotation
normalization, empty names, UUID/error precedence and raw NUL refusal match these fixtures.
Unpaired surrogate/out-of-range escapes fail as invalid Unicode. Two parser tests and one fake
CLI integration test verify active/list replies through the unique owner (113 Rust tests total).
This does not claim exhaustive Python literal grammar or actual native bridge acceptance.

`contracts/dbus-backend-cases.json` captures 22 real Python DBus backend calls against a synthetic
process boundary (`.venv/bin/python scripts/migration_live_dbus_reference.py`), without bus/GUI.
Native Dbus implements the common Transport with plain-SVG and PNG exports, parsed active-document
metadata/count, window viewport and selection style/transform actions. Results/errors and action
traces are checked; only minted export filenames bind to `<EXPORT>`. Three fixtures explicitly
retain reference traces that Rust refuses earlier: pan avoids introspection, and two malformed
transform tails no longer apply earlier mutations. Native whole-plan validation also bounds argv
before the first mutation. Export reads use no-follow regular-file descriptors and input caps;
missing/symlink/oversize/bad XML/UTF-8 and owned-directory cleanup regressions pass. Synthetic PNG
bytes check transport only, not pixels or native Undo. Three new tests bring the total to 116.
Managed guards/helper/probes/public integration, actual D-Bus and packaging remain pending.

`contracts/task-guard-cases.json` contains 50 exact Python selected-task guard outcomes (regenerate
with `.venv/bin/python scripts/migration_live_guard_reference.py`), without GUI or bus. Private
Managed now owns scoped regular stdout and advisory lock descriptors opened through no-follow
workspace descent. Deadline-bounded flock coordinates another process; same-thread/same-stream
nesting shares context/handles, different sessions refuse nesting, and nesting caps at 16. Scoped
error/panic cleanup releases handles. Select/list and plain-SVG/PNG reads route all guarded actions
through UUID Context.Activate on the pinned owner. Tests simulate a context switch and verify no
fallback or retry, selected-task mismatch and disconnect reset. Legacy unguarded reads remain
available; entering a guarded nested scope without a captured context refuses before actions.
Five new tests bring the total to 121. Selection stdout fencing, scene, effect transactions, fresh
liveness/probes/public wiring and actual native Undo/package acceptance are still pending. This
partial private Managed advertises only its implemented read commands; public count remains 84/26.
After the shared regular-file handle change, all 71 headless read acceptance scenarios passed again
with zero differences through real MCP STDIO.

`contracts/selection-reply-cases.json` captures 58 exact Python managed-selection parser replies
and errors (`.venv/bin/python scripts/migration_live_selection_reference.py`). Native parsing
preserves numeric/Unicode fences, ordered dedup, partial-line and Python splitlines/strip behavior,
including ignoring trailing data after a complete fence. Native byte and 10,000 unique-ID caps are
separate safety guards. Managed selection seeks conceptually to the current end using positional
reads of the pinned regular stdout handle; it sends exactly select-list/query-x under one scope,
filters the last exported root ID and times out without retry. Fake CLI tests cover Unicode/empty,
duplicate/filter, malformed/incomplete/invalid UTF-8/oversize replies and scope cleanup. Managed
exports now reset sticky id/id-only/text-to-path/plain-SVG options before each export, retaining
metadata; region render uses shared root user-unit mapping. One mapped SVG/PNG action-trace test
checks the reset order, transformed area and a single captured context. Five new tests bring the
total to 126; these are internal/fake CLI checks, not native GUI/pixel/Undo or public-tool parity.
Full managed geometry/error fixtures, scene/inspection, effect transactions and public wiring remain.

`contracts/managed-scene-cases.json` has 140 exact Python scene/inspection models, regenerated with
`.venv/bin/python scripts/migration_live_scene_reference.py`. Private live_scene reuses shared
headless ObjectInfo/tree helpers and preserves attribute-derived geometry, ancestor-transform
bbox refusal, inline/presentation visibility, descendant visibility overrides, duplicate-ID lookup,
selected-ID order/duplicates/stale IDs, metadata, canvas/viewBox and unavailable viewport notes.
All fields compare without normalization. Native XML/input and 10,000 scene-element/selection caps
are tested separately. The same script regenerates `unicode-decimal-starts.json` (Unicode 15.0.0)
and 705 `numeric-text-cases.json` Python float cases: every frozen Unicode decimal block, signed
zero, exponents/underscores, invalid syntax, infinity/NaN and whitespace classification. Native
decimal text decoding is shared with attribute-prefix bbox parsing; it executes no Python at runtime.
Managed scene/inspection now obtain selection and SVG under one context, and only managed scene's
active document receives the captured window/document UUID. A fake CLI test checks selection→export
order, one context per call, fields and cleanup. Four new tests bring the total to 130. Actual
managed geometry/error, GUI/pixels/Undo, helper transactions, liveness/probes and public integration
remain pending; public count stays 84/26.
After sharing inspection/tree/decimal helpers, 71 read and 45 find scenarios reran through real
MCP STDIO with zero differences.

`contracts/effect-data-cases.json` captures 11 exact Python helper document fingerprints, 18 edit
reply validation cases and the fixed public refusal whitelist (regenerate with
`.venv/bin/python scripts/migration_live_effect_reference.py`). Native fingerprints preserve Clark
namespaces, sorted attributes, root UI exclusions, metadata/namedview filtering and mixed text/tail,
including comments/PIs/entities; JSON separators match the helper's hash input. Hashing streams
records and bounds expanded bytes at min(8×operator input cap, 64 MiB), with per-record guards;
namespace amplification refusal is an explicit native protection. Reply parsing verifies nonce,
strict boolean discriminator, fingerprint/ID types and refusal types, with malformed/stale/capped
data remaining uncertain. Known public refusals pass; unknown private detail falls back to the
fixed invalid-document/selection reason. Private Exchange atomically writes mode-0600 fixed request
files, bounds bytes and validates regular entries before mutation; replies use no-follow reads.
Cleanup is once-only, including Drop fallback, so an already-cleaned exchange cannot delete a later
request. Tests cover mode/data, stale cleanup, symlink originals, request/reply caps and uncertainty.
Five new tests bring the total to 135. This is an internal data/file kernel; Managed activation,
post-effect fingerprint confirmation, insertion planning, actual helper/Undo/GUI/package and public
dispatch remain pending. Public tool count stays 84/26.

`contracts/edit-transaction-cases.json` captures 13 actual Python managed edit request/result/
refusal transactions against a synthetic effect boundary (regenerate with
`.venv/bin/python scripts/migration_live_transaction_reference.py`). Native Managed now uses
the fixed edit effect, not document replacement: selected-task scope spans selection, pre-export,
nonce/expected IDs+fingerprint request, activation, reply and post-export fingerprint confirmation.
Style/text plus eight typed structural/order operations are internal; fill fallback remains for
legacy helper-absent sessions. Requests/results/refusals compare exactly after validating/binding
the minted nonce only. Another fake effect changes SVG and computes its SHA with Python helper
code; Rust confirms the changed fingerprint. Lost/stale/missing/mismatched replies and guarded
context switch stay uncertain, with one activation and cleanup. A valid refusal after lost
activation is recovered; an applied reply remains uncertain after unique-owner quarantine, an
intentional conservative difference from the reference's generic action reconciliation. Unknown
refusal details use the captured whitelist fallback. Four new tests bring total to 139. Initial
trace assertions included Describe in mutation counts; the switch fixture also used unguarded
legacy mode. Corrected method filters and guarded document selection pass the complete repeat.
These are fake effect tests, not native Undo/GUI/effect-runtime evidence.

`contracts/insertion-plan-cases.json` and `contracts/insertion-transaction-cases.json` regenerate
with `.venv/bin/python scripts/migration_live_insert_reference.py`.
Private Managed insertion now validates the original fragment natively, then activates only the
fixed insertion effect under a selected task and one operation scope. 64 frozen Python preflight
cases match allowlist, namespace/attribute/reference/CSS restrictions and planned IDs; 3 complete
Python transactions match requests/results with only validated minted nonce binding. Synthetic
insertion uses Python prepare_fragment to append a group; a fresh scoped SVG must contain exactly
one direct SVG root group with all planned IDs in order. This deliberately strengthens the legacy
substring confirmation. Native guards bound 1 MiB/10,000 elements and 2 MiB helper requests, refuse
empty IDs before dispatch (legacy uncaught KeyError), require exact nonce/boolean reply and classify
stale/malformed/oversize as uncertain. Lost/missing/mismatched/context-switch replies have one
activation and cleaned files/scopes; trustworthy refusal after lost activation is recovered, while
applied transport loss remains uncertain. Six new tests bring total to 145. This is fake CLI/effect
evidence, not actual helper/native Undo/GUI/package acceptance. Public counts remain 84/26. Next
complete fresh liveness/probes, public approvals/records/live dispatch/resources and package/native
acceptance.


`contracts/transport-probe-cases.json` regenerates with
`.venv/bin/python scripts/migration_live_probe_reference.py`.
Private live_probe now derives readonly D-Bus/managed/socket readiness and constructs fixed
attach-only backends for Session::connect. 256 frozen Python host/process profiles plus 4 socket
advertisement profiles match every readiness field/command list, with no action activation.
D-Bus handshake resolves the broker once, then targets the unique owner; managed helper support
uses fresh Describe. Managed is_connected now performs a fresh List with timeout capped at 2 s,
against that same owner, without reconnect or replacement. Vanished-owner/changed-helper and
post-disconnect fixtures pass. Managed stdout readiness refuses file/ancestor symlinks; this is
an explicit stronger native guard. No-session gates perform no subprocess calls. Factory tests
verify no process before attach, fixed D-Bus/managed document capture and Session teardown/history
callback. Socket readiness intentionally means a rendezvous advertisement, as in Python; connect
still requires the authenticated handshake. Four new tests bring total to 149. These are fake
CLI/process-boundary tests, not actual D-Bus/helper/GUI acceptance. Public counts remain 84/26.
Next wire config/probes/factory/Session and public live policy/approvals/records/tools/resources,
then complete private runtime/bridge/bus packaging, native Undo and end-user package acceptance.

`live-status-comparison.json` regenerates with
`.venv/bin/python scripts/migration_live_status_acceptance.py`; raw actual STDIO traces are under
`results/live-status-acceptance/`.
Public native check_live_support/live_status/live_disconnect and the live/session resource now
use Session, fresh native probes and cached runtime helper/data-directory facts. These readonly
paths never launch or attach a GUI. Disconnect resets transport/token/cache/task state and invokes
bounded best-effort first-root history cleanup. One no-follow cleanup test preserves original/link
and non-record files, and verifies idempotence; links are retained, an intentional stronger guard
than the reference glob unlink. Actual STDIO acceptance compares 8 isolated profiles (master gate,
rendezvous advertisement and helper installed marker), 48 complete tool/resource/file observations
with zero differences. Responses include every probe/command list, sorted ranking, exact status/
recovery notes, helper reconciliation, JSON MIME/text/structured content and repeated disconnect.
Fresh marker transitions after cache initialization also match: support/socket readiness uses fresh
installation presence while session reconciliation retains the cached runtime helper flag.
Public count is now 87 native / 23 pending tools; live/session is ported and four live resources
remain pending. 150 Rust tests pass. Public live_connect, managed manifest/environment refresh,
records/mutation/render/sync/events/resources, packaging and actual native acceptance remain.
One repeated full run alongside STDIO acceptance hit the synthetic gdbus 1 s connect timeout;
the cause is unproven. The isolated fault test and subsequent full 150-test repeat passed, without
changing deadlines. These STDIO profiles stay disconnected; connected public transport/guard
acceptance follows live_connect integration.


`live-connect-comparison.json` and `live-managed-comparison.json` regenerate with
`.venv/bin/python scripts/migration_live_connect_acceptance.py` and
`.venv/bin/python scripts/migration_live_managed_acceptance.py`. Raw STDIO traces are retained in
`results/live-connect-acceptance/` and `results/live-managed-acceptance/`.
Public live_connect, live_get_active_document/live_get_selection/live_inspect_selection and
managed live_list_documents/live_select_document now use the native Session/fixed factory. Managed
connect refreshes an existing owned 0700 session manifest through no-follow bounded reads, validates
the fixed private Unix bus address/optional GUID, probes the unique owner/context bridge, then
attaches without starting a daemon or GUI. Native address overrides are per-transport, with no
process environment mutation. Strict address tails refuse network/fallback addresses, an intentional
stronger native guard. Managed root paths must use ASCII alphanumerics or /._-; this prevents
D-Bus metacharacter/percent decoding from changing the endpoint. All fixed action-plan argv caps
include the address. Reconnect clears task,
token/cache and history; failed preference/attach leaves the documented state. live/selection and
live/view resources now use their transport with exact disconnected fallbacks.
Actual STDIO comparisons add 26 owned-loopback connected/read/reconnect/failure/resource observations
and initially 10 synthetic managed manifest/task/ready-to-edit/reconnect observations, matching Python.
The expanded managed comparison is described below.
Only validated UTC connected_at timestamps are bound, preserving reuse/new-connect relationships.
The managed test verifies explicit task choice and binding reset, but is fake gdbus, not native GUI.
Two attach/security tests bring total to 152; format/clippy/Ruff/mypy pass. Public count is now 93
native / 17 pending tools, with live/events and live/operations the two remaining resources. Next
complete events/render/cache/sync and approved live edit records/pipeline/remaining tools, then
package private runtime/bridge/bus and perform native Undo/Redo and package/benchmark acceptance.
Connected runtime capabilities now use the adopted transport address without process environment
mutation. Managed preflight runs before the first capability probe; an existing cache stays unchanged
until diagnose_runtime, matching Python. Disconnected read tools and selection/view resources do
not probe or fill that cache. Managed STDIO acceptance initially covered 41 observations across both
cache initialization orders, full runtime/resource fields, refresh after disconnect and no process
calls for disconnected reads. Seven explicit Python-version identity differences remain; all other
fields match after validated UTC timestamp bindings and semantic JSON decoding. These are owned
synthetic process-boundary checks, not native GUI or packaged-runtime acceptance.


`live-events-comparison.json` regenerates with
`.venv/bin/python scripts/migration_live_events_acceptance.py`; raw traces are in
`results/live-events-acceptance/`.

Public live_wait_for_change and inkscape://live/events now share Session's last-token/change
baseline via fixed state_token calls. First observations report no change; selection/document/
viewport deltas remain independent. A timeout carries the latest token with empty convenience
selection IDs without overwriting the persisted observation. Reconnect/disconnect reset the baseline.
The wait validates a 0..60 s budget, floors positive polling intervals at 10 ms, uses Tokio timers,
and releases the session mutex before sleeping. An actual MCP cancellation notification stops
subsequent polls; another resource read proceeds during a pending wait. This proves cancellation
between polls, not interruption of an in-flight synchronous transport exchange (its own bounded
process/socket deadline still applies).
23 actual STDIO observations match Python through an owned authenticated loopback peer, covering
shared tool/resource state, simultaneous deltas, defensive token coercion, zero timeout, bounded
interval-floor timeout, changes on subsequent polls, reconnect and helper refusal. Full negative
read replies preserve the synchronized socket channel without retry, matching Python; mutation,
malformed and lost-reply quarantine remains. Helper rejection has Python's exact tool/resource
error rather than being silently turned into an empty resource. Managed acceptance expands to
45 observations (seven retained runtime-identity deltas only), including inherited unsupported-token
refusal and clean events fallback. Nothing launches Inkscape or captures document/scene/PNG per poll.
Two new tests bring the total to 154. Public native count is 94 / 16 pending tools, with 17 native
resources and only live/operations pending. All seven prompts remain native. More schema-error
precedence/coercion cases, live render/cache/sync/edit approvals/records and packaging/native GUI
acceptance remain; neither synthetic peer nor fake managed bus proves native manual-edit detection.

`live-viewport-comparison.json` and `live-render-comparison.json` regenerate with
`.venv/bin/python scripts/migration_live_viewport_acceptance.py` and
`.venv/bin/python scripts/migration_live_render_acceptance.py`. Their raw traces are in
`results/live-viewport-acceptance/` and `results/live-render-acceptance/`.

Public live_set_viewport, live_render_view and live_get_scene now use the fixed native
Transport methods. Shared preflight bounds fixed zoom/pan/fit modes, paired center/deltas,
complete region parts, finite coordinates/extents and positive bounded scale; fast defaults to
0.5 while explicit scale wins. View control creates no document mutation, artifacts or records.
46 actual STDIO viewport observations compare results/refusals and fixed command parameters,
including before-connect validation and a valid helper refusal followed by another successful read.
Managed viewport support remains refused by its frozen command set, as in Python.
Live frame persistence uses the first workspace root, no-follow directory descent and atomic
publication of a minted timestamp/nonce PNG path. Caps refuse before file publication; a native
security test preserves an external symlink target and leaves no partial artifacts. Artifacts and
metadata are cached using the existing Session LRU/coalescing model with revision plus rounded
region/scale keys. Cache-token reads do not alter the events baseline; unavailable tokens skip
caching. live_get_scene scopes the frame and scene together through the transport operation scope.
60 actual STDIO render observations match Python, including full PNG byte and RGBA pixel hashes,
returned metadata/path reuse, rounded-key hits, LRU eviction, revision-change invalidation,
fast/explicit scale, region/scales bounds, invalid inputs before rendering and reconnect reset.
Only validated artifact timestamp/nonce paths are bound; all other result and request fields stay
compared. Synthetic managed acceptance expands to 51 observations, seven explicit Python runtime
identity differences only, including PNG persistence and guarded region/scene reads. Fake gdbus
writes a fixed valid PNG for PNG exports and the existing SVG fixture for document exports.
155 Rust tests, all-target clippy/format, Ruff (285 files) and mypy (122 sources) pass. Current
native tool count is 97 / 13 pending, with 17 native resources / live operations still pending.
These are synthetic public transport checks; actual GUI rendering, native Undo/Redo, packaged
helper/bus/bridge, remaining live mutation/record/sync/export/loop/diff tools, broader schema-error
parity and full package/performance acceptance remain. Python production files are unchanged.

`contracts/session-state-cases.json` captures 872 exact Python status/recovery models and twelve
cache transitions (regenerate with `.venv/bin/python scripts/migration_live_session_reference.py`).
No probe/socket/GUI is needed to capture these state models. Native common Transport/Session/Cache
kernels and Socket adapter compare every fixture without normalization: all five connection states,
guarded/legacy identity and recovery, helper reconciliation, LRU/count/byte/replacement/touch behavior,
coalescing boundary and reference single-oversize-entry exception. Other tests verify capability/
no-freeze ranking, teardown/reconnect/failure state cleanup, history-clear hook, scoped context end
on error/panic and actual loopback Session→Socket reads/reconnect. Six new Rust tests pass (106 total).
Public probes/config/records/cache-key/frame pipeline/event wait/managed guards/DBus/GUI/package
integration remain pending; 84/26 public tool count is unchanged. History clearing is currently an
injected hook, not proof of persisted live-record cleanup. RAII mock scope is not native context
acceptance. Internal connection factories attach only; launch remains a separate explicit action.

`contracts/socket-binary-cases.json` retains 29 exact strict base64 result/error cases from Python
(regenerate with `.venv/bin/python scripts/migration_live_binary_reference.py`). Missing/wrong/
malformed types, alphabet, control/Unicode, padding and nonzero padding bits are compared without
normalization. Native transport binary decoding does not prove PNG validity. Private socket
render/view/edit methods now cover the fixed semantic surface: four typed viewport variants,
optional render region/scale, string style map/composed transform, SVG/text and selection export.
A real TCP peer verifies each fixed payload and modeled result; another verifies nonfinite
parameters send no request and a lost insert reply stays uncertain without retry. Three additional
Rust tests pass. Public policy/approval/validation/records/task guards and native effect/Undo are
still pending, as are all 26 public live dispatches; do not treat internal methods as public tools.

`contracts/socket-model-cases.json` captures 658 defensive live-result/model fixtures, regenerated
with `.venv/bin/python scripts/migration_live_models_reference.py`. Native model tests compare every
field without normalization: document/selection/inspection/mutation/viewport, scene, Python scalar/
container strings and server-hashed revision/ordered-selection/coarse-viewport token. 256 seeded
float cases span exponents -300..300; signed zero/rounding/malformed/default/bool/Unicode and large
integer counts are included. Separate 10,001-item tests verify the reference caps before filtering.
Socket semantic reads now model fixed commands internally; scene identity is fetched separately
from get_active_document, ignoring the scene's spoofed path. A real TCP peer verifies command
order and authoritative identity. Three new Rust tests pass; public live tools/resources and
session/backends/pipeline/cache/event-wait/packaging/GUI acceptance remain pending (84/26 tools).

`contracts/socket-protocol-cases.json` captures 96 exact reference cases (42 encoded requests,
15 parsed response/error frames, 39 rendezvous scalar/version cases). Regenerate with
`.venv/bin/python scripts/migration_live_protocol_reference.py`; Rust kernel tests consume this
fixture directly, not a hand-written expected model. Private protocol v5/socket modules carry
all 14 fixed enum commands; there is no raw command string API. Seven Rust tests cover all commands
on real loopback TCP, token-bearing hello/every request, capabilities and disconnect; malformed/
rejected/wrong-version hello, truncated/malformed/oversize replies, no retry after transmitted
uncertain mutation, explicit rejection, and bounded slow trickle with view/mutation distinction.
No GUI or Python helper is launched. Bounded outbound serialization counts UTF-8/escaping/newline before writing and refuses before
allocating an oversized frame; exact-boundary/one-byte-under tests pass. Native TCP is not live MCP/native Undo acceptance: public
live tools/resources still return migration-pending, and counts remain 84/26. Temporary dead-code
allowances cover only these private kernels awaiting integration. Stronger native guards are
separate: filesystem-root no-follow rendezvous descent (file/ancestor link tests), min(input,
1 MiB) rendezvous and 4096-byte token bounds, reject trailing unsolicited frames, whole-request
deadline, quarantine the failed stream and never retry uncertain edits. Complete typed result
coercion, all backend/session/tool wiring and packaged helper/GUI/Undo acceptance remain pending.

`capability-comparison.json` retains 46 observations over every sixteen-profile gated registry,
real Inkscape and absent backend. Shared list_capabilities/diagnose_runtime/resource cache has
no probes on repeated reads; refresh replaces tool/resource fields after a fixture version change.
Exact sorted registry names/count/purpose/risk and all 49 intents match. Six fixed CLI/font probes
run per controlled cache fill. Core's hidden tools stay hidden; its resource initializes the cache.
Tool text/structured fields and application/json resource readback must agree; full raw wire traces
are saved. Only separately validated UTC timestamps are bound. All 28 matrix differences are
retained: Python's interpreter version versus truthful Rust `not applicable (native Rust MCP)`.
`exact_parity` is false, with zero unexpected differences; helper-runtime packaging remains pending.
A unit covers refreshed registry overlays without mutating the probe, sorting and purpose/risk.

`action-discovery-comparison.json` covers 98 observations over fourteen real/controlled runtimes.
list_actions/discover_extensions match complete reply fields/ordered notes, action and allowlist
order/intersection, compact/count semantics, boolean coercion and persisted map names/content.
JSON text is decoded without deleting fields; only separately validated UTC probe times are bound.
Real/absent/failed/empty/timed-out/bad-version/non-executable/signaled CLI cases, data/inkex version,
fonts, owned/unowned bus and operator allowlists are checked. Every case retains an opened original
and working SVG with no snapshots/records. Expected successes are mandatory.
`results/action-discovery-acceptance/native-probe-guards.json` separately proves no truncated
Action map under an 8192-byte output cap and bounded no-follow refusal for both inkex file and
ancestor-directory symlinks, retaining protected helper sources and document bytes. Sources are
read only, never imported. Native bus probes first query the broker for a unique owner and use
that unique destination for Actions.List, preventing well-known-name activation even after an
owner race. Controlled traces assert no named destination and no Actions.List when unowned;
this is mocked IPC, not GUI/native-session acceptance. One unit covers versions/export lists and
unique-owner grammar. Shared process binary lookup now requires X_OK and preserves real exit/
signal codes; Action's 110 common/32 fault observations passed again.
Packaged helper Python reporting, further parser/filesystem/failure/coercion cases and warm-shell/live/package work remain.

`action-comparison.json` covers 110 observations of validate_action_chain, run_action_chain
and run_raw_action. Operator allowlist, version capability map and fixed grammar all gate the
bounded 32×16 plan. Complete response/error fields, normalized plans/argv, source/working hashes,
records/snapshots, actual preview PNG RGBA and exact restore match. The advanced gate is explicit
and expected successful calls are mandatory. Persisted map filename/version/tuple/actions/count/
source are compared; the only additional normalization is a separately validated UTC probed_at.
Cache bytes must remain unchanged between calls. Two native file/directory map-link guards retain
outside bytes and prove persistence cannot escape the root while an in-memory plan still works.
`action-fault-comparison.json` retains 32 fault/no-op observations and all six semantic audit
status differences (Python proposed vs native discarded), requiring zero unexpected differences.
Absent Inkscape rejects at the map availability gate with no audit. Other fixtures use genuine
1-second timeout, failed/missing/empty/unsafe/oversized output and no-op; dry-run cannot launch an
Action execution, each real engine run has exactly one fixed argv invocation and private staging
is removed. Three separately recorded native output structure/reference guards refuse unchanged.
Two units cover independent gates, malformed/absent/argument error order, 32×16 bounds, comma hints,
fixed argv, action-list deduplication/order and traversal-safe version keys.
Validation map probing is native version/action-list only. Full schema/coercion/error parity, map corruption/staleness/races and warm-shell/GUI/package work
remain pending. Shared CLI refactor reran all 170 path and 32 path-fault observations successfully.

`path-comparison.json` contains 170 real observations across all seven advanced path tools.
The harness explicitly enables `INKSCAPE_MCP_RAW_ACTION_ENABLED=true` and requires successful
expected calls before comparison; an initial run which only matched hidden-tool errors was invalid.
Fixed actions/argv-safe IDs, default dry-run, deduplication and bottom merge identity, explicit
approval, complete replies/SVG hashes/history, actual PNG previews and exact restore match.
Stroke outlining restores fill from the original stroke and removes only newly added empty stubs.
Private engine input/output use pinned bounded no-follow IO; originals are retained.
`path-fault-comparison.json` retains 32 additional observations and all seven intentional audit
status differences: engine faults stay `proposed` in Python but transition to `discarded` in Rust.
All public errors, other record fields, snapshots and no-op behavior match. No status normalization
or broad exclusions are applied; zero unexpected differences is required separately from exact
parity. Fixtures cover absent/failed/timed-out/missing/empty/unsafe/oversized engines and no-op,
assert one fixed invocation and no dry-run invocation, and prove original/working bytes unchanged.
`results/path-fault-acceptance/native-output-guards.json` records three separate native refusals
for non-SVG root, duplicate IDs and refs to removed original IDs, without snapshots or applied audit.
The Rust target list has an explicit 4096-item bound. Four units cover grammar/bottom order,
outline fill/marker scope, original comments/PIs plus incoming-document DTD serialization and unsafe engine structure.
Shared timeout now matches the Python 1-second floor with a dedicated config unit; the existing
find fixture was corrected from invalid 0.1s to genuine 1s, then all 45 observations passed again.
Broader CSS/reference/asset/schema/geometry/engine/failure and warm-shell/GUI/package work remains.

`transform-comparison.json` covers 116 selector-driven transform observations over 21 cases:
all eight allowed targeted operations, ordered list/single-target projected edits, default dry-run,
CSS paint and accurate CLI bbox selection, empty matches, no-op, match bounds, high-risk approval,
invalid color/pivot and document-root refusal. The native implementation composes the existing
find and batch kernels; the shared batch now accepts a transaction name/match-count parameter.
Whole responses, raw working SVG hashes, history/snapshots, real CLI preview RGBA and exact
restore match. Only existing validated document/audit/time bindings are normalized.
One native unit proves a 65-match dry-run plan and later 64-edit batch-cap refusal, excluded
creation operation, unchanged source/working bytes and empty history. Shared batch regression
passed all 122 edit scenarios again and discovery matched all 16 configurations. Full schema validation/error
formatting, broader limits/coercions/failure injection and GUI/package acceptance remain pending.

`grid-comparison.json` covers 102 full observations over 18 compose_grid cases: document/object
modes, repeated assets, existing/new target, row-major group plans, padding/gap, optional downscale,
mode/size/finite/empty/unknown errors and invalid-new-target refusal. Clone topology/suffix and
wrapper/plan identities validate every ID binding. Whole replies/SVG hashes/raw lengths/history
and real CLI preview pixels match; existing-target restore is exact and original/source working
bytes stay unchanged. Registry deltas prove only a successful new-target call creates a document.
`contracts/grid-plan-cases.json` has 28 exact DOM heuristic cases, used directly by a Rust unit.
Capture is explicit through migration_grid_plan_reference.py; normal Rust tests never refresh it.
Two further units cover plan limits/origins/canvas and pre-creation source-list/aggregate-source/
invalid-plan guards. The heuristic ignores transforms and never upscales, matching the reference.
Whole-document input totals obey an additional configured byte budget; object mode loads once.
Maximum CLI grid, further schema/numeric/import-context/asset/copy/race and native GUI/package
acceptance remain pending. New document creation and edit are separate stages like the reference.

`place-comparison.json` covers 69 native place_document observations over thirteen whole/rootless SVG
and self-contained group cases: translation/down/upscale, missing source/target/object and bad
scale. Original subtree positions/tags/IDs and six-hex suffixes validate each minted ID binding;
wrapper IDs are verified against the clone's minted top ID. Complete replies, raw byte lengths,
normalized full SVG SHA-256, history and real CLI preview pixels match; target restore is exact.
Both sources and working copies stay unchanged. Six native-only context guards separately
refuse outside defs, inherited source paint, viewport-relative geometry, external assets,
scripts and target root paint before mutation/history. Additional context preparation, asset
relocation, scalar/schema errors and additional composition edges remain pending. Two units cover these source
requirements and rootless copy/contained reference remapping. Duplicate/placement now share
one bounded remint kernel. An observed 1024-tile random root-token collision prompted bounded
128-attempt automatic root allocation; explicit IDs still refuse conflicts. A deterministic
unit covers retry/explicit refusal/exhaustion, an intentional robustness difference from Python.

`fragment-comparison.json` covers 131 complete observations over 25 replace_svg_fragment cases:
stable ID/qualified tag, duplicate/invalid/conflicting IDs, internal/external/unresolved/removed
refs, default and allow_retained policy, exclusive C14N no-op with reordered attributes,
retained externally referenced no-op, namespaces, tails, stylesheet preparation and refusal
paths. Whole replies/SVG hashes/history/CLI preview pixels/exact restore match; only validated
minted IDs/timestamps and their artifact paths plus JSON tool text are normalized. Native-only
removed-target timing/accessibility guards separately prove unchanged source/working bytes,
no snapshot and one discarded record. Three units cover early approval/input limits, canonical
no-op bytes and replacement slot/tail/reference policy. Full schema/Pydantic errors, further
canonicalization namespaces/encoding/entities and copy/filesystem failure injection are pending.

`adopt-comparison.json` covers 152 complete STDIO observations over 29 cases for native
set_document_svg/insert_svg_fragment: approval, strict allowlist/namespace/href/url checks,
whole-document/no-op replacement, DTD/top-level comments/PIs, nested/unwrapped/intact fragments,
parent selection, mixed text/tails, plain/xlink references and malformed/active/external content.
Every response field, working SVG hash, snapshot and Operation Record plus real CLI preview
pixels matches; restores recover source bytes exactly. Only validated minted IDs/timestamps,
corresponding artifact paths and JSON tool text are normalized. Two native-only ID guards
refuse insertion collisions with unchanged bytes/no snapshot/one discarded audit record.
Three units verify early approval/input limits, allowlist/entity refusal and independent mixed
namespace copies; a shared XML unit verifies top-level sibling/DTD serialization spacing.
Further scalar/schema/CSS escape/URI/namespace/encoding/failure cases remain pending.
expanded composition edge coverage remains pending. Read parity was rerun.

`optimize-comparison.json` covers 161 complete STDIO observations for svg_web_optimize/optimize_set.
Seven fixtures exercise editor metadata/comments/PIs, used/dead defs and IDs, kept IDs, nested
empty groups, precision 0/2/8, unused namespace declarations and mixed text. Responses/deltas,
serialized SVG SHA-256, history and real CLI PNG RGBA hashes match. Restore recovers original
source bytes before sequential set tests. No-op, ordered/reversed/empty/duplicate/unknown sets
and invalid precision match. Only validated minted IDs/timestamps and corresponding artifact
paths plus JSON text decoding are normalized. Native-only reference guards separately prove
five stronger refusals (CSS/script/SMIL/accessibility/referenced metadata), unchanged working/
source bytes, no snapshot and one discarded audit record. The set is sequential, not atomic
across documents. Full schema errors, Unicode numbers/rounding extremes and further injection
remain pending. Two units prove retained namespaces/references and cleanup idempotence.

`quality-comparison.json` covers 126 full report/set observations over actual MCP STDIO.
Thirteen fixtures run with present and absent fontconfig: metadata/comments, references/defs,
coordinates, rasters/fonts, duplicate/missing IDs, viewBox variants and truncated structure advice.
Four advice configurations plus ordered/reversed/single/empty/duplicate/unknown/malformed sets
match, normalizing only minted document IDs and decoding JSON tool text. No report field is
excluded. Both original and working SVGs stay unchanged; no snapshots or records are created.
Three native units verify advice bounds/counts, read-only analysis and single/set output limits.
Output limits are an extra Rust guard, separate from common parity. Invalid option schema errors,
unusual numeric rounding, races and additional optimizer edge cases remain pending.

`intent-comparison.json` covers 820 exact wire observations without normalization for native
how_do_i/runtime-intents: all 49 map entries and their keywords, four out-of-scope rules,
uppercase/mixed/tied/empty/unknown/Unicode/NUL goals, full/core and live on/off gates, stable
top-three ordering and exact resource text/MIME. No workspace state is created. The native
compiled `contracts/intent-data.json` is shared by both interfaces; provenance records the
preserved source hash and reference HEAD. Normal acceptance checks provenance and never
recaptures; use `migration_intent_acceptance.py --capture-data` explicitly after reviewing
reference guidance changes, then rebuild Rust. Suggested tools are checked against maximum
frozen discovery, including tools still pending native implementation. Input/output bounds
are unit-tested; further argument/schema/environment cases remain pending.

`stat-comparison.json` covers 19 exact wire comparisons with no normalization for streaming
artifact size/SHA-256: relative/absolute/Unicode paths, multiple roots, canonical internal links,
missing/broken/escaping/NUL paths, empty/2.56 MB/over-limit files and ordered/repeated/failed sets.
Fixtures remain byte-identical and no document/history is created. Native
`results/stat-acceptance/native-guards.json` separately proves directory refusal without host
path disclosure and a 1024-path set cap (the reference list has no cap). A unit test proves
known digest, direct final-link refusal and before-read size caps. Hashing uses a fixed 1 MiB
buffer and bounds growing files during reads; length/mtime changes are refused. A separate
`native-output-cap.json` verifies an 80-byte metadata limit without file changes. Further
concurrent-write/race/platform/argument cases remain pending acceptance.

`retention-comparison.json` covers 12 real STDIO observations for explicit snapshot/orphan
record/live-frame pruning: keep-window/count/byte caps, disabled/zero policies, empty state,
idempotence, unknown IDs, exact snapshot hashes, record survival and protected live preview/
diff frame survival. Working/baseline/source bytes stay unchanged. Minted ID lists bind only
previously validated fixture IDs. `results/retention-acceptance/native-path-guard.json` separately
checks tampered manifest basenames and symlink refusal without outside-byte changes. Directory
enumeration is capped at 10,000 entries and record reads at 4 MiB/workspace output cap.
This is explicit cleanup; boot-time sweep and crash-consistent index/file cleanup are pending.
File deletion precedes index rewrite as in the reference; no transactional rollback is claimed.

`repeat-comparison.json` covers 19 real STDIO scenarios for native root-space linked/copy
repetition: default/copy dry-run, transformed parents/source, internal references and gradients,
mixed text/tails, tangent orientation, local anchor, rectangle placement, wide integer seed
jitter, Unicode group labels and target/style/reference/conflict refusals. Complete wire
responses, snapshot bytes/metadata, discarded/applied audit records and real PNG RGBA pixels
are compared. Entire SVG hashes bind only strictly validated unique minted copy-ID suffixes;
the synthetic sibling tree in the harness validates those bindings and is never used for
hashing. Source files stay unchanged. Raw responses, SVGs and observed results are retained.
`repeat-plan-cases.json` additionally freezes 30 direct Python planner outputs/errors, compared
field-for-field without normalization. Seed parsing is bounded at 4096 decimal digits. The
1024 linked-instance maximum, projected byte budget and locked/accessibility guards are DOM
unit tests, not CLI pixel or GUI claims. Copies retain duplicate's stronger reference refusals.
Full schema/error precedence and more alias/namespace/limit cases remain pending. Reproduce
planner capture with `.venv/bin/python scripts/migration_repeat_plan_reference.py`.
The pinned serde_json enables float roundtrip and arbitrary integer precision; the 71 read
and 122 edit comparisons passed again after that parser configuration.

`reload-comparison.json` covers 11 observations: source updates, unconditional pre-reload
checkpoints, missing/directory fallback, internal/escaping symlinks, input-size refusal,
created/opened `document.svg` seed behavior, unknown IDs and actual pre-reload restoration.
Full wire results, snapshot metadata and complete SVG byte hashes are compared. Native
`results/reload-acceptance/native-malformed-guard.json` documents the intentional refusal
before malformed XML replaces either managed copy; Python copies first and fails inspection.
`native-write-failure.json` injects a read-only working directory and proves baseline rollback
when the second replacement fails, unchanged working/source bytes and retained checkpoints.
Each replacement is atomic; two-file crash journaling and further race/fsync injection remain
pending. These native safety fixtures are separate from parity comparisons.

`edit-comparison.json` covers 122 real STDIO edit/history/error scenarios. It compares
snapshot sizes/hashes, no-op behavior, audit records, original-file preservation, atomic
style/color/text/font/transform/canvas-batch failure and byte-exact restoration. UUIDs and timestamps are validated before
binding; artifact URI paths are decoded for explicit ID bindings. Actual Inkscape preview
PNGs are decoded and compared by RGBA pixel hashes. Recolor cases cover overlapping scopes,
style/presentation precedence, gradient stops, palette cascades, no-op and strict keyword refusal.
Text/font cases include tspan tails, literal markup, empty runs, no-op, input limits, own-family
glyph coverage, unavailable fonts and mixed batches. Transform cases verify parent-space
prepend order, numeric formatting/coercion, centred/origin rotation, invalid factors/nonfinite
values, error precedence and rollback after an earlier staged transform. Canvas cases compare
viewBox preservation, retarget/repair/synthesis, percentage/rem fallback, no-op, bleed validation,
unique background IDs and real pixels without changing original SVGs. Engine fit compares exact
working bytes, real pixels and repeat calls for transformed groups, stroke curves, mm/slice,
percentage fallback and empty-document refusal. Numeric repeats are no-op; the percentage
fallback changes repeatedly in both servers. Rename covers ID/label/no-op, href/xlink/paint and
connector reference rewriting, exact working bytes/pixels, conflicts and bounded labels. Native
regressions verify refusal before stylesheet ID mutation and explicit namespace on label attrs.
Stylesheet/timing/accessibility rename references are conservatively refused in Rust; this
intentional safety difference is documented separately from passing common contract scenarios.
Native regression also rejects hex-color/ID ambiguity without mutation. Delete cases compare
HIGH approval gates for direct calls and atomic batches, no-match no-op, duplicate/pre-edit
affected IDs, sequential parent/child removal, root refusal, tail removal and rollback;
working hashes, operation records and real pixels remain part of the same comparison. Unavailable/timeout fit cases remain pending. Raw evidence is in ignored `results/`.

`create-comparison.json` covers 93 vector-authoring observations for all eight primitive tools,
first-layer/explicit transformed-parent selection, explicit IDs, styles, literal/empty text,
analytic bbox, scalar coercion, input refusal and atomic creation/style batches. Exact working
and snapshot hashes, full operation records and real PNG RGBA are compared. The new harness is
`scripts/migration_create_acceptance.py`; raw evidence is in `results/create-acceptance/`.
Linear/radial gradients compare defs creation, offsets/colors/opacity, coordinates/focal points,
actual gradient fills, 1000-stop cap/refusals and gradient-to-fill atomic batches/rollback.
Native regression checks defs/comment/leading-text order and stops inheriting the SVG namespace.
Named-group defaults, explicit layers, child insertion, conversion/no-op, invalid labels/targets,
stylesheet refusals and group/shape/mode atomic batches/rollback are also compared. The working
bytes, snapshots, audit records and actual PNG pixels remain in the same exact comparison.
Instances compare position/transform, source updates, both href forms, new/existing alternate
xlink prefixes, same-document/transform validation and create-source-to-instance batches/rollback;
exact SVG hashes and PNG RGBA remain required. Auto-ID, cycles and additional namespace/cap/
parent/transform edges remain pending; no broad normalization is used.

Adjacent same-parent grouping compares exact order/bytes, unchanged preview pixels, errors
and mixed group/creation batches with rollback. Native-only evidence in
`results/create-acceptance/native-grouping-guards.json` records five intentional safety refusals
for cross-parent moves, paint-order changes, stylesheets and meaningful tails. Every refused
case keeps original/working bytes unchanged and creates no snapshot. The Python grouping
remapper permits such moves; this documented safety difference follows the repository
appearance-preservation requirement and is not a normalization exception.

`reparent-comparison.json` covers 42 observations: affine compensation (translate/scale,
rotation centre, skews, matrices/exponents), unchanged real PNGs, mixed tails, exact working
and snapshot bytes, audit records, no-op and style/reparent batches with rollback. Target/cycle,
ancestor style/effects/locks, references, paint order, singular/CSS/nested/invalid transforms
and undefined XML prefixes are checked through actual STDIO. Inherited namespace bindings
and conflicting destination prefixes compare exact serialized bytes and instance pixels.
`results/reparent-acceptance/native-legacy-guards.json` records five additional safety refusals
for legacy moves that could alter appearance/content; source/working bytes stay unchanged and
no snapshot is created. These refusals are intentional differences from unrestricted Python
preserve_appearance=False behavior, not normalization exclusions. More edge cases remain pending.

`duplicate-comparison.json` covers 19 real STDIO observations: explicit/automatic IDs,
internal/external href/xlink/connector/paint references, masks/clip paths/markers, namespace
adoption, mixed text/tails/comments, cloning an existing clone, batch movement and rollback.
Full results, audit records, snapshot manifests/sizes, SVG bytes and real PNG RGBA are compared.
Only newly minted IDs validated against corresponding original subtree positions (uniqueness,
prefix, six-hex format and shared suffix) are bound before hashing complete SVG bytes. No XML
reserialization or output field omission is used. Raw wire/SVG files remain in
`results/duplicate-acceptance/`. `native-reference-guards.json` records five intentional stronger
refusals for stylesheet/SMIL/accessibility/quoted-paint/hex-ID ambiguity with unchanged source/
working bytes and no snapshots. Those forms and rarer limits/collisions remain follow-up work.
The shared rename reference helper passed all 122 edit observations again.

`tile-comparison.json` covers 27 row-major grid observations, including real 128- and 1024-cell rendering,
groups/references, existing transforms, mixed text/tails/comments, negative/fractional/zero offsets,
coerced counts, 1x1 no-op, batches/rollback and cap/nonfinite/root/missing errors. Full results,
SVGs/snapshots (only validated minted-ID occurrences bound), audit records and actual PNG RGBA
are compared. The 1024-cell upper-bound fixture completed with the original plus 1023 copies,
unique IDs and matching full SVG/history/pixel results. Cap/target-error precedence is checked
separately. Native `results/tile-acceptance/native-size-guard.json` proves
incremental size refusal under a 512-byte configured limit without changing source/working SVGs
or creating snapshots. The shared insertion helper preserves tail order and namespace adoption;
all 19 duplicate comparisons were rerun successfully. More scalar/namespace/entity/collision and failure edges remain pending.

`save-comparison.json` covers 50 validation/save/create scenarios, with exact saved byte hashes,
root-qualified resource readback, approvals, managed files and symlinks, escape without side effects,
blank-document seeds, scalar coercions, ID/viewBox errors, external entity observations and actual
fontconfig glyph coverage. Embedded raster cases include the 5 MiB boundary with excess padding.
Only validated IDs/timestamps and their URI bindings are normalized.

`prompt-comparison.json` covers 96 exact prompt-get responses and four prompt-index resource reads.
For the index, only JSON text is decoded; MIME, fields, values and list order are retained.
`contracts/prompt-messages.json` contains actual reference message templates with one explicit
goal placeholder; native Rust applies the same whitespace/500-character cleaning to that slot.

`render-comparison.json` covers 74 public render/export observations through real STDIO and
Inkscape CLI. PNGs compare actual decoded RGBA hashes, plain SVGs compare exact byte hashes;
PDFs compare magic and exposed content-truth flags only (no PDF pixel/byte-equivalence claim).
Fixtures include relative raster references, whole-page and object previews/exports, inline
thresholds, path naming/readback, future working-copy mtime, size/dimension caps and failed-output
cleanup without working/source mutation. Validated filename timestamps and unique tokens are
normalized narrowly. Region fixtures cover transparent/white backgrounds, out-of-page bounds, physical units,
letterboxing, slice, nonuniform scaling and no viewBox, plus reference refusals for unsupported
root transforms/CSS/percent sizes/alignment. Frame captures/listings check sanitized series/labels, empty series, sequential and seeded
numbering, symlink exclusion and invalid widths; frame filenames/indices are compared exactly.
Historical region comparisons verify inline and artifact-readback PNGs before/after a style edit,
missing snapshots, mapping mismatch refusal without artifacts and immutable source/snapshot
bytes. Warm shell remains pending.

`export-batch-comparison.json` covers 21 observations: dry-run/default plans, output-directory
creation semantics, 32-item bounds, widths/IDs/formats, budget clamp/refusal, and real mixed
PNG/SVG exports. Exact total output bytes, artifact readback, PNG RGBA and SVG byte hashes are
compared. Sources and working SVGs remain byte-identical; preflight errors create no artifacts.
Only validated IDs/filename timestamps and decoded JSON text are normalized.

`profile-comparison.json` covers 28 observations: web widths/scales precedence and sorting,
icon order/empty lists, size errors and escapes; exact PNG pixels/SVG bytes and source immutability.
Print coverage checks real PDF 1.4, font outlining and applied options, without claiming complete
PDF byte or pixel equivalence. Native profile lists are capped at 32; duplicate icon sizes and
same-second naming collisions remain dedicated follow-up fixtures.

`set-comparison.json` covers 17 observations across seven SVG fixtures. It compares complete
consistency verdicts (viewBox/fallback, dominant stroke widths and ID naming, unknowns and ties),
dry-run totals, empty/duplicate/unknown/malformed failures, actual two-document PNG/SVG output
and exact total bytes. All source/working bytes are retained. Bound document IDs appearing in
verdict lists are explicitly normalized; no property value or error is omitted. Native set inputs
are capped at 32. Later export failures may retain earlier artifacts, matching reference semantics.

`python-baseline-summary.json` summarizes five repeated real STDIO scenario runs.
Raw requests, responses, timings, initialization RSS snapshots and stderr are retained locally
under `migration/results/python-baseline/` (gzip preserves complete raw traces). That directory
is ignored by Git to keep large observations separate from reviewable source and summaries.
The initialization RSS snapshot does not establish peak or Inkscape child-process memory.
The timings include client JSON parsing, IPC, file IO and engine processing; they do not yet
attribute time to those components. The separate release checkpoint comparison below now records limited local evidence.

`checkpoint-benchmark-comparison.json` preserves every sample from five fresh Python and
five optimized Rust runs. Startup medians are 519.40/35.75 ms; initialization server RSS is
92,336/12,688 KiB. Inspect roundtrips are lower for Rust, but edits/render/export are slower in
these fixtures. This is an incomplete developer build and a sequential warm-cache comparison,
not an overall speedup, package or peak-memory result. The live_status migration-pending error
is the sole response-status mismatch and its timing is explicitly not comparable. Build hash,
flags and linked system libraries are recorded in `release-checkpoint-build.json`.

Reproduce the new measurements after the documented release build:

```sh
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_probe.py \
  --output migration/results/rust-release-checkpoint --repeats 5 \
  -- rust/target/release/inkscape-mcp-rust
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_probe.py \
  --output migration/results/python-checkpoint --repeats 5 \
  -- .venv/bin/inkscape-mcp
.venv/bin/python scripts/migration_summarize.py \
  migration/results/rust-release-checkpoint migration/rust-release-checkpoint-summary.json
.venv/bin/python scripts/migration_summarize.py \
  migration/results/python-checkpoint migration/python-checkpoint-summary.json
.venv/bin/python scripts/migration_benchmark_compare.py \
  migration/results/python-checkpoint migration/results/rust-release-checkpoint \
  migration/checkpoint-benchmark-comparison.json
```

Reproduce:

```sh
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_probe.py \
  --output migration/results/python-baseline --matrix --repeats 5 \
  -- .venv/bin/inkscape-mcp
.venv/bin/python scripts/migration_summarize.py \
  migration/results/python-baseline migration/python-baseline-summary.json
.venv/bin/python scripts/migration_probe.py \
  --output migration/results/rust-discovery --matrix --discovery-only \
  -- rust/target/debug/inkscape-mcp-rust
.venv/bin/python scripts/migration_compare.py migration/contracts \
  migration/results/rust-discovery --report migration/discovery-comparison.json
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_capability_acceptance.py
.venv/bin/python scripts/migration_read_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_edit_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_save_acceptance.py
.venv/bin/python scripts/migration_prompt_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_render_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_batch_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_profile_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_set_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_create_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_reparent_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_duplicate_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_tile_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_find_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_reload_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_repeat_acceptance.py
.venv/bin/python scripts/migration_retention_acceptance.py
.venv/bin/python scripts/migration_stat_acceptance.py
.venv/bin/python scripts/migration_intent_acceptance.py
.venv/bin/python scripts/migration_quality_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_optimize_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_adopt_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_fragment_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_place_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_grid_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_transform_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_path_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_action_acceptance.py
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" \
  .venv/bin/python scripts/migration_action_discovery_acceptance.py
.venv/bin/python scripts/migration_action_fault_acceptance.py
.venv/bin/python scripts/migration_path_fault_acceptance.py
```

Run the console entry point for the Python reference. `python -m inkscape_mcp.server`
currently registers a different module's app instance and exposes zero tools; the harness
now rejects an empty surface rather than accepting a useless timing measurement.
That pre-existing issue has not been changed as part of the migration.

### Public live mutations and operation records (2026-10-02)

Native `live_apply_to_selection`, `live_set_selected_text`, `live_insert_svg` and
`live_edit_selection` now reuse the guarded transport scope, required-command checks,
high-risk approval policy, persisted proposed/applied/discarded records, captured previews
and explicit uncertain completion. `inkscape://live/operations` is native. Records retain
opaque window/document identities, bounded fixed parameters and original source bytes;
approval tokens are never persisted. Source-compatible path redaction includes the legacy
re-sanitization of relative document paths when updating records. Native reads additionally
require minted operation IDs, no-follow regular files and an aggregate output budget.

`migration/live-mutation-comparison.json` covers 62 real STDIO observations against an
owned authenticated loopback peer: validation/approval precedence, four mutation families,
all eight unsupported socket structural edits, no-op render caching, proposed records before
mutation, helper refusal, lost replies, reconnect/history clearing, complete disk/resource
records and PNG hashes. There are zero unexpected differences. Two exact checked differences
are retained for a mutation whose reply is lost: Rust reports uncertain completion and records
`completion_uncertain=true`, while Python reports communication failure and false. Neither
retries the mutation. Shared approval stripping now matches Python C0 whitespace as well.

`migration/live-managed-comparison.json` now covers 55 observations with a synthetic gdbus
peer, including an approved style effect, fingerprint reconciliation, persisted opaque IDs
and before/after PNG records. Seven explicit Python-runtime identity differences remain;
no unexpected differences. This does not prove native GUI effect execution or Undo/Redo.
All 157 Rust tests, format/all-target clippy, Ruff (286 files) and mypy (122 sources) pass.
Edit/save regressions pass all 122/50 observations; live status/connect/events/render/viewport
regressions pass 48/26/23/60/46 observations. Fresh synthetic servers load the changed code;
the user's MCP configuration and existing GUI are untouched.

Current count: 101 native tools, 9 pending tools, all 18 resources and 7 prompts native.
Pending tools: `live_arm_socket`, `live_diff_view`, `live_export_selection`,
`live_find_objects`, `live_install_helper`, `live_launch`, `live_preview_object`,
`live_session_step`, `live_sync_to_workspace`. Full schema/error coverage, native GUI/Undo,
packaged helper/bridge/bus, doctor, current-Mac installation, platform builds and new phase/peak
memory measurements remain required. The full migration goal is active and incomplete.

### Public selection export (2026-10-02)

Native `live_export_selection` uses the fixed transport command set, best-effort selection,
bounded PNG output and no-follow atomic artifact publication. It preserves source filenames
(`live-selection-<UTC second>.png`), including same-second replacement, without caching or
creating an Operation Record. Native pre-publication size checks preserve an existing artifact
when a later oversized export is refused; the reference checks the cap after publication.
One native security unit checks this stronger guard and artifact-directory symlink refusal.

`migration/live-export-comparison.json`: 17 real STDIO observations, zero differences; owned
loopback peer only, not GUI. Covers disconnected refusal, fixed socket support independent of
handshake advertisement, repeated/new PNG pixels, empty selection, helper refusal followed by
another successful read without reconnect, exact IDs, size/hash/pixels and no automatic retry.
Only second-resolution artifact timestamps are validated against process lifetime and bound.
Original SVG bytes and the empty operation directory are verified. No user windows were touched.
All 158 Rust tests and all-target clippy/format pass; Ruff checks 287 files and mypy 122 sources.
LLM indexes were regenerated with unchanged frozen discovery text.

Current count: 102 native tools / 8 pending, all 18 resources / 7 prompts native. Pending:
`live_arm_socket`, `live_diff_view`, `live_find_objects`, `live_install_helper`, `live_launch`,
`live_preview_object`, `live_session_step`, `live_sync_to_workspace`. Native GUI/Undo and package,
doctor, platform and complete phase/peak-memory acceptance remain unfinished; the goal is active.

### Public live discovery and isolated preview (2026-10-02)

Native `live_find_objects` captures active identity and one SVG under the existing guarded
operation scope. It validates a bounded self-contained snapshot before private CLI processing,
queries actual engine bounds and converts them to document user units with the shared
`geometry::root_mapping`. No local-attribute fallback is invented. The mapping now handles
source Unicode decimal/underscore parsing, absolute units and aspect alignment; 106 frozen
Python mappings compare every field/error. Search shares scene visibility and ObjectInfo,
adds nearest-layer identity, ancestor locks and normalized text, and uses the pinned source
Unicode 15.0 casefold table (1530 mappings) without a Python runtime.

Native `live_preview_object` takes a fresh guarded snapshot, checks its fingerprint and visible
ID, enforces safe isolated-export IDs and computed/actual PNG dimensions, then publishes a
bounded PNG through no-follow atomic workspace writes. CLI receives the intact immutable
snapshot, preserving ancestors and referenced definitions. Neither tool changes selection,
drawing or native Undo history; no Operation Record is created. Actual GUI behavior remains
unverified. Native additionally refuses saturated query stdout rather than accepting a partial
geometry table; the general input/output/XML and no-follow guards remain enforced.

`migration/live-discovery-comparison.json`: 118 real STDIO observations through an owned
authenticated loopback snapshot peer and real Inkscape CLI, zero differences. Includes AND
filters, Unicode casefold, layer matching/locks, transformed bounds, hidden/overridden visibility,
comma IDs for discovery, truncation/limits, physical units/viewBox/letterboxing, unsupported root
mappings, unsafe snapshots/PI/XML/element caps, fingerprint drift, unsupported/missing/hidden/zero
preview IDs, width/height limits and four isolated previews. Complete result/transport fields and
PNG bytes/RGBA pixels match; only validated 24-hex preview nonce paths are bound. Original bytes
and an empty operation-record directory are checked. Raw traces remain in ignored `results/`.
After sharing geometry, all 118 live observations and 74 headless render/export observations pass
again. All 160 Rust tests, all-target clippy/format, Ruff (289 files), mypy (122 sources) pass;
LLM indexes regenerated, frozen text unchanged. No GUI or user configuration was changed.

Current count: 104 native tools / 6 pending, all 18 resources / 7 prompts native. Pending:
`live_arm_socket`, `live_diff_view`, `live_install_helper`, `live_launch`, `live_session_step`,
`live_sync_to_workspace`. Full argument/failure coverage, helper/bridge/bus packaging, doctor,
ready current-Mac install, other-platform builds, native GUI Undo/Redo and fresh phase/peak-memory
benchmarks remain required. The full migration goal remains active and incomplete.

### Public live workspace sync (2026-10-02)

Native `live_sync_to_workspace` validates and creates in-sandbox parents before requiring a
transport, refuses existing destinations, captures bounded live SVG bytes and registers a new
headless baseline/working copy through the shared registry. It persists the medium-risk proposed
Operation Record, creates the shared `live sync` snapshot, then links the snapshot/artifact and
marks the record applied. All linked bytes remain identical. The registry seeding kernel was
split from summary inspection without changing open/create behavior: source sync registers raw
SVG bytes (including malformed XML), and later inspection remains independently guarded.

New POSIX `Workspace::atomic_create` fsyncs staging bytes and publishes with exclusive `linkat`
under pinned no-follow directory handles, then removes the staging name. It refuses a destination
created during the live read; source sync's `replace` can overwrite that concurrent name. This
stronger no-overwrite guard implements the stated sync contract. A native unit races eight
publishers (exactly one winner, complete identical-byte file, no staging leaks), and checks an
existing symlink/external bytes are preserved. It does not establish Windows handle behavior.

`migration/live-sync-comparison.json`: 45 real STDIO observations against an owned authenticated
loopback peer, zero differences. Validates destination/working/original/snapshot bytes and hashes,
full audit fields and timestamp lifecycle, nested relative and absolute second-root destinations,
mixed content and malformed raw snapshots, path/parent/symlink/NUL/empty/existing refusals before
connection, disconnected parent creation, helper refusal/recovery and exact transport requests.
Only validated minted IDs/times/snapshot filenames and the exact absolute input are bound.
Additional native STDIO race evidence in `results/live-sync-acceptance/native-race.json` proves a
concurrently created destination retains its bytes and the registry stays unchanged, without
retry or partial temp artifacts. No GUI was changed or exercised.

After sharing registry seeding, all 71 read and 50 save/create regression scenarios pass again.
All 161 Rust tests, all-target clippy/format, Ruff (290 files) and mypy (122 sources) pass;
LLM indexes regenerated, frozen guidance unchanged. Current count: 105 native tools / 5 pending,
all 18 resources / 7 prompts native. Pending tools: `live_arm_socket`, `live_diff_view`,
`live_install_helper`, `live_launch`, `live_session_step`. Full schema/failure coverage and ready
package/doctor/platform/native GUI/Undo/phase-memory acceptance remain required. Goal stays active.

### Public focused live diff (2026-10-02)

Native `live_diff_view` resolves only fixed-ID live records and their bounded no-follow PNG
frames under the live artifacts directory. It compares RGB channels (ignoring alpha-only noise),
refuses mismatched dimensions, converts the best-effort scene selection to pixel rectangles with
source ties-to-even/clamping, then draws cyan selection and red changed-region outlines. Source
narrow-outline overlap behavior is preserved; see the [Pillow drawing implementation](https://github.com/python-pillow/Pillow/blob/12.2.0/src/libImaging/Draw.c).
It atomically publishes `live-diff-<operation>.png` and best-effort appends the path to the existing
record; it never changes the drawing/selection or creates another operation. Unknown/disconnected
scene context simply omits selection outlines. Source frames are preserved byte-for-byte.

`migration/contracts/live-diff-cases.json` freezes 168 source pixel cases across RGB/RGBA,
grayscale/alpha, palette, 16-bit grayscale and one-bit PNG, with tiny/narrow rectangles,
clipped/reversed/fractional selection bounds and changed/no-change frames. All bboxes, highlighted
IDs and exact RGBA annotations match. Sixteen-bit grayscale conversion clips values as Pillow
does, rather than silently discarding low bytes. The implementation uses the native PNG decoder
and encoder; no Python image library is invoked by Rust.

`migration/live-diff-comparison.json`: 51 real STDIO observations, zero semantic differences.
Covers malformed/unknown IDs, missing previews/files, mismatched sizes, twelve fixture pairs,
repeat append-only record links, connected synthetic scene outlines and absent-bbox entries,
complete public/record fields and decoded pixels. Eighteen compressed PNG representation deltas
(size/hash) are retained separately: native/Pillow encoders produce different compressed bytes.
This is pixel/result parity, not byte-identical PNG encoding. Extra native evidence refuses
symlinked frames, keeps external bytes intact and writes no output. No actual GUI was exercised.

Native PNG memory/byte guards run before pixel allocation, with the source pixel-count ceiling
plus a stronger 512 MiB aggregate comparison budget and no-follow reads. A native unit checks
input caps, external frame links and a valid-CRC oversized header. Native checks output size
before replacing a previous diff, whereas source checks after publication. Further corrupted,
non-PNG/APNG/metadata/large-frame and failure-injection compatibility remains unverified.
All 163 Rust tests, all-target clippy/format, Ruff (292 files) and mypy (122 sources) pass.
LLM indexes regenerated; frozen instructions unchanged. Current count: 106 native tools / 4
pending, all 18 resources / 7 prompts native. Pending: `live_arm_socket`, `live_install_helper`,
`live_launch`, `live_session_step`. Package/doctor/platform/native Undo/Redo/full-schema and new
phase/peak-memory acceptance remain required; the full goal stays active and incomplete.

### Public one-step live orchestration (2026-10-02)

Native `live_session_step` composes existing frame/scene perception, governed style/text/SVG
validators, the high-risk mutation pipeline and focused diff. It performs one iteration only;
`action=null` returns perception with null act/observe fields and creates no record. Fixed action
enum rejection occurs before transport work. For an act, source-compatible perception precedes
parameter/approval checks; records use `live_session_step:apply|insert_svg|set_text`. Failed
post-act perception leaves both after fields null, while best-effort diff uses the captured
before scene as annotation fallback. Diff pixels always come from the record's saved frames.
There is no additional mutation path, raw Action/code execution, internal multi-step runner or
automatic retry. No new cancellation/GUI acceptance claim is made.

Text/fragment validation was extracted from the standalone mutators into shared functions;
standalone calls retain their behavior. The diff kernel now accepts an already-captured scene,
so step observation adds no redundant scene IPC. Public standalone diff retains its best-effort
scene capture. The reference's broad orchestrator exception mapping is preserved for helper
protocol refusals. Full noncanonical schema/enum validation remains unproven; the tested invalid
string enum error retains the current frozen Pydantic 2.13 message.

`migration/live-loop-comparison.json`: 46 real STDIO observations against an authenticated owned
loopback peer, zero semantic differences. Covers disconnected/enum precedence, perceive-only
ignored action parameters/no records, validation and empty/C0 approval refusals, all three acts,
no-op cached frames, proposed records before dispatch, before/after scenes/frame bindings,
append-only diff links, failed after-scene fallback, helper refusal and no retries. Complete IPC
ordering/parameters, operation-resource/disk records and RGBA pixels match. Ten compressed PNG
representation differences remain explicitly retained, as in standalone diff. Original SVG bytes
are unchanged; this is synthetic IPC acceptance, not native GUI Undo/Redo.

All 163 Rust tests, all-target clippy/format, Ruff (293 files), mypy (122 sources) pass. Shared
regressions rerun: mutation 62 observations (two explicit lost-reply uncertainty differences,
zero unexpected), diff 51 (18 retained encoding differences, zero semantic), loop 46 (ten retained
encoding differences, zero semantic). LLM indexes regenerated with unchanged frozen guidance.
Current count: 107 native tools / 3 pending, all 18 resources / 7 prompts native. Pending tools:
`live_arm_socket`, `live_install_helper`, `live_launch`. These lifecycle/install tools and ready
helper/bridge/bus packaging, doctor, current-Mac clean installation, platform builds, native GUI
Undo/Redo, full schema/security/failure coverage and new phase/peak-memory measurements remain
required. The full migration goal remains active and incomplete.

### Fixed live helper installation (2026-10-02)

Native `live_install_helper` installs only the two compile-time shipped socket-helper assets,
after the live gate and cached read-only Inkscape capability probe. It preserves source result
fields, ordered filenames and home-relative/basename path redaction. Startup and reconnect do
not install extensions or launch GUI. The shared POSIX descriptor/no-follow write pipeline
refuses linked parents and nonregular destinations before either asset is replaced. Atomic
replacement preserves an externally hardlinked original; the reference uses truncating copies.
The two-file upgrade is sequential, not a crash-atomic pair. Windows guards remain unported.

`migration/live-install-comparison.json`: five isolated profiles, 18 real STDIO observations,
zero differences and no normalization. Checks both fixed asset hashes, first install/upgrade,
read-only probe argv/cache, missing engine/user-data directory and blocked target errors,
path redaction, unchanged blocked bytes and no staging leaks. Only synthetic child HOME/data
paths were written. A native security unit separately checks symlink refusal, hardlink
preservation and preflight ordering. On macOS its temporary root is canonicalized to remove
the system `/var` alias; production no-follow checks remain intact.

All 164 Rust tests, all-target clippy/format, Ruff (294 files) and mypy (122 sources) pass.
LLM indexes regenerated with frozen discovery text unchanged. This proves installation of
reference helper bytes, not execution by packaged Python/inkex or native GUI acceptance.
Current count: 108 native tools / 2 pending (`live_arm_socket`, `live_launch`), all 18 resources
and 7 prompts native. Full schema/failure, packaged helper/supervisor/bridge/bus, ready Mac
installation, doctor, platform builds, native Undo/Redo and phase/peak-memory benchmarks
remain required. The full migration goal stays active and incomplete.

Reproduce with `.venv/bin/python scripts/migration_live_install_acceptance.py`.
Raw traces are in ignored `results/live-install-acceptance/`.

### Explicit socket helper arming (2026-10-02)

Native `live_arm_socket` requires the live gate and cached user-data capability, installs only
fixed shipped assets when neither system nor user directory has a regular helper marker,
then reuses an advertising rendezvous before binary/display checks. New launch uses only
`--with-gui`, the fixed helper action and a server-minted private blank SVG, with null stdio
and a detached process session. The document remains available for the GUI/OS temp policy.
Polling is bounded by max(5 seconds, configured process timeout); timeout or MCP exit never
kills the GUI. An owned-child waiter reaps normal exits. No arbitrary extension/action input
is accepted. Marker reads add no-follow parent/file protections; Windows remains unported.

`migration/live-arm-comparison.json`: six isolated profiles and 27 real STDIO observations,
zero differences. Covers first launch then reuse, pre-existing rendezvous, system helper marker,
missing user directory, blocked extension target, timeout without retry, and child survival
past timeout and MCP exit. Compares complete wire fields, installed asset hashes, cached probe
requests, fixed launch argv, exact blank SVG bytes, 0600 mode, detached session and retained
file. Only the validated minted document path is bound. The fake engine is compiled as Python
before execution. No real GUI was opened; helper execution/Undo remain unproved. Raw traces:
`migration/results/live-arm-acceptance/`. Reproduce with
`.venv/bin/python scripts/migration_live_arm_acceptance.py`.

All 164 Rust tests, all-target clippy/format, Ruff (295 files), mypy (122 sources) pass.
LLM indexes regenerated, frozen text unchanged. Current count: 109 native tools / one pending
(`live_launch`), all 18 resources and 7 prompts native. Ready packaged runtime/supervisor/bridge/bus,
doctor, current-Mac installation, other platforms, native GUI Undo/Redo, full argument/failure
coverage and fresh phase/peak-memory benchmarks remain required. Full goal stays active.


### Explicit managed launch and packaged supervisor foundation (2026-10-02)

Native `live_launch` now gates live/macOS, creates or validates an owned 0700 session through
no-follow descriptor descent, bounds Unix socket path length, serializes launch with a bounded
fixed lock, and adopts an existing reachable private managed bus. A surviving supervisor lock
with an unavailable bus refuses a second launch. Only the verified system `/tmp` and `/var`
aliases normalize on macOS; arbitrary parent/final links are rejected. The ready attached host
is retained internally for a later `live_connect`, without changing the process environment.
The bool response preserves the exact FastMCP result wrapper and wrap_result metadata.

New launch resolves fixed package-relative `libexec/inkscape-mcp` assets and starts only its
private Python with `-I` and fixed `supervise.py`, session root and trusted Inkscape binary.
It uses null stdin, owned bounded-path log files and detached session. The 15-second readiness
wait never kills a potentially running GUI/supervisor. The standalone stdlib supervisor in
`rust/package/supervise.py` imports no Python MCP server: it installs six fixed one-shot helper
assets and a private-runtime wrapper, copies the vendor executable/resources association,
uses a prebuilt context module, ad-hoc signs only the private executable copy, and owns a
private fixed-path bus until its GUI exits. It never compiles on the user's machine or changes
macOS settings. Bounded no-follow asset reads and atomic directory-relative writes preserve
external/hardlinked originals. Full directory-race/crash and native supervisor acceptance
remain unproved; the ready bundle supplying these paths does not exist yet.

`migration/live-launch-comparison.json`: three synthetic STDIO profiles, nine observations,
zero differences without normalization (existing ready session, wrong permissions, surviving
supervisor with lost bus). Separate native evidence covers a linked manifest, linked parents,
external/hardlinked byte preservation, asset caps and no staging leaks. An owned temporary
fake package with a copied Rust executable proves the new-launch fixed argv, isolated Python,
detached session, no startup launch, repeat reuse and child survival past MCP exit. Its runtime,
supervisor/bridge/bus are test substitutes: this does not validate the real packaged supervisor
or native GUI. Raw evidence is in `migration/results/live-launch-acceptance/`; reproduce with
`.venv/bin/python scripts/migration_live_launch_acceptance.py`.

`migration/context-build-comparison.json` records the real development clang command, source
and bridge hashes, arm64 Mach-O type and dependencies (only Apple system frameworks/libraries,
no Homebrew dylib). Build headers were available on the development host. No GUI loaded it.
A direct Inkscape bundled Python/inkex probe exited 137 without output; the cause is unproven
and this candidate is not accepted as a working runtime. The package still needs a verified
private runtime, relocated D-Bus and complete packaged/helper/GTK ABI/signing checks.

All 165 Rust tests, all-target clippy/format, Ruff (297 files), mypy (122 sources) pass.
Managed regression: 55 observations, seven retained Python runtime identity deltas, no unexpected
differences. Connect regression: 26 observations, zero differences. LLM indexes regenerated,
frozen discovery guidance unchanged. All 110 tool names now have native dispatches, all 18
resources and seven prompts are native; partially implemented batch/schema/error cases and
unverified packaged launch are still material limitations. Ready Mac installation, doctor,
platform builds, native GUI Undo/Redo and current phase/peak-memory benchmarks remain required.
The full migration goal is active and incomplete; dispatch coverage is not completion.


### Local macOS arm64 candidate and archive installation (2026-10-02)

A real local candidate is available at
`migration/results/packages/inkscape-mcp-macos-arm64-stage3/`, with archive
`migration/results/packages/inkscape-mcp-macos-arm64-stage3.tar.gz` (46,230,230 bytes;
SHA-256 `0c2d4086893f04facb0b87dfb24b2bb29d8e9e34e3f9631b597486e9e1009721`).
This is a current-Mac CLI/STDIO candidate, not the finished replacement or a published release.
The archive contains the optimized native Rust executable, an actual prebuilt arm64 context
module, a private CPython 3.12.14 runtime and six helper dependencies, fixed helper/supervisor
assets, relocated dbus-daemon/gdbus and their library closure, package provenance/licenses and
2258 file size/hash entries. Uncompressed regular bytes: 129,927,894. Native MCP execution does
not load or wrap Python; Python remains only for the fixed supervisor and inkex helpers. Vendor
inkex is read from the installed Inkscape bundle. Development used the existing uv-managed
[python-build-standalone runtime](https://github.com/astral-sh/python-build-standalone/blob/main/docs/running.rst)
([uv provenance](https://docs.astral.sh/uv/reference/environment/)); the user needs neither uv/pip,
Homebrew nor a compiler to run this candidate.

`scripts/migration_build_macos_package.py` refuses an existing output directory, copies only
helper dependencies (no Python MCP package), builds the bridge on the development machine,
relocates each non-system D-Bus dependency to loader-relative paths and ad-hoc signs modified
copies. Fixed package manifest enables native discovery of private gdbus and the vendor
Inkscape executable with an empty PATH; the development/reference PATH behavior is unchanged.
Private gdbus disables external GIO module directories. A dedicated packaged Unix bus config
uses EXTERNAL authentication and no host includes or service activation directories. Its
mandatory listener is overridden by the fixed owned socket argv, per the [D-Bus documentation](https://dbus.freedesktop.org/doc/dbus-daemon.1.html).
No system bus, GUI or security settings were modified. Developer ID/notarization, a complete
redistribution-license/source audit and other-platform packaging remain unfinished.

`migration/package-build-comparison.json` retains build/dependency provenance and archive hash.
`migration/package-comparison.json` proves installation from this actual archive into a fresh
owned temporary directory, file/hash and contained-link checks, and execution with an empty
PATH and isolated HOME. Private Python imports all helper libraries from the relocated bundle
(vendor inkex excepted); both insertion and socket helper CLIs run. The actual relocated bus
and gdbus complete an authenticated private Unix exchange, then only that exact owned bus is
terminated. Actual packaged STDIO discovery compares every field of tools/prompts/resources/
templates against the frozen full contract (110 tools, seven prompts, ten static resources and
eight templates). Native open/edit, no-op without audit, one-record atomic batch, failing-batch
rollback, approval refusal, save/resource bytes and untouched original pass. Real Inkscape CLI
preview and PNG export/resource readback have the expected blue RGBA pixel. Startup/headless
calls do not create the configured managed session. No native GUI was opened.

Reproduce:
```sh
.venv/bin/python scripts/migration_build_macos_package.py --output migration/results/packages/<new-name>
.venv/bin/python scripts/migration_package_acceptance.py --archive migration/results/packages/inkscape-mcp-macos-arm64-stage3.tar.gz
```
For a manual candidate test, extract the archive into an empty folder and set the MCP executable
to its `bin/inkscape-mcp`, with `INKSCAPE_MCP_WORKSPACE_ROOTS` pointing to a synthetic writable
workspace. No build/install command is needed. This exact local process configuration is tested:
```json
{
  "command": "/Users/bm/Documents/repos/inkscape-mcp-server/migration/results/packages/inkscape-mcp-macos-arm64-stage3/bin/inkscape-mcp",
  "env": {
    "INKSCAPE_MCP_WORKSPACE_ROOTS": "/absolute/path/to/synthetic-workspace",
    "INKSCAPE_MCP_LIVE_ENABLED": "1"
  }
}
```
The user's configured server is unchanged. Packaged managed GUI/supervisor/effect execution,
native Undo/Redo, doctor, signing/ABI/platform acceptance, complete behavioral coverage and new
phase/peak-memory measurements remain required; the full goal is active and incomplete.
All 165 Rust tests, all-target clippy/format, Ruff (299 files), mypy (122 sources) pass. Frozen LLM
indexes are regenerated; original source Python server is preserved.


### Native read-only doctor and refreshed candidate (2026-10-02)

The native binary now accepts `--doctor` before creating an MCP server. It emits structured
JSON and exits 0 only when current package prerequisites pass (1 otherwise). Checks cover
current macOS/arm64 package target, bounded no-follow package metadata and fixed assets,
Mach-O architecture of private Python/bridge/bus executables, official vendor GTK3/inkex,
actual Inkscape CLI minimum version, actual private Python/inkex/numpy/lxml/Pillow imports,
private bus/gdbus CLI startup and system codesign presence. It does not require a compiler,
Homebrew headers or a user Python installation. Missing/incompatible checks include concrete
next steps. `ready_to_launch` means prerequisites, with `native_gui_verified=false`; it does
not certify native GUI, Undo/Redo, signatures or notarization.

The real macOS launcher was observed creating preferences even for `--version`. Doctor now
runs that fixed probe with an owned ephemeral HOME/profile/config/cache and an unavailable
private bus address; bounded child completion precedes temporary cleanup. Private Python uses
`-I -B`, so it ignores external Python configuration and writes no bytecode. An internal typed
process environment wrapper is used only by fixed server callers; no tool accepts executable,
code or environment overrides. No GUI, bus service, extension install or automatic repair is
performed. Inkscape supports profile isolation through its documented [environment variables](https://wiki.inkscape.org/wiki/Environment_variables).

`migration/doctor-comparison.json`: ten owned relocated-package profiles pass exact ready/exit
and relevant failed-check assertions: ready, missing bridge/runtime/helper/bus/gdbus/manifest,
linked bridge, malformed architecture and actual mocked Inkscape 1.2.2 version output. Full
package file hashes and owned directory entries remain unchanged after diagnosis (including
no bytecode, managed directories or user profile creation). Raw JSON reports are in
`migration/results/doctor-acceptance/`. No native GUI was opened.

The refreshed local candidate is
`migration/results/packages/inkscape-mcp-macos-arm64-stage5/`, with archive
`migration/results/packages/inkscape-mcp-macos-arm64-stage5.tar.gz` (46,232,964 bytes; SHA-256
`cbc7887b0593bef43d964c8351d94012980d3acf852fe31aae4536cb19338953`). It contains 2258 regular
file hash entries, 129,944,806 regular bytes. `migration/package-build-comparison.json` now
points to this candidate; older candidates remain available as historical evidence. Installation
from the actual new archive again passes clean empty-PATH runtime/helper/private-bus/STDIO
full discovery, no-op, batch/rollback, approval refusal, original preservation, SVG save/resource
and real PNG preview/export pixel checks. Raw acceptance remains in `migration/results/`.

Reproduce/test:
```sh
migration/results/packages/inkscape-mcp-macos-arm64-stage5/bin/inkscape-mcp --doctor
.venv/bin/python scripts/migration_doctor_acceptance.py --package migration/results/packages/inkscape-mcp-macos-arm64-stage5
.venv/bin/python scripts/migration_package_acceptance.py --archive migration/results/packages/inkscape-mcp-macos-arm64-stage5.tar.gz
```
Configure normal MCP execution with that candidate's `bin/inkscape-mcp` and a synthetic writable
`INKSCAPE_MCP_WORKSPACE_ROOTS`, without `--doctor`. The user's current MCP configuration is
unchanged. Native packaged GUI/supervisor/effects/Undo, signing/ABI/license audit, other platform
builds, complete behavioral coverage and new phase/peak-memory measurements remain required.
All 165 Rust tests, all-target clippy/format, Ruff (300 files), mypy (122 sources) pass; LLM indexes
regenerated with frozen text unchanged. The full migration goal remains active and incomplete.
