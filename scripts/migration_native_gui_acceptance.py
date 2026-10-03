#!/usr/bin/env python3
"""Owned native managed session acceptance. Never terminate Inkscape by process name."""

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from tempfile import mkdtemp

from migration_probe import Wire, data, require, write


def main(package, output):
    output.mkdir(parents=True, exist_ok=True)
    require(
        not (output / "session.json").exists(),
        "inspect the previous owned session before another launch",
    )
    root = Path(mkdtemp(prefix="imcp-native-", dir=Path("/tmp").resolve())).resolve()  # noqa: S108
    home = root / "home"
    home.mkdir()
    workspace = root / "workspace"
    workspace.mkdir()
    session = root / "session"
    env = {
        "HOME": str(home),
        "TMPDIR": str(root),
        "PATH": "",
        "LANG": "en_US.UTF-8",
        "INKSCAPE_PROFILE_DIR": str(root / "profile"),
        "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
        "INKSCAPE_MCP_LIVE_ENABLED": "1",
        "INKSCAPE_MCP_TOOL_PROFILE": "full",
        "INKSCAPE_MCP_MANAGED_DIR": str(session),
        "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
        "INKSCAPE_MCP_PROCESS_TIMEOUT_S": "10",
    }
    engine = Path("/Applications/Inkscape.app/Contents/MacOS/inkscape")
    original_hash = hashlib.sha256(engine.read_bytes()).hexdigest()
    preflight = subprocess.run(
        [str(engine), "--user-data-directory"], env=env, capture_output=True, timeout=15, check=True
    )
    reported = Path(preflight.stdout.decode().strip())
    require(
        reported.is_absolute() and reported.is_relative_to(root),
        "Inkscape profile escaped owned root; do not install helpers",
    )
    evidence = {
        "root": str(root),
        "package": str(package),
        "binary_sha256": hashlib.sha256((package / "bin/inkscape-mcp").read_bytes()).hexdigest(),
        "profile": str(reported),
        "vendor_executable_sha256": original_hash,
        "state": "preflight",
        "env": env,
    }
    write(output / "session.json", evidence)
    print(json.dumps({"owned_root": str(root), "profile": str(reported)}), flush=True)
    wire = Wire([str(package / "bin/inkscape-mcp")], env, output / "mcp.stderr.log")
    try:
        wire.initialize()
        require(not session.exists(), "startup created managed session")
        launch = wire.call("live_launch", {})
        write(output / "launch.json", launch)
        require(data(launch)["result"] is True, "native launch did not become ready")
        manifest = json.loads((session / "session.json").read_text())
        require(
            manifest["address"].split(",", 1)[0] == "unix:path=" + str(session / "bus.sock"),
            "private bus escape",
        )
        require(manifest["context_bridge"] is True, "native context guard missing")
        evidence.update(state="launched", manifest=manifest)
        write(output / "session.json", evidence)
        connected = data(wire.call("live_connect", {"prefer": "no_freeze"}))
        write(output / "connected.json", connected)
        require(
            connected["connected"] and connected["transport"] == "managed-dbus",
            "wrong native transport",
        )
        documents = data(wire.call("live_list_documents", {}))
        write(output / "documents.json", documents)
        require(
            len(documents["documents"]) == 1,
            "unexpected windows on owned bus; inspect before editing",
        )
        document = documents["documents"][0]
        selected = data(
            wire.call(
                "live_select_document",
                {
                    "window_id": document["window_id"],
                    "document_id": document["document_id"],
                },
            )
        )
        require(selected["ready_to_edit"], "synthetic task drawing not ready")
        evidence.update(state="connected-owned-blank", document=document)
        write(output / "session.json", evidence)
        # This phase proves real packaged launch/context binding. Mutation/Undo follows
        # only after this ownership record can be independently inspected.
        require(
            hashlib.sha256(engine.read_bytes()).hexdigest() == original_hash,
            "vendor executable changed",
        )
        write(
            output / "launch-acceptance.json",
            {
                "passed": True,
                "native_GUI": True,
                "private_runtime_supervisor": True,
                "private_bus": True,
                "context_guard": True,
                "document_binding": True,
                "vendor_unchanged": True,
                "Undo_Redo_tested": False,
            },
        )
        print(
            "Owned native GUI launch/context acceptance passed; synthetic window retained.",
            flush=True,
        )
    finally:
        write(output / "mcp.trace.json", wire.trace)
        wire.close()
        # Preserve the owned synthetic GUI, supervisor and bus for Undo/reconnect acceptance.
        # On failure, inspect its exact recorded root/PIDs; never kill by name.


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument(
        "--output", type=Path, default=Path("migration/results/native-gui-acceptance")
    )
    args = parser.parse_args()
    main(args.package.resolve(), args.output)
