#!/usr/bin/env python3
"""Relocated local bundle acceptance with clean environment, real CLI and owned bus."""

import argparse
import base64
import hashlib
import json
import os
import shutil
import subprocess
import tarfile
import time
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write
from PIL import Image


def run(argv, env):
    result = subprocess.run(argv, env=env, capture_output=True, timeout=30)
    require(result.returncode == 0, f"packaged process failed: {argv[0]}: {result.stderr[:1000]!r}")
    require(len(result.stdout) <= 1024 * 1024, "unbounded package probe reply")
    return result.stdout.decode()


def verify_files(package):
    files = json.loads((package / "FILES.json").read_text())
    for relative, expected in files.items():
        path = package / relative
        require(path.is_file() and not path.is_symlink(), "packaged file missing/linked")
        raw = path.read_bytes()
        require(len(raw) == expected["bytes"], "packaged size drift")
        require(hashlib.sha256(raw).hexdigest() == expected["sha256"], "packaged hash drift")
    for path in package.rglob("*"):
        if path.is_symlink():
            require(path.resolve().is_relative_to(package), "package link escapes")
    return len(files)


def exercise(source, output, archive=False, engine_mode="per_call"):
    with TemporaryDirectory(prefix="imcp-package-", dir=Path("/tmp").resolve()) as temp:  # noqa: S108
        root = Path(temp).resolve()
        package = root / "installed"
        if archive:
            extracted = root / "extracted"
            extracted.mkdir()
            with tarfile.open(source) as stream:
                members = stream.getmembers()
                require(len(members) <= 10000, "archive file-count cap exceeded")
                require(
                    sum(member.size for member in members) <= 512 * 1024 * 1024,
                    "archive expanded-size cap exceeded",
                )
                tops = {Path(member.name).parts[0] for member in members if Path(member.name).parts}
                require(len(tops) == 1, "archive must contain exactly one package root")
                stream.extractall(extracted, filter="data")
            package = extracted / tops.pop()
        else:
            shutil.copytree(source, package, symlinks=True)
        count = verify_files(package)
        library = package / "libexec/inkscape-mcp"
        workspace = root / "workspace"
        workspace.mkdir()
        home = root / "home"
        home.mkdir()
        env = {
            "HOME": str(home),
            "PATH": "",
            "LANG": "en_US.UTF-8",
            "TMPDIR": str(root),
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(workspace),
            "INKSCAPE_MCP_LIVE_ENABLED": "1",
            "INKSCAPE_MCP_RAW_ACTION_ENABLED": "1",
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_TOOL_DESC": "full",
            "INKSCAPE_MCP_ENGINE_MODE": engine_mode,
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "not-launched"),
            "INKSCAPE_MCP_LIVE_RENDEZVOUS": str(root / "absent-rendezvous"),
            "GIO_MODULE_DIR": str(library / "dbus/lib/gio/modules"),
        }
        require(not (library / "python").exists(), "Python runtime still bundled")
        require(
            not any(p.suffix in (".py", ".pyc", ".pyo", ".whl") for p in package.rglob("*")),
            "Python source/bytecode/wheels still bundled",
        )
        manifest = json.loads((library / "package.json").read_text())
        require(
            manifest.get("runtime") == "native-rust"
            and not any(key in manifest for key in ("python", "runtime_source", "python_deps")),
            "package manifest still declares a Python runtime",
        )
        # Fixed native helpers with empty PATH and no project interpreter.
        native = root / "native-inx"
        native.mkdir(mode=0o700)
        source = native / "input.svg"
        fixture = json.loads(Path("migration/contracts/effect-data-cases.json").read_text())[
            "fingerprints"
        ][0]
        source.write_text(fixture["svg"])
        request = {
            "nonce": "mcp_" + "a" * 32,
            "operation": "style",
            "selection": ["r"],
            "style": {"fill": "blue"},
            "expected_ids": ["g", "r"],
            "expected_fingerprint": fixture["expected"],
        }
        from rust_socket_helper_acceptance import main as socket_acceptance

        socket_acceptance(package / "bin/inkscape-mcp-live", output / "native-socket")
        for changed in (True, False):
            (native / "insert-request.json").write_text(json.dumps(request))
            result = subprocess.run(
                [str(package / "bin/inkscape-mcp-inx"), "--id=r", str(source)],
                env={**env, "INKSCAPE_MCP_MANAGED_DIR": str(native)},
                capture_output=True,
                timeout=10,
                check=True,
            )
            reply = json.loads((native / "insert-result.json").read_text())
            require(
                reply["ok"] and bool(result.stdout) == changed and not result.stderr,
                "native one-shot change/noop failed without Python",
            )
            if changed:
                source.write_bytes(result.stdout)
                request["expected_fingerprint"] = reply["fingerprint"]
        bus_dir = root / "bus"
        bus_dir.mkdir(mode=0o700)
        address = "unix:path=" + str(bus_dir / "bus.sock")
        with (output / "packaged-bus.log").open("wb") as log:
            bus = subprocess.Popen(
                [
                    str(library / "dbus/bin/dbus-daemon"),
                    "--config-file=" + str(library / "dbus/session.conf"),
                    "--address=" + address,
                    "--nofork",
                    "--print-address=1",
                ],
                env=env,
                stdin=subprocess.DEVNULL,
                stdout=log,
                stderr=log,
            )
            try:
                deadline = time.monotonic() + 5
                while not (bus_dir / "bus.sock").exists() and time.monotonic() < deadline:
                    require(bus.poll() is None, "packaged private bus exited")
                    time.sleep(0.05)
                bus_id = run(
                    [
                        str(library / "dbus/bin/gdbus"),
                        "call",
                        "--address",
                        address,
                        "--dest",
                        "org.freedesktop.DBus",
                        "--object-path",
                        "/org/freedesktop/DBus",
                        "--method",
                        "org.freedesktop.DBus.GetId",
                    ],
                    env,
                )
                require(len(bus_id.strip()) > 30, "private bus exchange failed")
            finally:
                # This exact owned non-GUI bus is the only process terminated.
                if bus.poll() is None:
                    bus.terminate()
                    bus.wait(timeout=5)
        original = (
            b'<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">'
            b'<rect id="r" width="100" height="100" fill="#ff0000"/></svg>'
        )
        (workspace / "fixture.svg").write_bytes(original)
        wire = Wire([str(package / "bin/inkscape-mcp")], env, output / "packaged-server.stderr.log")
        try:
            wire.initialize()
            require(not (root / "not-launched").exists(), "MCP startup created managed session")
            contract = json.loads(
                Path("migration/contracts/live-true_raw-true_full_full.json").read_text()
            )
            for method in (
                "tools/list",
                "prompts/list",
                "resources/list",
                "resources/templates/list",
            ):
                require(
                    wire.request(method)["result"] == contract[method], "discovery contract drift"
                )
            caps = data(wire.call("list_capabilities", {}))
            require(caps["inkscape_available"], "minimal PATH did not find vendor Inkscape")
            doc = data(wire.call("open_document", {"path": "fixture.svg"}))["doc_id"]
            working = workspace / ".inkscape-mcp/documents" / doc / "working/document.svg"
            before = working.read_bytes()
            records = working.parent.parent / "operations"
            before_records = list(records.glob("*.json"))
            # A fixed old timestamp detects create-then-remove even when the final
            # file list is identical and the filesystem has coarse time resolution.
            os.utime(records, ns=(1_000_000_000, 1_000_000_000))

            def noop_inventory():
                return {
                    str(path.relative_to(workspace)): {
                        "mtime_ns": path.stat().st_mtime_ns,
                        "bytes": path.stat().st_size if path.is_file() else None,
                        "sha256": hashlib.sha256(path.read_bytes()).hexdigest()
                        if path.is_file()
                        else None,
                    }
                    for path in workspace.rglob("*")
                }

            noop_before = noop_inventory()
            noop = data(
                wire.call("set_fill", {"doc_id": doc, "object_ids": ["r"], "color": "#ff0000"})
            )
            require(not noop["changed"] and working.read_bytes() == before, "no-op changed bytes")
            require(list(records.glob("*.json")) == before_records, "no-op wrote audit records")
            noop_after = noop_inventory()
            write(output / "noop-filesystem.json", {"before": noop_before, "after": noop_after})
            require(noop_before == noop_after, "no-op changed filesystem bytes/sizes/mtimes")
            changed = data(
                wire.call("set_fill", {"doc_id": doc, "object_ids": ["r"], "color": "#0000ff"})
            )
            require(changed["changed"], "packaged edit did not apply")
            before_batch_count = len(list(records.glob("*.json")))
            batch = data(
                wire.call(
                    "apply_edits",
                    {
                        "doc_id": doc,
                        "edits": [
                            {
                                "op": "set_stroke",
                                "object_ids": ["r"],
                                "color": "black",
                                "width": "1px",
                            },
                            {"op": "move_object", "object_id": "r", "dx": 1, "dy": 0},
                        ],
                    },
                )
            )
            require(batch["changed"], "packaged atomic batch did not apply")
            require(
                len(list(records.glob("*.json"))) == before_batch_count + 1,
                "batch audit count drift",
            )
            batch_bytes = working.read_bytes()
            refused = wire.call(
                "apply_edits",
                {
                    "doc_id": doc,
                    "edits": [
                        {"op": "move_object", "object_id": "r", "dx": 50, "dy": 0},
                        {"op": "set_opacity", "object_ids": ["missing"], "opacity": 0.2},
                    ],
                },
            )
            require(
                refused["result"].get("isError") and working.read_bytes() == batch_bytes,
                "invalid batch did not roll back",
            )
            refused = wire.call("delete_object", {"doc_id": doc, "object_ids": ["r"]})
            require(
                refused["result"].get("isError") and working.read_bytes() == batch_bytes,
                "approval guard failed",
            )
            saved = data(wire.call("save_document_as", {"doc_id": doc, "dest_path": "saved.svg"}))
            resource = wire.request("resources/read", {"uri": saved["artifact"]["uri"]})["result"]
            require(
                base64.b64decode(resource["contents"][0]["blob"]) == working.read_bytes(),
                "readback drift",
            )
            require((workspace / "fixture.svg").read_bytes() == original, "original changed")
            # New batch members must work from the installed optimized artifact too.
            (workspace / "batch-members.svg").write_bytes(original)
            batch_doc = data(wire.call("open_document", {"path": "batch-members.svg"}))["doc_id"]
            batch_directory = workspace / ".inkscape-mcp/documents" / batch_doc
            batch_working = batch_directory / "working/document.svg"
            repeated = {
                "op": "repeat_objects",
                "object_id": "r",
                "group_id": "repeated",
                "placement": {
                    "kind": "polyline",
                    "points": [{"x": 0, "y": 0}, {"x": 10, "y": 10}],
                    "count": 2,
                },
            }
            fragment = {
                "op": "replace_svg_fragment",
                "object_id": "r",
                "svg": '<rect xmlns="http://www.w3.org/2000/svg" width="100" '
                'height="100" fill="blue"/>',
                "reference_policy": "allow_retained",
            }
            mixed = data(
                wire.call(
                    "apply_edits",
                    {"doc_id": batch_doc, "edits": [repeated, fragment], "approval_token": "test"},
                )
            )
            require(mixed["changed"] and mixed["risk_class"] == "high", "mixed batch failed")
            require(
                len(list((batch_directory / "snapshots").glob("*.svg"))) == 1,
                "mixed batch snapshot count",
            )
            require(
                len(list((batch_directory / "operations").glob("*.json"))) == 1,
                "mixed batch audit count",
            )
            committed = batch_working.read_bytes()
            refused = wire.call(
                "apply_edits",
                {
                    "doc_id": batch_doc,
                    "edits": [
                        {**repeated, "group_id": "next"},
                        {**fragment, "object_id": "missing"},
                    ],
                    "approval_token": "test",
                },
            )
            require(
                refused["result"].get("isError") and batch_working.read_bytes() == committed,
                "mixed batch late failure did not roll back",
            )
            require(
                len(list((batch_directory / "snapshots").glob("*.svg"))) == 1,
                "failed mixed batch added snapshot",
            )
            restored = wire.call(
                "restore_snapshot", {"doc_id": batch_doc, "snapshot_id": mixed["snapshot_id"]}
            )
            require(
                not restored["result"].get("isError") and batch_working.read_bytes() == original,
                "mixed batch restore failed",
            )
            require(
                (workspace / "batch-members.svg").read_bytes() == original,
                "mixed batch original changed",
            )
            preview = data(
                wire.call("render_preview", {"doc_id": doc, "width_px": 100, "inline": False})
            )
            write(output / "packaged-preview.json", preview)
            with Image.open(workspace / preview["artifact_path"]) as image:
                require(image.size == (100, 100), "preview dimensions drift")
                require(
                    image.convert("RGBA").getpixel((50, 50)) == (0, 0, 255, 255),
                    "real pixels not blue",
                )
            exported = data(
                wire.call("export_document", {"doc_id": doc, "format": "png", "width_px": 100})
            )
            binary = wire.request("resources/read", {"uri": exported["artifact"]["uri"]})["result"]
            import io

            with Image.open(io.BytesIO(base64.b64decode(binary["contents"][0]["blob"]))) as image:
                require(
                    image.convert("RGBA").getpixel((50, 50)) == (0, 0, 255, 255),
                    "export pixels drift",
                )
            require(not (root / "not-launched").exists(), "headless tools launched managed GUI")
            write(output / "packaged-server.trace.json", wire.trace)
        finally:
            wire.close()
        return {
            "passed": True,
            "installed_from_archive": archive,
            "engine_mode": engine_mode,
            "relocated_files": count,
            "minimal_PATH": "empty",
            "bundled_Python_absent": True,
            "helper_CLIs": 2,
            "native_one_shot_without_python": True,
            "native_socket_without_python": True,
            "private_bus_exchange": True,
            "stdio_surface": [110, 7, 18],
            "real_CLI_blue_pixels": True,
            "real_CLI_export_resource": True,
            "atomic_batch_and_rollback": True,
            "repeat_fragment_batch_and_restore": True,
            "approval_refusal": True,
            "no_op_without_audit": True,
            "no_op_without_transient_filesystem_writes": True,
            "source_original_preserved": True,
            "native_GUI_acceptance": False,
        }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    source_group = parser.add_mutually_exclusive_group(required=True)
    source_group.add_argument("--package", type=Path)
    source_group.add_argument("--archive", type=Path)
    parser.add_argument("--engine-mode", choices=("per_call", "shell"), default="per_call")
    parser.add_argument("--output", type=Path, default=Path("migration/results/package-acceptance"))
    parser.add_argument("--report", type=Path, default=Path("migration/package-comparison.json"))
    args = parser.parse_args()
    output = args.output
    output.mkdir(parents=True, exist_ok=True)
    source = args.archive or args.package
    result = exercise(
        source.resolve(), output, archive=args.archive is not None, engine_mode=args.engine_mode
    )
    write(args.report, result)
    print(json.dumps(result))
