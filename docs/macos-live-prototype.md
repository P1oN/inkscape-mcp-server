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
- MCP startup never opens Inkscape. Use `live_launch()` only when the user asks to open it.
- Stopping the MCP server does not kill Inkscape. A later connection reuses the same GUI process.

The prototype uses a private Unix D-Bus and Inkscape's own GActions. `select-list` writes ids to
the GUI process's stdout; a subsequent `query-x` supplies a numeric completion marker. The
launcher captures this stream separately from MCP stdout. This is a version-sensitive output
format, not a stable structured selection API: unexpected output fails instead of guessing.
No persistent modal effect extension runs, so the canvas stays interactive.

## Install on the second Mac

Install official Inkscape in `/Applications/Inkscape.app` and Homebrew first. Then:

```sh
xcode-select --install # if Apple command line tools are not installed
brew install dbus glib uv
git clone https://github.com/P1oN/inkscape-mcp-server.git
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

New managed sessions also build the [native context bridge](document-context.md) and a private
ad-hoc signed executable copy. The original vendor application stays unchanged; the copy does not
retain its vendor signature or hardened runtime. Native acceptance passed on 2026-10-01;
see the [recorded results](document-context.md#validation). The integration remains experimental.
Existing legacy sessions are reused with a guard-unavailable note.

The configured command starts MCP without opening a window. It attaches to an existing managed
session when available. Closing the window leaves later MCP connections idle until an explicit
`live_launch()` request. For an explicit terminal launch use `.venv/bin/inkscape-mcp-macos --launch`
(and optionally `--document /absolute/path/drawing.svg`).

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

## Setup and connection diagnosis

Run this before configuring Codex, or when a managed session fails:

```sh
.venv/bin/inkscape-mcp-macos --doctor
```

For a custom session directory, add `--session-dir /absolute/path/to/session`. Keep any
`INKSCAPE_PROFILE_DIR` override the same as the managed session. The command emits JSON and
exits 0 for `ready_to_launch` or `running`; other states exit 1 with suggested next steps.
It checks Inkscape, private-bus tools, the MCP interpreter/dependencies, vendor inkex,
helper files, existing private-session metadata, captured selection output and the live
insertion action. `ready_to_launch` means prerequisites are present, not that a GUI is running.
`helper_installed` describes the current CLI profile; `insertion_available` probes the live
native action. Neither is a guarantee that every drawing/edit will succeed.

Diagnosis does not create a session directory, install/repair the helper, start or close a GUI,
or discard stale metadata. A broken session advises saving and closing its managed drawing
before restarting. Existing logs remain available for inspection. A filesystem/access failure
is reported as `diagnosis_failed` without echoing raw exception details.

For connected managed macOS sessions, `live_status` now reads the current drawing rather than
retaining the connect-time drawing. The real Inkscape `sodipodi:docname` namespace is used in
both status and scene reports. Missing/unreadable active documents are reported as null with
a note; a vanished bus is reported as disconnected. Use `live_connect` to reconnect. Other
transports keep their existing connect-time document semantics.

A filename is not a unique window/document identity, and paths remain null when the transport
cannot report them. New managed sessions provide runtime window/document UUIDs through
`live_list_documents`; `live_select_document` activates and binds the drawing for the task.
The native module rejects actions after a window/document change. Reconnect clears that binding.
See [implementation and acceptance results](document-context.md) before using this experimental
version. `live_status` exposes `document_guard_available`, `ready_to_edit`, `connection_state`
and `recovery_actions`. Legacy sessions have no document guard.

### Diagnosis verification (2026-09-30)

The actual launcher entry point reported `running` for the managed Inkscape 1.4.3 session and
`ready_to_launch` for a nonexistent session directory without creating it. A real FastMCP
STDIO connection reported `inkscape-acceptance.svg` through `live_status`. Native acceptance completed on 2026-10-01: switching the active window from
`inkscape-acceptance.svg` to `inkscape-window-b.svg` and back updated `live_status` in one
continuous MCP STDIO connection, with `connected` remaining true and `connected_at` unchanged.
An in-memory session test also covers recovery from a failed document read.

After merging main on 2026-10-01, the full suite passed: **1058 passed, 74 skipped**
(Inkscape CLI absent from the test PATH). The previously observed intermittent upstream CLI
engine framing failure did not recur. The focused transport/session/scene/diagnosis suite passed
56 tests; strict mypy (108 source files), focused Ruff and MCP surface smoke also passed.
The real launcher still reported `running`, `ready: true` and insertion available for the native
session. The generated tool manifest was refreshed before this acceptance.

## First user trial

1. Ask Codex to open Inkscape (`live_launch`), then connect with `live_connect(prefer="no_freeze")` and confirm `managed-dbus`.
2. Ask Codex to list drawings with `live_list_documents` and choose the task drawing with
   `live_select_document`. Check `live_status.ready_to_edit`. Select a rectangle manually, then ask: “Tell me which object is selected and its fill.”
3. Ask: “Change the selected object's fill to #cc3344.” The inherited tool requires a nonempty
   `approval_token`; this is a request marker, not cryptographic authorization. Codex should
   supply it only for an edit the user requested.
4. Click another object manually and ask Codex to report the new selection.
5. Use Edit → Undo in Inkscape. Confirm the original object's color is restored.
6. Restart the MCP connection. The same drawing, including unsaved work, should remain open.

Save through Inkscape normally when you want to keep the changes.

## Limits and next milestones

Everyday edits now support fill, stroke, stroke width, opacity, document-space transforms,
single-run text, duplication, deletion, grouping and stacking. Each changed call is one native
Undo transaction. See [supported cases and constraints](everyday-edits.md).
Insertion accepts vector shapes,
groups, text, gradients, clipping, masks and patterns up to 1 MiB / 10,000 elements. It rejects
scripts, images, foreignObject, stylesheet elements, event handlers and external references.
References must resolve inside the fragment. Authored ids are remapped; returned affected ids
include the wrapper and remapped authored ids, not every anonymous child. The group is appended
to the document root rather than inheriting the active layer's transform.

Before insertion the helper checks both document ids and a drawing-content fingerprint. A
changed document is refused. This narrows the race with manual edits but does not lock out human
input. Do not switch drawings or edit while an operation runs. If insertion times out, inspect
the canvas before retrying because completion may be uncertain.

Viewport control and change notifications remain unsupported by this transport. Existing headless tools remain available separately;
changing a file with them does not update the managed canvas automatically.

Selection inspection reports explicit element paint, not computed inherited CSS. Simple
bounding boxes may be unavailable for paths or transformed objects. Scene visibility follows
inline/presentation attributes and ancestors, without computing stylesheet CSS. Hierarchy
includes definitions; the visible-object list excludes definitions and hidden ancestors. Rendering uses the page,
not a screenshot of the current pan/zoom, and changes the GUI's persistent export options.
Do not change the selection or switch documents while an edit is executing. Locks serialize
our action sequences across MCP processes; they cannot lock out human input.

Next: test realistic illustrations and improve scene understanding, then package a simpler installer. Rust would not by itself
solve Inkscape's selection/transaction integration; the working transport is the first thing
to validate.

## Verification

Manual test on the real macOS GUI: select `left`, change its fill from #3366cc to #cc3344,
click `right`, observe its new id, then Edit → Undo restores `left` to #3366cc. The same path
also passed through a real FastMCP STDIO client, including before/after PNG operation artifacts.
Two STDIO connections reused the same GUI PID after their server processes stopped.
Keyboard Undo shortcuts were not verified.

Final automated suite on the locked install: 1029 passed, 74 skipped (Inkscape CLI absent
from test PATH). All 36 focused scene/insertion/transport tests pass. Strict mypy (107 source
files) and focused Ruff passed; `uv sync --frozen`, full MCP surface smoke and wheel contents
were also verified during this milestone. Repository-wide lint has 22 pre-existing long-line
findings. Earlier full runs hit the pre-existing intermittent
`test_unknown_action_surfaces_engine_action_error`, reproduced on unchanged upstream; the
final run passed without changing that engine code.

Milestone 2 also passed actual FastMCP STDIO insertion of a gradient rectangle and circle into
the live macOS fixture, with before/after PNGs and a subsequent connection observing the same
GUI PID and added objects. The content-fingerprint check passed with the real inkex input.
Native Edit → Undo/Redo was verified with user assistance on 2026-09-30 for both a simple
rectangle/circle/text fragment and a fragment containing a linear gradient, rectangle, circle
and text. One Undo removed the entire inserted group (including its gradient definition), and
the exported SVG drawing-content fingerprint matched the pre-insertion state. One Redo restored
the group, definition and references; the fingerprint matched the post-insertion state. The user
also observed the gradient reappear. Previously existing objects were preserved. This branch
remains experimental; see the completed acceptance checklist in [ROADMAP.md](ROADMAP.md).

Native Save As (Inkscape SVG) and File → Revert also passed: both the saved file and the
reloaded live SVG drawing-content fingerprint matched the post-insertion state, preserving
the gradient definition/references and original objects. The saved drawing also passed closing its window and reopening the SVG while another managed
window remained open; the live drawing-content fingerprint again matched the saved state.
The MCP scene read followed the active document to window B and back to the reopened drawing.

Seven simulated native-effect failure cases cover timeout, malformed/non-object reply, stale
nonce, refusal, action failure and a refusal reply arriving before a D-Bus failure. Each verifies request/reply cleanup and a subsequent
successful insertion. Malformed replies now produce a stable LiveError. These are automated
transport checks, not a native GUI failure acceptance test.

A controlled stale-fingerprint request exercised refusal in the real native helper. The user
closed its “Insertion refused” dialog; the drawing-content fingerprint remained unchanged.
The dialog initially caused the D-Bus activation call to time out. The server now reconciles
an existing helper reply after an activation failure, reporting explicit refusal when available;
without a reply it reports uncertain completion and asks the client to inspect Inkscape before
retrying. The corrected path was verified with the real helper and a 2-second test timeout.

After the corrected refusal path, the live SVG fingerprint still matched the saved drawing.
Two new actual FastMCP STDIO connections read the scene and reused the same Inkscape PID.

The repeated refusal dialog was closed through native UI automation. Pressing 5 then changed
the page zoom from 25% to 60%, confirming canvas interaction; the drawing-content fingerprint
remained unchanged. The milestone-2 acceptance checklist is complete for the tested fixtures.

Review follow-up (2026-10-01): diagnosis now probes whether a persistent supervisor lock
is actually held without creating/removing files, and rejects resolved socket paths at the
launcher's 104-byte limit. Active-document refresh maps filesystem export failures to
`LiveError`, so status can report an unavailable document instead of failing the entire call.
Regression cases include held/unheld locks, existing/missing directories at 103/104 bytes,
and a real managed export lock-path failure.
