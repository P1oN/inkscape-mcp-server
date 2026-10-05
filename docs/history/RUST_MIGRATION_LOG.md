> Historical checkpoint. Counts, commands, package paths and “current” labels
> apply only to the recorded build. See the [history index](README.md).

Current package checkpoint: stage36 (2026-10-03). Actual cold/warm archive install,
doctor10/launcher11/notices17 pass.2532FILES entries verified. Exact comparison to35:
2506 existing files unchanged, including every executable/runtime/helper/native asset;
only inventory/package metadata changes and24 added CPython/source notices. Thus35
binary-specific security/discovery/CLI/diagnostic evidence applies to identical bytes,
without inferring new GUI/performance acceptance. Build index: package-stage36-build-comparison.json.

Provenance review: six exact PyPI wheels,1259 matching members. Every upstream RECORD
row retained; only hashed uv INSTALLER/empty REQUESTED metadata and two omitted NumPy
CLI records added. Installed GLib recipe matches exact SHA-verified SBOM bottle; current
API-cache recipe differs and was not substituted. Sole GLib patch history commit predates
the bottle; patch applies to exact upstream2.90.0. Native source kit includes four exact
upstream archives/recipes/receipts/SBOMs, patch and recipe's gobject-introspection resource.
Raw: wheel-source-audit-stage35-final and homebrew-source-audit. Initial wheel-report
serialization failure was a local audit variable-shadowing bug, fixed; earlier raw retained.

19CPython license texts/PYTHON.json/provenance and GLib recipe/patch now packaged in36;
wheel hashes enforced by builder. Homebrew recipe BSD2 license additionally retrieved
and prepared in collector, but not yet in36; include it and the source-kit license in the
next consolidated package. Do not claim complete redistribution clearance. Source kit
is a source-input artifact, not a reproduced binary. Current GUI/performance evidence
remains historical. Goal active: finish remaining publication/output scope and current
synthetic native assessment, then final requirement/documentation audit. Windows,
clean-machine installation and historical incident cause remain user-deferred.

Current package checkpoint: stage36 (2026-10-03). Includes bounded validation diagnostics,
compare_region private rendering before pair publication, and exact upstream notice
supplements for GLib2.90.0/D-Bus1.16.2/gettext1.0/PCRE2 10.48, matched to installed SBOM
URL/hash/version. Full GLib LGPL, referenced D-Bus alternatives and runtime libintl LGPL
are packaged; receipt/formula metadata is bound by FILES. Rust213/1ignored, fmt/clippy,
15notice checks/102crates, doctor10/launcher11/security35/files12/discovery16,
asset17/routes8 both engines, debug/package diagnostics3 and compare refusal/pixel readback
pass. Actual cold/warm archive installs with empty PATH pass. Archive/current index:
migration/package-stage36-build-comparison.json. No current GUI/performance acceptance.

Stage34 was an intermediate archive missing gettext-runtime/intl/COPYING.LIB; the
strengthened completeness check failed. It was never current. Stage36 includes the
runtime-specific LGPL text and passes. Exact archives/hash-verified notice extraction
are preserved in migration/results/native-source-audit. This closes missing native
license-text evidence, not full corresponding-source/relink clearance: Homebrew GLib
patches and private CPython native/source provenance remain to audit. Foreign target
attribution and actual execution remain unverified. No publication or user configuration
changes. CI notice/diagnostic/compare gates are prepared, not remotely executed.

Next: audit private CPython and exact modified native source/build metadata, finish scoped
output/publication review and assess current-package synthetic native smoke, then audit
the full objective. Earlier checkpoint notes below retain their original package scope.

Current checkpoint: stage36, private bounded SVG/raster/CSS/import staging and output
preflight. Rust213passed/1ignored; archive cold/warm, assets17/routes8 both engines,
security35/files12/doctor10/launcher11/notices17 (102crates)/discovery16 pass.
See RUST_SECURITY_AUDIT.md and package-stage36-build-comparison.json.

User-approved Rust-only development update (2026-10-03): the legacy Python MCP
implementation and paired comparison harnesses are retired. Frozen JSON contracts and
historical evidence remain, but no executable Python oracle or repeated parity runs are
part of the current plan. Use Rust regression/invariant tests, STDIO, package and native
acceptance. Required Python helpers/supervisor remain in runtime/ and rust/package/.
Historical preservation/parity requirements below are superseded by this user decision.
Recovery location is recorded in migration/python-retirement.json. See CONTRIBUTING.md.

# Rust migration report

User-approved scope update (2026-10-03): [current next plan](../RUST_NEXT_PLAN.md).
Historical live-incident investigation is deferred unless it recurs; clean-machine
installation will be checked later by the user; Windows port is backlog. Compatibility
and security are the main next gate; phase attribution is secondary and bounded.
Bundled-dependency provenance/licensing remains active. Stage27 is pre-namespace-fix;
rebuild and validate a new package from current source before final delivery. Historical
pending items below do not override these explicit user decisions.

Checkpoint: 2026-10-03, branch `codex/rust-migration`, reference HEAD `c50a924`.
**Migration is in progress. A local macOS arm64 CLI/STDIO candidate passes archive-install acceptance; the full replacement is not ready.**
The Python implementation is preserved and remains the usable server. Nothing has been
committed, published or switched in the user's MCP configuration. Nine owned synthetic GUI sessions were opened for acceptance: stage5/6 verified closed, stage12 closure unconfirmed, stage13 retained, stage18 crash UI retained, stage19 closure dialog retained, stage21 retained, stage22 retained and stage29 retained. All seven known remaining sessions are preserved.

Current stage29 native acceptance on the actual stage27 package passes **46 fixed checks**,
then the same 46 checks pass from captured SVG/PNG artifacts without reading the current
GUI workspace. This covers insertion, style repeat, duplicate, native Undo/Redo, reconnect
and approval/binding guards. Extended structural checks are recorded below; current native
benchmarks are measured separately. Initial selection refusals and protected sync destination
refusal occurred before mutation and are retained; they are not artwork disappearance.
The launch harness now records the binary SHA needed by structural acceptance.
Reports: `migration/native-stage29-comparison.json`,
`migration/native-stage29-captured-comparison.json`. Historical stage21 cause remains unknown.

Stage29 structural follow-up passes **38 additional native checks** for group, ungroup
and delete with Undo/Redo and final restoration to seven objects. Exact scenes, IDs, paint
order, RGBA and applied audit records pass again from saved captures. Reports:
`migration/native-structure-stage29-comparison.json` and its captured counterpart.
The successful distinct `group-selection-apply` phase is read explicitly by the verifier;
original selection and destination refusals remain retained. Transform/order follow-up is recorded below; historical causes are still pending.

Stage29 transform/order follow-up passes **22 + 16 additional native checks** and repeats
those checks using saved artifacts. The compound affine operation changes only the two
selected groups; native Undo/Redo/restoration preserves IDs, drawing and RGBA. Lower
reverses sibling paint order and preserves both subtrees, with exact Undo/Redo/restoration.
Identical overlapping copies do not prove different-color occlusion. Together with fixed
and structural checks, the current package has **122 scoped native checks**. Reports:
`migration/native-transform-stage29-comparison.json`, `migration/native-order-stage29-comparison.json`
and their captured counterparts. No current sequence reproduced historical stage21
disappearance; that observation does not establish its cause or a fix.

Stage29 current-package read-only live benchmark: **five alternating Python/Rust pairs,
80 real STDIO requests, 40 exact scene/selection/PNG comparisons, zero sampler errors**.
The scene remains seven objects. Median startup to initialized is Python **506.854 ms**,
Rust **6.882 ms**. Connect is **1108.579 / 1106.663 ms**, scene **201.468 / 206.803 ms**,
render **106.122 / 125.651 ms**. This proves lower startup latency for this fixture;
it does not prove general live operation speedup. Separate sampled MCP-tree RSS at scene
is **102.17 / 26.70 MiB**, with the shared warm GUI/supervisor tree about **282.09 MiB**
for both. RSS includes shared pages and sampling can miss transient children. GUI/bus
are warm; client roundtrip includes JSON, IPC, files and Inkscape. No exclusive IPC or
live mutation performance claim. Raw: `migration/results/live-process-benchmark-stage29`;
verified report: `migration/live-process-benchmark-stage29-verified.json`.

## Install and check the current local candidate

Current stage36 macOS arm64 archive includes the namespace fix and Rust-only layout.
Python MCP server and paired comparison harnesses are retired. Private live helpers,
supervisor, Objective-C bridge and D-Bus remain. Install Inkscape 1.4 or newer normally.
Ready archive users do not need Python, uv/pip, Homebrew or Rust. Local ad-hoc signing
remains; no OS security setting was bypassed.

Cold/per-call and warm/shell archive installation passes with empty PATH, actual CLI
pixels/export/resource, rollback/snapshots/approvals, original preservation and strict
complete-tree no-op checks. Doctor10, launcher11, notices17 and discovery16 pass.
No native/performance stage27 evidence is transferred to this binary.
Archive SHA-256: `bfd012af9c3f21d191e3d0d9009a344ffe7cc6cd0eca74b880869bbe6368bfb0`.
Binary SHA-256: `3ad57c33d9e4530d2ddb7b7bb66fdc7975b853deb4b5a0d360b29719f39145c4`.
Index: migration/package-stage36-build-comparison.json.

Extract into a new destination and configure an existing SVG workspace:

```sh
mkdir -p "$HOME/Applications/inkscape-mcp-stage36"
tar -xzf /Users/bm/Documents/repos/inkscape-mcp-server/migration/results/packages/inkscape-mcp-macos-arm64-stage36.tar.gz \
  -C "$HOME/Applications/inkscape-mcp-stage36"
cd "$HOME/Applications/inkscape-mcp-stage36/inkscape-mcp-macos-arm64-stage36"
./setup.sh --workspace /absolute/path/to/your/svgs
```

Setup detects the package and ordinary Inkscape installation, asks for missing paths,
runs read-only doctor and saves private local settings. Optional `--inkscape /path/to/Inkscape.app`,
`--live false` and `--engine shell` select explicit alternatives. Configuration is six
plain data lines, never sourced or evaluated as shell code. Failed setup preserves the
previous configuration; launcher errors go to stderr. Paths containing the POSIX `:`
separator or newlines are refused. Run setup again after moving the package/workspace.
Shell entry points bootstrap standard system paths even when the MCP client passes an empty PATH.

MCP configuration now needs only the launcher command:

```json
{
  "mcpServers": {
    "inkscape-rust-candidate": {
      "command": "/Users/bm/Applications/inkscape-mcp-stage36/inkscape-mcp-macos-arm64-stage36/run-mcp.sh"
    }
  }
}
```

This example was not written to the user's MCP configuration. The primary source flow
from a Git checkout is now:

```sh
./setup.sh                         # asks for an existing SVG workspace, builds and runs doctor
./run-mcp.sh                       # configure this absolute command in the MCP client
```

For noninteractive setup use `./setup.sh --workspace /absolute/path/to/svgs`. To use an
already unpacked artifact instead of compiling, add `--package /absolute/path/to/package`.
`--build` explicitly requests source compilation and cannot be combined with `--package`.
A checkout without `--package` builds a new private package for each setup run; older builds
and settings remain available. Paths/settings require no manual environment editing.

Source builds need **Rust/rustc 1.99.0**, native SDK/libxml2 build inputs, GLib headers on
macOS, D-Bus tools and (on GNU/Linux) patchelf. They reuse an existing exactly pinned
Python 3.12.14 helper environment or use installed uv to obtain that runtime and its six
wheel-only dependencies. Missing prerequisites give a diagnostic; the script does not install
a system package manager, accept OS licenses or alter shell profiles. It automatically sets
SDK/library/PATH values, forces the native Cargo target/output directory and supplies that
exact executable to the package builder. A ready archive requires none of those build tools.

Actual local acceptance uses an isolated source copy with no target artifacts, the existing
host build prerequisites and a reused pinned .venv. It proves build/setup/doctor and genuine
empty-PATH launcher STDIO; it is **not** a fresh-OS or uv-download-path test. Four native
CI jobs now include source setup and launcher acceptance, but no remote job was run/published.

Start a fresh MCP process to load candidate instructions; startup/reconnect never opens
Inkscape. For headless checks, create a named ordinary group/rectangle, apply a style batch,
preview, restore its snapshot and save a new SVG. Stage27's live readiness remains experimental;
any native check must explicitly request a separate synthetic managed session and preserve
existing drawings. Ad-hoc signing suffices for the documented local artifact; no credential
request or system security override is part of this installation.

## Compatibility matrix

| Component | Current evidence | Remaining work |
|---|---|---|
| MCP surface / arguments | 110 native tools, 7 prompts, 18 resources; review release matches all 16 frozen discovery configurations; 4351 current negative observations +547 positive coercion observations; prior4927 count includes duplicate finite package matrix | Rerun/bind affected final-candidate argument/family checks; no implication of exhaustive input coverage |
| Headless tool families / pipeline | All exposed families and 33 typed batch variants implemented; shared historical fixtures; fresh 122 edit scenarios pass | Final coverage manifest and targeted fault/security regressions for further changes |
| XML / filesystem safety | Existing bounded libxml2 and descriptor-based POSIX guards, XML/security/pipeline tests | Windows handle/reparse/rename port and native target validation remain incomplete |
| CLI / warm shell / artifacts | Stage27 actual archive passes cold/warm render/export, private helper/bus, snapshots/restore, approval, no-op, rollback and resource readback | Other-host execution and final sensitive/native fixtures |
| Live integration / native history | Native Rust IPC/session/guards and synthetic fault harnesses; review release passes 62 mutation observations; stage21 has 46 fixed native checks | Stage22 passes 122 scoped native checks; stage21 duplicate disappearance remains unexplained, and stage23 native acceptance is pending |
| Package / source onboarding | Stage23 source build/setup/doctor/empty-PATH launcher and cold/warm archive pass on this build-ready Mac | UV-download/fresh-OS/other-target execution, final native/live measurements and documented native source-offer gaps; Developer ID credentials are optional for local delivery |
| Measurement / delivery | Repeated stage21 real-STDIO headless/live measurements and raw evidence; review/installation/checklist docs | Stage23 paired headless direct/diagnostic runs now saved; exclusive phase attribution, current live measurements and gate-by-gate completion audit |

