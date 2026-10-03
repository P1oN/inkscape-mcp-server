#!/usr/bin/env python3
"""Capture the existing owned native drawing; never launch or mutate its contents."""

import argparse
import json
from pathlib import Path

from migration_probe import Wire, data, require, write


def main(phase, output):
    require(
        phase
        in {
            "undone",
            "redone",
            "guarded",
            "styled",
            "style-undone",
            "style-redone",
            "style-undone-twice",
            "fixed-style-undone",
            "duplicate-undone",
            "duplicate-redone",
            "closure-preflight",
        },
        "unknown capture phase",
    )
    session = json.loads((output / "session.json").read_text())
    wire = Wire(
        [session["package"] + "/bin/inkscape-mcp"], session["env"], output / (phase + ".stderr.log")
    )
    try:
        wire.initialize()
        connected = data(wire.call("live_connect", {"prefer": "no_freeze"}))
        require(connected["connected"] and not connected["ready_to_edit"], "reconnect guard")
        documents = data(wire.call("live_list_documents", {}))
        require(len(documents["documents"]) == 1, "unexpected owned windows")
        actual = documents["documents"][0]
        expected = session["document"]
        require(
            all(actual[key] == expected[key] for key in ("window_id", "document_id", "path")),
            "owned context changed",
        )
        require(
            actual["name"] in (expected["name"], "*" + expected["name"]),
            "unexpected owned window title",
        )
        doc = session["document"]
        wire.call(
            "live_select_document",
            {"window_id": doc["window_id"], "document_id": doc["document_id"]},
        )
        synced = wire.call("live_sync_to_workspace", {"dest_path": "native-" + phase + ".svg"})
        require(not synced["result"].get("isError"), "sync failed")
        scene = data(wire.call("live_get_scene", {}))
        write(output / (phase + ".scene.json"), scene)
        write(output / (phase + ".sync.json"), synced)
        print(json.dumps({"phase": phase, "objects": scene["scene"]["object_count"]}))
    finally:
        write(output / (phase + ".trace.json"), wire.trace)
        wire.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "phase",
        choices=[
            "undone",
            "redone",
            "guarded",
            "styled",
            "style-undone",
            "style-redone",
            "style-undone-twice",
            "fixed-style-undone",
            "duplicate-undone",
            "duplicate-redone",
            "closure-preflight",
        ],
    )
    parser.add_argument(
        "--output", type=Path, default=Path("migration/results/native-gui-acceptance")
    )
    args = parser.parse_args()
    main(args.phase, args.output)
