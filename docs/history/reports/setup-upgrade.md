# Setup replacement policy — 2026-10-06

The user reported that a new Apple Silicon package refused an old Codex `inkscape`
registration and requested replacement of installed MCP/skill contents while retaining
configuration and supplementing missing parameters. This is a local unpublished follow-up;
v0.1.2 release archives retain their original preserve/refuse behavior.

Current setup selects the new ready runtime (or rebuilds changed/unidentified source),
refreshes existing Codex/Claude user registrations and installed skills even without flags.
Explicit client/skill flags select new installations. `upgrade` checks the new MCP's bounded
initialization/discovery/workspace handshake before atomically updating the binding.
Command/args/transport and obsolete HTTP fields change; other preferences, servers, env,
timeouts and Codex TOML comments survive. Missing args/env receive defaults. Config input
is capped at 4 MiB; malformed/symlinked config and concurrent observed changes are refused.
Previous config bytes are retained privately beside the profile config. Standalone `connect`
continues to refuse differing bindings; disconnect/uninstall cannot remove another launcher.

Setup replaces the complete skill tree, removes stale active files and resets its upstream
baseline/owner. Previous trees are archived under the profile's `inkscape-mcp-backups`,
outside skill discovery; failed publication restores the old tree. Symlink protections
remain. Standalone `install-skill.sh --update` retains explicit three-way merge semantics.
Saved workspace/Inkscape/live/engine and telemetry choices survive ordinary reruns;
missing optional positional fields are supplemented with current defaults. Inkscape GUI
and existing user client processes are never launched/closed by these checks.

Validation on the development host: native client unit graph 14 passed, tooling graph 17
passed, both fmt/Clippy graphs and Bash syntax passed. Real Codex plus a synthetic Claude
profile passed the package client lifecycle, including setup replacing an old launcher
without losing unrelated config, strict standalone refusal and failed-handshake preservation.
Empty-PATH launcher passed 11 checks, including supplementation of missing live/engine.
Synthetic Sentry/setup passed 53 checks, including automatic skill replacement, stale-file
removal, backups, settings preservation and automatic unidentified-source rebuilding.
The acceptance launcher uses isolated HOME/CODEX_HOME to prevent default-upgrade probes
from touching the development user's registrations. No GUI or clean-machine/real-Claude
acceptance is implied. Raw local results are in ignored `migration/results/setup-upgrade/`.

Git-free installation initially failed its identity assertion: a cached build-script binary
had captured its compilation checkout with `env!(CARGO_MANIFEST_DIR)`. Shared target-directory
reuse could report that original checkout's SHA/fingerprint for another source tree. The
build script now reads CARGO_MANIFEST_DIR at execution time. A regression compiles once in
Git checkout A and executes against Git-free source B, first unknown and then with a different
committed marker. It failed before repair and passed afterward. Repaired working-tree source
archive installation passed all eight isolated real-Codex/CLI/skill/uninstall/reinstall checks.
The original failed archive/results remain retained; no failure is hidden by a retry.

Final rebuilt ready archive passed headless relocation/FILES verification (443 files),
empty-PATH real CLI pixels, approval/rollback/no-op checks and the real-Codex/synthetic-Claude
client lifecycle. Documentation link checks covered 98 relative targets with no missing files.
The public v0.1.2 assets and the development user's installed runtime were not replaced.

PR #12 review follow-up: automatic skill refresh failures now include the client and a
quoted standalone retry command after the installer's diagnostic. Isolated Codex/Claude
symlink-refusal checks confirmed nonzero exit, saved setup settings and an unchanged
symlink target. The 53 setup/Sentry checks, Bash syntax and diff checks passed again.
