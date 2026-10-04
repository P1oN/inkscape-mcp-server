#!/usr/bin/env python3
"""Extract source, build, install skill, register an isolated Codex client and render."""

import argparse
import json
import os
import shutil
import subprocess
import tarfile
from pathlib import Path

from migration_probe import Wire, data, require


def main(archive, output):
    repo = Path.cwd()
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    extracted = output / "extracted source"
    extracted.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(extracted, filter="data")
    source = extracted / "inkscape-mcp-source-bootstrap"
    require(not (source / ".git").exists(), "archive contains Git metadata")
    require(not (source / ".inkscape-mcp-local").exists(), "archive contains local settings")
    home = output / "isolated-home"
    home.mkdir()
    workspace = output / "workspace"
    workspace.mkdir()
    codex = shutil.which("codex")
    require(codex, "Codex CLI is required for real client registration acceptance")
    env = {
        **os.environ,
        "HOME": str(home),
        "RUSTUP_HOME": str(Path.home() / ".rustup"),
        "CARGO_HOME": str(Path.home() / ".cargo"),
        "CODEX_HOME": str(home / ".codex"),
        "PATH": str(repo.parent)
        + ":"
        + str(Path.home() / ".cargo/bin")
        + ":"
        + os.environ.get("PATH", ""),
        "INKSCAPE_MCP_BUILD_PYTHON": str(repo / ".venv/bin/python"),
        "INKSCAPE_MCP_BUILD_TARGET_DIR": str(repo / "rust/target"),
        "SENTRY_DSN": "",
    }

    def command(*args, success=True):
        result = subprocess.run(
            args, cwd=source, env=env, text=True, capture_output=True, timeout=600
        )
        with (output / "commands.log").open("a") as log:
            log.write(str(args) + "\n" + result.stdout + result.stderr)
        require((result.returncode == 0) == success, "unexpected command status: " + str(args))
        return result

    command(
        str(source / "setup.sh"),
        "--local-tools",
        "--workspace",
        str(workspace),
        "--inkscape",
        "/Applications/Inkscape.app",
        "--live",
        "false",
        "--sentry",
        "false",
        "--install-skill",
        "codex",
        "--connect-client",
        "codex",
    )
    checks = [
        "extracted Git-free unpublished source archive",
        "real native setup build with existing pinned host tools",
        "skill installation",
        "real Codex CLI user registration",
        "handshake/discovery/first workspace request",
    ]
    config = home / ".codex/config.toml"
    require(str(source / "run-mcp.sh") in config.read_text(), "wrong client launcher")
    command(str(source / "scripts/mcp-client.sh"), "--client", "codex", "connect")
    version = command(str(source / "setup.sh"), "--version").stdout
    require(
        '"build_id"' in version and '"revision"' in version, "installed version metadata missing"
    )
    checks.append("idempotent registration and installed version")
    skill = home / ".codex/skills/inkscape-mcp"
    skill_file = skill / "SKILL.md"
    skill_file.write_text(skill_file.read_text() + "\nUser-specific retained instruction.\n")
    upstream = source / "skills/inkscape-mcp/SKILL.md"
    upstream.write_text(upstream.read_text().replace("# Inkscape", "# Updated Inkscape", 1))
    command(str(source / "scripts/install-skill.sh"), "--client", "codex", "--update")
    require(
        "User-specific retained" in skill_file.read_text()
        and "# Updated Inkscape" in skill_file.read_text(),
        "three-way skill merge lost content",
    )
    baseline = skill / ".inkscape-mcp-upstream/SKILL.md"
    skill_file.write_text(
        skill_file.read_text().replace("# Updated Inkscape", "# User Inkscape", 1)
    )
    upstream.write_text(
        upstream.read_text().replace("# Updated Inkscape", "# Supplier Inkscape", 1)
    )
    before = skill_file.read_bytes()
    require("# Updated Inkscape" in baseline.read_text(), "baseline not advanced")
    command(
        str(source / "scripts/install-skill.sh"), "--client", "codex", "--update", success=False
    )
    require(skill_file.read_bytes() == before, "conflicting merge changed installed skill")
    checks.append("skill customizations merge and conflicts preserve installed bytes")
    wire = Wire([str(source / "run-mcp.sh")], env, output / "wire.log")
    try:
        wire.initialize()
        doc = data(wire.call("create_document", {"width": 64, "height": 64}))["doc_id"]
        data(
            wire.call(
                "create_rect",
                {"doc_id": doc, "x": 4, "y": 4, "width": 40, "height": 40, "fill": "blue"},
            )
        )
        require(
            not wire.call("render_preview", {"doc_id": doc})["result"].get("isError"),
            "real render failed",
        )
        data(wire.call("save_document_as", {"doc_id": doc, "dest_path": "first.svg"}))
        require((workspace / "first.svg").is_file(), "first SVG not saved")
    finally:
        wire.close()
    checks.append("real create/edit/Inkscape render/save request")
    saved = (workspace / "first.svg").read_bytes()
    command(str(source / "uninstall.sh"), "--client", "codex")
    require(not (source / ".inkscape-mcp-local").exists(), "installation settings remained")
    require(
        not skill.exists() and (workspace / "first.svg").read_bytes() == saved,
        "uninstall changed drawings or left owned skill",
    )
    backups = list(source.glob(".inkscape-mcp-backup-*"))
    require(
        len(backups) == 1 and (backups[0] / "removed-skill/SKILL.md").read_bytes() == before,
        "uninstall did not preserve customized skill",
    )
    package = Path((backups[0] / "setup.conf").read_text().splitlines()[1]).parents[1]
    # The archive move relocates an internal package. Reinstall selects its actual new location.
    old_local = source / ".inkscape-mcp-local"
    package = backups[0] / package.relative_to(old_local)
    command(
        str(source / "setup.sh"),
        "--package",
        str(package),
        "--workspace",
        str(workspace),
        "--inkscape",
        "/Applications/Inkscape.app",
        "--live",
        "false",
        "--sentry",
        "false",
        "--install-skill",
        "codex",
        "--connect-client",
        "codex",
    )
    checks.append(
        "owned uninstall archives skill/settings/builds, preserves drawing, then clean reinstall"
    )
    report = {
        "checks": checks,
        "source": str(source),
        "package": str(package),
        "native_gui_acceptance": False,
        "clean_machine": False,
        "source_state": "uncommitted working tree",
    }
    (output / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    main(args.archive, args.output)
