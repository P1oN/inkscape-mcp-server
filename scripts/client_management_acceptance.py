#!/usr/bin/env python3
"""Client configuration guards with real MCP and an isolated home; Claude CLI is synthetic."""

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import require


def main(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    helper = Path("scripts/mcp_client.py").resolve()
    checks = []
    with TemporaryDirectory(prefix="imcp-client-guards-") as temporary:
        root = Path(temporary)
        home = root / "home"
        home.mkdir()
        repo = root / "source space"
        repo.mkdir()
        (repo / ".inkscape-mcp-local").mkdir()
        launcher = repo / "run-mcp.sh"
        launcher.write_text("#!/bin/sh\nexec " + shlex.quote(str(binary.resolve())) + ' "$@"\n')
        launcher.chmod(0o700)
        vendor = root / "vendor"
        vendor.mkdir()
        cli = vendor / "claude"
        cli.write_text(
            f"#!{sys.executable}\n"
            "import sys,json\nfrom pathlib import Path\n"
            "a=sys.argv[1:];p=Path.home()/'.claude.json'\n"
            "d=json.loads(p.read_text()) if p.exists() else {'unrelated':{'keep':True}}\n"
            "s=d.setdefault('mcpServers',{})\n"
            "if a[:2]==['mcp','add']:\n"
            " assert a[2:8]==['--transport','stdio','--scope','user','inkscape','--']\n"
            " s['inkscape']={'type':'stdio','command':a[8],'args':[]}\n"
            "elif a==['mcp','remove','inkscape','--scope','user']: s.pop('inkscape',None)\n"
            "else: sys.exit(2)\n"
            "p.write_text(json.dumps(d))\n"
        )
        cli.chmod(0o700)
        codex = shutil.which("codex")
        require(codex, "Real Codex CLI required for isolated config guards")
        (vendor / "codex").symlink_to(codex)
        env = {
            **os.environ,
            "HOME": str(home),
            "CODEX_HOME": str(home / ".codex"),
            "PATH": str(vendor) + ":/usr/bin:/bin",
            "SENTRY_DSN": "",
            "INKSCAPE_MCP_WORKSPACE_ROOTS": str(root),
            "INKSCAPE_MCP_LIVE_ENABLED": "false",
        }

        def run(client, action, success=True):
            result = subprocess.run(
                [
                    sys.executable,
                    "-I",
                    str(helper),
                    "--repo",
                    str(repo),
                    "--client",
                    client,
                    action,
                ],
                env=env,
                text=True,
                capture_output=True,
                timeout=45,
            )
            require((result.returncode == 0) == success, result.stderr or result.stdout)
            return result

        for client in ("codex", "claude"):
            config = home / (".codex/config.toml" if client == "codex" else ".claude.json")
            config.parent.mkdir(exist_ok=True)
            config.write_text(
                'model = "user-model"\n' if client == "codex" else '{"unrelated":{"keep":true}}'
            )
            before = config.read_bytes()
            snippet = run(client, "config").stdout
            require(
                str(launcher) in snippet and config.read_bytes() == before,
                "config action mutated client",
            )
            run(client, "connect")
            installed = config.read_bytes()
            run(client, "connect")
            require(config.read_bytes() == installed, "idempotent connect modified config")
            retained = (
                "user-model" in config.read_text()
                if client == "codex"
                else json.loads(config.read_text())["unrelated"]["keep"]
            )
            require(retained, "unrelated client data lost")
            config.write_text(config.read_text().replace(str(launcher), "/foreign/run-mcp.sh"))
            foreign = config.read_bytes()
            run(client, "connect", False)
            run(client, "disconnect", False)
            require(config.read_bytes() == foreign, "foreign entry overwritten or removed")
            config.write_bytes(installed)
            run(client, "disconnect")
            checks.append(
                client + ": snippet/idempotence/unrelated settings/foreign ownership/remove"
            )
        record = repo / ".inkscape-mcp-local/clients.json"
        record.unlink()
        external = root / "external.json"
        external.write_text("[]")
        record.symlink_to(external)
        before = (home / ".codex/config.toml").read_bytes()
        run("codex", "connect", False)
        require(
            external.read_text() == "[]" and (home / ".codex/config.toml").read_bytes() == before,
            "linked record mutated",
        )
        checks.append("symlinked client record refused before registration")
        record.unlink()
        launcher.write_text("#!/bin/sh\nexit 0\n")
        run("codex", "connect", False)
        require(
            (home / ".codex/config.toml").read_bytes() == before, "failed handshake changed config"
        )
        checks.append("failed handshake does not register client")
    report = {"checks": checks, "claude_cli": "synthetic", "codex_cli": "real isolated profile"}
    (output / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(checks))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.binary, args.output)
