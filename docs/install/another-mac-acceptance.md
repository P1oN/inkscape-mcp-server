# Another-Mac signed delivery acceptance

Use this checklist on the separate Apple Silicon Mac with macOS 15+ and Inkscape
1.4+. Use the published Manager DMG linked from [README](../../README.md), downloaded
by clicking the release link in a browser. Record the actual release tested; a newer
release needs its own artifact-bound evidence. This checklist describes required
observations, not completed acceptance.

Use an owned test account/profile, synthetic SVG workspace and test client settings.
Keep original drawings, unrelated client settings and existing Inkscape windows intact.
For upgrade cases, retain the original installation and configuration backups and
record the baseline's actual source/build identity. A local synthetic baseline must
be labelled as such rather than described as an older published release.

## Ordinary installation

1. Open the browser-downloaded DMG in Finder and record every macOS prompt verbatim.
   The mounted disk should offer one Manager application. Keep quarantine enabled.
2. Open Manager, accept its per-user relocation when offered, choose the test clients
   and synthetic workspace, and review the requested changes. Ordinary installation
   uses the native wizard without Terminal commands or a compiler.
3. Complete installation and record the displayed distribution, runtime, instructions,
   Manager and launcher identities. Close and reopen Manager and compare its status.
4. Restart/reconnect the selected real clients. Record each actual client version,
   registration scope and a workspace request. Use the owned SVG for any authorized
   render/edit test; MCP startup and reconnect must not launch Inkscape.
5. Disconnect the owned client binding and confirm unrelated registrations/settings
   remain. Record the outcome separately for Codex and Claude, including unavailable
   clients and authentication constraints.

## Required matrix

Record pass, failure or pending for every row, with baseline/final identities and the
observed result. Separate accounts or retained test fixtures may be needed to keep
fresh-install, upgrade and offline-first-launch cases independent.

| Case | Observation to retain |
| --- | --- |
| Fresh install | Initial not-installed state, client/workspace review, completion and reopened installed status. |
| Legacy upgrade | Original legacy source and binding retained, new permanent registration, unrelated settings and drawings preserved. |
| Permanent-install upgrade | Baseline and final runtime/bootstrap paths and identities, preserved workspace/engine/channel/private settings. |
| First online launch | All actual macOS prompts for the quarantined DMG, Manager and extracted installed runtime/helper. |
| First offline launch | Disconnect network before the first launch of the case's downloaded artifact; offline bundled installation and runtime/helper execution, with exact prompts. A prior online launch does not qualify this case. |
| Real clients | Actual Codex/Claude registration, handshake/discovery, first workspace request, reconnect and disconnect. Record authenticated interaction separately from CLI diagnostics. |
| Instructions-only update | Distinct instruction identities, unchanged runtime, managed skill edits/conflicts preserved. |
| Runtime-only update | Distinct runtime identities, retained instructions, older running session/helper path preserved and a new client using the selected runtime. |
| Combined update | Both components change, final identities match the release, drawings and unrelated client state are unchanged. |
| Bootstrap upgrade | Manager/launcher replacement is reviewed, deferred while the Manager is in use, and reflected after reopening. |
| Rollback/no update | Previous pair restored with preserved skill edits; a subsequent check reports the actual eligible state/channel. |
| Network failure/retry | Failed download ends in a recoverable UI, the installed pair stays launchable, retry succeeds. |
| Interrupted installation/recovery | Record the actual interruption boundary and recovery result in an owned fixture. Use offered cancellation/recovery actions; forced process/storage failure belongs in a separate controlled technical test. A cancelled review alone is not activation-interruption evidence. |
| Customized/conflicting/removed skills | Custom edits survive; conflicts are reported before activation; removed, unmanaged and foreign-owned destinations are not silently adopted. |
| Source bootstrap on a clean host | Apple SDK/pinned dependency preparation, actual prompts, setup, registration, render and reinstall. An already configured host or ready-DMG install does not prove this separate advanced route. |

Live drawing/reconnect checks require separately authorized owned synthetic Inkscape
documents. Keep their results separate from headless tests and preserve user windows.
Unsupported or unavailable cases remain pending, with the exact reason recorded.

## Technical evidence and report

Retain the release's `distribution-evidence.json` and `FINAL-ACCEPTANCE.json`, browser
download path and SHA-256, quarantine attribute, OS version/build and architecture,
Inkscape/client versions, signing Team, notarization submission IDs, signature/ticket
assessment results and observed system prompts. Compare the downloaded checksum with
the published checksum asset; never clear quarantine or disable Gatekeeper to pass.
Existing-host approval caches must be disclosed when describing first-open results.

For each case, include baseline/final component identities, exact operations, expected
and observed behavior, preserved-state checks and retained diagnostic locations. Keep
credentials, saved monitoring DSNs and unrelated private configuration out of reports.
Link the report from the [history index](../history/README.md) and update the
[active backlog](../RUST_NEXT_PLAN.md) only for the cases actually proved. Automated
CI receipts and local configured-host checks keep their original scope.

See [distribution/recovery operations](macos-manager-distribution.md),
[client management](client-management.md), [independent updates](independent-updates.md)
and [source prerequisites](local-bootstrap.md) for the fixed supported operations.
