---
name: inkscape-mcp
description: Create, inspect, refine and export editable SVG artwork through a configured Inkscape MCP server, including edits to an open Inkscape drawing. Use for vector illustration and SVG editing with this server.
---

# Inkscape MCP

Use the configured Inkscape MCP tools to deliver editable vector artwork and inspect
rendered results. The server's initialization instructions and authoring prompts are
the maintained source of its vector-authoring and group/layer guidance; follow them.
In particular, use bitmaps as visual references, deliberately author vector geometry,
and organize semantic objects with named ordinary groups by default.

## Establish the target

- Discover the current tools and schemas rather than assuming a fixed tool count or
  client-specific prefix. `how_do_i` routes a goal to tools; `list_capabilities` reports
  the available engine and live capabilities. If the MCP is unavailable, report that
  connection problem before claiming to have edited a drawing.
- Call `get_workspace_info` or read `inkscape://workspace` before choosing file paths.
  Use the returned root IDs and relative paths. A server workspace may be remote;
  do not substitute client filesystem paths for server paths.
- For a new drawing use `create_document`; for an existing SVG use `open_document`
  and inspect its working copy. Keep the returned `doc_id` and object IDs for later
  calls. Resolve existing targets with `inspect_document` or `find_objects`.
- For a request concerning the open Inkscape canvas, use the live tools. Startup and
  reconnect do not launch Inkscape. `live_launch` requires the user's request to open
  it. Connect with `prefer="no_freeze"`, list/select the task drawing and check
  `live_status.ready_to_edit` before mutation. Reconnect resets the drawing binding;
  select it again. Do not close unrelated windows or mix working-copy IDs with live IDs.

## Edit, inspect, refine

Use typed tools and the current schemas. The `compose_artwork` and `restyle_artwork`
prompts help with authoring; `live_canvas_assist` helps with live operation.
Batch known compatible edits with `apply_edits` to obtain one atomic edit and snapshot.
Use stable IDs and keep existing references, styles and document organization intact.

For reorganizing working-copy artwork, use `reparent_object` with
`preserve_appearance=True`. If it refuses, inspect the reason and choose a supported
change; do not retry with appearance preservation disabled just to force the move.
Live operations have their own limits; headless support does not imply live support.

Render and inspect each meaningful edit or batch with `render_preview`, or
`live_render_view` for the open drawing. Use an object/region preview when details are
hard to see. `compare_region` can compare a working-copy edit with its pre-edit snapshot
at matching canvas coordinates. Refine based on the image, not just a success response.
Use `restore_snapshot` for a rejected working-copy edit. A no-op has no new snapshot;
read the actual result rather than assuming one exists.

HIGH-risk tools require explicit user confirmation through the client's approval flow
before the client supplies `approval_token`. The server checks non-emptiness only and
does not authenticate tokens or prevent reuse; the client owns this approval boundary.
Never invent a string or reuse confirmation for a different operation. Use dry-run when supported to make the change
reviewable before requesting the required approval. On refusal, stop that operation
and explain what is needed; do not bypass the gate with raw actions, shell or extensions.

## Deliver the result

Save the editable SVG with `save_document_as` and export the requested formats with
the relevant export tools. Opening a file creates a working copy; editing alone does
not update the user's original SVG. Avoid overwriting an existing destination without
the server's required approval.

Report the saved/exported artifacts from tool results. When available, read their
`inkscape://artifact/...` resource URIs through MCP; these also work with a remote
server and are not public download URLs. Describe remaining visual differences and
which checks were performed. A successful headless preview does not establish native
GUI selection, Undo/Redo or clean-machine compatibility.
