# Documentation

This index separates current usage, remaining work, architecture decisions and dated evidence.
Current usage distinguishes published v0.1.2 from newer source features, including opt-in
independent updates. The handoff records release-specific evidence and outstanding acceptance
as of 2026-10-07.

## Start here

| Need | Read |
| --- | --- |
| Install and run | [Repository README](../README.md), [installation index](install/README.md) |
| Drive MCP safely | [Agent usage guide](agent-usage-guide.md), [portable skill](../skills/inkscape-mcp/SKILL.md) |
| Understand current implementation and validation | [Agent handoff](AGENT_HANDOFF.md) |
| Choose the next task | [Active backlog](RUST_NEXT_PLAN.md) |
| Develop and validate | [CONTRIBUTING](../CONTRIBUTING.md), [security policy](../SECURITY.md) |
| Discover exact tools and schemas | [llms.txt](../llms.txt), [llms-full.txt](../llms-full.txt) |

## Current guides

- **Installation:** [source/package setup](install/install.md), [source prerequisites](install/local-bootstrap.md),
  [GitHub builds](install/github-builds.md), [independent updates](install/independent-updates.md),
  [client management](install/client-management.md),
  [host configuration](install/host-configs.md), [agent skill](install/agent-skill.md),
  [compatibility](install/compatibility.md), [troubleshooting](install/troubleshooting.md).
- **Live integration:** [overview](live/README.md), [managed macOS session](live/macos-live-prototype.md),
  [document context](live/document-context.md), [everyday edits](live/everyday-edits.md),
  [discovery/previews](live/live-discovery.md), [Rust helper kernels](live/live-helper-kernels.md).
- **Operations:** [Sentry monitoring](operations/sentry.md).
- **Architecture:** [decision records](adr/README.md).
- **History:** [milestones, archived plans and acceptance reports](history/README.md).

## Maintenance rules

Keep current behavior in guides, current implementation/validation in the handoff and
unfinished work in one backlog. Move completed plans and dated acceptance details into
history; record architectural rationale in ADRs rather than copying test logs into them.
Update affected links when moving a file. Keep `AGENT_HANDOFF.md`, `RUST_NEXT_PLAN.md` and
`agent-usage-guide.md` as stable entry points; `ROADMAP.md` points to the same backlog.

Historical JSON evidence and source snapshots in `migration/` retain their original paths,
hashes and labels. Moving a Markdown guide does not rewrite those immutable records.
Python source history remains beside the archived code in `scripts/history/python/`.
Generated MCP manifests describe the registry and are regenerated only when its exposed
surface or instructions change; documentation rearrangement does not change that surface.

See [computed styles and reviewed live packages](live/reviewed-workflow.md) for the new full-profile tools and the deferred artist pilot protocol.
