# Interactive macOS prototype

Milestone 2 adds structured scene inspection and bounded SVG insertion to the interactive
selection/fill prototype. Insertion uses a short native effect, then returns control to the canvas. Tested on macOS with official Inkscape 1.4.3
(0d15f75), Python 3.12 and FastMCP 3.4.2 (locked install) / 3.4.7 on 2026-09-30.
Other builds are unverified.

## What works

- `live_connect(prefer="no_freeze")` selects `managed-dbus`.
- `live_get_selection` reads the current selection after manual clicks.
- `live_inspect_selection` returns selected ids, element type and explicit paint attributes.
- Full SVG export preserves Inkscape metadata; PNG page previews feed existing operation records.
- `live_apply_to_selection(approval_token="ok", fill="#cc3344")` changes the current selection
  in the open GUI. One fill operation produces one native Undo step.
- `live_get_scene` returns the SVG hierarchy, layers/labels/transforms, selected and visible
  objects, explicit paint and canvas viewBox, alongside a PNG preview.
- `live_insert_svg` adds a self-contained vector fragment as one group at the SVG root in
  document coordinates, remaps ids and internal paint/use references, and records previews.
- Stopping the MCP server does not kill Inkscape. A later connection reuses the same GUI process.

The prototype uses a private Unix D-Bus and Inkscape's own GActions. `select-list` writes ids to
the GUI process's stdout; a subsequent `query-x` supplies a numeric completion marker. The
launcher captures this stream separately from MCP stdout. This is a version-sensitive output
format, not a stable structured selection API: unexpected output fails instead of guessing.
No persistent modal effect extension runs, so the canvas stays interactive.

## Install on the second Mac

Install official Inkscape in `/Applications/Inkscape.app` and Homebrew first. Then:

```sh
brew install dbus glib uv
git clone --branch macos-scene-insertion https://github.com/P1oN/inkscape-mcp-server.git
cd inkscape-mcp-server
uv sync --python 3.12 --frozen
```

No `brew services` daemon is needed. The launcher starts a private bus for its own Inkscape.
The persistent socket extension does not need to be installed or armed. The launcher installs
a separate one-shot insertion helper into the current Inkscape user extensions directory. Its
shell wrapper runs the MCP Python interpreter with Inkscape’s bundled inkex source; the locked
macOS dependencies supply numpy, cssselect and tinycss2. No bundled Python executable is used.
After upgrading this helper, save and close the managed Inkscape before reconnecting so its
extension manifest is reloaded. Restarting MCP alone preserves the existing GUI.

Add an MCP entry to Codex's configuration, substituting absolute paths on that Mac:

```toml
[mcp_servers.inkscape]
command = "/absolute/path/inkscape-mcp-server/.venv/bin/inkscape-mcp-macos"
startup_timeout_sec = 30
tool_timeout_sec = 60

[mcp_servers.inkscape.env]
INKSCAPE_MCP_WORKSPACE_ROOTS = "/Users/yourname/Documents/Drawings"
```

Create that drawing directory first. It contains preview artifacts and snapshots. This root
restricts the server's file operations, but is not a jail for the GUI: the user can open other
files in Inkscape, and live edits target its active document. Keep one managed drawing window
open during this first trial. On first launch, dismiss the Inkscape welcome screen if shown.
Open a COPY of a drawing using File → Open inside the newly launched Inkscape window.
An Inkscape window started normally from Finder is outside this managed session.

The server uses a private session directory `/tmp/inkscape-mcp-<uid>` (0700). It is not a
login service. Closing Inkscape ends its supervisor and private bus. A stalled bus while the
GUI remains open causes restart to refuse: save and close that window before trying again.
Logs are `supervisor.log`, `inkscape.stderr.log`, `bus.log` inside the session directory.

## First user trial

