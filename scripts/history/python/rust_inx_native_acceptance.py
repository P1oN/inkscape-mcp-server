#!/usr/bin/env python3
"""Fixed stage-4 operations on the single recorded, owned synthetic native drawing.

Launch with migration_native_gui_acceptance.py first. Selection and native Undo/Redo
are performed through the owned GUI; this runner never guesses or changes a selection.
The Python fingerprint function is development fixture construction for the frozen v1
wire format only. Packaged one-shot execution uses Rust with no interpreter.
"""

import argparse
import ast
import json
import subprocess
import sys
import time
from pathlib import Path

from lxml import etree
from migration_probe import Wire, data, require, write

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "runtime"))
from insert_payload import document_fingerprint


def main(output, phase, label):
    evidence = json.loads((output / "session.json").read_text())
    package = Path(evidence["package"])
    root = Path(evidence["root"])
    manifest = evidence["manifest"]
    require(
        json.loads((root / "session/session.json").read_text()) == manifest,
        "owned manifest changed; refuse operation",
    )
    env = evidence["env"]
    wire = Wire([str(package / "bin/inkscape-mcp")], env, output / (label + ".stderr.log"))
    try:
        wire.initialize()
        require(data(wire.call("live_connect", {"prefer": "no_freeze"}))["connected"], "connect")
        docs = data(wire.call("live_list_documents", {}))["documents"]
        expected = evidence["document"]
        require(
            len(docs) == 1
            and all(docs[0][k] == expected[k] for k in ("window_id", "document_id", "path")),
            "owned synthetic context changed; refuse operation",
        )
        wire.call(
            "live_select_document",
            {"window_id": expected["window_id"], "document_id": expected["document_id"]},
        )

        def capture(suffix):
            name = f"native-{label}-{suffix}.svg"
            reply = wire.call("live_sync_to_workspace", {"dest_path": name})
            write(output / f"{label}-{suffix}.sync.json", reply)
            require(not reply["result"].get("isError"), "capture failed")
            return root / "workspace" / name

        before = capture("before")
        selected = data(wire.call("live_get_selection", {}))["object_ids"]
        if phase == "capture":
            write(output / (label + ".json"), {"captured": str(before), "selection": selected})
            return
        if phase.startswith("stale-"):
            require(selected, "select a synthetic object first")
            svg = etree.parse(str(before)).getroot()
            request = {
                "nonce": "mcp_" + "f" * 32,
                "operation": "style",
                "selection": selected,
                "style": {"opacity": "0.2"},
                "expected_ids": [e.get("id") for e in svg.iter() if e.get("id")],
                "expected_fingerprint": document_fingerprint(svg),
            }
            reasons = {
                "stale-content": "drawing content changed before insertion",
                "stale-ids": "document changed before insertion",
                "stale-selection": "selection changed before edit",
            }
            if phase == "stale-content":
                request["expected_fingerprint"] = "0" * 64
            elif phase == "stale-ids":
                request["expected_ids"] = []
            else:
                request["selection"] = ["missing-selection"]
            exchange = root / "session"
            require(
                not (exchange / "insert-request.json").exists()
                and not (exchange / "insert-result.json").exists(),
                "existing exchange; inspect before proceeding",
            )
            path = exchange / "insert-request.json"
            path.write_text(json.dumps(request))
            path.chmod(0o600)
            gdbus = package / "libexec/inkscape-mcp/dbus/bin/gdbus"

            def dbus(destination, object_path, method, *args):
                return subprocess.check_output(
                    [
                        str(gdbus),
                        "call",
                        "--address",
                        manifest["address"],
                        "--dest",
                        destination,
                        "--object-path",
                        object_path,
                        "--method",
                        method,
                        *args,
                    ],
                    env=env,
                    text=True,
                    timeout=10,
                )

            owner = ast.literal_eval(
                dbus(
                    "org.freedesktop.DBus",
                    "/org/freedesktop/DBus",
                    "org.freedesktop.DBus.GetNameOwner",
                    "org.inkscape.Inkscape",
                )
            )[0]
            # The bridge compares both UUIDs while invoking the fixed effect action.
            dbus(
                owner,
                "/org/inkscape/Inkscape/MCPContext",
                "org.inkscape.MCP.Context1.Activate",
                expected["window_id"],
                expected["document_id"],
                "org.inkscape-mcp.edit.noprefs",
                "[]",
            )
            deadline = time.monotonic() + 10
            while not (exchange / "insert-result.json").exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            reply = json.loads((exchange / "insert-result.json").read_text())
            write(output / (label + ".json"), reply)
            require(reply["nonce"] == request["nonce"] and not reply["ok"], "stale request applied")
            require(reply["error"] == reasons[phase], "wrong refusal")
            path.unlink()
            (exchange / "insert-result.json").unlink()
        else:
            approval = "owned-stage4-acceptance"
            if phase == "insert":
                tool = "live_insert_svg"
                args = {
                    "svg_fragment": '<g transform="translate(10,5)"><rect id="r" x="10" y="10" '
                    'width="40" height="30" fill="#1464dc"/><circle id="c" cx="60" cy="30" '
                    'r="20" fill="#dc6414"/></g>',
                }
            elif phase == "insert-text":
                tool = "live_insert_svg"
                args = {"svg_fragment": '<text id="t" x="20" y="40" font-size="15">Hello</text>'}
            else:
                require(selected, "select a synthetic object first")
                if phase == "style":
                    tool, args = "live_apply_to_selection", {"opacity": 0.75, "dx": 3, "dy": 2}
                elif phase == "noop":
                    tool, args = "live_apply_to_selection", {"opacity": 0.75}
                elif phase == "text":
                    tool, args = "live_set_selected_text", {"text": "New & editable"}
                else:
                    tool, args = "live_edit_selection", {"operation": phase}
            reply = wire.call(tool, {**args, "approval_token": approval})
            write(output / (label + ".json"), reply)
            require(not reply["result"].get("isError"), "operation failed; inspect, do not retry")
            operations = wire.request("resources/read", {"uri": "inkscape://live/operations"})
            write(output / (label + ".operations.json"), operations)
            records = json.loads(operations["result"]["contents"][0]["text"])["operations"]
            record = [r for r in records if r["operation_id"] == data(reply)["operation_id"]]
            require(len(record) == 1 and record[0]["status"] == "applied", "audit confirmation")
        after = capture("after")
        if phase.startswith("stale-") or phase == "noop":
            a, b = (etree.parse(str(p)) for p in (before, after))
            for doc in (a, b):
                for view in doc.xpath('//*[local-name()="namedview"]'):
                    view.getparent().remove(view)
            require(etree.tostring(a) == etree.tostring(b), "refusal/noop changed SVG")
        print(f"{label}: native {phase} passed on owned synthetic drawing", flush=True)
    finally:
        write(output / (label + ".trace.json"), wire.trace)
        wire.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--label", required=True)
    parser.add_argument(
        "phase",
        choices=[
            "insert",
            "insert-text",
            "style",
            "noop",
            "text",
            "duplicate",
            "delete",
            "group",
            "ungroup",
            "raise",
            "lower",
            "front",
            "back",
            "capture",
            "stale-content",
            "stale-ids",
            "stale-selection",
        ],
    )
    args = parser.parse_args()
    require(
        args.label.isascii() and args.label.replace("-", "").isalnum(), "invalid evidence label"
    )
    main(args.output, args.phase, args.label)
