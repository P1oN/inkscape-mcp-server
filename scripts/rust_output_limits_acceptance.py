#!/usr/bin/env python3
"""Current Rust resource/render byte and pixel limits on owned fixtures; no GUI."""

import argparse
import base64
import hashlib
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write
from rust_security_acceptance import inventory


def main(binary, output):
    binary = binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    with TemporaryDirectory(prefix="imcp-output-limits-") as temporary:
        root = Path(temporary).resolve()
        workspace = root / "workspace"
        workspace.mkdir()
        safe = (
            b'<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
            b'<rect id="r" width="16" height="16" fill="blue"/></svg>'
        )
        (workspace / "source.svg").write_bytes(safe)
        (workspace / "oversize.svg").write_bytes(safe + b" " * 4096)
        (workspace / "exact.bin").write_bytes(b"X" * 64)
        (workspace / "excess.bin").write_bytes(b"X" * 65)
        env = {
            key: value for key, value in os.environ.items() if not key.startswith("INKSCAPE_MCP_")
        }
        env.update(
            {
                "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
                "INKSCAPE_MCP_TOOL_PROFILE": "full",
                "INKSCAPE_MCP_LIVE_ENABLED": "false",
                "INKSCAPE_MCP_MANAGED_DIR": str(root / "absent-session"),
                "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
                "INKSCAPE_MCP_ENGINE_MODE": "per_call",
                "INKSCAPE_MCP_MAX_INPUT_BYTES": "4096",
                "INKSCAPE_MCP_MAX_OUTPUT_BYTES": "64",
                "INKSCAPE_MCP_MAX_EXPORT_PX": "16",
            }
        )
        wire = Wire([str(binary)], env, output / "server.stderr.log")
        try:
            wire.initialize()
            before = inventory(workspace)
            reply = wire.call("open_document", {"path": "oversize.svg"})
            require(reply["result"].get("isError"), "oversized input accepted")
            require(inventory(workspace) == before, "oversized open changed workspace")
            checks.append("input 4096 cap refuses oversize before writes")
            info = data(wire.call("get_workspace_info", {}))
            root_id = info["roots"][0]["root_id"]
            for name, size, accepted in [("exact.bin", 64, True), ("excess.bin", 65, False)]:
                token = base64.urlsafe_b64encode(name.encode()).decode().rstrip("=")
                before = inventory(workspace)
                reply = wire.request(
                    "resources/read", {"uri": f"inkscape://artifact/{root_id}/{token}"}
                )
                if accepted:
                    require("result" in reply, "exact-size resource refused")
                    blob = reply["result"]["contents"][0]["blob"]
                    require(
                        base64.b64decode(blob, validate=True) == b"X" * size, "resource bytes drift"
                    )
                else:
                    require("error" in reply, "oversized resource accepted")
                require(inventory(workspace) == before, "resource read changed workspace")
                checks.append(f"artifact {size} bytes accepted={accepted}; no writes")
            doc = data(wire.call("open_document", {"path": "source.svg"}))["doc_id"]
            before = inventory(workspace)
            reply = wire.call("render_preview", {"doc_id": doc, "width_px": 17, "inline": False})
            require(reply["result"].get("isError"), "pixel cap accepted oversized dimensions")
            require(inventory(workspace) == before, "pixel cap refusal changed workspace")
            checks.append("pixel cap refuses width17 before writes")
            reply = wire.call("export_document", {"doc_id": doc, "format": "png", "width_px": 16})
            require(reply["result"].get("isError"), "output byte cap accepted PNG")
            require(not list(workspace.rglob("*.png")), "oversized PNG published")
            require((workspace / "source.svg").read_bytes() == safe, "original source changed")
            checks.append("64-byte output cap refuses actual PNG before file publication")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    write(
        output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "checks": checks,
            "scope": "Selected file/resource/render byte and pixel limits; not every JSON output "
            "or allocation. Output-byte failure may create empty planned directories.",
        },
    )
    print(f"{len(checks)} output-limit checks passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
