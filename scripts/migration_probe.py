#!/usr/bin/env python3
"""Capture reference contracts and repeatable timings over real MCP STDIO.

No GUI is launched. All drawings and writes use isolated temporary roots.
Raw wire results are retained: comparison must not discard unspecified fields.
"""

from __future__ import annotations

import argparse
import gzip
import itertools
import json
import os
import platform
import queue
import subprocess
import threading
import time
from pathlib import Path
from tempfile import TemporaryDirectory


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


class Wire:
    """Sequential JSON-RPC harness with bounded waits and captured stderr."""

    def __init__(self, command: list[str], env: dict[str, str], log: Path):
        self.log = log.open("w")
        self.process = subprocess.Popen(
            command,
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self.log,
            text=True,
            bufsize=1,
        )
        self.messages: queue.Queue = queue.Queue()
        self.sequence = 0
        self.trace: list[dict] = []
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self) -> None:
        if self.process.stdout is None:
            raise RuntimeError("stdout pipe unavailable")
        for line in self.process.stdout:
            try:
                self.messages.put(json.loads(line))
            except json.JSONDecodeError:
                self.messages.put({"invalid_stdout": line})
        self.messages.put({"eof": self.process.poll()})

    def request(self, method: str, params: dict | None = None) -> dict:
        self.sequence += 1
        message = {"jsonrpc": "2.0", "id": self.sequence, "method": method}
        if params is not None:
            message["params"] = params
        start = time.perf_counter_ns()
        self.send(message)
        deadline = time.monotonic() + 90
        while True:
            reply = self.messages.get(timeout=max(0.01, deadline - time.monotonic()))
            if "invalid_stdout" in reply or "eof" in reply:
                raise RuntimeError(reply)
            if reply.get("id") == self.sequence:
                self.trace.append(
                    {
                        "request": message,
                        "response": reply,
                        "roundtrip_ns": time.perf_counter_ns() - start,
                    }
                )
                return reply

    def send(self, message: dict) -> None:
        if self.process.stdin is None:
            raise RuntimeError("stdin pipe unavailable")
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()

    def initialize(self) -> dict:
        result = self.request(
            "initialize",
            {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "migration-probe", "version": "1"},
            },
        )
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        return result

    def call(self, name: str, arguments: dict) -> dict:
        return self.request("tools/call", {"name": name, "arguments": arguments})

    def close(self) -> None:
        if self.process.stdin is None:
            raise RuntimeError("stdin pipe unavailable")
        self.process.stdin.close()
        try:
            self.process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.process.terminate()  # Only the exact harness-owned server process.
            self.process.wait(timeout=10)
        self.log.close()


def data(reply: dict) -> dict:
    result = reply["result"]
    if result.get("isError"):
        raise RuntimeError(result)
    return result["structuredContent"]


