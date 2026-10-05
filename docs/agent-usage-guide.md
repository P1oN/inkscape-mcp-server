# Agent usage guide — driving inkscape-mcp from an agent

How to drive this server from an LLM agent: the core create→render→export loop, the
working-copy + snapshot reversibility model, the risk classes and the approval-token gate for
HIGH-risk tools, and how to pick the right tool. The full surface is **110 small typed tools / 7 prompts /
18 resources** — deliberately *not* a portmanteau `run_action(string)` / `do_task(prompt)` design
([architecture decisions](adr/README.md)). The trade-off: more tools to navigate, but each is explicit, typed, and risk-classed.
Use the discovery tools below instead of grepping the list; gates may narrow the visible surface.
The generated [manifest](../llms.txt) is the authoritative full catalog.

For clients supporting installable skills, the repository includes
[inkscape-mcp](../skills/inkscape-mcp/SKILL.md). See
[skill installation](install/agent-skill.md) and
[client connection/update/removal](install/client-management.md). It complements the MCP initialization
instructions without changing the tools or approval gates.

For managed macOS, follow the [setup guide](live/macos-live-prototype.md). MCP startup and
`live_connect` never open a window. Use `live_launch` only when the user asks to open Inkscape,
then connect with `prefer="no_freeze"`, list and select the task drawing, and check
`live_status.ready_to_edit`. Reconnect preserves an existing GUI and resets the drawing binding.
A closed GUI requires a new explicit launch request.

The server also ships a concise **system overview as MCP `instructions`** (E19-02), provided
at MCP initialization. It covers the document model, `doc_id` lifecycle, snapshot/restore,
risk classes, approval tokens, tool ordering and the render-and-look default below.
The client controls how it includes this guidance in model context. This page is the fuller
companion. Restart the MCP server after changing these instructions; an already running
process keeps its previous copy.

This is the agent-facing companion to the two machine-readable manifests
[`llms.txt`](../llms.txt) (concise index) and
[`llms-full.txt`](../llms-full.txt) (full per-tool manifest),
both **generated from the live registry** by `scripts/dev-tools.sh manifests`.

---

## 1. Picking the right tool

Three read-only discovery tools answer "which tool do I call?" without reading source:

