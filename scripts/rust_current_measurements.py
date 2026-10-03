#!/usr/bin/env python3
"""Rust-only current-package measurements over real STDIO and owned SVG fixtures; no GUI."""

import argparse
import base64
import hashlib
import io
import os
import platform
import statistics
import subprocess
import threading
import time
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write
from PIL import Image


def process_tree(pid):
    raw = subprocess.check_output(
        ["/bin/ps", "-axo", "pid=,ppid=,rss=,comm="], text=True, timeout=5
    )
    rows = []
    for line in raw.splitlines():
        fields = line.strip().split(None, 3)
        if len(fields) == 4:
            rows.append(
                dict(
                    zip(
                        ["pid", "ppid", "rss_kib", "command"],
                        [int(fields[0]), int(fields[1]), int(fields[2]), fields[3]],
                        strict=True,
                    )
                )
            )
    owned = {pid}
    while True:
        added = {r["pid"] for r in rows if r["ppid"] in owned} - owned
        if not added:
            return [r for r in rows if r["pid"] in owned]
        owned.update(added)


class Sampler:
    def __init__(self, pid):
        self.pid = pid
        self.samples = []
        self.errors = []
        self.stop = threading.Event()
        self.phase = "ready"
        self.thread = threading.Thread(target=self.run)
        self.thread.start()

    def run(self):
        while not self.stop.is_set():
            start = time.perf_counter_ns()
            phase = self.phase
            try:
                self.samples.append(
                    {
                        "time_ns": start,
                        "phase": phase,
                        "processes": process_tree(self.pid),
                        "sample_duration_ns": time.perf_counter_ns() - start,
                    }
                )
            except (OSError, subprocess.SubprocessError, ValueError) as error:
                self.errors.append(str(error))
            self.stop.wait(0.05)

    def close(self):
        self.stop.set()
        self.thread.join(timeout=6)
        require(not self.thread.is_alive(), "RSS sampler did not stop")


def fixture(count):
    shapes = "".join(
        f'<rect id="r{i}" x="{i % 100}" y="{(i // 100) % 100}" '
        'width="0.75" height="0.75" fill="blue"/>'
        for i in range(count)
    )
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">' + shapes + "</svg>"
    ).encode()


