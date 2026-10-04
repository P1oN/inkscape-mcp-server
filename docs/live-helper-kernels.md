# Shared Rust SVG helper kernels

Python removal stage 3 provides `inkscape_mcp_rust::helper_svg` as a library, with
no filesystem, subprocess, GUI, arbitrary execution or MCP entry point. The server
uses its fingerprint and insertion preflight, and shares its affine arithmetic
with headless appearance-preserving reparenting. Both consumers use the same safe
`xml` parser and iterative element traversal. Stage 4 adds owned plan application
and the fixed `inkscape-mcp-inx` consumer. Stage 5 adds the fixed Rust socket consumer; no Python helper participates in live execution.

## Fingerprint wire representation (v1)

The wire value is lowercase SHA-256 hex over a UTF-8 JSON array. Records follow
preorder element traversal, including the SVG root. Each record is an array of:

1. Element depth, with the root at zero.
2. Expanded element name (`{namespace URI}local`, or `local` without a namespace).
3. Attribute pairs sorted lexicographically by expanded name and value.
4. Leading text, trimmed using Python-compatible Unicode whitespace.
5. Element tail text, trimmed in the same way.

JSON uses literal Unicode, ordinary JSON string escaping, and one space after
commas and colons outside strings, matching `json.dumps(..., ensure_ascii=False)`.
Namespace prefixes/declarations are not attributes. For the root alone, omit all
qualified attributes and unqualified `id` and `version`. Skip any element whose
local name, or an ancestor's local name, is `metadata` or `namedview`. Comments and
processing instructions do not produce records; their adjacent text remains
subject to the leading-text/tail rules. This is a stale drawing-content guard,
not a signature or a canonical rendering equivalence test. Whitespace normalization
can hide whitespace-only changes; callers must keep independent document/selection
identity guards.

The XML input respects the caller's byte cap and libxml2's ordinary depth limits
(no HUGE, entity substitution, DTD loading, network loading or recovery). The
fingerprint record stream has a cumulative budget of `min(8 * cap, 64 MiB)`;
namespace expansion cannot produce an unbounded digest workload. The frozen
`migration/contracts/effect-data-cases.json` values remain regression evidence,
without invoking a retired Python MCP server or generating paired parity results.

## Fragment preparation

`fragment::plan` validates the existing fixed live insertion allowlist and returns
ordered minted IDs. `fragment::prepare` repeats that validation and returns an
owned serialized ordinary `<g>` plus the same IDs. Neither modifies a destination.
The root ID is `mcp_` plus 32 lowercase hex digits; source IDs become `_0`, `_1`, …
in preorder. Internal href (including qualified XLink) and supported CSS URL
references are rewritten; duplicate IDs, missing references, external URLs, event
attributes, foreign elements, comments/PIs and malformed CSS URLs refuse.

The fragment limit remains 1 MiB, at most 10,000 elements including the synthetic
wrapper, with safe XML depth limits and at most 8 MiB prepared output. Leading
wrapper text is discarded as in the established insertion contract; element mixed
content and child tails survive. The serializer's namespace prefixes/formatting
need not match lxml byte-for-byte. Use expanded names and preserved content for
structural checks. This live allowlist is deliberately separate from headless
`adopt`/fragment replacement policy.

## Fixed live edit planning

`edit::Request` accepts only typed style, text, duplicate, delete, group, ungroup,
raise, lower, front and back operations. `edit::plan` returns owned semantic steps
and affected IDs; it never mutates the source or publishes a document. Empty steps
mean a genuine planned no-op. Input SVG and serialized request respect the caller's
byte cap; documents/selections are capped at 10,000 elements/IDs. Plans have a cumulative output budget of `min(8 * cap, 64 MiB)`, checked before
multiplying style values across targets and by a capped JSON byte counter. Request
and output JSON checks do not allocate a serialized copy before enforcing the cap. Reference scans
have a 64 MiB work budget, and affine transform strings are capped at 200,000 bytes.
Requests with unknown fields/operations, missing or duplicate selection IDs,
identity collisions, locked ancestors/descendants and definition/layer selections
refuse before a plan is returned. Parent+child selections collapse to their roots.

