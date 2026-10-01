"""In-context system overview delivered as MCP server ``instructions``.

The Penpot survey's real onboarding edge is that its ``high_level_overview`` is delivered as MCP
server **instructions** — so the model holds the document model + idioms every turn, not only if it
happens to open a ``docs/`` file. Our equivalent prose already exists in
``docs/agent-usage-guide.md``, but that sits OUTSIDE the agent's context. This module
delivers a CONCISE orientation in-context via the FastMCP ``instructions`` field.

It is deliberately a SHORT orientation + pointers, NOT a second copy of the guide: it states the
load-bearing invariants (document model + ``doc_id`` lifecycle, the working-copy + snapshot/restore
reversibility idiom, the risk classes + ``approval_token`` gate, the intended tool ordering, and the
render-and-look default) and then routes to the authoritative, generated discovery surface
(``how_do_i`` / ``list_capabilities`` / ``llms.txt``) and the full guide for everything else. The
deep detail stays single-sourced in the guide + the generated manifest; this never restates it.

Audit: before this module the server was constructed as ``FastMCP("inkscape-mcp")`` with NO
``instructions`` field — wired tool ANNOTATIONS, TAGS, progressive disclosure, and the
``inkscape://prompts`` index, but no always-in-context overview. This closes that gap.
"""

from __future__ import annotations

#: Author original vector geometry even when a bitmap is supplied as a visual reference.
VECTOR_AUTHORING_GUIDANCE = """\
Vector authoring. Do not propose or perform bitmap tracing or automatic raster-to-vector
conversion when working through this MCP. This applies to Inkscape Trace Bitmap, external
tracers (such as VTracer or Potrace), shell commands, scripts and preprocessing before SVG
adoption; importing their traced output through MCP does not satisfy this rule. Treat a PNG
or other bitmap as a visual reference only. Reconstruct the artwork with deliberately
authored, editable shapes, Bezier curves, fills and gradients, organized into semantic groups.
Do not substitute an embedded bitmap for vector artwork. Render, compare with the reference
and refine; be honest about remaining differences rather than promise pixel-identical results.
"""

#: Shared by the always-delivered overview and the opt-in authoring prompt.
ARTWORK_STRUCTURE_GUIDANCE = """\
Editable artwork structure. Unless the user requests otherwise, make each semantic object
(cat, sofa, tree, flower, cloud, etc.) an ordinary named SVG group, selectable as a whole.
Use stable unique ids and readable inkscape:label names. For new illustrations, prefer one
general artwork layer containing these groups; reserve additional layers for genuine scene
organization, not one layer per object. Inkscape layers expose their children to selection;
Select All may be scoped to the current layer. Do not silently convert existing layers or
flatten groups, merge paths, or introduce masks merely to organize objects. Preserve paint
order, transforms, clipping, masks and styles when regrouping; render and compare afterwards.
A group/layer conversion changes inkscape:groupmode on the same <g>, not its geometry;
preserve its id, children, styles, visibility, locks and position in the parent.
Layer highlight colours are editor UI metadata, not artwork fill colours.
"""

#: The concise, always-in-context overview handed to FastMCP as ``instructions``. Kept short on
#: purpose — it orients, then points at the generated discovery surface for specifics.
SYSTEM_OVERVIEW = f"""\
inkscape-mcp makes Inkscape/SVG documents agent-ready through SMALL TYPED TOOLS (not a free-text
run_action / execute_code portmanteau). Orientation:

Document model & lifecycle. Work is keyed by a `doc_id`: `create_document` (new blank) or
`open_document` (existing workspace SVG) returns one; every other tool takes it. Typical flow:
open/create -> inspect (`inspect_document` / `find_objects` for object ids) -> edit (typed DOM
tools) -> `render_preview` to look -> `export_document` / `save_document_as`. All writes land on a
WORKING COPY; the original source file is never modified.

Workspace and artifacts. Call `get_workspace_info` or read `inkscape://workspace` before choosing
paths. Existing relative paths use the first server root, never client CWD; open/save accept
an explicit root_id. Read returned artifact resource URIs through MCP; server paths are not
client-local paths. Use render_preview(object_id/region) for details and compare_region with a
pre-edit snapshot for fixed-area comparison.

Reversibility. Every real mutation auto-snapshots first and emits an Operation Record (ADR-004). A
genuine no-op writes nothing and reports `changed: false`. Undo with
`restore_snapshot(doc_id, snapshot_id)`; `create_snapshot` checkpoints on demand; `list_snapshots`
browses; `reload_document` re-reads the working copy from disk.

Risk classes & approval. Each tool declares a risk class: low (read/inspect/render/export),
medium (create/style/text/transform — reversible, snapshot-backed), high (overwrite/delete/path
geometry/Action chains — requires a per-operation `approval_token`, minted out of band), restricted
(never ships). A high-risk tool refuses without a non-empty `approval_token`.

Batching. `apply_edits` applies an ordered list of typed edits as ONE atomic, reversible operation
(validate-all first, all-or-nothing, one snapshot) — use it to make several edits in a single call
instead of N round-trips. Its effective risk is the max over its members.

{ARTWORK_STRUCTURE_GUIDANCE}

{VECTOR_AUTHORING_GUIDANCE}

Render and look before you trust an edit. After a mutating call (especially a batch), render and
INSPECT the result — `render_preview` (headless) or `live_render_view` (live mode) — before relying
on it; `restore_snapshot` reverts if it is wrong.

Managed macOS live documents. MCP startup and live_connect never open a window. Use live_launch
only when the user asks to open Inkscape; otherwise attach to a running session.
After `live_connect`, use `live_list_documents` and
`live_select_document(window_id, document_id)` to bind the task drawing. Check
`live_status.ready_to_edit`: writes refuse after a window switch or document replacement.
Reconnect preserves the GUI and resets the binding. Inspect a drawing after an edit timeout
before retrying: the edit may have applied.

Finding the right tool. Don't grep the surface: call `how_do_i(goal)` (natural-language goal ->
tool names + how-to, and it flags out-of-scope goals), or `list_capabilities` for the runtime matrix
plus the full intent map. The generated `llms.txt` / `llms-full.txt` manifest and
`docs/agent-usage-guide.md` carry the full per-tool detail.
"""

__all__ = ["ARTWORK_STRUCTURE_GUIDANCE", "SYSTEM_OVERVIEW", "VECTOR_AUTHORING_GUIDANCE"]
