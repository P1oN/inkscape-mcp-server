# Actual local reinstall and system Applications repair — 2026-10-13

Installation failed on macOS after Manager v0.1.8 was copied to `/Applications`.
The failure was reproduced with the published app on the development machine.

## Published-byte reproduction and local reinstall

Release v0.1.8 is published from `93e5080cf06d00434f6597e1e303c6ba3da9e65d`.
Protected run `38275933162` completed validation, preparation, verification and
publication successfully. The downloaded Manager DMG passes its checksum; the
extracted app passes deep/strict codesign verification and stapled-ticket validation.
This was a CLI download, not a browser-quarantine acceptance claim.

The actual local Codex binding used the source checkout's `run-mcp.sh`; its saved
package no longer existed. The native client manager disconnected that binding and
archived the old runtime and its diagnostics before the clean installation.

CUA launched the published Manager from the mounted DMG, approved its explicit
per-user relocation, observed it reopen in `~/Applications`, selected the existing
`~/Documents/Drawings` workspace, reviewed, cancelled, reviewed again and activated.
The app reported All set, then reopened with Installed / Ready. Inkscape was discovered
at `/Applications/Inkscape.app/Contents/MacOS/inkscape` without manual selection.
The actual permanent launcher reports build `0a01a8b8f950cdd6`, source `93e5080`,
Developer ID Team `DN263AX69U`; its fresh STDIO discovery matches the exact contract.
Authenticated real-client interaction is not claimed; Codex must reconnect to load
the newly registered permanent launcher.

Checks compare all 249 existing drawing hashes before/after activation and 101
unrelated Codex configuration sections against the pre-uninstall private backup:
none changed. No Inkscape GUI was launched/closed and no user processes were killed.

Copying that exact signed Manager to the previously absent system application path
reproduced `{"ok":false,"error":"managed path is writable by other users"}` from
`manager-prepare`. The failure precedes inspection, explaining why even standard
Inkscape discovery never populates the window. `/Applications` has the normal
root:admin group-writable permissions; changing those permissions is not a repair.

## Local source repair

The repair keeps `guarded()` strict for all
managed write destinations. Read-only admission is confined to the exact running
`/Applications/Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher`,
its Info.plist and Resources tree. Only the root-owned, admin-group-owned,
non-world-writable `/Applications` ancestor receives the group-write exception.
Symlinks, foreign owners and writable descendants remain refused. Root-owned
immutable app inputs are accepted for administrator-installed applications.

Manager startup still verifies the app signature. Installation preparation explicitly
verifies the complete current app before accepting its bundled inputs. Instruction
loading, bounded reads, source inventories, helper identity hashing and quarantine
source reads use the narrow read policy. Quarantine destinations retain strict
write guards; signature, manifest, inventory and receipt checks remain enforced.
Writable-path diagnostics now include the actual rejected ancestor.

The first development iteration passed startup/discovery but exposed further strict
reads during preparation. The helper identity and quarantine-copy source paths were
then covered; intermediate failures are retained rather than counted as acceptance.
Final matching ad-hoc build `d25bc0b2c3422350` runs from the actual system app path
with an isolated HOME: manager preparation, automatic Inkscape discovery, offline
runtime preparation, activation and installed inspection all pass, preserving the
synthetic SVG. No Codex CLI exists on that profile's PATH. The local verification retained the operation results and stderr.

The development app also opens natively from `/Applications`. Its ad-hoc actor has
no receipt for the user's separately signed runtime and correctly cannot verify
that installation; reopening the published per-user Manager reports Ready again.
This mixed-actor observation is not counted as native patched-install acceptance.
Development system-app fixtures were moved back into the private evidence directory,
the DMG was detached, and the user retains the published per-user Manager/runtime.

## Validation scope

Sixteen update-module tests (including new source-scope, writable-descendant and
symlink refusals), eleven serial update-manager integration tests, thirty tooling
tests, both all-target Clippy checks, formatting and diff checks pass. The existing
opt-in DMG mount/remount test is not part of the default tooling test result.
The final development Runtime also passes the full installer matrix: nineteen
publication/recovery boundaries, custom-engine repair, foreign-binding and unsafe
workspace refusals, skill preservation, legacy migration and no-op rollback retention.
The isolated system-path installer operations above use real native doctor/STDIO
probes and a synthetic client profile, not an authenticated client/model session.

The development build is unpublished. A future release still needs its exact
committed CI inputs, Developer ID/notarization and final signed-byte qualification
including installation from `/Applications`. Another-Mac confirmation remains a
separate gate. MCP schemas/instructions are unchanged; no catalog regeneration is needed.