Rust discovery currently lists the frozen existing surface for contract work. Matching discovery and dispatch coverage do **not** prove complete behavior, argument/error
parity or installable runtime readiness. The binary does not spawn or wrap the Python MCP server.
There are now 110 native tool dispatches (including all 33 typed `apply_edits` variants
and headless `render_preview` with object/region support), covering every name in the maximum 110-tool profile. All 18 resources are native, including live/session/selection/view/events/operations. All seven prompt message
templates are compiled into Rust; no Python runtime is involved in rendering them.


### Stage28 transport-boundary diagnostic

Isolated STDIO read/write/flush observation adds50 exact frame/handler partitions and50
file/sync/process partitions over five pairs/160requests. Twenty inspection envelopes match,
sampler errors0. First flush-envelope subtraction was rejected because flush can overlap
client roundtrip; corrected read-to-first-write + residual is additive, write/flush retained
separately. CLOCK_MONOTONIC and Python perf_counter epochs differ; only durations are compared.
Not pure IPC/production/native evidence. Stage27 package remains unchanged. Details and
raw bindings: [phase measurements](reports/RUST_PHASE_MEASUREMENTS.md#stage28-transport-boundary-diagnostic).

### Stage27 filesystem boundary review (2026-10-03)

Review traced POSIX directory-descriptor descent, regular-file reads/hashes/locks,
exclusive/atomic publication, artifact resolution, fixed helper installation and XML flags.
No new confirmed production defect was found in the reviewed paths. A missing Rust regression
was added for FIFO refusal through regular_file/read_optional/hash_file/lock_file. A private
watchdog peer releases an accidentally blocking FIFO open and fails, avoiding an indefinite
test hang. Temporarily removing O_NONBLOCK only from regular_file produces the expected
failure in1second; exact reviewed source was restored, then full Rust199pass/1ignored,
all-target clippy-Dwarnings and fmt pass. The production methods did not change.

Actual existing stage27 package passes12 STDIO refusals: open_document/stat_artifact/artifact
read for a FIFO, directory, external final symlink and external parent symlink. Original
outside bytes and fixture entries remain unchanged; no .inkscape-mcp files are created.
This is fixed-fixture refusal evidence, not proof against every concurrent filesystem writer.
The package bytes remain unchanged; the only Rust source addition is the regression test.

Raw: migration/results/filesystem-review-stage27 (reviewed/fault-injected source snapshots,
failed injected test and passing full checks) and migration/results/special-files-stage27
(actual packaged wire replies and exact used harness). The formatted harness initially had
unused import/long line lint errors; source corrected, exact executed version preserved.
Existing package/headless/live/argument/measurement bindings remain immutable. Windows
reparse/rename-handle implementation and actual foreign target execution are still incomplete.

### Stage27 current argument and resource audit (2026-10-03)

Actual packaged binary passes all12 negative argument matrices:4351 observations/24 actual
STDIO starts, exact Python/Rust errors with no normalization and no workspace mutations.
Current trusted-reference audit covers110 tools/51 models; all36 finite-number paths match
the frozen contract. Prior4927 total included a second576-case package finite matrix;
current4351 is the twelve distinct matrices, not reduced coverage hidden by a new count.

Nine positive coercion suites pass547 observations with316 actually transformed requests:
edit, repeat, batch members, find, live events/mutation/loop/viewport/render. They use the
fresh unbundled release with the exact current package bytes because absent-engine fixtures
must not use package auto-discovery. Real CLI suites run with explicit vendor PATH; live
uses fixed owned synthetic peers. This proves named numeric/boolean string conversions,
not all possible signatures/defaults/coercions or native GUI semantics. live_connect has no
coercible numeric/boolean argument in its fixture: ordinary26 observations pass but the
zero-transformation guard correctly refuses coercion evidence; it is explicitly excluded.
Initial orchestration sent --cases instead of --suite; argparse rejected all10 harnesses
before server starts. That failed attempt is preserved separately.

Current wire resource-read map matches all18 frozen resource/template names to at least one
successful read, including document/artifact/live/runtime/prompts/workspace resources.
Named original suites validate specific content/MIME/URI/binding/error cases. Presence in
this map alone does not prove full resource compatibility. No new server/package source
change or GUI action occurred. Historical manifests remain immutable.

Reports: migration/results/current-arguments-stage27/comparison.json,
migration/results/current-coercion-stage27-verified/comparison.json (references actual
passing v2 attempts), migration/argument-schema-stage27-audit.json,
migration/current-resource-stage27-coverage.json. Review/source snapshots and failed attempts
are included in migration/current-arguments-stage27-evidence-binding.json.
Remaining: full final compatibility/security audit, current packaged/native live success and
history cause, precise phase attribution, Windows/foreign-platform and licensing gates.

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

### Stage27 synthetic live follow-up (2026-10-03)

The fresh normal release with the current package's exact binary SHA passes 15 finite
synthetic live families, 651 reported observations. Coverage includes lifecycle/reconnect,
status, arm/install/launch controls, task/context binding, polling/events, sync/render/export,
viewport/diff/discovery and mutation approval/audit/lost-reply behavior. Tests use owned
loopback peers or fixed fake process boundaries; live discovery additionally uses real
Inkscape CLI. No real GUI launch/edit/Undo or actual-package live acceptance is inferred.
The unbundled release location is required for absent-engine fixtures because ready packages
intentionally discover Inkscape/private bus dependencies independently of PATH.

All initial attempts are retained. Discovery first failed in the Python reference due to
missing CLI PATH; only discovery was repeated with the vendor path. Explicit differences
remain: loop10/diff18 encoded PNG representations, managed7 and mutation2 intentional
safety differences. Zero unexpected differences in accepted reports. Coverage now recognizes
*.trace.json as well as compressed raw logs; the corrected named union has 4619 actual tool
calls, all110 names called and109 with a successful reply. live_edit_selection has eight
socket-transport structural refusals and no positive result in this current set. Historical
stage22 native structural successes do not close its current native/package gate.

Reports: migration/results/current-live-synthetic-stage27-verified/comparison.json references
original passing attempts; migration/current-live-stage27-coverage.json maps actual envelopes.
Review snapshots are in migration/results/current-live-stage27-review/. Full behavioral,
argument/error/native coverage remains incomplete. Current production measurements, native
history investigation, foreign-platform/Windows and licensing gates remain open.

### Stage27 remaining headless review (2026-10-03)

Current byte-identical release location passes 16 headless families (1715 reported
scenarios/observations); actual package location passes 11 families (1462). Counts overlap,
and neither is a uniform test count. Real CLI action/path/render/export/profile comparisons
retain SVG/pixel/artifact/rollback assertions. Fault suites retain six action/seven path
explicit stronger discarded-audit differences, with zero unexpected differences.

Initial runs are preserved: missing vendor PATH prevented reference CLI scenarios;
package auto-discovery of /Applications intentionally invalidates historical absent-PATH
fixtures. Those fixtures use the byte-identical unbundled release; package cold/warm checks
separately prove auto-discovery. A later quality fixture lacked fc-list on PATH; only failed
subsets were rerun with the correct environment. The runner now handles the exact fault
report format and records terminal process state before parsing reports.

Coverage review fixed omission of gzip raw traces. The corrected union records 4145 actual
tools/call requests, 84/110 names with successful replies. Remaining 26 names are live tools;
this does not establish complete arguments/error/native/appearance coverage. Prior coverage
reports remain historical; use migration/current-headless-stage27-coverage-with-package-v2.json.
Selected passing attempts reference their actual original reports/raw directories in
migration/results/current-headless-{release,package}-stage27-verified/comparison.json.
Failed attempts are preserved; no normal server/package source changed in this cycle.
Native GUI, current production measurements, Windows/foreign-platform and licensing gates
remain pending. Review snapshots: migration/results/current-headless-stage27-review/.

### Stage27 affected-family follow-up (2026-10-03)

A finite 15-family cohort runs against the current packaged executable, preserving historical
harnesses/reports. Copies rewrite only executable/output/report path literals; the recovery
harness receives its existing required CLI arguments. The initial preparation rejected that
argument-only harness before any child launch; the corrected cohort is retained separately.
All fifteen pass: 1035 reported scenarios/startup recovery profiles across create, duplicate,
reparent, tile, grid, set, place, optimize, transform, adopt, repeat, fragment, batch members,
save and recovery. Comparisons and native-only safety refusals retain their original fixtures,
normalizers, raw replies and SVG/appearance checks. This is headless evidence, not GUI acceptance.

The wire-call coverage map joins that cohort with the proven byte-identical fresh-release
edit trace and current cold/warm archive traces. It records 1155 tools/call requests and
53/110 tool names with at least one successful reply. Success may be plan/dry-run/no-op;
changed=true is counted separately. The remaining 57 names are explicitly listed, with live,
read/inspection, CLI paths/actions, profiles/quality and export/frame families still requiring
current evidence or audit. No inference of full behavioral or argument coverage is made.
Reports: `migration/results/current-family-stage27-v2/comparison.json`,
`migration/current-family-stage27-coverage.json` (cohort only),
`migration/current-family-stage27-coverage-with-package.json` (named current evidence union).
The binary hash matches the current stage27 build report throughout. Old bindings unchanged.

### Stage27 current packaged fix (2026-10-03)

The builder consumes the exact fresh release tested in stage26, with no profiler linked.
The archive's stronger cold/warm checks compare complete workspace bytes, sizes and mtimes
around a genuine no-op; no transient audit-directory change is observed. 16 discovery
configurations match frozen contracts exactly, 10 doctor profiles, 9 notices/92 crates and
11 launcher cases pass. Stage27's 2453 FILES entries differ from stage23 only in the executable,
package metadata and license inventory. No native acceptance, performance transfer or other-host
execution is claimed. Current build report: `migration/package-stage27-build-comparison.json`.
Raw: package-stage27-cold/warm, doctor-stage27, launcher-stage27 and discovery-stage27 under
`migration/results/`. Historical source/review/native/measurement bindings remain unchanged.

### Stage26 sync measurement and DOM no-op fix (2026-10-03)

Individual file/directory sync spans explain most of the measured Rust logical-file region:
single-edit medians 41.173/18.532 ms. They also exposed a transient proposed-record write
on no-op, which older final-state checks missed. A deterministic directory-mtime regression
failed before the fix; 22 pure DOM families now stage through `apply_dom` and decide no-op
before audit creation. CLI-staging actions/paths/fit preserve the pre-dispatch audit gate.
Snapshots, previews and working publication remain after durable intent; sync protections
are retained. [Details](reports/RUST_PHASE_MEASUREMENTS.md) separate pre-fix timing from post-fix
validation. Fresh release no-op STDIO preserves the complete tree/bytes/sizes/mtime_ns.
Rust 198 passed/1 ignored and clippy pass; new packaged/final-candidate validation remains.
The existing stage23 archive is historical and still contains the transient-write behavior.

### Stage25 paired Python/Rust phase follow-up (2026-10-03)

The explicit Python diagnostic entry point now wraps fixed filesystem/process APIs and
binds worker events through ContextVars. Five alternating pairs make 160 actual requests;
50 Python middleware and 50 Rust handler partitions validate, complete small/100-object
inspection envelopes match, and sampler errors are zero. [Phase details](reports/RUST_PHASE_MEASUREMENTS.md)
retain differing framework/file API boundaries. Rust single-edit logical-file median is
64.510 ms versus Python 1.041 ms; individual file/directory synchronization latency remains
unmeasured. Do not infer identical IO scopes or remove sync/path protections from this data.
Pure IPC, unguarded APIs, warm/live and final production attribution remain pending. The
normal Python server, Rust sources and stage23 archive are unchanged by this diagnostic.

### Stage24 diagnostic phase follow-up (2026-10-03)

[Phase measurements](reports/RUST_PHASE_MEASUREMENTS.md) use isolated instrumented Rust copies,
not a new distributable candidate. Two five-pair experiments are preserved: initial immediate
nested logging was found to contaminate outer file spans; corrected buffering emits records
after the handler timestamp. The corrected 50 Rust handlers have exact bounded interval-union
partitions, complete matching inspection envelopes and zero sampler errors. Render medians:
252.987 ms Inkscape process envelope, 8.612 ms guarded logical-file API, 0.511 ms handler
remainder. Outside-handler time includes diagnostic output, IPC and serialization; pure IPC,
Python/warm/live phases and unguarded file calls remain pending. No production speedup or
native-history fix is claimed. Stage23 archive/source and historical bindings are unchanged.

### Stage23 live follow-up (2026-10-03)

The new packaged binary passes 26 connect, 46 loop and 62 mutation/fault observations
through owned synthetic transports (**134 observations**). There are no unexpected
semantic differences. The mutation report retains two intentional stronger lost-reply
uncertainty differences; loop retains ten encoding differences. These are transport tests,
not native history acceptance and not evidence that the stage21 disappearance is fixed.

An explicitly authorized ninth isolated managed launch attempt used root
`/private/tmp/imcp-native-60hza24y` and the exact stage23 runtime/supervisor/bus/bridge.
It terminated with the existing guard diagnostic:
`MCP startup refused: no primary monitor; unlock the Mac and retry.`
The bus and supervisor terminated after that GUI process exited; a current process inventory
shows no surviving process at that owned root. No second launch was attempted, no Mac
security/session setting was changed, and the earlier eight launched GUI sessions remain
historical evidence. Current native acceptance is **failed/incomplete**, not transferred
from stage22. An available native primary monitor is required for that remaining gate;
independent implementation/validation can continue without new authorization.

Reports: `migration/live-{connect,loop,mutation}-stage23-comparison.json`,
`migration/live-stage23-followup-comparison.json`,
`migration/native-stage23-launch-comparison.json`. Raw transport directories are
`migration/results/live-{connect,loop,mutation}-stage23/`; the failed owned launch logs,
original session record, MCP trace and exact harness copies are preserved in
`migration/results/native-gui-stage23/`. Exclusive phase attribution, Windows and other
completion gates remain open. This refusal is distinct from the earlier native history bugs.

### Stage23 headless measurement follow-up (2026-10-03)

The exact stage23 packaged binary ran five alternating Python/Rust pairs directly and
five diagnostic pairs through a fixed vendor-CLI wrapper: **360 real STDIO requests**.
All 60 full inspection envelopes match after only request-bound document-ID substitution
and decoding the JSON text representation; no result fields are discarded. Fixtures include
100, 10,000 and 100,000 objects in the direct run. Original SVG bytes remain unchanged.

| Median direct measurement | Python | Rust |
|---|---:|---:|
| Process start through actual MCP initialize | 824.42 ms | 20.18 ms |
| Inspect one-object SVG | 13.44 ms | 2.30 ms |
| Inspect 100,000 objects | 15,322.69 ms | 1,966.01 ms |
| Single edit, including automatic preview | 693.59 ms | 760.55 ms |
| Atomic batch, including preview | 505.21 ms | 582.10 ms |
| Render | 255.09 ms | 264.27 ms |
| Export | 255.82 ms | 261.34 ms |

Startup and inspection are faster in this fixture; these observations do not establish a
general edit/render/export speedup. The small-inspection sampled tree RSS medians are
102.41/18.86 MiB, but the 100,000-object response reaches sampled medians of 1184.70/1216.94
MiB: large output memory is still high in both implementations. Tree RSS sums shared pages.
Python run 0 has one retained `rusage` ESRCH sampling error (PID 40919), so its direct
memory observations are explicitly marked incomplete. The strict validator refused this
sampling gap; the explicit record-errors mode subsequently verified every response without
removing the error. Sampling gaps reach 337.56/317.41 ms; these are observed, not exact peaks.

The separate diagnostic run has zero sampling errors. Vendor CLI wall medians are
247.44/253.50 ms for render and 247.78/253.08 ms for export, against wrapped roundtrips
270.27/285.68 and 272.15/283.53 ms. Each run retains CLI arguments, intervals and child CPU.
The outside-CLI remainder still contains server work, IPC, files and wrapper bootstrap;
exclusive wall-time attribution remains incomplete. Do not subtract independently computed
medians or interpret CPU/physical disk counters as exclusive wall phases. Measurements use
warm local caches without cache flushing or exclusive machine load; no GUI was launched.

Raw: `migration/results/process-benchmark-stage23/` and
`migration/results/process-benchmark-cli-stage23/`. Reports:
`migration/process-benchmark-stage23-comparison.json`,
`migration/process-benchmark-cli-stage23-comparison.json`,
`migration/process-benchmark-stage23-followup.json`. Exact harness copies and hashes are
retained separately from the immutable source-onboarding checkpoint. No native GUI evidence
is transferred to stage23, and the full migration remains incomplete.

### Stage23 source-first onboarding (2026-10-03)

`scripts/build-local-package.sh` implements the fixed native build recipe; setup invokes it
from a checkout and configures the resulting package. Cargo target and artifact path are
explicit, preventing user CARGO_TARGET_DIR/CARGO_BUILD_TARGET settings from selecting a
stale or cross-target executable. Builders accept optional developer `--binary`, while
existing callers keep their previous default. macOS SDK paths and installed GLib/D-Bus
locations are detected; helper Python version, native architecture and six pins are checked.

A fresh source copy with an empty target directory builds successfully with intentionally
wrong external Cargo target/output settings. The canonical implicit release artifact is
absent; packaged bytes equal the newly compiled explicit native artifact. No unrelated output
directory was created. Setup then passes doctor and saves private settings; genuine launcher
STDIO initializes, lists the exact default contract and opens/inspects an owned SVG with empty PATH.

An initial launcher attempt exposed missing system-tool PATH bootstrap (`dirname` failed);
all three shell scripts now establish standard system paths before any external command.
The failed log and initial sources are preserved; the fixed build and launcher pass.
The stable stage23 archive passes cold/warm install, ten doctor profiles, nine notice checks,
eleven empty-PATH launcher checks, 16 exact discovery configurations and 122 edit scenarios.
Source preflight is tested on this Mac using existing dependencies; the uv download branch,
foreign hosts and fresh operating systems are not claimed as exercised. No GUI was opened
or restarted in this source-onboarding checkpoint. Native/performance evidence remains stage22.

Raw evidence: `migration/results/source-onboarding-stage23/`. Summaries/bindings:
`migration/source-onboarding-stage23-comparison.json`,
`migration/source-onboarding-stage23-evidence-binding.json`,
`migration/package-stage23-build-comparison.json`. Historical stage22 and review bindings
remain unchanged. Full migration is incomplete; final native/history, measurement, Windows
and other-host/source-offer gates remain explicit in the checklist.

### Stage22 review fixes and fresh native follow-up (2026-10-03)

[The review](reports/RUST_REVIEW.md) fixes bounded pipe drains, failed nonblocking setup and
post-dispatch audit-loss handling. Its release binary SHA-256 is
`ea78a3430065316c47fcb0362d21feba62e29b93a65c1dd3d854fd418af97ccb`.
Review/archive/launcher evidence is scoped in `migration/review-cycle1-comparison.json`;
source snapshots and failed logs remain preserved separately from this native follow-up.

A fresh explicitly authorized synthetic session uses supervisor **30626**, native PID
**30645**, root `/private/tmp/imcp-native-l3sosx3s`, window UUID
`fc3f641c-5f10-4903-84d8-e313db7e5c7b` and document UUID
`e2453f0b-1f33-4730-8675-0e01e629bce1`. Actual parent/full executable path and manifest
are checked; structural phases also verify the packaged executable hash before dispatch.
The original stage21 drawing/evidence is unchanged. No native results were merely transferred.

Current-package native verification passes **46 fixed insertion/style-repeat/duplicate checks**,
**38 group/ungroup/delete checks**, **22 transform checks** and **16 lower/order checks**.
Each family includes native Undo/Redo, exact tree/ID/pixel comparisons and restored-fixture checks.
The captured-only rechecks also pass: 24 + 26 + 10 + 10 source/pixel artifacts are retained
in their respective evidence directories. These are scoped fixtures, not exhaustive live tests.

Five alternating Python/Rust read-only pairs on that warm GUI pass **40 comparisons** and all
**80 requests**; all ten scene captures contain seven objects. After the trials, native Select All
and the next read-only capture still contain seven objects; guarded group then applies to eight.
**Stage21's disappearance did not reproduce in this attempt. Its cause remains unproven.**
The final stage22 drawing has been restored to its seven-object synthetic fixture and retained.

Median ms (Python / Rust): startup **509.78 / 6.95**, connect **1106.52 / 1123.86**,
documents **19.30 / 24.77**, bind **349.31 / 208.47**, status **179.41 / 182.45**,
selection **39.47 / 51.83**, scene **221.03 / 253.64**, render **107.90 / 110.14**.
Sampled scene MCP-tree RSS is **102.55 / 26.98 MiB**; separately sampled GUI trees are
**272.78 / 274.86 MiB**. Independent maxima must not be added. Warm GUI, filesystem/cache
state and retained background sessions limit interpretation; exclusive phase attribution and
cold GUI performance are not established. The raw sampler-error inventory is part of the report.

Guarded preflights stopped attempts with empty selection after native Undo/Redo and a
capture destination collision before any editing request. Existing sync files were preserved;
order acceptance used a new owned workspace. UI capture once reported ScreenCaptureKit failure
after Redo; the action was not repeated and exact-tree readback confirmed it had completed.
A coordinate click reported noWindowsAvailable; verified native PIDs survived, and subsequent
fresh UI binding/keyboard selection identified original then duplicate before lower dispatch.
These UI/harness incidents are not counted as duplicate disappearance or Rust mutation failures.

Reports: `native-stage22-comparison.json`, `native-structure-stage22-comparison.json`,
`native-transform-stage22-comparison.json`, `native-order-stage22-comparison.json`, their
captured-only counterparts, `live-process-benchmark-stage22-comparison.json` and
`native-stage22-evidence-binding.json`. Raw files are under `migration/results/` in
`native-gui-stage22/`, `native-structure-stage22/`, `native-order-stage22/` and
`live-process-benchmark-stage22/`. Automatic source package building, Windows, other-host
execution and remaining measurement/history investigation gates remain in the finite checklist.

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
  scripts/history/python/migration_process_benchmark.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage21 \
  --output migration/results/process-benchmark-stage21-new \
  --repeats 5 --counts 100 10000 100000
# Run the diagnostic after the direct run has ended, into a different new directory.
PATH=/Applications/Inkscape.app/Contents/MacOS:$PATH .venv/bin/python \
  scripts/history/python/migration_process_benchmark.py \
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
.venv/bin/python scripts/history/python/migration_verify_native.py --output migration/results/native-gui-stage12 --report migration/native-stage12-captured-comparison.json --captured-only
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
with `scripts/history/python/migration_verify_large_response.py --output migration/results/process-benchmark-stage9
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

`scripts/history/python/migration_process_benchmark.py` now benchmarks the packaged optimized stage8 binary
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
  scripts/history/python/migration_process_benchmark.py \
  --package migration/results/packages/inkscape-mcp-macos-arm64-stage8 \
  --output migration/results/process-benchmark-stage8-new --repeats 5
# Separate diagnostic trial; never mix this with direct timing comparisons:
PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" .venv/bin/python \
  scripts/history/python/migration_process_benchmark.py \
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

### Packaged native insertion, Undo/Redo and reconnect (2026-10-02)

One explicitly authorized isolated managed session was launched from candidate stage5,
using private HOME/profile/workspace, packaged Python supervisor, prebuilt bridge and private
D-Bus. Real context discovery found exactly one synthetic blank drawing. The packaged insertion
helper authored an editable named vector group containing a blue rectangle. Native Edit → Undo
removed every inserted ID in one step; native Edit → Redo restored each inserted SVG subtree
byte-for-byte and the rendered PNG RGBA pixels exactly. Reconnecting over fresh MCP STDIO
cleared the task binding, and an approved style request before document selection was refused.
The official vendor executable SHA-256 remained unchanged. The owned test window is retained
for further acceptance; no user drawing/window was changed or closed.

Evidence: `migration/native-gui-comparison.json`, raw wire/SVG/PNG captures in
`migration/results/native-gui-acceptance/` and its recorded isolated workspace. Launch harness
`scripts/history/python/migration_native_gui_acceptance.py` refuses a second launch while that ownership record
exists. `scripts/history/python/migration_native_capture.py` attaches only to the existing private bus and checks
exact window/document IDs and allowed blank/dirty window titles before capture. An initial
Redo capture assertion incorrectly required an unchanged title; its failed trace is retained,
and the capture was corrected to allow Inkscape's observed leading dirty `*`, preserving exact
identity checks. This is actual GUI/package/helper acceptance for insertion, single-step Undo,
Redo and reconnect refusal, not full native coverage. Selected style/text/structural edits,
document-switch guards, uncertain/crash cases and socket-helper runtime remain unverified.

The subsequent repeated-style GUI test exposed an empty native Undo entry in stage5:
after two identical fill calls the first Undo left the SVG byte-identical and only the second
removed the style. `migration/native-style-comparison.json` deliberately retains that failed
result. Investigation with the actual vendor inkex/private Python found that replacing an
identical root discarded the document-level comment/PI; inkex detected a serialization change.
The shared helper now compares exact root serialization after full planning/validation and keeps
the original document tree for an exact no-op. Two regression cases cover no-op prolog/root
preservation and a changed edit. The actual private inkex probe changed from 1149 output bytes
to zero for the same validated no-op. An overlay of this fixed helper in the owned test profile
passed the native test: one Undo removed the changed fill after a repeated identical call.
`migration/native-style-fixed-comparison.json` records this scoped overlay evidence. Stage5's
archive remains unchanged and still contains the faulty helper; stage6 rebuild/archive-install and ten-profile doctor acceptance have passed.
The stage6 archive contains the fixed helper; its full fresh native session has not yet been tested. Live Operation Records still exist for every approved call, matching the Python
reference; an initial no-record assertion was corrected and its failed trace retained.

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
`migration/results/packages/inkscape-mcp-macos-arm64-stage6/`, with archive
`migration/results/packages/inkscape-mcp-macos-arm64-stage6.tar.gz` (46,232,949 bytes; SHA-256
`77e92c31d0ab43120daa0a344043c8ccca760cdcfe002614a32b21011afc1757`). It contains 2258 regular
file hash entries, 129,945,135 regular bytes. `migration/package-build-comparison.json` now
points to this candidate; older candidates remain available as historical evidence. Installation
from the actual new archive again passes clean empty-PATH runtime/helper/private-bus/STDIO
full discovery, no-op, batch/rollback, approval refusal, original preservation, SVG save/resource
and real PNG preview/export pixel checks. Raw acceptance remains in `migration/results/`.

Reproduce/test:
```sh
migration/results/packages/inkscape-mcp-macos-arm64-stage6/bin/inkscape-mcp --doctor
.venv/bin/python scripts/history/python/migration_doctor_acceptance.py --package migration/results/packages/inkscape-mcp-macos-arm64-stage6
.venv/bin/python scripts/history/python/migration_package_acceptance.py --archive migration/results/packages/inkscape-mcp-macos-arm64-stage6.tar.gz
```
Configure normal MCP execution with that candidate's `bin/inkscape-mcp` and a synthetic writable
`INKSCAPE_MCP_WORKSPACE_ROOTS`, without `--doctor`. The user's current MCP configuration is
unchanged. Native packaged GUI/supervisor/effects/Undo, signing/ABI/license audit, other platform
builds, complete behavioral coverage and new phase/peak-memory measurements remain required.
All 165 Rust tests, all-target clippy/format, Ruff (302 files), mypy (122 sources) pass; LLM indexes
regenerated with frozen text unchanged. The full migration goal remains active and incomplete.

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

`scripts/history/python/migration_build_macos_package.py` refuses an existing output directory, copies only
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
.venv/bin/python scripts/history/python/migration_build_macos_package.py --output migration/results/packages/<new-name>
.venv/bin/python scripts/history/python/migration_package_acceptance.py --archive migration/results/packages/inkscape-mcp-macos-arm64-stage3.tar.gz
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
`.venv/bin/python scripts/history/python/migration_live_launch_acceptance.py`.

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
`.venv/bin/python scripts/history/python/migration_live_arm_acceptance.py`.

All 164 Rust tests, all-target clippy/format, Ruff (295 files), mypy (122 sources) pass.
LLM indexes regenerated, frozen text unchanged. Current count: 109 native tools / one pending
(`live_launch`), all 18 resources and 7 prompts native. Ready packaged runtime/supervisor/bridge/bus,
doctor, current-Mac installation, other platforms, native GUI Undo/Redo, full argument/failure
coverage and fresh phase/peak-memory benchmarks remain required. Full goal stays active.

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

## Current dependencies and build

The current-Mac archive is stage15, built with Rust 1.99.0, CPython 3.12.14 and official
Inkscape 1.4.3. It includes a private Python runtime only for fixed inkex helpers/supervisor,
a prebuilt Objective-C context module and relocated private D-Bus/GLib dependencies. The
Rust MCP server never starts the reference Python MCP server. End users need Inkscape and
the archive, without a compiler, Python/pip/uv or Homebrew. Python remains the reference.

See [native packaging inputs/jobs and exact limits](reports/RUST_PACKAGING.md). Four native Linux/
macOS jobs are prepared; no remote or local Linux/Intel package result is claimed. Native
Windows filesystem/runtime support and jobs remain pending. Ad-hoc assets are available;
Developer ID/notarization, full ABI/source/license audit and reproducibility remain incomplete.
No security settings were bypassed and no user configuration/GUI was changed by packaging.

The local developer build uses system libxml2 through the Xcode SDK stub:

```sh
export LIBXML2="$(xcrun --show-sdk-path)/usr/lib/libxml2.tbd"
~/.cargo/bin/cargo build --release --locked --manifest-path rust/Cargo.toml
~/.cargo/bin/cargo fmt --manifest-path rust/Cargo.toml --check
~/.cargo/bin/cargo clippy --locked --manifest-path rust/Cargo.toml --all-targets -- -D warnings
~/.cargo/bin/cargo test --locked --manifest-path rust/Cargo.toml
```

## Measurements and validation

Five Python processes were measured to completed MCP initialization over real STDIO.
Median startup: **664.6 ms** (individual runs 517.3, 664.6, 680.8, 686.3, 662.5 ms).
See [raw-derived summary](../../migration/python-baseline-summary.json) and
[reproduction commands](../../migration/README.md). These measurements are end-to-end; they
do not isolate server CPU, IPC, IO or Inkscape. The new checkpoint comparison below is separate from this earlier baseline.
The baseline also records inspect/open on 100/10,000/100,000-object SVGs, one edit,
an atomic batch, no-op, approval refusal, failed-batch rollback, snapshot restore,
save/resource readback and real PNG render/export. Complete raw responses are retained.

Five new Python runs and five release Rust runs use the same real-STDIO probe on this Mac.
See `migration/checkpoint-benchmark-comparison.json`, both checkpoint summaries and
`migration/release-checkpoint-build.json` for all samples, build flags/hash and system libraries.
These are sequential local warm-cache measurements of the 50-dispatch checkpoint before
the reparent/namespace-parser changes below; the retained release binary hash identifies it.

| Median measurement | Python | Rust release |
|---|---:|---:|
| Spawn through completed MCP initialization | 519.40 ms | 35.75 ms |
| Server RSS at initialization (single snapshot per run) | 92,336 KiB | 12,688 KiB |
| Inspect 100 objects | 26.10 ms | 2.76 ms |
| Inspect 10,000 objects | 1,485.18 ms | 191.49 ms |
| Inspect 100,000 objects | 16,241.94 ms | 2,030.83 ms |
| Single set_fill including operation previews | 532.00 ms | 588.95 ms |
| Atomic two-member batch including previews | 525.63 ms | 591.89 ms |
| Public render_preview | 264.27 ms | 277.52 ms |
| PNG export_document | 267.90 ms | 273.82 ms |

Startup and read roundtrips are lower in this fixture; edits and engine exports are slower.
No overall speedup or peak-memory claim follows. Request timings include client JSON parsing,
IPC, IO and Inkscape. Phase CPU/IO/IPC attribution, peak server/child memory, packaged runs and
live measurements remain pending. `live_status` is the sole response-status mismatch in these
traces because Rust returns migration-pending; its timing is explicitly not comparable.
The probe verifies original preservation, no-op snapshots, rollback and approval refusal;
full output parity is established separately by acceptance suites, not the timing script.

- Python full suite with real Inkscape CLI in PATH: **1288 passed, 12 skipped**.
- Ruff lint and format check passed (302 formatted files); strict mypy passed (122 source files).
- Rust tests: **165 passed** (XML including depth/entity amplification refusal, entity text,
  CSS/optional argument validation, no-follow/root swap, timeout, no-op/restore, atomic batch,
  approval/tampered manifest, audit-failure rollback, save/hard-link/parent-swap protection,
  PNG transparency/content truth, dimension estimates/caps, SVG href relocation,
  root coordinate mappings, region overflow/background validation, persisted frame numbering,
  non-clobber/link exclusion, bounded directory enumeration, snapshot comparison preflight and consistency signal ties and eager palette/scope validation).
- Rust format and clippy with `-D warnings` passed. Python migration scripts pass Ruff.
- Rust discovery through real STDIO: **16/16 exact matches**, zero fields normalized.
- Native read comparisons: **71 scenarios, zero differences**, including complete tree, layers,
  objects, styles, fonts, assets, aggregate inspect and registry index. Fixtures include Latin-1,
  mixed text, CDATA/entities, layer visibility/locking, geometry, transforms and asset references.
- Native edit comparisons: **122 scenarios, zero differences**. Exact meaningful outputs, errors,
  snapshot byte hashes/sizes and Operation Records are compared. Minted IDs/timestamps are
  validated and explicitly bound; render comparison decodes actual PNGs to RGBA pixel hashes.
  Additional recolor fixtures include overlapping scopes without duplicate replacements,
  presentation/style precedence, gradient stop colors, sequential palette cascades, no-op,
  invalid CSS named colors and mixed style/color atomic batches. Native CSS palette keywords
  are frozen from the reference and compiled in; no Python runtime is involved.
  Text cases verify literal markup/ampersands, targeted tspan sibling tails, collapsed runs,
  empty text, no-op, control/length refusal and mixed text/font batches. Font responses compare
  actual Arial own-family coverage, Japanese missing glyphs/suggestions and unavailable families.
  Transform comparisons cover parent-space prepending, uniform/nonuniform scale, rotation with
  and without a centre, six-decimal formatting, scalar coercion, zero translation, invalid
  factors/nonfinite values, target/centre error precedence and mixed atomic batch rollback.
  Canvas checks compare existing viewBox preservation, explicit retargeting, synthesized/repaired
  malformed or degenerate boxes, percentage/rem fallback, no-op, bleed/color validation,
  repeated bleed with unique IDs and unchanged original/child geometry. Bleed is a namespaced
  vector rect after defs and its tail, behind existing artwork; native tests preserve mixed text.
  Engine-derived fit uses fixed `--query-all` on a private bounded root-only identity probe
  (outer DTD omitted as in Python); fallback stages a safely read copy with relative assets rebased.
  Fit cases compare working SHA-256 and operation-preview PNGs for transformed groups, curves
  with stroke, mm dimensions, slice alignment, percentages and empty-document refusal. Repeated
  fits with numeric dimensions are no-op; percentage fallback remains non-idempotent like Python.
  Engine-unavailable/timeout fit acceptance and more asset/entity cases still need dedicated runs.
  Rename checks compare ID/label results, new/existing label namespaces, href/xlink href, paint-server
  and connector references, no-op labels, same/conflicting/invalid IDs, bounded control-safe labels
  and mixed batches. Exact working hashes and actual before/after pixels match on these fixtures.
  Rust refuses ID renames mentioning the old ID in stylesheet text or timing/accessibility
  attributes before mutation; Python remaps only selected attributes and can leave those references
  stale. This intentional conservative safety difference is not full stylesheet/SMIL support;
  Direct fill/stroke hex-color values identical to the old ID are now rejected before mutation
  as ambiguous; a native regression proves byte preservation. Broader reference forms remain pending.
  Delete checks cover missing/whitespace approvals (even for no-match), explicit approval,
  missing/duplicate IDs, root refusal, group-before-descendant ordering and tail removal.
  `affected_ids` preserves the reference pre-edit subset (including duplicates/descendants);
  the mutation summary reports the actual sequential removals. Delete escalates an atomic
  batch to HIGH; approved mixed batches, failed-batch rollback, snapshots, audit records,
  exact working hashes and actual preview pixels match. Native delete target lists cap at 4096.
  Palette mappings cap at 256. Remaining batch families (beyond style, color, text, font, transform, canvas, rename, delete, primitive creation, gradients, group creation/mode, instances, grouping, reparenting, duplicate and tile) are still pending.
- Vector authoring comparisons: **93 scenarios, zero differences** in `create-comparison.json`.
  All eight native primitives (rect/circle/ellipse/line/polygon/polyline/path/text) compare public
  results, analytic bbox, exact working/snapshot byte hashes, audit records and real PNG RGBA.
  Explicit IDs, first direct layer selection, explicit transformed parents, rounded corners,
  inline fill/stroke/width, literal and empty text, scalar coercion, invalid geometry/style/IDs,
  missing parents and mixed creation/style batch rollback are covered. Point lists cap at 100,000,
  path data at 200,000 characters and text at 100,000, matching reference bounds. Geometry path
  commands are charset-validated as in Python, not geometrically parsed. Automatically minted
  IDs, point/path caps and additional namespace/parent edge cases still need expanded wire fixtures.
  Linear/radial gradients use shared native ID/namespace helpers, first-root-child defs creation
  and eager 1..1000-stop validation. Stops compare scalar/percentage offsets, color canonicalization,
  optional opacity, percentage/exponent/negative coordinates and radial focal coordinates.
  Actual gradient fills, created defs bytes, invalid values/caps/IDs and atomic gradient-to-fill
  batches plus rollback match. Native regression proves leading text/comment order, inherited
  SVG namespace on stops and collision refusal without DOM mutation. Automatic gradient IDs,
  imported/nonstandard defs namespace cases and more scalar/error combinations remain pending.
  Group checks compare named ordinary-group default, explicitly requested layers, shared direct-layer
  parent selection, child creation, mode-to-layer/back, true no-op, preserved children/styles/
  transform/order, bounded labels, invalid IDs/parents/targets and mixed group/shape/mode batches
  with rollback. Stylesheet mode changes and layer creation refuse before working mutation;
  a matching mode remains no-op even with stylesheets, matching Python. ID/parent/SVG namespace
  and Inkscape attribute helpers are shared with primitive creation and rename. More namespace/
  legacy metadata and automatic group-ID wire cases remain pending.
  Native create_use writes both plain and namespaced href to existing same-document safe IDs.
  Shared namespace helpers preserve inherited xlink prefixes or add an explicit fresh prefix.
  Instances compare default/explicit parents, numeric positions, bounded 2000-character SVG
  transform grammar, source-style propagation, existing alternate xlink prefix, exact bytes,
  preview pixels and instance-after-create batches/rollback. External/fragment/url tokens,
  missing href/parent, conflicts, invalid transform/functions/injection/cap and nonfinite positions
  refuse without working changes. Automatic use IDs, cycles/legacy link forms and additional
  transform grammar/scalar edge cases still need expanded fixtures.
  Native group_objects compares adjacent same-parent grouping, exact child order/bytes,
  actual identical before/after pixels, IDs/errors, mixed creation/group batches and rollback.
  It shares the namespace, ID, paint-order and stylesheet helpers. The reference allows
  appearance-changing moves; Rust intentionally refuses different parents, changed paint order,
  stylesheets and meaningful text tails before working-file mutation. Five independent native
  STDIO guards prove unchanged source/working bytes and no snapshots in
  `results/create-acceptance/native-grouping-guards.json`. These additional refusals are
  recorded separately, not excluded from common scenario comparison. Ancestor compensation
  and broader structural operations remain pending.
- Reparent/XML comparisons: **42 scenarios, zero differences** in `reparent-comparison.json`.
  Native reparent_object and its atomic batch member share affine parsing, multiplication,
  inverses and Python-compatible 17-significant-digit matrix serialization. Compensation
  covers translate/scale, rotation centres, skews, matrix composition and exponent values;
  complete results, exact working/snapshot hashes, Operation Records and real before/after
  RGBA match. Successful standalone moves keep their preview pixels unchanged. Same-parent
  preserve mode is true no-op; a neutral legacy XML-only move also matches. Mixed tails move
  intact in preserve mode. Batch style/reparent changes and later-member rollback match.
  Missing/root/cyclic targets, non-group destinations, stylesheet/inherited style/effect/lock
  changes, external subtree/ancestor href or quoted URL references, paint order changes,
  singular transforms, CSS transforms, nested viewports, invalid arity/numbers/nonfinite
  transforms refuse before working mutation. Subtree membership/context checks use pointer
  sets, avoiding a quadratic whole-document membership scan. SVG transforms cap at 200,000 bytes.
  Inherited namespace bindings are reconciled after reattachment; conflicting destination
  prefixes mint nsN bindings matching Python, with exact SVG bytes and instance pixels checked.
  Native-only `results/reparent-acceptance/native-legacy-guards.json` proves five intentional
  refusals beyond preserve_appearance=False's legacy Python move behavior: changed transform,
  inherited style, paint order, meaningful tail loss and non-group destination. Every refusal
  preserves source/working bytes and creates no snapshot. This follows the required structural
  appearance safety and is not a claim of unrestricted legacy error/behavior parity.
  Undefined attribute/element prefixes now fail through an owned libxml2 parser context checking
  both wellFormed and nsWellFormed; the non-null-document return alone is insufficient. Two actual
  STDIO XML cases match Python. Read (71) and save/validation (50) suites were rerun successfully.
  More namespace/alias/scalar and moved-subtree context edges remain pending.
- Duplicate comparisons: **19 scenarios, zero differences** in `duplicate-comparison.json`.
  Native duplicate_object and its atomic batch member copy a subtree immediately after its
  source, preserve tail text/comment order, mint collision-checked six-hex ID suffixes, and
  rewrite intra-clone href/xlink/connector/paint/style references through the same native helper
  as rename. The original and external references remain intact. SVG namespace adoption removes
  redundant inherited declarations; local/inherited xlink prefixes compare exactly.
  Scenarios cover explicit/automatic top IDs, gradients, masks/clip paths/markers, transformed
  groups, mixed text, cloning an existing clone, copy-and-move batch plus rollback, missing/root/
  conflicting/invalid IDs. Full public results, snapshots and sizes, audit records, whole working
  SVGs and snapshot byte hashes after narrowly bound minted IDs, and real RGBA pixels match.
  Fresh IDs are validated at corresponding subtree positions for uniqueness, prefix/six-hex
  format and shared suffix, then bound to stable six-hex values. No SVG serialization or field
  omission is used in comparison. Explicit leaf copies therefore retain raw byte-equivalence;
  group/auto copies have only their validated random ID occurrences bound. Full raw wire/SVG
  evidence is retained in `results/duplicate-acceptance/`.
  Five independent native STDIO guards prove source/working bytes and snapshots remain unchanged
  when document stylesheets, intra-clone SMIL/accessibility references, quoted paint references or
  hex-color/ID ambiguity cannot be safely remapped. Python's partial remapper allows these forms;
  these stronger refusals are documented separately in `native-reference-guards.json`, not hidden
  as normalization exceptions. Full support for those reference forms and rarer namespace/entity/
  collision/limit combinations remains pending. The rename/edit suite was rerun: all 122 scenarios
  still match after sharing the reference helper. Descendant suffix collision retries cap at 128; a generated top-ID collision fails like Python.
- Tile comparisons: **27 scenarios, zero differences** in `tile-comparison.json`.
  Native tile and its atomic batch member reuse duplicate's namespace/tail/ID/reference kernel,
  carry one occupied-ID set across the row-major grid, and prepend parent-space translations.
  Complete results, whole SVG and snapshot bytes after validated minted-ID bindings, manifest
  sizes, audit records and real PNG RGBA match. Fixtures cover groups with intra-clone and external
  references, existing transforms, mixed text/tails/comments, one-row/column, scalar count coercion,
  negative/fractional/zero offsets, 1x1 no-op (including stylesheets), real 128- and 1024-cell grids, batches
  and rollback. Invalid counts/product overflow, >1024 cells, nonfinite/overflowed offsets, missing
  and root targets refuse without working changes. The 1024-cell upper-bound fixture completed
  successfully in both implementations (original plus 1023 copies), with real PNGs, full SVG
  byte comparison after bound IDs, uniqueness checks and history parity. Cap/target-error
  precedence is also compared independently.
  Incremental serialized clone/tail byte accounting bounds staged growth against the configured
  input limit; source DOM plus at most one pending copy can exceed that serialization budget
  transiently. Separate native STDIO with a 512-byte limit proves refusal preserves source/working
  SVGs and creates no snapshot (`results/tile-acceptance/native-size-guard.json`). This avoids
  growing an oversized grid until final transaction serialization. The final pipeline still checks
  exact serialized bytes before working mutation. Nontrivial grids use duplicate's conservative
  unsupported-reference/stylesheet guards; 1x1 does not copy and stays no-op. Duplicate's 19
  comparisons were rerun after extracting the shared insertion helper and still match. More
  scalar/namespace/entity/collision cases and failure injection remain pending.
- Filtered search comparisons: **45 observations, zero differences** in
  `find-comparison.json`. Native `find_objects` shares inspection's ObjectRef extraction,
  AND-filters tag/ID prefix/collapsed Unicode text and effective fill/stroke using the
  reference CSS subset and exact lowercase `inherit` token. Returned paint remains authored.
  Inclusive region intersections use DOM bounds or one actual Inkscape `--query-all` call;
  engine bounds retain the reference physical coordinate behavior. The guarded working SVG
  is staged privately with relative asset paths rebased. Missing/failed/timed-out engines
  fall back to DOM bounds. Synthetic CSV tests cover malformed rows, negative/zero extents,
  duplicate IDs and nonfinite values (JSON null), with exactly one child per accurate call.
  Originals/working bytes stay unchanged and no snapshots/audit records are created.
  A separate native capped-output fixture verifies bounded stdout also falls back to DOM;
  it is not a Python parity claim. The 71 read comparisons passed again after sharing
  ObjectRef extraction. One earlier repeat timed out during Rust MCP initialization with no
  stderr; its cause remains undetermined. Subsequent complete repeats passed. Additional
  scalar/schema errors, asset races and platform-specific cases remain pending.
- Grid composition comparisons: **102 observations, zero differences** in
  `grid-comparison.json`. Native `compose_grid` supports whole-document and same-source object
  modes, repeated assets, existing or newly created targets, row-major ordinary cell groups,
  gap/padding and optional uniform downscale. Eighteen scenarios compare complete wire results,
  position/suffix-validated minted IDs, normalized full SVG SHA-256/raw lengths, Operation
  Records/snapshot lists, real CLI preview RGBA hashes and restore replies. Registry count
  deltas verify that only successful new-target requests create a document; an invalid new
  plan creates none. Existing targets restore exactly; sources/originals remain unchanged.
  Three new units cover plan limits/origins/canvas, 28 exact unnormalized DOM geometry fixtures,
  and source-list/aggregate-source/invalid-plan refusal before any new document or history.
  The bbox heuristic intentionally matches the reference: transforms are ignored, primitive
  attributes are unioned, malformed/unit-bearing/negative extents are skipped, zero boxes and
  overflow are observable, and content is never upscaled. This is not accurate engine geometry.
  `contracts/grid-plan-cases.json` is explicitly captured from the preserved Python planner;
  the normal Rust fixture test does not invoke or refresh Python. Source list length is capped
  at 1024 before loading; whole-document input totals additionally obey the configured input
  budget (a stronger bound than the reference's separately loaded files). Object mode loads
  its source once. Imported source/target contexts reuse placement's conservative guards.
  Full schema/error coercions, expanded inheritance/asset relocation, arbitrary primitive
  numeric spellings, maximum-grid CLI acceptance, copy/filesystem failure injection and native
  GUI/package acceptance remain pending. New-target creation and the subsequent edit preserve
  the reference's separate-stage behavior; it is not a cross-stage crash-atomic transaction.
- Action/extension runtime discovery: **98 observations, zero differences** in
  `action-discovery-comparison.json`. Native list_actions/discover_extensions probe real fixed
  CLI version/action/help/data-dir commands, bounded read-only inkex sources, fontconfig and
  D-Bus diagnostics; extension sources are never imported or executed. Fixtures cover real/absent/
  failed/empty/timed-out/bad-version/non-executable/signaled engines, data/inkex declarations,
  font failures, owned/unowned bus and operator additions. Full replies, ordered notes/lists,
  compact/count semantics, scalar coercion, persisted map filename/content and unchanged original/
  working SVG/no history are compared. JSON text is decoded semantically, and only validated UTC
  map timestamps are bound. Native detection may truthfully enumerate non-allowlisted Actions;
  execution still uses both independent gates.
  Three separate native guards reject truncated Action stdout and file/directory symlinks in
  inkex source reads without modifying the document or protected helper sources. D-Bus first
  asks the broker for GetNameOwner, validates a unique name and sends Actions.List only to that
  unique owner: missing/racing well-known names cannot trigger activation. Controlled bus traces
  verify no Actions.List without an owner and no named-service target. This is mock IPC evidence,
  not native GUI acceptance. One unit covers version/export parsing and rejects named/malformed
  owner responses. Shared process discovery now checks execute permission, and outcomes retain
  actual signal/exit codes for truthful notes. Both 110 Action comparisons and 32 fault comparisons
  passed again. Helper runtime packaging, further malformed/symlink/race/output/failure
  details, warm shell and GUI/package acceptance remain pending.
- Private native D-Bus IPC kernel now uses fixed Action/Parameter/Target/Context enums,
  bounded gdbus argument-list subprocesses and a pinned unique owner. Connect first queries only
  the broker GetNameOwner, validates the shared unique-name grammar, then sends Actions.List to
  that owner; Describe/Activate/introspect and all context methods use the same pinned name.
  Missing/bad owner never calls the application. Window paths are constructed from an integer;
  managed identities are validated UUID strings before SelectDocument/context Activate.
  Parameters guard quote/backslash/control characters and nonfinite doubles; no arbitrary action,
  method, shell or extension execution route was added. Outbound argv and incoming stdout are
  bounded; bridge replies additionally cap at min(operator output, 1 MiB). Shared process outcomes
  now retain already-bounded stderr internally (64 KiB), solely to classify the fixed native
  ContextChanged error without exposing private details. Mutation transport/timeout/nonzero/
  oversized or malformed-response faults are uncertain; the owner binding is removed and no
  retry occurs. A native context-changed refusal stays actionable and retains the owner so
  documents can be listed/chosen again. These are intentional stronger safety semantics than
  the reference generic action failures/well-known destination; full live comparisons are pending.
  `contracts/bus-variant-cases.json` has **25 exact Python-reference GVariant fixtures**. Four new
  Rust tests use controlled fake gdbus argv traces to verify broker/unique destination, all fixed
  methods/context identities, guarded parameters, absent/malformed owner and genuine 1 s timeout,
  failure/oversized mutation replies with no subsequent dispatch. Total Rust tests: 110.
  An initial 200 ms test timeout failed during connect; its cause was not established. The fixture
  now uses the production 1 s floor and a 2 s stalled mutation; the complete repeat passes.
  After shared process stderr retention, all 98 Action-discovery observations passed again.
  Fake CLI traces are not evidence of an actual D-Bus service or native GUI. Public live dispatch,
  DBus export/view/style transport, managed context/locking/helper
  transaction, actual bus/component packaging and native Undo/Redo acceptance remain required.
- Private native context-reply parser now consumes only bounded literal data, without evaluation.
  **54 exact Python fixtures** compare active/list replies and errors: UUID validity, names,
  Unicode/hex/octal escapes, adjacent strings, annotation normalization, malformed/code-like
  input and raw NUL. Separate native tests cap bytes at 1 MiB, rows at 10,000, nesting/nodes and
  refuse unpaired surrogate/out-of-range Unicode escapes. The existing global annotation removal
  (including inside titles) is preserved explicitly for compatibility. One fake CLI integration
  reads and lists matching documents through the pinned unique owner. Three additional tests
  bring the total to 113. This specialized grammar does not claim exhaustive Python literal
  support; actual native bridge/GUI acceptance and public live integration remain pending.
- Private Dbus backend now implements the common Transport with stateful fixed export actions,
  no-follow bounded regular-file reads, safe XML active-document metadata/count, strict UTF-8 SVG,
  PNG transport bytes, integer window viewport actions and selection style/transform mutations.
  Export settings are an honest side effect; no document save substitutes for Undo. **22 Python
  backend cases** retain internal results/errors and exact action traces through synthetic process
  boundaries. Only minted export filenames bind to `<EXPORT>`. Three explicit differences retain
  legacy traces: pan now refuses before introspection, and two malformed transform tails refuse
  before earlier style/transform mutations. Whole-plan guarded variants and argv byte limits are
  checked before the first mutation. Missing/symlink/oversize/bad XML/UTF-8 export tests verify owned
  temp cleanup and original preservation. Numeric overflow/NaN and later unsafe/oversize style
  values send no application call. Three new tests bring the total to 116. These tests do not
  establish actual D-Bus, PNG pixels, native Undo or public-tool parity. Managed context binding,
  locks/helper transactions, real probes/public session wiring and packaged components remain.
- Private Managed now implements selected-document guards, scoped no-follow stdout/lock handles,
  deadline-bounded POSIX flock, list/select and managed plain-SVG/PNG reads. Same-thread nested
  clients for the same stream share the lock and captured context; different sessions refuse
  nesting. A 16-level bound and cross-thread active-scope refusal prevent stale context reuse.
  All guarded export actions use typed UUID Context.Activate through the pinned owner; there is
  no app-global fallback. **50 exact Python task-guard fixtures** cover unselected/current/changed
  document/window/title and legacy cases. Five additional tests verify selected/list/read binding,
  context-switch refusal, shared nesting, error/unwind cleanup, symlink stdout/lock refusal and a
  real separate child process holding the lock: timeout sends no context request. Disconnect
  clears selected identity. Managed metadata preserves its actual sodipodi namespace and counts
  only elements, distinct from legacy DBus's metadata/count. A guarded nested scope without a
  captured context now refuses before actions. Total Rust tests: 121. This private backend only
  advertises its completed read methods; selection stdout fencing, scene, native effect
  transactions, fresh liveness/probes, public wiring and actual GUI/Undo/package acceptance remain.
  All 71 headless read scenarios reran after sharing regular-file handle validation: zero differences.
- Private managed selection now appends fixed select-list/query-x actions under one captured
  scope and reads new stdout with positional reads of the pinned regular-file descriptor. Numeric
  fencing distinguishes complete empty selection from an incomplete reply; ordered dedup,
  Unicode IDs, exported-root filtering and Python splitlines/strip behavior are preserved.
  **58 exact Python parser fixtures** match internal results/errors. Native 1 MiB and 10,000-ID
  bounds refuse excess; incomplete/invalid UTF-8 streams time out without action retry. Fake CLI
  tests verify Unicode/empty/dedup/root filtering, malformed/incomplete/oversize failures and scope
  cleanup. Managed export now resets sticky export-id/id-only/text-to-path/plain options and keeps
  Inkscape metadata; the legacy Dbus plain-SVG behavior is unchanged. Root IDs refresh on SVG
  export, and region rendering maps user units through the shared coordinate kernel. A mapped
  synthetic SVG/PNG trace checks reset order, transformed export-area and one context capture.
  Five new tests bring the total to 126. These do not prove actual managed geometry/error parity,
  native pixels/Undo or public live-tool behavior; scene/inspection/effect/public/package work remains.
- Private live_scene now reuses headless ObjectInfo/tree helpers for scene and selection inspection.
  **140 exact Python scene/inspection fixtures** compare every field without normalization:
  inline/presentation visibility and descendant overrides, nonrendered ancestors, transforms and
  unavailable bboxes, duplicate IDs, selection order/duplicates/stale IDs, metadata, canvas/viewBox,
  hierarchy and explicit paint/unavailable viewport notes. Native XML/input and 10,000-element/
  selection guards are separate tests. Shared numeric text decoding accepts frozen Unicode 15
  decimal digits and Python underscore/exponent syntax; **705 Python numeric-text fixtures**
  compare finite values and nonfinite/invalid classifications. Frozen digit starts and both fixtures
  regenerate with the scene reference script; no runtime Python is used. Headless bbox prefix
  parsing shares the decoder, preserving the existing prefix grammar. Managed scene/inspection
  capture selection and SVG under one context, apply captured UUIDs only to the scene's document,
  and preserve DOM order for inspection. A fake CLI integration test checks command order, one
  context per call and scope cleanup. Four new tests bring the total to 130. Native GUI/pixels/Undo,
  full managed geometry/error parity, helper effects, fresh liveness/probes and public wiring remain.
  After the shared inspection/decimal refactor, all 71 read and 45 find comparisons passed again.
- Private native effect-data kernel now matches **11 exact helper document fingerprints** and
  **18 edit reply validation fixtures**. Hash input preserves expanded names/sorted attributes,
  root UI and metadata/namedview exclusions, exact mixed text/tail/comment/PI/entity behavior and
  Python JSON separators/UTF-8. Records stream into SHA-256 with an expanded-record budget capped
  at min(8×operator input limit, 64 MiB); per-record namespace expansion is bounded before any effect
  activation. This is intentional protection beyond the reference. Replies verify nonce, strict
  boolean discriminator and applied/refused field types; stale/malformed/oversize/unsafe-file faults
  remain uncertain. Captured public refusal reasons form a whitelist; unknown details use the fixed
  invalid-document/selection fallback. Private Exchange preflights fixed regular entries, atomically
  writes mode-0600 request bytes and reads replies through no-follow bounded descriptors. Cleanup
  is once-only, including best-effort Drop; a completed exchange cannot remove a later request.
  Tests preserve symlink originals and verify request/reply caps, nonce/type outcomes, data/mode and
  cleanup. Five new tests bring the total to 135. This kernel does not invoke an effect, rewrite a
  drawing, or establish native Undo. Managed activation and post-effect confirmation, insertion
  planning, actual helper/package/GUI validation and public live wiring remain required.
- Private Managed edits now connect Exchange to the fixed native edit-effect action, under one
  selected-task/operation scope covering selection, pre-export, request preparation, activation,
  reply and post-export SHA confirmation. Internal style/text methods and eight typed structural/
  order variants reuse the preserved helper protocol; no SVG disk save replaces an effect. Legacy
  fill-only fallback remains guarded when the helper is absent. **13 exact Python request/result/
  refusal transactions** match after validating/binding only the minted nonce. A separate fake
  effect mutates a synthetic SVG and computes SHA with Python helper code; Rust confirms the
  changed document. Lost/stale/missing/mismatched replies and a guarded context switch remain
  uncertain, without activation retry; files/scope are cleaned. A validated refusal after activation
  loss is recovered safely. Applied replies stay uncertain after transport loss because unique-owner
  binding is quarantined: an intentional conservative difference from reference reconciliation.
  Dynamic refusal text is emitted only from the fixed whitelist/fallback. Four new tests bring
  total to 139. Initial trace assertions counted readonly Describe as activation; one context-switch
  fixture used legacy unguarded mode. Corrected method filters and explicit guarded selection pass
  the full repeat. Synthetic effects do not establish actual native Undo/Redo, installed helper or
  GUI acceptance; public count stays 84/26.
- Private Managed insertion now validates the original fragment natively, then activates only the
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
- Private live_probe now derives readonly D-Bus/managed/socket readiness and constructs fixed
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
- Public native check_live_support/live_status/live_disconnect and the live/session resource now
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
- Public live_connect, live_get_active_document/live_get_selection/live_inspect_selection and
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
  After this integration, the full Python CLI suite was rerun: 1286 passed / 12 skipped.
  All 152 Rust tests and all-target clippy/format passed; Ruff checked 282 files and mypy 122.
  Existing connected/status/capability/action-discovery STDIO comparisons were rerun successfully
  (26 / 48 / 46 / 98 observations respectively); generated LLM manifests remain unchanged.
- Public live_wait_for_change and inkscape://live/events now share Session's last-token/change
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
- Public live_set_viewport, live_render_view and live_get_scene now use the fixed native
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
- Private common live Transport/Session/Cache kernels now share fixed semantic methods with
  the socket adapter. Transport exposes capabilities, identity guard/selected-document hooks
  and operation begin/end; unsupported defaults fail explicitly. Ranking is stable by available/
  rank and filters full read commands or no_freeze + active-document support, without assuming OS.
  Session performs teardown before backend choice, attaches through a fixed backend factory,
  captures the document/timestamp and creates a new frame cache. Failed connect/document read
  disconnects the attempted transport and leaves no old token/cache/document/binding. Disabled
  connect refuses before teardown/factory. Teardown resets change state and invokes a history-clear
  hook (actual persisted live-record integration is pending). RAII operation scope ends backend
  context on success, error and unwind; mock error/panic tests verify end calls.
  `contracts/session-state-cases.json` captures **872 exact Python status/recovery fixtures**
  and **12 deterministic cache transitions**, with no normalization/probes/GUI. Disabled/disconnected/
  connection_lost/document_unavailable/connected, guarded matching/mismatching/absent IDs, legacy
  managed recovery and installed-helper reconciliation all match. LRU count/byte floors, replacement,
  touch order, exact coalescing boundary and the reference single-oversized-entry exception match;
  u128 byte accounting avoids aggregate machine-size overflow. Managed status refreshes identity;
  socket status retains the connect-time document, as in Python. Six new Rust tests verify these
  reference cases, capability selection, failed attach/cleanup, operation error/panic and a real
  TCP Session→Socket→read→reconnect path resetting token/change state. Total Rust tests: 106.
  These are private kernels with temporary dead-code allowances, not public live acceptance:
  26 live dispatches/five resources remain pending. Actual probe/config integration, persisted
  record clearing, guarded managed operation context, DBus/managed backends, render cache keys/
  artifact existence/coalescing pipeline, bounded event wait and packaged/native Undo remain required.
- Private socket semantic render/view/edit methods now complete the fixed transport surface.
  render_view carries optional typed region/scale, export_selection decodes the same binary result;
  viewport arguments have a four-variant enum (zoom/pan/fit_selection/fit_page). apply_selection
  accepts a string-property map and optional composed transform, insert_svg/text carry their fixed
  semantic payloads and use the shared mutation model. These are internal transport methods:
  public validation, approvals, audit, before/after render, task binding and session gates must
  still be wired before exposing live tools. No new public tool is claimed. Nonfinite view
  arguments refuse before framing so serde_json cannot silently convert them to null.
  `contracts/socket-binary-cases.json` captures **29 exact Python strict base64 results/errors**,
  including wrong/missing types, malformed alphabet, UTF-8/control bytes, absent/excess padding
  and accepted nonzero unused padding bits. Native decoding preserves this reference behavior;
  decoding transport bytes alone does not prove PNG validity (visual validation is in the render
  pipeline). Error categories preserve capability-unsupported versus communication failure.
  A real loopback peer verifies all wrapper argv-equivalent semantic params, Unicode text,
  style/transform/SVG and modeled undo_friendly results; no Inkscape mutation occurs in that test.
  A second peer verifies lost insert reply remains uncertain/no retry and old stream is dropped.
  Three new tests pass (100 Rust tests total). Numeric public bounds, SVG/text/style validation,
  approval/live records, actual effect/native Undo and packaged acceptance remain required.
- Private live result/perception model kernel (public live dispatch still pending):
  `contracts/socket-model-cases.json` captures **658 exact Python-reference cases** for
  socket document/selection/inspection/mutation/viewport results, defensive scene modeling,
  Python string conversion and cheap revision/selection/viewport token digests. Malformed/default/
  Unicode/bool/non-numeric/scalar/list/object, arbitrary-size integer document counts, signed zero,
  scientific floats and rounding boundaries are compared without normalization; 256 reproducible
  float fixtures spanning exponents -300..300 exercise both string representation and token hashes.
  Scene selection/objects and token selection each keep the reference first-10,000 input-item cap
  before filtering. Reordered selection remains distinct; initial/same/change classification matches.
  Scene payload cannot supply document identity: the socket semantic scene method performs the
  separate authoritative active-document read before modeling. Native typed command wrappers
  now serve active document, selection, inspection, SVG, scene and state token internally;
  handshake capability coercion shares the Python-compatible string model. Real TCP tests verify
  fixed semantic command order and reject a spoofed scene path. Three new units cover all compiled
  reference models, 10,001-item clipping/count/digest equivalence and authoritative document
  readback over loopback. The total is 97 Rust tests, with public dispatch still 84/26 and five
  live resources pending. These private modules retain limited temporary dead-code allowance
  until session/backends/tool integration. Public render/view/edit validation and pipeline, bounded event wait/session cache/audit, DBus/managed identity guards and supervisor/packaged
  helper/native GUI Undo acceptance remain required. No user GUI/document/process was touched.
- Private native live wire/socket kernel (not yet public MCP dispatch): fixed protocol v5
  and all 14 enum commands are compiled into Rust. `contracts/socket-protocol-cases.json`
  captures **96 unnormalized Python-reference cases**: 42 exact request UTF-8 frames,
  15 response success/error discriminators and 39 rendezvous/version/Pydantic scalar cases.
  Native tests consume this fixture directly, including Unicode/control text and literal shell
  characters, without executing any input. The private client always dials 127.0.0.1, carries the
  helper token on every request, validates the hello response and caps incoming messages at
  min(operator max_output, 64 MiB). Real loopback peers exercise handshake plus every remaining
  fixed command, capabilities, repeated disconnect and refusal after disconnect. Other peers
  cover rejected/invalid-version hello, malformed JSON, truncated EOF, oversized frames and
  mutation replies lost after dispatch. There is no retry; untrusted/lost mutation completion
  is typed uncertain, its socket is quarantined and the public recovery message requires
  inspecting the drawing. Explicit helper rejection remains a rejection, not uncertain.
  Bounded outbound serialization refuses during writing, avoiding an unbounded encoded
  buffer; exact UTF-8/escape/newline boundary and one-byte-under caps are tested.
  A slow-trickle test verifies a whole-request deadline and distinguishes view-only faults from
  mutations. Seven new Rust tests pass. This is native TCP kernel evidence, **not MCP live
  acceptance, helper integration or native GUI Undo evidence**; all 26 live tools and five live
  resources remain pending. Module-level temporary dead-code allowance is limited to these
  private kernels until session/transport/tool wiring; no migration-pending tool was relabeled.
  Intentional stronger guards: anchored no-follow rendezvous file/ancestor reads; rendezvous
  limit min(operator input, 1 MiB), token ≤4096 bytes; unsolicited data after a reply frame
  refuses, hard request deadline prevents indefinite trickle, faults drop the stream and uncertain
  mutations are never silently retried. Link/cap/token and unchanged protected-file tests pass.
  Windows filesystem implementation, remaining defensive/rendezvous integer-range cases, all
  backends, session state/reconnect, supervisor/bridge packaging, live audit/cache/perception,
  packaged/native Undo acceptance remain required.
- Runtime capability cache/registry: **46 observations**, with **28 explicit runtime identity
  differences and zero unexpected differences**, in `capability-comparison.json`. Native
  list_capabilities/diagnose_runtime and `inkscape://runtime/capabilities` share a per-server
  probe cache; cached tool/resource reads do not spawn new probes, diagnose refreshes both.
  Six fixed probes run once per cache fill in controlled CLI fixtures. Changing the fixture
  version proves cached reads remain stable and diagnosis replaces the cache with fresh fields.
  All sixteen live/raw/core/full/description profiles compare the sorted actual registry count,
  first-line purpose, risk and 49 intents; hidden core tools remain hidden while its resource
  works. Real Inkscape and absent-backend cases also pass. Full raw wire traces are retained;
  tool text/structured fields and resource JSON must agree. Only validated UTC probe timestamps
  are normalized. The legacy string field `python_version` truthfully contains
  `not applicable (native Rust MCP)` in Rust; the Python reference reports its interpreter.
  This unavoidable architecture difference is retained in all 28 full matrices, so the report
  explicitly records `exact_parity: false`; it does not pretend Python runs the native server.
  The frozen schema field/type/description are retained for compatibility; private helper runtime
  reporting and packaging remain pending. One unit proves per-read registry overlay refresh,
  deterministic sort/purpose/risk and no mutation of the underlying cached probe. No GUI launch,
  document, snapshot or Operation Record is caused by these probes.
- Controlled Action validation/runners: **110 observations, zero differences** in
  `action-comparison.json`. Native validate_action_chain/run_action_chain/run_raw_action use
  operator allowlist AND detected/cached version map AND fixed action/argument grammar. Chains
  remain bounded at 32 steps × 16 args; approval/doc/error precedence, normalized plans/argv,
  raw default dry-run and ordered real CLI edits match. Complete replies, SVG hashes, records,
  snapshots, actual preview PNGs and exact restore are compared. Persisted map filename,
  raw version/tuple, ordered actions/count/source also match; only a validated UTC probe timestamp
  is normalized. Cache bytes remain unchanged across repeated calls. Two separate native link
  fixtures prove file/directory symlinks cannot redirect map reads or persistence outside the root.
  **32 additional raw-run engine-fault/no-op observations** have only six explicit audit safety
  differences (legacy engine faults remain proposed; native transitions to discarded); every
  difference is retained. Absent engine refuses at the action-availability gate before audit.
  Three native structural output guards are exercised separately without SVG/snapshot changes.
  Two units cover gate/error ordering, grammar, hints/bounds, map parsing/order and safe version keys.
  Paths share a new bounded private CLI runner; all 170 path observations and 32 path-fault
  observations passed again after the refactor. Native map acquisition currently probes only
  version and action-list (no Python runtime involved); warm shell, complete Pydantic errors/coercions, additional map corruption/race/stale/security
  cases and GUI/package acceptance remain pending. No new extension execution was introduced.
- Typed path geometry: **170 observations, zero differences** in `path-comparison.json`.
  All seven tools use fixed Actions, deduplicated argv-safe IDs, default dry-run and explicit
  high-risk approval, private no-follow staging, bounded engine output and the shared transaction.
  Complete replies, whole SVG hashes, audit/snapshots, actual PNG previews and exact restore
  match for simplify/cleanup, union/difference/combine, break-apart and stroke outlining;
  document-level DTD/comment/PI and referenced gradient paint fixtures also match.
  Merge IDs standardize to document-order bottom; outline fill restoration and new empty-marker
  cleanup match. The harness explicitly enables the advanced gate and requires expected success
  before comparing; an initial gated-out harness run was invalid and is not evidence.
  **32 additional engine-fault/no-op observations** match public replies/history fields except
  **seven explicit audit safety differences**: Python leaves engine-fault records `proposed`,
  Rust marks them `discarded`. No semantic field is normalized to hide this; the fault report
  retains every difference and requires zero unexpected ones. Missing engine, nonzero exit,
  genuine 1-second timeout, missing/empty/unsafe/oversized output and no-op are exercised.
  Three separately recorded stronger output guards refuse non-SVG roots, duplicate IDs and
  references to removed original IDs without SVG/snapshot mutation. Four units cover ID grammar/
  merge order, stroke fill/marker retention, original comments/PIs with incoming DTD and unsafe
  output structure. Broader references/CSS/asset context, schema errors, geometry edges, warm
  shell and GUI/package acceptance remain pending. The input target list adds a 4096-item cap.
  The shared timeout now follows Python's 1-second minimum; sub-second config falls back to 60s.
  A unit covers the floor plus finite/86400s bounds (nonfinite values remain a stricter refusal).
  Find's fixture now uses a genuine 1-second timeout, and all 45 observations passed again.
- Selector-driven transforms: **116 observations, zero differences** in
  `transform-comparison.json`. All eight targeted ops reuse native `find_objects` and
  `apply_edits` kernels with one named transaction/record. Fixtures cover CSS paint selection,
  accurate CLI geometry, dry-run plans and default behavior, empty matches, no-op, delete
  approval, match caps, invalid colors/pivots and document-root deletion refusal. Complete
  responses, whole working SVG hashes, snapshots/audit, actual PNG previews and exact restore
  match. A native unit verifies the 65-match dry-run plan, real-run 64-edit fan-out refusal and
  excluded creation op without byte/history changes. The shared batch refactor also passed all
  122 edit scenarios again; frozen discovery passed 16/16 exact comparisons. Full Pydantic validation errors/coercions,
  additional failure injection and GUI/package acceptance remain pending.
- Cross-document placement comparisons: **69 observations, zero differences** in
  `place-comparison.json`. Native `place_document` copies whole/rootless SVG or a selected
  self-contained group, rewrites contained references through the common duplicate remint
  kernel, creates one ordinary named wrapper with translate/scale and preserves source files
  and working copies. Thirteen cases compare whole replies, topology-validated minted clone/wrapper
  IDs, normalized full SVG SHA-256 and raw byte lengths, snapshot/Operation Records, real CLI
  preview RGBA hashes and exact target restore, including down/upscale, finite/positive scale
  validation and missing source/target/object errors. Normalization binds only minted IDs
  verified against each original subtree position and six-hex suffix, corresponding wrapper
  IDs and document/operation/snapshot IDs/timestamps; semantic fields and pixels are retained.
  `results/place-acceptance/native-context-guards.json` separately verifies six stronger
  preflight refusals for source defs outside the selected subtree, inherited container paint,
  viewport-relative geometry, external assets, scripts and target inherited root paint.
  Neither document's source/working bytes or history changes on these refusals. These guards
  protect appearance/reference ambiguity which legacy deep copy does not resolve; expanded
  context preparation, asset relocation and full scalar/schema error parity remain pending.
  Two units cover self-contained dependencies/neutral ancestry and rootless ID/remapped refs.
  `compose_grid` now reuses the shared append/remint kernel; expanded import contexts remain pending.
  A 1024-copy tile regression encountered a random six-hex root suffix collision (a legacy
  failure mode); automatic root allocation now retries at most 128 times like descendant
  allocation. Explicit supplied-ID conflicts still fail immediately. A deterministic unit
  verifies successful retry, explicit conflict and 128-attempt exhaustion. This is an intentional
  robustness difference from the reference's immediate random-root collision refusal, not an
  ignored parity delta. Native GUI, packaging and multi-platform checks remain incomplete.
- Fragment replacement comparisons: **131 observations, zero differences** in
  `fragment-comparison.json`. Native `replace_svg_fragment` shares adoption's strict allowlist
  and approval/input bounds, validates the selected container's qualified tag and stable ID,
  rejects duplicate/invalid/conflicting IDs and unresolved/removed references, and preserves
  insertion slot and mixed tail. Twenty-five cases compare whole replies, SVG SHA-256,
  snapshots/Operation Records, real CLI preview RGBA hashes and exact restoration. Exclusive
  C14N with comments matches the reference's no-op detection (including reordered attributes
  and externally referenced identical content); changed retained references require explicit
  `allow_retained`. Default-policy refusal, allowed appearance change, removed target refusal,
  inside/outside refs, namespace mismatch, stylesheet preparation, missing/root targets and
  invalid/disallowed inputs match. Only validated minted IDs/timestamps and corresponding
  artifact paths plus JSON tool text are normalized. Three units cover input/approval order,
  byte-preserving canonical no-op and replacement tail/reference-policy behavior.
  `results/fragment-acceptance/native-reference-guards.json` separately proves two stronger
  guards for removed timing/accessibility targets: unchanged source/working bytes, no snapshot
  and one discarded audit record. The reference helper does not track these reference kinds.
  Full schema/Pydantic error formatting, further canonicalization namespace/encoding/entity
  cases and filesystem/copy failure injection remain pending. This is automated CLI parity,
  not native GUI or packaged runtime acceptance.
