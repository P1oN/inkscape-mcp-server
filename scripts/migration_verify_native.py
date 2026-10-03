#!/usr/bin/env python3
"""Verify captured owned native GUI acceptance without launching or changing Inkscape."""

import argparse
import hashlib
import json
import shutil
from pathlib import Path

from lxml import etree
from migration_probe import data, require, write
from PIL import Image


def main(output, report, captured_only=False):
    session = json.loads((output / "session.json").read_text())
    workspace = Path(session["env"]["INKSCAPE_MCP_WORKSPACE_ROOTS"])
    phases = [
        "insert",
        "undone",
        "redone",
        "style-repeat",
        "style-undone",
        "style-redone",
        "duplicate",
        "duplicate-undone",
        "duplicate-redone",
        "guarded",
    ]
    scenes = {phase: json.loads((output / (phase + ".scene.json")).read_text()) for phase in phases}
    replies = {
        phase: data(json.loads((output / (phase + ".json")).read_text()))
        for phase in ("insert", "style-changed", "style-repeated", "duplicate")
    }
    checks = {}
    artifacts = {}
    saved = output / "captured-artifacts"
    saved.mkdir(exist_ok=True)

    def check(name, condition):
        checks[name] = bool(condition)
        require(condition, name)

    def record(path):
        target = saved / path.name
        if captured_only:
            require(target.is_file() and not target.is_symlink(), "missing/linked saved artifact")
            artifacts[path.name] = {
                "sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
                "bytes": target.stat().st_size,
            }
            return target
        require(path.is_file() and not path.is_symlink(), "missing/linked native artifact")
        if target.exists():
            require(target.read_bytes() == path.read_bytes(), "capture name collision")
        else:
            shutil.copyfile(path, target)
        artifacts[path.name] = {
            "sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
            "bytes": target.stat().st_size,
        }
        return target

    def rgba(relative):
        require(
            not Path(relative).is_absolute() and ".." not in Path(relative).parts,
            "escaped artifact",
        )
        path = record(workspace / relative)
        with Image.open(path) as image:
            image = image.convert("RGBA")
            return image.size, image.tobytes()

    def pixels(phase):
        return rgba(scenes[phase]["render"]["artifact_path"])

    def tree(phase):
        return scenes[phase]["scene"]["tree"]

    audit_phases = ["insert", "style-changed", "style-repeated", "duplicate"]
    audit_presence = [(output / (phase + ".operations.json")).is_file() for phase in audit_phases]
    require(not any(audit_presence) or all(audit_presence), "incomplete immediate audit evidence")
    if all(audit_presence):
        for phase in audit_phases:
            resource = json.loads((output / (phase + ".operations.json")).read_text())
            operations = json.loads(resource["result"]["contents"][0]["text"])["operations"]
            reply = replies[phase]
            audit_records = [
                item for item in operations if item["operation_id"] == reply["operation_id"]
            ]
            check(
                phase + "_immediate_applied_audit",
                len(audit_records) == 1
                and audit_records[0]["status"] == "applied"
                and audit_records[0]["affected_ids"] == reply["affected_ids"]
                and audit_records[0]["previews"]
                == {"before": reply["preview_before"], "after": reply["preview_after"]},
            )
        check(
            "approved_repeat_has_distinct_audit",
            replies["style-changed"]["operation_id"] != replies["style-repeated"]["operation_id"],
        )

    def ids(node):
        return [node["id"]] + [ident for child in node["children"] for ident in ids(child)]

    wrapper, rectangle = replies["insert"]["affected_ids"]
    inserted = next(child for child in tree("insert")["children"] if child["id"] == wrapper)
    check("native_anonymous_group_id_confirmed", inserted["children"][0]["id"] == "g1")
    check(
        "named_semantic_group",
        inserted["children"][0]["label"] == "Native acceptance"
        and not inserted["children"][0]["is_layer"],
    )
    check(
        "rectangle_geometry_and_fill",
        inserted["children"][0]["children"][0]["id"] == rectangle
        and inserted["children"][0]["children"][0]["bbox"]
        == {"x": 20.0, "y": 20.0, "width": 100.0, "height": 60.0}
        and inserted["children"][0]["children"][0]["paint"]["fill"] == "#1464dc",
    )
    blank = dict(
        tree("insert"),
        children=[child for child in tree("insert")["children"] if child["id"] != wrapper],
    )
    check(
        "one_insert_Undo_removes_entire_group",
        tree("undone") == blank and not set(ids(inserted)) & set(ids(tree("undone"))),
    )
    check("insert_Redo_exact_tree", tree("redone") == tree("insert"))
    check(
        "insert_Undo_exact_before_RGBA",
        pixels("undone") == rgba(replies["insert"]["preview_before"]),
    )
    check("insert_Redo_exact_RGBA", pixels("redone") == pixels("insert"))
    check("insert_has_visible_effect", pixels("insert") != pixels("undone"))
    check(
        "repeated_style_identical_RGBA",
        rgba(replies["style-changed"]["preview_after"])
        == rgba(replies["style-repeated"]["preview_before"])
        == rgba(replies["style-repeated"]["preview_after"]),
    )
    styled = next(child for child in tree("style-repeat")["children"] if child["id"] == wrapper)
    check(
        "style_changes_selected_wrapper",
        styled == dict(inserted, paint=dict(inserted["paint"], fill="#dc6414")),
    )
    check("explicit_child_fill_preserves_RGBA", pixels("style-repeat") == pixels("insert"))
    check(
        "one_style_Undo_no_extra_repeat_step",
        tree("style-undone") == tree("insert") and pixels("style-undone") == pixels("insert"),
    )
    check(
        "style_Redo_exact_tree_RGBA",
        tree("style-redone") == tree("style-repeat")
        and pixels("style-redone") == pixels("style-repeat"),
    )
    added = set(ids(tree("duplicate"))) - set(ids(tree("style-redone")))
    check(
        "duplicate_three_unique_new_ids",
        added == set(replies["duplicate"]["affected_ids"])
        and len(added) == 3
        and len(ids(tree("duplicate"))) == len(set(ids(tree("duplicate")))),
    )
    check(
        "duplicate_original_preserved",
        next(child for child in tree("duplicate")["children"] if child["id"] == wrapper)
        == next(child for child in tree("style-redone")["children"] if child["id"] == wrapper),
    )
    check(
        "one_duplicate_Undo_exact_tree_RGBA",
        tree("duplicate-undone") == tree("style-redone")
        and pixels("duplicate-undone") == pixels("style-redone"),
    )
    check(
        "duplicate_Redo_exact_tree_RGBA",
        tree("duplicate-redone") == tree("duplicate")
        and pixels("duplicate-redone") == pixels("duplicate"),
    )
    for phase, message in [
        ("unbound-guard", "live_select_document"),
        ("approval-guard", "approval"),
    ]:
        reply = json.loads((output / (phase + ".json")).read_text())["result"]
        check(phase + "_refused", reply.get("isError") and message in reply["content"][0]["text"])
    check(
        "guards_preserve_tree_RGBA",
        tree("guarded") == tree("duplicate-redone")
        and pixels("guarded") == pixels("duplicate-redone"),
    )
    for phase in phases:
        trace = json.loads((output / (phase + ".trace.json")).read_text())
        connected = [
            data(item["response"])
            for item in trace
            if item.get("request", {}).get("method") == "tools/call"
            and item["request"].get("params", {}).get("name") == "live_connect"
        ]
        check(
            phase + "_reconnect_requires_binding",
            len(connected) == 1 and connected[0]["connected"] and not connected[0]["ready_to_edit"],
        )
        source = record(workspace / ("native-" + phase + ".svg"))
        root = etree.fromstring(
            source.read_bytes(), etree.XMLParser(resolve_entities=False, no_network=True)
        )
        check(phase + "_editable_vectors_only", not root.xpath("//*[local-name()='image']"))
    check(
        "vendor_executable_unchanged",
        hashlib.sha256(
            Path("/Applications/Inkscape.app/Contents/MacOS/inkscape").read_bytes()
        ).hexdigest()
        == session["vendor_executable_sha256"],
    )
    write(
        report,
        {
            "passed": True,
            "native_GUI": True,
            "package": session["package"],
            "source": str(output),
            "immediate_audit_captured": all(audit_presence),
            "verification_mode": "captured artifacts" if captured_only else "owned workspace",
            "checks": checks,
            "artifacts": artifacts,
            "scope": (
                "One owned blank synthetic document; fixed insertion, wrapper style repeat, "
                "duplicate, Undo/Redo, reconnect and approval/binding guards. "
                "The explicit blue child fill overrides the changed orange wrapper fill. "
                "Not all live effects or failure scenarios."
            ),
        },
    )
    print(json.dumps({"passed": True, "checks": len(checks), "artifacts": len(artifacts)}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--captured-only", action="store_true")
    args = parser.parse_args()
    main(args.output, args.report, args.captured_only)
