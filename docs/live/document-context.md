# Managed macOS document context

The managed GTK 3 session identifies and selects a drawing window, then verifies that
identity inside Inkscape immediately before native dispatch. The current implementation
is in `rust/src/live_context.rs`, `live_session.rs` and `live_managed.rs`. The integration
remains experimental; [current status](../AGENT_HANDOFF.md) records validation limits.

## Workflow

1. Configure the [managed macOS launcher](macos-live-prototype.md). If no managed window is
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

The context module refuses startup when no primary monitor is available, rather than
bypassing the guard. Historical locked-Mac/startup and document-selection results are
[archived](../history/reports/document-context-through-pr9.md). Current regression and
explicit owned-GUI acceptance commands are in [CONTRIBUTING](../../CONTRIBUTING.md);
old Python test counts do not establish Rust acceptance.
