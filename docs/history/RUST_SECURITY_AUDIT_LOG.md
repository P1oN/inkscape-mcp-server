> Historical checkpoint. Counts, commands, package paths and “current” labels
> apply only to the recorded build. See the [history index](README.md).

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

# Current packaged asset checkpoint: stage33

Actual current archive includes private bounded dependency staging and refusal before
capture/custom-export directory planning. Rust209/1ignored, fmt/clippy, assets17/routes8
in both per_call/shell modes, security35/special-files12, cold/warm archive installs,
doctor10/launcher11/notices9 (102crates), discovery16 and llms drift checks pass.
Build index: migration/package-stage33-build-comparison.json. Stage32 capture-frame
workspace-side-effect failure is preserved in engine-routes-stage32-v3; fix changes only
headless preparation order. New regression/actual inventory checks prove selected refusals
leave bytes and mtime unchanged. Initial unit artifacts-root assumption was incorrect:
opening already creates that root; it now checks the frames subtree.

Remaining focused audit: error precedence/output caps and publication failures, dependency
provenance, current-native smoke where needed. Finite passing suites are not exhaustive
security/race/crash proof. PI/xml:base, non-UTF-8 CSS and unsupported URI/functions fail
explicitly. New CI asset/route steps are prepared, not remote results. Live guarantee and
benchmark evidence remains scoped to its original binary.

## Historical audit checkpoints

# Rust compatibility and security audit

Checkpoint 2026-10-03. Current candidate: stage31 macOS arm64. Rust-only regressions,
fixed JSON contracts and actual packaged STDIO are used; no Python MCP oracle.

Confirmed and fixed: malformed open_document inputs previously created immutable/working
copies and a registry entry before reporting an XML parse failure. Stage30 failure trace
and before/after inventory: migration/results/security-stage30-before-fix. The common
headless seed pipeline now parses and builds the summary before any managed write,
then binds the minted document ID to the already validated summary. Error text and valid
open/create results are preserved. Raw live sync seed_entry behavior is unchanged.
This is an intentional improvement over the retired implementation's failure side effects.

Current Rust test suite: 202 passed / 1 ignored; fmt and all-target clippy pass.
Retained helper suite: 6 passed; Python tooling lint/format pass. Automated tests are not GUI acceptance.

| Boundary | Current evidence | Limit |
|---|---|---|
| Tool/schema/annotation/instruction surface | Stage31 16 discovery configurations match frozen JSON exactly, 110/7/18 full surface | Does not prove every behavioral input |
| Arguments/defaults/models | Current Rust tests validate frozen required/default/coercion/model cases; packaged35-case suite rejects unknown fields, invalid numbers, missing arguments and container overflow before writes | Not exhaustive input fuzzing or transport allocation proof |
| XML opening | Malformed, namespace error and depth cases leave byte/mtime inventory unchanged; valid create/open after refusals | Safe parse preserves unexpanded entity nodes; not universal renderer asset isolation |
| XML core | Rust parser tests cover encoding/mixed content/namespace/references, external entity non-expansion, depth/amplification rejection | This is parser evidence, not all Inkscape behavior |
| Filesystem | Current package12 FIFO/directory/outside-link/parent-link tool/resource refusals and no writes; descriptor-based POSIX root/parent/race tests in current suite | Windows backlog; not exhaustive concurrent adversarial races |
| Edit pipeline | Actual stage31 cold/warm archive checks: atomic batch/rollback, snapshot restore, approval refusal, original preservation and strict no-op tree bytes/sizes/mtime | Selected fixture families, not all combinations |
| Resources/prompts | Packaged suite reads10 static and7 document resources with URI/MIME/ID checks; renders7 prompts; invalid/traversal/missing URI refusals; artifact readback in archive suite | Selected contents/binding cases, not every MIME/error variant |
| Live lifecycle | Current Rust tests cover guards, locks, context pinning, timeout/uncertain audit handling; startup/doctor/package acceptance do not create GUI sessions | Current31 has no new native GUI run; historical stage29 on27 remains scoped evidence |
| Installation/engine | Actual31 archive passes cold/per-call and warm/shell install, real CLI pixels/export/resources, doctor10, launcher11 | Clean-machine test user-owned; foreign CI only prepared |
| Dependencies | Notices9 pass with92 crate inventories | Source/provenance/redistribution audit remains separate active work |

Confirmed remaining defect (owned-file probe on stage31): `render_preview` delegates
absolute paths, file URIs and inside-workspace symlinks to Inkscape without applying
workspace read protections to their targets. Three synthetic outside PNGs are visible in
rendered pixels; in-workspace PNG positive control also passes. No user assets, network
or GUI were used and source PNGs remain unchanged. This is a real workspace read escape,
not XML entity expansion. Current package must not be described as isolating renderer
assets. Script `scripts/history/python/rust_renderer_asset_probe.py --require-isolation` fails on this
candidate by design; raw trace/pixels are in migration/results/renderer-assets-stage31-regression.

