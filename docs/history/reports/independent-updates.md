# Independent instructions/runtime updates — implementation and acceptance

Recorded **2026-10-07**, uncommitted source changes requested by the user. The user's
pre-existing backlog design is preserved in [the historical plan](../plans/independent-updates.md).
Actual commands and limitations are in [the installation guide](../../install/independent-updates.md).
At the time of this acceptance, no installed runtime/client changes, commits, PRs or release
publication had been performed. The user subsequently authorized documentation updates and
a PR into `main`; the local acceptance remains tied to the identities below.

## Implemented source

- Shared typed format-1 release/runtime/instruction identities, deterministic content hashes,
  macOS/aarch64/minimum-OS checks, launcher minimum and native compatibility epoch 5.
- A bounded text-only instruction loader, embedded fallback and explicit external-bundle
  refusal. A process loads one bundle; initialization and compose share one authoring text.
  Compiled prompt names/arguments/message types, schemas and executable gates remain fixed.
- Permanent native launcher/CLI, one-time source/ready configuration migration, fixed paths,
  offline STDIO, private settings/backups and multiple recorded Codex/Claude bindings.
  Disconnect/uninstall recovery does not require the replaceable helpers.
- Fixed-repository GitHub discovery, explicit stable/prerelease/tag selection, bounded HTTPS
  redirects/downloads, length/hash/target checks and shared archive guards. Old distributions
  without manifests are skipped; downloaded install scripts are never executed.
- Complete runtime/text staging, candidate identity/doctor/STDIO/workspace checks, baseline
  three-way skill merges with snapshot inputs, durable multi-location journal, crash recovery,
  no-ops and rollback that refuses later skill edits. Retained versions protect older helpers;
  there is no automatic garbage collection, client restart or launcher self-replacement.
- Instruction/launcher/update assets from native CI, publication contract verification and
  immutable compatible runtime references. Publication builds nothing or executes archive
  contents. The native management app reuses fixed CLI/JSON commands and is ad-hoc signed.

## Evidence

Local raw evidence is Git-ignored under `migration/results/independent-updates/`.
It is tied to each recorded build, not automatically transferred to later binaries.

| Check | Result/scope |
| --- | --- |
| Both locked Cargo graphs | Final runtime sequential suite: 321 passed, two opt-in ignored; tooling: 17 passed. All 11 update tests passed, including the additional write-failure case. Final fmt/Clippy checks passed for both graphs; logs retained. |
| Default-parallel runtime run | An initial full run passed. A later run failed the pre-existing `live_launch::tests::private_session_and_locks_refuse_link_escape_and_parallel_launch` fixture (234 main-binary tests passed, one failed, one ignored). Sequential run passed. Failure retained in `runtime-tests.log`; no claim that the underlying concurrency issue is fixed. |
| Native fixture dependency repair | Earlier cached-artifact selection mixed serde versions between Cargo graphs. The test fixture now uses the runtime library's exact serde/libc reexports rather than selecting unrelated newest artifacts. Initial errors remain in the tool transcript. |
| Text/contracts | `update_instructions` covers deterministic IDs, shared guidance, changed text, corrupt/unknown/oversized bundles, paths/ownership/modes, interfaces/placeholders and incompatible releases. Explicit external startup is checked through real candidate STDIO. |
| Transactions | `update_manager` covers real text activation/custom skill merge, conflicts/no-op, later-edit rollback refusal, lock exclusion, crash recovery at each move boundary and post-selector publication-write failure with restoration and a launchable old pair. |
| Download/package migration | `cli-final/`: actual native package migration in an isolated profile with Codex TOML and synthetic Claude JSON; offline startup after moving the old source, public CLI from an unrelated directory, preserved settings/drawings/skill customizations and unchanged registrations across component updates/rollback. |
| Component downloads | Deterministic fixed-repository API fixtures transfer real instruction/runtime archives through the shared backend. Instructions-only, runtime-only and combined updates succeeded and rolled back. Initial runtime `ee6511dd91f4db53`; candidate `5221526e23f16ff5`; revision `e639ed78684b35b418dee2728c8a681ac3addc1b`. Individual JSON results retain content/version/build identities. |
| Discovery/default catalogs | All 16 profiles passed. `manifests` regenerated `llms.txt`/`llms-full.txt`; default text/catalog bytes remain unchanged. Surface: 112 tools, seven prompts, 18 resources. |
| Real STDIO invariants | Security, frame, startup, responsiveness, defects, diagnostic, compare, special-file, renderer and engine-routes suites passed for `final/inkscape-mcp-macos-arm64/bin/inkscape-mcp`. |
| Relocated package | Headless per-call package acceptance passed, 453 files. Complete native package includes six Rust executables and the manager app. Cold-package record distinguishes headless CLI pixels from native GUI acceptance. |
| Asset production | Working-tree source archive includes the new launcher, update engine, management app and historical plan. Instruction bundle/archive and typed update manifest generation passed. A macOS temporary-path symlink refusal found during asset verification was repaired by canonicalizing the owned temporary extraction root. |
| Publication | Bash syntax and actionlint v1.7.7 passed. Mock release fixtures passed refusal checks, staged ten-asset publication, immutable runtime reuse without execution/republication, and incompatible referenced-runtime refusal. No live release was published. |

## Explicitly authorized native management-window acceptance

The user explicitly authorized isolated management GUI acceptance in this chat. The owned
profile is `gui-profile-final`; native AX observations/screenshots are in the conversation
and the observation ledger is `native-gui.txt`.

Observed readable offline installed versions and enabled controls; successful rollback from
`ee6511dd91f4db53`/`migration` to `5221526e23f16ff5`/`v9.0.0`, reconnect/skill-reload instructions,
and rollback back to the original pair. Old-release discovery correctly reported no compatible
update assets. A scoped failing proxy exercised check and all three update buttons without
activation. The revised connection/proxy recovery message was visually verified using the
current-source native debug launcher in that owned profile. A deliberately added test skill
file caused rollback refusal with the file preserved; the file was removed after the check.
The explicit prerelease UI choice survived an offline check without changing the saved channel.
The owned manager app was closed and app inventory confirmed it was no longer running.

No Inkscape GUI was launched; no user window was closed, client restarted or user installation
changed. Component download success is backend/CLI evidence; native successful GUI downloads
against published assets, clean-machine installation, actual Claude client behavior, live
Inkscape reconnect/older-helper interactions and full media/ENOSPC failure qualification remain
separate tasks in the active backlog. Ad-hoc signing is not Developer ID/notarization.

## PR preparation after main integration

On **2026-10-07**, branch `codex/independent-updates` incorporated `origin/main` at
`26f21fc`, including its coordinated dependency updates. The two documentation conflicts
were resolved by retaining both entries. Both locked Cargo graphs passed again: 321 runtime
tests with two opt-in ignored (sequential), and 17 tooling tests. Both fmt/Clippy graphs,
publication fixtures, Bash syntax and manifest regeneration passed; catalogs remained unchanged.
The release build passed with the documented macOS `LIBXML2` SDK setting. An initial build
without that setting failed to locate libxml; both logs remain in local evidence.

These automated results are in `pr-*.log` under the existing raw-evidence directory.
Earlier native package/management-window acceptance remains bound to its original builds;
it is not transferred to this dependency-integrated source. Documentation indexes, changelog,
usage and authorization notes were reconciled for the requested PR into `main`.
