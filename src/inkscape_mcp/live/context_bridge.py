"""Bounded client for the optional in-process GTK document context bridge."""

from __future__ import annotations

import ast
import re

from inkscape_mcp.live.dbus_backend import DBusTransport
from inkscape_mcp.live.transport import LiveConnectionError, LiveContextError, LiveDocumentRef
from inkscape_mcp.workspace.subprocess_exec import ProcessError, run_process

ENV_BRIDGE = "INKSCAPE_MCP_CONTEXT_BRIDGE"
OBJECT_PATH = "/org/inkscape/Inkscape/MCPContext"
INTERFACE = "org.inkscape.MCP.Context1"
_UUID = re.compile(r"^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$")


def context_call(method: str, timeout: float, *params: str) -> str:
    argv = DBusTransport._actions_call_argv(method, *params, object_path=OBJECT_PATH)
    index = argv.index("--method") + 1
    argv[index] = f"{INTERFACE}.{method}"
    try:
        result = run_process(argv, timeout_s=timeout)
    except ProcessError as exc:
        raise LiveConnectionError(
            "document bridge unavailable; reconnect after saving work"
        ) from exc
    if result.timed_out:
        raise LiveConnectionError(
            "document bridge timed out; inspect Inkscape before retrying an edit"
        )
    if result.returncode:
        if "org.inkscape.MCP.ContextChanged" in result.stderr:
            raise LiveContextError(
                "document context unavailable or changed; list and select the drawing again"
            )
        raise LiveConnectionError("document bridge communication failed")
    if len(result.stdout.encode()) > 1024 * 1024:
        raise LiveContextError("document bridge response exceeds size cap")
    return result.stdout.strip()


def _ref(row: object) -> LiveDocumentRef:
    if not isinstance(row, tuple) or len(row) != 3 or not all(isinstance(x, str) for x in row):
        raise LiveContextError("invalid document context reply")
    window, document, title = row
    if not _UUID.fullmatch(window) or not _UUID.fullmatch(document):
        raise LiveContextError("no drawing window available; activate a drawing and retry")
    return LiveDocumentRef(window_id=window, document_id=document, name=title or None)


def read_context(timeout: float) -> LiveDocumentRef:
    try:
        row = ast.literal_eval(context_call("GetContext", timeout))
    except (ValueError, SyntaxError) as exc:
        raise LiveContextError("invalid document context reply") from exc
    return _ref(row)


def list_contexts(timeout: float) -> list[LiveDocumentRef]:
    try:
        reply = context_call("ListDocuments", timeout).replace("@a(sss) ", "")
        rows = ast.literal_eval(reply)
    except (ValueError, SyntaxError) as exc:
        raise LiveContextError("invalid document list reply") from exc
    if not isinstance(rows, tuple) or len(rows) != 1 or not isinstance(rows[0], list):
        raise LiveContextError("invalid document list reply")
    return [_ref(row) for row in rows[0]]


def validate_identity(window_id: str, document_id: str) -> None:
    if not _UUID.fullmatch(window_id) or not _UUID.fullmatch(document_id):
        raise LiveContextError("use window_id and document_id returned by live_list_documents")
