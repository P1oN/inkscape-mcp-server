# Compact Manager identities — 2026-11-15

The user approved the compact Settings & versions mockups and requested their
implementation. Source changes live in `codex/manager-compact-identities`, based
on `1a2f286` in a separate worktree. The original dirty checkout is preserved.

## Design and behavior

The Penpot reference is file `38954aef-1e2c-81ac-8008-f0de1d0c768f`, page
`65ac301c-d568-8021-8008-f1aac2ff0572`: updated boards 33/37/40, compact dark/light
boards 41/42, the full-value sheets and behavior board 43. They were read through
the explicitly bound MCP session before editing UI code.

Settings now shows a version/source/status above an eight-character build or
content preview. Component descriptions and update scopes remain separate.
The running Manager keeps its explicit label so recovery from another bundle
cannot misidentify the restored installed application. Runtime origin comes from
its own recorded distribution; an unknown origin does not inherit the combined
runtime/instructions pair tag. Missing components retain Not installed / Not verified.

The app path in the overview preserves the app name and opens full values through
**Build IDs & paths**. The full path is also available to accessibility and in the
tooltip. The monitoring copy retains **New monitoring off**: this is the default
for new configuration, not a claim that an existing user's monitoring was disabled.

The separate resizable sheet contains selectable complete builds, revisions,
instructions content/version, paths and actual bundled-helper signing status.
It does not invent an Inkscape version or claim verification of the running app
from the helper's signing result. Fields wrap without inserting presentation
characters into the data. Individual Copy and Copy all details use the original
full values, show Copied for two seconds, and announce the result to accessibility.
Copy buttons share a trailing edge and reserve space for feedback. Done / Escape
returns to the same Settings page; the scrollable body leaves footer actions visible.

Native colors follow the system's light/dark appearance. Existing update review,
activation, rollback and recovery actions keep their backend and consent behavior.
Only an available update uses the purple Manager action; the read-only check is
secondary. No Rust backend, MCP surface, generated catalog or startup instruction
changes are involved; no MCP restart is needed.

## Validation

- Strict production frontend syntax and linked AppKit fixture builds pass with
  `clang -fobjc-arc -Wall -Wextra -Werror -Wno-unused-parameter` and Cocoa.
- The extended hidden AppKit fixture passes existing state/action-routing checks
  plus compact previews, absent/damaged identities, separate runtime origin,
  exact full content/revision/path copying, Copy all, feedback reset, duplicate
  sheet protection and return to Settings. Clipboard tests use a private named
  pasteboard and release it; the user's clipboard is not used by these tests.
- Settings is checked at widths 620/800 and details at 480/700 in Aqua/Dark Aqua.
  Text frames, wrapped line heights and the shared Copy-button trailing edge are
  checked. A long path with spaces exercises wrapping and byte-preserving copying.
  Both legacy and overlay scrollers are exercised. The first PR CI run reproduced
  an assertion that incorrectly compared the content edge with the whole sheet,
  ignoring the legacy scroller gutter on macOS 15. Explicit legacy scrollers
  reproduced the failure locally; the corrected assertion uses the row content edge.
- The repository wrapper passes both tests:

  ```sh
  cargo test --locked --offline --no-default-features \
    --manifest-path rust/tooling/Cargo.toml \
    --target-dir /Users/bm/Documents/repos/inkscape-mcp-server/rust/tooling/target \
    --test manager_task
  ```

- An owned native preview with a unique bundle identity, synthetic profile and
  controlled component data was inspected interactively through CUA. The final
  view shows compact rows, a right-aligned details action, full selectable values,
  aligned Copy buttons, scrollable lower signing fields and a pinned footer.
  Escape returned to Settings. The preview cannot start helpers or update checks;
  no real installation, client, Inkscape window or MCP session was used. Only the
  owned preview windows were closed after inspection. Interactive copy was not
  needed; private-pasteboard automated checks cover that action.
- `git diff --check` passes.

An independent read-only agent reviewed the complete product diff and the
legacy/overlay-scroller follow-up before release. It found no actionable remaining
correctness or regression issue and independently passed strict frontend syntax
validation. Existing CodeRabbit review contained no concrete blocking finding or
inline comment. The focused wrapper passes with both scroller styles.

This is local source/UI validation, not a newly signed or installed release.
No full package, notarization, Gatekeeper or another-Mac acceptance is claimed.

## Artifact retention

Task-owned output is under `.inkscape-mcp-local/compact-identities/` with an
ownership record and a 5 MiB budget; its measured peak was about 1.5 MiB. The
shared warm tooling target was reused (about 847 MiB); no separate Cargo cache
was created. Fixture source, app metadata and native offscreen layout PNGs are
retained; the PNGs are not desktop screenshots. Disposable preview/test binaries
and the copied preview icon are removed after validation. Shared caches and other
tasks' artifacts are preserved. After cleanup the retained task directory is
348 KiB; filesystem free space is about 83 GiB. The original checkout's Git
status is unchanged.
