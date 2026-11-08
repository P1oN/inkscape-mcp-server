# Post-switch disk failure and retained-helper acceptance

Local acceptance on 2026-11-08 used macOS 27.0 (26A428) on Apple Silicon and
Inkscape 1.4.3. The two CI packages used helper protocol 5. The candidate was
built from CI merge revision `b584344cffdd61b734d939820103c9528203d0f2`, whose
Git tree matches [PR #58's merge commit](https://github.com/P1oN/inkscape-mcp-server/commit/1b89f5c833a46d55fba714e585b033abbb7d547a).
Both runtime archive checksums matched their CI manifests.

| Runtime | Build | Archive SHA-256 |
| --- | --- | --- |
| [Baseline CI](https://github.com/P1oN/inkscape-mcp-server/actions/runs/42434168722) | `0c95f6ce10208ae5` | `f0489b2faf7a1e46fd00eb674cb52bbec1c3c063179071ecd27fd529f48c5aba` |
| [Candidate CI](https://github.com/P1oN/inkscape-mcp-server/actions/runs/42480917552) | `2c8499d368dba575` | `cb210b97035cc6134afa9975c42a883efdf5efcb436e64bc47427ffd7d7100fe` |

## ENOSPC after switching the selected runtime

A 128 MiB APFS sparse image held only the synthetic installation's history
directory. Filling that volume produced actual `ENOSPC` (`errno 28`). The rest
of the installation had space for staging and recovery. The component update
published the new selector and staged skill, then hit `ENOSPC` while creating
the temporary history file. Both staged selector files had been consumed by
the switch and rollback, and the interrupted skill copy was retained in the
transaction backup.

Recovery restored the old selector and the exact prior skill inventory. The
settings and original SVG matched byte for byte, the journal was cleared, and
another recovery call reported no changes. The permanent launcher completed
the MCP handshake and `get_workspace_info` on the restored runtime while the
history volume was still full. After ejecting that volume, retry selected
`2c8499d368dba575`, retained `0c95f6ce10208ae5` as the previous runtime, and
preserved the local skill customization.

This exercise covers the post-switch history-write failure with recovery space
available on the installation volume. ENOSPC during recovery itself and storage
media failure remain open.

## Native reconnect with the previous helper

A separate synthetic installation launched a private Inkscape window with a
one-rectangle drawing. After a runtime-only update, the permanent launcher
started the new runtime and reconnected over managed D-Bus. The process,
supervisor, window, document and session manifest matched the baseline.
The extension wrapper still pointed to the old `inkscape-mcp-inx` binary,
with SHA-256 `35ca352dce969705e4d79843cfcc86e04389bf1bdd42ba9f7d20aa6a2e3f3515`.

After selecting the drawing, a reviewed package changed the rectangle group's
fill and opacity and recorded one applied operation. The captured SVGs show the
change. A fresh reconnect rejected the repeated package until the drawing was
selected again. After reselection, the package returned `changed: false` with
the same SVG content and operation history. This covers the isolated native
window and a style edit. Signed distribution and real-client acceptance
remain tracked separately in the [backlog](../../RUST_NEXT_PLAN.md).
