#!/usr/bin/env python3
"""Fixed synthetic native acceptance operations on an already recorded owned session."""

import argparse
import json
from pathlib import Path

from migration_probe import Wire, data, require, write


def main(output, phase):
    session = json.loads((output / "session.json").read_text())
    wire = Wire(
        [session["package"] + "/bin/inkscape-mcp"], session["env"], output / (phase + ".stderr.log")
    )

    def capture_operation(label, reply):
        resource = wire.request("resources/read", {"uri": "inkscape://live/operations"})
        write(output / (label + ".operations.json"), resource)
        require(not resource.get("error"), "mutation applied; audit read failed; do not retry")
        operations = json.loads(resource["result"]["contents"][0]["text"])["operations"]
        expected = data(reply)
        record = [item for item in operations if item["operation_id"] == expected["operation_id"]]
        require(
            len(record) == 1
            and record[0]["status"] == "applied"
            and record[0]["affected_ids"] == expected["affected_ids"],
            "mutation applied; audit confirmation failed; do not retry",
        )

    try:
        wire.initialize()
        connected = data(wire.call("live_connect", {"prefer": "no_freeze"}))
        require(connected["connected"] and not connected["ready_to_edit"], "reconnect guard")
        documents = data(wire.call("live_list_documents", {}))["documents"]
        expected = session["document"]
        require(len(documents) == 1, "unexpected owned window; no mutation")
        require(
            all(documents[0][key] == expected[key] for key in ("window_id", "document_id", "path")),
            "owned context changed",
        )
        require(
            documents[0]["name"] in (expected["name"], "*" + expected["name"]),
            "unexpected owned window title",
        )
        if phase == "unbound-guard":
            refused = wire.call(
                "live_apply_to_selection", {"fill": "red", "approval_token": "test"}
            )
            require(refused["result"].get("isError"), "unbound mutation allowed")
            write(output / (phase + ".json"), refused)
            return
        selected = data(
            wire.call(
                "live_select_document",
                {"window_id": expected["window_id"], "document_id": expected["document_id"]},
            )
        )
        require(selected["ready_to_edit"], "binding failed")
        if phase == "approval-guard":
            refused = wire.call("live_apply_to_selection", {"fill": "red"})
            write(output / (phase + ".json"), refused)
            require(refused["result"].get("isError"), "unapproved mutation allowed")
            return
        if phase == "insert":
            args = {
                "svg_fragment": '<g xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" '
                'inkscape:label="Native acceptance"><rect id="rectangle" x="20" y="20" '
                'width="100" height="60" fill="#1464dc"/></g>',
                "approval_token": "native-acceptance",
            }
            reply = wire.call("live_insert_svg", args)
            write(output / "insert.json", reply)
            require(not reply["result"].get("isError"), "insertion failed; do not retry")
            capture_operation("insert", reply)
        else:
            ident = data(json.loads((output / "insert.json").read_text()))["affected_ids"][0]
            actual = data(wire.call("live_get_selection", {}))
            require(actual["object_ids"] == [ident], "unexpected selection; no mutation")
            if phase == "style-repeat":
                for label in ("changed", "repeated"):
                    reply = wire.call(
                        "live_apply_to_selection",
                        {"fill": "#dc6414", "approval_token": "native-acceptance"},
                    )
                    write(output / ("style-" + label + ".json"), reply)
                    require(not reply["result"].get("isError"), "style failed; do not retry")
                    capture_operation("style-" + label, reply)
            elif phase == "duplicate":
                reply = wire.call(
                    "live_edit_selection",
                    {"operation": "duplicate", "approval_token": "native-acceptance"},
                )
                write(output / "duplicate.json", reply)
                require(not reply["result"].get("isError"), "duplicate failed; do not retry")
                capture_operation("duplicate", reply)
        synced = wire.call("live_sync_to_workspace", {"dest_path": "native-" + phase + ".svg"})
        require(not synced["result"].get("isError"), "sync failed")
        write(output / (phase + ".sync.json"), synced)
        write(output / (phase + ".scene.json"), data(wire.call("live_get_scene", {})))
        print(phase + " completed on owned synthetic document", flush=True)
    finally:
        write(output / (phase + ".trace.json"), wire.trace)
        wire.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "phase", choices=["insert", "style-repeat", "duplicate", "unbound-guard", "approval-guard"]
    )
    args = parser.parse_args()
    main(args.output, args.phase)
