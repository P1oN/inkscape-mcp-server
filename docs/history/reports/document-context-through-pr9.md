> Historical guide snapshot, archived 2026-10-05. Commands and acceptance
> claims belong to the recorded implementations; use the [documentation index](../../README.md).

Current Rust implementation: rust/src/live_context.rs, live_session.rs and live_managed.rs.
Legacy Python source paths and commands below describe historical implementation only;
the Python MCP server is retired. Use README.md and CONTRIBUTING.md for current operation.

# Managed macOS document context

The managed GTK 3 session can identify and select a drawing window, then check that
identity inside Inkscape immediately before dispatching an action. Historical stage38 native
acceptance passed on official Inkscape1.4.3, including continuous STDIO two-window mutation
refusal and explicit rebinding. See RUST_MIGRATION_REPORT.md for that evidence and
[AGENT_HANDOFF.md](../../AGENT_HANDOFF.md) for current scope; this PR has no new GUI acceptance.
The integration remains experimental.

## Workflow

1. Configure the [managed macOS launcher](../../live/macos-live-prototype.md). If no managed window is
   running and the user asks to open Inkscape, call `live_launch()`. Then call
   `live_connect(prefer="no_freeze")`; connecting itself never opens a window.
2. `live_list_documents()` returns open drawing windows with titles and opaque
   `window_id` / `document_id`. Names and identical SVG content do not identify a window.
3. `live_select_document(window_id=..., document_id=...)` activates the chosen window and
   binds it to the task. Check `live_status.ready_to_edit` and inspect `live_get_scene`.
4. Make the requested fill or SVG insertion. Switching to another window or replacing
   its document causes refusal. Inspect the drawing and choose it again before continuing.
5. After reconnect, choose the task drawing again. Reconnect keeps an existing GUI and unsaved
   work. It does not reopen a closed GUI; that requires another explicit launch request.

A window UUID belongs to its live GTK window object. The document UUID belongs to the
Inkscape document's GAction group. Neither depends on filenames, SVG root IDs or content
fingerprints. IDs survive edits while those GTK objects remain alive. Saving does not itself mint an ID;
if any operation replaces the document group, choose it again. Closing/reopening or
replacing a document can invalidate its identity. Two windows showing the same document
share its document identity but have different window identities. IDs are runtime handles,
not permanent file IDs, and are not credentials.

`connection_state` distinguishes disabled, disconnected, connected, connection_lost and
document_unavailable. `recovery_actions` explains the next step. A working private bus does
not guarantee an active drawing. An edit timeout may mean the edit applied: inspect the
selected task drawing before retrying. A discarded operation with `completion_uncertain=true`
is not proof of rollback. Never restart or kill an unsaved GUI automatically.

## Bridge and installation

Official macOS Inkscape 1.4.3 exports document actions but omits the usual GTK window action
paths. Its application actions operate on the currently active document; comparing exported
SVGs cannot reliably distinguish two identical drawings. The implementation therefore uses
a small GTK module (`runtime/native/context.m`) and only public GTK/GIO/Cocoa
APIs. It does not depend on Inkscape's C++ object layout or modify drawing XML to assign IDs.

The module exposes a fixed private D-Bus interface: list, get context, select and dispatch
an allowlisted adapter action. The GUI main-loop callback compares the expected window and
document UUIDs and immediately activates the GAction, without returning to the event loop
between the check and dispatch. The insertion effect captures that document and retains its
existing ID/content fingerprint guard and native Undo transaction. Each related scene/frame
or edit/preview sequence shares one context; manual focus changes cause a later action to refuse.

The vendor executable's hardened runtime rejects external GTK modules. The launcher makes a
**session-local executable copy, removes its vendor signature by signing that copy ad hoc,
and loads the module there**. The copied bundle has a separate identifier and links to vendor
resources. `/Applications/Inkscape.app` stays unchanged. The copy does not retain the vendor
executable's hardened-runtime protection or signature; this remains an experimental integration,
not a supported vendor plugin installation. Only the session's private bus is used.

Ready archive sessions require compatible official GTK3 Inkscape. The archive supplies a
prebuilt bridge and private runtime/bus; end users do not install clang or Homebrew headers.
The fixed supervisor prepares the owned copy on explicit launch. Source/package development
still needs native tools. `./run-mcp.sh --doctor` checks packaged prerequisites. A content hash
invalidates the cache after native source or executable changes. Updating Inkscape's resource bundle requires saving
and restarting a managed GUI before use. A fresh session launch passed native acceptance.

A GUI started by an older version is reused with its existing behavior and an explicit
legacy-session note: `document_guard_available` / `ready_to_edit` are false and the new
selection tools are unsupported. Save and close that managed GUI, explicitly launch it again, then reconnect to load
the module. Restarting MCP alone never upgrades a running GUI's native module.

Inkscape 1.4.3 was observed crashing during primary-monitor initialization while the Mac was
locked. The module now refuses startup before loading any drawing when no primary monitor
is available, with an unlock-and-retry message in `inkscape.stderr.log`. The locked-Mac refusal
has been observed in a native run; startup and full acceptance also passed after unlocking the Mac.

## Historical stage 2 validation (2026-10-01)

The counts and intermittent failure below record stage 2 before PR #5 and the explicit-launch
change. For current development status see [AGENT_HANDOFF.md](../../AGENT_HANDOFF.md).

Automated tests cover malformed and missing identities, identical titles with distinct IDs,
window switches and document replacement, refusal before edits, native dispatch parameters,
lock cleanup, explicit binding, connection-loss guidance, bridge diagnosis without repair,
and private-copy build/cache behavior. Strict mypy (110 source files), focused Ruff, MCP surface smoke (101 tools) and wheel
build passed at that stage. Full pytest at that stage: 1078 passed, 74 skipped, 1 failed — the previously documented
intermittent fake-shell `test_unknown_action_surfaces_engine_action_error`; an isolated repeat
passed. Native acceptance passed through actual MCP STDIO on 2026-10-01: two identical
SVGs received distinct live identities, edits required explicit choice, fill and insertion
returned to exact before/after content fingerprints with native Undo/Redo, a switch between
context read and dispatch was refused without changing drawing B, and a new STDIO client
reused the GUI and IDs while clearing the task binding. The fresh test GUI was closed only
after verifying both synthetic drawing identities. The report recorded `passed: true`.

Those results belong to the retired Python implementation. Current automated
context/dispatch refusals use native Rust fixtures in `cargo test`; current explicitly
authorized owned GUI phases are documented in [CONTRIBUTING](../../../CONTRIBUTING.md).
No historical count or GUI result establishes acceptance of a new Rust build.

Rust migration is deferred for this PR: native identity and macOS integration required additional
work and exposed actual startup/activation problems. The Python server and inkex insertion helper
retain their functionality. A full migration needs a separate feature-parity and packaging plan;
this implementation does not claim Rust performance improvements.
