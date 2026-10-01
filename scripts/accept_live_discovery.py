"""Run discovery + edit/Undo/Redo pilot on a COPY in a new private macOS session.

Cases JSON is a list of {label, tool, params}; label must match exactly one visible
object. Only live_apply_to_selection and live_edit_selection are allowed. Originals
are never saved. Failure preserves the disposable GUI and evidence directory.
"""

# ruff: noqa: S101, S106 -- acceptance assertions and user-requested test edits

from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
import os
import shutil
import sys
import tempfile
import time
from pathlib import Path

from fastmcp import Client
from fastmcp.client.transports import StdioTransport
from PIL import Image

from inkscape_mcp.config import Settings
from inkscape_mcp.live.dbus_backend import DBusTransport, _variant_empty, _variant_string
from inkscape_mcp.live.geometry import root_mapping
from inkscape_mcp.live.insert_payload import document_fingerprint
from inkscape_mcp.live.managed_dbus import ManagedDBusTransport
from inkscape_mcp.workspace.xml_safety import parse_svg_bytes


async def accept(source: Path, cases: list[dict], root: Path, keep_gui: bool) -> dict:
    original_hash = hashlib.sha256(source.read_bytes()).hexdigest()
    drawing = root / "pilot-copy.svg"
    shutil.copyfile(source, drawing)
    env = dict(os.environ)
    env.update(
        PATH="/Applications/Inkscape.app/Contents/MacOS:/opt/homebrew/bin:"
        "/usr/local/bin:" + env.get("PATH", ""),
        INKSCAPE_PROFILE_DIR=str(root / "profile"),
        INKSCAPE_MCP_WORKSPACE_ROOTS=str(root),
        INKSCAPE_MCP_TOOL_PROFILE="full",
    )
    args = ["-m", "inkscape_mcp.live.macos_launcher", "--session-dir", str(root)]
    rows = []
    async with Client(
        StdioTransport(
            command=sys.executable,
            args=[*args, "--launch", "--document", str(drawing)],
            env=env,
        )
    ) as client:

        async def call(name: str, **params: object) -> dict:
            response = await client.call_tool(name, params, raise_on_error=False)
            assert not response.is_error, (name, response.content)
            assert isinstance(response.structured_content, dict)
            return response.structured_content

        await call("live_connect", prefer="no_freeze")
        manifest = json.loads((root / "session.json").read_text())
        os.environ.update(
            PATH=env["PATH"],
            DBUS_SESSION_BUS_ADDRESS=manifest["address"],
            INKSCAPE_MCP_CONTEXT_BRIDGE="1",
            INKSCAPE_MCP_MANAGED_DIR=str(root),
            INKSCAPE_MCP_MANAGED_STDOUT=str(root / "inkscape.stdout.log"),
        )
        native = ManagedDBusTransport(Settings(process_timeout_s=20))
        raw = DBusTransport(Settings(process_timeout_s=20))
        deadline = time.monotonic() + 10
        while True:
            documents = (await call("live_list_documents"))["documents"]
            if documents:
                break
            assert time.monotonic() < deadline, "pilot drawing did not open"
            await asyncio.sleep(0.05)
        assert len(documents) == 1
        doc = documents[0]
        await call(
            "live_select_document", window_id=doc["window_id"], document_id=doc["document_id"]
        )

        def fingerprint() -> str:
            return document_fingerprint(
                parse_svg_bytes(native.get_document_svg().encode()).getroot()
            )

        baseline = fingerprint()
        initial_selection = await call("live_get_selection")
        found = await call("live_find_objects", limit=1000)
        (root / "discovery.json").write_text(json.dumps(found, ensure_ascii=False, indent=2))
        assert fingerprint() == baseline
        assert await call("live_get_selection") == initial_selection
        assert found["active_document"]["document_id"] == doc["document_id"]
        await call("live_render_view", scale=0.25)

        for index, case in enumerate(cases):
            tool = case["tool"]
            assert tool in {"live_apply_to_selection", "live_edit_selection"}
            candidates = await call("live_find_objects", label=case["label"])
            assert candidates["total_matches"] == 1, ("ambiguous pilot label", case, candidates)
            obj = candidates["objects"][0]
            assert obj["bbox"] is not None and not obj["locked"]
            preview = await call(
                "live_preview_object",
                object_id=obj["id"],
                expected_fingerprint=candidates["fingerprint"],
            )
            box = obj["bbox"]
            region = await call(
                "live_render_view",
                region_x=box["x"],
                region_y=box["y"],
                region_width=box["width"],
                region_height=box["height"],
                scale=0.25,
            )
            sx, sy, _, _ = root_mapping(
                parse_svg_bytes(native.get_document_svg().encode()).getroot()
            )
            with Image.open(root / region["artifact_path"]) as image:
                assert abs(image.width - box["width"] * sx * 0.25) <= 1
                assert abs(image.height - box["height"] * sy * 0.25) <= 1
            assert await call("live_get_selection") == initial_selection
            assert fingerprint() == baseline
            raw._activate("select-clear", _variant_empty())
            raw._activate("select-by-id", _variant_string(obj["id"]))
            result = await call(tool, approval_token="requested-copy-pilot", **case["params"])
            assert result["undo_friendly"]
            changed = fingerprint()
            assert changed != baseline
            (root / f"case-{index + 1}-after.svg").write_text(native.get_document_svg())
            frame = await call("live_render_view", scale=0.25)
            stale = await client.call_tool(
                "live_preview_object",
                {
                    "object_id": obj["id"],
                    "expected_fingerprint": candidates["fingerprint"],
                },
                raise_on_error=False,
            )
            assert stale.is_error, "stale preview should refuse"
            document_path = "/org/inkscape/Inkscape/document/1"
            raw._activate("undo", _variant_empty(), object_path=document_path)
            assert fingerprint() == baseline, ("Undo mismatch", case)
            raw._activate("redo", _variant_empty(), object_path=document_path)
            assert fingerprint() == changed, ("Redo mismatch", case)
            raw._activate("undo", _variant_empty(), object_path=document_path)
            assert fingerprint() == baseline
            raw._activate("select-clear", _variant_empty())
            initial_selection = await call("live_get_selection")
            rows.append(
                {
                    "case": case,
                    "object": obj,
                    "preview": preview,
                    "region": region,
                    "after": frame,
                    "undo_redo": "exact fingerprints",
                    "stale_preview_refused": True,
                }
            )
        assert hashlib.sha256(source.read_bytes()).hexdigest() == original_hash
        if not keep_gui:
            documents = (await call("live_list_documents"))["documents"]
            assert len(documents) == 1 and documents[0]["document_id"] == doc["document_id"]
            assert fingerprint() == baseline
            raw._activate("quit-immediate", _variant_empty())
    return {
        "passed": True,
        "source_sha256": original_hash,
        "original_unchanged": True,
        "objects": found["total_matches"],
        "cases": rows,
        "kept_gui": keep_gui,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--cases", type=Path, required=True)
    parser.add_argument("--keep-gui", action="store_true")
    options = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("requires macOS managed Inkscape")
    root = Path(tempfile.mkdtemp(prefix="imcp-discovery-pilot-")).resolve()
    print(f"Pilot directory: {root}", flush=True)
    cases = json.loads(options.cases.read_text())
    try:
        report = asyncio.run(accept(options.source, cases, root, options.keep_gui))
    except Exception as exc:
        (root / "acceptance.json").write_text(json.dumps({"passed": False, "error": str(exc)}))
        raise
    (root / "acceptance.json").write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
