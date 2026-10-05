#!/usr/bin/env python3
"""Doctor readonly/provisioning/failure checks using an owned relocated package."""

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_package_acceptance import verify_files
from migration_probe import require, write


def tree(root):
    return sorted(str(path.relative_to(root)) for path in root.rglob("*"))


def check(executable, env, label, output, expected):
    result = subprocess.run([str(executable), "--doctor"], env=env, capture_output=True, timeout=40)
    report = json.loads(result.stdout)
    require(
        report["ready"] == expected and result.returncode == (0 if expected else 1),
        "doctor ready/exit drift",
    )
    require(report["native_gui_verified"] is False, "doctor claimed GUI acceptance")
    require(
        "clang" not in report["checks"] and "glib_headers" not in report["checks"],
        "doctor requires user compiler",
    )
    write(output / (label + ".json"), report)
    return report


def main(package, output, report_path):
    output.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory(prefix="imcp-doctor-", dir=Path("/tmp").resolve()) as temp:  # noqa: S108
        root = Path(temp).resolve()
        installed = root / "installed"
        shutil.copytree(package, installed, symlinks=True)
        executable = installed / "bin/inkscape-mcp"
        library = installed / "libexec/inkscape-mcp"
        home = root / "home"
        home.mkdir()
        env = {
            "PATH": "",
            "HOME": str(home),
            "TMPDIR": str(root),
            "INKSCAPE_MCP_MANAGED_DIR": str(root / "never-created"),
        }
        before = tree(root)
        baseline = check(executable, env, "ready", output, True)
        require(all(baseline["checks"].values()), "failed ready prerequisites")
        require(not (library / "python").exists(), "doctor fixture contains Python")
        require(
            not any("python" in key or "inkex" in key for key in baseline["checks"]),
            "doctor still requires Python/inkex",
        )
        require(tree(root) == before, f"doctor changed paths: {set(tree(root)) - set(before)}")
        verify_files(installed)
        observations = 1
        missing_cases = [
            ("missing-supervisor", "../../bin/inkscape-mcp-supervisor", "fixed_supervisor"),
            ("missing-bridge", "context.so", "prebuilt_context_architecture"),
            ("missing-helper", "helpers/inkscape_mcp_insert.inx", "fixed_helper_assets"),
            ("missing-socket", "../../bin/inkscape-mcp-live", "fixed_socket_helper"),
            ("missing-inx", "../../bin/inkscape-mcp-inx", "fixed_inx_helper"),
            ("missing-bus", "dbus/bin/dbus-daemon", "dbus_daemon_architecture"),
            ("missing-gdbus", "dbus/bin/gdbus", "gdbus_architecture"),
            ("missing-manifest", "package.json", "package_manifest"),
        ]
        if sys.platform != "darwin":
            missing_cases = [case for case in missing_cases if case[0] != "missing-bridge"]
        for label, relative, key in missing_cases:
            path = library / relative
            held = root / "held-asset"
            path.rename(held)
            try:
                before = tree(root)
                report = check(executable, env, label, output, False)
                require(not report["checks"][key], "missing component not named")
                require(tree(root) == before, "failure diagnosis mutated setup")
            finally:
                held.rename(path)
            observations += 1
        relative = "context.so" if sys.platform == "darwin" else "../../bin/inkscape-mcp-inx"
        key = "prebuilt_context_architecture" if sys.platform == "darwin" else "fixed_inx_helper"
        path = library / relative
        held = root / "held-asset"
        path.rename(held)
        try:
            path.symlink_to(held)
            report = check(executable, env, "linked-bridge", output, False)
            require(
                not report["checks"][key],
                "doctor followed linked bridge",
            )
            path.unlink()
            path.write_bytes(b"bad architecture".ljust(32, b"\x00"))
            report = check(executable, env, "wrong-bridge", output, False)
            require(
                not report["checks"][key],
                "doctor accepted malformed module",
            )
        finally:
            path.unlink(missing_ok=True)
            held.rename(path)
        observations += 2
        fake = root / "bin"
        fake.mkdir()
        engine = fake / "inkscape"
        engine.write_text(
            '#!/bin/sh\n[ "$1" = --version ] || exit 73\nprintf "Inkscape 1.2.2\\n"\n'
        )
        engine.chmod(0o700)
        report = check(executable, {**env, "PATH": str(fake)}, "old-engine", output, False)
        require(report["inkscape_version"] == "Inkscape 1.2.2", "old-version fixture failed to run")
        require(not report["checks"]["inkscape_minimum_1_4"], "old engine accepted")
        observations += 1
        verify_files(installed)
        require(not (root / "never-created").exists(), "doctor launched managed session")
    result = {
        "passed": True,
        "profiles": observations,
        "ready_and_exit_codes": True,
        "read_only_no_bytecode": True,
        "missing_assets_named": True,
        "link_and_architecture_guards": True,
        "minimum_version_checked": True,
        "GUI_launched": False,
    }
    write(report_path, result)
    print(json.dumps(result))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--output", type=Path, default=Path("migration/results/doctor-acceptance"))
    parser.add_argument("--report", type=Path, default=Path("migration/doctor-comparison.json"))
    args = parser.parse_args()
    main(args.package.resolve(), args.output, args.report)
