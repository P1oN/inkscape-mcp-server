# Signed runtime release follow-up — 2026-10-09

PR #30 merged as `6499d29ea3a067963efa3ecb07134c2fb7b668d1` after its checks passed.
Main native CI [37851864135](https://github.com/P1oN/inkscape-mcp-server/actions/runs/37851864135)
passes; its verified Runtime build is `f80f0fd90eef4ad9`. Validation-only release
preparation retained exact artifact/checksum provenance without creating a tag/release.
The intended next prerelease is v0.1.6; no tag or release was created.

## Exact CI inputs, local signing

`.inkscape-mcp-local/main-signed-v0.1.6-37851864135` contains the locally signed
transformation of those CI inputs. Team `DN263AX69U` and the local notarization
profile authenticate. Runtime, Manager and DMG are Accepted and stapled:

- Runtime submission: `a4d9a4c6-df3f-4192-9ea2-5ae81176e76a`.
- Manager submission: `6032a85e-2e2d-405e-b34f-d3bf9b62325b`.
- DMG submission: `dd3ff709-31b4-49e9-93ff-66c318082c7c`.
- DMG SHA-256: `badf069776e158dbb1865394f0eae45cbef9076f7c7386905308ec4737ab33cc`.

This is local preparation, not protected signing-job evidence. Its local receipt
records publication_eligible=false and final_qualification_passed=false.
Package acceptance passes 449 files (`migration/results/main-signed-v0.1.6-package`).
Installer acceptance fails at the independent tooling actor's final probe because it
has no receipt; the old tool wrote its success report prematurely. Update acceptance
fails real STDIO after activation because the permanent launcher has no receipt for
a runtime staged in-process by a different management helper. Those reports do not
qualify the candidate, despite successful notarization.

## Repairs and scoped verification

Each activation probe independently verifies its signed Runtime before recording its
own actor-bound receipt. Staging additionally grants a receipt to an installed
bootstrap only after guarded identity checks of both command files, signature checks
against the management actor, and a second byte-identity check. Changed, linked or
foreign-signed bootstrap files are never adopted. Launch still validates the full
inventory and exact actor receipt without running codesign.

Modified signed tooling SHA-256
`180a8491b230dd4408de0625eeefb6ff0227c52031994f25776037b98f8b7226`
passes the 19-boundary installer matrix and distinct-runtime update/rollback matrix
against the unchanged signed CI Runtime/Manager. Reports are
`migration/results/receipt-fix-installer` and `receipt-fix-updates`.
These are modified-tool checks, not qualification of a fresh shipping distribution.

Startup against the original final installed launcher fails after seven sessions:
its two-second recovery lock starves another concurrent launch while it hashes the
inventory (`receipt-fix-startup`). The fix releases the lock after recovery and reading
the selector/settings snapshot; selected Runtime/text trees are immutable.
A fresh release helper from the modified source was Developer ID signed and installed
through prepare/activate in `.inkscape-mcp-local/receipt-fix-startup-profile2`.
It passes 128 sessions with four workers in 16.65 seconds (`receipt-fix-startup2`).
Helper SHA-256 is `b7e5b9332183bc538f49bd26f41d98d6529b827c2c60f6e55e4e59090417e3c6`;
its build is `ff32a9f5acbeab4a`, dirty source revision `6499d29`.
This helper is not a new notarized Manager/DMG or a committed CI artifact.

The protected signing acceptance script now runs the same startup check against its
installed final helper. The recorder checks exact helper hash, 128 completions and
four workers before writing acceptance; the publication verifier requires that gate.
Refusal fixtures cover failed/incomplete/serial/wrong-helper reports without replacing
an existing receipt. Installer success reports are written after the final probe.

Automated validation: 37 library tests, nine update-manager tests serially, 20 tooling
tests, both Clippy/format checks, release publication and signed-distribution fixtures,
Bash syntax and actionlint pass. Release helper compilation and native isolated
prepare/activate/startup pass. Native GUI acceptance was not repeated. No user runtime,
client configuration, artwork or Inkscape windows were changed.

## Remaining delivery gates

Read-only GitHub inspection finds zero environments, secrets and repository variables.
Protected signing/publication setup and execution, a fresh committed main-CI candidate,
new whole-distribution qualification and the other-Mac browser acceptance remain open.
Real Claude Code is unavailable locally. No public release or unsigned fallback was
published; prior signed candidates retain their own historical scope.

## Later delivery state — 2026-10-09

PR #31 quick/native CI `37903764794` and CodeRabbit status passed, with no diff
comments. It merged as `97707eac7f60723affd083dd72f5c407cd653a99`. Fresh main CI
`37970554352` is running; only its successful main-push artifacts may feed signed
preparation. The earlier observations above retain their original checkpoint scope.

GitHub now contains `release-signing` and `release-publication`, each restricted to
the `main` branch with P1oN as required reviewer. Repository `SIGNING_TEAM_ID` is
`DN263AX69U`; the signing environment has `SIGNING_IDENTITY`, `NOTARY_KEY_ID` and
`NOTARY_ISSUER_ID`. Its three required secret names are present. Secret contents
were not read or exported. Credential validity still requires actual protected CI.
No release was published at this checkpoint.
