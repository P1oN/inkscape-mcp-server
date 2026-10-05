# Computed live styles and reviewed changes

The full tool profile adds `live_inspect_objects` and `live_change_package`. Both read a
fresh exported SVG inside the transport's document operation scope. Connect to an existing
session and choose the task drawing on managed macOS first. Neither tool launches a GUI.

## Computed styles and resource links

`live_inspect_objects(object_ids=["petals", "panel", "leaf-clone"])` accepts 1–100 unique
IDs, each 1–256 UTF-8 bytes, in a snapshot bounded by `max_input_bytes` and 10,000 elements.
It returns a content fingerprint and one entry per requested ID, including missing IDs.
It leaves content, selection, references and Undo intact and does not invoke a renderer.

Each property has `certainty`, `value` and, for unknowns, `reason`. The static cascade
reuses the bounded CSS analysis engine: simple type/ID/class/universal selectors,
importance, source order, presentation/inline attributes, inheritance and defaults.
The computed properties include fill, stroke, color/currentColor, stroke width, opacity,
fill/stroke opacity, display, visibility and local mask/clip/filter/marker references.
Opacity is clamped to the computed 0–1 range. Stroke widths requiring font, percentage or
physical-unit resolution are unknown. Unsupported selectors, at-rules, syntax, dynamic
values, animation and external stylesheets produce unknowns; external assets are never fetched.
Limits include 256 KiB stylesheet text, 2,048 rules, 128 ancestor levels and a shared two
million CSS work units per request. This is static SVG analysis, not Inkscape's full CSS engine.

`ancestor_effects` distinguishes ancestor compositing (opacity, display, mask, clipping,
filters) from inherited paint. `relationships` identifies source ID/property, target ID/tag
and whether the target exists. It follows local resource `href` chains for gradients,
patterns and clones, retaining missing references and repeated/cyclic-link uncertainty.
Expansion stops at 128 links per object with `relationships_truncated=true`. Resource
existence does not establish correct resource type, coverage, resolved server paint or visibility.
Nested links inside resource geometry are not expanded; inspect their IDs separately.

For `<use>`, properties describe the use element, not its shadow tree. Source paint,
instance overrides, symbol viewports and rendered visibility need preview review.
Use `live_find_objects` for engine bounds and `live_preview_object` for an isolated render
at that discovery fingerprint. Those renderer-backed tools retain their stricter
self-contained-snapshot gate: stylesheets, external assets and active content require
preparation. Computed-style inspection can report uncertainty on drawings that cannot
safely pass that render gate. Neither tool changes those existing limits.

## Review a bounded package

```json
{
  "edits": [
    {"op": "style", "fill": "#d95d72", "opacity": 0.8},
    {"op": "text", "text": "Reviewed resource study"}
  ],
  "dry_run": true
}
```

Pass this to `live_change_package` with the intended text selected. A package has 1–16
ordered edits and a fixed current selection of at most 1,000 objects. Style edits accept
only the existing fill/stroke/stroke_width/opacity/dx/dy/scale/rotate parameters; text edits
retain the existing single-line, single-run restrictions. Text requires exactly one text
object. No selection changes, structural edits, insertion or arbitrary Actions are included.
Existing lock, transform, parser, ID/reference and output bounds apply to every member.

Planning runs the same native SVG kernels on disposable candidates. Every member must
succeed before any publication. It returns page PNG `preview_before`/`preview_after`,
`affected_ids`, `changed`, `expected_document`, `expected_selection`,
`expected_fingerprint` and `expected_package_digest`. `preview_artifacts` includes portable
root-qualified resource URIs; read those through MCP when the server is remote. No GUI mutation, live record or
Undo step occurs. Review the PNGs, obtain the client's explicit approval for this particular
package, then resubmit the identical edits with those four guards, `dry_run=false` and
`approval_token`. The digest binds the canonical edits to review; it is not an approval token
or authenticated server consent. Guards are freshness checks, not durable leases.

