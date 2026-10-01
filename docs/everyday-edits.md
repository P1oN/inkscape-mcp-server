# Everyday edits in managed macOS Inkscape

Implemented 2026-10-01 for official Inkscape 1.4.3. This extends the experimental
[managed document context](document-context.md); the installation/launcher requirements stay
as described there. Save and close an old managed GUI before reconnecting to load the new
`org.inkscape-mcp.edit` effect and native allowlist. Restarting MCP alone preserves the existing
GUI and does not upgrade its extensions. No running user GUI is restarted automatically.

## Workflow and operations

Connect with `live_connect(prefer="no_freeze")`, list documents, select the task drawing with
`live_select_document`, then select objects in Inkscape. Inspect the selection/scene before
editing. Each write uses the existing live approval-token policy and records before/after
previews. The task/document guard applies to all reads and edits.

| Tool | Supported operation | Coordinates and preservation |
|---|---|---|
| `live_apply_to_selection` | Fill, stroke, stroke width, opacity; several properties and objects in one call | Existing unrelated styles remain. Locked objects, locked descendants and locked ancestor layers refuse. |
| `live_apply_to_selection` | Translate (`dx`, `dy`), uniform scale, rotate; can combine with style | SVG document user units, scale/rotation about document origin. Order: scale, rotate, translate. The parent's composed transform is accounted for; selecting a parent and child changes the subtree once. |
| `live_set_selected_text` | Exactly one single-line, single-run `<text>` object, optionally nested in `<tspan>` | Run attributes, position and formatting remain; input is text, never markup. Multiple runs, text paths and flowed text refuse. |
| `live_edit_selection(operation="duplicate")` | Copy the selected subtrees at their current positions, above their originals | New IDs for every identified descendant; internal href/paint/connector references remap. External references keep their existing targets. |
| `live_edit_selection(operation="delete")` | Delete selected subtrees | Refuses if a surviving element or stylesheet references their IDs. Deleting a reference together with its target is allowed. |
| `live_edit_selection(operation="group")` | Group consecutive siblings of one parent | Preserves transforms and stacking; does not reparent across layers. Returns the new group ID and member IDs. |
| `live_edit_selection(operation="ungroup")` | Ungroup plain groups | Composes each child's transform with the group transform. Groups with inherited style, effects or external references refuse. |
| `live_edit_selection(operation="raise" / "lower" / "front" / "back")` | Move one stacking position or to the front/back | Stays within the original parent; preserves selected objects' relative order. |

Structural edits can invalidate the GUI selection: select the returned objects again before
the next operation. Editing layer containers or definition nodes is unsupported. Structural
copy/group/ungroup with stylesheet rules refuses to avoid changing selector-dependent paint.
Transforms with CSS transform, nested SVG viewports or singular parent matrices also refuse.
These restrictions preserve the drawing; the server does not flatten its structure automatically.

## Undo and failure behavior

All everyday edits run through a fixed one-shot inkex effect. It checks document content and
captured selection, validates and prepares the complete change on a copy, then publishes it
through Inkscape's normal extension transaction. A changed call adds **one native Undo step**,
even when it edits several objects/properties. One Redo restores the entire result. A call that
leaves drawing content unchanged adds no Undo entry; Undo then affects the previous change.

The native dispatch guard rejects window/document switches. Validation refusal returns the
unchanged document and an allowlisted, path-free reason; it does not open an error dialog.
Timeouts, malformed replies and unconfirmed result fingerprints report **uncertain completion**:
inspect the chosen task drawing before retrying. An uncertain operation record is not evidence
of rollback. MCP client locks do not prevent a person from editing during an operation.

## Verification

`tests/test_managed_everyday_edits.py` uses installed vendor inkex for actual edit planning;
it skips when vendor inkex is unavailable. Tests cover transformed parents, parent/child
selection, locks, reference remapping/deletion, sibling order, group geometry and text formatting.
`tests/test_managed_edit_failures.py` covers reply/activation failures, cleanup and subsequent
successful calls. Existing policy/record tests cover the approval boundary.

Run `.venv/bin/python scripts/accept_document_context.py` on an unlocked Mac. It creates a
new private GUI with two synthetic drawings and checks all edit families through real MCP STDIO,
exact before/after content with Undo/Redo, unchanged calls, lock/text refusal, document races,
stale bindings and reconnect. Its directory retains `acceptance.json`, operation previews and
`stage3-*.svg`. Success closes only its verified synthetic windows; failures preserve the GUI.