Live policy differs from the headless mutation pipeline:

- Styles compare explicit inline values without color normalization or presentation
  attribute removal. Style steps merge values into the existing inline declaration
  map; they preserve unrelated declarations and presentation attributes.
- Document-coordinate transform deltas are conjugated through the parent transform,
  using shared finite affine kernels; headless ordinary transforms use parent space.
  CSS transforms, nested viewports and singular/ill-conditioned parents refuse.
- Text accepts one simple single-run text/tspan tree and one line of replacement text.
  Text steps replace the sole leaf's leading text, preserving the surrounding tree.
- Duplicates use one deterministic cross-selection ID map, allowing internal links
  between clones. A consumer copies each subtree and its tail immediately after the
  source, rewrites mapped CSS URLs and href/connector-start/connector-end references,
  and preserves references to IDs outside the map.
- Deletion checks references to every removed descendant, including CSS selectors and
  connector attributes. Grouping requires consecutive siblings in one parent and
  creates an ordinary group. Ungrouping accepts only plain groups with ID, transform
  and Inkscape label, checks references, and compensates each child transform.
- Ordering retains selection order and parent boundaries. The order step lists the
  complete permutation of element children, including nonpaintable elements; its
  parent path uses element-child indices from the captured source. Nonpaintable
  elements are not stacking anchors. Consumers must preserve comments/PIs and tails.

The Rust preparation kernel deliberately refuses complex inline CSS in style edits,
stylesheets during transform planning, and minted descendant collisions rather than
claiming unrestricted inkex equivalence. Ungrouping also refuses descendant CSS
transforms and nested SVG viewports whose coordinate mapping cannot be preserved.
Stage 4 applies freshly computed plans on disposable DOM candidates. Native
acceptance remains a separate gate; a planner unit test is not native Undo evidence.

`edit::guard` compares captured IDs/fingerprint and an independently supplied native
selection, refusing stale content, IDs and selection before planning.

A stage-4 consumer must compare document/window identity, exported fingerprint,
expected IDs and the actual native selection before planning and again as needed
before publication. It must retain root metadata, comments/PIs, namespaces, mixed
content and paint order; application/serialization limits also apply. Approval,
snapshots, Operation Records and rollback remain owned by the existing server
pipeline. This library cannot authorize a mutation or create a native Undo step.

## One-shot INX application (stage 4)

`oneshot::Request` rejects unknown fields and cross-mode insertion/edit payloads.
The fixed executable accepts an absolute native SVG temporary-file argument,
repeated `--id=ID` and native `--selected-nodes=...` metadata only. It reads the
fixed `insert-request.json` under the managed session, capped at 2 MiB. SVG input
and output are capped at 16 MiB, preserving the existing 10,000-element/ID and
fragment bounds. No script, process, environment or extension execution is exposed.
The executable uses no-follow bounded regular-file reads and descriptor-anchored
atomic result publication, refusing linked/nonregular result files. The input is
never written. Publication failure emits no SVG; the server reconciles actual
native content instead of treating a helper acknowledgement as proof of application.

`oneshot::prepare` verifies the SVG root namespace, captured IDs/fingerprint and
native selection before applying. Insertion intentionally ignores selection:
Inkscape supplies the root ID when no drawable object is selected. Editing still
compares native `--id` arguments with the captured GUI selection. Window/document
UUID ownership is checked by the existing server/Objective-C scoped context action
before extension invocation; the helper's fingerprint is not an identity substitute.

`apply::edit` computes its own plan, retains the document root and top-level XML,
and mutates only the owned candidate. Style application retains unrelated raw
declarations; text is created as a text node so `&` and `<` remain literal content.
Structural edits preserve element text tails and non-element nodes, copy namespaces
and remap supported local references. Ungroup compensates child transforms;
order plans retain nonpaintable nodes and comment/PI anchors. Serialization may
normalize XML formatting; original files are never replaced. Output is reparsed
and bounded before publication. `apply::insert` appends one prepared ordinary group
in document coordinates, without inheriting the current layer transform.

