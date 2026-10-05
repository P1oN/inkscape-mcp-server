> Historical guide snapshot, archived 2026-10-05. Commands and acceptance
> claims belong to the recorded implementations; use the [documentation index](../../README.md).

# Live discovery and previews (roadmap stage 4, first slice)

Implemented 2026-10-01. Use this to locate candidates in an existing drawing before an edit.
The existing `live_get_scene` retains its inexpensive, attribute-derived geometry contract;
`live_find_objects` explicitly asks the Inkscape engine for accurate bounds.

## Workflow

1. Connect to an already running session. On managed macOS list and select the task drawing.
2. Call `live_find_objects(label="tree", tag="g")` or search by `layer`, `text`, `id_prefix`.
   Supplied filters combine with AND. Labels/text match case-insensitive substrings; a layer
   filter also accepts an exact layer id. Omitted filters list visible addressable objects.
3. Inspect `total_matches` and `truncated`. `limit` defaults to 100 and is capped at 1000.
   Multiple candidates require visual inspection or clarification; names alone do not prove identity.
4. Call `live_preview_object(object_id=..., expected_fingerprint=...)` using discovery's
   fingerprint. This renders the candidate in isolation from a fresh SVG snapshot. A changed
   drawing refuses with an instruction to find again. It does not change the GUI selection.
5. To see surroundings on managed macOS, pass the object's `bbox` into `live_render_view`'s
   `region_x/y/width/height`. This region is a fresh view without a fingerprint guard;
   rediscover after a drawing change before relying on old coordinates.
6. Select the confirmed object and use the existing undoable edit tools. Discovery does not
   automatically select or edit a candidate, and its fingerprint does not authorize a write.

The result includes active-document identity, nearest layer id/label, text, inherited lock
status, explicit paint and a bbox. Bounds use **document user units**, the same coordinates as
managed everyday transforms. They come from one batched `--query-all` on an exported snapshot,
with the root pixel/viewBox mapping inverted. Root origin, absolute physical units,
`preserveAspectRatio` alignment, meet/slice and nonuniform `none` scaling are accounted for.
No DOM box is substituted when the engine has no bounds for an object.

Inkscape's [command-line documentation](https://inkscape.org/doc/inkscape-man.html) specifies
pixel coordinates for export areas and describes `--query-all` and isolated `--export-id-only`.
Managed region export now maps document coordinates to those output pixels; it previously
passed user coordinates directly, producing an incorrect crop when the viewBox was scaled.

## Limits

- Requires Inkscape CLI on PATH in addition to a connected live transport. Reads remain bounded
  by input-size, element-count, subprocess-timeout and preview output/dimension limits.
- A root with only a viewBox uses Inkscape's standalone default: unit scale with viewBox-origin
  translation. Root CSS sizing, percentage dimensions, only one missing dimension, and unsupported
  aspect syntax refuse rather than guessing the mapping. Nested SVG geometry is delegated to the renderer.
- Stylesheets, scripts, foreignObject, active attributes, external assets and external paint
  references refuse, including Inkscape's image fallback paths (`sodipodi:absref`). Embedded
  PNG/JPEG images, internal gradients, filters and clipping are supported in the snapshot. The exported snapshot never enables network fetches by design.
- Paint is authored paint, not full computed CSS. The existing visibility approximation applies
  to inline/presentation attributes. Fully transparent, clipped-away or off-page items may have
  engine bounds without being perceptually visible; confirm the preview.
- Isolated export rejects ids containing commas, semicolons, whitespace or controls. Empty bounds
  cannot be previewed. Query/render failure is explicit; it never returns guessed geometry.
- Snapshot reads use the transport's operation scope. Human edits can still occur; returned
  geometry describes the captured SVG rather than promising a lock on future edits.
- Managed SVG reads change persistent GUI export options, as existing live inspection does.
  Selection, drawing content and Undo history are preserved by discovery/isolated previews.
- Object previews are ephemeral `live-view-object-*.png` frames. The existing startup/explicit
  retention pass applies `live_frame_keep_days` and `live_frame_max_bytes`, keeping newest
  unreferenced frames within budget and protecting frames referenced by operation records.
  Returned paths can expire after maintenance; render again if a preview is removed.

## Real illustration pilot

On 2026-10-01, a separate private Inkscape 1.4.3 GUI was launched with a copy of the supplied
`background.svg`. The 3.8 MiB drawing contains 430 elements, 178 paths, 16 groups, gradients,
filters, one clipping path and an embedded image. Its 4774×3600 pixel page has a
1263.1208×952.50001 viewBox and a transformed layer.

- All 178 paths received engine bounds in document coordinates. Labeled groups made searching
  for the three trees and flower groups practical; individual paths often lack useful labels.
- Three requested pilot operations passed through actual MCP STDIO: move `tree_one` by 20 user
  units horizontally, set `bush_flower` group opacity to 0.75, duplicate `mini_flowers_3_yellow`.
- Each edit changed the copy, one Undo restored its exact drawing-content fingerprint, one Redo
  restored the edited fingerprint, and a final Undo restored the baseline.
- Isolated preview/search preserved selection and content. Stale preview requests refused.
  Region preview dimensions matched the engine bbox after pixel/viewBox conversion.
- The source SHA-256 stayed unchanged. Only the checked private test window was closed after
  success; pre-existing sessions were not adopted or terminated.

Reproduce with a source and a JSON cases file on macOS:

```sh
python scripts/history/python/accept_live_discovery.py --source /path/to/drawing.svg --cases /path/to/cases.json
```

Cases are a list of objects such as
`{"label":"tree_one","tool":"live_apply_to_selection","params":{"dx":20,"dy":0}}`.
Labels must resolve to one candidate. Allowed case tools are `live_apply_to_selection` and
`live_edit_selection`. The script creates a copy and prints its evidence directory, retaining
`acceptance.json`, discovery output, SVGs after each edit and PNG previews. Failure preserves GUI.
Use `--keep-gui` to leave the disposable baseline open after successful acceptance.

This pilot validates these operations on one illustration. Mask/pattern/clone coverage and
computed styles/reference analysis remain subsequent stage-4 work; stage 4 is not complete.
