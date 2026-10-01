"""Read-only setup/session diagnosis; never launch a GUI or install an extension."""

from __future__ import annotations

import ast
import importlib.util
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field

from inkscape_mcp.live.context_bridge import INTERFACE, OBJECT_PATH
from inkscape_mcp.live.managed_dbus import INSERT_ACTION
from inkscape_mcp.live.native_launcher import build_dependencies


class MacOSDiagnosis(BaseModel):
    state: Literal[
        "unsupported_platform",
        "missing_dependencies",
        "setup_incomplete",
        "ready_to_launch",
        "unsafe_session",
        "session_unavailable",
        "running",
        "running_helper_unavailable",
        "running_context_unavailable",
        "diagnosis_failed",
    ]
    ready: bool = False
    inkscape_version: str | None = None
    checks: dict[str, bool] = Field(default_factory=dict)
    helper_installed: bool = False
    insertion_available: bool = False
    context_bridge_available: bool = False
    next_steps: list[str] = Field(default_factory=list)


def _stdout(argv: list[str]) -> str | None:
    try:
        result = subprocess.run(argv, capture_output=True, text=True, timeout=3, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return result.stdout.strip() if result.returncode == 0 else None


def _supervisor_lock_held(root: Path) -> bool:
    """Probe an existing lock without creating files or waiting for the supervisor."""
    if sys.platform == "win32":
        raise RuntimeError("managed session requires a POSIX platform")
    import fcntl

    try:
        lock = (root / "supervisor.lock").open("rb")
    except FileNotFoundError:
        return False
    with lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return True
        fcntl.flock(lock, fcntl.LOCK_UN)
    return False


def diagnose_macos(root: Path) -> MacOSDiagnosis:
    try:
        return _diagnose_macos(root)
    except (OSError, RuntimeError):
        return MacOSDiagnosis(
            state="diagnosis_failed",
            next_steps=[
                "Could not inspect setup/session; check file access and installation paths."
            ],
        )


def _ready_to_launch(report: MacOSDiagnosis) -> MacOSDiagnosis:
    report.checks.update(build_dependencies())
    if not all(report.checks[name] for name in ("clang", "codesign", "glib_headers")):
        report.state = "missing_dependencies"
        report.next_steps = [
            "Install Apple command line tools (xcode-select --install) and brew install glib.",
            "The context bridge builds a private launcher copy; the vendor app stays unchanged.",
        ]
    else:
        report.state = "ready_to_launch"
        report.ready = True
        report.next_steps = [
            "Use live_launch or inkscape-mcp-macos --launch to build the bridge "
            "and launch Inkscape."
        ]
    return report


def _diagnose_macos(root: Path) -> MacOSDiagnosis:
    if sys.platform != "darwin":
        return MacOSDiagnosis(
            state="unsupported_platform", next_steps=["Use this launcher on macOS."]
        )
    paths = {name: shutil.which(name) for name in ("inkscape", "dbus-daemon", "gdbus")}
    checks = {name: path is not None for name, path in paths.items()}
    checks["python_3_12"] = sys.version_info >= (3, 12)
    for name in ("numpy", "cssselect", "tinycss2"):
        checks[name] = importlib.util.find_spec(name) is not None
    if not all(checks.values()):
        steps = []
        if not checks["inkscape"]:
            steps.append("Install official Inkscape in /Applications/Inkscape.app.")
        if not checks["dbus-daemon"] or not checks["gdbus"]:
            steps.append("Install Homebrew dependencies: brew install dbus glib uv.")
        if not all(checks[n] for n in ("python_3_12", "numpy", "cssselect", "tinycss2")):
            steps.append("Run uv sync --python 3.12 --frozen and use the project's interpreter.")
        return MacOSDiagnosis(state="missing_dependencies", checks=checks, next_steps=steps)

    binary = Path(paths["inkscape"] or "").resolve()
    vendor = binary.parent.parent / "Resources/share/inkscape/extensions/inkex"
    checks["vendor_inkex"] = vendor.is_dir()
    version = _stdout([str(binary), "--version"])
    data_dir = _stdout([str(binary), "--user-data-directory"])
    checks["inkscape_cli"] = bool(version and data_dir and Path(data_dir).is_absolute())
    report = MacOSDiagnosis(state="setup_incomplete", checks=checks, inkscape_version=version)
    if not checks["vendor_inkex"] or not checks["inkscape_cli"]:
        report.next_steps = ["Check the official Inkscape installation and its CLI/inkex files."]
        return report
    extensions = Path(data_dir or "") / "extensions"
    report.helper_installed = all(
        (extensions / name).is_file()
        for name in (
            "inkscape_mcp_insert.inx",
            "inkscape_mcp_insert.py",
            "inkscape_mcp_insert_payload.py",
            "inkscape_mcp_insert_run.sh",
        )
    )
    if root.is_symlink():
        report.state = "unsafe_session"
        report.next_steps = ["Choose a private session directory that is not a symlink."]
        return report
    if len(os.fsencode(root.resolve() / "bus.sock")) >= 104:
        report.state = "unsafe_session"
        report.next_steps = ["Use a shorter session directory for the macOS Unix socket."]
        return report
    if not root.exists():
        return _ready_to_launch(report)
    info = root.stat()
    if not root.is_dir() or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
        report.state = "unsafe_session"
        report.next_steps = ["Use a session directory owned by you with permissions 0700."]
        return report

    from inkscape_mcp.live.macos_launcher import _read_session

    session = _read_session(root.resolve())
    if session is None:
        if not (root / "session.json").exists() and not _supervisor_lock_held(root):
            return _ready_to_launch(report)
        else:
            report.state = "session_unavailable"
            report.next_steps = [
                "The managed bus cannot be reached. Save and close its drawing before restarting.",
                "Inspect supervisor.log, inkscape.stderr.log and bus.log in the session directory.",
            ]
        return report
    report.checks["managed_stdout"] = (root / "inkscape.stdout.log").is_file()
    if not report.checks["managed_stdout"]:
        report.state = "session_unavailable"
        report.next_steps = [
            "Managed selection output is missing. Save and close its drawing before restarting."
        ]
        return report
    description = _stdout(
        [
            paths["gdbus"] or "gdbus",
            "call",
            "--address",
            session["address"],
            "--dest",
            "org.inkscape.Inkscape",
            "--object-path",
            "/org/inkscape/Inkscape",
            "--method",
            "org.gtk.Actions.Describe",
            INSERT_ACTION,
        ]
    )
    report.insertion_available = bool(description and description.startswith("((true,"))
    report.state = "running" if report.insertion_available else "running_helper_unavailable"
    report.ready = report.insertion_available
    report.next_steps = (
        ["Connect with live_connect(prefer='no_freeze'); live_status reports the active drawing."]
        if report.ready
        else [
            "Insertion helper is unavailable. Save and close the managed Inkscape, then reconnect.",
            "Restarting MCP alone preserves the GUI and does not reload its extension manifest.",
        ]
    )
    if session.get("context_bridge") is True:
        reply = _stdout(
            [
                paths["gdbus"] or "gdbus",
                "call",
                "--address",
                session["address"],
                "--dest",
                "org.inkscape.Inkscape",
                "--object-path",
                OBJECT_PATH,
                "--method",
                f"{INTERFACE}.ListDocuments",
            ]
        )
        try:
            rows = ast.literal_eval((reply or "").replace("@a(sss) ", ""))
            report.context_bridge_available = (
                isinstance(rows, tuple)
                and len(rows) == 1
                and isinstance(rows[0], list)
                and all(
                    isinstance(row, tuple)
                    and len(row) == 3
                    and all(isinstance(value, str) for value in row)
                    for row in rows[0]
                )
            )
        except (SyntaxError, ValueError):
            pass
        report.checks["context_bridge"] = report.context_bridge_available
        if not report.context_bridge_available:
            report.state = "running_context_unavailable"
            report.ready = False
            report.next_steps = [
                "Context bridge unavailable: save work before restarting the managed GUI.",
                "Restarting MCP alone preserves the GUI and cannot reload its GTK module.",
            ]
        elif report.ready:
            report.next_steps = [
                "Choose the task drawing with live_list_documents and live_select_document."
            ]
    elif report.ready:
        report.next_steps.append(
            "Legacy GUI has no task document guard; save work before restarting to load the bridge."
        )
    return report