1. Ask Codex to connect with `live_connect(prefer="no_freeze")` and confirm `managed-dbus`.
2. Select a rectangle manually, then ask: “Tell me which object is selected and its fill.”
3. Ask: “Change the selected object's fill to #cc3344.” The inherited tool requires a nonempty
   `approval_token`; this is a request marker, not cryptographic authorization. Codex should
   supply it only for an edit the user requested.
4. Click another object manually and ask Codex to report the new selection.
5. Use Edit → Undo in Inkscape. Confirm the original object's color is restored.
6. Restart the MCP connection. The same drawing, including unsaved work, should remain open.

Save through Inkscape normally when you want to keep the changes.

## Limits and next milestones

A fill edit still supports only one fill change per call. Insertion accepts vector shapes,
groups, text, gradients, clipping, masks and patterns up to 1 MiB / 10,000 elements. It rejects
scripts, images, foreignObject, stylesheet elements, event handlers and external references.
References must resolve inside the fragment. Authored ids are remapped; returned affected ids
include the wrapper and remapped authored ids, not every anonymous child. The group is appended
to the document root rather than inheriting the active layer's transform.

Before insertion the helper checks both document ids and a drawing-content fingerprint. A
changed document is refused. This narrows the race with manual edits but does not lock out human
input. Do not switch drawings or edit while an operation runs. If insertion times out, inspect
the canvas before retrying because completion may be uncertain.

Stroke edits, transforms, existing-text changes, viewport control and change notifications
remain unsupported by this transport. Existing headless tools remain available separately;
changing a file with them does not update the managed canvas automatically.

Selection inspection reports explicit element paint, not computed inherited CSS. Simple
bounding boxes may be unavailable for paths or transformed objects. Scene visibility follows
inline/presentation attributes and ancestors, without computing stylesheet CSS. Hierarchy
includes definitions; the visible-object list excludes definitions and hidden ancestors. Rendering uses the page,
not a screenshot of the current pan/zoom, and changes the GUI's persistent export options.
Do not change the selection or switch documents while an edit is executing. Locks serialize
our action sequences across MCP processes; they cannot lock out human input.

Next: complete save/reopen, document-switch and failure acceptance checks; add existing-text
changes as native transactions,
then test realistic illustrations and package a simpler installer. Rust would not by itself
solve Inkscape's selection/transaction integration; the working transport is the first thing
to validate.

## Verification

Manual test on the real macOS GUI: select `left`, change its fill from #3366cc to #cc3344,
click `right`, observe its new id, then Edit → Undo restores `left` to #3366cc. The same path
also passed through a real FastMCP STDIO client, including before/after PNG operation artifacts.
Two STDIO connections reused the same GUI PID after their server processes stopped.
Keyboard Undo shortcuts were not verified.

Final automated suite on the locked install: 1021 passed, 74 skipped and one failure in the
pre-existing intermittent `test_unknown_action_surfaces_engine_action_error` (also reproduced
on unchanged upstream). The earlier stage-2 full run passed 1020 tests before the last two
capability regression cases were added. All 29 focused scene/insertion/transport tests pass.
`uv sync --frozen`, full MCP surface smoke and strict mypy (107 source files) passed.
Focused Ruff checks passed; the repository-wide lint still has 22 pre-existing long-line findings.
The CLI engine framing failure is outside this managed-session change.

Milestone 2 also passed actual FastMCP STDIO insertion of a gradient rectangle and circle into
the live macOS fixture, with before/after PNGs and a subsequent connection observing the same
GUI PID and added objects. The content-fingerprint check passed with the real inkex input.
Native Edit → Undo/Redo was verified with user assistance on 2026-09-30 for both a simple
rectangle/circle/text fragment and a fragment containing a linear gradient, rectangle, circle
and text. One Undo removed the entire inserted group (including its gradient definition), and
the exported SVG drawing-content fingerprint matched the pre-insertion state. One Redo restored
the group, definition and references; the fingerprint matched the post-insertion state. The user
also observed the gradient reappear. Previously existing objects were preserved. This branch
remains experimental pending save/reopen, document-switch and failure acceptance checks.
