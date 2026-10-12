# Manager DMG presentation — 2026-10-12

Development in `codex/manager-dmg-design`, initially based on
`1806dac7bce8320e6e1a69e8a7520bd49e91e887`, in the managed
`manager-dmg-design/inkscape-mcp-server` worktree. The original checkout is reserved
for concurrent work and was not edited. No publication, installation or GUI launch
of Inkscape was performed. The user subsequently authorized a commit and PR to `main`;
release publication remains outside this scope.

## Implementation

The approved image concept is implemented with deliberately authored editable SVG
masters, not tracing: light Bezier background and an ivory mountain/pen/connected-node
app icon. Checked-in `.icns` and 1×/2× TIFF resources make ordinary builds independent
of an image generator, a rendering engine, Finder automation and Python. The optional
Bash maintainer renderer uses headless Inkscape and native image tools.

The package builder includes the icon and `CFBundleIconFile` before signing. The
distribution builder retains its existing app/runtime verification, signing and
notarization order; Finder presentation is finished before final DMG signing.
The bounded native development helper emits a six-record, one-leaf DS_Store,
native relative background alias. The resize follow-up stores the TIFF inside the
Manager's Resources before signing, without a separate root background folder, and
fades the artwork's right/bottom edges to white. Finder bounds specify an initial
size rather than a resize lock. A temporary writable
HFS+ image is detached, compressed, remounted readonly at a different path and checked.
Detach failures retain scratch directories rather than deleting a mounted tree.
The helper, artwork masters and rendering tools are not copied into the Manager.

The two draggable objects are the genuine Manager app and `/Applications` shortcut.
A signed Manager explicitly placed in `/Applications` reports ready without creating
a duplicate per-user copy. Launches from the image, Downloads or other locations
retain the existing guarded relocation to `~/Applications`. The test covers both
accepted installation locations and rejects Downloads, mounted images, other app
names and nested subfolders as already-installed locations.

## Local validation

- Both locked Cargo test graphs passed: 346 runtime tests, two opt-in ignored;
  tooling 29 tests, one explicit native layout test ignored by default.
- Both all-target Clippy graphs with warnings denied and both format checks passed.
- Explicit native layout test passed: writable mount, metadata/alias checks,
  UDZO conversion, different readonly mount and existing-output refusal.
- Objective-C helper compiles with warnings denied (deprecated Carbon alias APIs
  are explicitly allowed only in this development helper).
- Artwork renderer, Bash syntax and `git diff --check` passed.
- Signed-distribution/publication fixtures passed without external publication.
- Fresh source-built package and full ad-hoc `build-distribution` passed, including
  final-image alias checks. Final-layout runtime package acceptance passed (452 files,
  per-call CLI engine, actual headless rendering and preservation checks).
- Final readonly mount independently passed `codesign --verify --deep --strict`
  for the app, `CFBundleIconFile=Manager.icns` and the background's hidden attribute.
- Signed Manager installer acceptance was attempted and correctly refused at the
  preflight: `Manager acceptance requires a verified matching Developer ID helper`.
  The local app/tooling are ad-hoc, so this does not qualify that signed-only matrix
  and the guard was not bypassed.
- Runtime-only installer acceptance passed all 19 publication boundaries, custom-engine
  preservation/repair, real doctor/STDIO probes, ownership/refusal/recovery, no-ops and
  settings/artwork preservation with synthetic client homes (`native_gui=false`).
- `iconutil` extracted all ten native icon representations; `tiffutil` confirms both
  TIFF pixel sizes and AppKit reports the intended overall 720×480-point image size.

Ignored local evidence is under `.inkscape-mcp-local/` in this worktree. The candidate
package build is `948367db4b373012`; final cosmetic metadata was assembled separately
from that unchanged package, using current development tooling. The final image is:

```text
dmg-distribution-final/Inkscape-MCP-Manager.dmg
SHA-256 5524534e85b48e4aa11aa4a94c9c6fc6500169b7c4195585a7d2e7679f11e080
57,345,212 bytes
```

## Native Finder scope and remaining release gates

### Resize follow-up

The new candidate replaces the separate `.background` directory with the TIFF sealed
in `Inkscape MCP Manager.app/Contents/Resources/installer-background.tiff`. Packaging
writes it before app signing; DMG assembly only references it and refuses an app
without it. The generated background was inspected visually: right/bottom edges
fade fully to white, matching Finder's expanded canvas. Finder remains resizable.

All 29 tooling tests passed (one native test ignored by default), plus the explicit
native compression/remount test, tooling Clippy with warnings denied, format, Bash
syntax and diff checks. A fresh package (463 files) and full ad-hoc distribution
passed. Independent readonly mounting verified the app's deep/strict signature,
exact bundled TIFF bytes and absence of the `.background` root entry; the image
was detached. No new native Finder screenshot or signed-release qualification is
claimed. Runtime sources were unchanged in this follow-up; their earlier checks
above were not repeated.

Current local preview, superseding the earlier final-layout image:

```text
dmg-distribution-resize/Inkscape-MCP-Manager.dmg
SHA-256 38c5dc0046a1f25486288e15cdde1640a468e630ab1fb3fc93fa7a1647ca5d86
57,512,763 bytes
```

Evidence: `.inkscape-mcp-local/resize-{tooling-tests,native-test,clippy,package-build,distribution-build,mounted-check}.log`.

### Earlier Finder observation

The initial actual Finder window showed the title, background, Manager icon, chevron
and native Applications icon in the intended arrangement. This host enables global
status/path bars and filename extensions; those preferences can override presentation
flags and are not changed by packaging. That first window exposed a vertical-scroll
allowance issue, corrected by reserving 32 points for the current titlebar (stored
window bounds 720×512 for 720×480 artwork), and the background directory now has an
explicit hidden attribute. Both changes passed the final native metadata/remount test.

After ejecting the initial image, CUA returned `cgWindowNotFound` even after rebinding;
the final visual screenshot is therefore not claimed. The final image was independently
mounted readonly, inspected and detached through the disk-image CLI. No global Finder
settings, application copies or existing user windows were modified intentionally.

This is an ad-hoc, unnotarized local checkpoint. It does not establish a protected-CI
Developer ID release, clean-host quarantine acceptance or actual drag replacement of
an installed user app. Existing release/another-Mac gates remain applicable. No MCP
tool, schema or server instruction changed; llms catalogs and installed MCP connections
do not need regeneration/restart for this presentation change.
