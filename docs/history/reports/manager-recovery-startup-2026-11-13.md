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

The complete runtime Cargo suite passes serially outside the sandbox, including the
15 update-manager regressions. The initial sandbox run refused a Unix socket; an
outside-sandbox parallel run exceeded three synthetic D-Bus fixtures' one-second
budgets. The serial run passes without relaxing any timeout or changing live behavior.
Both locked Cargo graphs pass formatting and all-target Clippy with warnings denied.
The tooling graph passes 30 tests; its separate native DMG mount test remains ignored.
Hidden AppKit state/layout and task-I/O fixtures pass, including channel changes,
unavailable/error/timeout states, stale replies and component preflight routing.
These are automated checks, not interactive GUI acceptance.

The release package (build `d9fb7a4d6a4014b6`, based on the unpublished `c97fb36`
working-tree snapshot) passes isolated installer acceptance: 19 ordinary publication
boundaries plus all nine boundaries of the stale-binding transfer, idempotent recovery,
whole-skill concurrent edits, preserving customizations, conflicts/foreign refusal,
legacy migration, saved engine repair and nonstandard CODEX_HOME. The permanent
installed launcher completes a real MCP initialize, tools/list and get_workspace_info.
Discovery exactly matches the current live-enabled/raw-disabled contract (101 tools);
the test compares the complete tools result, not a historical count.

The original `/private/tmp` acceptance profiles exposed the existing external-engine
guard against world-writable ancestors in the custom-engine fixture. The passing profile
is under the task's user-owned visualization directory; no guard or system permission
was weakened. Package doctor uses Inkscape 1.4.3 headlessly and passes outside the sandbox.

The implementation is in [PR #62](https://github.com/P1oN/inkscape-mcp-server/pull/62).
Main requires Quick Rust checks, Native candidate (aarch64-apple-darwin), an up-to-date
base and resolved review threads. The ruleset requires zero approving GitHub reviews;
the separate independent agent review below is still completed as requested.

The independent agent's first pass found four issues: same-tag partial updates, incomplete
review identity, first-install external-change race and incompatible-latest discovery.
The second pass also found channel changes routing the primary action into offline setup.
All five were addressed with dedicated regression coverage. The final independent pass
found no remaining blocking findings and independently passed strict Clang syntax and
diff whitespace checks. It did not perform interactive GUI acceptance.

## Scope and retention

No real user installation, profile, skill, drawing, Manager/Inkscape session, system
permissions, Gatekeeper or quarantine settings are changed. Isolated profiles and owned
fixtures are under `/private/tmp/inkscape-manager-*` and the task visualization directory;
their ownership/recreation policy
is in `/private/tmp/inkscape-manager-task.json`. Cargo caches have a combined 6 GiB budget.
Successful disposable packages/profiles are archived and round-trip verified (every
path/type, bytes, symlink target, owner/group and permissions), then removed after
process/open-file checks. Compact reports, the real MCP trace, inventories/provenance
and complete archives retain the matching executables and dSYM bundles under
`manager61-evidence` in the task visualization directory.

Interactive Manager review/activation, signed-final-byte behavior, Gatekeeper/browser
download and another-Mac acceptance remain unverified for this change. No release is
published. MCP schemas and authoring instructions are unchanged, so documentation-catalog
regeneration and restarting the user's MCP server are unnecessary.
