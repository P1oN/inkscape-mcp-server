#!/usr/bin/env python3
"""Verify private launcher telemetry configuration, opt-out and hidden TTY input; no network."""

import argparse
import json
import os
import pty
import select
import shlex
import shutil
import subprocess
import sys
import termios
import time
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_build_macos_package import bundle_agent_skill


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
        bundle_agent_skill(checkout)
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
        for helper in ("client", "supervisor", "inx", "live"):
            shutil.copy2(binary, binary.parent / f"inkscape-mcp-{helper}")
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
        env = {
            **os.environ,
            "SENTRY_DSN": "ambient-value",
            "SENTRY_ENVIRONMENT": "ambient",
            "HOME": str(root / "home"),
            "CODEX_HOME": str(root / "custom codex home"),
        }

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
        client_script = checkout / "scripts/mcp-client.sh"
        saved_client_script = client_script.read_bytes()
        client_script.write_text("#!/bin/bash\necho 'injected connection failure' >&2\nexit 1\n")
        result = run("setup.sh", *base, "--connect-client", "codex")
        require(
            result.returncode != 0
            and (checkout / ".inkscape-mcp-local/setup.conf").is_file()
            and "MCP settings were saved, but client connection failed" in result.stderr
            and "scripts/mcp-client.sh --client codex connect" in result.stderr,
            "failed client connection retains settings and explains standalone retry",
        )
        client_script.write_bytes(saved_client_script)
        codex_skill = Path(env["CODEX_HOME"]) / "skills/inkscape-mcp"
        require(not codex_skill.exists(), "skill installation is opt-in")
        result = run("setup.sh", *base, "--install-skill", "codex")
        require(
            result.returncode == 0
            and (codex_skill / "SKILL.md").read_bytes()
            == (checkout / "skills/inkscape-mcp/SKILL.md").read_bytes()
            and (codex_skill / "agents/openai.yaml").is_file()
            and not marker.exists(),
            "packaged setup installs skill in custom CODEX_HOME without runtime execution",
        )
        require(
            run("setup.sh", *base, "--install-skill", "codex").returncode == 0,
            "identical skill reinstall succeeds",
        )
        saved_skill = b"user-customized skill\n"
        (codex_skill / "SKILL.md").write_bytes(saved_skill)
        require(
            run("setup.sh", *base, "--install-skill", "codex").returncode != 0
            and (codex_skill / "SKILL.md").read_bytes() == saved_skill,
            "different existing skill preserved",
        )
        (codex_skill / "SKILL.md").unlink()
        sentinel = root / "skill-sentinel"
        sentinel.write_bytes(saved_skill)
        (codex_skill / "SKILL.md").symlink_to(sentinel)
        require(
            run("scripts/install-skill.sh", "--client", "codex").returncode != 0
            and sentinel.read_bytes() == saved_skill,
            "existing skill file symlink refused without touching target",
        )
        require(
            run("setup.sh", *base, "--install-skill", "claude").returncode == 0
            and (Path(env["HOME"]) / ".claude/skills/inkscape-mcp/SKILL.md").is_file()
            and not marker.exists(),
            "Claude skill installation uses isolated home without runtime execution",
        )
        custom_skills = root / "project skills"
        require(
            run("scripts/install-skill.sh", "--destination", str(custom_skills)).returncode == 0
            and (custom_skills / "inkscape-mcp/SKILL.md").is_file(),
            "standalone installation supports explicit skills directory",
        )
        symlink_skills = root / "linked skills"
        symlink_skills.symlink_to(custom_skills, target_is_directory=True)
        require(
            run("scripts/install-skill.sh", "--destination", str(symlink_skills)).returncode != 0,
            "symlink destination refused",
        )
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
        build_marker = root / "builder-called"
        builder_scripts = checkout / "scripts"
        for name, mode in (
            ("bootstrap-local-package.sh", "auto"),
            ("build-local-package.sh", "local"),
        ):
            expected = "false" if mode == "auto" else "true"
            builder = builder_scripts / name
            builder.write_text(
                "#!/bin/bash\nset -eu\n"
                f'[ "$INKSCAPE_MCP_BUILD_LOCAL_TOOLS_ONLY" = {expected} ]\n'
                f"printf '%s' {mode} > {shlex.quote(str(build_marker))}\n"
                f"printf '%s\\n' {shlex.quote(str(package))}\n"
            )
            builder.chmod(0o700)
        setup_config = checkout / ".inkscape-mcp-local/setup.conf"
        setup_config.unlink()
        require(
            run("setup.sh", *base[2:]).returncode == 0
            and build_marker.read_text() == "auto"
            and not marker.exists(),
            "fresh source setup automatically chooses provisioning without starting runtime",
        )
        build_marker.unlink()
        require(
            run("setup.sh", *base[2:]).returncode == 0 and not build_marker.exists(),
            "repeat source setup skips both builders",
        )
        require(
            run("setup.sh", *base[2:], "--local-tools").returncode == 0
            and build_marker.read_text() == "local",
            "local-tools explicitly rebuilds with offline tools-only policy",
        )
        build_marker.unlink()
        require(
            run("setup.sh", *base).returncode == 0 and not build_marker.exists(),
            "explicit ready package skips source builders",
        )
        saved_setup = setup_config.read_bytes()
        require(
            run("setup.sh", *base, "--local-tools").returncode != 0
            and run("setup.sh", *base[2:], "--local-tools", "--bootstrap").returncode != 0
            and setup_config.read_bytes() == saved_setup
            and not build_marker.exists(),
            "conflicting build and package options fail before mutation",
        )
        for option, expected in (
            ("--build", "local"),
            ("--bootstrap", "auto"),
            ("--rebuild", "auto"),
        ):
            require(
                run("setup.sh", *base[2:], option).returncode == 0
                and build_marker.read_text() == expected,
                "legacy build option remains compatible: " + option,
            )
        build_marker.unlink()
        require(
            run("setup.sh", "--live", "false", "--engine", "shell").returncode == 0
            and run("setup.sh").returncode == 0
            and setup_config.read_text().splitlines()[4:6] == ["false", "shell"]
            and not build_marker.exists(),
            "plain rerun preserves workspace Inkscape live and engine settings",
        )
        require(
            run("setup.sh", "--live", "true", "--engine", "per_call").returncode == 0
            and setup_config.read_text().splitlines()[4:6] == ["true", "per_call"],
            "explicit options override saved settings",
        )
        saved_setup = setup_config.read_bytes()
        invalid_dsn = root / "bad-dsn-early"
        invalid_dsn.write_text("not-a-dsn\n")
        for invalid in (
            ["--sentry", "invalid"],
            ["--sentry-environment", "bad\nlabel"],
            ["--sentry-dsn-file", str(invalid_dsn)],
            ["--sentry", "false", "--sentry-environment", "wife"],
            ["--engine", ""],
        ):
            require(
                run("setup.sh", "--bootstrap", *invalid).returncode != 0
                and not build_marker.exists()
                and setup_config.read_bytes() == saved_setup,
                "invalid telemetry fails before build: "
                + invalid[0]
                + ":"
                + (invalid[1].splitlines()[0] if invalid[1] else "<empty>"),
            )
        setup_config.unlink()
        setup_config.mkdir()
        require(
            run("setup.sh", *base).returncode != 0
            and not list(setup_config.iterdir())
            and not build_marker.exists(),
            "directory configuration refuses without false success or build",
        )
        setup_config.rmdir()
        setup_config.write_bytes(saved_setup)
        manifest = package / "libexec/inkscape-mcp/package.json"
        manifest.write_text(json.dumps({"source_head": "1" * 40}, indent=2) + "\n")
        source_revision = checkout / "SOURCE_REVISION"
        source_revision.write_text("inkscape-mcp-source-v1\n" + "1" * 40 + "\n")
        require(
            run("setup.sh").returncode == 0 and not build_marker.exists(),
            "same source revision reuses installed runtime",
        )
        source_revision.write_text("inkscape-mcp-source-v1\n" + "2" * 40 + "\n")
        require(
            run("setup.sh").returncode == 0 and build_marker.read_text() == "auto",
            "changed committed source revision triggers automatic rebuild",
        )
        build_marker.unlink()
        require(
            run("setup.sh", "--package", str(package)).returncode == 0
            and not build_marker.exists(),
            "explicit package wins over source revision mismatch",
        )
        (builder_scripts / "bootstrap-local-package.sh").write_text("#!/bin/bash\nexit 22\n")
        saved_setup = setup_config.read_bytes()
        require(
            run("setup.sh", *base[2:], "--bootstrap").returncode != 0
            and setup_config.read_bytes() == saved_setup,
            "failed build preserves saved configuration",
        )
        require(
            run("setup.sh").returncode != 0 and setup_config.read_bytes() == saved_setup,
            "failed automatic update preserves saved runtime configuration",
        )
        source_revision.unlink()
        manifest.write_text("{}")
        (checkout / "bin").mkdir()
        shutil.copy2(binary, checkout / "bin/inkscape-mcp")
        for helper in ("client", "supervisor", "inx", "live"):
            shutil.copy2(binary, checkout / f"bin/inkscape-mcp-{helper}")
        (checkout / "libexec/inkscape-mcp").mkdir(parents=True)
        (checkout / "libexec/inkscape-mcp/package.json").write_text("{}")
        require(
            run("setup.sh", *base[2:]).returncode == 0
            and str(checkout / "bin/inkscape-mcp") in setup_config.read_text()
            and not build_marker.exists()
            and not marker.exists(),
            "unpacked ready package default setup skips all builds and runtime execution",
        )
        shutil.rmtree(checkout / "bin")
        shutil.rmtree(checkout / "libexec")
        setup_config.write_bytes(saved_setup)
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