For a no-op or refusal the helper emits **zero SVG bytes**. Inkscape's script effect
returns before document rebase when stdout is empty (see the upstream
[script implementation](https://inkscape.gitlab.io/inkscape/doxygen/script_8cpp_source.html)).
Successful changes return one complete SVG for the normal native extension Undo
transaction. Approval, snapshots, Operation Records, rollback and post-application
fingerprint/ID confirmation remain in the existing server pipeline. Native tests
must verify Undo/no-op/refusal separately on an owned synthetic drawing. Ready
packages contain `bin/inkscape-mcp-inx`; the supervisor installs a fixed quoted
wrapper pointing at that relocated binary. Python one-shot sources are retained
only as development/historical fixtures and omitted from the package helper assets.


## Socket snapshot bridge (stage 5)

`inkscape-mcp-live` accepts only native SVG input, repeated `--id` and selected-node
metadata. It retains protocol v5 and its 13 capabilities, binds only loopback,
uses a fresh UUID token, and serves one client until disconnect or a 120-second
accept/frame deadline. Frames (including newline) are capped at 64 MiB; coalesced
frames remain buffered. SVG input/candidates are capped at 16 MiB and 10,000 elements,
with unique IDs and a validated captured selection. The native blank-root fallback
is interpreted as no object selection. No client can choose an executable, filename,
extension, shell text or environment. Startup/reconnect do not invoke this helper.

The rendezvous is atomically published with mode 0600 through the existing no-follow
filesystem primitive. An exclusive adjacent lock serializes advertisers; lock losers
leave the active rendezvous intact. Symlink/nonregular destinations and linked locks
refuse. Cleanup removes only a matching advertisement, while holding that lock.
Original SVG input is never written. Metadata/scene names and explicit `has_style`
retain the socket model; `visible_objects` remains the historical object summary,
including definition descendants, rather than a computed CSS visibility promise.
Canvas retains the leading-number/viewBox-fallback socket representation. Viewport
values are null and viewport requests explicitly report `applied:false`.

This is still a **modal document snapshot**, captured at extension invocation. It
cannot observe subsequent GUI edits or selection/viewport changes. A revision hashes
the owned snapshot bytes. Render/selection export use fixed CLI arguments, bounded
owned subprocesses and private temporary SVG/PNG files. Page size, scale, region,
selection bounds, a 16,384 dimension limit, 64-million-pixel budget and 32 MiB PNG
limit guard raster growth. An export retains the original socket region/scale flags;
it does not introduce a new coordinate mapping contract. Self-contained local paint,
inline styles, simple stylesheets and PNG/JPEG data references are supported. External
assets, active content, DTDs/PIs, CSS escapes/comments/imports and stylesheet canvas
sizing require preparation before CLI execution. Native vendor resources are retained.

Selection geometric boxes use one bounded CLI query over isolated, unpainted shape
roots: each object's own transform applies, ancestor transforms and stroke/filter/
clip/mask/marker paint do not inflate its box. Rectangles, circles, ellipses, paths,
lines, polygons, polylines and groups of these are supported. Text, use/reference,
nested viewport, percentage and CSS geometry return null rather than an invented box.
Duplicated query geometry has a cumulative 10,000-element / 16 MiB budget; roots that
exceed it return null. These conservative limits replace inkex's best-effort boxes.

Socket style/text/insertion use the same bounded candidate kernels as INX. Their
conservative reference/style/transform/text restrictions above also apply here;
insertions create a remapped ordinary group. The server still owns approvals,
snapshots, Operation Records and confirmation. Socket acknowledgements describe the
extension's candidate, which Inkscape adopts only when the modal extension exits;
they do not establish native application or a separate Undo step per socket call.
All accumulated changes return one full SVG on disconnect/timeout; unchanged sessions
emit zero bytes. Publication preserves the full XML candidate. One-shot managed edits
continue through `inkscape-mcp-inx`. Ready packages omit CPython, helper wheels and their runtime notices; Python remains build/development tooling until stage 7.