def write(path: Path, value: object) -> None:
    payload = json.dumps(value, indent=2, ensure_ascii=False) + "\n"
    if path.suffix == ".gz":
        with gzip.open(path, "wt", encoding="utf-8") as handle:
            handle.write(payload)
    else:
        path.write_text(payload)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--matrix", action="store_true")
    parser.add_argument("--discovery-only", action="store_true")
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command
    if command[:1] == ["--"]:
        command = command[1:]
    if not command or not 1 <= args.repeats <= 30:
        parser.error("provide command and 1..30 repeats")
    args.output.mkdir(parents=True, exist_ok=True)
    write(
        args.output / "environment.json",
        {
            "command": command,
            "platform": platform.platform(),
            "machine": platform.machine(),
            "python": platform.python_version(),
            "PATH": os.environ.get("PATH"),
            "measurement": "perf_counter_ns; real JSON-RPC roundtrip; ps RSS in KiB",
        },
    )
    variants = list(
        itertools.product(("true", "false"), ("true", "false"), ("full", "core"), ("full", "short"))
    )
    if not args.matrix:
        variants = [("true", "true", "full", "full")]
    for live, raw, profile, desc in variants:
        key = f"live-{live}_raw-{raw}_{profile}_{desc}"
        with TemporaryDirectory(prefix="imcp-contract-") as temporary:
            env = {
                **os.environ,
                "INKSCAPE_MCP_WORKSPACE_ROOTS": temporary,
                "INKSCAPE_MCP_LIVE_ENABLED": live,
                "INKSCAPE_MCP_RAW_ACTION_ENABLED": raw,
                "INKSCAPE_MCP_TOOL_PROFILE": profile,
                "INKSCAPE_MCP_TOOL_DESC": desc,
            }
            wire = Wire(command, env, args.output / f"{key}.stderr.log")
            try:
                contract = {"initialize": wire.initialize()["result"]}
                for method in (
                    "tools/list",
                    "resources/list",
                    "resources/templates/list",
                    "prompts/list",
                ):
                    contract[method] = wire.request(method)["result"]
                require(bool(contract["tools/list"]["tools"]), "empty tool surface")
                write(args.output / f"{key}.json", contract)
                print(key, len(contract["tools/list"]["tools"]), flush=True)
            finally:
                wire.close()
    for run in range(0 if args.discovery_only else args.repeats):
        with TemporaryDirectory(prefix="imcp-benchmark-") as temporary:
            root = Path(temporary)
            fixture = (
                '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" '
                'viewBox="0 0 100 100"><rect id="r1" width="40" height="40" '
                'fill="#3366cc"/></svg>'
            )
            (root / "fixture.svg").write_text(fixture)
            env = {
                **os.environ,
                "INKSCAPE_MCP_WORKSPACE_ROOTS": temporary,
                "INKSCAPE_MCP_LIVE_ENABLED": "true",
                "INKSCAPE_MCP_RAW_ACTION_ENABLED": "true",
                "INKSCAPE_MCP_TOOL_PROFILE": "full",
                "INKSCAPE_MCP_TOOL_DESC": "full",
            }
            start = time.perf_counter_ns()
            wire = Wire(command, env, args.output / f"run-{run}.stderr.log")
            try:
                wire.initialize()
                startup_ns = time.perf_counter_ns() - start
                rss = subprocess.check_output(["ps", "-axo", "pid=,ppid=,rss=,comm="], text=True)
                wire.call("get_workspace_info", {})
                doc_id = data(wire.call("open_document", {"path": "fixture.svg"}))["doc_id"]
                wire.call("inspect_document", {"doc_id": doc_id})
                before = data(wire.call("list_snapshots", {"doc_id": doc_id}))
                noop = data(
                    wire.call(
                        "set_fill", {"doc_id": doc_id, "object_ids": ["r1"], "color": "#3366cc"}
                    )
                )
                after = data(wire.call("list_snapshots", {"doc_id": doc_id}))
                require(noop["changed"] is False and before == after, "no-op added state")
                changed = data(
                    wire.call(
                        "set_fill", {"doc_id": doc_id, "object_ids": ["r1"], "color": "#ff0000"}
                    )
                )
                wire.call(
                    "restore_snapshot", {"doc_id": doc_id, "snapshot_id": changed["snapshot_id"]}
                )
                wire.call(
                    "apply_edits",
                    {
                        "doc_id": doc_id,
                        "edits": [
                            {"op": "set_fill", "object_ids": ["r1"], "color": "#00ff00"},
                            {"op": "set_opacity", "object_ids": ["r1"], "opacity": 0.5},
                        ],
                    },
                )
                stable = data(wire.call("inspect_document", {"doc_id": doc_id}))
                rejected = wire.call(
                    "apply_edits",
                    {
                        "doc_id": doc_id,
                        "edits": [
                            {"op": "set_fill", "object_ids": ["r1"], "color": "#0000ff"},
                            {"op": "set_fill", "object_ids": ["absent"], "color": "#0000ff"},
                        ],
                    },
                )
                require(rejected["result"].get("isError", False), "invalid batch accepted")
                require(
                    stable == data(wire.call("inspect_document", {"doc_id": doc_id})),
                    "failed batch mutated document",
                )
                denied = wire.call("delete_object", {"doc_id": doc_id, "object_ids": ["r1"]})
                require(
                    denied["result"].get("isError", False) and "approval" in json.dumps(denied),
                    "approval gate not reached",
                )
                saved = data(
                    wire.call("save_document_as", {"doc_id": doc_id, "dest_path": "out/saved.svg"})
                )
                wire.request("resources/read", {"uri": saved["artifact"]["uri"]})
                wire.call("render_preview", {"doc_id": doc_id, "width_px": 128, "inline": False})
                wire.call(
                    "export_document",
                    {"doc_id": doc_id, "format": "png", "out_dir": "out", "width_px": 128},
                )
                wire.call("open_document", {"path": "../escape.svg"})
                wire.call("inspect_document", {"doc_id": "d_missing"})
                wire.call("live_status", {})
                for count in (100, 10000, 100000):
                    name = f"size-{count}.svg"
                    (root / name).write_text(
                        '<svg xmlns="http://www.w3.org/2000/svg">'
                        + "".join(
                            f'<rect id="r{i}" x="{i}" width="1" height="1"/>' for i in range(count)
                        )
                        + "</svg>"
                    )
                    large_id = data(wire.call("open_document", {"path": name}))["doc_id"]
                    wire.call("inspect_document", {"doc_id": large_id})
                require((root / "fixture.svg").read_text() == fixture, "source mutated")
                write(
                    args.output / f"run-{run}.json.gz",
                    {
                        "startup_to_initialized_ns": startup_ns,
                        "server_pid": wire.process.pid,
                        "process_rss_snapshot": rss,
                        "trace": wire.trace,
                        "assertions": [
                            "original intact",
                            "no-op no snapshot",
                            "atomic rollback",
                            "approval refusal",
                        ],
                    },
                )
                print(f"run {run}: initialized {startup_ns / 1e6:.1f} ms", flush=True)
            finally:
                wire.close()


if __name__ == "__main__":
    main()
