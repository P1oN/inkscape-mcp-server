# Managed macOS live integration

The Rust server uses private D-Bus, an Objective-C/GTK context bridge, a separate native
supervisor and fixed Rust INX/socket helpers. It contains no project-supplied Python/inkex
runtime. [Current status](../AGENT_HANDOFF.md) distinguishes local/CI acceptance from dated
GUI evidence; [historical guide](../history/reports/macos-live-prototype-through-pr9.md)
retains earlier native milestones.

## Install and connect

Install Inkscape and use the [installation guide](../install/install.md), or run `./setup.sh`
from the checkout, then `./run-mcp.sh`. Source builds require development tools; ready
archive users do not install Python, uv/pip, Homebrew, Rust or compiler. Doctor:

```sh
./run-mcp.sh --doctor
```

Doctor is read-only and never opens a GUI. Its `ready` status describes prerequisites,
not a running drawing or complete native acceptance. Configure a host to execute the
absolute `run-mcp.sh` path; see [host configuration](../install/host-configs.md).
The Rust launcher does not support the retired Python terminal `--launch`/`--document`
entry points. Request GUI launch through the typed `live_launch` MCP tool.

1. When the user explicitly requests opening Inkscape, call `live_launch` if no managed
   drawing is running. Startup and reconnect themselves never launch it.
2. Connect with `live_connect(prefer="no_freeze")`, list drawings with `live_list_documents`
   and bind the intended window/document UUIDs through `live_select_document`.
3. Check `live_status.ready_to_edit`, inspect the scene/selection and perform the requested
   bounded operation with its required approval marker.
4. Inspect the result. Native Edit → Undo/Redo should reverse/reapply one changed effect.
   After reconnect, select the drawing again; the GUI and unsaved work remain.

Never edit/close user drawings for acceptance or kill processes by name. Native acceptance requires explicit authorization and uses
only separately owned synthetic managed sessions. A normally launched Finder window belongs
to a different session. Keep manual selection/focus changes out of an operation's execution.

## Runtime and limits

The managed supervisor prepares an owned application copy and private bus/profile. Vendor
Inkscape remains unchanged. The packaged bridge is loaded into the ad-hoc signed copy; this
is experimental integration and does not retain the vendor executable's hardened runtime.
No macOS security setting is bypassed. The archive includes five Rust executables, fixed INX assets and private bus libraries.
A source build compiles/packages these native components.

Window/document guards refuse focus/document switches. Reconnect resets the binding. Effect
helpers validate captured content/selection and publish through Inkscape's native transaction;
file save is not a substitute for native Undo. Timeout may leave uncertain completion:
inspect the canvas and audit before retrying. Do not automatically restart an unsaved GUI.
A closed managed drawing needs a fresh explicit launch request. MCP restart does not upgrade
extensions/native code already loaded by a running GUI.

Supported bounded operations and restrictions are in [everyday edits](everyday-edits.md),
[document context](document-context.md) and [agent usage](../agent-usage-guide.md). Scene paint is
explicit attributes rather than computed stylesheet CSS. Rendering is page export rather
than a screenshot of pan/zoom; it may alter GUI export options. Private action output is
version-sensitive and unexpected framing fails rather than guessing.

Dated monitor-startup and GUI acceptance results are retained in
[history](../history/reports/macos-live-prototype-through-pr9.md). They do not establish
acceptance of a new runtime. Capture first-divergence evidence if the historical live group
incident recurs; do not automatically restart or close the drawing.