- **`how_do_i(goal)`** — map a natural-language goal ("draw a rectangle", "make my svg smaller for
  web", "find the red shapes") to the concrete tool name(s) + a one-line how-to. It also flags
  out-of-scope goals (raster/pixel editing, arbitrary Actions/extensions/scripts, network fetch, code
  execution) with the reason, instead of mis-routing them. Guidance only — it executes nothing.
- **`list_capabilities`** — the runtime capability matrix (Inkscape version, available Actions,
  export formats, live availability, fonts) **plus** the full `intents` goal→tool map (same data
  `how_do_i` matches against). Browse the whole map at once here.
- **`find_objects(doc_id, …)`** — resolve the **object ids** that the id-taking edit tools need
  (filter by fill/stroke/tag/text/id_prefix/bbox). `inspect_document` gives the same ids plus the
  full structure (tree, layers, styles, fonts, assets).

Use `how_do_i` and `list_capabilities` for representative asks and current tool routing.
Their intent catalog is embedded in the Rust server.

Rule of thumb: simple structural edits (create/style/text/transform) go through the Rust DOM
edit pipeline; render, export, and complex path geometry go through the Inkscape engine ([helper semantics](live/live-helper-kernels.md)).

---

## 2. The create → render → export loop

The generative flow (E14). All writes land on a **working copy**, never the original file.

1. **Create or open a document.**
   - `create_document(width, height, viewBox=None, background=None)` — a brand-new
     blank tracked document (no source file required). Returns a `doc_id` used by every other tool.
   - `open_document(path)` — open an existing workspace SVG as a working copy; also returns a `doc_id`.

2. **Draw / compose.** Add elements with the typed creation tools:
   `create_rect`, `create_circle`, `create_ellipse`, `create_line`, `create_polygon`,
   `create_polyline`, `create_path`, `create_text`; group with `create_group` / `group_objects`;
   reuse with `create_use`; add gradients with `add_linear_gradient` / `add_radial_gradient`.
   Each returns the new `object_id` (+ analytic bbox), so the next call can target it.

   **Author vectors; do not trace bitmaps.** The MCP instructions prohibit proposing or
   performing bitmap tracing and automatic raster-to-vector conversion, including Inkscape
   Trace Bitmap, external tracers (VTracer/Potrace), scripts, shell commands and preprocessing
   before importing SVG. Passing traced output through MCP does not make it acceptable.
   Use supplied PNGs and other bitmaps only as visual references; reconstruct them with
   deliberately authored, editable shapes, Bezier curves, fills and gradients. Do not replace
   vector artwork with an embedded bitmap. Render, compare and refine, and report remaining
   differences honestly rather than promise pixel-identical reproduction. This is agent
   guidance in MCP `instructions` and `compose_artwork`, not a new runtime tracing detector.

   **Make complete objects easy to select.** Unless the user requests another structure,
   use ordinary named groups for semantic objects (`cat-1`, `sofa-1`, `flower-1`, etc.),
   with stable unique ids and readable `inkscape:label` names. For a new illustration,
   prefer one general artwork layer containing these groups. Additional layers should
   organize the scene, rather than turning every object into a layer: Inkscape selects
   children of layers, and Select All can be scoped to the current layer.
   Preserve existing document organization unless the requested change requires otherwise.
   Regrouping must preserve paint order, transforms, clipping, masks and styles; do not
   merge paths, flatten structure or add masks just to arrange the object tree.
   Render and compare the result after structural changes.

   A layer and an ordinary group are both SVG `<g>` containers. Converting between them
   changes `inkscape:groupmode` (`layer` versus ordinary group mode); preserve the same
   id, children, styles, visibility, locks and parent position. It changes editor selection
   behaviour, not geometry. Layer highlight colours are editor UI metadata and do not
   change the artwork's fill colours. This guidance is delivered automatically in MCP
   `instructions` and reused by the `compose_artwork` prompt; it is a default, not a ban
   on user-requested layers.

3. **Style.** `set_fill`, `set_stroke`, `set_opacity` and `set_font` take `object_ids` (a list);
   `replace_text` takes one `object_id`. Use `find_objects` to resolve ids.
   For a gradient, use `set_fill(doc_id, [object_id], color="url(#gradient-id)")` after defining it.
   `replace_color` / `apply_palette` recolor document-wide.

   **Batch several edits in one call.** `apply_edits(doc_id, edits)` applies an ordered list (≤ 64) of
   **typed** edits — a discriminated union over the DOM ops (each tagged by an `op` field, e.g.
   `{"op": "create_rect", …}`, `{"op": "set_fill", …}`, `{"op": "move_object", …}`) — through the
   SAME edit kernel as ONE atomic, reversible operation: validate-all first (one bad edit leaves the
   document byte-identical), all-or-nothing rollback, and a single snapshot + Operation Record (one
   `restore_snapshot` reverts the whole batch). Effective risk = MAX over members (a `delete_object`
   member escalates the batch to HIGH and needs an `approval_token`). Prefer it over N round-trips
   when you already know the edits; path geometry and cross-document composition are not batchable.

4. **Render and look before you trust the edit (a loud default).** `render_preview(doc_id, …)`
   rasterizes the working copy and returns the PNG **inline** as an MCP image content block (when
   under the inline byte threshold) — so the agent can *look* at what it just made and decide the
   next edit, without writing a file. Set `inline=False` (or exceed the threshold) to get the
   artifact path instead. Make this the default close of every mutating step (especially an
   `apply_edits` batch, which changes several things at once): render, inspect, and `restore_snapshot`
   if it is wrong. In live mode the equivalent is `live_render_view`.

5. **Export the final artifact.** `export_document` writes a PNG/PDF/SVG/etc.; `export_object`
   exports a single object; `export_batch` / `create_icon_set` produce many sizes/formats at once;
   `export_web_profile` / `export_print_profile` apply a web/print preset; `svg_web_optimize`
   losslessly shrinks the SVG first. Save the SVG itself with `save_document_as`.

Every artifact-producing tool returns a root-relative `workspace_relative_path` (and a managed
`artifact_path`) — **never an absolute host path** (sec.12).

---

## 3. Working copy, snapshots, and reversibility

The server is safe to hand to an autonomous agent because every mutation is reversible and originals
are never touched:

- **Working copies.** `open_document` opens a copy; no tool overwrites the source. Persisting writes
  elsewhere via `save_document_as` (overwriting an existing file is itself HIGH-risk — see below).
- **Operation Records + snapshots (ADR-004).** Every *real* mutating op snapshots the working copy
  first and emits an **Operation Record** (what changed, the risk class, the policy decision,
  before/after preview). A genuine no-op (e.g. `set_fill` to the colour already present,
  `replace_color` matching nothing) reports `changed: false` and writes **no** snapshot or record —
  nothing happened, so nothing clutters the history.
- **Undo / restore.** `create_snapshot` checkpoints on demand; `list_snapshots` browses the history;
  `restore_snapshot(doc_id, snapshot_id)` rolls the working copy back to any indexed snapshot.
  `reload_document(doc_id)` re-reads the working copy from disk (e.g. after an external/live edit).
- **Retention.** Snapshot/artifact/live-frame growth is bounded by an **explicit** sweep (boot-time +
  the `prune_snapshots` tool) — never an implicit side effect of a mutating tool.

Practical loop: snapshot (or rely on the automatic pre-mutation snapshot) → edit → `render_preview`
to inspect → if wrong, `restore_snapshot` back and try again.

Headless `delete_object` refuses to remove IDs referenced by remaining objects, including
IDs inside a selected group. Remove references first or explicitly select the dependent
objects in the same deletion. Stylesheets require preparation before deletion. The same
guard applies to `apply_edits`; it does not extend the native live deletion protocol.

---

## 4. Risk classes and the approval-token gate

Every tool declares a risk class (in its docstring and in `llms.txt`):

| Class | What | Gate |
|---|---|---|
| **low** | read / inspect / validate / quality / render / export / discovery | always permitted |
| **medium** | write-new / element-creation / style / text / transform / web-optimize | permitted; reversible, snapshot-backed |
| **high** | overwrite an existing file · path geometry (`simplify_path`, `cleanup_paths`, `combine_paths`, `boolean_union`/`boolean_difference`, `stroke_to_path`, `break_apart`) · Action chains (`run_action_chain`) · raw Action (`run_raw_action`) | requires a per-operation **`approval_token`** |
| **restricted** | code / network / fs-escape | never ships in the MVP |

**The approval gate.** A HIGH-risk tool refuses (`high-risk operation requires explicit approval`)
unless it is called with a non-empty `approval_token`. **Approval is enforced by the client:**
the client must obtain explicit user confirmation for the particular operation before supplying
the string. The server checks non-emptiness only; it does not issue, authenticate, bind,
expire or consume tokens. This is a client-trust boundary, not an independent server-side
authorization mechanism. An agent must not invent approval strings or carry confirmation
over to another operation. Many HIGH-risk path tools also default to `dry_run=True`: call them
once to validate + preview the change with no mutation, then call again with `dry_run=False` **and**
the `approval_token` to apply it. Compose/adopt tools (`rust/src/adopt.rs`) are HIGH + approval-gated
for the same reason (they ingest arbitrary SVG).

So a typical HIGH-risk flow is: `how_do_i` → dry-run the tool to preview → have the client obtain
explicit user confirmation for this operation → re-call with the client-supplied `approval_token` →
`render_preview` to confirm → `restore_snapshot` if unhappy.

---

## 5. Resources and prompts

- **Resources** (read-only, addressable by URI): `inkscape://runtime/capabilities`,
  `inkscape://documents` (index of open docs), `inkscape://document/{doc_id}/{summary,tree,layers,
  objects,styles,fonts,assets}`, and the `inkscape://live/*` set. Read these for state instead of
  re-deriving it.
- **Prompts** (orientation only, grant no capability): `live_canvas_assist`, `prepare_web_export`,
  `prepare_icon_set`, `prepare_print_export`, `theme_recoloring`, and the E15-04 authoring pair
  `compose_artwork` / `restyle_artwork`. They tell the agent *how* to drive the relevant tools safely.

---

## 6. Out of scope

`how_do_i` returns an explicit reason for these rather than a tool: raster/photo **pixel** editing
(this is a vector/SVG server), arbitrary Inkscape Actions / extensions / scripts (ADR-003 — no
free-text escape hatch in the MVP surface), network/URL fetch (offline by policy, sec.12), and
arbitrary code execution (restricted). Provide local workspace files; use the typed tools.

## Workspace discovery and portable artifacts

Call `get_workspace_info` or read `inkscape://workspace` before choosing paths. Each configured
server root has an opaque `root_id` and a readable directory name; `relative_path_root_id`
identifies the existing first-root anchor. These are server roots, not client filesystem paths.
For an outside-workspace error, choose a relative destination such as `output/final.svg` under
that anchor. Absolute host paths remain private. Save and export results now include an
`artifact` with a root-qualified `inkscape://artifact/{root_key}/{token}` URI. Read that URI using
MCP resources, including when the server is remote. Existing relative path fields keep their
meaning. Resource reads enforce containment, no-follow access and the artifact `max_output_bytes` limit
on the opened file; SVG imports retain their separate `max_input_bytes` limit. Artifacts may
become unavailable after deletion/pruning. A resource URI is not a public HTTP download URL.

## Group/layer organization (headless working copies)

`create_group(..., object_id="cat", label="Cat", mode="group" | "layer")` creates named
containers. `rename_object(doc_id, "cat", label="Sleeping cat")` already changes only the label.
`set_group_mode` changes only `inkscape:groupmode` on the same `<g>` and treats matching mode as
a no-op. IDs, children, sibling position, style, visibility and locks remain intact.

For `reparent_object`, explicitly set `preserve_appearance=True`. This compensates affine
parent transforms at floating-point precision and requires unchanged global paint order.
Changed ancestors must be plain groups carrying only ID/transform/label/groupmode. Inherited
styles, effects, locks, CSS stylesheets/transforms, nested viewports, singular transforms and
external references to moved or changed ancestor containers cause refusal before disk mutation.
This conservative scope cannot safely reorganize arbitrary overlapping artwork. The default
False keeps the legacy XML-only behavior for compatibility; it promises no visual preservation.
The new creation fields and safe reparent flag also work in `apply_edits`. These additions apply
to tracked working copies; they do not extend the native live command protocol.

## Editability advice

`quality_report(doc_id, editability={...})` adds a separate structural report. Configure
`check_labels`, explicit `semantic_group_ids`, `layer_advisory_threshold`, `max_group_depth`
and `fragmentation_threshold`; `enabled=False` disables advice. Style/effect and stylesheet
risks are factual observations. Missing labels, depth, single-child wrappers and layer counts
are optional recommendations; they do not affect `ok`, validation counts or the quality score.
Only explicitly supplied semantic IDs produce semantic-layer advice. Counts and names cannot
establish object meaning, tracing provenance or artistic quality. Output advice is capped at 200.

## Focused previews and comparison

`render_preview(doc_id, object_id="paw", width_px=800)` delegates to existing `export_object`.
For a detail including nearby artwork, pass `region={"x":2,"y":7,"width":2,"height":2}` in
root SVG user units; `width_px` controls resolution and `background` controls the region PNG.
Root dimensions/viewBox mapping is accounted for; unsupported root CSS sizing/transforms
fail rather than guessing. Whole-document defaults remain intact.

`compare_region(doc_id, snapshot_id, region, width_px=800, background="white")` renders the
pre-edit snapshot and current working copy without restoring or mutating either. Use the
snapshot ID from an edit result. Both PNGs share explicit bounds, resolution and background;
results include resource URIs and inline before/after images where size permits. This is a
visual inspection aid, not an artistic score. Snapshot and artifact retention still apply.

## Replace a fragment with stable IDs

`replace_svg_fragment(doc_id, object_id="paw", svg='<g xmlns="http://www.w3.org/2000/svg">…</g>',
approval_token=...)` adopts one complete element through the existing safe parser/allowlist and
HIGH-risk gate. The selected root's qualified tag and ID must survive (omit its ID or repeat it).
The new fragment supplies all other root attributes and children; internal IDs survive only
when explicitly present. Duplicate/conflicting IDs and new dangling references are rejected.
External references to removed IDs always reject. Default `reference_policy="reject_changes"`
refuses a changed subtree that is referenced from outside; `"allow_retained"` explicitly accepts
that references to surviving IDs may show the new artwork. Stylesheets require preparation.
One successful change produces one snapshot and Operation Record; identical content is a no-op.
The rest of the scene remains structurally intact; serialization can normalize XML formatting.
The corresponding HIGH-risk `replace_svg_fragment` batch member uses the same checks.

## Declarative repetition

`repeat_objects(doc_id, "leaf", group_id="garland", placement={"kind":"polyline",
"points":[{"x":10,"y":20},{"x":90,"y":20}],"count":12})` validates a plan without writing
by default. Apply with `dry_run=False`. Two points define a line; additional points define an
explicit piecewise-linear path. Curved SVG path strings/IDs are not supported. A polyline takes
exactly one of `count` or `spacing`; count spans endpoints (count=1 starts at the first point),
spacing starts at distance zero and includes the endpoint only when the step reaches it.
Rectangle placement takes x/y/width/height and count/optional columns, or spacing_x/spacing_y.
It places grid cell centers inside the area; jitter can extend outside it and does not clip shapes.

Coordinates describe placement of `anchor` (default source local origin) in document user units.
`orientation="tangent"` follows polyline segments; rectangles use fixed orientation. Seeded
`variation` bounds offsets (0..1000 units), scale variation (0..0.5 around 1) and rotation variation
(0..180 degrees). The seed reproduces geometry, not opaque copy IDs. `mode="linked"` uses `<use>`
references to the source; `"copies"` creates independent trees with remapped IDs/internal refs,
reusing the duplicate engine. Source and inherited parent styling remain in place; the new named
ordinary group is inserted immediately after it. Copies continue to share any external defs.
Maximum 1024 instances; conservative projected and actual SVG size checks prevent oversized
expansion. Stylesheets, locks, animations and nested viewports require preparation. Dry-run
checks source/IDs/transforms and full expansion on a disposable tree, without snapshots or
Operation Records. Apply uses one shared edit transaction; `apply_edits` also supports it.
For a simple existing local grid use `tile`; `compose_grid` remains for cross-document layout.

For multiple roots, `open_document(path, root_id=...)` and
`save_document_as(doc_id, dest_path, root_id=...)` select the root returned by workspace discovery.
The explicit-root path must be relative without `..`; the sandbox still checks the resolved path.
Outside-workspace tool errors retain their stable prefix and now point to workspace discovery.
Fixed-scale comparison refuses snapshots whose canvas coordinate mapping differs from the current
canvas; choose a snapshot from before a local edit with the same canvas setup.

## Editable vector authoring quality

Initialization and `compose_artwork` share the policy from
`migration/contracts/authoring-guidance.txt`; discovery and prompt files are generated
snapshots, checked against that source. Follow the geometry and rendered-review guidance
as well as the vector-only, semantic-group and approval rules above.

For a flower, author a smooth continuous petal silhouette as one outline and keep the
center separate if it has different paint. A geometric union can remove internal overlaps
of construction primitives; merely combining paths does not. Keep genuinely independent
parts separate. For fabric folds, create separate `fill="none"` path objects for each fold
and put them in an ordinary named group. A snowball should have a rounded silhouette;
semantic shape choice requires inspecting the render and is not inferred by a tool.

Use explicit roles when asking for structural advice:

```json
{
  "doc_id": "YOUR_DOCUMENT_ID",
  "editability": {
    "object_roles": [
      {"object_id": "folds", "role": "independent_strokes"},
      {"object_id": "whisker-left", "role": "stroke_only"}
    ]
  }
}
```

Pass this to `quality_report`. Roles are optional, limited to 200 unique IDs of 1–256
UTF-8 bytes each. Missing or duplicate SVG IDs yield an unknown target finding; repeated
role IDs and malformed inputs are rejected. `independent_strokes` designates a `<g>` and
its scene geometry; definitions are excluded. The report never guesses roles from names,
open paths or primitive counts. A compound filled path without a stroke role is valid.

`editability.authoring.findings` identifies the object ID, code, certainty (`known` or
`unknown`) and reason. Advice does not affect validity or the existing quality score.
A stroke's effective fill must be `none`, even if its fill is transparent or covered.
The static cascade supports simple type, ID, class and universal selectors, presentation
attributes, inline declarations, importance, source order and inheritance. Complex or
external stylesheets, dynamic values, animation and unsupported syntax yield unknown
findings; they are never a passing check. The legacy inspector remains unchanged.

Review is bounded to 20,000 elements, 256 KiB stylesheet text, 2,048 rules, 128 ancestor
levels, two million CSS work units and 200 findings. Path parsing allows 256 KiB/10,000
segments per path and 4 MiB/200,000 segments across a report. Group-role traversal has
a 200,000-element visit budget. Limit exhaustion produces uncertainty/truncation, not
evidence of safe geometry. Setting `enabled=false` disables all editability findings.

Hidden/transparent scene findings account for ancestry, resource containers and local
references. Required `defs`, masks, clipping shapes and `use` source subtrees are preserved.
Unresolved external/encoded references make cleanup advice uncertain. General occlusion
is deferred: overlapping bounding boxes or a plausible render do not establish coverage.
Partial overlap is allowed. Review the SVG and render, then use existing gated deletion
or path operations for explicitly approved repairs within the requested scope. Compare
before/after renders and retain snapshots and Operation Records; this report never edits
or deletes anything.

## Computed live paint and reviewed packages

In the full profile, use `live_inspect_objects(object_ids=[...])` for bounded static computed
paint, ancestor effects and local resource links with explicit unknowns. Discover engine bounds
and preview separately. `live_change_package(edits=[...])` defaults to read-only planning and
page previews; apply identical style/transform/text edits with all returned document, selection,
content and package-digest guards plus explicit per-package approval. Only the guarded managed
native helper applies. Native package GUI Undo/Redo passed for a synthetic style/text pair (see the acceptance ledger); do not promise general live atomicity
or retry an uncertain result before inspection. See [supported scope, recovery and pilot protocol](live/reviewed-workflow.md).
