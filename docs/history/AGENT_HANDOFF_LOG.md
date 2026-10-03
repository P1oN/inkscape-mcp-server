Development STDIO frame checkpoint (2026-10-03): pinned rmcp AsyncRwTransport uses
unbounded read_until before argument validation. Current Rust wraps stdin with a per-line
byte cap before SDK buffering/deserialization; default6*MAX_INPUT_BYTES+1MiB accommodates
JSON escaping, override positive INKSCAPE_MCP_MAX_REQUEST_BYTES. Newline/CR count toward
cap. Oversized input closes only the MCP connection; startup/reconnect do not launch GUI.
Actual stage36 unfinished5KB line waits without EOF under configured4096 cap; current debug
refuses immediately, ordinary3ping frames pass, workspace bytes/mtime unchanged. Rust215/
1ignored, fmt/clippy and tooling lint/format pass. Not a whole-process RSS/output cap proof.
Fix not yet packaged; current36 unchanged. CI frame gate prepared, not remotely executed.

Native36 launch attempted on owned synthetic root/private profile and failed before drawing:
bridge reports no primary monitor. Native safety guard remains intact. Logs preserved in
native-stage36/saved-launch-diagnostics; native-stage36-launch-comparison.json is a failure,
not acceptance. No surviving owned GUI/supervisor found. A CUA display-name lookup also
launched ordinary Inkscape PID8571; acknowledged to user and left untouched. No UI edits or
window closures. User input requested for an unlocked Mac/active screen; meanwhile continue
independent work. This is one verified native environmental refusal, not recurrence of
historical disappearing-group incident. Do not retry native launch until screen is available.

Next: package current frame fix with prepared Homebrew BSD text/source kit, final scoped
output/requirement/documentation audit, then native acceptance when monitor is available.
Goal active; Windows/clean-machine/old incident remain user-deferred.

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

CPython provenance checkpoint (development, 2026-10-03): actual stage35 runtime matches
Astral CPython3.12.14+20260929 macOS arm64 stripped release. Both downloaded release
archives match GitHub asset SHA-256.946 regular runtime members outside site-packages
match byte-for-byte; libpython matches exact install_name_tool reproduction with the
original basename; all sysconfig values match published uv transformations; the added
EXTERNALLY-MANAGED marker matches explicitly. Additional executable sysconfig statements
are refused. Earlier codesign simulation used a different basename and yielded a different
signature; that was an audit-fixture issue, not a runtime defect. Raw failures are retained.
Pinned build recipe b498734a5791d0e6786695a226fd398a41c6f7f6 records source URL/hash/version;
actual packaged OpenSSL3.5.9/SQLite3.53.1/Expat2.8.5/mpdecimal4.0.0 agree with the recipe.
Report: migration/cpython-stage35-provenance-comparison.json.

19 full-distribution license texts plus PYTHON.json/provenance are now checked in under
migration/vendor-notices/cpython. Collector selects them only for the exact reviewed
executable hash; unknown targets/binaries remain explicit gaps. Collection tests and
Python lint/format pass. Current archive remains35 and DOES NOT yet include these texts.
This is regular-file runtime attribution, not wheel provenance, complete source rebuild
or legal clearance; conservative licenses do not imply every named library is linked.
Next: exact Homebrew GLib patch/source metadata, wheel attribution and fresh packaged
notices, scoped remaining audit/current native assessment. Goal remains incomplete.

Current package checkpoint: stage35 (2026-10-03). Includes bounded validation diagnostics,
compare_region private rendering before pair publication, and exact upstream notice
supplements for GLib2.90.0/D-Bus1.16.2/gettext1.0/PCRE2 10.48, matched to installed SBOM
URL/hash/version. Full GLib LGPL, referenced D-Bus alternatives and runtime libintl LGPL
are packaged; receipt/formula metadata is bound by FILES. Rust213/1ignored, fmt/clippy,
15notice checks/102crates, doctor10/launcher11/security35/files12/discovery16,
asset17/routes8 both engines, debug/package diagnostics3 and compare refusal/pixel readback
pass. Actual cold/warm archive installs with empty PATH pass. Archive/current index:
migration/package-stage35-build-comparison.json. No current GUI/performance acceptance.

Stage34 was an intermediate archive missing gettext-runtime/intl/COPYING.LIB; the
strengthened completeness check failed. It was never current. Stage35 includes the
runtime-specific LGPL text and passes. Exact archives/hash-verified notice extraction
are preserved in migration/results/native-source-audit. This closes missing native
license-text evidence, not full corresponding-source/relink clearance: Homebrew GLib
patches and private CPython native/source provenance remain to audit. Foreign target
attribution and actual execution remain unverified. No publication or user configuration
changes. CI notice/diagnostic/compare gates are prepared, not remotely executed.

Next: audit private CPython and exact modified native source/build metadata, finish scoped
output/publication review and assess current-package synthetic native smoke, then audit
the full objective. Earlier checkpoint notes below retain their original package scope.

Development comparison-publication checkpoint (2026-10-03): stage33 reproduction
confirmed compare_region left a before PNG when the after source was unsafe. Both PNGs
are now rendered privately before publication. Per-file exclusive atomic creation avoids
partial PNG exposure; second-write failure rolls back a preceding unchanged artifact,
preserves existing destinations and reports recovery when inspection/cleanup is uncertain.
Actual debug STDIO refusal leaves complete workspace bytes/mtime unchanged; a successful
blue/red pair is read back through resources/read. Collision/unusable-parent regressions
pass. Rust213/1ignored, fmt/clippy and Python tooling lint/format pass. This is not pair-level
crash atomicity or exhaustive concurrent-race proof; directory mtimes may change on a
publication failure. Current archive remains stage33 and does not include these development
fixes or bounded diagnostics yet. Raw evidence: migration/results/compare-stage33-before-fix,
compare-publication-final and compare-publication-review. CI gate prepared, not remotely run.
Next: remaining output/publication scope, dependency provenance, fresh package and scoped
native assessment; user-deferred Windows/clean-machine/live-incident work stays deferred.

Development diagnostic checkpoint: streaming bounded repr preserves ordinary frozen
validation errors, long keys (>256chars)/tags (>50chars) abbreviated. Stage33 reproduces
250527-byte unknown-field response; current debug3large-input refusals stay below4096wire
bytes with unchanged workspace. Rust211/1ignored and clippy pass. Current package remains33;
fix not packaged yet. Not a global transport/output/RSS proof. Next: remaining kernel/resource
errors and publication audit, provenance, fresh package/native assessment. See security audit.

# Current candidate: stage33 (2026-10-03)

Private bounded staging protects linked SVG/raster/CSS URL/import resources. Relative
origins, nested dependencies and restored links/absref metadata have scoped evidence.
Confirmed stage32 capture/custom-export directory planning preceded unsafe-asset refusal;
fixed before all output planning. Earlier route harness mistakes (missing raw gate and
expecting accurate find_objects to error instead of safe DOM fallback) are retained.
Initial unit expected the already-created artifacts root absent; corrected to newly-created
frames subtree. Actual complete-tree/mtime refusal checks remain strict and pass.

