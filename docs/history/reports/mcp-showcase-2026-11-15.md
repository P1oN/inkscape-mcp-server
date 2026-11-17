# Five-task Inkscape MCP sprint — 2026-11-15

## Outcome

Five repository communication assets were deliberately authored through the configured
Inkscape MCP, rendered, visually inspected and exported. A sixth reusable asset supports
the feature illustration and social card. The README now introduces the product,
shows editable output, explains installation and links to the existing detailed docs.
All 12 delivered SVG/PNG files match their server-side SHA-256 and byte counts.

The implementation is isolated on `codex/inkscape-mcp-showcase`, based on `origin/main`
`c029f02`. The original dirty checkout and its unfinished work were left in place.
The configured runtime was used for artwork; a separately built candidate was used
to validate the code fix. No installed runtime, client profile or Inkscape GUI was changed.

## Five real tasks

| Task | Tools exercised | Reviewed result |
| --- | --- | --- |
| README cover | create_document, apply_edits, create_path guards, gradients, text/fonts, named groups, set_fill | [1440 × 720 cover](../../media/github-hero.png), with three smooth closed petals, editable handles and readable type |
| Architecture overview | typed cards/text/paths, named arrow groups, quality_report | [1440 × 600 diagram](../../media/workflow.png); shafts and arrowheads are separate stroke objects |
| Editable-output illustration | whole-document placement, deep-copy ID/reference remapping, find_objects, stroke/closure roles | [1440 × 760 feature illustration](../../media/editable-vectors.png); two gradients and the grouped bloom survive composition |
| First-install overview | atomic batch with 55 ordered edits, typography and semantic cards | [1440 × 660 quickstart](../../media/quickstart.png); requirements and four installation steps fit their cards |
| Share card | reuse at a different scale, dark composition, text, export | [1280 × 640 social card](../../media/social-preview.png); complete margins, visible bloom and repository address |

The standalone [Bezier bloom](../../media/bezier-bloom.svg) is a 640 × 480 transparent
SVG master. It is illustrative artwork, not a new Inkscape or Manager application icon.
The social card is a candidate asset; GitHub's social-preview setting is separate.

Authoring followed create → typed edits → render/inspect → refine → quality →
vector-only save → PNG export. No bitmap tracing, raw SVG adoption, embedded artwork,
arbitrary execution tool, approval token or high-risk file overwrite was used.

## Problems found and repaired

### Newly created lines violated the stroke-role contract

`create_line` omitted fill, including when the caller used a filled parent. The line
rendered as a stroke, but `quality_report` reported `stroke_fill` for the designated
handles. Explicit `set_fill(..., "none")` repaired the artwork through MCP.

The shared primitive-creation kernel now writes `fill:none` for every new line,
covering both the dedicated tool and `apply_edits`. A regression checks inherited
fill and the actual authoring analysis. Real STDIO in both `per_call` and `shell`
modes additionally verifies dedicated/batch creation, a genuine subsequent no-op,
vector-only saving, real PNG dimensions, source preservation and byte-exact rollback.
No tool schema or initialization instruction changed, so manifests need no regeneration.

### Diagram strokes needed better editable structure

The first arrows used two subpaths per object. Explicit stroke roles correctly
reported `combined_strokes`. A fresh revision was created with a shaft and a
continuous arrowhead path inside each named arrow group. The original draft remains
available in the task workspace; the reviewed final has no such findings.

### GitHub installation entry was stale

The previous README linked directly to v0.1.7 although GitHub already listed v0.1.11
as a prerelease. The primary entry now points to Releases, with the DMG filename and
prerelease status made explicit. Existing detailed installation, update, monitoring
and validation documentation remains linked rather than copied into the landing page.

### A client resource bridge truncated large binary blobs

Reading a PNG resource through the generic client bridge produced a base64 string
containing a truncation marker. This is not evidence of a corrupt server export.
The initially transferred copies were rejected and replaced using the MCP export's
native image block. Every final PNG and SVG was then matched against `stat_artifact`.
Do not treat a base64-looking text response or a PNG signature alone as a complete file.
The bridge itself is outside this repository and was not modified.

### Safe placement refusal was retained

Placing only the bloom group from the cover refused because its gradients lived
outside that subtree. This was expected preservation behavior. A self-contained
standalone document was authored and placed instead; no guard was relaxed.

## Validation

- `cargo test --locked --manifest-path rust/Cargo.toml -- --test-threads=1` passed;
  opt-in native GUI/render tests retained their documented ignored status.
- Main Cargo formatting and all-target Clippy with `-D warnings` passed.
- Real-STDIO authoring acceptance passed in `per_call` and `shell` engine modes.
- Security, frame, startup, responsiveness, defects, diagnostic, comparison,
  special-file, renderer and engine-routes acceptance all passed against the candidate.
- A dedicated two-mode STDIO line regression passed creation, quality, no-op,
  rendering, original-file and rollback checks.
- All six SVGs passed structural validation, explicit geometry/stroke checks and
  the vector-only save gate. All local SVGs parse and all 12 file hashes/sizes match
  the server originals; details are in [provenance.json](../../media/provenance.json).
- Every artwork was inspected from an MCP render. Final exported cover/social PNGs
  were inspected again. The GitHub Markdown API rendered the README; browser review
  confirmed all four embedded illustrations loaded and no horizontal page overflow
  at the existing 1280-pixel viewport. This was a local preview of GitHub's HTML,
  not a claim about an already merged default branch.

The first source-binary authoring check lacked Inkscape on PATH and refused rendering.
The source test environment was corrected to include the installed Inkscape directory;
the later passing check has a separate evidence directory. No application was installed
or launched to correct this test setup.

## Scope and remaining limits

Inkscape 1.4.3 performed the rendering. Fontconfig was unavailable to the configured
server, so font availability was explicitly reported as unchecked; actual Arial text
was reviewed in the renders. Gradient visibility remains a documented static-analysis
unknown and was reviewed visually. Vector-content status is passed, not unknown.
The SVG masters retain live text, so other viewers may substitute fonts; PNGs provide
the reviewed portable appearance.

This sprint does not qualify native selection, GUI Undo/Redo, clean-machine installation,
signing/notarization, another Mac or general artist acceptance. No release was published.
Temporary acceptance workspaces are owned by the existing test tooling; compact reports,
traces and the new regression evidence were retained locally. Shared build caches were reused.