Next fix must preserve legitimate in-workspace linked artwork while obtaining asset bytes
through descriptor-anchored bounded reads before passing input to the engine. Merely
canonicalizing a filename is insufficient: the child could reopen a swapped symlink.
Inspect all engine input routes (render/capture/export, previews, bounds queries and CLI
edits), nested SVG and CSS dependencies, data URIs, xml:base and stylesheet PIs. Do not
claim isolation from a href-only patch. Preserve original/working bytes and avoid publishing
artifacts on rejection. Staging resources is an execution detail, not authoring embedded
raster replacements. Reject unsupported unsafe dependency forms explicitly before engine
execution instead of silently deleting them or changing appearance.

Remaining focused review: resource/argument error precedence and output caps across broader
families; renderer external asset handling and filesystem failure publication boundaries;
current native package smoke where required by changed live components. No global security
certification is claimed. Historical live incident cause investigation is deferred until recurrence.
Full crash/race/property fuzzing is not represented by the finite passing fixtures.

Scripts: scripts/history/python/rust_security_acceptance.py and scripts/history/python/migration_special_file_acceptance.py.
Build/check index: migration/package-stage31-build-comparison.json. CI now runs the Rust
security suites on candidate packages; prepared jobs are not remote results.

## In-progress asset staging implementation

The development tree now has `rust/src/engine_input.rs`, used by render/capture/export,
operation previews, bbox queries, both fit probes and CLI action inputs. It owns copied
raster assets until processing ends, reads them through Workspace, bounds the aggregate
input+asset bytes, and restores original hrefs for CLI edits/export. PNG/JPEG/GIF/WebP
linked files and raster data URIs are supported. No embedded raster authoring replacement
is made. Current stage31 archive remains unchanged and still has the confirmed defect.

This is an intermediate implementation, not final compatibility acceptance: external SVG
image/use references, stylesheet imports, external CSS URLs, escaped CSS dependencies,
xml:base, DTDs/PIs and foreignObject/script currently fail explicitly before engine execution.
Remaining work includes safe support for legitimate nested SVG/CSS dependency forms,
relative linked-file base semantics, duplicate href/data MIME cases, preservation/restoration
of metadata fallback references, all engine entry-point positive/error acceptance and a
fresh archive. Do not declare renderer isolation complete from these selected fixtures.

Development real-CLI probe (with Inkscape available in PATH) refuses all three owned
outside references with no workspace publication; in-workspace PNG pixels survive and
SVG export keeps links without temporary asset paths or embedded raster substitution.
Raw: migration/results/renderer-assets-first-fix-debug-v4. The initial debug probe without
Inkscape in PATH failed its positive control and is retained; it was not security evidence.
Rust tests cover pinned-copy survival after the source becomes an outside symlink,
restoration, local gradient URLs and selected indirect-dependency refusals.

### Nested SVG and origin checkpoint

Development now resolves top-level assets from the registry's original source path, not
its managed working-copy directory. This applies to historical renders as well as current
input; fit/CLI action probes share the same origin. Nested `<image>` SVG files are parsed,
validated and staged recursively with their own asset base, including base64 SVG image
data. All owned nested staging directories remain alive through engine processing.
Maximum depth is8, dependency count128, aggregate source+asset bytes and positive serialized
growth are bounded by max_input. Cycles fail at the depth/byte boundary. Outside dependencies
of an otherwise in-workspace SVG are rejected before engine/artifact publication.
Absref-only fallback images are refused rather than silently rendered without their resource.

Current development probe has7 cases:4 outside refusals (absolute/file URI/symlink/nested)
with unchanged workspace inventory, plus3 absolute/relative/nested positive pixel checks
and3 SVG exports without private staging links or new raster embedding. Evidence:
migration/results/renderer-assets-nested-debug-v2. FullRust205passed/1ignored; clippy passes.
The previous section's blanket refusal of external SVG images/base64 SVG images is now
superseded by this checkpoint. CSS external URLs/imports/escapes, external use/feImage,
xml:base and complete metadata restoration remain active compatibility work. No current
archive/native acceptance is claimed for this development source.

### External use/feImage and fallback metadata checkpoint

Development stages external `use` and `feImage` hrefs through the same bounded recursive
asset pipeline; SVG fragments are retained and validated as safe IDs. A raster dependency
cannot be used as a SVG fragment/use target. Qualified absref fallback attributes are
removed only from the private engine input and restored for CLI edit outputs together
with original href values; missing namespaces are recreated without prefix collisions.
SVG export keeps the existing plain-export metadata behavior, restores/rebases source
asset links, and refuses a result containing private asset-directory paths.

Actual current debug CLI probe:11cases,6outside refusals preserving complete workspace
inventory,5positive pixel checks (absolute/relative/nested image, feImage, external SVG use),
and5SVG export checks for preserved links/no new raster embedding/no staging path leak.
Evidence: migration/results/renderer-assets-use-feimage-debug-v2. Rust205passed/1ignored;
unit regression also restores qualified fallback metadata after an engine-like output
has removed its namespace. Full metadata roundtrip across arbitrary CLI edits remains
broader than this selected test. External CSS/import/escaped syntax and xml:base are
still explicit refusals; supporting legitimate forms and package validation remain active.
Current stage31 archive remains unchanged. Earlier blanket use/feImage refusals are
superseded only for these staged dependency forms, not for arbitrary external executables,
network requests or unsupported URI schemes.