Current Rust209/1ignored, fmt/clippy; actual33 archive cold/warm install, resources17 and
routes8 in both engine modes, security35/special-files12, doctor10/launcher11/notices9
(102crates), discovery16 and manifest drift check pass. GUI/native/performance evidence
remains historical, not transferred. Current-package.json and stage33-build index artifact.

Next: remaining error/output/publication audit, dependency source/provenance and scoped
current-native smoke assessment, then requirement-by-requirement completion/delivery.
PI/xml:base and non-UTF-8 stylesheet syntax are explicit supported-surface limits. No
new GUI launch, window closure, user document/config mutation or remote CI/publish occurred.
Legacy Python MCP is retired; runtime helpers/supervisor remain. Windows, clean machine
and historical incident investigation remain user-deferred; timing is secondary.

## Historical checkpoint notes

Stylesheet checkpoint: development safely stages local UTF-8 @import with nested bases,
order/media suffixes and shared depth/count/byte limits. Actual CLI17cases:9outside
refusals,8positive pixels/8SVG exports. Direct native CLI8fixtures prove @import works;
selected PI/xml:base behavior differs, so explicit unsupported refusals remain documented.
Rust208/1ignored, clippy pass. Next: broader engine route checks and fresh package/notices,
remaining error/output/publication audit and provenance. Stage31 archive is unchanged.
See RUST_SECURITY_AUDIT.md; native GUI and other versions are not inferred from CLI.

CSS URL checkpoint: development uses pinned cssparser0.38.0 (10 added dependencies),
stages ordinary CSS URL assets and restores CSS links. Real CLI13cases:7outside refusals,
6positive pixels/6SVG linked exports, Rust207/1ignored and clippy pass. Initial probe prefix
collision was a harness false positive, retained/corrected with strict assertion unchanged.
Next: imports/stylesheets+xml:base compatibility, broader route checks, fresh package with
updated dependency notices, remaining audit/provenance. Stage31 archive is unchanged.
See RUST_SECURITY_AUDIT.md and renderer-assets-css-debug-v4; no native/GUI claim.

Use/feImage checkpoint: development stages SVG fragments and raster feImage resources,
restores absref metadata for CLI edits (including namespace recreation). Actual CLI11cases:
6outside refusals/no publication,5positive pixel checks and5SVG linked exports. Rust205/1
ignored; staged assets remain bounded. Next: complete CSS/xml:base handling and broader
engine route acceptance, fresh package, remaining security/provenance audit. Stage31
archive is unchanged. See RUST_SECURITY_AUDIT.md and use-feimage-debug-v2 evidence.

Nested asset checkpoint: development uses original SVG asset base, recursively stages
SVG image dependencies with depth8/count128/aggregate limits. Real CLI7cases:4outside
refusals (including nested),3positive PNG pixels and3SVG exports preserving links. Rust205/1
ignored and clippy pass. Current stage31 archive is unchanged. Next: external CSS/use/feImage,
metadata restoration, wider route checks and fresh package; then remaining audit/provenance.
See RUST_SECURITY_AUDIT.md and renderer-assets-nested-debug-v2 for scoped evidence.

In-progress asset staging (2026-10-03): engine_input owns bounded workspace-read raster
copies for all headless engine input routes, restores links on outputs; development real
CLI checks refuse outside absolute/fileURI/symlink refs and preserve inside PNG/SVG links.
Stage31 archive is unchanged/unsafe for external asset isolation. Nested SVG/CSS dependency
compatibility, relative base semantics/metadata restoration, broader routes and fresh package
remain pending. See RUST_SECURITY_AUDIT.md; do not transfer this to native or stage31 proof.

# Current Rust development status (2026-10-03)

Current package stage31 includes namespace fix and XML open preflight fix. Malformed
open used to write managed copies/registry before failing; now refused before all writes.
Rust202passed/1ignored + fmt/clippy, helper6, packaged35security/12special-file checks,
cold/warm CLI/STDIO archive install, doctor10, launcher11, notices9 and discovery16 pass.
See RUST_SECURITY_AUDIT.md for exact proof/limits; current-package.json indexes artifact.

Legacy Python MCP server and paired comparison scripts are retired. Required Python
components are only runtime/helpers and rust/package/supervise.py; frozen JSON contracts
remain schema/instruction/regression data. Use Rust-only iterations. README/CONTRIBUTING
and generated llms describe actual Rust operation.

Confirmed next fix: stage31 renderer reads owned outside-workspace PNG via absolute/file URI/
symlink references. Failing owned regression and pixels saved; see RUST_SECURITY_AUDIT.md.
Next: close renderer dependency reads, then focused security/behavior review (caps, errors,
publication boundaries), dependency source/provenance, final delivery. Do not redo whole
suites without changed code/failure/concern. Native/performance stage29 results describe
stage27 only. Historical incident research is deferred until recurrence; Windows backlog;
clean-machine install user-owned; timing secondary. No user GUI launch/close in this audit.

## Historical checkpoint log (newest first)

Entries below describe their individual checkpoint; “pending” there is historical.
Use the current status and report above to determine the next work.

Namespace review fix (2026-10-03): ordinary SVG attribute reads/removals use explicit
no-namespace methods across37 Rust DOM modules; foreign q:fill/q:style/q:id no longer
cause false no-ops, metadata removal or wrong targets. Two regressions failed before,
pass after; fullRust201/1ignored, fmt/clippy, Python1211/90skipped + Ruff/mypy pass.
Fresh release15namespace, debug71read/122realCLIedit and release16discovery comparisons
pass without differences. See RUST_REVIEW and namespace-review-comparison.json.
Existing stage27 packages/native/perf evidence are PRE-FIX; no install/config/restart
or GUI mutation occurred. Rebuild/revalidate packages before transferring evidence.

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

# Продолжение работы

Stage28 isolated transport diagnostic:50 frame/handler+50file/process partitions pass,
5pairs160requests/20full inspection envelopes equal/sampler errors0. Normal stage27 unchanged.
Initial flush envelope subtraction refused: flush may overlap client roundtrip; v2 read-to-first
write + residual additive, write/flush separate. Clock epochs differ, duration comparison only.
No pureIPC/native/production claim. See RUST_PHASE_MEASUREMENTS/transport-profile-stage28 binding;
continue Python/warm/live boundaries, native-positive/package live, Windows/foreign/licensing.

Stage27 filesystem review adds test-only FIFO regression for4read/hash/lock paths; injected
O_NONBLOCK removal fails in1s without hang, exactsource restored. FullRust199/ignored1,
clippy/fmt pass. Actual unchanged package12special-file/escape STDIO refusals preserve external
original and create zero managed files. No production defect/fix claimed; Windows/foreign
port still pending. See filesystem-review-stage27 and special-files-stage27 evidence.

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

Stage27 synthetic live: 15 families/651 observations pass on byte-identical unbundled release.
Owned fixed peers/fake boundaries, real CLI discovery; no current native/package live proof.
Current gzip+trace coverage union4619calls, all110called/109success; live_edit_selection only
8structural socket refusals. Initial reference discovery PATH failure retained/repeated alone.
Explicit loop10/diff18encoded deltas, managed7/mutation2safety deltas retained; zero unexpected.
See current-live-stage27-coverage.json and migration report. Next current production benchmark,
argument/prompt/resource audit and native-positive/package live gate; preserve old bindings.

