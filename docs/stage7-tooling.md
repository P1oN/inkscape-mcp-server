# Stage 7: native development and acceptance tooling

The active entry point is `scripts/dev-tools.sh`, backed by the standalone locked
`rust/tooling` crate. Source bootstrap/build, package construction, source export,
discovery/manifests and acceptance no longer provision or execute project Python.
The tool is development-only and is neither packaged nor exposed through MCP.

## Gate inventory

| Former active entry point | Current gate |
| --- | --- |
| Python synthetic engine/gdbus/lock fixtures in Cargo tests | Test-only native `live-process.rs`, compiled with Cargo's Rust dependencies |
| bootstrap_native / bootstrap_acceptance | `bootstrap-native`; archive/input/cleanup Cargo regressions and real six-bottle verification |
| migration_build_posix_package / migration_posix_builder_acceptance | `build-package`; native target, Mach-O resolver, ELF closure/source-preservation and source identity regressions |
| migration_probe / migration_compare | `discovery --matrix`, exact frozen contracts, bounded wire failure traces |
| gen_llms_txt | `manifests`, registry-derived output with preserved formatting |
| migration_package / doctor / notices / launcher acceptance | Same named Rust acceptance commands, plus explicit socket acceptance in CI |
| launcher_sentry_acceptance | `sentry-setup-acceptance`, including real PTY hidden-input checks |
| install_path / client_management / client_package acceptance | `install-acceptance`, `client-acceptance`, native client-manager Cargo tests |
| security / startup / responsiveness / diagnostic / frame / defects / special-file / compare / renderer / engine-route harnesses | Rust commands listed in CONTRIBUTING, with original invariant gates |
| migration_native_gui / rust_inx_native / rust_socket_native acceptance | Explicit `native-gui`, `native-inx`, `native-socket` phases; recorded ownership/context before actions and graceful closure |
| Retired Python consumer unit tests | Historical only; authoritative shared-SVG/INX/socket native consumer regressions remain in `rust/tests` |

The 55 moved Python source/config files retain their exact original bytes in
`scripts/history/python`. Frozen contracts, historical reports and vendor provenance
are preserved. Historical Python wheel/CPython provenance is not consumed by the
current package builder. No Python MCP server or paired parity run was restored.

## Local evidence, 2026-10-05

The working tree started clean on `codex/python-removal-stages-1-3`, HEAD `85c67cd`.
Changes remain uncommitted. Evidence is local Apple Silicon/macOS acceptance; CI YAML
is prepared, not remotely executed evidence.

- Runtime: 288 passed, two existing opt-in tests ignored; fmt and all-target Clippy passed.
- Development tooling: 15 tests passed, fmt and all-target Clippy passed. Safety cases
  include raw tar traversal/size/link/duplicate refusal before publication, notice checksums,
  Mach-O ambiguous paths, ELF missing/colliding dependencies, preserved source bytes,
  native input mapping/escape refusal, revision markers, Git worktrees, nested package
  copies, bootstrap cleanup and completed/pending STDIO traces with bounded cleanup.
- Six locked native bottles downloaded, hash-verified and extracted without Python.
- Git-free unpublished source install: eight checks passed with real isolated Codex,
  skill merge/conflict preservation, real CLI render, uninstall/archive and reinstall.
- Final package: 435 inventory files, 193,143,208 uncompressed inventory bytes. Ready
  archive contains five native binaries, symbols and native assets/dependencies, no
  project Python, wheels or development executable.
- Discovery: all 16 configurations match frozen contracts exactly. Regenerated manifests
  preserve the previous registry content; only their obsolete Python header changes.
- Final archive passes per-call and shell headless/real CLI acceptance and native INX,
  private bus and socket checks. Doctor passes 12 profiles; notices 11 checks/192 crates;
  launcher 11 checks; Sentry/setup 51 checks; isolated client lifecycle passes.
- STDIO suites cover 35 security checks, frame cap, 128 fresh sessions/four workers,
  cancellation/responsiveness in both modes, diagnostics, reference/approval/rollback
  defects, special files, compare publication and engine routes. Renderer passes 22
  linked-resource cases plus export/reopen in both engine modes.

Evidence roots/logs: `migration/results/stage7-delivery-*`,
`stage7-runtime-tests-final.log`, `stage7-runtime-clippy-delivery.log`,
`stage7-tooling-tests-delivery.log`, `stage7-tooling-clippy-delivery.log`, and
`stage7-bootstrap-final/`. Earlier `stage7-*` experiments include failures and superseded
packages and are not delivery evidence. `stage7-verified-notices-retry.log` records
verification after fixing nested-copy parents; a dedicated Cargo regression now covers it.

No new native GUI was launched, and native Undo/Redo acceptance was not rerun or inferred.
The replacement GUI harness has automated XML/ownership refusal checks; its full real
GUI execution remains opt-in for future live changes. Linux, Intel Mac, Windows, remote
CI, truly clean-machine SDK provisioning and real Claude remain unverified. The user's
windows, installed runtime/configuration and published releases were not changed.
