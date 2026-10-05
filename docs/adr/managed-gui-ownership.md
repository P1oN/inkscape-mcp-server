# Owned managed GUI and guarded edits

Status: accepted; managed macOS integration remains experimental.
Recorded retrospectively 2026-10-05 from the implemented live workflow.

## Context

Exported SVG content and filenames cannot distinguish identical drawings in separate windows.
A client reconnect must preserve unsaved artwork. A changed drawing also needs native Undo,
which cannot be implemented merely by overwriting its on-disk SVG.

## Decision

Startup/reconnect never launch Inkscape. Explicit user-requested launch creates an owned managed
session through a separate supervisor/private bus and context bridge. Bind opaque window/document
identities for the task and guard native dispatch within the GUI. Fixed INX/socket consumers
validate captured state, references and selection, then use native extension transactions.

## Consequences and validation

A reconnect clears the task binding while preserving the GUI; a new launch needs a user request.
The managed app copy is signed ad hoc and does not retain the vendor hardened runtime; the
vendor bundle is preserved. Approval markers are supplied by the client and do not authenticate
user consent. Timeouts or lost confirmation may mean uncertain completion; inspect before retry.

GUI acceptance uses separately owned synthetic documents, independently captured SVG/renders
and Undo/Redo evidence. Headless tests cannot establish that acceptance. Native helpers already
loaded in a running GUI are not upgraded by MCP restart. See
[session guide](../live/macos-live-prototype.md), [context](../live/document-context.md),
[CONTRIBUTING](../../CONTRIBUTING.md) and [history](../history/README.md).
