#!/usr/bin/env python3
"""Verify private launcher telemetry configuration, opt-out and hidden TTY input; no network."""

import argparse
import json
import os
import pty
import select
import shutil
import subprocess
import sys
import termios
import time
from pathlib import Path
from tempfile import TemporaryDirectory


def main(output):
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    dsn = "https://0123456789abcdef0123456789abcdef@sentry.invalid/123"
    with TemporaryDirectory(prefix="imcp-sentry-setup-") as temporary:
        root = Path(temporary)
        checkout = root / "checkout with spaces"
        checkout.mkdir()
        for name in ["setup.sh", "run-mcp.sh"]:
            shutil.copy2(name, checkout / name)
        package = root / "package"
        (package / "bin").mkdir(parents=True)
        (package / "libexec/inkscape-mcp").mkdir(parents=True)
        (package / "libexec/inkscape-mcp/package.json").write_text("{}")
        binary = package / "bin/inkscape-mcp"
        binary.write_text(
            "#!" + sys.executable + "\nimport json, os, sys\nfrom pathlib import Path\n"
            f"Path({str(root / 'server-started')!r}).write_text('started')\n"
            'print(json.dumps({"doctor": "--doctor" in sys.argv, '
            '"dsn": os.environ.get("SENTRY_DSN"), '
            '"environment": os.environ.get("SENTRY_ENVIRONMENT")}))\n'
        )
        binary.chmod(0o700)
        vendor = root / "vendor"
        vendor.mkdir()
        inkscape = vendor / "inkscape"
        inkscape.write_text("#!/bin/bash\nexit 0\n")
        inkscape.chmod(0o700)
        workspace = root / "workspace"
        workspace.mkdir()
        base = [
            "--package",
            str(package),
            "--workspace",
            str(workspace),
            "--inkscape",
            str(inkscape),
        ]
        env = {**os.environ, "SENTRY_DSN": "ambient-value", "SENTRY_ENVIRONMENT": "ambient"}

        def run(script, *args):
            return subprocess.run(
                ["/bin/bash", str(checkout / script), *args],
                env=env,
                capture_output=True,
                text=True,
                timeout=10,
            )

        def require(value, label):
            if not value:
                raise RuntimeError(label)
            checks.append(label)

        require(run("setup.sh", *base).returncode == 0, "legacy noninteractive setup")
        marker = root / "server-started"
        require(not marker.exists(), "default setup does not execute package code")
        require(
            run("setup.sh", *base, "--check").returncode == 0 and marker.exists(),
            "explicit setup check executes doctor",
        )
        marker.unlink()
        (checkout / "rust").mkdir()
        (checkout / "rust/Cargo.toml").write_text("synthetic source checkout")
        require(
            run("setup.sh", *base[2:]).returncode == 0 and not marker.exists(),
            "rerun reuses configured package without execution or rebuild",
        )
        require(
            json.loads(run("run-mcp.sh").stdout)["dsn"] == "ambient-value",
            "legacy environment retained without local setting",
        )
        config = checkout / ".inkscape-mcp-local/sentry.conf"
        setup = checkout / ".inkscape-mcp-local/setup.conf"
        input_file = root / "dsn-input"
        input_file.write_text(dsn + "\n")
        result = run(
            "setup.sh",
            *base,
            "--sentry",
            "true",
            "--sentry-dsn-file",
            str(input_file),
            "--sentry-environment",
            "wife",
        )
        require(
            result.returncode == 0 and dsn not in result.stdout + result.stderr,
            "enable without DSN disclosure",
        )
        require(config.stat().st_mode & 0o777 == 0o600, "DSN file mode600")
        subprocess.run(["/usr/bin/git", "-C", str(checkout), "init", "--quiet"], check=True)
        ignored = subprocess.run(
            [
                "/usr/bin/git",
                "-C",
                str(checkout),
                "check-ignore",
                ".inkscape-mcp-local/sentry.conf",
            ],
            capture_output=True,
        )
        require(ignored.returncode == 0, "DSN ignored in a checkout without root gitignore")

        require(
            json.loads(run("run-mcp.sh").stdout)["environment"] == "wife",
            "wife environment and saved DSN override ambient",
        )
        require(json.loads(run("run-mcp.sh").stdout)["dsn"] == dsn, "saved DSN forwarded as data")
        saved = config.read_bytes()
        require(
            run("setup.sh", *base).returncode == 0 and config.read_bytes() == saved,
            "unattended rerun preserves telemetry",
        )
        input_file.write_text(dsn + "\nsecond line\n")
        before = setup.read_bytes()
        result = run("setup.sh", *base, "--sentry-dsn-file", str(input_file))
        require(
            result.returncode != 0
            and config.read_bytes() == saved
            and setup.read_bytes() == before,
            "multiline DSN refuses before settings writes",
        )
        input_file.write_text("$(touch SHOULD_NOT_EXIST)\n")
        result = run("setup.sh", *base, "--sentry-dsn-file", str(input_file))
        require(
            result.returncode != 0
            and config.read_bytes() == saved
            and not (root / "SHOULD_NOT_EXIST").exists(),
            "DSN injection refuses without execution or disclosure",
        )
        result = run("setup.sh", *base, "--sentry-environment", "wife\nprivate")
        require(
            result.returncode != 0 and config.read_bytes() == saved,
            "invalid environment preserves saved DSN",
        )
        config.write_bytes(saved + b"$(touch SHOULD_NOT_EXIST)\n")
        result = run("run-mcp.sh")
        require(
            result.returncode != 0 and not result.stdout,
            "extra telemetry rows refuse without stdout",
        )
        config.unlink()
        target = root / "external"
        target.write_bytes(saved)
        config.symlink_to(target)
        require(
            run("run-mcp.sh").returncode != 0
            and run("setup.sh", *base).returncode != 0
            and target.read_bytes() == saved,
            "linked telemetry refuses and preserves external target",
        )
        config.unlink()
        config.write_bytes(saved)
        require(
            run("setup.sh", *base, "--sentry", "false").returncode == 0
            and json.loads(run("run-mcp.sh").stdout)["dsn"] is None,
            "explicit opt-out overrides ambient DSN",
        )
        require(config.read_text().splitlines()[2] == "", "opt-out removes stored DSN")

        # Drive the genuine terminal path, waiting until echo is disabled before DSN input.
        master, slave = pty.openpty()
        process = subprocess.Popen(
            ["/bin/bash", str(checkout / "setup.sh"), *base],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
        )
        os.close(slave)
        transcript = bytearray()
        try:
            for prompt, answer in [
                ("Enable Sentry error reporting?", "y"),
                ("Sentry DSN (hidden", dsn),
                ("Sentry environment label", "wife"),
            ]:
                deadline = time.monotonic() + 10
                while prompt.encode() not in transcript:
                    if time.monotonic() >= deadline:
                        raise RuntimeError("terminal prompt timeout")
                    if select.select([master], [], [], 0.1)[0]:
                        chunk = os.read(master, 65536)
                        if not chunk:
                            raise RuntimeError("terminal closed before prompt")
                        transcript.extend(chunk)
                if prompt.startswith("Sentry DSN"):
                    while termios.tcgetattr(master)[3] & termios.ECHO:
                        if time.monotonic() >= deadline:
                            raise RuntimeError("hidden input did not disable terminal echo")
                        time.sleep(0.005)
                os.write(master, (answer + "\n").encode())
            process.wait(timeout=10)
            while select.select([master], [], [], 0.1)[0]:
                try:
                    chunk = os.read(master, 65536)
                    if not chunk:
                        break
                    transcript.extend(chunk)
                except OSError:
                    break
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)
            os.close(master)
        require(
            process.returncode == 0 and dsn.encode() not in transcript,
            "interactive DSN input is hidden and setup succeeds",
        )
        require(
            json.loads(run("run-mcp.sh").stdout)["environment"] == "wife",
            "interactive label survives launcher",
        )
    (output / "comparison.json").write_text(
        json.dumps(
            {
                "passed": True,
                "checks": checks,
                "scope": (
                    "Isolated launcher/TTY/privacy/refusal fixtures with inert executable; "
                    "no Sentry or GUI."
                ),
            },
            indent=2,
        )
        + "\n"
    )
    print(f"Sentry setup acceptance: {len(checks)} checks")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    main(parser.parse_args().output)
