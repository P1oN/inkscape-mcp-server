#!/usr/bin/env python3
"""Verify actual packaged STDIO refuses special inputs and artifact paths; no GUI."""

import argparse
import base64
import hashlib
import os
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    sha = hashlib.sha256(binary.read_bytes()).hexdigest()
    observations = []
    with TemporaryDirectory(prefix="imcp-special-files-") as temporary:
        root = Path(temporary).resolve()
        workspace = root / "workspace"
        workspace.mkdir()
        (workspace / "directory").mkdir()
        os.mkfifo(workspace / "pipe")
        outside = root / "outside"
        outside.write_bytes(b"preserved outside original")
        (workspace / "link").symlink_to(outside)
        (workspace / "parent").symlink_to(root)
        env = {
            **os.environ,
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "HOME": str(root / "home"),
            "XDG_CONFIG_HOME": str(root / "config"),
            "XDG_CACHE_HOME": str(root / "cache"),
            "INKSCAPE_PROFILE_DIR": str(root / "profile"),
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
        }
        wire = Wire([str(binary)], env, args.output / "server.stderr.log")
        try:
            wire.initialize()
            key = data(wire.call("get_workspace_info", {}))["roots"][0]["root_id"]
            for path in ["pipe", "directory", "link", "parent/outside"]:
                for tool in ["open_document", "stat_artifact"]:
                    reply = wire.call(tool, {"path": path})
                    require(reply["result"].get("isError"), "special/escaped input accepted")
                    observations.append({"tool": tool, "path": path})
                token = base64.urlsafe_b64encode(path.encode()).decode().rstrip("=")
                reply = wire.request(
                    "resources/read", {"uri": f"inkscape://artifact/{key}/{token}"}
                )
                require("error" in reply, "special/escaped artifact accepted")
                observations.append({"resource": "artifact", "path": path})
            require(outside.read_bytes() == b"preserved outside original", "outside file changed")
            require(
                not (workspace / ".inkscape-mcp").exists(), "refused calls created managed files"
            )
            require(
                sorted(p.name for p in workspace.iterdir())
                == ["directory", "link", "parent", "pipe"],
                "fixture changed",
            )
        finally:
            write(args.output / "server.trace.json", wire.trace)
            wire.close()
    require(hashlib.sha256(binary.read_bytes()).hexdigest() == sha, "binary changed")
    write(
        args.output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": sha,
            "observations": observations,
            "scope": "Special-file/escape refusal and zero writes; not full race/security proof",
        },
    )
    print(f"{len(observations)} special-file/escape refusals pass")


if __name__ == "__main__":
    main()