Applying requires the guarded managed macOS backend and the updated native INX helper.
Plain D-Bus and snapshot sockets refuse before creating a record or dispatching a write.
An older running helper may safely refuse the new payload; do not silently fall back to
several single-edit calls. Rebuild/reconnect the selected server to load the new tool
schemas; loading a new GUI helper requires a separately authorized owned/new GUI session.
Never close or replace a user's current windows to upgrade it.

The managed backend checks document lifetime/context, fingerprint and exact selection
again inside its cross-client operation lock. The native helper checks fingerprint,
complete ID inventory and native selection on the actual input before recomputing the
whole candidate. It emits one candidate document only after success, using the existing
one-shot publication path. A net DOM no-op emits no bytes and the public tool creates no
record. Restoring an effective property while leaving a new inline declaration is a DOM
change, not a no-op. A changed call uses one existing Live Operation Record with previews.

Failures before publication preserve the drawing. Lost activation/reply or audit persistence
failure after dispatch is uncertain: inspect the drawing and operation history before any
retry. There is no automatic retry or compensating sequence that could overwrite human edits.
Use native Undo for a confirmed GUI change and `live_sync_to_workspace` when a tracked
working copy/snapshot is wanted. Headless `apply_edits` remains independently atomic.

**Scoped native GUI acceptance passed on 2026-10-05:** one two-member style/text package
on a synthetic text object, a repeat no-op, and one native Undo/Redo restored matching SVG
and PNG pairs on the recorded candidate. See the [acceptance ledger](../history/reports/live-drawing-workflow.md).
This does not establish arbitrary package atomicity or intervening human-review races.
Per-call results retain `native_undo_verified=false`; package records do not assert
`undo_friendly`, since a particular request/environment has no automatic verification.

## Artist pilot and installation usability

The user deferred the pilot on 2026-10-05 until copies of real work are supplied.
No separate-Mac, artist-observation or terminal-free installation result is claimed.
Use this protocol when that evidence is available:

1. Record the separate Mac, OS, Inkscape/client versions, build identity and installation state.
   Work from copies and record original hashes; preserve other windows and unsaved work.
2. Observe installation/update using the current supported setup and record every terminal
   command, permission prompt, confusing choice, failure and recovery. Do not install a
   candidate over an owned runtime without authorization.
3. On each artwork locate a semantic object, inspect computed paint/resources, confirm engine
   bounds and preview, select it manually, plan a small package, review/apply, then inspect.
   Include masks, linked patterns, clones and unsupported CSS, recording uncertainty/refusals.
4. Independently capture before/after/Undo/Redo SVGs and PNGs in the owned session; test a net
   no-op and a stale plan after a human edit/selection/window change. Record actual outcomes.
5. Reconnect, sync a copy, recover, and compare originals' hashes. Record task time, wrong-object
   selections, preview/refinement difficulty, lost context and recovery friction.
6. Derive scoped fixes from observed problems. For a terminal-free setup/update proposal,
   evaluate an app entry point, workspace picker, Inkscape/client detection, build progress,
   client registration, update conflicts and recovery against the observed issues. The current
   CLI implementation alone is not evidence that such a UI works for artists.

The synthetic acceptance fixture deliberately authors editable gradients, masked/clipped
geometry, linked patterns and a clone. It establishes renderer/identity/reference behavior
within its supported scope, not acceptance of arbitrary real artwork or artist usability.

After explicit launch authorization, use `native-gui --phase setup` with the candidate
package, then `native-inx --phase insert-text` and select that synthetic text in its recorded
private app. `native-inx --phase package` plans/previews/applies style plus text;
`--phase package-noop` verifies unchanged SVG and operation history. Capture independent
Undo/Redo using fresh labels via `--phase capture`; compare with the package before/after
captures, including a single Undo after the no-op. Restore the owned blank before
`native-gui --phase close-owned`. The ledger records the executed scope and retained evidence; new runs need their own context/build evidence.