Stage27 remaining headless review passes 16 release families/1715 observations and 11 actual
package families/1462 (overlapping). Runner fixes terminal-state/report format; coverage fixes
gzip omission. Corrected current trace union4145calls/84of110 names, remaining26live; not full
argument/native proof. Failed PATH/auto-discovery/fontconfig attempts retained, selected passing
reports reference exact original raw dirs. No server source changed. See migration report and
current-headless-stage27-coverage-with-package-v2.json; continue live/current perf/platform gates.

Stage27 affected-family cohort completed: 15 families, 1035 scenarios/profiles pass on
packaged binary. Isolated copies change only paths and preserve old reports. Initial prep
rejected argument-only recovery before launch; v2 passes. Coverage union with current
byte-identical release/archive traces: 1155 tool calls, 53/110 names with successful replies.
Remaining57 in current-family-stage27-coverage-with-package.json; presence is not exhaustive
behavior/args/native proof. Next current read/export/actions/path/quality/live audit, plus
native monitor, pure IPC, foreign-platform and licensing/source-offer gates still pending.


Current package stage27: stage26 no-op fix, exact tested fresh-release bytes, cold/warm
archive installation plus full-tree mtime no-op checks pass; discovery16/doctor10/notices9/
launcher11 pass. Helper/native components unchanged from stage23. Current pointer indexes
package-stage27-build-comparison.json. Native/performance stage27 pending, not transferred.
Earlier stage23 native launch refused without primary monitor. Preserve all old bindings.


Stage26 sync spans found transient audit creation on DOM no-op. Source fixed: 22 memory-only
families use transaction::apply_dom; actions/paths/fit keep pre-dispatch audit apply. No sync
or safety gate removed. Directory-mtime regression failed before fix; full Rust198/ignored1
and clippy pass. Fresh release STDIO preserves full tree bytes/sizes/mtime on no-op. Existing
stage23 archive is pre-fix; build a replacement package after affected comparisons complete.
Pre-fix sync timing is separate from fixed release validation. See RUST_PHASE_MEASUREMENTS.


Stage25 paired diagnostic: 50 Python ContextVar middleware requests + 50 Rust handlers
partition correctly; full small/100-object inspection envelopes match, sampler errors0.
No normal server/package change. See docs/RUST_PHASE_MEASUREMENTS.md and
migration/phase-profile-{python,rust}-stage25-comparison.json. Rust logical-file edit
region is larger; next measure file/directory sync_all separately before changing anything.
Pure IPC, unguarded IO, warm/live and native-primary-monitor gate remain open.


Stage24 diagnostic phase work: scripts/migration_prepare_phase_profile.py creates an
isolated source copy; diagnostic spans are never compiled into the normal server.
Corrected 5-pair run binds/partitions 50 Rust handlers via interval union. Initial nested
stderr logging polluted file spans and is retained separately; v2 buffers outside handler.
Read docs/RUST_PHASE_MEASUREMENTS.md. Pure IPC/Python/warm/live and unguarded file
coverage remain pending; stage23 is still the installable candidate. No GUI retry occurred.


Stage23 live follow-up: 134 synthetic observations pass (connect26/loop46/mutation62),
intentional uncertainty/encoding differences retained. Ninth isolated managed launch attempt
at /private/tmp/imcp-native-60hza24y terminated: no primary monitor guard. No surviving
owned process, no retry/unlock/security change. Native stage23 still unverified, earlier
stage21 disappearance unexplained. See live-stage23-followup-comparison.json and
native-stage23-launch-comparison.json; continue independent phase/platform/coverage work.


Stage23 measurement follow-up: five alternating direct pairs plus five diagnostic CLI pairs,
360 requests, 60 complete inspection envelopes equal. Startup median Python/Rust
824.42/20.18 ms; inspect 100k 15322.69/1966.01 ms. No general edit/render speedup.
One retained Python direct sampler ESRCH; memory observations incomplete, large response
RSS still high in both. Raw process-benchmark-stage23 and process-benchmark-cli-stage23;
comparison/followup JSONs bind the exact stage23 binary. Native stage23/current live and
exclusive server/IPC/logical-file timing remain pending. Historical bindings unchanged.


Обновлено 2026-10-03. Это индекс состояния, не дополнительные разрешения.
Перед действиями проверяй Git/PR/процессы; не закрывай GUI с несохранённой работой.

## Текущее состояние main


### Stage23 source build/setup and PATH regression fix (2026-10-03)

setup.sh now invokes scripts/build-local-package.sh from a checkout; a ready archive
still auto-detects itself and needs only Inkscape. Native build prerequisites are detected,
SDK/PATH configured, helper Python 3.12.14/native architecture/six pins checked. Existing
pinned runtime is reused or installed uv manages the helper environment. No system package
manager/profile/MCP-client config is changed; no GUI launch/restart occurs.

Isolated fresh source copy/empty target build passes with intentionally wrong Cargo
external target/output settings: explicit native artifact is packaged, stale implicit release
is absent. Empty-PATH launcher initially failed dirname; all three scripts now bootstrap
system paths and the fixed build/setup/doctor/STDIO succeeds. Stage23 actual cold/warm archive,
ten doctor profiles, nine notices checks, eleven launcher checks, 16 discovery profiles and
122 edit scenarios pass. UV download/foreign/fresh-OS paths are unverified.

Stage23 binary SHA dfadfbc0a11cc14a1454c40d35956348070b353115ed726b8b450035b73afd1f
has no native GUI/performance evidence yet. Do not transfer stage22 results to it.
Raw: migration/results/source-onboarding-stage23/. Summary/binding:
migration/source-onboarding-stage23-comparison.json and -evidence-binding.json.
Current commands/archive are in RUST_MIGRATION_REPORT.md. Next: final native/history,
phase measurement, target ports/validation and coverage audit. Preserve all retained GUI.

### Stage22 fresh native and read-only benchmark follow-up (2026-10-03)

Fresh stage22 explicitly owned session: supervisor 30626, native PID 30645,
/private/tmp/imcp-native-l3sosx3s; see results/native-gui-stage22/session.json.
46 fixed effects/history + 38 structure + 22 transform + 16 lower/order checks pass,
with exact-tree/ID/RGBA and captured-only rechecks. Five paired Python/Rust live reads:
80 requests and 40 comparisons pass; every scene has seven objects. After benchmark,
Select All still has seven; group-apply proceeds to eight. Stage21 disappearance did not
reproduce in this attempt, but no cause/fix is proven. Stage21 state is preserved.
Final stage22 fixture is restored to seven objects and retained; do not relaunch/repair it
without examining the evidence/current PID/path/parent. Eight synthetic sessions have now
been opened total; six known remaining sessions are preserved. No user windows were changed.

Current data and UI/harness incidents are documented in RUST_MIGRATION_REPORT.md;
binding: migration/native-stage22-evidence-binding.json. Native UI capture error after Redo
was handled by inspecting completion, not repeating the action. Empty-selection and existing
sync-destination guards refused before dispatch; lower/order uses a separate owned workspace.
Automatic source build after clone, Windows/other-host validation, phase attribution and
history investigation remain pending. Review's earlier no-GUI checkpoint is historical.

### Review cycles and stage22 shell onboarding (2026-10-03)

