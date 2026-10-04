# Shared Rust SVG helper kernels

Python removal stage 3 provides `inkscape_mcp_rust::helper_svg` as a library, with
no filesystem, subprocess, GUI, arbitrary execution or MCP entry point. The server
uses its fingerprint and insertion preflight, and shares its affine arithmetic
with headless appearance-preserving reparenting. Both consumers use the same safe
`xml` parser and iterative element traversal. The installed Python extensions
remain active until stages 4–5; this stage does not change their publication or Undo.

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
wrapper text is discarded as in the current Python insertion helper; element mixed
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
claiming inkex equivalence. The current native helper retains its established
behavior. Plans are **not yet applied by the GUI**; stage 4 must implement application
on a disposable candidate and independently verify full SVG preservation, native
Undo, unchanged-result behavior and stale-state refusal before replacing inkex.
In particular, a planner unit test is not evidence of native publication equivalence.

`edit::guard` compares captured IDs/fingerprint and an independently supplied native
selection, refusing stale content, IDs and selection before planning.

A stage-4 consumer must compare document/window identity, exported fingerprint,
expected IDs and the actual native selection before planning and again as needed
before publication. It must retain root metadata, comments/PIs, namespaces, mixed
content and paint order; application/serialization limits also apply. Approval,
snapshots, Operation Records and rollback remain owned by the existing server
pipeline. This library cannot authorize a mutation or create a native Undo step.
