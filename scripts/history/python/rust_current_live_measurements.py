#!/usr/bin/env python3
"""Rust-only live measurements on an already recorded, PID-verified synthetic session."""

import argparse
import hashlib
import json
import statistics
import subprocess
import time
from pathlib import Path

from migration_probe import Wire, data, require, write
from PIL import Image
from rust_current_measurements import process_tree


def ownership(session):
    root = Path(session["root"])
    actual = json.loads((root / "session/session.json").read_text())
    require(actual == session["manifest"], "owned manifest changed")
    pid, parent = actual["inkscape_pid"], actual["supervisor_pid"]
    raw = subprocess.check_output(
        ["/bin/ps", "-p", f"{pid},{parent}", "-o", "pid=,ppid=,args="], text=True
    )
    rows = {
        int(parts[0]): (int(parts[1]), parts[2])
        for line in raw.splitlines()
        if len(parts := line.strip().split(None, 2)) == 3
    }
    require(
        rows.get(pid)
        == (
            parent,
            str(root / "session/context-bridge/Inkscape.app/Contents/MacOS/inkscape")
            + " --with-gui",
        ),
        "owned GUI PID/parent/path changed",
    )
    require(
        str(Path(session["package"]) / "bin/inkscape-mcp-supervisor") in rows[parent][1],
        "owned supervisor changed",
    )
    return {"manifest": actual, "ps": raw}


def main(source, output):
    session = json.loads((source / "session.json").read_text())
    binary = Path(session["package"]) / "bin/inkscape-mcp"
    require(
        hashlib.sha256(binary.read_bytes()).hexdigest() == session["binary_sha256"],
        "package changed",
    )
    output.mkdir(parents=True, exist_ok=False)
    write(
        output / "environment.json",
        {
            "command": [str(binary)],
            "env": session["env"],
            "binary_sha256": session["binary_sha256"],
            "clock": "perf_counter_ns; mixed STDIO roundtrip",
            "rss": "ps point observations; shared pages, no peak claim",
        },
    )
    baseline = None
    records, memory = [], []
    for repeat in range(5):
        write(output / f"{repeat}-ownership.json", ownership(session))
        start = time.perf_counter_ns()
        wire = Wire([str(binary)], session["env"], output / f"{repeat}-stderr.log")
        try:
            wire.initialize()
            records.append(
                {"repeat": repeat, "operation": "startup", "ns": time.perf_counter_ns() - start}
            )

            def call(name, arguments, wire=wire, repeat=repeat):
                value = data(wire.call(name, arguments))
                records.append(
                    {"repeat": repeat, "operation": name, "ns": wire.trace[-1]["roundtrip_ns"]}
                )
                return value

            connected = call("live_connect", {"prefer": "no_freeze"})
            require(
                connected["connected"] and not connected["ready_to_edit"], "reconnect failed guard"
            )
            docs = data(wire.call("live_list_documents", {}))["documents"]
            require(
                len(docs) == 1
                and all(
                    docs[0][key] == session["document"][key]
                    for key in ["window_id", "document_id", "path"]
                ),
                "document changed",
            )
            doc = session["document"]
            selected = data(
                wire.call(
                    "live_select_document",
                    {"window_id": doc["window_id"], "document_id": doc["document_id"]},
                )
            )
            require(selected["ready_to_edit"], "binding failed")
            for observation in range(3):
                selection = call("live_get_selection", {})
                scene = call("live_get_scene", {})
                render = call("live_render_view", {})
                relative = Path(render["artifact_path"])
                require(not relative.is_absolute() and ".." not in relative.parts, "escaped render")
                path = Path(session["env"]["INKSCAPE_MCP_WORKSPACE_ROOTS"]) / relative
                require(path.is_file() and not path.is_symlink(), "missing/linked PNG")
                png = path.read_bytes()
                (output / f"{repeat}-{observation}.png").write_bytes(png)
                with Image.open(path) as image:
                    image = image.convert("RGBA")
                    pixels = (image.size, hashlib.sha256(image.tobytes()).hexdigest())
                current = {
                    "tree": scene["scene"]["tree"],
                    "selection": selection["object_ids"],
                    "pixels": pixels,
                }
                if baseline is None:
                    baseline = current
                require(current == baseline, "read-only benchmark changed scene/selection/pixels")
                memory.append(
                    {
                        "repeat": repeat,
                        "observation": observation,
                        "server": process_tree(wire.process.pid),
                        "supervisor_GUI_bus": process_tree(session["manifest"]["supervisor_pid"]),
                    }
                )
        finally:
            write(output / f"{repeat}-trace.json.gz", wire.trace)
            wire.close()
    write(output / "baseline.json", baseline)
    write(output / "raw-timings.json", records)
    write(output / "memory.json", memory)
    grouped = {}
    for row in records:
        grouped.setdefault(row["operation"], []).append(row["ns"] / 1_000_000)
    summary = {
        key: {
            "n": len(values),
            "median_ms": statistics.median(values),
            "min_ms": min(values),
            "max_ms": max(values),
        }
        for key, values in grouped.items()
    }
    write(
        output / "comparison.json",
        {
            "passed": True,
            "binary_sha256": session["binary_sha256"],
            "timings": summary,
            "observations_unchanged": 15,
            "scope": "Five Rust server reconnects,15 identical tree/selection/RGBA observations; "
            "current owned GUI. Absolute mixed roundtrips/point RSS; no Python, "
            "speedup, pure IPC or peak claim. Mutation timings remain in native traces.",
        },
    )
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--session-output", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.session_output, args.output)
