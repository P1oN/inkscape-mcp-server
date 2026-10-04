#!/usr/bin/env python3
"""Relocated ready-package management with broken helpers and no Python on client PATH.

Python drives development assertions only; every installation command executes native
Rust/Bash with isolated client homes. Real Codex, synthetic shell Claude; no GUI.
"""

import argparse
import json
import os
import shutil
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import require


def main(package, output):
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    codex = shutil.which("codex")
    require(codex, "Real Codex CLI required")
    inkscape = next(
        (
            p
            for p in ("/Applications/Inkscape.app/Contents/MacOS/inkscape", "/usr/bin/inkscape")
            if Path(p).is_file()
        ),
        None,
    )
    require(inkscape, "Installed Inkscape required (never executed by this check)")
    with TemporaryDirectory(prefix="imcp-native-clients-") as temporary:
        root = Path(temporary).resolve()
        repo = root / "ready package space"
        shutil.copytree(package, repo, symlinks=True)
        home = root / "home"
        home.mkdir()
        codex_home = root / "custom codex home"
        codex_home.mkdir()
        vendor = root / "bin"
        vendor.mkdir()
        # Client commands have a minimal PATH without any Python executable.
        for tool in ("dirname", "sed"):
            (vendor / tool).symlink_to(shutil.which(tool))
        (vendor / "codex").symlink_to(codex)
        claude = vendor / "claude"
        claude.write_text(
            "#!/bin/sh\n"
            'if [ "$1 $2" = "mcp add" ]; then\n'
            ' [ "$3 $4 $5 $6 $7 $8" = "--transport stdio --scope user inkscape --" ] || exit 2\n'
            ' printf \'{"unrelated":{"keep":true},"mcpServers":{"inkscape":'
            '{"type":"stdio","command":"%s","args":[]}}}\\n\' "$9" > "$HOME/.claude.json"\n'
            'elif [ "$*" = "mcp remove inkscape --scope user" ]; then\n'
            ' printf \'{"unrelated":{"keep":true},"mcpServers":{}}\\n\' > "$HOME/.claude.json"\n'
            "else exit 2; fi\n"
        )
        claude.chmod(0o700)
        env = {
            **os.environ,
            "HOME": str(home),
            "CODEX_HOME": str(codex_home),
            "PATH": str(vendor),
            "SENTRY_DSN": "",
            "SENTRY_TRACES_SAMPLE_RATE": "0",
        }
        workspace = root / "drawings"
        workspace.mkdir()
        drawing = workspace / "existing.svg"
        drawing.write_text('<svg xmlns="http://www.w3.org/2000/svg"><rect id="original"/></svg>')
        original = drawing.read_bytes()

        def command(*args, success=True):
            result = subprocess.run(args, env=env, capture_output=True, text=True, timeout=50)
            require((result.returncode == 0) == success, result.stderr or result.stdout)
            return result

        command(
            str(repo / "setup.sh"),
            "--workspace",
            str(workspace),
            "--inkscape",
            inkscape,
            "--live",
            "false",
            "--sentry",
            "false",
        )
        runtime = repo / "libexec/inkscape-mcp"
        require(not (runtime / "python").exists(), "ready package still bundles Python")
        (runtime / "python").write_text("damaged helper runtime")
        (repo / "bin/inkscape-mcp-supervisor").write_text("damaged supervisor")
        checks.append("relocated ready package setup; Python runtime and supervisor damaged")
        require(not (repo / "scripts/mcp_client.py").exists(), "Python manager still bundled")
        manager = repo / "scripts/mcp-client.sh"
        configs = {"codex": codex_home / "config.toml", "claude": home / ".claude.json"}
        for client in configs:
            config = configs[client]
            config.write_text(
                'model = "user-model"\n' if client == "codex" else '{"unrelated":{"keep":true}}'
            )
            before = config.read_bytes()
            snippet = command(str(manager), "--client", client, "config").stdout
            require(
                str(repo / "run-mcp.sh") in snippet and config.read_bytes() == before,
                "snippet mutated settings",
            )
            command(str(manager), "--client", client, "check")
            command(str(manager), "--client", client, "connect")
            installed = config.read_bytes()
            command(str(manager), "--client", client, "connect")
            require(config.read_bytes() == installed, "idempotent connect mutated config")
            require(
                "user-model" in config.read_text()
                if client == "codex"
                else json.loads(config.read_text())["unrelated"]["keep"],
                "unrelated settings lost",
            )
            config.write_text(
                config.read_text().replace(str(repo / "run-mcp.sh"), "/foreign/run-mcp.sh")
            )
            foreign = config.read_bytes()
            for action in ("connect", "disconnect", "uninstall"):
                command(str(manager), "--client", client, action, success=False)
            require(config.read_bytes() == foreign, "foreign entry modified")
            config.write_bytes(installed)
            command(str(manager), "--client", client, "disconnect")
            command(str(manager), "--client", client, "disconnect")
            checks.append(
                client
                + ": config/check/connect/disconnect/idempotence/foreign ownership without Python"
            )

        for client in configs:
            command(str(manager), "--client", client, "connect")
        local = repo / ".inkscape-mcp-local"
        saved = (local / "setup.conf").read_bytes()
        command(str(repo / "uninstall.sh"), "--client", "codex", success=False)
        require((local / "setup.conf").read_bytes() == saved, "blocked uninstall removed settings")
        require(
            "inkscape" in configs["codex"].read_text(), "blocked uninstall removed selected client"
        )
        checks.append("another recorded client on the same launcher blocks before removal")
        # Disconnect target Codex; switch the other recorded client to a different installation.
        command(str(manager), "--client", "codex", "disconnect")
        configs["claude"].write_text(
            configs["claude"].read_text().replace(str(repo / "run-mcp.sh"), "/other/run-mcp.sh")
        )
        switched = configs["claude"].read_bytes()
        skill = codex_home / "skills/inkscape-mcp"
        skill.mkdir(parents=True)
        (skill / ".inkscape-mcp-owner").write_text(str(repo) + "\n")
        (skill / "SKILL.md").write_text("user customizations")
        command(str(repo / "uninstall.sh"), "--client", "codex")
        backups = list(repo.glob(".inkscape-mcp-backup-*"))
        require(len(backups) == 1 and not local.exists(), "uninstall failed to archive")
        require((backups[0] / "setup.conf").read_bytes() == saved, "archived settings changed")
        require(
            (backups[0] / "removed-skill/SKILL.md").read_text() == "user customizations",
            "owned customized skill lost",
        )
        require(configs["claude"].read_bytes() == switched, "switched client modified")
        checks.append("uninstall archives settings/owned skill and ignores switched client")
        # Restore setup from backup and exercise the Claude uninstall route too.
        backups[0].rename(local)
        configs["claude"].write_text('{"unrelated":{"keep":true}}')
        command(str(manager), "--client", "claude", "connect")
        command(str(repo / "uninstall.sh"), "--client", "claude")
        require(drawing.read_bytes() == original, "existing drawing changed")
        checks.append(
            "restored installation; Claude uninstall user scope; existing drawing preserved"
        )
    report = {
        "checks": checks,
        "python_on_client_path": False,
        "helper_runtime": "damaged",
        "codex_cli": "real isolated CODEX_HOME",
        "claude_cli": "synthetic shell",
        "native_gui_acceptance": False,
    }
    (output / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    main(args.package.resolve(), args.output)