Read [RUST_REVIEW.md](RUST_REVIEW.md) and the finite
[RUST_COMPLETION_CHECKLIST.md](RUST_COMPLETION_CHECKLIST.md) first for migration follow-up.
The high-risk process/audit review fixed three conditions: continuously readable pipes
could defeat completed-child drain bounds; ignored fcntl failures could enter blocking reads;
failed post-dispatch live audit writes could hide completion uncertainty. Focused regressions
and rereview pass. Full gates: 197 Rust / 1 ignored, 1289 Python / 12 skipped, fmt/clippy,
Ruff (325 files) and mypy (122 sources). Fresh release: 16 exact discovery profiles,
122 edit scenarios and 62 synthetic live observations with no unexpected differences.

Stage22 includes those fixes plus setup.sh/run-mcp.sh. Cold/warm installation from its
actual archive with empty PATH, ten doctor profiles, nine notice checks and eleven launcher
checks pass. Settings are literal data, not shell source; no user MCP config was changed.
The source-first automatic package build after Git clone remains pending. Stage22 has
**no native GUI acceptance** or fresh benchmark evidence. Stage21's disappearance is not
fixed or causally explained by the review changes. Preserve its drawing and earlier evidence.
No new GUI was opened/closed/restarted. Other-platform execution, Windows port and exclusive
phase attribution remain pending; no credentials are needed for local source/archive setup.

Raw review evidence: migration/results/review-cycle1/; summaries:
migration/review-cycle1-comparison.json and migration/review-cycle1-evidence-binding.json.
Current install commands/archive hashes are in RUST_MIGRATION_REPORT.md. Full goal remains
incomplete; do not mark historical scoped evidence as final-candidate acceptance.

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

[The helper investigation](RUST_HELPER_INVESTIGATION.md) documents a feasible compiled
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
`scripts/migration_native_gui_acceptance.py` refuses a second launch while that ownership record
exists. `scripts/migration_native_capture.py` attaches only to the existing private bus and checks
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
.venv/bin/python scripts/migration_doctor_acceptance.py --package migration/results/packages/inkscape-mcp-macos-arm64-stage6
.venv/bin/python scripts/migration_package_acceptance.py --archive migration/results/packages/inkscape-mcp-macos-arm64-stage6.tar.gz
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

### Active Rust migration (2026-10-02)

Local branch `codex/rust-migration` starts from clean `c50a924`. The full objective is in
`/Users/bm/.codex/attachments/10e0e4a1-1fc1-48af-9f93-2b78ff295044/goal-objective.md`.
No commits/PRs/releases/messages are authorized. The goal is active, not complete.
See [Rust migration checkpoint](RUST_MIGRATION_REPORT.md) and
[contract/measurement evidence](../migration/README.md).

