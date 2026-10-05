#!/usr/bin/env python3
"""Stress first-request dispatch across bounded, owned STDIO server sessions; no GUI."""

import argparse
import hashlib
import json
import os
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write


def main(binary, output, sessions=128, parallel=4):
    require(1 <= sessions <= 512 and 1 <= parallel <= 8, "invalid startup stress bounds")
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve(strict=True)

    def session(index):
        with TemporaryDirectory(prefix="imcp-startup-") as temporary:
            env = {
                **os.environ,
                "SENTRY_DSN": "",
                "INKSCAPE_MCP_WORKSPACE_ROOTS": temporary,
                "INKSCAPE_MCP_LIVE_ENABLED": "false",
                "INKSCAPE_MCP_TOOL_PROFILE": "core",
                "INKSCAPE_MCP_TOOL_DESC": "short",
            }
            wire = Wire(
                [str(binary)], env, output / f"session-{index}.stderr.log", request_timeout=5
            )
            try:
                wire.initialize()
                doc = data(wire.call("create_document", {"width": 32, "height": 32}))["doc_id"]
                require(bool(doc), "first tool request did not create a document")
                for method in (
                    "tools/list",
                    "resources/list",
                    "resources/templates/list",
                    "prompts/list",
                ):
                    require("result" in wire.request(method), "discovery response missing")
                return {
                    "session": index,
                    "roundtrips_ns": [item["roundtrip_ns"] for item in wire.trace],
                }
            except Exception:
                write(
                    output / f"session-{index}.trace.json",
                    {"completed": wire.trace, "pending": wire.pending},
                )
                raise
            finally:
                wire.close()

    started = time.monotonic()
    results = []
    executor = ThreadPoolExecutor(max_workers=parallel)
    futures = [executor.submit(session, index) for index in range(sessions)]
    try:
        for future in as_completed(futures):
            results.append(future.result())
    finally:
        # A failed session must not start further server processes while unwinding.
        executor.shutdown(wait=True, cancel_futures=True)
        write(
            output / "comparison.json",
            {
                "passed": len(results) == sessions,
                "sessions": sessions,
                "parallel": parallel,
                "completed": len(results),
                "elapsed_seconds": time.monotonic() - started,
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "results": sorted(results, key=lambda item: item["session"]),
                "native_gui_acceptance": False,
            },
        )
    print(json.dumps({"passed": True, "sessions": sessions, "parallel": parallel}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--sessions", type=int, default=128)
    parser.add_argument("--parallel", type=int, default=4)
    args = parser.parse_args()
    main(args.binary, args.output, args.sessions, args.parallel)
