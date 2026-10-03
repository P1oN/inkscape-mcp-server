#!/usr/bin/env python3
"""Verify real document-switch refusal across a continuous owned Rust STDIO connection."""

import argparse
import hashlib
import json
import sys
from pathlib import Path

from migration_probe import Wire, data, require, write
from PIL import Image
from rust_current_live_measurements import ownership


def pixels(workspace, render):
    relative = Path(render["artifact_path"])
    require(not relative.is_absolute() and ".." not in relative.parts, "escaped PNG")
    path = workspace / relative
    require(path.is_file() and not path.is_symlink(), "missing PNG")
    with Image.open(path) as image:
        image = image.convert("RGBA")
        return image.size, hashlib.sha256(image.tobytes()).hexdigest()


def main(source, output):
    session = json.loads((source / "session.json").read_text())
    output.mkdir(parents=True, exist_ok=False)
    write(output / "ownership.json", ownership(session))
    workspace = Path(session["env"]["INKSCAPE_MCP_WORKSPACE_ROOTS"])
    wire = Wire(
        [session["package"] + "/bin/inkscape-mcp"], session["env"], output / "server.stderr.log"
    )
    try:
        wire.initialize()
        connected = data(wire.call("live_connect", {"prefer": "no_freeze"}))
        require(connected["connected"] and not connected["ready_to_edit"], "reconnect guard failed")
        docs = data(wire.call("live_list_documents", {}))["documents"]
        require(
            len(docs) == 1 and docs[0]["document_id"] == session["document"]["document_id"],
            "unexpected windows",
        )
        original = session["document"]
        arguments = {key: original[key] for key in ["window_id", "document_id"]}
        require(
            data(wire.call("live_select_document", arguments))["ready_to_edit"], "binding failed"
        )
        before = data(wire.call("live_get_scene", {}))
        write(output / "before.scene.json", before)
        before_pixels = pixels(workspace, before["render"])
        print(
            "BOUND_OWNED_FIRST_WINDOW: create a second blank window via native File > New",
            flush=True,
        )
        require(sys.stdin.readline().strip() == "verify", "expected fixed verify command")
        write(output / "ownership-after-UI.json", ownership(session))
        refused = wire.call(
            "live_apply_to_selection", {"fill": "red", "approval_token": "native-guard-acceptance"}
        )
        write(output / "refused.json", refused)
        message = refused["result"]["content"][0]["text"].lower()
        require(
            refused["result"].get("isError")
            and "select" in message
            and ("document" in message or "drawing" in message),
            "document-switch mutation not refused",
        )
        docs = data(wire.call("live_list_documents", {}))["documents"]
        write(output / "documents.json", docs)
        require(
            len(docs) == 2
            and any(all(doc[key] == original[key] for key in arguments) for doc in docs),
            "unexpected second window",
        )
        second = next(doc for doc in docs if doc["window_id"] != original["window_id"])
        require(second["document_id"] != original["document_id"], "new drawing identity collided")
        wire.call("live_select_document", {key: second[key] for key in arguments})
        blank = data(wire.call("live_get_scene", {}))
        write(output / "second.scene.json", blank)
        require(
            not any(
                node["tag"] in ["rect", "circle", "path", "image"]
                for node in blank["scene"]["tree"]["children"]
            ),
            "new blank drawing was mutated",
        )
        require(
            data(wire.call("live_select_document", arguments))["ready_to_edit"],
            "return binding failed",
        )
        after = data(wire.call("live_get_scene", {}))
        write(output / "after.scene.json", after)
        require(after["scene"]["tree"] == before["scene"]["tree"], "original tree changed")
        require(pixels(workspace, after["render"]) == before_pixels, "original pixels changed")
        write(
            output / "comparison.json",
            {
                "passed": True,
                "binary_sha256": session["binary_sha256"],
                "checks": [
                    "document-switch refused",
                    "distinct window/document identities",
                    "second drawing blank",
                    "first exact tree/RGBA unchanged",
                    "explicit rebind returns ready",
                ],
                "scope": "Two owned synthetic windows, continuous Rust STDIO; no user documents.",
            },
        )
        print("Window guard acceptance passed", flush=True)
    finally:
        write(output / "server.trace.json", wire.trace)
        wire.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--session-output", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.session_output, args.output)