Python remains the usable reference server. Rust currently implements STDIO discovery,
workspace/open/create/reload, aggregate inspect/validate/find_objects and document resources, safe save and bounded artifact reading,
the shared transaction kernel, five style/color tools plus replace_text/set_font and move/scale/rotate plus resize_canvas/normalize_viewbox and fit_to_content plus rename_object/delete_object and eight vector primitive creators and two gradient creators plus create_group/set_group_mode/create_use/group_objects/reparent_object/duplicate_object/tile/repeat_objects, style/color/text/font/transform/canvas atomic batches and snapshots;
all seven prompt renderers plus prompt index and headless public PNG/PDF/SVG render/export and frame series plus historical region comparison and bounded export_batch plus web/icon/print profiles and export_set; the full frozen
discovery surface includes pending tools which return explicit migration errors.
Do not confuse 16/16 discovery parity with complete behavioral migration.
Checks: 71 native read, 122 edit, 50 save/create, 100 prompt, 74 render, 21 export-batch and 28 profile and 17 export-set plus 93 vector-authoring and 42 reparent/XML plus 19 duplicate and 27 tile plus 45 filtered-search and 11 reload and 19 repeat and 12 retention and 19 artifact-stat and 820 intent-guidance and 126 quality and 161 optimizer and 152 SVG adoption and 131 fragment and 69 placement and 102 grid and 116 selector-transform and 170 path comparisons match;
163 Rust tests, format/clippy pass. There are 107 native dispatches and 3 pending tools; all 18 resources are native.
Real operation-preview PNG pixels, audit records and snapshot hashes match Python.
Native grouping also has five explicit appearance-safety refusals with unchanged source/working
bytes and no snapshots; see the report for intentional differences from the legacy remapper.
Native reparent compensation preserves transforms, paint order, mixed tails and inherited/conflicting
namespace bindings; five stronger legacy-mode refusals are documented. Undefined XML prefixes now
refuse even when libxml2 returns a document pointer. Read/save comparisons were rerun successfully.
Native duplicate and batch reuse rename reference rewriting, preserve tail/namespace order, and
validate unique intra-clone IDs. Complete SVGs are compared with only validated random ID suffixes
bound; pixels/history match. Five intentional reference-safety refusals are documented separately.
The 122 edit scenarios were rerun after sharing the helper.
Native tile shares the copy kernel, occupied IDs and incremental size accounting. Real 128- and 1024-cell
grids and no-op/rollback/error scenarios match; a 512-byte native guard preserves originals/working
bytes and snapshots. The 1024-cell upper-bound render (original plus 1023 copies) also matched
full SVGs/history and unique-ID checks; cap/target-error precedence is tested separately.
Native filtered search matches authored ObjectRefs, reference CSS paint filtering and Unicode text,
with one private bounded CLI query for accurate bounds and DOM fallback on engine faults.
Malformed/duplicate/nonfinite CSV rows and a separate native stdout-cap fallback are checked;
working/original bytes and history stay unchanged. Read comparisons passed after sharing ObjectRef.
One initialization timeout remains unexplained; subsequent complete repeats passed.
Native reload snapshots before source resolution and keeps identity; source/seed/fallback/error
behavior and pre-reload restoration match Python. Malformed replacements refuse before copying
(an intentional stronger guard); read-only working-directory injection verifies baseline rollback,
unchanged working/source bytes and retained snapshots. Two-file crash journaling remains pending.
Native repeat now creates bounded root-space linked/copy instances in ordinary named groups,
with affine compensation, local anchors, tangent orientation and compatible seeded jitter.
19 STDIO comparisons include raw SVGs with strictly validated copy-ID bindings, real preview
pixels/history, dry-run and late refusal with discarded records. 30 planner cases also match.
1024-instance/size/locked/reference guards are unit-tested; this is not GUI acceptance.
Full schema-error precedence and broader namespace/scalar/limit cases remain pending.
JSON precision features are enabled; 71 read and 122 edit comparisons passed again.
Native explicit pruning now matches snapshot keep-union/count/byte policies, orphan records
and root live-frame age/byte budgets with protected preview/diff frames. 12 STDIO observations
include idempotence and unchanged original/current SVGs; a separate native tampered-basename
and symlink guard passes. Boot-time sweep and crash-consistent pruning remain pending.
Native artifact stats now stream SHA-256 through pinned no-follow descriptors with a 1 MiB
buffer. 19 exact wire comparisons cover binary/empty files, roots/links/path and size failures,
ordered/repeated sets and unchanged bytes/no document state. Native directory/path-count guards
are separate intentional safety differences. An 80-byte metadata output-cap fixture passed; concurrent-write injection remains
pending acceptance; the code bounds growing-file reads and rejects changed length/mtime.
Private live_bus now implements bounded typed gdbus IPC (fixed actions/parameters/target/context),
broker GetNameOwner → validated unique owner → Actions.List and owner-pinned later calls. Four
new tests compare 25 exact Python GVariant fixtures and fake CLI argv traces for context/window
params, missing/bad owner, 1 s timeout/nonzero/capped mutation → uncertain with no retry/owner
quarantine. ContextChanged remains a fixed actionable refusal. Shared process retains bounded
stderr internally for its classifier. Initial 200 ms connect timeout cause is unproven; production
1 s floor + 2 s stalled action passed. All 98 Action-discovery comparisons passed after shared
stderr retention. No actual bus/GUI acceptance or public count change.
Private live_context now parses bounded string/tuple/list replies without evaluation. 54 exact
Python fixtures cover identities, titles, escapes, invalid shapes and errors; native tests bound
depth/nodes/bytes/row count and reject raw NUL and invalid Unicode scalar escapes. One additional
fake CLI test reads active/list contexts through the pinned owner. No public surface change.
Private live_dbus now implements the common Transport: plain-SVG/PNG export, parsed active doc,
viewport through a fixed integer window path and selection style/transform actions. 22 captured
Python backend cases compare internal results and fixed action traces. Three explicit preflight
differences retain legacy traces: pan refuses before introspection; malformed transform tails
refuse before any mutation. Entire plans include byte-cap preflight. Export reads use no-follow
regular-file descriptors and size bounds; missing/symlink/oversize/invalid XML/UTF-8 and owned
temp cleanup tests pass. This is fake CLI evidence, not actual D-Bus/GUI/Undo; public count unchanged.
Private live_managed now provides no-follow stdout/lock handles, deadline-bounded flock, selected
task guards, list/select and scoped managed SVG/PNG reads. Shared same-thread/same-stream scopes
reuse the captured context and lock; different sessions refuse nesting. Error/unwind cleanup and
separate-process lock contention are verified. All guarded export actions use UUID Context.Activate,
not app-global Actions.Activate. 50 Python task-guard fixtures match; five new tests pass. Actual
managed selection/scene/helper effect transactions and liveness probes remain pending; only finished
read commands are advertised internally, and no public live tools are counted as ported.
All 71 headless read comparisons passed again after sharing regular-file handle validation.
Private Managed selection now uses pinned positional stdout reads, select-list/query-x fencing,
1 MiB/10,000-ID bounds, Unicode/dedup/root-ID filtering and timeout without retry. 58 exact Python
parser fixtures cover numeric fences, incomplete lines, splitlines/strip behavior and invalid replies.
Managed export now resets sticky id/id-only/text/plain options and retains Inkscape metadata;
region rendering maps root user units to pixels. Five new tests pass (126 total), including fake
CLI scoped selection and mapped export traces. Actual managed geometry/error parity, scene/inspection,
helper transactions and GUI acceptance remain pending; no public live tools are counted as ported.
Private live_scene now reuses headless ObjectInfo/tree helpers for scene and selection inspection.
140 exact Python fixtures match canvas/viewBox, metadata, visibility/inheritance, ancestor-transform
bbox refusal, duplicate/stale/reordered selection and hierarchy. Unicode/underscore numeric text
decoding uses frozen Unicode 15 decimal starts; 705 Python float fixtures match, and shared bbox
numeric parsing now accepts Unicode decimal prefixes. Scene guards cap elements/selection at 10,000.
Managed scene/inspection run selection and SVG export under one context; fake CLI trace tests pass.
Four new tests bring total to 130. No public dispatch count change or actual GUI/Undo acceptance.
After the shared inspection/decimal refactor, all 71 read and 45 find scenarios passed again.
Private live_effect now provides exact helper document fingerprints and typed nonce-bound edit
reply parsing, with captured refusal whitelist/fallback. 11 Python fingerprint and 18 reply fixtures
match. Hashing streams records with a bounded expanded-record budget (min(8×input cap,64 MiB));
namespace amplification refuses before any effect. Exchange prepares fixed files atomically/mode
0600, reads no-follow bounded regular replies, classifies stale/malformed as uncertain and cleans
once only. Three additional filesystem/cap tests preserve originals and avoid late Drop deleting
a later request. Five new tests bring total to 135. No actual effect activation/Undo/public change.
Private Managed now connects edit Exchange to fixed edit-effect activation and fingerprint
confirmation, with selected task guards and cross-client scope across selection/pre-export/request/
effect/post-export. Style/text and eight typed order/structure commands are supported internally;
legacy helper-absent fill fallback remains guarded. 13 exact Python request/result/refusal fixtures
match with validated nonce binding. A fake effect changes synthetic SVG; Python helper-code SHA
is confirmed by Rust. Lost/stale/missing/mismatched and guarded context-switch outcomes stay
uncertain with one activation; a trustworthy refusal after lost activation is recovered safely.
Applied replies after transport loss remain uncertain because unique-owner binding is quarantined.
Four new tests bring total to 139. Initial trace tests counted Describe as activation and a switch
test used unguarded legacy mode; corrected method filters/guarded setup pass. No actual native
effect/Undo/GUI acceptance or public count change.
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

