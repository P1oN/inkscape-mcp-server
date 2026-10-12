# Manager window refresh — 2026-10-12

The user requested a UI/UX improvement based on the v0.1.7 error-state screenshot.
Implementation is local in `codex/manager-ui-refresh-v2`, based on `3c4ad43` after
the shared Codex profile and DMG presentation changes. It is not a signed release.

## Behavior

- Native AppKit cards group connections, workspace, Inkscape and updates; the
  existing app icon and system fonts/colors work with the user's appearance.
- The resizable 680×740-point window scrolls settings above pinned status/actions,
  with a 620×600-point minimum. Paths truncate in the middle and retain selectable
  text/full-path tooltips. Status is limited to four lines with full-message tooltip.
- Advanced starts collapsed and contains installation selection, detailed
  identities/signature information, component updates and rollback. Multiple
  installation choices expand it automatically.
- The primary action is Review installation / Review changes, then Install now
  for an explicit preparation. An available update uses Update now when setup has
  not been edited. Option changes discard the preparation; channel changes discard
  stale update availability. Fresh installs hide Check updates.
- Busy operations disable conflicting controls; progress appears only while busy.
  Error/recovery state preserves installed-version context and exposes recovery.
  Missing unselected clients do not mask the next step.
- Fixed Rust operations, helper deadlines, install/update journals, approval and
  filesystem protections are unchanged. No arbitrary executable surface was added.

## Automated validation

Strict Objective-C compilation (`-fobjc-arc -Wall -Wextra -Werror`, with unused
delegate parameters exempted) passes. The tooling Cargo graph passes 30 tests,
with the existing native DMG mount/remount test intentionally ignored by its
default annotation. Clippy with warnings denied, formatting and `git diff --check`
pass. No server behavior/schema/instruction changed, so catalog regeneration and
server restart are unnecessary.

The new hidden AppKit fixture runs with a temporary HOME and no helper/client
launches. It checks unavailable-action gating, retained metadata on failure,
busy-state gating, exact prepared-activation routing, preparation invalidation,
update routing, channel invalidation and recovery. In light and dark appearances,
the primary action/status remain inside the minimum window and a long workspace
path stays within its width.

Local AppKit-rendered content previews (illustrative paths, not release screenshots)
are retained under `.inkscape-mcp-local/manager-ui-refresh-2026-10-12/` in the worktree.

## Native acceptance scope

The authorized isolated profile was
`~/.inkscape-mcp-native-acceptance-manager-ui-20261012`. Its ad-hoc development app
used the redesigned UI with a retained compatible development helper/package.
CUA observed the real window, default Codex (app + CLI) selection, grouped sections,
collapsed Advanced, missing-workspace guidance and disabled Review installation.
The folder picker opened and navigated to the profile's owned drawings folder.
CUA then repeatedly timed out; a native process sample showed the application
idle in its event loop. This does not establish the cause of the automation failure.
The owned test process was stopped afterward.

Real-window review/cancel/activation, user-driven resizing and live appearance
switching remain unverified for this UI. Automated state/layout checks are not
substitutes for that acceptance. No runtime was activated in the test profile;
no installed user runtime, client configuration or Inkscape GUI was changed.
The original synthetic SVG remains unchanged. Another-Mac and signed final-byte
qualification retain their existing separate gates.