### Tokenized CSS URL staging checkpoint

Pinned cssparser0.38.0 replaces regex CSS checks (Cargo.lock gains10 new packages). The
parser walks nested blocks/functions with depth32, recognizes decoded escaped URL/import
tokens, leaves non-URL strings/comments/source text intact, and rejects malformed URLs,
imports and unsupported image/src-function forms before engine use. Primary API reference:
https://docs.rs/cssparser/0.38.0/cssparser/struct.Parser.html . New dependency source/license
inventory must be included in the next package; stage31 notices do not cover this change.

Ordinary external CSS url() resources now go through the same bounded workspace-read
staging and recursive SVG preparation. Proxy inputs and dependencies consume the shared
byte/count/depth budget; all owned copies remain alive. CSS URL links in attributes/style
nodes are restored after CLI processing/export. Output-relative/file-URI forms can be
matched lexically against known staged mappings without reading arbitrary output paths.
Private-path detection includes nested staging directories.

Actual debug CLI13cases:7outside refusals/no workspace publication,6positive pixels and
6SVG exports, including an external SVG gradient referenced from inline CSS. Rust207passed/
1ignored, fmt/clippy and tooling lint/format pass. Raw: renderer-assets-css-debug-v4.
Initial probe failures retained in css-debug/v2/v3: exported CSS had the correct original
asset path but the synthetic workspace prefix `imcp-assets-probe-` collided with the strict
private-staging check. Renaming the fixture root to `imcp-render-probe-` retained the strict
assertion; no production defect is claimed from that harness false positive.

Imports/stylesheets and xml:base remain explicit refusals pending scoped compatibility
work; arbitrary URI schemes/network requests are unsupported. Escaped local URLs and
external URL staging are now supported rather than the blanket refusals in older notes.
Current archive remains31. Broader engine route and fresh packaged acceptance pending.

### Stylesheet import checkpoint

Direct Inkscape1.4.3 CLI baseline (8 owned fixtures) proves string/url CSS @import
applies a stylesheet with marker pixels. Inline CSS and ordinary linked image controls
pass; the selected xml-stylesheet PI and relative/absolute/group xml:base fixtures do not
apply their expected resource/style. Raw: migration/results/inkscape-stylesheet-base-probe.
This is finite version-specific behavior, not universal evidence that these syntax forms
are inert. PI/xml:base remain explicit pre-engine refusals, with no silent metadata removal
or promise of compatibility for those unsupported forms.

CSS tokenizer now exposes import URI tokens separately, including escaped @import.
Protected UTF-8 stylesheet reads are copied to private CSS files; recursive imports retain
order/media suffixes and their own source-directory base. Stylesheet URL resources reuse
bounded SVG/raster staging. Shared depth8/count128/byte budgets cover stylesheet content,
proxy inputs and generated growth, and cyclic imports fail before engine execution.
No network fetching or arbitrary CSS/script execution was added. Original CSS and SVG
files remain unchanged; CLI outputs restore the source stylesheet links.

Actual debug CLI17cases:9outside refusals/no publication,8positive pixel checks and8SVG
exports, including ordinary/nested CSS import. Raw: renderer-assets-import-debug.
Rust208passed/1ignored, fmt/clippy and tooling lint/format pass. Earlier blanket import
refusals are superseded by bounded protected local UTF-8 import support. Non-UTF-8 CSS,
unsupported URI schemes, PI/xml:base and unsupported image/src syntax fail explicitly.
Fresh package checks, broader route validation and dependency notices are still pending.

### In-progress bounded validation diagnostics

Development argument_validation now streams Python-style diagnostic repr into a bounded
prefix/suffix collector instead of constructing the full nested representation and a full
Vec<char>. The existing <=50-character repr and long repr25/.../24 format remain exact.
Numbers use the pinned arbitrary-precision Number's lexical str without an extra number
String. Unknown field locations are abbreviated beyond256characters; unexpected user
union tags beyond50characters. These intentional size-limit behavior changes affect long
inputs, while the current frozen ordinary-error matrix still passes unchanged.

Stage33 actual STDIO reproduced an unknown-field error of250527UTF-8 wire bytes; raw:
migration/results/diagnostic-stage33-before-fix. Current debug server passes3oversized
unknown-field/discriminator/nested-invalid-scalar scenarios with <4096wire bytes each and
unchanged workspace inventory: diagnostic-stream-debug. Two Rust regressions check
repr-equivalence and bounded prefix/suffix storage on large Unicode/nested inputs, and
long keys/tags with real frozen schemas. No Python MCP counterpart was executed.

This is bounded validation diagnostic formatting, not a transport/request deserialization
cap, all-kernel error/output guarantee, global RSS proof or input-processing CPU speedup.
Other kernel/resource errors, response families and publication review remain active.
Current package is33 and does not yet include this development fix. CI diagnostic gate
is prepared, not a remote run. FullRust211passed/1ignored and clippy pass.