All actual native Undo/packaged runtime/build/doctor/benchmark requirements remain active.
Private live_transport/live_session/live_cache now provide common semantic interface + Socket
adapter, read/no_freeze capability ranking, teardown/attach/document-capture and session cache.
872 exact Python status/recovery + 12 cache fixtures match without probes/GUI. Tests cover all
five connection states, guarded/legacy identities, helper reconciliation, LRU floors/replacement/
coalescing/oversize exception, failed attach cleanup and operation RAII end on error/panic.
Actual loopback Session→Socket reads and reconnect reset tokens/change/cache; six new units,
total 110 Rust tests. Public 84/26 count unchanged. Actual history-file clearing is a pending hook;
public probes/config/MCP wiring, cache keys/frames/event wait and DBus/managed guards are required.
Next implement native DBus and managed transport probes/IPC, preserving unique-owner activation
protection and task binding; connect through this Session with explicit-only launch semantics.
Private socket now covers all semantic methods, including render/export binary decode, four typed
viewport enum modes and string-map transform/SVG/text writes. 29 captured Python strict base64
cases match exact decoded bytes/error categories (including padding-bit semantics); this is not
PNG visual validation. TCP tests verify fixed request params and Unicode/mutation model, refuse
nonfinite view args before IO, and preserve uncertain after lost insert reply without retry.
Three new tests, total 100. Public live policy/approvals/bounds/fragment/text/style validation,
records/previews/task guards, session/reconnect/backend/native Undo/package acceptance remain.
Next implement the common live transport interface and session state/cache/teardown, then wire
socket and DBus/managed backends into public MCP without relaxing startup/launch/GUI guarantees.
Private live_models now has 658 exact captured Python-reference fixtures for document/selection/
inspection/mutation/viewport/scene and token hashes, including 256 seeded float cases (-300..300),
malformed/default/Unicode/bool, large integer count, signed-zero and rounding behavior. Native
socket reads internally model fixed active-doc/selection/inspection/SVG/scene/token commands;
scene performs a separate authoritative document read, ignoring payload identity. TCP test proves
command order/readback; units verify first-10,000 caps before filtering and unchanged token digest
for a dropped 10,001st ID. Three new tests; total 97. Public live tools/resources remain pending.
Next session/backend wiring must include typed render/view/edit wrappers, event wait/cache/audit,
managed document guards, supervisor/helper packaging and real native Undo/Redo acceptance.
Private native live_protocol/live_socket kernels now implement fixed v5/all 14 enum commands,
loopback/token-bearing handshake, bounded framed IO and disconnect. Seven new Rust tests use
96 exact compiled Python-reference cases and real loopback peers for all commands/faults.
Lost/untrusted mutation reply is uncertain with no retry; failed stream is dropped. View-only
faults remain communication failures, explicit rejection remains rejection. Slow trickle has a
whole-request deadline. No-follow rendezvous file/ancestor/size/token guards retain protected
bytes. These modules have limited temporary dead-code allowances pending session/tool wiring:
public live dispatch is still pending, so do not count them as completed tools or GUI acceptance.
Next build typed live result coercion and transport/session abstraction, then native managed/DBus
IPC and public live pipeline; all 26 live tools/five resources and packaging/Undo remain required.
Native list_capabilities/diagnose_runtime/runtime capabilities resource now share one per-server
cache with fresh registry overlays. 46 observations cover all sixteen gated profiles, real/absent
engines, cached reads without re-probes and version-changing refresh shared by tool/resource.
28 explicit python_version differences are retained (native says not applicable, Python reports
its interpreter); exact_parity=false, zero unexpected differences. Full wire traces and matching
text/structured/resource fields are checked; only validated UTC timestamps are bound. Registry
count/purpose/risk/49 intents match. One unit tests overlay refresh/sort without changing probe.
Five live resources and 26 live tools, helper packaging, warm-shell/native acceptance remain.
Native list_actions/discover_extensions now have 98 comparisons over fourteen runtime cases:
real/absent/failed/empty/timeout/unparsed/nonexec/signal engines, inkex/data, fonts, bus and operator
allowlists; complete wire/notes/map content/counts and unchanged original/working SVG/no history match.
Three native guards reject truncated Action stdout and inkex file/ancestor symlinks, preserving
protected sources. Broker GetNameOwner + validated unique destination prevents D-Bus activation;
mock traces prove no well-known destination/no Actions.List without owner. One parser/owner unit.
Shared binary discovery now uses X_OK and captures real exit/signal codes. All 110 Action/32 fault
observations passed again. Packaged Python/helper runtime reporting and further failure/race/coercion/warm-shell/GUI/package acceptance remain pending.
Native validate_action_chain/run_action_chain/run_raw_action now use operator allowlist, version
map and fixed grammar with 32×16 bounds, approval and shared private CLI/pipeline. 110 observations
match full wire/plans/argv, persisted map fields (validated UTC timestamp only normalized), SVG
hashes/history, actual previews and exact restore. Cache bytes stay fixed between calls. Two map
link guards preserve outside files/directories; two units cover gate precedence, grammar/hints/bounds,
ordered parser and safe version keys. 32 raw engine-fault/no-op observations retain six audit safety
differences (proposed vs discarded); absent runtime refuses before audit. Three additional output
guards refuse unchanged SVGs without snapshots. All 170 path/32 path-fault observations reran after
sharing the runner. Native maps probe version/action-list only; full schema/map corruption/staleness/race/warm-shell/GUI/package work remain pending.
Seven native path tools now use fixed high-risk approved CLI Actions and the shared pipeline.
170 actual observations match complete wire, SVG hashes/history, PNG previews and exact restore.
Harness explicitly enables the advanced gate and asserts expected successes; its first hidden-tool
error-only run was invalid. 32 engine-fault/no-op observations have seven explicit audit differences
(Python proposed vs Rust discarded), retained without normalization; all other fields match.
Three separate native output guards refuse wrong root/duplicate IDs/removed-ID refs without mutation.
Four units cover target grammar/order, outline fill/marker scope, original comment/PI siblings and incoming DTD serialization
and structural refusals. Target list capped at 4096. Shared timeout now matches the 1-second floor;
a config unit covers finite bounds; find's genuine 1-second timeout fixture passes all 45 checks.
Full schema/CSS/ref/asset/geometry/warm-shell/failure and GUI/package acceptance remain pending.
Native transform_objects composes find_objects and the shared typed batch for all eight targeted
ops, default dry-run and effective delete approval. 116 observations match complete wire, full
SVG hashes/history, actual PNG previews and exact restore across 21 cases. One unit proves
65-match dry-run planning, 64-edit real-run cap and rejected creation without byte/history changes.
Full schema/error/coercion, failure-injection and GUI/package acceptance remain pending.
Native compose_grid now supports both source modes, repeated assets, existing/new sheets,
row-major ordinary groups, padding/gap and optional downscale. 102 observations match wire,
verified minted IDs, full SVG hashes/lengths/history, real previews and restore. Registry deltas
prove invalid new plans create no document. Three units cover limits/origins/canvas, 28 exact
reference DOM heuristic fixtures and early source-list/aggregate-byte/invalid-plan refusal.
Object mode loads its source once; document mode adds a configured aggregate input budget.
Accurate geometry is not claimed: naive primitive bounds intentionally ignore transforms.
Further schema/numeric/import-context/asset/copy/race/maximum-CLI-grid and GUI/package acceptance
remain pending; create-then-edit stages are not crash-atomic across the whole call.
Native place_document copies rootless/whole SVG or self-contained objects through a common
append/remint kernel, creates one ordinary translated/scaled wrapper, and preserves both sources.
69 comparisons match verified minted ID topology, full SVG normalized hashes/lengths/history,
real CLI previews and exact target restore. Six separate preflight context guards leave both
sources/working copies/history unchanged. Inheritance/defs/relative-asset context preparation,
full argument errors, further composition edges and native GUI/package acceptance remain pending.
Automatic root ID allocation now retries at most 128 times after an observed six-hex collision
in a 1024-tile fixture; explicit IDs still refuse immediately. One deterministic unit proves
retry/exhaustion/explicit refusal, separately from common parity. Two units cover rootless
remap and self-contained references/neutral ancestry.
Native replace_svg_fragment now validates stable/qualified container identity, ID conflicts,
unresolved/removed refs, exclusive C14N no-op and explicit allow_retained appearance changes.
131 observations over 25 cases match whole responses, working bytes/history, real preview
pixels and exact restore. Three units cover early input/approval, canonical no-op and slot/tail/
reference policy. Two separately recorded stronger guards reject removed timing/accessibility
targets with unchanged SVGs/no snapshot/one discarded record. Full schema/canonicalization/
encoding/copy/filesystem injection and native GUI/package acceptance remain pending.
Native approved set_document_svg/insert_svg_fragment share strict allowlist/input/href checks,
staged audit/snapshot edits and post-adopt validation. 152 comparisons match complete wire,
SVG hashes/history, CLI preview pixels and exact restore; 29 cases include qualified/unqualified
roots, DTD/top-level comments/PIs, no-op, wrapped/nested/intact insertion and refusal paths.
Two separate ID-collision guards leave SVGs unchanged, create no snapshot and audit refusal.
The shared XML serializer now matches lxml's top-level comment/PI/DTD spacing; 71 read checks
were rerun successfully. Four new units cover XML spacing, early caps/approval, allowlist/entities
and independent namespace/mixed text copying. Full schema/CSS escape/URI/encoding/failure
edges and further composition edges remain pending.
Native svg_web_optimize/optimize_set now share the existing staged edit pipeline and quality
analysis. 161 observations match: sizes/deltas/summaries, whole SVG hashes, snapshot/operation
history, real CLI PNG pixels, precision 0/2/8, kept references and exact snapshot restore.
Five separately recorded stronger refusals preserve CSS/script/SMIL/accessibility/referenced
metadata targets; they create one discarded audit record and leave SVGs/history snapshots
unchanged. Namespace cleanup keeps every referenced namespace pointer. Sets preserve sequential
partial-success behavior; they are not atomic across documents. Further schemas/numeric/race
and native GUI/package validation remain pending.
Native quality_report/quality_report_set now share validation/font/collection inspection plus
read-only optimization signals and bounded optional editability advice. 126 observations match
with fontconfig present/absent, including viewBox variants, metadata/definitions/coordinates,
rasters/fonts, 200-advice truncation and malformed/unknown/ordered set reports. Source/working
SVGs and history remain unchanged. Three native units cover read-only analysis and output bounds.
Full argument/schema errors and further optimizer edge cases remain pending; no GUI acceptance claim.
Native how_do_i and runtime/intents now share one compiled 49-entry/four-rule table.
820 exact STDIO observations cover all keywords, stable scoring/scope precedence, gates and
exact resource text/MIME. No Python or workspace state is involved. Six live/runtime resources
remain pending; size guards are unit-tested, full argument errors remain pending. Capture
provenance and reproduction commands are in the migration evidence.
The reversible edit/save vertical slice and selected CLI exports are verified. Next:
warm shell,
remaining tool families and full argument errors, live session/IPC, packaging and measured comparisons.
No GUI launch/close or native acceptance has occurred during this checkpoint.

