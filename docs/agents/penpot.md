# Penpot MCP access for Manager UI

Use `penpot_mcp` when reading or changing Manager designs and before implementing Manager UI. The project document is **Inkscape MCP Manager**, file ID `38954aef-1e2c-81ac-8008-f0de1d0c768f`, headless alias `inkscape-manager`. Inspect the page, boards and states referenced by the current task; other pages or proposals do not establish design approval.

## Select the document

1. Read the server's `high_level_overview` once per conversation before design operations. Obtain current sessions from its session-selection response or the configured headless worker's `status`. Match the exact file ID; a matching name alone is insufficient.
2. Pass that `sessionId` explicitly to document tools, including `execute_code` and `export_shape`. First read `penpot.currentFile.id` and the intended page through MCP, then inspect the task's boards. Use the `inkscape-manager` worker entry even if another document makes the overall worker status not ready.
3. Rediscover the session after a disconnect, stale-ID error or browser restart. IDs are runtime state, so keep them out of repository instructions. If the client schema lacks `sessionId`, refresh tool discovery or use the installed CLI's named-document route. Browser focus and Local Budget's session are not fallbacks.

Read designs by default. Modify Penpot only when the current task authorizes design edits; check `penpot.currentFile.id` against the expected ID inside the same code execution before mutation. An interrupted write has an unknown outcome: inspect the document before deciding whether to repeat it. Missing access or an unapproved design blocks the affected UI implementation until resolved.

## Local headless worker

The launcher and its multi-document configuration live outside the repository. On the configured workstation:

```sh
penpot_dir="${PENPOT_HEADLESS_DIR:-$HOME/Documents/Codex/Penpot/headless}"
node "$penpot_dir/penpot-headless.cjs" status "$penpot_dir/config.json"
node "$penpot_dir/penpot-headless.cjs" call "$penpot_dir/config.json" inkscape-manager "$penpot_dir/inspect-document.js"
```

The second command is a read-only MCP access check with an in-execution file-ID guard. If the worker is stopped, `start` with the existing configuration restores its sessions. For a running/recovering worker, diagnose the named entry instead of starting a duplicate or stopping the shared browser. Missing login requires the owner's normal private-profile login. Preserve the other project's session, the existing account/key, and the current manual-start policy. Keep credentials and private browser state outside commits, reports and logs.

Penpot is the product UI design source. SVG authoring through this repository's Inkscape MCP continues to follow the [agent usage guide](../agent-usage-guide.md), including its separate document binding and launch rules.
