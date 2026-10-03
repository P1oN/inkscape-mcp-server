#!/usr/bin/env python3
"""Exercise shell onboarding with a real package and literal/untrusted configuration paths."""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import Wire, data, require, write


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    package = args.package.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    checks = []
    with TemporaryDirectory(prefix="imcp-launcher-") as temporary:
        root = Path(temporary)
        source = root / "checkout with spaces"
        source.mkdir()
        for script in ("setup.sh", "run-mcp.sh"):
            packaged = package / script
            if packaged.is_file():
                require(
                    packaged.read_bytes() == Path(script).read_bytes(), "stale packaged launcher"
                )
            shutil.copy2(packaged if packaged.is_file() else Path(script), source / script)
        workspace = root / "SVGs $(touch SHOULD_NOT_EXIST); 'quoted'"
        workspace.mkdir()
        original = (
            '<svg xmlns="http://www.w3.org/2000/svg"><rect id="r" width="2" height="3"/></svg>'
        )
        (workspace / "fixture.svg").write_text(original)
        environment = {
            **os.environ,
            "PATH": "",
            "INKSCAPE_MCP_RAW_ACTION_ENABLED": "false",
            "INKSCAPE_MCP_TOOL_PROFILE": "full",
            "INKSCAPE_MCP_TOOL_DESC": "full",
        }

        def invoke(script, *arguments):
            result = subprocess.run(
                ["/bin/bash", str(source / script), *map(str, arguments)],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
                timeout=60,
            )
            return {"exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr}

        missing = invoke("run-mcp.sh")
        require(missing["exit"] != 0 and not missing["stdout"], "missing config must fail silently")
        checks.append("missing configuration refuses without stdout")
        configured = invoke("setup.sh", "--package", package, "--workspace", workspace)
        write(args.output / "setup.json", configured)
        require(configured["exit"] == 0 and not configured["stdout"], "real package setup failed")
        config = source / ".inkscape-mcp-local/setup.conf"
        stored = config.read_bytes()
        require(config.stat().st_mode & 0o777 == 0o600, "configuration is not private")
        require(str(workspace.resolve()) in config.read_text(), "literal workspace changed")
        checks.append("config-only literal-path setup with empty user tool PATH")
        wire = Wire(
            ["/bin/bash", str(source / "run-mcp.sh")], environment, args.output / "mcp.stderr.log"
        )
        try:
            wire.initialize()
            tools = wire.request("tools/list")["result"]["tools"]
            # The ordinary default keeps the advanced gate disabled.
            expected = json.loads(
                Path("migration/contracts/live-true_raw-false_full_full.json").read_text()
            )["tools/list"]["tools"]
            require(tools == expected, "default tools differ from the frozen contract")
            document = data(wire.call("open_document", {"path": "fixture.svg"}))["doc_id"]
            data(wire.call("inspect_document", {"doc_id": document}))
            write(args.output / "mcp.trace.json", wire.trace)
        finally:
            wire.close()
        require((workspace / "fixture.svg").read_text() == original, "original changed")
        require(not (root / "SHOULD_NOT_EXIST").exists(), "configuration was interpreted as shell")
        checks.append("real STDIO startup/discovery/open/inspect and original preservation")
        lines = stored.decode().splitlines()
        for name, content in (
            ("truncated", "\n".join(lines[:3]) + "\n"),
            ("trailing", stored.decode() + "$(touch SHOULD_NOT_EXIST)\n"),
            ("engine", "\n".join([*lines[:5], "$(touch SHOULD_NOT_EXIST)"]) + "\n"),
            ("workspace", "\n".join([*lines[:3], str(root) + ":/", *lines[4:]]) + "\n"),
        ):
            config.write_text(content)
            refused = invoke("run-mcp.sh")
            write(args.output / f"refused-{name}.json", refused)
            require(refused["exit"] != 0 and not refused["stdout"], f"accepted {name} config")
            require(not (root / "SHOULD_NOT_EXIST").exists(), "invalid config executed shell")
            checks.append(f"{name} configuration refuses without execution/stdout")
        config.write_bytes(stored)
        for name, arguments in (
            ("colon", ["--workspace", root / "bad:workspace"]),
            ("newline", ["--workspace", str(root) + "\n/"]),
            ("invalid-mode", ["--workspace", workspace, "--engine", "invalid"]),
        ):
            (root / "bad:workspace").mkdir(exist_ok=True)
            refused = invoke("setup.sh", "--package", package, *arguments)
            require(refused["exit"] != 0 and config.read_bytes() == stored, f"setup damaged {name}")
            checks.append(f"{name} setup failure preserves saved configuration")
        config.unlink()
        sentinel = root / "sentinel"
        sentinel.write_bytes(stored)
        config.symlink_to(sentinel)
        refused = invoke("run-mcp.sh")
        require(refused["exit"] != 0 and not refused["stdout"], "accepted linked config")
        require(sentinel.read_bytes() == stored, "external configuration changed")
        checks.append("linked configuration refuses and preserves target")
    write(
        args.output / "comparison.json",
        {
            "passed": True,
            "checks": checks,
            "package": str(package),
            "binary_sha256": hashlib.sha256(
                (package / "bin/inkscape-mcp").read_bytes()
            ).hexdigest(),
            "script_sha256": {
                script: hashlib.sha256(Path(script).read_bytes()).hexdigest()
                for script in ("setup.sh", "run-mcp.sh")
            },
            "scope": "actual package/CLI/STDIO; no native GUI or source build acceptance",
        },
    )
    print(f"Launcher acceptance: {len(checks)} checks")


if __name__ == "__main__":
    main()