Current checks: Python 1286 passed/12 skipped with Inkscape 1.4.3, Ruff lint/format and
strict mypy passed. The initial Rust checkpoint had 5 XML/POSIX tests and 13 read comparisons;
current Rust counts are recorded above.
Five fresh Python/release Rust runs now record startup/read/edit/export and initialization RSS;
see the report. Startup/read are lower, edits/exports slower; phase/peak/live/package evidence
remains pending. Raw five-run baseline traces remain in ignored `migration/results/python-baseline/`.
Preserve all local migration files and measurement data when continuing.

PR #4 (document context, `bfe9e4f`) и PR #5 (everyday edits, `e6e4e80`) объединены в `main`.
Коммит `d948a25` добавил явный запуск Inkscape и переносимое определение session directory;
текущий HEAD при начале этой задачи — `852c73e` (Add agent guidance and vector authoring rules).
PR #6 (`0f1a766`) добавил live discovery и fingerprinted previews.
Это ориентир, а не требование откатывать более новые изменения.
Origin: https://github.com/P1oN/inkscape-mcp-server.

- MCP startup и `live_connect` не открывают окно. `live_launch()` запускает или использует
  managed GUI только по явному запросу пользователя. Терминальный вариант:
  `.venv/bin/inkscape-mcp-macos --launch`; начальный SVG требует `--launch --document ...`.
- Уже открытый managed GUI и несохранённая работа сохраняются между MCP-подключениями.
  Закрытое окно не открывается заново при reconnect. Для обновления helper/native bridge:
  сохранить и закрыть старое окно, явно запустить новое, затем подключиться.
- Рабочий порядок: `live_connect(prefer="no_freeze")`, `live_list_documents`,
  `live_select_document`, проверка `live_status.ready_to_edit`, чтение сцены/выделения и правки.
  Каждый reconnect сбрасывает привязку task drawing. Окно из Finder не является managed session.
- Managed macOS поддерживает заливку/обводку/прозрачность, document-space transforms,
  простой однострочный текст и duplicate/delete/group/ungroup/raise/lower/front/back.
  Один изменяющий вызов — один Undo; неизменяющий шаг не добавляет Undo.
- Правки готовятся на копии SVG в one-shot inkex effect; проверяются контекст/выделение,
  блокировки и ссылки. Таймаут/неподтверждённый результат сообщает неопределённость:
  проверить рисунок перед повтором. Native integration остаётся экспериментальной.
- Инструкции: [macOS setup](macos-live-prototype.md), [document context](document-context.md),
  [everyday edits](everyday-edits.md). Актуальный полный manifest: [llms.txt](../llms.txt)
  (110 инструментов, 7 prompts, 18 resources; видимость зависит от gates).

## Проверки и доказательства

Проверка документации 2026-10-01: полный pytest — **1142 passed, 74 skipped**
(Inkscape CLI отсутствовал в тестовом PATH). Ruff, format check и strict mypy
(111 source files) прошли. Исторические результаты этапов 2/3
в тематических документах не являются текущим статусом тестов.

Native acceptance этапа 3 после исправлений ревью прошёл на official Inkscape 1.4.3
через настоящий MCP STDIO на разблокированном Mac. Проверены семейства правок, точные
отпечатки Undo/Redo, неизменяющие вызовы, блокировки/неверный выбор текста,
guard/race/stale-binding и STDIO reuse.
Доказательство: `/private/tmp/imcp-context-r79j3lvj/acceptance.json` (`passed: true`),
`stage3-*.svg` и preview PNG в том же каталоге. Эти временные файлы могут быть уже удалены.

Команда воспроизведения: `.venv/bin/python scripts/accept_document_context.py`.
Она явно запускает отдельный тестовый GUI с `--launch`, затем проверяет MCP reconnect
без launch. Успех закрывает только два проверенных синтетических окна; ошибка сохраняет GUI.
Native GUI acceptance не запускался в ходе проверки документации.

Проверка инструкций 2026-10-02: **41 passed** в тестах authoring/prompts/tool descriptions
и `test_llms_txt.py` (включая сверку каталога с реестром). Ruff check/format для двух
измененных Python-файлов и `git diff --check` прошли. Полный pytest, mypy и native GUI
acceptance в этой проверке не перезапускались; результаты выше относятся к 2026-10-01.

## Сессии и следующий объем

Историческая пользовательская сессия `/tmp/imcp-stage2-501` не закрывалась в предыдущих задачах;
это не утверждение, что она сейчас работает. Не использовать сохранённые PID:
перепроверять manifest и command line. Не завершать процессы по имени Inkscape.

Этап 3 уже объединён. Пользователь выбрал следующий объем: шесть улучшений ниже,
последовательно в указанном порядке; реализация завершена для рабочих копий. Общий этап 4 и остальные
долгосрочные цели остаются в [ROADMAP.md](ROADMAP.md).

## Перед началом новой задачи

Прочитай [AGENTS.md](../AGENTS.md), README, CONTRIBUTING и
[agent usage guide](agent-usage-guide.md), затем проверь `git status` и текущие сигнатуры.
В рабочем дереве уже есть незакоммиченные изменения инструкций и документации. Сохрани их:
не делай reset/checkout/clean и не заменяй файлы целиком из HEAD.

