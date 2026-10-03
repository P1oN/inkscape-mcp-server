#!/usr/bin/env python3
"""Owned compare_region second-source refusal and successful pair publication; no GUI."""

import argparse
import base64
import hashlib
import io
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write
from PIL import Image
from rust_security_acceptance import inventory


def main(binary, output):
    binary = binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    with TemporaryDirectory(prefix="imcp-compare-publish-") as temporary:
        root = Path(temporary).resolve()
        workspace = root / "workspace"
        workspace.mkdir()
        safe = (
            '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
            '<rect id="r" width="16" height="16" fill="blue"/></svg>'
        )
        (workspace / "source.svg").write_text(safe)
        Image.new("RGBA", (16, 16), (213, 47, 89, 255)).save(root / "outside.png")
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "absent-session"),
            "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
            "INKSCAPE_MCP_ENGINE_MODE": "per_call",
        }
        wire = Wire([str(binary)], env, output / "server.stderr.log")
        try:
            wire.initialize()
            doc = data(wire.call("open_document", {"path": "source.svg"}))["doc_id"]
            snapshot = data(wire.call("create_snapshot", {"doc_id": doc}))["snapshot_id"]
            working = workspace / ".inkscape-mcp/documents" / doc / "working/document.svg"
            unsafe = safe.replace(
                '<rect id="r" width="16" height="16" fill="blue"/>',
                f'<image href="{root}/outside.png" width="16" height="16"/>',
            )
            working.write_text(unsafe)  # Only this owned synthetic fixture.
            args = {
                "doc_id": doc,
                "snapshot_id": snapshot,
                "region": {"x": 0, "y": 0, "width": 16, "height": 16},
                "width_px": 16,
                "inline": False,
            }
            before = inventory(workspace)
            refused = wire.call("compare_region", args)
            require(refused["result"].get("isError"), "unsafe second source accepted")
            after = inventory(workspace)
            write(output / "refusal-filesystem.json", {"before": before, "after": after})
            require(before == after, "second-source refusal published first image")
            working.write_text(safe.replace("blue", "red"))
            compared = data(wire.call("compare_region", args))
            for key, pixel in [("before", (0, 0, 255, 255)), ("after", (255, 0, 0, 255))]:
                resource = wire.request("resources/read", {"uri": compared[key]["uri"]})
                png = base64.b64decode(resource["result"]["contents"][0]["blob"])
                (output / (key + ".png")).write_bytes(png)
                with Image.open(io.BytesIO(png)) as image:
                    require(image.convert("RGBA").getpixel((8, 8)) == pixel, "wrong pair pixels")
            require((workspace / "source.svg").read_text() == safe, "original source changed")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    write(
        output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "scope": "Unsafe second-source refusal before publication and correct blue/red pair; "
            "not exhaustive filesystem crash/rollback proof",
        },
    )
    print("Compare publication checks passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