- SVG adoption comparisons: **152 observations, zero differences** in `adopt-comparison.json`.
  Native `set_document_svg` and `insert_svg_fragment` preserve high-risk approval, bounded
  UTF-8 input, the reference element/attribute/namespace allowlist and href/url guards, then
  reuse the shared edit pipeline and post-adopt validation. Twenty-nine cases cover qualified/
  unqualified whole SVG, document replacement no-op, DTD and top-level comments/PIs,
  intact/nested/unwrapped fragments, parent selection, mixed wrapper tails, plain/xlink local
  refs, empty containers and malformed/disallowed/active/external/unknown-attribute content.
  Complete replies, full working SVG SHA-256, snapshots, Operation Records, real CLI preview
  RGBA hashes and byte-exact snapshot restore match. Only validated minted IDs, timestamps,
  their artifact paths and JSON tool-text decoding are normalized. Source files never change.
  `results/adopt-acceptance/native-id-guards.json` separately verifies two stronger insertion
  refusals for existing/document-local duplicate IDs; working/original SVGs stay unchanged,
  no snapshot is created, and one discarded record audits each refusal. Whole-document
  replacement intentionally follows the reference's post-adopt validation rather than imposing
  insertion collision checks against the replaced scene. Three native units cover approval/
  input caps before parsing/registry access, allowlist/entity refusal and independent namespace/
  mixed-text copying. A shared serializer unit covers top-level comment/PI/DTD spacing matching
  lxml without stripping semantic bytes. Read's 71 comparisons were rerun successfully after
  this serializer change. Full scalar/schema errors, CSS escape/URI edge cases, additional
  namespace/encoding/document-sibling combinations and failure injection remain pending;
  additional composition edges and native GUI/package acceptance remain
  unported/unverified.
