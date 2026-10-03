#!/usr/bin/env python3
"""Fixed structure/transform/order acceptance on a recorded owned native session."""

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from migration_probe import Wire, data, require, write


def main(output, phase):
    require(
        phase
        in {"group-selection-apply"}
        | {
            f"{op}-{step}"
            for op in ("group", "ungroup", "delete", "transform", "lower")
            for step in ("apply", "undone", "redone", "restored")
        },
        "unknown fixed native phase",
    )
    path = output / (phase + ".trace.json")
    require(not path.exists(), "phase already captured; inspect before repeating")
    session = json.loads((output / "session.json").read_text())
    package = Path(session["package"])
    root = Path(session["root"])
    actual = json.loads((root / "session/session.json").read_text())
    require(actual == session["manifest"], "owned process manifest changed")
    require(
        hashlib.sha256((package / "bin/inkscape-mcp").read_bytes()).hexdigest()
        == session["binary_sha256"],
        "owned package binary changed",
    )
    pids = [actual["supervisor_pid"], actual["inkscape_pid"]]
    ps = subprocess.run(
        ["/bin/ps", "-p", ",".join(map(str, pids)), "-o", "pid=,ppid=,args="],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    rows = {
        int(parts[0]): (int(parts[1]), parts[2])
        for line in ps.splitlines()
        if len(parts := line.strip().split(None, 2)) == 3
    }
    require(
        rows.get(pids[1])
        == (
            pids[0],
            str(root / "session/context-bridge/Inkscape.app/Contents/MacOS/inkscape")
            + " --with-gui",
        ),
        "owned native PID/parent/path changed",
    )
    require(
        str(package / "libexec/inkscape-mcp/supervise.py") in rows[pids[0]][1]
        and str(root / "session") in rows[pids[0]][1],
        "owned supervisor changed",
    )
    write(output / (phase + ".ownership.json"), {"manifest": actual, "ps": ps})
    wire = Wire(
        [str(package / "bin/inkscape-mcp")], session["env"], output / (phase + ".stderr.log")
    )

    def capture(label):
        sync = wire.call("live_sync_to_workspace", {"dest_path": "native-" + label + ".svg"})
        write(output / (label + ".sync.json"), sync)
        require(not sync["result"].get("isError"), "capture sync failed; do not retry mutation")
        scene = data(wire.call("live_get_scene", {}))
        write(output / (label + ".scene.json"), scene)
        return scene

    try:
        wire.initialize()
        connected = data(wire.call("live_connect", {"prefer": "no_freeze"}))
        require(connected["connected"] and not connected["ready_to_edit"], "reconnect guard")
        docs = data(wire.call("live_list_documents", {}))["documents"]
        expected = session["document"]
        require(
            len(docs) == 1
            and all(docs[0][k] == expected[k] for k in ("window_id", "document_id", "path")),
            "owned document identity changed",
        )
        require(
            docs[0]["name"] in (expected["name"], "*" + expected["name"]),
            "owned document title changed",
        )
        selected = data(
            wire.call(
                "live_select_document", {k: expected[k] for k in ("window_id", "document_id")}
            )
        )
        require(selected["ready_to_edit"], "binding failed")
        if phase.endswith("-apply"):
            op = phase.split("-", 1)[0]
            before = capture(phase + "-before")
            allowed = {
                node["id"]
                for node in before["scene"]["tree"]["children"]
                if node["tag"] == "g" and not node["is_layer"]
            }
            selection = data(wire.call("live_get_selection", {}))
            write(output / (phase + ".selection.json"), selection)
            if op == "lower":
                duplicate = data(json.loads((output / "duplicate.json").read_text()))
                require(
                    selection["object_ids"] == [duplicate["affected_ids"][0]]
                    and selection["object_ids"][0] in allowed,
                    "expected the known duplicate group alone; no mutation",
                )
            else:
                require(
                    allowed
                    and set(selection["object_ids"]) == allowed
                    and len(selection["object_ids"]) == len(allowed),
                    "unexpected selection; no mutation",
                )
            require(
                len(allowed) == (1 if op == "ungroup" else 2), "unexpected owned fixture structure"
            )
            if op == "transform":
                reply = wire.call(
                    "live_apply_to_selection",
                    {
                        "dx": 20,
                        "dy": 15,
                        "scale": 1.25,
                        "rotate": 15,
                        "approval_token": "native-transform-acceptance",
                    },
                )
            else:
                reply = wire.call(
                    "live_edit_selection",
                    {"operation": op, "approval_token": "native-structure-acceptance"},
                )
            write(output / (phase + ".reply.json"), reply)
            require(not reply["result"].get("isError"), "mutation failed/uncertain; do not retry")
            audit = wire.request("resources/read", {"uri": "inkscape://live/operations"})
            write(output / (phase + ".operations.json"), audit)
            record = [
                r
                for r in json.loads(audit["result"]["contents"][0]["text"])["operations"]
                if r["operation_id"] == data(reply)["operation_id"]
            ]
            require(
                len(record) == 1 and record[0]["status"] == "applied",
                "applied mutation audit missing; do not retry",
            )
        scene = capture(phase)
        print(json.dumps({"phase": phase, "objects": scene["scene"]["object_count"]}))
    finally:
        write(path, wire.trace)
        wire.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("phase")
    args = parser.parse_args()
    main(args.output, args.phase)
