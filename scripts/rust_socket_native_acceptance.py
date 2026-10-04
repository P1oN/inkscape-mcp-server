#!/usr/bin/env python3
"""Owned native socket acceptance; Undo/Redo stays an independent native UI check."""

import argparse
import ast
import base64
import hashlib
import json
import socket
import subprocess
import time
from pathlib import Path

from lxml import etree
from migration_native_gui_acceptance import close_owned_session
from migration_native_gui_acceptance import main as launch
from migration_probe import Wire, data, require, write


def same(a, b):
    def tree(path):
        root = etree.parse(str(path)).getroot()
        for n in list(root):
            if etree.QName(n).localname == "namedview":
                root.remove(n)
        return etree.tostring(root, method="c14n")

    return tree(a) == tree(b)


def main(package, output, phase):
    if phase == "setup":
        launch(package, output)
    evidence = json.loads((output / "session.json").read_text())
    require(evidence["state"] == "connected-owned-blank", "owned session unavailable")
    root = Path(evidence["root"])
    env = evidence["env"]
    expected = evidence["document"]
    manifest = evidence["manifest"]
    require(json.loads((root / "session/session.json").read_text()) == manifest, "manifest changed")
    wire = Wire([str(package / "bin/inkscape-mcp")], env, output / f"{phase}.stderr.log")
    wire.initialize()
    wire.call("live_connect", {"prefer": "no_freeze"})
    docs = data(wire.call("live_list_documents", {}))["documents"]
    require(
        len(docs) == 1 and all(docs[0][k] == expected[k] for k in ("window_id", "document_id")),
        "context changed",
    )
    wire.call(
        "live_select_document",
        {"window_id": expected["window_id"], "document_id": expected["document_id"]},
    )

    def capture(label):
        name = f"socket-{label}.svg"
        if (root / "workspace" / name).exists():
            name = f"socket-{label}-{time.time_ns()}.svg"
        reply = wire.call("live_sync_to_workspace", {"dest_path": name})
        require(not reply["result"].get("isError"), "native capture failed")
        return root / "workspace" / name

    gdbus = package / "libexec/inkscape-mcp/dbus/bin/gdbus"

    def dbus(dest, path, method, *args):
        return subprocess.check_output(
            [
                str(gdbus),
                "call",
                "--address",
                manifest["address"],
                "--dest",
                dest,
                "--object-path",
                path,
                "--method",
                method,
                *args,
            ],
            env=env,
            text=True,
            timeout=15,
        )

    owner = ast.literal_eval(
        dbus(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.GetNameOwner",
            "org.inkscape.Inkscape",
        )
    )[0]

    def bridge(label, command=None, params=None):
        rendezvous = Path(env["INKSCAPE_MCP_LIVE_RENDEZVOUS"])
        require(not rendezvous.exists(), "previous socket still active")
        activation = subprocess.Popen(
            [
                str(gdbus),
                "call",
                "--address",
                manifest["address"],
                "--dest",
                owner,
                "--object-path",
                "/org/inkscape/Inkscape/MCPContext",
                "--method",
                "org.inkscape.MCP.Context1.Activate",
                expected["window_id"],
                expected["document_id"],
                "org.inkscape-mcp.live.noprefs",
                "[]",
            ],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        deadline = time.monotonic() + 15
        while not rendezvous.exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        info = json.loads(rendezvous.read_text())
        ps = subprocess.check_output(
            ["/bin/ps", "-p", str(info["pid"]), "-o", "command="], text=True
        )
        require(str(package / "bin/inkscape-mcp-live") in ps, "wrong native helper process")
        with socket.create_connection(("127.0.0.1", info["port"]), timeout=40) as conn:
            reader = conn.makefile("rb")

            def request(cmd, args=None):
                conn.sendall(
                    json.dumps(
                        {"v": 5, "cmd": cmd, "token": info["token"], "params": args or {}}
                    ).encode()
                    + b"\n"
                )
                reply = json.loads(reader.readline(64 * 1024 * 1024))
                require(reply["ok"], f"native socket {cmd} failed")
                return reply["result"]

            request("hello")
            before = request("get_document_svg")["svg"]
            scene = request("get_scene")
            png = base64.b64decode(
                request("render_view", {"region": [0, 0, 100, 100], "scale": 1})["png_base64"]
            )
            (output / f"{label}.png").write_bytes(png)
            require(png.startswith(b"\x89PNG"), "native render invalid")
            if command:
                reply = request(command, params)
                require(reply["undo_friendly"], "native edit acknowledgment")
            if label == "style":
                request(
                    "insert_svg",
                    {"svg": '<circle id="socket-circle" cx="70" cy="40" r="5" fill="green"/>'},
                )
            after = request("get_document_svg")["svg"]
            write(
                output / f"{label}.socket.json",
                {
                    "scene": scene,
                    "changed": before != after,
                    "helper_pid": info["pid"],
                    "helper_sha256": hashlib.sha256(
                        (package / "bin/inkscape-mcp-live").read_bytes()
                    ).hexdigest(),
                },
            )
            reader.close()
        deadline = time.monotonic() + 15
        while rendezvous.exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        require(not rendezvous.exists(), "socket did not close")
        _stdout, stderr = activation.communicate(timeout=15)
        require(activation.returncode == 0, "native activation failed: " + stderr.decode())
        return capture(label)

    try:
        if phase == "setup":
            capture("blank")
            reply = wire.call(
                "live_insert_svg",
                {
                    "svg_fragment": (
                        '<rect id="r" x="10" y="10" width="30" height="20" '
                        'style="fill:red"/><text id="t" x="10" y="60">Hello</text>'
                    ),
                    "approval_token": "owned-stage5-native-acceptance",
                },
            )
            require(not reply["result"].get("isError"), "native baseline insertion failed")
            ids = data(reply)["affected_ids"]
            write(output / "ids.json", ids)
            capture("baseline")
            print(
                "Select the synthetic rectangle in the recorded private app, then phase style",
                flush=True,
            )
        elif phase == "style":
            ids = json.loads((output / "ids.json").read_text())
            require(
                data(wire.call("live_get_selection", {}))["object_ids"] == [ids[1]],
                "select the owned rectangle first",
            )
            baseline = capture("baseline-selected")
            after = bridge("style", "apply_to_selection", {"style": {"fill": "blue"}})
            require(not same(baseline, after), "native style output not applied")
            noop = bridge("style-noop", "apply_to_selection", {"style": {"fill": "blue"}})
            require(same(after, noop), "native no-op changed SVG")
        elif phase == "text":
            ids = json.loads((output / "ids.json").read_text())
            require(
                data(wire.call("live_get_selection", {}))["object_ids"] == [ids[2]],
                "select the owned text first",
            )
            text = bridge("text", "set_selected_text", {"text": "Native & editable"})
            require("Native &amp; editable" in text.read_text(), "native text not applied")
            noop = bridge("text-noop", "set_selected_text", {"text": "Native & editable"})
            require(same(text, noop), "native text no-op changed SVG")
        elif phase == "verify-text-undo":
            path = capture(phase)
            require(
                same(path, root / "workspace/socket-style.svg"),
                "one native Undo did not reverse text after no-op",
            )
            write(output / "verify-text-undo.json", {"passed": True, "capture": str(path)})
        elif phase == "verify-style-undo":
            path = capture(phase)
            require(
                same(path, root / "workspace/socket-baseline.svg"),
                "one native Undo did not reverse style after no-op",
            )
            write(output / "verify-style-undo.json", {"passed": True, "capture": str(path)})
        elif phase in ("finish", "close-owned"):
            if phase == "close-owned":
                path = capture(phase)
                require(
                    same(path, root / "workspace/socket-blank.svg"), "drawing not restored to blank"
                )
                close_owned_session(package, output, evidence)
                return
            path = capture(phase)
            require(
                same(path, root / "workspace/socket-blank.svg"), "drawing not restored to blank"
            )
            require(
                all(
                    json.loads((output / name).read_text())["passed"]
                    for name in ("verify-text-undo.json", "verify-style-undo.json")
                ),
                "native Undo evidence missing",
            )
            for label, changed in (
                ("style", True),
                ("style-noop", False),
                ("text", True),
                ("text-noop", False),
            ):
                require(
                    json.loads((output / (label + ".socket.json")).read_text())["changed"]
                    == changed,
                    "native change/no-op evidence missing",
                )
            write(
                output / "acceptance.json",
                {
                    "passed": True,
                    "native_GUI": True,
                    "actual_INX_socket": True,
                    "accumulated_style_insertion_one_step_Undo": True,
                    "style_text_one_step_Undo": True,
                    "no_op_Undo": True,
                    "private_Python_disabled": not (
                        package / "libexec/inkscape-mcp/python/bin/python3"
                    ).exists(),
                    "helper_sha256": hashlib.sha256(
                        (package / "bin/inkscape-mcp-live").read_bytes()
                    ).hexdigest(),
                },
            )
            close_owned_session(package, output, evidence)
    finally:
        write(output / f"{phase}.trace.json", wire.trace)
        wire.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "phase",
        choices=(
            "setup",
            "style",
            "text",
            "verify-text-undo",
            "verify-style-undo",
            "finish",
            "close-owned",
        ),
    )
    args = parser.parse_args()
    main(args.package.resolve(), args.output.resolve(), args.phase)
