# Live integration

The current Rust server provides bounded live inspection and edits through native helpers.
Managed macOS sessions use a separate supervisor, private bus and Objective-C context bridge.
This integration is experimental; prepared transports/platform jobs are not evidence of
validated Windows or foreign-target support.

Read the [managed session guide](macos-live-prototype.md), then
[document selection/context](document-context.md), [everyday edits](everyday-edits.md),
[discovery/previews](live-discovery.md) and [helper implementation limits](live-helper-kernels.md).
Current validation is in [the handoff](../AGENT_HANDOFF.md); dated GUI results are in
[history](../history/README.md). Development and explicit owned-GUI acceptance commands
are in [CONTRIBUTING](../../CONTRIBUTING.md).

Startup/reconnect never launch or close Inkscape. A new GUI requires a user request.
Select the task drawing after reconnect and preserve unrelated windows and unsaved work.

See [computed styles and reviewed live packages](reviewed-workflow.md) for the new full-profile tools and the deferred artist pilot protocol.