- Optimizer comparisons: **161 observations, zero differences** in `optimize-comparison.json`.
  Native `svg_web_optimize` and `optimize_set` compose the existing staged edit pipeline;
  each changed document has an auditable pre-snapshot and before/after previews, while a
  byte no-op retracts the operation and creates no snapshot. Seven fixtures cover metadata,
  comments/PIs, live/dead defs and IDs, nested empty groups, retained IDs, precision 0/2/8,
  unused namespace declarations and mixed text. Independent pre-mutation opportunity counts,
  order-dependent summary counts, serialized sizes, full working SVG SHA-256, snapshot lists,
  Operation Records and real CLI-exported PNG RGBA hashes match the Python reference.
  Restore recovers original bytes exactly before set tests; set aggregates and pre-edit
  consistency verdicts match, including reverse order, empty/duplicate/unknown IDs and invalid
  precision. Normalization binds only validated minted document/snapshot/operation IDs and
  timestamps, artifact paths/URIs containing those IDs, and decodes JSON tool text.
  `results/optimize-acceptance/native-reference-guards.json` separately proves five intentional
  stronger refusals: CSS, scripts, SMIL, accessibility references and an existing referenced
  metadata target. Original/working SVGs stay byte-identical, no snapshot is created, and
  one discarded Operation Record records the refusal. The reference cannot safely retain
  all such dependencies. Two units verify used namespace preservation, cleanup idempotence,
  early selector refusal and retained target validation. Set behavior preserves the reference's
  sequential semantics: earlier successful documents remain changed/reversible if a later
  document fails; it is not an atomic whole-set edit. Full schema/error coercions, Unicode
  decimal tokens/rounding extremes, namespace races and additional failure injection remain
  pending. These are automated/CLI checks, not native GUI or installable-package acceptance.
