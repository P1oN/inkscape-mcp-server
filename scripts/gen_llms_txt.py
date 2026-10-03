#!/usr/bin/env python3
"""Generate agent manifests from the actual Rust MCP STDIO surface; no GUI launch."""

import argparse
import json
import os
import tempfile
from pathlib import Path

from migration_probe import Wire, require


def generate(binary, output):
    env = {
        **os.environ,
        "INKSCAPE_MCP_TOOL_PROFILE": "full",
        "INKSCAPE_MCP_TOOL_DESC": "full",
        "INKSCAPE_MCP_LIVE_ENABLED": "true",
        "INKSCAPE_MCP_RAW_ACTION_ENABLED": "true",
    }
    with tempfile.TemporaryDirectory(prefix="inkscape-manifest-") as temporary:
        env["INKSCAPE_MCP_WORKSPACE_ROOTS"] = temporary
        env["INKSCAPE_MCP_MANAGED_DIR"] = str(Path(temporary) / "absent-session")
        env["INKSCAPE_MCP_LIVE_RENDEZVOUS"] = str(Path(temporary) / "absent-rendezvous")
        wire = Wire([str(binary.resolve())], env, Path(temporary) / "stderr.log")
        try:
            initialized = wire.initialize()
            surface = {}
            for method in (
                "tools/list",
                "prompts/list",
                "resources/list",
                "resources/templates/list",
            ):
                reply = wire.request(method, {})
                require(not reply.get("error"), "discovery failed")
                surface[method] = reply["result"]
            require(not (Path(temporary) / "absent-session").exists(), "startup mutated session")
        finally:
            wire.close()
    tools = surface["tools/list"]["tools"]
    prompts = surface["prompts/list"]["prompts"]
    resources = surface["resources/list"]["resources"]
    templates = surface["resources/templates/list"]["resourceTemplates"]
    header = (
        "# inkscape-mcp: Rust STDIO server\n\nRun ./setup.sh then ./run-mcp.sh. "
        "See README.md and docs/agent-usage-guide.md.\n"
        "Python is used only for private live helpers and development/packaging tools.\n"
    )
    index = [header, "\n## Tools\n"]
    for tool in sorted(tools, key=lambda t: t["name"]):
        purpose = tool.get("description", "").split("\n")[0]
        index.append(f"- {tool['name']}: {purpose}\n")
    for title, values in (
        ("Prompts", prompts),
        ("Resources", resources),
        ("Resource templates", templates),
    ):
        index.append(f"\n## {title}\n")
        index.extend(
            f"- {item['name']}: {item.get('uri', item.get('uriTemplate', ''))}".rstrip() + "\n"
            for item in values
        )
    output.mkdir(parents=True, exist_ok=True)
    (output / "llms.txt").write_text("".join(index))
    (output / "llms-full.txt").write_text(
        header
        + "\n## Actual MCP initialization and surface\n\n"
        + json.dumps({"initialize": initialized, **surface}, ensure_ascii=False, indent=2)
        + "\n"
    )
    print(
        f"Generated {len(tools)} tools, {len(prompts)} prompts, "
        f"{len(resources) + len(templates)} resources"
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary", type=Path, default=Path("rust/target/release/inkscape-mcp-rust")
    )
    parser.add_argument("--output", type=Path, default=Path("."))
    args = parser.parse_args()
    generate(args.binary, args.output)
