# Install the Inkscape MCP agent skill

The bundled [inkscape-mcp skill](../../skills/inkscape-mcp/SKILL.md) teaches an agent
the server's discovery, working-copy/live, preview/refinement and artifact workflow.
It uses current MCP schemas and the server's shared authoring guidance. The skill
is optional; the MCP server works without it.

From [PR #8 sources](https://github.com/P1oN/inkscape-mcp-server/pull/8), include it
when configuring/building the server:

```sh
./setup.sh --install-skill codex
# Also register the client:
./setup.sh --install-skill codex --connect-client codex
```

`codex` installs into `${CODEX_HOME:-$HOME/.codex}/skills/inkscape-mcp`.
`claude` installs into `$HOME/.claude/skills/inkscape-mcp` for Claude Code.
Default setup does not install a skill. Noninteractive setup still needs the usual
workspace/package/Inkscape options; `--install-skill` changes only skill installation.
If skill installation fails after configuration, MCP settings remain saved and the
installer explains how to retry independently.

To install separately, without configuring or executing the server:

```sh
./scripts/install-skill.sh --client codex
./scripts/install-skill.sh --client claude
```

For another compatible client or project-scoped installation, supply its skills
directory explicitly:

```sh
./scripts/install-skill.sh --destination /absolute/path/to/project/.agents/skills
```

The installer copies `SKILL.md`, `agents/openai.yaml`, a hidden upstream baseline and
an installation-owner marker. It uses shell tools, downloads nothing and changes no
client configuration. Identical content is adopted/reused; extra user files survive.
Symlinked skill trees are refused.

```sh
./scripts/install-skill.sh --client codex --update
# Or together with setup:
./setup.sh --install-skill codex --update-skill
```

Updates use a three-way merge against the saved upstream version. Unmodified files receive
the new version; nonconflicting user edits are retained. A prior version is archived beside
the skill. On conflicts, the installed files and baseline remain untouched; the installer
prints a directory containing proposed merge files for manual review. Customized legacy
skills without a baseline are preserved and require a manual comparison. Identical legacy
installs can be adopted by rerunning the installer before adding customizations.

Restart your client to discover the skill. In Codex, invoke it with `$inkscape-mcp`
or let the client select it for relevant SVG requests. Configure the MCP connection
separately with the absolute path to `run-mcp.sh` as described in the README.

Ready packages built from this source include the same installer and skill; committed
source archives include them as well. Previously published v0.1.0/v0.1.1 assets remain
unchanged and do not contain the skill.
