"""Native macOS acceptance on a NEW disposable private session and two synthetic SVGs.

Run with the project interpreter. No existing session or drawing is adopted. By default
only the two generated test windows are closed after success; --keep-gui preserves them.
A failed test preserves the GUI for inspection. Results and logs stay in the printed directory.
"""

# ruff: noqa: S101, S106 -- acceptance assertions and a non-secret request marker

from __future__ import annotations

import argparse
import asyncio
import json
import os
import sys
import tempfile
import time
from pathlib import Path

from fastmcp import Client
from fastmcp.client.transports import StdioTransport

from inkscape_mcp.config import Settings
from inkscape_mcp.live.context_bridge import context_call, read_context
from inkscape_mcp.live.dbus_backend import DBusTransport, _variant_empty, _variant_string
from inkscape_mcp.live.insert_payload import document_fingerprint
from inkscape_mcp.live.managed_dbus import ManagedDBusTransport
from inkscape_mcp.live.transport import LiveContextError
from inkscape_mcp.workspace.xml_safety import parse_svg_bytes

SVG = (
    '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">'
    '<rect id="box" width="80" height="80" fill="red"/></svg>'
)


async def accept(root: Path, keep_gui: bool) -> dict[str, object]:
    a_path, b_path = root / "a.svg", root / "b.svg"
    a_path.write_text(SVG)
    b_path.write_text(SVG)
    env = dict(os.environ)
    env.update(
        PATH="/Applications/Inkscape.app/Contents/MacOS:/opt/homebrew/bin:"
        "/usr/local/bin:" + env.get("PATH", ""),
        INKSCAPE_PROFILE_DIR=str(root / "profile"),
        INKSCAPE_MCP_WORKSPACE_ROOTS=str(root),
    )
    args = ["-m", "inkscape_mcp.live.macos_launcher", "--session-dir", str(root)]
    transport = StdioTransport(
        command=sys.executable, args=[*args, "--document", str(a_path)], env=env
    )
    async with Client(transport) as client:

        async def call(name: str, **params: object) -> dict:
            result = await client.call_tool(name, params, raise_on_error=False)
            assert not result.is_error, (name, result.content)
            assert isinstance(result.structured_content, dict)
            return result.structured_content

        async def refuses(name: str, **params: object) -> None:
            result = await client.call_tool(name, params, raise_on_error=False)
            assert result.is_error, (name, result.structured_content)

        async def documents(count: int) -> list[dict]:
            # D-Bus registration and file-open replies can precede GTK window creation.
            deadline = time.monotonic() + 10
            while True:
                rows = (await call("live_list_documents"))["documents"]
                if len(rows) == count:
                    return rows
                assert len(rows) < count, ("unexpected drawing", rows)
                assert time.monotonic() < deadline, ("drawing did not open", rows)
                await asyncio.sleep(0.02)

        await call("live_connect", prefer="no_freeze")
        manifest = json.loads((root / "session.json").read_text())
        os.environ.update(
            PATH=env["PATH"],
            DBUS_SESSION_BUS_ADDRESS=manifest["address"],
            INKSCAPE_MCP_CONTEXT_BRIDGE="1",
            INKSCAPE_MCP_MANAGED_DIR=str(root),
            INKSCAPE_MCP_MANAGED_STDOUT=str(root / "inkscape.stdout.log"),
        )
        raw = DBusTransport(Settings(process_timeout_s=10))
        native = ManagedDBusTransport(Settings(process_timeout_s=10))

        def fingerprint() -> str:
            return document_fingerprint(
                parse_svg_bytes(native.get_document_svg().encode()).getroot()
            )

        a = (await documents(1))[0]
        await refuses("live_apply_to_selection", fill="blue", approval_token="requested")
        raw._activate("file-open-window", _variant_string(str(b_path)))
        rows = await documents(2)
        b = next(row for row in rows if row["window_id"] != a["window_id"])
        assert b["document_id"] != a["document_id"]  # identical SVG, distinct live objects

        async def select(row: dict) -> None:
            result = await call(
                "live_select_document", window_id=row["window_id"], document_id=row["document_id"]
            )
            assert result["ready_to_edit"]

        await select(b)
        b_before = fingerprint()
        await select(a)
        scene = (await call("live_get_scene"))["scene"]
        assert scene["active_document"]["document_id"] == a["document_id"]
        raw._activate("select-clear", _variant_empty())
        raw._activate("select-by-id", _variant_string("box"))
        before_fill = fingerprint()
        await call("live_apply_to_selection", fill="#123456", approval_token="requested")
        after_fill = fingerprint()
        assert before_fill != after_fill
        # A is the first window in this newly created GUI. Undo/Redo are native document actions.
        doc_path = "/org/inkscape/Inkscape/document/1"
        raw._activate("undo", _variant_empty(), object_path=doc_path)
        assert fingerprint() == before_fill
        raw._activate("redo", _variant_empty(), object_path=doc_path)
        assert fingerprint() == after_fill

        fragment = '<circle id="test" cx="50" cy="50" r="10" fill="blue"/>'
        await call("live_insert_svg", svg_fragment=fragment, approval_token="requested")
        after_insert = fingerprint()
        assert after_insert != after_fill
        raw._activate("undo", _variant_empty(), object_path=doc_path)
        assert fingerprint() == after_fill
        raw._activate("redo", _variant_empty(), object_path=doc_path)
        assert fingerprint() == after_insert
        (root / "after.svg").write_text(native.get_document_svg())

        # Pin A in the server, then change to B before the GAction reaches the GUI.
        with native._operation():
            context_call("SelectDocument", 10, b["window_id"], b["document_id"])
            deadline = time.monotonic() + 2
            while read_context(10).window_id != b["window_id"]:
                assert time.monotonic() < deadline
                await asyncio.sleep(0.02)
            try:
                native._activate("object-set-property", _variant_string("fill, blue"))
            except LiveContextError:
                pass
            else:
                raise AssertionError("native dispatch did not reject the document switch")
        assert fingerprint() == b_before
        status = await call("live_status")
        assert (
            not status["ready_to_edit"]
            and status["selected_document"]["document_id"] == a["document_id"]
        )
        await refuses("live_insert_svg", svg_fragment=fragment, approval_token="requested")
        await refuses("live_apply_to_selection", fill="blue", approval_token="requested")
        assert fingerprint() == b_before
        await select(b)
        await select(a)
        assert fingerprint() == after_insert

    # New STDIO connection reuses the GUI and deliberately requires a new task binding.
    async with Client(StdioTransport(command=sys.executable, args=args, env=env)) as client:
        status = (
            await client.call_tool("live_connect", {"prefer": "no_freeze"})
        ).structured_content
        assert status["connected"] and status["selected_document"] is None
        assert (
            json.loads((root / "session.json").read_text())["inkscape_pid"]
            == manifest["inkscape_pid"]
        )
        rows = (await client.call_tool("live_list_documents")).structured_content["documents"]
        assert {(row["window_id"], row["document_id"]) for row in rows} == {
            (a["window_id"], a["document_id"]),
            (b["window_id"], b["document_id"]),
        }
        if not keep_gui:
            # Close only the two synthetic windows from this new session.
            expected = {(a["window_id"], a["document_id"]), (b["window_id"], b["document_id"])}
            if {(row["window_id"], row["document_id"]) for row in rows} != expected:
                raise AssertionError("unexpected drawing: preserve GUI for inspection")
            raw._activate("quit-immediate", _variant_empty())
    return {
        "passed": True,
        "session_dir": str(root),
        "gui_pid": manifest["inkscape_pid"],
        "kept_gui": keep_gui,
        "checks": [
            "identical SVG identities",
            "explicit choice",
            "fill and insert Undo/Redo",
            "native dispatch race refusal",
            "stale binding refusal",
            "A/B selection",
            "STDIO reuse and binding reset",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keep-gui", action="store_true")
    options = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("requires macOS and official Inkscape 1.4.3")
    root = Path(tempfile.mkdtemp(prefix="imcp-context-", dir="/tmp")).resolve()
    print(f"Acceptance directory: {root}", flush=True)
    report = asyncio.run(accept(root, options.keep_gui))
    (root / "acceptance.json").write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
