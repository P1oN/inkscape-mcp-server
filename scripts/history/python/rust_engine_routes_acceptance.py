#!/usr/bin/env python3
"""Owned packaged asset checks across bbox/fit/CLI/preview/capture routes; no GUI."""

import argparse
import hashlib
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write
from PIL import Image
from rust_security_acceptance import inventory


def main(binary, output, engine_mode="per_call"):
    binary = binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    with TemporaryDirectory(prefix="imcp-engine-routes-") as temporary:
        root = Path(temporary).resolve()
        workspace = root / "workspace"
        workspace.mkdir()
        Image.new("RGBA", (16, 16), (213, 47, 89, 255)).save(workspace / "tile.png")
        Image.new("RGBA", (16, 16), (213, 47, 89, 255)).save(root / "outside.png")
        svg = (
            '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="16">'
            '<image id="img" href="tile.png" width="16" height="16"/>'
            '<path id="p" d="M20,0 L28,0 L28,16 L20,16 Z" fill="blue"/></svg>'
        )
        (workspace / "safe.svg").write_text(svg)
        (workspace / "unsafe.svg").write_text(svg.replace("tile.png", str(root / "outside.png")))
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_ENGINE_MODE": engine_mode,
            "INKSCAPE_MCP_RAW_ACTION_ENABLED": "true",
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "absent-session"),
            "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
        }
        wire = Wire([str(binary)], env, output / "server.stderr.log")
        try:
            wire.initialize()
            doc = data(wire.call("open_document", {"path": "safe.svg"}))["doc_id"]
            working = workspace / ".inkscape-mcp/documents" / doc / "working/document.svg"
            for tool, args in [
                ("find_objects", {"accurate_bbox": True}),
                ("fit_to_content", {}),
                (
                    "cleanup_paths",
                    {"object_ids": ["p"], "dry_run": False, "approval_token": "owned fixture"},
                ),
                ("capture_frame", {"width_px": 32, "inline": False}),
            ]:
                result = data(wire.call(tool, {"doc_id": doc, **args}))
                require(result, "empty result: " + tool)
                if tool == "find_objects":
                    path = next(item for item in result["objects"] if item["object_id"] == "p")
                    require(path["bbox"] is not None, "accurate bbox did not use engine")
                checks.append(tool + " accepted linked workspace asset")
            require(b"tile.png" in working.read_bytes(), "original linked asset was lost")
            require(b"imcp-assets-" not in working.read_bytes(), "private path in working SVG")
            preview = data(
                wire.call("render_preview", {"doc_id": doc, "width_px": 32, "inline": False})
            )
            with Image.open(workspace / preview["artifact_path"]) as image:
                require(
                    image.convert("RGBA").getpixel((4, 4)) == (213, 47, 89, 255),
                    "asset pixels lost",
                )
            checks.append("linked image remains correct after fit and CLI mutation")
            bad = data(wire.call("open_document", {"path": "unsafe.svg"}))["doc_id"]
            for tool, args in [
                ("find_objects", {"accurate_bbox": True}),
                ("capture_frame", {"width_px": 32, "inline": False}),
                ("export_document", {"format": "png", "out_dir": "new-output"}),
            ]:
                before = inventory(workspace)
                reply = wire.call(tool, {"doc_id": bad, **args})
                if tool == "find_objects":
                    value = data(reply)
                    path = next(item for item in value["objects"] if item["object_id"] == "p")
                    require(path["bbox"] is None, "unsafe accurate query did not fall back to DOM")
                else:
                    require(reply["result"].get("isError"), "outside dependency accepted: " + tool)
                after = inventory(workspace)
                write(output / (tool + "-filesystem.json"), {"before": before, "after": after})
                require(before == after, "refused " + tool + " changed workspace")
                checks.append(
                    tool
                    + (
                        " safe DOM fallback before writes"
                        if tool == "find_objects"
                        else " outside dependency refused before writes"
                    )
                )
            require((workspace / "safe.svg").read_text() == svg, "original SVG changed")
            require(not (root / "absent-session").exists(), "managed GUI created")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    write(
        output / "comparison.json",
        {
            "passed": True,
            "engine_mode": engine_mode,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "checks": checks,
            "scope": "Selected linked-resource routes; no GUI or exhaustive route proof",
        },
    )
    print(f"{len(checks)} engine route checks passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--engine-mode", choices=("per_call", "shell"), default="per_call")
    args = parser.parse_args()
    main(args.binary, args.output, args.engine_mode)
