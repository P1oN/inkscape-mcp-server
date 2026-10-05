#!/usr/bin/env python3
"""Verify fixed owned group/ungroup/delete captures; never launch or edit Inkscape."""

import argparse
import hashlib
import json
import shutil
from pathlib import Path

from lxml import etree
from migration_probe import data, require, write
from PIL import Image


def main(output, report, captured_only, order_only=False, group_apply_phase="group-apply"):
    session = json.loads((output / "session.json").read_text())
    workspace = Path(session["env"]["INKSCAPE_MCP_WORKSPACE_ROOTS"])
    saved = output / "structure-captured-artifacts"
    saved.mkdir(exist_ok=True)
    artifacts, checks = {}, {}

    def check(name, condition):
        checks[name] = bool(condition)

    def capture(relative):
        relative = Path(relative)
        require(not relative.is_absolute() and ".." not in relative.parts, "escaped artifact")
        source = workspace / relative
        dest = saved / relative.name
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

    def load(phase):
        scene = json.loads((output / (phase + ".scene.json")).read_text())
        sync = data(json.loads((output / (phase + ".sync.json")).read_text()))
        svg = capture(sync["saved_path"])
        root = etree.fromstring(
            svg.read_bytes(), etree.XMLParser(resolve_entities=False, no_network=True)
        )
        ids = [node.get("id") for node in root.iter() if node.get("id")]
        check(phase + "_unique_ids", len(ids) == len(set(ids)))
        png = capture(scene["render"]["artifact_path"])
        with Image.open(png) as image:
            image = image.convert("RGBA")
            pixels = image.size, image.tobytes()
        return scene["scene"], root, pixels

    def same_scene(first, second):
        return all(
            first[key] == second[key]
            for key in ("tree", "visible_objects", "canvas", "object_count")
        )

    families = (
        (("lower", (7, 7)),)
        if order_only
        else (("group", (7, 8)), ("ungroup", (8, 7)), ("delete", (7, 1)))
    )
    for op, counts in families:
        apply_phase = group_apply_phase if op == "group" else op + "-apply"
        before, before_root, before_pixels = load(apply_phase + "-before")
        after, after_root, after_pixels = load(apply_phase)
        undone, _, undo_pixels = load(op + "-undone")
        redone, _, redo_pixels = load(op + "-redone")
        check(
            op + "_counts",
            (
                before["object_count"],
                after["object_count"],
                undone["object_count"],
                redone["object_count"],
            )
            == (*counts, *counts),
        )
        check(op + "_undo_exact_scene", same_scene(before, undone))
        check(op + "_redo_exact_scene", same_scene(after, redone))
        check(op + "_undo_exact_rgba", before_pixels == undo_pixels)
        check(op + "_redo_exact_rgba", after_pixels == redo_pixels)
        if op != "delete":
            check(op + "_appearance_preserved", before_pixels == after_pixels)
        else:
            check("delete_changes_pixels", before_pixels != after_pixels)
            check("delete_removes_geometry", not after_root.xpath('//*[local-name()="rect"]'))
        reply = data(json.loads((output / (apply_phase + ".reply.json")).read_text()))
        audit = json.loads((output / (apply_phase + ".operations.json")).read_text())
        records = json.loads(audit["result"]["contents"][0]["text"])["operations"]
        matching = [r for r in records if r["operation_id"] == reply["operation_id"]]
        check(
            op + "_applied_audit",
            len(matching) == 1
            and matching[0]["status"] == "applied"
            and matching[0]["affected_ids"] == reply["affected_ids"]
            and not matching[0]["completion_uncertain"],
        )
        if op == "group":
            ident = reply["affected_ids"][0]
            groups = after_root.xpath("//*[@id=$ident]", ident=ident)
            top = [
                n.get("id")
                for n in before_root
                if etree.QName(n).localname == "g"
                and not n.get("{http://www.inkscape.org/namespaces/inkscape}groupmode")
            ]
            check(
                "group_plain_wrapper_and_paint_order",
                len(groups) == 1
                and dict(groups[0].attrib) == {"id": ident}
                and [n.get("id") for n in groups[0]] == top,
            )
        if op == "lower":

            def groups(root):
                return [
                    n
                    for n in root
                    if n.tag == "{http://www.w3.org/2000/svg}g"
                    and not n.get("{http://www.inkscape.org/namespaces/inkscape}groupmode")
                ]

            original, ordered = groups(before_root), groups(after_root)
            check(
                "lower_reverses_two_sibling_groups",
                len(original) == len(ordered) == 2
                and [n.get("id") for n in ordered] == [n.get("id") for n in reversed(original)],
            )
            check(
                "lower_preserves_group_subtrees",
                {n.get("id"): etree.tostring(n, method="c14n", with_tail=False) for n in original}
                == {
                    n.get("id"): etree.tostring(n, method="c14n", with_tail=False) for n in ordered
                },
            )

    final_op = "lower" if order_only else "delete"
    restored, _, pixels = load(final_op + "-restored")
    before, _, expected_pixels = load(final_op + "-apply-before")
    check(final_op + "_final_restoration_scene", same_scene(before, restored))
    check(final_op + "_final_restoration_rgba", pixels == expected_pixels)
    result = {
        "passed": all(checks.values()),
        "checks": checks,
        "artifacts": artifacts,
        "session": session,
        "captured_only": captured_only,
        "order_only": order_only,
        "scope": (
            "One fixed current-package synthetic native group/ungroup/delete sequence "
            "with AX Undo/Redo. Does not establish the root cause of stage18 crash, "
            "other native effects or lost-result/document-switch behavior. "
            "Order-only mode checks lower on two identical overlapping group copies; "
            "it establishes XML paint order and preserved pixels, not differing-color occlusion."
        ),
    }
    write(report, result)
    print(
        json.dumps({"passed": result["passed"], "checks": len(checks), "artifacts": len(artifacts)})
    )
    require(result["passed"], "native structure checks failed; inspect captured evidence")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--captured-only", action="store_true")
    parser.add_argument("--order-only", action="store_true")
    parser.add_argument(
        "--group-apply-phase",
        choices=["group-apply", "group-selection-apply"],
        default="group-apply",
    )
    args = parser.parse_args()
    main(args.output, args.report, args.captured_only, args.order_only, args.group_apply_phase)
