# Manager first-install and shared Codex registration — 2026-10-12

The other-Mac user reports successful v0.1.7 Manager application installation,
followed by a missing `~/.codex/settings.json` error on Check updates and a
`codex client is missing` error on Install. They report the Codex desktop app in
`/Applications`, no CLI, and no manual selection of `~/.codex` as the installation
folder. The origin of that installation path and the failed app discovery have
not been reproduced on the other Mac. Runtime installation/reconnect acceptance
on that host remains open.

The follow-up keeps Codex selected by default, labels it **Codex (app + CLI)**,
and writes the preserving shared profile registration without requiring client
executable discovery. It does not install a Codex client. App and CLI using
different `CODEX_HOME` profiles still require separate registration. Existing
binding ownership and configuration safety checks remain enforced. Check/update
before installation now explains the next action before creating files or
requesting release data; existing recovery journals retain their recovery flow.
The window displays the selected MCP installation folder.

## Development validation

The local release-profile package has compiled build `440e8d81d292548d`, based on
parent `1806dac7bce8320e6e1a69e8a7520bd49e91e887` plus the uncommitted fix.
This ad-hoc development package is not a signed release candidate.

- The retained older ad-hoc Manager helper reproduces the missing-Codex error
  against its matching runtime with an isolated profile and `/usr/bin:/bin` PATH.
  The new matching package prepares, activates and inspects successfully using
  the same profile, preserving preferences, another MCP entry and the SVG.
- The complete installer acceptance passes without synthetic Codex executables:
  19 publication boundaries, repeat recovery, legacy migration, custom engine
  repair, foreign-binding/ownership/failed-probe refusal, skill conflicts,
  instruction relabel, no-op and artwork preservation. Claude's missing-client
  refusal remains exercised. These are synthetic client profiles with actual
  native MCP/doctor probes, not real app/CLI reconnect observations.
- An authorized isolated native Manager window shows the checked app + CLI
  option and its owned installation path. Clicking Check updates before install
  displays the new guidance with the controls and message visible. The window
  was closed afterward; no real installation/client settings or Inkscape window
  was changed.
- All 11 updater integration tests, 13 update-module unit tests, Manager task
  deadline fixture, both Cargo graph all-target Clippy checks, formatting,
  native Manager compilation and diff whitespace checks pass.

An initial mixed-helper/runtime experiment correctly refused the differing
installer identity. A debug package build also stopped because release dSYM
inputs were absent. The successful run uses a freshly built matching release
package; neither refusal was bypassed.

Local evidence is retained under
`.inkscape-mcp-local/manager-first-install-2026-10-12/`, including `check.cjs`,
`profile-p5UVaU/report.json`, per-operation results and
`installer-acceptance/report.json`. The native window used the separate owned
`~/.inkscape-mcp-native-acceptance-manager-first-install-20261012` profile.
No MCP schema or server instruction changed, so manifests were not regenerated.
Signed publication and another-Mac acceptance require their own subsequent evidence.
