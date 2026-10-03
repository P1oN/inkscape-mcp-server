#!/usr/bin/env python3
"""Owned STDIO checks for bounded validation diagnostics; no engine or GUI."""

import argparse
import hashlib
import json
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, require, write
from rust_security_acceptance import inventory


def main(binary, output):
    binary = binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    with TemporaryDirectory(prefix="imcp-diagnostic-") as temporary:
        root = Path(temporary).resolve()
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(root),
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "absent-session"),
            "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
        }
        wire = Wire([str(binary)], env, output / "server.stderr.log")
        try:
            wire.initialize()
            for label, tool, arguments, kind in [
                (
                    "unknown-field",
                    "create_document",
                    {"width": 16, "height": 16, "q" * 250_000: ["🌿" * 250_000]},
                    "unexpected_keyword_argument",
                ),
                (
                    "discriminator",
                    "apply_edits",
                    {"doc_id": "none", "edits": [{"op": "tag" * 250_000}]},
                    "union_tag_invalid",
                ),
                (
                    "nested-wrong-scalar",
                    "create_document",
                    {"width": [{"large": "🌿\n" * 250_000}], "height": 16},
                    "float_type",
                ),
            ]:
                before = inventory(root)
                reply = wire.call(tool, arguments)
                require(reply["result"].get("isError"), "invalid input accepted")
                text = reply["result"]["content"][0]["text"]
                size = len(json.dumps(reply, ensure_ascii=False).encode())
                write(output / (label + "-size.json"), {"response_bytes": size, "limit": 4096})
                require(kind in text, "unexpected validation error")
                require(
                    size < 4096, "validation response exceeds selected diagnostic cap: " + label
                )
                require(before == inventory(root), "invalid arguments changed workspace")
                checks.append(label + " bounded error before writes")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    write(
        output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "checks": checks,
            "scope": "Three oversized inputs; not a transport/all-output allocation proof",
        },
    )
    print(f"{len(checks)} diagnostic checks passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
