"""Launch/reuse a private Inkscape session, then run the usual MCP stdio server.

A detached supervisor owns the bus and GUI. Closing/restarting Codex's MCP
connection therefore never terminates a drawing with unsaved work. The supervisor
shuts down its own bus only after its Inkscape process exits.
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import stat
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from inkscape_mcp.live.managed_dbus import ENV_DIR, ENV_STDOUT


def install_insertion_helper() -> None:
    """Install a fixed one-shot effect before Inkscape scans extensions at startup."""
    result = subprocess.run(
        [_binary("inkscape"), "--user-data-directory"],
        capture_output=True,
        text=True,
        check=True,
        timeout=10,
    )
    directory = result.stdout.strip()
    if not directory or not Path(directory).is_absolute():
        raise RuntimeError("Inkscape did not report its user data directory")
    target = Path(directory) / "extensions"
    target.mkdir(parents=True, exist_ok=True)
    source = Path(__file__).parent
    for name in ("inkscape_mcp_insert.py", "inkscape_mcp_insert.inx"):
        shutil.copyfile(source / "helper_extension" / name, target / name)
    shutil.copyfile(source / "insert_payload.py", target / "inkscape_mcp_insert_payload.py")
    vendor = Path(_binary("inkscape")).resolve().parents[1] / "Resources/share/inkscape/extensions"
    if not (vendor / "inkex").is_dir():
        raise RuntimeError("official Inkscape inkex source is unavailable")
    wrapper = target / "inkscape_mcp_insert_run.sh"
    wrapper.write_text(
        "#!/bin/sh\nunset PYTHONHOME PYTHONPATH\n"
        f"export PYTHONPATH={shlex.quote(str(vendor))}\n"
        f"exec {shlex.quote(sys.executable)} "
        f'{shlex.quote(str(target / "inkscape_mcp_insert.py"))} "$@"\n'
    )
    wrapper.chmod(0o700)


def secure_directory(path: Path) -> Path:
    if path.is_symlink():
        raise RuntimeError("session directory must not be a symlink")
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = path.stat()
    if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise RuntimeError("session directory must be owned by you with permissions 0700")
    path = path.resolve()
    if len(os.fsencode(path / "bus.sock")) >= 104:
        raise RuntimeError("session directory path is too long for a macOS Unix socket")
    return path


def _binary(name: str) -> str:
    found = shutil.which(name)
    if found is None:
        raise RuntimeError(f"{name} is missing; install it before starting the managed session")
    return found


def _reachable(address: str) -> bool:
    try:
        result = subprocess.run(
            [
                _binary("gdbus"),
                "call",
                "--address",
                address,
                "--dest",
                "org.inkscape.Inkscape",
                "--object-path",
                "/org/inkscape/Inkscape",
                "--method",
                "org.gtk.Actions.List",
            ],
            capture_output=True,
            timeout=2,
            check=False,
        )
        return result.returncode == 0
    except (OSError, subprocess.TimeoutExpired):
        return False


def _read_session(root: Path) -> dict[str, Any] | None:
    manifest = root / "session.json"
    if not manifest.is_file() or manifest.is_symlink() or manifest.stat().st_size > 8192:
        return None
    try:
        data = json.loads(manifest.read_text())
        # Adopt only our private Unix bus and fixed stdout path, never an arbitrary
        # address or readable file supplied via stale/corrupt session metadata.
        prefix = f"unix:path={root / 'bus.sock'}"
        if not isinstance(data, dict) or not isinstance(data.get("address"), str):
            return None
        if data["address"].split(",", 1)[0] != prefix:
            return None
        if _reachable(data["address"]):
            return data
    except (OSError, ValueError, TypeError):
        pass
    return None


def ensure_session(root: Path, document: Path | None = None) -> dict[str, Any]:
    import fcntl

    root = secure_directory(root)
    with (root / "launch.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        existing = _read_session(root)
        if existing is not None:
            if document is not None:
                raise RuntimeError(
                    "session already running; open the drawing in its Inkscape window"
                )
            return existing
        with (root / "supervisor.lock").open("a") as supervisor_lock:
            try:
                fcntl.flock(supervisor_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                raise RuntimeError(
                    "the managed GUI is still running but its bus is unavailable; "
                    "save and close that Inkscape window before restarting"
                ) from None
            finally:
                fcntl.flock(supervisor_lock, fcntl.LOCK_UN)
        (root / "session.json").unlink(missing_ok=True)
        with (root / "supervisor.log").open("wb") as log:
            argv = [
                sys.executable,
                "-m",
                "inkscape_mcp.live.macos_launcher",
                "--supervise",
                "--session-dir",
                str(root),
            ]
            if document is not None:
                argv += ["--document", str(document)]
            child = subprocess.Popen(
                argv,
                stdin=subprocess.DEVNULL,
                stdout=log,
                stderr=log,
                start_new_session=True,
            )
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            data = _read_session(root)
            if data is not None:
                return data
            if child.poll() is not None:
                raise RuntimeError(
                    "managed session failed; see supervisor.log in the session directory"
                )
            time.sleep(0.1)
        raise RuntimeError("Inkscape did not become ready; see supervisor.log")


def supervise(root: Path, document: Path | None) -> None:
    root = secure_directory(root)
    install_insertion_helper()
    daemon = _binary("dbus-daemon")
    inkscape = _binary("inkscape")
    (root / "bus.sock").unlink(missing_ok=True)
    with (
        (root / "bus.log").open("wb") as bus_log,
        (root / "inkscape.stdout.log").open("wb") as stdout,
        (root / "inkscape.stderr.log").open("wb") as stderr,
    ):
        bus = subprocess.Popen(
            [
                daemon,
                "--session",
                f"--address=unix:path={root / 'bus.sock'}",
                "--nofork",
                "--print-address=1",
            ],
            stdout=subprocess.PIPE,
            stderr=bus_log,
            text=True,
        )
        gui: subprocess.Popen[bytes] | None = None
        try:
            if bus.stdout is None:
                raise RuntimeError("private D-Bus stdout is unavailable")
            address = bus.stdout.readline().strip()
            if not address:
                raise RuntimeError("private D-Bus did not start")
            env = os.environ.copy()
            env["DBUS_SESSION_BUS_ADDRESS"] = address
            env[ENV_DIR] = str(root)
            argv = [inkscape, "--with-gui"]
            if document is not None:
                argv.append(str(document))
            gui = subprocess.Popen(argv, env=env, stdout=stdout, stderr=stderr)
            manifest = {"address": address, "inkscape_pid": gui.pid, "supervisor_pid": os.getpid()}
            pending = root / "session.tmp"
            pending.write_text(json.dumps(manifest))
            pending.chmod(0o600)
            pending.replace(root / "session.json")
            # Preserve unsaved work if the bus fails; the GUI is never force-killed.
            gui.wait()
        finally:
            if bus.poll() is None:
                bus.terminate()
                try:
                    bus.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    bus.kill()
                    bus.wait()
            (root / "session.json").unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser(description="Experimental macOS Inkscape + Codex session")
    parser.add_argument("--session-dir", type=Path)
    parser.add_argument("--document", type=Path, help="initial drawing, only for a new session")
    parser.add_argument(
        "--doctor", action="store_true", help="print read-only setup/session diagnosis"
    )
    parser.add_argument("--supervise", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if sys.platform != "darwin" and not args.doctor:
        parser.error("this launcher currently supports macOS only")
    # Desktop MCP hosts often start with a minimal PATH. Include vendor/brew
    # binary directories explicitly; no shell evaluation or sudo is required.
    os.environ["PATH"] = os.pathsep.join(
        [
            "/Applications/Inkscape.app/Contents/MacOS",
            "/opt/homebrew/bin",
            "/usr/local/bin",
            os.environ.get("PATH", ""),
        ]
    )
    root = args.session_dir or Path(f"/tmp/inkscape-mcp-{getattr(os, 'getuid', lambda: 0)()}")  # noqa: S108
    if args.doctor:
        from inkscape_mcp.live.macos_diagnostics import diagnose_macos

        report = diagnose_macos(root)
        print(report.model_dump_json(indent=2))
        parser.exit(0 if report.ready else 1)
    document = args.document.resolve() if args.document else None
    if document is not None and (not document.is_file() or document.suffix.lower() != ".svg"):
        parser.error("--document must point to an existing SVG")
    if args.supervise:
        import fcntl

        root = secure_directory(root)
        with (root / "supervisor.lock").open("a") as supervisor_lock:
            fcntl.flock(supervisor_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            supervise(root, document)
        return
    try:
        data = ensure_session(root, document)
    except (RuntimeError, OSError) as exc:
        parser.exit(1, f"{exc}\n")
    os.environ["DBUS_SESSION_BUS_ADDRESS"] = data["address"]
    os.environ[ENV_STDOUT] = str(root.resolve() / "inkscape.stdout.log")
    os.environ[ENV_DIR] = str(root.resolve())
    from inkscape_mcp.server import main as run_server

    run_server()


if __name__ == "__main__":
    main()
