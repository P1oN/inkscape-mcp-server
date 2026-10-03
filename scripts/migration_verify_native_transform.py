#!/usr/bin/env python3
"""Check fixed native compound-transform evidence, without GUI actions or edits."""

import argparse
import hashlib
import json
import math
import re
import shutil
from copy import deepcopy
from pathlib import Path

from lxml import etree
from migration_probe import data, require, write
from PIL import Image


def main(output, report, captured_only):
    session = json.loads((output / "session.json").read_text())
    workspace = Path(session["env"]["INKSCAPE_MCP_WORKSPACE_ROOTS"])
    saved = output / "transform-captured-artifacts"
    saved.mkdir(exist_ok=True)
    artifacts, checks, samples = {}, {}, {}

    def capture(relative):
        relative = Path(relative)
        require(not relative.is_absolute() and ".." not in relative.parts, "escaped artifact")
        source, dest = workspace / relative, saved / relative.name
        if not captured_only:
            require(source.is_file() and not source.is_symlink(), "missing/linked native artifact")
            if dest.exists():
                require(dest.read_bytes() == source.read_bytes(), "capture name collision")
            else:
                shutil.copyfile(source, dest)
        require(dest.is_file() and not dest.is_symlink(), "missing/linked saved artifact")
        content = dest.read_bytes()
        artifacts[dest.name] = {
            "bytes": len(content),
            "sha256": hashlib.sha256(content).hexdigest(),
        }
        return dest

    for phase in ("apply-before", "apply", "undone", "redone", "restored"):
        label = "transform-" + phase
        scene = json.loads((output / (label + ".scene.json")).read_text())
        synced = data(json.loads((output / (label + ".sync.json")).read_text()))
        root = etree.fromstring(
            capture(synced["saved_path"]).read_bytes(),
            etree.XMLParser(resolve_entities=False, no_network=True),
        )
        ids = [n.get("id") for n in root.iter() if n.get("id")]
        checks[label + "_unique_ids"] = len(ids) == len(set(ids))
        with Image.open(capture(scene["render"]["artifact_path"])) as image:
            image = image.convert("RGBA")
            pixels = image.size, image.tobytes()
        samples[phase] = root, scene["scene"], ids, pixels

    def drawing(root):
        root = deepcopy(root)
        for node in list(root):
            if node.tag == "{http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd}namedview":
                root.remove(node)
        return etree.tostring(root, method="c14n")

    before, after = samples["apply-before"], samples["apply"]
    for phase, expected in (("undone", before), ("redone", after), ("restored", before)):
        actual = samples[phase]
        checks[phase + "_exact_drawing"] = drawing(actual[0]) == drawing(expected[0])
        checks[phase + "_exact_rgba"] = actual[3] == expected[3]
        checks[phase + "_exact_scene"] = all(
            actual[1][key] == expected[1][key]
            for key in ("tree", "visible_objects", "canvas", "object_count")
        )
    checks["object_counts_unchanged"] = all(s[1]["object_count"] == 7 for s in samples.values())
    checks["IDs_and_paint_order_preserved"] = before[2] == after[2]
    checks["pixels_changed"] = before[3] != after[3]
    reply = data(json.loads((output / "transform-apply.reply.json").read_text()))
    checks["two_objects_transformed"] = len(reply["affected_ids"]) == 2
    restored = deepcopy(after[0])
    expected_matrix = (
        1.25 * math.cos(math.radians(15)),
        1.25 * math.sin(math.radians(15)),
        -1.25 * math.sin(math.radians(15)),
        1.25 * math.cos(math.radians(15)),
        20,
        15,
    )
    matrices = {}
    for ident in reply["affected_ids"]:
        nodes = restored.xpath("//*[@id=$ident]", ident=ident)
        require(len(nodes) == 1, "transformed ID missing")
        raw = nodes[0].attrib.pop("transform", "")
        require(re.fullmatch(r"matrix\([0-9.eE+\- ,]+\)", raw) is not None, "unexpected matrix")
        values = [float(v) for v in re.split(r"[ ,]+", raw[7:-1])]
        matrices[ident] = {"raw": raw, "values": values}
        checks[ident + "_expected_affine"] = len(values) == 6 and all(
            math.isclose(actual, expected, rel_tol=0, abs_tol=5e-6)
            for actual, expected in zip(values, expected_matrix, strict=True)
        )
    checks["only_selected_transforms_changed"] = drawing(restored) == drawing(before[0])
    audit = json.loads((output / "transform-apply.operations.json").read_text())
    records = json.loads(audit["result"]["contents"][0]["text"])["operations"]
    matches = [r for r in records if r["operation_id"] == reply["operation_id"]]
    checks["applied_audit"] = (
        len(matches) == 1
        and matches[0]["status"] == "applied"
        and matches[0]["affected_ids"] == reply["affected_ids"]
        and not matches[0]["completion_uncertain"]
    )
    result = {
        "passed": all(checks.values()),
        "checks": checks,
        "artifacts": artifacts,
        "matrices": matrices,
        "expected_matrix": expected_matrix,
        "matrix_absolute_tolerance": 5e-6,
        "session": session,
        "captured_only": captured_only,
        "scope": (
            "One compound translate(20,15), scale(1.25), rotate(15) on two owned groups "
            "and native Undo/Redo/restoration. Drawing C14N excludes only sodipodi:namedview "
            "GUI metadata; raw SVGs retained. Affine tolerance covers six-digit SVG output. "
            "No document-switch, arbitrary-parent-transform or performance claim."
        ),
    }
    write(report, result)
    print(
        json.dumps({"passed": result["passed"], "checks": len(checks), "artifacts": len(artifacts)})
    )
    require(result["passed"], "native transform verification failed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--captured-only", action="store_true")
    args = parser.parse_args()
    main(args.output, args.report, args.captured_only)
