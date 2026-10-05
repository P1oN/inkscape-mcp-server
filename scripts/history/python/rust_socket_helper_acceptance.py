#!/usr/bin/env python3
"""Exercise the fixed Rust socket helper in an owned directory, without a GUI."""

import argparse
import base64
import hashlib
import io
import json
import shutil
import socket
import subprocess
import time
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import require, write
from PIL import Image

SVG = """<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="0 0 100 80">
<style>path {fill:#00ff00}</style>
<g id="g" transform="translate(30,20)">
<rect id="r" x="2" y="3" width="20" height="10"
 style="fill:red;stroke:black;stroke-width:4" transform="translate(5,7)"/></g>
<text id="t" x="5" y="65">Hi</text>
<path id="p" d="M0,0 C0,20 20,20 20,0Z"/></svg>"""


def exercise(
    binary, root, env, *, change=False, unauthorized=False, drawing=SVG, bbox=(7, 10, 20, 10)
):
    source = root / "input.svg"
    source.write_text(drawing)
    rendezvous = root / "rendezvous.json"
    process = subprocess.Popen(
        [str(binary), "--id=r", str(source)],
        env={**env, "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(rendezvous)},
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    conn = None
    try:
        deadline = time.monotonic() + 10
        while not rendezvous.exists() and time.monotonic() < deadline:
            require(process.poll() is None, "helper exited before readiness")
            time.sleep(0.01)
        info = json.loads(rendezvous.read_text())
        require(
            info["pid"] == process.pid and info["protocol_version"] == 5, "wrong helper identity"
        )
        require(rendezvous.stat().st_mode & 0o777 == 0o600, "rendezvous permissions")
        conn = socket.create_connection(("127.0.0.1", info["port"]), timeout=40)
        reader = conn.makefile("rb")
        trace = []

        def request(cmd, params=None, token=None):
            msg = {"v": 5, "cmd": cmd, "token": token or info["token"], "params": params or {}}
            conn.sendall(json.dumps(msg).encode() + b"\n")
            reply = json.loads(reader.readline(64 * 1024 * 1024))
            trace.append({"command": cmd, "ok": reply["ok"]})
            return reply

        hello = request("hello", token="wrong" if unauthorized else None)
        if unauthorized:
            require(not hello["ok"], "invalid token accepted")
        else:
            require(
                hello["ok"] and len(hello["result"]["capabilities"]) == 13, "wrong capabilities"
            )
            require(request("get_selection")["result"]["object_ids"] == ["r"], "wrong selection")
            require(
                request("inspect_selection")["result"]["objects"][0]["has_style"],
                "style perception",
            )
            require(request("get_active_document")["result"]["object_count"] == 4, "object count")
            before = request("get_document_svg")["result"]["svg"]
            revision = request("get_state_token")["result"]["revision"]
            scene = request("get_scene")["result"]
            require(
                scene["selection"][0]["bbox"] == (list(bbox) if bbox is not None else None),
                "local geometric bbox includes ancestor/stroke or lost own transform",
            )
            require(scene["viewport"]["zoom"] is None, "invented viewport")
            for params in ({}, {"region": [0, 0, 25, 20], "scale": 2}):
                png = base64.b64decode(request("render_view", params)["result"]["png_base64"])
                require(png.startswith(b"\x89PNG\r\n\x1a\n"), "render PNG")
                if params:
                    require(int.from_bytes(png[16:20], "big") == 50, "region scale")
                elif bbox is not None:
                    with Image.open(io.BytesIO(png)) as image:
                        require(
                            image.convert("RGBA").getpixel((10, 5)) == (0, 255, 0, 255),
                            "stylesheet paint lost",
                        )
            require(request("export_selection")["ok"], "selection export")
            require(
                not request("set_viewport", {"mode": "fit_page"})["result"]["applied"],
                "viewport mutated",
            )
            require(not request("arbitrary", {"code": "x"})["ok"], "unknown command accepted")
            require(
                not request("render_view", {"region": [0, 0, 1e8, 20]})["ok"], "render bound bypass"
            )
            require(request("apply_to_selection", {"style": {"fill": "red"}})["ok"], "no-op style")
            require(request("get_document_svg")["result"]["svg"] == before, "no-op serialization")
            require(request("get_state_token")["result"]["revision"] == revision, "no-op revision")
            require(
                not request("apply_to_selection", {"transform": "translate(NaN)"})["ok"],
                "unsafe transform",
            )
            if change:
                require(
                    request("apply_to_selection", {"style": {"fill": "blue"}})["ok"], "style edit"
                )
                require(
                    request("get_state_token")["result"]["revision"] != revision, "change token"
                )
                require(
                    request("insert_svg", {"svg": "<circle id='c' cx='40' cy='40' r='5'/>"})["ok"],
                    "insertion",
                )
                require(
                    not request("insert_svg", {"svg": "<image href='file:///secret'/>"})["ok"],
                    "unsafe insert",
                )
            # Buffered frames must not be discarded if two requests share one TCP write.
            msg = {"v": 5, "cmd": "ping", "token": info["token"], "params": {}}
            conn.sendall((json.dumps(msg) + "\n").encode() * 2)
            require(
                all(json.loads(reader.readline())["ok"] for _ in range(2)), "coalesced frames lost"
            )
        reader.close()
        conn.close()
        conn = None
        stdout, stderr = process.communicate(timeout=10)
        require(process.returncode == 0 and not stderr, "helper failed on disconnect")
        require(bool(stdout) == change, "unexpected SVG publication/no-op")
        if change:
            require(b"blue" in stdout and b"mcp_" in stdout, "acknowledged edits lost")
        require(
            source.read_text() == drawing and not rendezvous.exists(),
            "input changed or rendezvous leaked",
        )
        return trace
    finally:
        if conn:
            conn.close()
        if process.poll() is None:
            process.terminate()  # exact owned helper only, never a GUI
            process.communicate(timeout=5)


def main(binary, output):
    output.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory(prefix="imcp-socket-acceptance-") as tmp:
        root = Path(tmp).resolve()
        if not (binary.parent.parent / "libexec/inkscape-mcp/package.json").is_file():
            package = root / "relocated"
            (package / "bin").mkdir(parents=True)
            (package / "libexec/inkscape-mcp").mkdir(parents=True)
            shutil.copy2(binary, package / "bin/inkscape-mcp-live")
            (package / "libexec/inkscape-mcp/package.json").write_text("{}")
            binary = package / "bin/inkscape-mcp-live"
        env = {"HOME": str(root), "PATH": "", "LANG": "en_US.UTF-8", "TMPDIR": str(root)}
        traces = [
            exercise(binary, root, env, unauthorized=True),
            exercise(binary, root, env),
            exercise(binary, root, env, change=True),
            exercise(
                binary,
                root,
                env,
                drawing=SVG.replace("path {fill:#00ff00}", "#r {transform:translate(1px)}"),
                bbox=None,
            ),
        ]
        # Refuse linked rendezvous/ancestors; do not modify the target or input.
        original = root / "original"
        original.write_text("original")
        link = root / "linked.json"
        link.symlink_to(original)
        refused = subprocess.run(
            [str(binary), str(root / "input.svg")],
            env={**env, "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(link)},
            capture_output=True,
            timeout=10,
        )
        require(
            refused.returncode != 0 and not refused.stdout and original.read_text() == "original",
            "linked rendezvous followed",
        )
        write(
            output / "acceptance.json",
            {
                "passed": True,
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "empty_PATH": True,
                "Python_invoked_by_runtime": False,
                "native_GUI": False,
                "traces": traces,
            },
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    main(args.binary.resolve(), args.output)
