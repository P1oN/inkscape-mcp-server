#!/usr/bin/env python3
"""Actual STDIO refuses an oversized unfinished line before EOF; owned fixture only."""

import argparse
import hashlib
import os
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, require, write
from rust_security_acceptance import inventory


def main(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    with TemporaryDirectory(prefix="imcp-frame-cap-") as temporary:
        root = Path(temporary)
        workspace = root / "workspace"
        workspace.mkdir()
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_MAX_REQUEST_BYTES": "4096",
        }
        wire = Wire([str(binary.resolve())], env, output / "server.stderr.log")
        try:
            wire.initialize()
            for _ in range(3):
                require("result" in wire.request("ping"), "ordinary frame rejected")
            before = inventory(workspace)
            # No newline/EOF: the server must refuse before read_until can finish.
            payload = '{"jsonrpc":"2.0","id":99,"method":"tools/call","params":{"x":"'
            wire.process.stdin.write(payload + "a" * 5000)
            wire.process.stdin.flush()
            refused = True
            try:
                wire.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                refused = False
            after = inventory(workspace)
            write(
                output / "comparison.json",
                {
                    "passed": refused and before == after,
                    "unfinished_frame_refused_without_EOF": refused,
                    "ordinary_frames_accepted": 3,
                    "workspace_unchanged": before == after,
                    "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                    "scope": "Configured4096-byte request cap, one unfinished oversized line; "
                    "not all output caps or whole-process RSS proof",
                },
            )
            require(refused, "unfinished oversized input waits without frame cap")
            require(before == after, "refusal changed workspace")
        finally:
            write(output / "server.trace.json", wire.trace)
            wire.close()
    print("STDIO frame cap acceptance passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