- Quality comparisons: **126 observations, zero differences** in `quality-comparison.json`.
  Native `quality_report` and `quality_report_set` reuse validation, font inspection and
  collection verdicts, with shared read-only optimization signals and optional structural
  advice. Thirteen SVG fixtures run with present and absent fontconfig: metadata/comments,
  used/unused definitions and IDs, coordinate precision, embedded/external rasters, missing
  fonts, duplicate/missing references, missing/zero/negative/invalid viewBox and 220 groups.
  Defaults, disabled advice, designated semantic layer thresholds and disabled label checks
  preserve every metric/score/advice field, including the 200-item advice cap. Ordered,
  reversed, single, empty, duplicate, unknown and malformed-document sets match. Comparison
  normalizes only validated minted document IDs (including verdict buckets) and decodes
  JSON tool text; it does not exclude report fields. Source/working bytes are unchanged and
  no snapshots or Operation Records are written. Three units independently verify bounded
  advice, comments/depth/disabled counts, read-only opportunity analysis and single/set output
  caps. The configured output cap is a stronger guard than the reference. Full option/schema
  error formatting, unusual numeric tokens/rounding, concurrent filesystem changes and native
  GUI acceptance remain pending; additional optimizer edge cases remain pending.
- Intent guidance comparisons: **820 observations, zero differences** in
  `intent-comparison.json`, without normalization. Native `how_do_i` and
  `inkscape://runtime/intents` share one compiled table of 49 entries and four out-of-scope
  rules, captured from the preserved Python source with a SHA-256 provenance record.
  Every map goal/keyword and uppercase out-of-scope keyword, mixed/tied goals, empty/unknown/
  Unicode/NUL goals, first-rule precedence and top-three stable ordering match full wire
  results. Full/core and live on/off configurations match, including core's unknown-tool
  gate and exact resource text/MIME (not merely decoded JSON). No Python/runtime executable
  is invoked for guidance; no workspace state is created. A unit test verifies every suggested
  tool exists in the frozen maximum discovery contract; suggestions can still name pending
  Rust capabilities, as the contract migration remains incomplete. Input/output size guards
  are separately unit-tested and are stronger bounds than the reference's uncapped guidance.
  Wrong scalar/missing arguments and further schema/environment cases remain pending.
  Capture is explicit via `migration_intent_acceptance.py --capture-data`; normal acceptance
  checks provenance and never silently refreshes the compiled table. Future guidance changes
  must update this captured data alongside the shared Python policy/templates.
