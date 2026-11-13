# Manager stale recovery and launch update checks — 2026-11-13

Issue: [#61](https://github.com/P1oN/inkscape-mcp-server/issues/61).
Worktree: `codex/manager-recovery-startup`, based on `c97fb36`.
The original checkout's unrelated uncommitted changes remain untouched.

## Implementation

Read-only discovery exposes the effective profile/configuration (honoring CODEX_HOME),
existing installation candidates and explicit uncertainty. A stale legacy source must
be absent and agree with the exact transport binding, skill owner and known baseline.
Known baseline bytes come from compiled project files or a fully validated staged text
bundle. Names alone do not authorize adoption. Discovered commands are never executed.

Review stages complete private config and skill backups and an existing three-way merge.
Only Install now publishes registration/skill owner/settings/selector changes through
the installer journal. The original baseline, additional files and customizations survive
in the complete backup. Conflicts refuse before publication; whole-skill/config snapshots
and source absence are rechecked before activation. Partial publication restores the
registration and skill owner together. Unknown bindings/skills remain preserved.

Manager performs one asynchronous launch check, plus manual checks. It uses the fixed
GitHub repository, release contracts, channel and target compatibility. Valid incompatible
releases are skipped; malformed manifests still fail. A separate task has a 20-second
deadline and termination escalation; local setup stays usable offline. Fresh profiles
compare sealed offline component metadata without settings.json. First installation
retains that distribution identity, including distributions reusing older runtime bytes.

Generation/root/channel/review/activation guards discard stale UI replies; backend
snapshots also detect external installation changes during release discovery. Component
actions show a plan before confirmation and pin the entire target and release manifest,
including content/archive hashes. Same-release partial component updates can complete;
implicit checks do not offer older semantic versions. MCP startup/reconnect remains offline.

## Validation and independent review

Validation results are recorded after completion of the final checks. Native AppKit
state/layout and task-I/O fixtures are automated checks, not interactive GUI acceptance.

The independent agent's first pass found four issues: same-tag partial updates, incomplete
review identity, first-install external-change race and incompatible-latest discovery.
The second pass also found channel changes routing the primary action into offline setup.
All five were addressed with dedicated regression coverage. The final independent pass
found no remaining blocking findings and independently passed strict Clang syntax and
diff whitespace checks. It did not perform interactive GUI acceptance.

## Scope and retention

No real user installation, profile, skill, drawing, Manager/Inkscape session, system
permissions, Gatekeeper or quarantine settings are changed. Isolated profiles and owned
fixtures are under `/private/tmp/inkscape-manager-*`; their ownership/recreation policy
is in `/private/tmp/inkscape-manager-task.json`. Cargo caches have a combined 6 GiB budget.
Successful disposable packages/profiles are removed after compact evidence is retained.

Interactive Manager review/activation, signed-final-byte behavior, Gatekeeper/browser
download and another-Mac acceptance remain unverified for this change. No release is
published. MCP schemas and authoring instructions are unchanged, so documentation-catalog
regeneration and restarting the user's MCP server are unnecessary.
