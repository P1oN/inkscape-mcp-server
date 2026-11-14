# Manager update screens — Penpot follow-up, 2026-11-14

This follows Issue [#63](https://github.com/P1oN/inkscape-mcp-server/issues/63) and the
interactive audit of PR #64. Implementation starts from `origin/main` `a32dacb` in a
separate `codex/manager-penpot-update-screens` worktree; the original dirty checkout
and other tasks' worktrees remain preserved. No release/tag is authorized here.

## Design and behavior

The read-only Penpot reference is file `38954aef-1e2c-81ac-8008-f0de1d0c768f`, page
`65ac301c-d568-8021-8008-f1aac2ff0572`, boards 33–40 and the exact reviewed changes
overlay. The named headless route checks both file and page IDs before reading.
Other proposals, the browser's focused document and Local Budget are not references.

| Reference | Implemented behavior |
| --- | --- |
| 33 Settings & versions | Separate component identities, Manager offer/check, technical details and component rollback |
| 34 Preparation | Separate four-stage explanation, bounded download progress and protected busy actions |
| 35 Review | Compact Manager version/build, launcher scope, destination, publisher and preservation summary |
| Exact changes overlay | Separate selectable, scrollable full paths, revisions, hashes and compatibility values |
| 36 Handoff | Restart explanation, preservation list, second-copy warning and disabled action |
| 37 Success | Startup acknowledgment produces the Manager completion state with current bundle identity |
| 38 Failure/permissions | Full readable reason, preserved-state guidance and Back to settings |
| 39 Recovery | Separate app/launcher recovery explanation and Recover Manager action |
| 40 Restored | Confirmed restoration banner, retained component identities and explicit completion message |

Cancel returns to Settings with an immediately usable offer. It never retains activation
consent: Review again runs verified preparation again. Stale background replies cannot
replace an open Review. An accepted asynchronous Manager response updates both the
visible banner and button action, including offline/no-update states. Component rollback
returns to the existing connection view before running so its error/reconnect outcome is
visible. Complete component identities remain available under Technical details.

Recovery from another distribution must not claim that its running bundle is the restored
app. The table explicitly identifies **Running Manager**, and restoration copy states that
the previous installed Manager/launcher were restored without inventing their version.
Runtime origin uses its recorded distribution identity rather than the combined pair tag.

The native controls follow the system's current appearance. The interactive host was dark;
the Penpot boards are light. The acceptance compares states, hierarchy, text, controls and
readability; it does not claim identical platform pixels or final release qualification.

## Recovery evidence and safeguards

Rust remains responsible for all verification, locking, mutation, backups and restart.
The GUI adds no download, replacement or trust bypass. Startup verification failures lead
to bounded read-only inspection, preserving the real error and exposing Manager recovery
when its journal exists. A missing/damaged helper cannot become a successful update.
Malformed JSON roots normalize to no response before dictionary access.

After actual Manager recovery, `manager-recovery.json` is durably written before retiring
the journal. Inspection recognizes it only for the matching current prepared plan and
with no remaining Manager journal. The coordinator can still report the original failure
in `manager-outcome.json` without overwriting this independent restoration fact. A failed
receipt write retains the journal; retry is idempotent. Completion/warnings are scoped to
the selected installation, so selecting another root cannot inherit an old success banner.
No legacy release manifest or MCP tool/schema/instruction changes are introduced.

## Interactive acceptance

The production AppKit frontend was compiled with strict warnings and opened through an
owned wrapper with a separate HOME/CFFIXED_USER_HOME and nonstandard CODEX_HOME. The app
uses a unique synthetic bundle identity. All update responses in this interactive run are
controlled typed fixtures; they cannot replace real applications/bootstrap or register clients.

The run exercised Settings → preparation → Review → exact changes → Cancel → immediate
Review retry; simulated permission failure and Back to settings; actual frontend startup
error → inspection → recovery → restored; explicit consent → readiness → exit → automatic
helper-driven reopen. The final handoff log records parent PID 51907 exiting before new
owned PID 51969 appears, before CUA binds the reopened process. Profile, installation and
stable channel match. The fixture app is deliberately not replaced, so its completion
banner correctly uses its actual v0.1.8 bundle identity instead of the sample target v0.1.9.

Screenshots, accessibility snapshots, source/binary hashes, controlled backend, guarded
Penpot read and request/handoff logs are retained locally under the task-owned ignored
`.inkscape-mcp-local/manager-penpot/`. Synthetic config/auth/custom skill/SVG hashes are
checked separately. The owned window is closed after acceptance; no real installed app,
client profile, drawing, Inkscape window or MCP session is used or changed. Gatekeeper,
quarantine and system permissions are unchanged.

## Automated validation and review

The hidden AppKit regression covers Cancel/retry without activation, current-plan consent,
late responses, accepted/no-update Settings refresh, root changes, startup recovery routing,
malformed helper roots, readable errors, rollback routing and minimum-window action bounds.
Rust recovery regression covers failed receipt write, retry, idempotence, independent
coordinator error, newer preparation and an outstanding journal.

Both locked Cargo test graphs passed, as did both formatting/Clippy graphs and strict
native compilation. The final hidden AppKit fixture passes after the review fixes.
The opt-in Developer ID synthetic replacement/recovery test passed in 112.11 seconds:
verified staging, five publication/recovery boundaries, idempotence, Manager-only and
coordinated replacement, fresh/prepared/damaged roots, failed relaunch rollback/retry,
actual Launch Services restart/acknowledgment, custom profile and retained MCP session.
It performed real initialize → tools/list against the current contract →
get_workspace_info after update/recovery. This uses actual current release-build
binaries and task-owned synthetic signed apps, not a production release.

An independent agent reviewed the complete final code diff without authoring changes.
Its findings were fixed and reviewed again, including accepted asynchronous replies,
startup failure/inspection timeout, root-scoped outcomes, durable recovery receipt,
restored identity wording, malformed JSON, visible rollback results and refreshed
component snapshots. Final review has no remaining actionable findings; strict native
syntax, Rust formatting and diff checks also passed independently.

The complete interactive run above passed before the final defensive review fixes.
A final smoke check of those exact source bytes was interrupted by the host screen lock;
manual unlock was requested. No automatic unlock/protection change is used. This last
interactive step is pending separately from the passing automated checks. Automated AppKit/Launch Services and signed synthetic backend evidence
remain separate from the interactive frontend fixture described above.

## Remaining qualification

Final Developer ID/notarized distribution bytes, browser quarantine/Gatekeeper behavior,
real `/Applications` permission scenarios and another Mac remain release acceptance gates.
A synthetic signed backend test is not final-package qualification. No release is published.
