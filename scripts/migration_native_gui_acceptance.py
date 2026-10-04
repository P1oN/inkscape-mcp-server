#!/usr/bin/env python3
"""Owned native managed session acceptance. Never terminate Inkscape by process name."""

import argparse
import ast
import hashlib
import json
import subprocess
import time
from pathlib import Path
from tempfile import mkdtemp

from migration_probe import Wire, data, require, write


def close_owned_session(package, output, evidence):
    root = Path(evidence["root"])
    session = root / "session"
    env = evidence["env"]
    manifest = evidence["manifest"]
    document = evidence["document"]
    require(
        json.loads((session / "session.json").read_text()) == manifest,
        "owned session manifest changed; refuse quit",
    )
    gdbus = package / "libexec/inkscape-mcp/dbus/bin/gdbus"

    def dbus(destination, path, method, *arguments):
        result = subprocess.run(
            [
                str(gdbus),
                "call",
                "--address",
                manifest["address"],
                "--dest",
                destination,
                "--object-path",
                path,
                "--method",
                method,
                *arguments,
            ],
            env=env,
            capture_output=True,
            text=True,
            timeout=10,
            check=True,
        )
        return result.stdout

    owner = ast.literal_eval(
        dbus(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.GetNameOwner",
            "org.inkscape.Inkscape",
        )
    )[0]
    require(owner.startswith(":"), "quit must target a unique existing bus owner")
    # Darwin D-Bus lacks peer PID credentials on some builds. Verify
    # exact owned ancestry/path plus the same bound context in that case.
    pid = manifest["inkscape_pid"]
    pid_source = "D-Bus peer credentials"
    try:
        pid_reply = dbus(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.GetConnectionUnixProcessID",
            owner,
        )
        require(
            int(pid_reply.strip().removeprefix("(uint32 ").removesuffix(",)")) == pid,
            "bus owner PID differs from the owned GUI",
        )
    except subprocess.CalledProcessError as error:
        require("UnixProcessIdUnknown" in error.stderr, "unexpected bus ownership failure")
        contexts = ast.literal_eval(
            dbus(
                owner,
                "/org/inkscape/Inkscape/MCPContext",
                "org.inkscape.MCP.Context1.ListDocuments",
            )
        )[0]
        require(
            len(contexts) == 1
            and contexts[0][0] == document["window_id"]
            and contexts[0][1] == document["document_id"],
            "owned context changed; refuse quit",
        )
        pid_source = (
            "exact supervisor ancestry/private binary and bound context (Darwin PID unavailable)"
        )
    ps = subprocess.check_output(["/bin/ps", "-p", str(pid), "-o", "ppid=,command="], text=True)
    parent, command = ps.strip().split(None, 1)
    require(
        parent == str(manifest["supervisor_pid"])
        and command
        == str(session / "context-bridge/Inkscape.app/Contents/MacOS/inkscape") + " --with-gui",
        "owned GUI parent/path changed; refuse quit",
    )
    actions = ast.literal_eval(dbus(owner, "/org/inkscape/Inkscape", "org.gtk.Actions.List"))[0]
    require("quit" in actions, "owned GUI has no graceful quit action")
    dbus(owner, "/org/inkscape/Inkscape", "org.gtk.Actions.Activate", "quit", "[]", "{}")
    deadline = time.monotonic() + 15
    while (session / "session.json").exists() and time.monotonic() < deadline:
        time.sleep(0.05)
    require(not (session / "session.json").exists(), "supervisor did not clean its manifest")
    # gdbus cannot contact this private bus after the owned GUI has exited.
    result = subprocess.run(
        [
            str(gdbus),
            "call",
            "--address",
            manifest["address"],
            "--dest",
            "org.freedesktop.DBus",
            "--object-path",
            "/org/freedesktop/DBus",
            "--method",
            "org.freedesktop.DBus.ListNames",
        ],
        env=env,
        capture_output=True,
        timeout=5,
    )
    require(result.returncode != 0, "owned private bus survived GUI exit")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        alive = subprocess.run(
            ["/bin/ps", "-p", f"{pid},{manifest['supervisor_pid']}", "-o", "pid="],
            capture_output=True,
            text=True,
            check=False,
        )
        if not alive.stdout.strip():
            break
        time.sleep(0.05)
    require(not alive.stdout.strip(), "owned GUI/supervisor did not exit")
    evidence.update(state="gracefully-closed", graceful_gui_exit=True, bus_cleaned=True)
    write(output / "session.json", evidence)
    write(
        output / "shutdown-acceptance.json",
        {
            "passed": True,
            "unique_owned_PID": pid,
            "ownership_evidence": pid_source,
            "graceful_quit": True,
            "manifest_removed": True,
            "owned_bus_stopped": True,
        },
    )


def main(package, output, close_owned=False):
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
        # The supervisor owns the GUI independently of the STDIO server lifecycle.
        wire.close()
        wire = Wire([str(package / "bin/inkscape-mcp")], env, output / "reconnect.stderr.log")
        wire.initialize()
        require(
            json.loads((session / "session.json").read_text()) == manifest,
            "reconnect changed the owned session",
        )
        reconnected = data(wire.call("live_connect", {"prefer": "no_freeze"}))
        require(
            reconnected["connected"] and reconnected["transport"] == "managed-dbus",
            "reconnect failed to attach without launch",
        )
        write(output / "reconnected.json", reconnected)
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
                "reconnect_without_launch": True,
                "vendor_unchanged": True,
                "Undo_Redo_tested": False,
            },
        )
        if close_owned:
            close_owned_session(package, output, evidence)
        print(
            "Owned native GUI launch/context acceptance passed; "
            "synthetic window retained unless --close-owned.",
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
    parser.add_argument(
        "--close-owned",
        action="store_true",
        help="Gracefully quit only this verified owned blank GUI and check supervisor cleanup",
    )
    args = parser.parse_args()
    main(args.package.resolve(), args.output, args.close_owned)
