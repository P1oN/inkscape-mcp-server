#!/usr/bin/env python3
"""Owned-file renderer boundary probe. No GUI, network or user assets.

Use --require-isolation to turn outside reads into a failing regression gate.
In-workspace cases also verify linked SVG export/reopen/render, including URI-reserved names.
"""

import argparse
import hashlib
import os
from pathlib import Path
from tempfile import TemporaryDirectory
from urllib.parse import quote

from migration_probe import Wire, data, require, write
from PIL import Image
from rust_security_acceptance import inventory


def probe(binary, output, require_isolation=False, engine_mode="per_call"):
    binary = binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    observations = []
    with TemporaryDirectory(prefix="imcp-render-probe-") as temporary:
        root = Path(temporary).resolve()
        workspace = root / "workspace"
        workspace.mkdir()
        outside = root / "owned-outside.png"
        inside = workspace / "owned-inside.png"
        marker = (213, 47, 89, 255)
        for asset in (outside, inside):
            Image.new("RGBA", (16, 16), marker).save(asset)
        (workspace / "owned-link.png").symlink_to(outside)
        (workspace / "assets").mkdir()
        (workspace / "assets/leaf.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
            '<image href="../owned-inside.png" width="16" height="16"/></svg>'
        )
        (workspace / "assets/unsafe.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
            f'<image href="{outside}" width="16" height="16"/></svg>'
        )
        shape = (
            '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
            '<rect id="marker" width="16" height="16" fill="rgb(213,47,89)"/></svg>'
        )
        (workspace / "assets/shape.svg").write_text(shape)
        (root / "outside.svg").write_text(shape)
        (workspace / "assets/paint.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="paint">'
            '<stop stop-color="rgb(213,47,89)"/></linearGradient></defs></svg>'
        )
        (root / "outside-paint.svg").write_bytes((workspace / "assets/paint.svg").read_bytes())
        (workspace / "assets/style.css").write_text(".mark { fill:#d52f59; }")
        (workspace / "assets/sub").mkdir()
        (workspace / "assets/sub/nested.css").write_text('@import "../style.css";')
        (root / "outside.css").write_text(".mark { fill:#d52f59; }")
        (workspace / "assets/unsafe.css").write_text(f'@import "{root}/outside.css";')
        # Literal URI syntax in filesystem names must survive SVG export/reopen.
        reserved_image = workspace / "owned#%? &é.png"
        Image.new("RGBA", (16, 16), marker).save(reserved_image)
        (workspace / "assets/shape#%?.svg").write_text(shape)
        (workspace / "assets/paint#%?.svg").write_bytes(
            (workspace / "assets/paint.svg").read_bytes()
        )
        (workspace / "assets/style#%?.css").write_text(".mark { fill:#d52f59; }")
        (workspace / "assets/leaf#%?.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
            f'<image href="../{quote(reserved_image.name)}" width="16" height="16"/></svg>'
        )
        before = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in (outside, inside)}
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "absent-session"),
            "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
            "INKSCAPE_MCP_ENGINE_MODE": engine_mode,
        }
        wire = Wire([str(binary)], env, output / "server.stderr.log")
        try:
            wire.initialize()
            cases = [
                ("outside-absolute", str(outside), True),
                ("outside-file-uri", outside.as_uri(), True),
                ("outside-symlink", str(workspace / "owned-link.png"), True),
                ("outside-nested-svg", "assets/unsafe.svg", True),
                ("inside-absolute", str(inside), False),
                ("inside-relative", "owned-inside.png", False),
                ("inside-nested-svg", "assets/leaf.svg", False),
                ("outside-feimage", str(outside), True),
                ("outside-use", str(root / "outside.svg") + "#marker", True),
                ("inside-feimage", "owned-inside.png", False),
                ("inside-use", "assets/shape.svg#marker", False),
                ("inside-css", "assets/paint.svg#paint", False),
                ("outside-css", str(root / "outside-paint.svg") + "#paint", True),
                ("inside-import", "assets/style.css", False),
                ("inside-nested-import", "assets/sub/nested.css", False),
                ("outside-import", str(root / "outside.css"), True),
                ("outside-nested-import", "assets/unsafe.css", True),
                ("inside-reserved-image", quote(reserved_image.name), False),
                ("inside-reserved-use", quote("assets/shape#%?.svg") + "#marker", False),
                ("inside-reserved-css", quote("assets/paint#%?.svg") + "#paint", False),
                ("inside-reserved-import", quote("assets/style#%?.css"), False),
                ("inside-reserved-nested-svg", quote("assets/leaf#%?.svg"), False),
            ]
            for name, href, outside_reference in cases:
                if name.endswith("-import"):
                    body = (
                        f"<style>@import &quot;{href}&quot;;</style>"
                        '<rect class="mark" width="16" height="16" fill="blue"/>'
                    )
                elif name.endswith("-css"):
                    body = f'<rect width="16" height="16" style="fill:url({href})"/>'
                elif name.endswith("-feimage"):
                    body = (
                        f'<defs><filter id="f" x="0" y="0" width="1" height="1">'
                        f'<feImage xlink:href="{href}"/></filter></defs>'
                        '<rect width="16" height="16" filter="url(#f)"/>'
                    )
                elif name.endswith("-use"):
                    body = f'<use xlink:href="{href}"/>'
                else:
                    body = f'<image xlink:href="{href}" width="16" height="16"/>'
                svg = (
                    '<svg xmlns="http://www.w3.org/2000/svg" '
                    'xmlns:xlink="http://www.w3.org/1999/xlink" width="16" height="16">'
                    f"{body}</svg>"
                )
                (workspace / f"{name}.svg").write_text(svg)
                doc = data(wire.call("open_document", {"path": f"{name}.svg"}))["doc_id"]
                before_render = inventory(workspace)
                reply = wire.call(
                    "render_preview", {"doc_id": doc, "width_px": 16, "inline": False}
                )
                row = {
                    "case": name,
                    "outside_reference": outside_reference,
                    "refused": reply["result"].get("isError", False),
                    "marker_rendered": False,
                }
                if not row["refused"]:
                    value = data(reply)
                    rendered = workspace / value["artifact_path"]
                    (output / f"{name}.png").write_bytes(rendered.read_bytes())
                    with Image.open(rendered) as image:
                        pixel = image.convert("RGBA").getpixel((8, 8))
                    row["pixel"] = list(pixel)
                    row["marker_rendered"] = pixel == marker
                if outside_reference and require_isolation:
                    require(row["refused"], "outside asset must be explicitly refused")
                    require(before_render == inventory(workspace), "refused render published files")
                    row["refusal_preserved_workspace"] = True
                if not outside_reference:
                    require(row["marker_rendered"], "in-workspace asset rendering failed")
                    export_args = {"doc_id": doc, "format": "svg", "inline": False}
                    if "reserved" in name:
                        export_args["out_dir"] = "exports#%? &é"
                    exported = data(wire.call("export_document", export_args))
                    exported_bytes = (workspace / exported["artifact_path"]).read_bytes()
                    (output / f"{name}-export.svg").write_bytes(exported_bytes)
                    require(
                        b"imcp-assets-" not in exported_bytes,
                        "staged asset path leaked into export",
                    )
                    require(
                        b"data:image" not in exported_bytes, "linked artwork replaced by embedding"
                    )
                    (output / f"{name}-export.svg").write_bytes(exported_bytes)
                    row["svg_export_links_restored"] = True
                    reopened = data(
                        wire.call("open_document", {"path": exported["artifact_path"]})
                    )["doc_id"]
                    rerendered = data(
                        wire.call(
                            "render_preview",
                            {"doc_id": reopened, "width_px": 16, "inline": False},
                        )
                    )
                    with Image.open(workspace / rerendered["artifact_path"]) as image:
                        require(
                            image.convert("RGBA").getpixel((8, 8)) == marker,
                            "exported SVG asset did not survive reopen/render",
                        )
                    row["svg_export_reopened_and_rendered"] = True
                observations.append(row)
            require(
                all(
                    hashlib.sha256(p.read_bytes()).hexdigest() == digest
                    for p, digest in before.items()
                ),
                "original asset changed",
            )
            require(not (root / "absent-session").exists(), "managed session created")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    violations = [
        r["case"] for r in observations if r["outside_reference"] and r["marker_rendered"]
    ]
    report = {
        "probe_completed": True,
        "engine_mode": engine_mode,
        "isolation_passed": not violations,
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "observations": observations,
        "outside_reads": violations,
        "original_assets_unchanged": True,
        "scope": (
            "Owned image/use/feImage and CSS URLs/imports, nested dependencies and SVG export; "
            "no network/GUI/user assets. Does not cover xml:base or every engine route."
        ),
    }
    write(output / "probe.json", report)
    print(f"Renderer probe completed: {len(violations)} outside-workspace reads")
    if require_isolation:
        require(not violations, "renderer read assets outside workspace: " + ", ".join(violations))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--require-isolation", action="store_true")
    parser.add_argument("--engine-mode", choices=("per_call", "shell"), default="per_call")
    args = parser.parse_args()
    probe(args.binary, args.output, args.require_isolation, args.engine_mode)
