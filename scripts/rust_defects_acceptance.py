#!/usr/bin/env python3
"""Exercise reference-safe deletion and failed registration through real Rust STDIO."""

import argparse
import json
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require


def main(binary, output):
    binary = binary.resolve(strict=True)
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    with TemporaryDirectory(prefix="imcp-defects-") as temporary:
        root = Path(temporary)
        # Fixed inert engine prevents GUI/CLI rendering; this checks DOM and wire invariants.
        vendor = root / "vendor"
        vendor.mkdir()
        engine = vendor / "inkscape"
        engine.write_text("#!/bin/bash\nexit 1\n")
        engine.chmod(0o700)
        env = {
            **os.environ,
            "PATH": str(vendor),
            "SENTRY_DSN": "",
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_TOOL_DESC": "full",
        }
        workspace = root / "workspace"
        workspace.mkdir()
        original = (
            b'<svg xmlns="http://www.w3.org/2000/svg"><g id="g"><rect id="r"/></g>'
            b'<use id="u" href="#r"/><animate id="a" begin="r.click+1.5s"/></svg>'
        )
        source = workspace / "fixture.svg"
        source.write_bytes(original)
        wire = Wire(
            [str(binary)],
            {**env, "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace)},
            root / "wire.log",
        )
        try:
            instructions = wire.initialize()["result"]["instructions"]
            require("client-trust boundary" in instructions, "approval trust boundary missing")
            doc = data(wire.call("open_document", {"path": "fixture.svg"}))["doc_id"]
            working = workspace / ".inkscape-mcp/documents" / doc / "working/document.svg"
            before = working.read_bytes()
            args = {"doc_id": doc, "object_ids": ["g"], "approval_token": "client-confirmed-test"}
            refused = wire.call("delete_object", args)["result"]
            require(
                refused.get("isError") and working.read_bytes() == before,
                "referenced subtree was deleted",
            )
            checks.append("referenced subtree refuses without working-copy mutation")
            refused = wire.call(
                "apply_edits",
                {
                    "doc_id": doc,
                    "approval_token": "client-confirmed-test",
                    "edits": [
                        {"op": "set_fill", "object_ids": ["r"], "color": "blue"},
                        {"op": "delete_object", "object_ids": ["g"]},
                    ],
                },
            )["result"]
            require(
                refused.get("isError") and working.read_bytes() == before,
                "batch reference refusal did not roll back",
            )
            checks.append("batch reference refusal rolls back earlier staged edit")
            denied = wire.call("delete_object", {"doc_id": doc, "object_ids": ["g", "u"]})["result"]
            require(
                denied.get("isError") and working.read_bytes() == before,
                "missing client approval accepted",
            )
            timing_refused = wire.call("delete_object", {**args, "object_ids": ["g", "u"]})[
                "result"
            ]
            require(
                timing_refused.get("isError") and working.read_bytes() == before,
                "fractional SMIL event reference was broken",
            )
            checks.append("fractional SMIL event reference refuses even after use is selected")
            changed = data(wire.call("delete_object", {**args, "object_ids": ["g", "u", "a"]}))
            require(
                changed["changed"] and b'href="#r"' not in working.read_bytes(),
                "joint deletion left dangling use",
            )
            data(
                wire.call(
                    "restore_snapshot", {"doc_id": doc, "snapshot_id": changed["snapshot_id"]}
                )
            )
            require(
                working.read_bytes() == before and source.read_bytes() == original,
                "restore/original preservation failed",
            )
            checks.append(
                "client-confirmed joint deletion and byte-exact restore preserve original"
            )
        finally:
            wire.close()
        failed_root = root / "failed-workspace"
        failed_root.mkdir()
        (failed_root / "fixture.svg").write_bytes(original)
        (failed_root / ".inkscape-mcp/registry.json").mkdir(parents=True)
        wire = Wire(
            [str(binary)],
            {**env, "INKSCAPE_MCP_WORKSPACE_ROOTS": str(failed_root)},
            root / "failed-wire.log",
        )
        try:
            wire.initialize()
            refused = wire.call("open_document", {"path": "fixture.svg"})["result"]
            docs = wire.request("resources/read", {"uri": "inkscape://documents"})["result"]
            require(refused.get("isError"), "invalid registry destination accepted")
            require(
                json.loads(docs["contents"][0]["text"])["documents"] == [],
                "failed open published document",
            )
            require(
                not (failed_root / ".inkscape-mcp/documents").exists(),
                "failed preflight created candidate copies",
            )
            checks.append("failed registration has no active documents or candidate copies")
        finally:
            wire.close()
    (output / "comparison.json").write_text(
        json.dumps(
            {
                "passed": True,
                "checks": checks,
                "scope": "Real STDIO with isolated workspaces and inert engine; no native GUI.",
            },
            indent=2,
        )
        + "\n"
    )
    print(f"Rust defect acceptance: {len(checks)} checks")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
