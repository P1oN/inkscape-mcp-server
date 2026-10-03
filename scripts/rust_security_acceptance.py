#!/usr/bin/env python3
"""Rust-only packaged STDIO security invariants; no reference server or GUI."""

import argparse
import hashlib
import json
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write


def inventory(root):
    return {
        str(p.relative_to(root)): {
            "mtime": p.stat().st_mtime_ns,
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None,
        }
        for p in [root, *root.rglob("*")]
    }


def main(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    checks = []
    with TemporaryDirectory(prefix="rust-security-") as temporary:
        root = Path(temporary).resolve()
        workspace = root / "workspace"
        workspace.mkdir()
        cases = {
            "malformed": b"<svg><g></svg>",
            "namespace": b"<x:svg/>",
            "depth": ("<svg>" + "<g>" * 300 + "</g>" * 300 + "</svg>").encode(),
        }
        for name, content in cases.items():
            (workspace / (name + ".svg")).write_bytes(content)
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "absent-session"),
            "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
        }
        wire = Wire([str(binary)], env, output / "server.stderr.log")
        try:
            wire.initialize()
            for name in cases:
                before = inventory(workspace)
                reply = wire.call("open_document", {"path": name + ".svg"})
                require(reply["result"].get("isError"), "unsafe XML accepted")
                after = inventory(workspace)
                write(output / (name + "-filesystem.json"), {"before": before, "after": after})
                require(before == after, "refused XML open changed workspace: " + name)
                checks.append(name + " rejected before writes")
            for label, tool, arguments in [
                ("unknown-field", "create_document", {"width": 10, "height": 10, "shell": "x"}),
                ("invalid-number", "create_document", {"width": "NaN", "height": 10}),
                ("missing-path", "open_document", {}),
                (
                    "container-cap",
                    "set_fill",
                    {"doc_id": "none", "object_ids": [None] * 10001, "color": "red"},
                ),
            ]:
                before = inventory(workspace)
                reply = wire.call(tool, arguments)
                require(reply["result"].get("isError"), "invalid arguments accepted: " + label)
                require(before == inventory(workspace), "invalid arguments changed workspace")
                checks.append(label + " rejected before writes")
            doc = data(wire.call("create_document", {"width": 10, "height": 10}))["doc_id"]
            require(doc, "valid operation failed after refusals")
            resources = wire.request("resources/list", {})["result"]["resources"]
            require(len(resources) == 10, "static resource count drift")
            for item in resources:
                reply = wire.request("resources/read", {"uri": item["uri"]})
                require(not reply.get("error"), "static resource read failed")
                contents = reply["result"]["contents"]
                require(len(contents) == 1, "unexpected resource envelope")
                require(contents[0]["uri"] == item["uri"], "resource URI binding mismatch")
                require(contents[0]["mimeType"] == "application/json", "resource MIME mismatch")
                json.loads(contents[0]["text"])
                checks.append("static resource " + item["name"])
            for leaf in ("summary", "tree", "layers", "objects", "styles", "fonts", "assets"):
                uri = f"inkscape://document/{doc}/{leaf}"
                reply = wire.request("resources/read", {"uri": uri})
                require(not reply.get("error"), "document resource failed")
                content = reply["result"]["contents"][0]
                require(content["uri"] == uri, "document resource URI mismatch")
                value = json.loads(content["text"])
                require(value["doc_id"] == doc, "document resource ID mismatch")
                checks.append("document resource " + leaf)
            prompts = wire.request("prompts/list", {})["result"]["prompts"]
            require(len(prompts) == 7, "prompt count drift")
            for item in prompts:
                arguments = {a["name"]: "security audit goal" for a in item.get("arguments", [])}
                reply = wire.request("prompts/get", {"name": item["name"], "arguments": arguments})
                require(not reply.get("error"), "prompt render failed")
                messages = reply["result"]["messages"]
                require(messages, "empty prompt")
                require(
                    all("__MIGRATION_GOAL_SLOT_" not in m["content"]["text"] for m in messages),
                    "unrendered prompt slot",
                )
                checks.append("prompt " + item["name"])
            for uri in (
                f"inkscape://document/{doc}/../../outside",
                "inkscape://document/missing/tree",
                "inkscape://unknown",
            ):
                reply = wire.request("resources/read", {"uri": uri})
                require("error" in reply, "invalid resource URI accepted")
                checks.append("resource refusal " + uri)
            require(not (root / "absent-session").exists(), "startup launched/created session")
            for name, content in cases.items():
                require((workspace / (name + ".svg")).read_bytes() == content, "original changed")
            checks.append("valid create after refusals; originals preserved; no managed GUI")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    write(
        output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "checks": checks,
            "scope": "Selected XML and argument refusals before writes; "
            "not exhaustive security/race/native acceptance",
        },
    )
    print(f"{len(checks)} Rust STDIO security checks passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
