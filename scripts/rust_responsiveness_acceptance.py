#!/usr/bin/env python3
"""Real STDIO responsiveness/cancellation checks against owned synthetic headless engines."""

import argparse
import json
import os
import time
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require


def main(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    evidence = []
    for mode in ("per_call", "shell"):
        with TemporaryDirectory(prefix="imcp-cancel-") as temporary:
            root = Path(temporary)
            vendor = root / "vendor"
            vendor.mkdir()
            ready = root / "ready"
            engine = vendor / "inkscape"
            interpreter = Path(".venv/bin/python").resolve()
            engine.write_text(
                f"#!{interpreter}\n"
                "import sys,os,time\nfrom pathlib import Path\n"
                "if '--version' in sys.argv:\n print('Inkscape 1.4');sys.exit(0)\n"
                "if '--action-list' in sys.argv: sys.exit(0)\n"
                "def slow():\n"
                " Path(os.environ['IMCP_CANCEL_READY']).write_text(str(os.getpid()))\n"
                " time.sleep(60)\n"
                "if '--shell' in sys.argv:\n"
                " print('> ',end='',flush=True)\n"
                " for line in sys.stdin:\n"
                "  if 'export-do' in line: slow()\n"
                "  print('> ',end='',flush=True)\n"
                "else: slow()\n"
            )
            engine.chmod(0o700)
            env = {
                **os.environ,
                "PATH": str(vendor),
                "SENTRY_DSN": "",
                "IMCP_CANCEL_READY": str(ready),
                "INKSCAPE_MCP_WORKSPACE_ROOTS": str(root),
                "INKSCAPE_MCP_LIVE_ENABLED": "false",
                "INKSCAPE_MCP_ENGINE_MODE": mode,
                "INKSCAPE_MCP_PROCESS_TIMEOUT_S": "60",
            }
            wire = Wire([str(binary.resolve())], env, output / (mode + ".log"))
            try:
                wire.initialize()
                doc = data(wire.call("create_document", {"width": 32, "height": 32}))["doc_id"]
                working = root / ".inkscape-mcp/documents" / doc / "working/document.svg"
                before = working.read_bytes()
                wire.send(
                    {
                        "jsonrpc": "2.0",
                        "id": 100,
                        "method": "tools/call",
                        "params": {"name": "render_preview", "arguments": {"doc_id": doc}},
                    }
                )
                deadline = time.monotonic() + 5
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                require(ready.exists(), "headless engine did not start")
                pid = int(ready.read_text())
                started = time.monotonic()
                wire.send({"jsonrpc": "2.0", "id": 101, "method": "tools/list", "params": {}})
                wire.send(
                    {
                        "jsonrpc": "2.0",
                        "id": 102,
                        "method": "resources/read",
                        "params": {"uri": "inkscape://workspace"},
                    }
                )
                replies = {}
                while not {101, 102}.issubset(replies):
                    reply = wire.messages.get(timeout=2)
                    replies[reply.get("id")] = reply
                require(time.monotonic() - started < 2, "discovery/workspace blocked by render")
                wire.send(
                    {
                        "jsonrpc": "2.0",
                        "method": "notifications/cancelled",
                        "params": {"requestId": 100, "reason": "acceptance"},
                    }
                )
                wire.send(
                    {
                        "jsonrpc": "2.0",
                        "id": 103,
                        "method": "tools/call",
                        "params": {
                            "name": "create_document",
                            "arguments": {"width": 16, "height": 16},
                        },
                    }
                )
                while 103 not in replies:
                    reply = wire.messages.get(timeout=3)
                    replies[reply.get("id")] = reply
                require(
                    not replies[103].get("result", {}).get("isError"),
                    "gate not released after cancellation",
                )
                try:
                    os.kill(pid, 0)
                    raise RuntimeError("cancelled headless process survived")
                except ProcessLookupError:
                    pass
                require(before == working.read_bytes(), "cancelled render changed working copy")
                require(
                    not list((root / ".inkscape-mcp").rglob("*.png")),
                    "cancelled render published PNG",
                )
                evidence.append(
                    {
                        "engine": mode,
                        "responsive": True,
                        "cancelled_child_reaped": True,
                        "followup_edit": True,
                    }
                )
            finally:
                wire.close()
    (output / "comparison.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print(json.dumps(evidence))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    main(args.binary, args.output)