В `overview.py` и `prompts/authoring.py` уже добавлены общие инструкции:

- семантические объекты — обычные именованные группы; для новой иллюстрации по умолчанию
  один общий слой, дополнительные слои по назначению или просьбе пользователя;
- не предлагать и не выполнять трассировку PNG, в том числе внешними трассировщиками,
  скриптами или до импорта через MCP; не подменять вектор встроенным растром;
- сохранять порядок, трансформации, стили и ссылки; проверять результат визуально.

Это уже добавленное руководство для агента, а не реализованные новые инструменты или
детектор трассировки. Работающий MCP читает overview при старте: после изменения инструкции
нужен перезапуск сервера, без закрытия пользовательского GUI.

## Шесть улучшений: реализованы для рабочих копий

Реализованы 2026-10-02; текущее состояние публикации проверяй в Git/PR.
Существующие инструкции о семантических группах и запрете трассировки сохранены.

1. **Workspace и артефакты.** `get_workspace_info`, `inkscape://workspace`, root-qualified
   artifact URIs и read-only ресурс чтения с sandbox/size проверками. `open_document` и
   `save_document_as` принимают optional `root_id`; относительные пути по умолчанию по-прежнему
   используют первый root. Абсолютные server paths не выдаются за client paths. Ошибки вне
   workspace указывают на discovery; сохранение во второй root и чтение через MCP Client проверены.
2. **Группы/слои.** `create_group(label, mode)`, существующий label-only `rename_object`,
   `set_group_mode` на том же g и `reparent_object(preserve_appearance=True)`. Последний
   компенсирует affine transforms и требует неизменного глобального paint order. Отказывает
   при stylesheets, CSS transforms, singular transforms, nested viewports, внешних ссылках
   и непустом оформлении/effects/locks на изменяемой цепочке родителей. Legacy default False
   сохранён и не обещает сохранение вида. Batch-параметры и операции синхронизированы.
3. **Редактируемость.** `quality_report(editability=...)` возвращает отдельные optional
   рекомендации и factual observations; не меняет SVG validity/score. Семантические ID задаются
   явно; thresholds настраиваются, советы отключаются и ограничены 200. Это не детектор трассировки.
4. **Детали.** `render_preview(object_id/region)` переиспользует object export или рендерит
   прямоугольник в document user units. `compare_region(snapshot_id, region)` рендерит фиксированные
   bounds/scale/background без restore; разные canvas mappings отклоняются. Resource URIs и
   inline PNG доступны; artistic score не вычисляется.
5. **Фрагменты.** HIGH-risk `replace_svg_fragment` через существующий parser/allowlist и
   approval gate. Корневые ID/tag сохраняются; внутренние ID только при явном включении.
   Конфликты/duplicate IDs, unresolved refs и удаление внешне используемых ID отклоняются.
   `allow_retained` явно разрешает изменение вида surviving references; default отвергает такие
   изменения. Один snapshot/record, no-op без записи; есть соответствующий batch member.
6. **Повторение.** `repeat_objects` по explicit polyline (два пункта — линия) или rectangle grid.
   Count/spacing, fixed/tangent orientation, ограниченные jitter/scale/rotation и seed.
   Linked use и независимые copies различаются; copies переиспользуют remap duplicate engine.
   Dry-run по умолчанию проверяет полную disposable expansion без записи. Max 1024 и предварительный
   size budget; ID group задаётся явно. Anchor — local source point в document user units;
   copies могут совместно использовать внешние defs. SVG curves/path strings не поддерживаются.

Новые изменения относятся к tracked working copies, не к native live mutation protocol.
Автоматические проверки не подтверждают новый GUI Undo; GUI acceptance в этой задаче не запускался.
Для headless edits Undo обеспечен существующим snapshot/restore pipeline.

### Проверки реализации

- Итоговый полный pytest с Inkscape **1.4.3 (0d15f75)** в PATH: **1275 passed, 6 skipped**.
  Команда: `PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" .venv/bin/pytest -q`.
- Ruff check, format check (226 files), strict mypy (121 source files), manifest regeneration
  и `git diff --check` прошли. CI surface smoke обновлён до 110/7/18 и прошёл. Discovery eval: **41/41**, 100% accuracy.
- Реальные PNG проверяют cropped red→blue snapshot, совпадение всех RGBA каналов после
  safe reparent/group-layer conversion и между linked/copies. Это настоящие CLI рендеры,
  не GUI acceptance. Контейнеры/ссылки/отказы/seed/snapshot restore проверены автоматически.
- MCP Client проверил roots, сохранение во второй root, бинарное чтение resource URI и отказ
  после удаления артефакта. Отдельный свежий процесс через `.venv/bin/inkscape-mcp` проверил
  настоящий STDIO: **110 tools**, create/save/resource readback на synthetic workspace.
- Новую native GUI acceptance и native GUI Undo/Redo не запускали. Пользовательские окна
  не запускали/не закрывали; существующий live bridge не изменяли.
- Контракты и границы: [agent usage guide](agent-usage-guide.md).

MCP нужно перезапустить/переподключить для загрузки новых tools/resources/instructions.
Не закрывать пользовательский GUI: startup/reconnect по-прежнему не запускают окно.

## CI follow-up for PR #7

The initial Linux CI installed unsupported Inkscape 1.2.2 from Ubuntu's default archive.
The full-suite job now uses Ubuntu 24.04 and the official stable PPA, with an explicit
runtime-minimum check. Windows mypy exposed unguarded POSIX APIs: managed macOS helpers
now reject Windows explicitly. Missing-directory creation and save use native Windows
no-follow handles with ancestors held against renames; POSIX safeguards remain in place.
Windows-specific tests cover nested creation, overwrite, exclusive writes, symlink refusal
before truncation/descent, and parent rename prevention. Native GUI acceptance is unchanged.

The next CI run passed the Linux full suite. Windows then exposed existing CRLF shell
framing and path separator issues; shell frames normalize CRLF, registry source paths
use portable forward slashes, and DBus export filenames use forward slashes before
GVariant validation. macOS tests requiring actual POSIX ownership/locking are explicitly
platform-gated; the launch-policy fake uses the same socket path construction as production.

## PR #7 review corrections

Roadmap repetition scope explicitly names polylines and rectangles. Linked repeats set
both SVG2 href and legacy XLink href. Render artifacts use the caller's settings, and
object previews reuse unique preview tokens to preserve before/after files. Engine-routing
test settings retain their configured workspace roots instead of constructing rootless settings.
Artifact resources apply max_output_bytes independently of SVG imports. POSIX reads traverse
with no-follow directory descriptors; Windows reads reuse native no-reparse handles with
ancestor rename protection. Size/type validation and bounded reading use the opened file.
Regression coverage includes file/parent/root symlink swaps, post-open replacement, size growth,
non-regular files, explicit roots and repeated object preview preservation. GUI acceptance
is unchanged; these fixes require the usual MCP reconnect to load new code.

Local validation after these review corrections: 1286 passed, 12 skipped with Inkscape
1.4.3; ruff lint/format, strict mypy for macOS and Windows, diff check, MCP surface smoke
and fresh STDIO boot smoke passed. Native Windows read-handle tests run in CI.