- Artifact stat comparisons: **19 scenarios, zero differences** in `stat-comparison.json`,
  without normalization. Native `stat_artifact`/`stat_artifacts` resolve contained relative/
  absolute paths across roots, return relative paths, preserve list order/repeated entries
  and hash through one pinned no-follow regular-file descriptor using a 1 MiB buffer.
  Empty, binary 2.56 MB, Unicode paths, internal symlinks, lexical parents, missing/broken/
  escaping paths, NUL, oversize files, nonempty/empty sets and late-set failures match complete
  Python wire results. All fixture bytes stay unchanged and no document/history is created.
  Size is checked before hashing and during reads to bound a concurrently growing file;
  changed length/mtime is refused after hashing. Additional concurrent-write race injection
  remains pending, so this is not a complete immutable-file guarantee or measured peak-memory
  claim. Native fixtures separately verify a host-path-free regular-file refusal for directories,
  1024-path cap (a stronger bound than the reference's uncapped list).
  A separate native STDIO fixture verifies an 80-byte metadata output cap with unchanged
  file bytes (`results/stat-acceptance/native-output-cap.json`). A unit test proves
  direct final-symlink refusal, canonical inside-link stat, known SHA-256 and input limits.
  More scalar/schema and platform/race cases remain pending.
- Explicit retention comparisons: **12 observations, zero differences** in
  `retention-comparison.json`. Native `prune_snapshots` retains the union of last-N and
  keep-days snapshots, bounded by absolute count/declared-byte caps, prunes linked orphan
  records and folds root-scoped live frames into the same pass. Live frame age/newest-first
  byte budgets exclude protected before/after/diff frames. Empty history, zero/disabled caps,
  repeat idempotence, unknown documents, snapshot byte hashes, record survival and frame
  counters match Python. Working/baseline/source bytes are unchanged. Enumeration is bounded
  to 10,000 entries and records to 4 MiB (also constrained by workspace output cap).
  Native path guards separately prove tampered manifest basenames and snapshot symlinks
  remain indexed, without touching outside bytes or the working SVG; links are refused by
  pinned-directory no-follow metadata/unlink, a stronger policy than legacy link stat/unlink.
  A unit test covers invalid clock retention, naive UTC timestamps and hard-cap selection.
  Cleanup remains explicit; boot-time sweep of prior on-disk registries is still pending.
  Further malformed-model/environment variants, race/failure injection, Windows ports and
  crash-consistent index/file pruning remain pending. Like the reference, file deletion
  precedes index rewrite; this maintenance pass does not claim transactional rollback.
- Repeat comparisons: **19 scenarios, zero differences** in `repeat-comparison.json`,
  plus **30 direct planner plans/errors** in `repeat-plan-cases.json`. Native `repeat_objects`
  places root-space linked/copy instances in named ordinary groups with affine compensation,
  anchors, fixed/tangent orientation and Python-compatible integer-seeded MT19937 jitter.
  Original geometry stays intact; copies share duplicate's ID/reference/tail kernel and
  cached occupied IDs include the newly created group. Source/group IDs, unresolved/duplicate
  references, stylesheets, unsupported animation/viewport contexts, singular transforms and
  projected/final document size are guarded. Dry-run builds the full disposable tree without
  writes, snapshots or records. Apply uses the shared transaction pipeline; late apply refusal
  retains a discarded Operation Record, matching Python. Complete wire responses, SVG byte
  hashes (only strictly validated minted ID suffixes bound), unique IDs, snapshot bytes,
  audit records and real RGBA preview pixels match for transformed groups, linked references,
  internal gradients, mixed text/tails, anchor/orientation, Unicode labels and wide-seed jitter.
  String `mode` literal error matches the captured Pydantic 2.13 wrapper; complete schema
  error combinations and precedence remain pending. Unit tests exercise the 1024-instance
  maximum, projected size refusal before DOM mutation, locked ancestors and the intentionally
  stronger duplicate accessibility/timing reference guard. The 1024-instance check is a DOM
  unit test, not a claimed CLI pixel or GUI acceptance. Further alias/namespace/scalar edges,
  reference error coverage and failure injection remain pending. The integer seed parser is
  bounded at 4096 decimal digits. JSON parsing uses `float_roundtrip` and `arbitrary_precision`
  on pinned serde_json; 71 read and 122 edit comparisons passed after that configuration.
- Reload comparisons: **11 scenarios, zero differences** in `reload-comparison.json`.
  Native `reload_document` keeps the same identity, snapshots working bytes before source
  resolution and refreshes both baseline and working copy from bounded, workspace-guarded
  source reads. Even unchanged reloads create a checkpoint, matching Python. Source updates,
  missing/directory fallback, inside/escape symlinks, oversize refusal, created seeds, the
  legacy `document.svg` sentinel, unknown IDs and restoring the pre-reload snapshot match.
  Snapshot metadata and full SVG byte hashes are compared; no audit record is added by reload.
  Malformed replacement XML is intentionally refused before either managed copy changes;
  Python's legacy implementation copies it before summary failure. A native fixture proves
  this guard retains edited bytes and the checkpoint. Read-only working-directory injection
  proves failure of the second replacement restores the baseline, preserves working/source
  bytes and retains the checkpoint (`results/reload-acceptance/native-write-failure.json`).
  Each replacement is atomic; the two-file sequence is rolled back on observed errors,
  but crash-consistent two-file journaling and further race/fsync injection remain pending.
  The actual legacy missing-source fallback differs from its tool description; it is retained
  and explicitly tested rather than silently changed.
- Native save/create/validation comparisons: **50 scenarios, zero differences**. These include
  managed-file refusal, internal/external links, escape without mkdir, explicit second-root save,
  overwrite approvals, scalar coercion, original preservation, created SVG byte hashes, duplicate/
  dangling IDs, viewBox errors, observable DOCTYPE/XXE, actual Arial/Japanese missing-glyph
  warnings/suggestions and embedded raster threshold with excess base64 padding.
- Public render/export comparisons: **74 scenarios, zero differences** with real Inkscape CLI.
  Inline PNGs and PNG artifact readback compare decoded RGBA hashes; plain SVG artifact bytes
  compare SHA-256. Relative raster references in managed working directories are included,
  together with object previews, output-directory naming, future working-copy mtime/staleness,
  limit/error refusals and oversized-output cleanup. Fixed document-space region PNGs cover
  transparent/white backgrounds, out-of-page bounds, physical units, letterboxing, slice,
  nonuniform scaling and missing viewBox; unsupported root transforms, CSS sizing, percentage
  dimensions and invalid aspect alignment fail exactly as the reference. Frame coverage checks
  empty/unused listings, series/label sanitization, sequential captures, pre-existing frame
  indices, planted symlink exclusion and zero/negative/oversize widths. PNG hashes are compared
  for every captured frame. Snapshot comparison checks inline and URI-readback before/after
  PNGs, missing snapshots, canvas mismatch refusal without artifacts, and byte preservation of
  working/snapshot sources. A separate native test rejects traversal in a tampered snapshot index
  before invoking Inkscape. Numbering reads existing files; registry reattachment after a whole
  server restart is not claimed (the reference registry also starts empty). PDF coverage verifies magic and public
  vector/font flags; it does **not** establish PDF pixel or full byte equivalence.
  Both public and operation-linked previews stage render-only absolute hrefs to retain image
  resolution; working/source/snapshot SVG bytes remain unchanged. Explicit xml:base, percent-
  encoded references and unusual SVG links still need dedicated pixel fixtures.
  Rust also rejects actual PNG dimensions above the cap after CLI completion, even if an
  intrinsic-size estimate understated them; this is intentional extra protection. Same-second
  export filename collisions currently refuse replacement rather than clobber an older artifact.
- Export batch comparisons: **21 scenarios, zero differences**. Dry-run plans, 32-item cap,
  width/object/format validation, budget clamping and preflight refusal match. A real batch
  verifies exact summed output bytes, PNG RGBA hashes and SVG byte hashes through artifact URIs.
  Explicit out_dir dry-runs can create an empty directory, matching the reference; no export
  artifacts are produced. Typed scalar extraction runs before directory creation. Projection
  overflow is refused in Rust rather than wrapping. Duplicate same-second output names and
  actual budget overrun/partial artifacts still need additional dedicated acceptance fixtures.
- Profile comparisons: **28 scenarios, zero differences**. Web widths/scales precedence,
  sorting/deduplication, icon ordering/empty sets, size errors, escape refusal and source
  preservation match. Actual PNG RGBA and SVG byte hashes match through artifact readback.
  Print uses fixed PDF 1.4/text-to-path flags and verifies PDF header, font outlining and
  applied settings; PDF pixel/full-byte equivalence remains unverified. Native profiles cap
  width/scale/size input lists at 32 and refuse scale multiplication overflow. Duplicate icon
  sizes/same-second collisions, default ladders and more pathological lists need further fixtures.
- Export-set comparisons: **17 scenarios, zero differences**. Shared native consistency
  verdict covers viewBox dimensions/fallback, stroke-width attribute plus inline-style counts,
  naming styles, unknown properties, first-seen ties and input ordering. Dry-run aggregates and
  real two-document PNG/SVG artifacts, exact total bytes and source preservation match. Duplicate,
  empty, unknown and malformed members fail before exports. Native collection inputs cap at 32
  documents; overflow in aggregate counts/bytes is refused. Later per-document spec or engine
  failure can leave earlier artifacts, as in the reference; no export-set rollback is claimed.
- Prompt comparisons: **100 scenarios, zero differences**. Prompt-get responses are compared
  exactly; only JSON text in the four prompt-index resource cases is decoded before comparison.
- Intentional save hardening: Rust atomically replaces an approved existing destination rather
  than truncating its inode. A separate Rust regression proves that overwriting a hard link to
  the source cannot change that source. This strengthens the original-file invariant; the Python
  reference's truncate path does not provide this hard-link guarantee.
- Directory enumeration for frames is descriptor-pinned, ignores symlink entries and refuses
  symlink parents. It caps visits at 10,000 entries (plus dot entries), distinguishes EOF from
  read faults and uses exclusive output adoption. Extremely large numeric frame indices are
  rejected rather than truncated; these bounded guards strengthen the Python reference path.
- Native packaged insertion/Undo/Redo/reconnect acceptance: **passed** as scoped above. Other native cases and new CI jobs/results: **not run**.

## Next work

All 110 tool names dispatch natively; complete argument/default/coercion/error compatibility
still needs verification. Continue isolated native effect/document-switch acceptance, live
mutation benchmarks beyond the completed repeated read measurements, and helper-to-Rust
implementation after the separate offline investigation. Prepare installable
other-platform packages and Windows filesystem protection, then execute the prepared CI jobs
when an appropriate environment is available. Signing/ABI/license audit and broader crash,
filesystem race and XML fault coverage remain. Preserve the uncertain stage12 and owned stage13
sessions until native window targeting is unambiguous.

The goal remains active; this checkpoint is not completion.