def main(binary, output, repeats):
    binary = binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    require(binary.is_file(), "missing binary")
    records = []
    memory = []
    inputs = output / "inputs"
    inputs.mkdir()
    fixtures = {}
    for count in [100, 3000, 15000]:
        content = fixture(count)
        (inputs / f"{count}.svg").write_bytes(content)
        fixtures[count] = content
    write(
        output / "environment.json",
        {
            "platform": platform.platform(),
            "python": platform.python_version(),
            "binary": str(binary),
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "argv": [str(binary)],
            "repeats": repeats,
            "engine_order": ["per_call", "shell"] * repeats,
            "clock": "perf_counter_ns; timings include JSON encoding, IPC, handler, serialization",
            "rss": "ps RSS in KiB; 50ms target plus ps runtime; observed samples, not true peak",
            "input_sizes": {str(k): len(v) for k, v in fixtures.items()},
            "no_python_oracle": True,
            "native_gui": False,
        },
    )
    with TemporaryDirectory(prefix="imcp-rust-measure-") as temporary:
        root = Path(temporary).resolve()
        for repeat in range(repeats):
            for mode in ["per_call", "shell"]:
                label = f"{repeat:02}-{mode}"
                directory = output / label
                directory.mkdir()
                private = root / label
                workspace = private / "workspace"
                workspace.mkdir(parents=True)
                home = private / "home"
                home.mkdir()
                for count, content in fixtures.items():
                    (workspace / f"{count}.svg").write_bytes(content)
                render_source = (
                    b'<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">'
                    b'<rect id="r" width="16" height="16" fill="blue"/></svg>'
                )
                (workspace / "render.svg").write_bytes(render_source)
                env = {k: v for k, v in os.environ.items() if not k.startswith("INKSCAPE_MCP_")}
                env.update(
                    {
                        "HOME": str(home),
                        "PATH": "/Applications/Inkscape.app/Contents/MacOS:/usr/bin:/bin",
                        "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
                        "INKSCAPE_MCP_ENGINE_MODE": mode,
                        "INKSCAPE_MCP_TOOL_PROFILE": "full",
                        "INKSCAPE_MCP_LIVE_ENABLED": "false",
                        "INKSCAPE_MCP_MANAGED_DIR": str(private / "not-launched"),
                        "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(private / "absent-rendezvous"),
                    }
                )
                write(
                    directory / "environment.json",
                    {
                        k: env[k]
                        for k in env
                        if k.startswith("INKSCAPE_MCP_") or k in ["HOME", "PATH"]
                    },
                )
                start = time.perf_counter_ns()
                wire = Wire([str(binary)], env, directory / "server.stderr.log")
                sampler = None
                run_memory = {"mode": mode, "repeat": repeat}
                try:
                    ready = wire.initialize()
                    require("result" in ready, "initialize failed")
                    startup = time.perf_counter_ns() - start
                    records.append(
                        {
                            "mode": mode,
                            "repeat": repeat,
                            "operation": "startup",
                            "elapsed_ns": startup,
                        }
                    )
                    sampler = Sampler(wire.process.pid)

                    def timed(
                        name,
                        arguments,
                        operation=None,
                        size=None,
                        sampler=sampler,
                        wire=wire,
                        mode=mode,
                        repeat=repeat,
                    ):
                        sampler.phase = operation or name
                        reply = wire.call(name, arguments)
                        result = data(reply)
                        records.append(
                            {
                                "mode": mode,
                                "repeat": repeat,
                                "operation": operation or name,
                                "svg_rectangles": size,
                                "elapsed_ns": wire.trace[-1]["roundtrip_ns"],
                            }
                        )
                        return result

                    for count in fixtures:
                        doc = timed("open_document", {"path": f"{count}.svg"}, size=count)["doc_id"]
                        timed("inspect_document", {"doc_id": doc}, size=count)
                        edit = timed(
                            "set_fill",
                            {"doc_id": doc, "object_ids": ["r0"], "color": "red"},
                            "single_edit",
                            count,
                        )
                        require(edit["changed"], "single edit did not change")
                        edits = [
                            {"op": "set_fill", "object_ids": [f"r{i}"], "color": "green"}
                            for i in range(8)
                        ]
                        batch = timed(
                            "apply_edits", {"doc_id": doc, "edits": edits}, "atomic_batch_8", count
                        )
                        require(batch["changed"], "batch did not change")
                        saved = timed(
                            "save_document_as",
                            {"doc_id": doc, "dest_path": f"saved-{count}.svg"},
                            "save",
                            count,
                        )
                        sampler.phase = "resource_readback"
                        resource = wire.request("resources/read", {"uri": saved["artifact"]["uri"]})
                        require("result" in resource, "saved resource failed")
                        content = base64.b64decode(
                            resource["result"]["contents"][0]["blob"], validate=True
                        )
                        require(
                            content == (workspace / f"saved-{count}.svg").read_bytes(),
                            "save resource mismatch",
                        )
                        records.append(
                            {
                                "mode": mode,
                                "repeat": repeat,
                                "operation": "resource_readback",
                                "svg_rectangles": count,
                                "elapsed_ns": wire.trace[-1]["roundtrip_ns"],
                            }
                        )
                        require(
                            (workspace / f"{count}.svg").read_bytes() == fixtures[count],
                            "source changed",
                        )
                    doc = data(wire.call("open_document", {"path": "render.svg"}))["doc_id"]
                    export_bytes = {}
                    for sequence in range(3):
                        preview = timed(
                            "render_preview",
                            {"doc_id": doc, "width_px": 64, "inline": False},
                            "render_first" if sequence == 0 else "render_reused",
                        )
                        with Image.open(workspace / preview["artifact_path"]) as image:
                            require(
                                image.size == (64, 64)
                                and image.convert("RGBA").getpixel((32, 32)) == (0, 0, 255, 255),
                                "preview pixel mismatch",
                            )
                        exported = timed(
                            "export_document",
                            {"doc_id": doc, "format": "png", "width_px": 64},
                            "export",
                        )
                        resource = wire.request(
                            "resources/read", {"uri": exported["artifact"]["uri"]}
                        )
                        png = base64.b64decode(
                            resource["result"]["contents"][0]["blob"], validate=True
                        )
                        with Image.open(io.BytesIO(png)) as image:
                            require(
                                image.size == (64, 64)
                                and image.convert("RGBA").getpixel((32, 32)) == (0, 0, 255, 255),
                                "export pixel mismatch",
                            )
                        artifact_path = workspace / exported["artifact_path"]
                        require(
                            artifact_path not in export_bytes, "repeated export reused filename"
                        )
                        export_bytes[artifact_path] = png
                        (directory / f"export-{sequence}.png").write_bytes(png)
                    require(
                        (workspace / "render.svg").read_bytes() == render_source,
                        "render original changed",
                    )
                    require(
                        not (private / "not-launched").exists(), "headless measurement launched GUI"
                    )
                    for artifact_path, png in export_bytes.items():
                        require(
                            artifact_path.read_bytes() == png, "previous export was overwritten"
                        )
                    sampler.phase = "idle_after_operations"
                    # Idle observation; short requests can finish between RSS samples.
                    run_memory["idle_processes"] = process_tree(wire.process.pid)
                    memory.append(run_memory)
                finally:
                    try:
                        if sampler:
                            sampler.close()
                            write(
                                directory / "rss-samples.json",
                                {"samples": sampler.samples, "errors": sampler.errors},
                            )
                            run_memory["sampled_server_max_kib"] = max(
                                (
                                    r["rss_kib"]
                                    for sample in sampler.samples
                                    for r in sample["processes"]
                                    if r["pid"] == wire.process.pid
                                ),
                                default=None,
                            )
                            run_memory["sampled_children_max_kib"] = max(
                                (
                                    sum(
                                        r["rss_kib"]
                                        for r in sample["processes"]
                                        if r["pid"] != wire.process.pid
                                    )
                                    for sample in sampler.samples
                                ),
                                default=None,
                            )
                            write(directory / "memory.json", run_memory)
                        write(directory / "wire.trace.json.gz", wire.trace)
                    finally:
                        wire.close()
                require(not sampler.errors, "RSS sampler errors")
    grouped = {}
    for record in records:
        key = f"{record['mode']}:{record['operation']}:{record.get('svg_rectangles') or '-'}"
        grouped.setdefault(key, []).append(record["elapsed_ns"] / 1_000_000)
    summaries = {
        key: {
            "n": len(values),
            "median_ms": statistics.median(values),
            "min_ms": min(values),
            "max_ms": max(values),
        }
        for key, values in grouped.items()
    }
    write(output / "raw-timings.json", records)
    write(output / "memory.json", memory)
    write(
        output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "timings": summaries,
            "memory": memory,
            "scope": "Rust-only current package, bounded fixtures and absolute roundtrips. "
            "Observed RSS, no true peak or speedup claim, no GUI/live, no pure IPC attribution.",
        },
    )
    print(f"{len(records)} measured requests/startups; {len(memory)} owned server runs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=5)
    args = parser.parse_args()
    if not 3 <= args.repeats <= 10:
        parser.error("repeats must be 3..10")
    main(args.binary, args.output, args.repeats)
