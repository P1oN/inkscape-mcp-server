# Current Rust improvement plan

The installation and responsiveness improvements from
[PR #8](https://github.com/P1oN/inkscape-mcp-server/pull/8) are merged into `main`
as `a4dc71f`. The MCP deadline/client ownership follow-up is committed as `ccd1b0d`.
Inspect Git status and [AGENT_HANDOFF.md](AGENT_HANDOFF.md) before continuing.
Historical stage38 completion notes are in [history](history/rust-stage38-plan.md).

The requested installation, client onboarding, skill update, build identity, responsiveness,
archive acceptance, documentation separation and uninstall improvements are implemented in that PR.
PR review corrections cover cancellation cache integrity, linked-worktree build watches,
uninstall rollback, working-tree archive identity, skill conflict proposals and documentation.
Validation and its limitations are recorded in the handoff and local acceptance reports.

## Python removal roadmap

The first goal is a ready package with no project-supplied Python runtime. Removing Python
from active development/CI tools follows afterward. Keep thin Bash bootstrap/launch wrappers,
the existing Objective-C macOS context bridge and YAML workflows. Python removal does not
require rewriting these components or historical migration evidence.

Stages increase in integration complexity until the live helpers are replaced; packaging
cleanup is simpler but depends on those replacements. Each stage must preserve the current
contracts and safety guarantees rather than expanding the MCP execution surface.

| Stage | Scope | Benefit | Complexity | Status / completion criterion |
| --- | --- | --- | --- | --- |
| 1 | Client management: config, check, connect, disconnect, uninstall | Installation management independent of helper Python, including damaged runtimes | Low | Implemented locally in this branch: separate Rust CLI and existing Bash interfaces; isolated real Codex and synthetic Claude acceptance recorded in the handoff |
| 2 | Separate managed GUI supervisor | Native session/process ownership without Python supervision | Medium | Implemented locally in this branch: Rust supervisor; synthetic lifecycle and owned native macOS launch/connect/reconnect/shutdown acceptance recorded in the handoff |
| 3 | Shared helper logic: document fingerprint, fragment validation and edit planning | Reuse Rust XML/style/transform logic and reduce duplicated checks | Medium | Implemented locally in this branch: reusable Rust SVG library, fingerprint/insertion preflight integration and pure typed live edit plans; bounded refusals/no-op regressions, documented live differences and wire representation (see handoff) |
| 4 | One-shot Inkscape insert/edit extensions through INX | Remove inkex from native insertion/editing | High | Implemented locally: fixed Rust executable, full SVG candidates, scoped context/selection guards; all ten native edits, one-step Undo, no-op/stale refusal, Redo and appearance acceptance on owned synthetic drawings without private Python (see handoff) |
| 5 | Socket live bridge and its inkex-dependent perception/geometry | Remove the remaining Python live helper | High | Implemented locally: bounded Rust v5 snapshot bridge, fixed CLI scene/render/export, shared SVG edits and no-op output; relocated no-Python-helper and owned native socket/Undo acceptance recorded in the handoff |
| 6 | Ready-package cleanup: CPython, wheels, wrappers, manifests/notices and doctor | Deliver a smaller dependency set and simpler release maintenance | Medium; depends on 1–5 | Implemented locally: archive omits CPython/wheels/Python helper assets, native doctor/notices and relocated install/headless/socket/owned GUI acceptance passed (see handoff) |
| 7 | Active bootstrap/package/development tooling and regression/acceptance harnesses | Build and validate without Python | Medium–high by volume | Pending: active CONTRIBUTING and CI commands use Rust/Bash; retain historical scripts/reports as evidence without keeping them on the active path |

### Completed stage 3: shared SVG helper logic

`rust/src/helper_svg/` now provides fingerprinting, fixed fragment validation/preparation,
shared affine arithmetic, typed live edit plans and captured-state guards. The server
uses the shared fingerprint/preflight; headless reparenting shares affine kernels.
The planner returns semantic steps without applying them to GUI documents. Deliberate
conservative differences from inkex are documented in
[live-helper-kernels.md](live-helper-kernels.md), including complex CSS and collision
refusals. The stage-4 consumer below now applies these plans. The Python socket helper and
packaged runtime remain until stages 5–6. Validation evidence and conservative
limits are recorded in the handoff.

### Completed stage 4: one-shot INX extensions

The required small Rust prototype passed native insertion/style, selection input,
one-step Undo after a no-op and stale fingerprint refusal before the full route
was replaced. The separate `inkscape-mcp-inx` executable now reads the fixed bounded
managed request, validates captured SVG/IDs/selection, applies a fresh owned plan
and publishes a nonce-bound result. It returns a full SVG only for changes; no-ops
and refusals emit no SVG. The root, namespaces, comments/PIs/DTD and mixed content
are retained; structural edits preserve references and compensate safe transforms.

The supervisor installs the two fixed INX manifests and a quoted wrapper pointing
to the relocated Rust binary. Ready packages, setup completeness checks and doctor
include it; one-shot Python helper assets are omitted. The socket helper still uses
inkex and the private interpreter. MCP tools, instructions and discovery are unchanged.

Fresh local acceptance covers all ten native operations and insertion, one Undo
per changed operation, repeated style/text no-ops, stale IDs/content/selection,
duplicate/text Redo and pixel-identical group/ungroup renders. It uses one owned
synthetic GUI and a copy of the final package with its private Python executable
disabled. Both synthetic sessions were restored to blank and gracefully closed
with verified context/PIDs/bus ownership. Automated, relocated archive and native
results are recorded separately in the handoff. Installed user runtime/configuration,
source/vendor files and user drawings were not replaced; no commit/publication.

### Completed stage 5: socket live bridge

The remaining live extension runs through `inkscape-mcp-live` and a fixed quoted
wrapper, with no Python/inkex fallback. It retains protocol v5, token authentication,
loopback binding and modal snapshot semantics. Framing, SVG/selection/candidate bounds,
no-follow input, atomically published locked rendezvous and fixed owned CLI processes
replace the Python implementation. Scene metadata and null viewport behavior remain;
best-effort geometric boxes use one bounded isolated CLI query. Rendering retains
self-contained paint and simple stylesheets. Conservative asset/CSS/geometry limits
are explicit in [the helper documentation](live-helper-kernels.md).

Style/text/insertion share the existing typed SVG kernels; unchanged sessions emit
no SVG. A socket session adopts its accumulated candidate only on extension exit,
with one native Undo transaction. One-shot managed edits continue through stage 4.
The supervisor and `live_install_helper` install the native launcher; arming and the
context bridge use the actual normalized Inkscape action name. Setup/package/doctor
include the fifth Rust binary and omit the Python socket asset. Fresh package and
owned native acceptance, with private Python disabled, are recorded in the handoff.

### Completed stage 6: ready-package cleanup

Ready packages now omit project-supplied CPython, helper wheels and Python helper
sources, including their manifest/notices entries. Doctor uses native architecture,
fixed asset/bridge/bus and engine prerequisites with no interpreter/vendor-inkex
imports. Fixed Bash launch/bootstrap interfaces, the Objective-C context bridge,
Inkscape vendor resources and private D-Bus dependencies remain. Development/package
scripts still use Python until stage 7.

Fresh relocated archive acceptance covers install/headless/native helpers and an
owned GUI socket/Undo session with no bundled Python. Approvals, originals, snapshots,
Operation Records, rollback and no-op behavior passed; detailed counts, archive sizes
and untested native/foreign-target scope are recorded in the handoff. Installed user
configuration/runtime and published releases were not replaced.

### Next stage: active tooling cleanup (stage 7)

Replace active Python bootstrap/package/development and regression/acceptance tooling
with Rust/Bash interfaces. Inventory the current CI and CONTRIBUTING entry points first;
retain historical sources, vendor provenance and reports outside the active execution
path. Keep the stage-6 native-only ready package, frozen MCP contracts and all safety
and validation gates. Build-tool Python removal is a separate stage from runtime cleanup.

### Validation and delivery rules

- Complete each stage independently with meaningful Rust regression/invariant checks and
  the relevant package/launcher/doctor/notices acceptance. Do not revive Python MCP parity.
- Use owned synthetic sessions for native live changes; preserve all user windows/drawings.
  Startup/reconnect/doctor/client checks must never launch Inkscape.
- Preserve per-operation approvals, snapshots, Operation Records, rollback and no-op behavior.
  No arbitrary code, shell, executable, extension or environment execution through MCP.
- Update documentation/handoff with evidence and remaining limitations; regenerate manifests
  only when their exposed surface/instructions change. Do not transfer old acceptance claims
  to a new build without validation.
- Commits, publication, releases and changes to the user's installed runtime/configuration
  remain separate user-authorized actions. Stage completion means locally validated code;
  it does not imply a published release or clean-machine/foreign-target acceptance.

## Editable vector authoring quality (requested 2026-10-04)

Status: planned; no runtime, prompt or schema changes implemented or validated yet.
This is an additional workstream; the Python removal roadmap above remains pending as recorded.
Implement in the order below. The goal is both appropriate silhouettes and independently
editable geometry, rather than merely a visually plausible render.

### 1. Shared authoring guidance

Add the English guidance below to MCP initialization instructions and `compose_artwork`
through one shared source. Inspect the current contract/template loading first; keep frozen
discovery contracts and prompt templates consistent with the delivered guidance. Preserve
existing vector-only, semantic-group, appearance-preservation and approval policies.
Document flower and fold examples in `agent-usage-guide.md`.

Draft guidance:

```text
When creating artwork, consider both its appearance and its editable structure:

- Choose geometry that matches the subject: use circles, ellipses or smooth closed
  paths for round objects, and avoid unintended sharp corners in soft forms.
  A snowball should have a rounded silhouette. Preserve angular forms when requested
  by the user or shown in the reference.
- Create each continuous filled form as one final outline. For example, a flower
  should have one petal outline and a separate center. Keep parts with different
  paint or independent editing purposes separate. If overlapping primitives are
  used during construction, use a geometric union when appropriate; combining paths
  alone does not remove internal overlaps. Existing approval gates still apply.
- Explicitly set fill="none" on stroke-only lines; do not hide their fill through
  transparency or by covering it with another shape.
- Create each independent stroke as a separate object, and place related strokes
  in a named group. Do not put independent folds, whiskers or veins into one path
  with multiple subpaths.
- Do not leave unnecessary hidden or fully covered construction shapes in the final
  artwork. Preserve elements required by references, masks and clipping paths.
  Partial overlap is allowed. Do not remove existing artwork outside the requested scope.

Before delivery, inspect the SVG structure and rendered image. Check that silhouettes
match their subjects, and look for unintended corners, redundant shapes, fills on
stroke-only lines and independent strokes combined into one object. Correct any issues
found, preserving appearance and references and following the existing approval rules.
```

Acceptance: initialization and the compose prompt deliver the same rules without policy
conflicts; regressions check their delivery across discovery configurations. Regenerate
`llms.txt` and `llms-full.txt` from the rebuilt server. Reconnect/restart the MCP server
when deploying changed guidance, preserving the GUI; do not replace the installed runtime
as part of planning or assume an instruction guarantees model compliance.

### 2. Explicit roles and read-only structural checks

Extend the existing bounded `quality_report` / `editability` report. Define typed, optional
role inputs for specific object/group IDs (for example stroke-only objects and groups of
independent strokes), with count limits and clear behavior for unknown IDs. Do not infer
semantic roles from names, primitive counts or whether a path is open. Keep advice separate
from document validity and the existing quality score.

For designated strokes, report an effective fill other than `none`, including a transparent
fill or an inherited/default fill. Reuse existing CSS cascade resolution; expose unsupported
cases as unknown rather than passing them. Check separate objects versus multiple subpaths
using proper bounded SVG path parsing, not counting command letters. Reuse an existing parser
if suitable. A multi-subpath filled path is not itself an error.

Acceptance: meaningful regressions cover presentation attributes, inline styles, inherited
paint, stylesheets, `fill-opacity=0`, SVG default fill, separate stroke objects and compound
paths. Inspection leaves bytes, snapshots and Operation Records unchanged. Findings identify
object IDs, reasons and uncertainty. Rounded silhouettes remain a semantic/render review;
do not automatically replace rectangles with circles.

### 3. Hidden-geometry review and safe cleanup

First add bounded read-only findings for hidden/transparent scene objects, accounting for
ancestor visibility, styles and references. Distinguish scene artwork from required `defs`,
`use` sources, masks and clipping geometry. Defer general occlusion detection until a separate
bounded design is demonstrated: overlapping bounding boxes do not prove full coverage.
Any occlusion findings must state the supported scope and uncertainty; never auto-delete
objects based only on bounding boxes or an inconclusive render comparison.

Use existing deletion and path operations for explicitly reviewed repairs rather than adding
an unrestricted cleanup tool. Preserve IDs/references, paint order, originals and approval
gates; reuse snapshots, Operation Records and no-op behavior. Compare renders before/after
structural repair; refuse unsupported cases before mutation.

Acceptance: required referenced elements and partially visible shapes survive; approved
removal of redundant synthetic shapes preserves appearance. Regression cases include
transparency, inherited visibility, masks/clipping and external references to subtree IDs.
No silent cleanup of existing user drawings.

### Validation and completion

Run CONTRIBUTING checks appropriate to each implemented stage, including Rust fmt/clippy/tests
for code changes and discovery consistency/manifests for exposed guidance or schema changes.
Use real Inkscape CLI rendering for geometry/cleanup acceptance on synthetic flower, folds,
snowball and occluded-shape fixtures. Keep structural assertions separate from visual checks.
Native GUI acceptance is required only for changed live behavior and must use owned synthetic
documents. Record fresh evidence and remaining limits in the handoff; do not reuse old results
as proof of these changes. No commit, PR, publication or installation is authorized by this plan.

## Other follow-up work (existing backlog)

Remaining follow-up scope after review:

1. Run the source bootstrap on a truly clean Apple Silicon machine, including Apple's SDK
   installation, downloads, quarantine and client installation. Existing-host archive
   acceptance does not establish this.
2. Run Claude Code registration on a real installed client; synthetic CLI/config regression
   checks establish argument routing only. Codex registration is exercised in an isolated home.
3. Measure representative workload latency before finer concurrency changes. Headless
   edits/rendering remain serialized intentionally; do not weaken snapshot/record ordering.
4. Repeat native GUI acceptance only when changes affect live behavior. Preserve current
   user windows; use owned synthetic documents and distinguish GUI results from headless tests.
5. Windows and foreign-target executions remain deferred. Investigate the historical live
   group incident only on recurrence; capture first divergent tree/PNG, selection, document
   and window IDs, audit/wire/logs and Undo state before any restart or closure.

Review and release remain separate actions. Merging PR #8 has not replaced the published
v0.1.1 archive or the user's configured runtime.
