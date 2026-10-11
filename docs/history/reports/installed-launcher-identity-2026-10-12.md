# Installed launcher identity — 2026-10-12

The published v0.1.7 Manager (`ea2ebc8d2255576c`) was relocated over the owned
v0.1.6 Manager in a separate profile on the configured macOS 27.0 host. Native
controls discovered and downloaded the published v0.1.7 runtime
`65afcd383ad71d71`, completed the combined update, rolled back to
`ec46066ff9bb32b0`, retried the update, and reported no remaining update. The
settings file stayed byte-identical and the original SVG retained SHA-256
`69ad36cc9f247f83691b3d2469f10d77687b435f1e393b84b1100cf0df4e1204`.
The owned Manager was closed after the checks. No drawing GUI was launched.

Both releases have identical instruction content `1bb8fc45fd410e26f83960e0ec958d807ac23b586bdec319b9d434148836a698`;
content deduplication retained the existing instruction version. This observation
qualifies a published runtime transition, not distinct-text update acceptance.
An initial app selection opened without the synthetic environment; its real-home
relocation prompt was cancelled before activation and the window closed. The
subsequent process and relocation destination were verified against the owned
profile before proceeding. These are configured-host observations, not browser
quarantine, another-Mac, authenticated client or live Inkscape acceptance.

Rollback exposed a status defect: the new bundled helper reported its own build
as the installed launcher's build, although the permanent launcher's verified
record and bytes remained those of v0.1.6. Operation and version responses now use
the same byte-bound bootstrap discovery already used by installation inspection.
The executing helper remains identified separately as `installer_build`. Legacy
installations without a record report an unknown launcher build; a damaged
record or either changed launcher copy reports damage and an unknown identity.
No existing launcher is executed to obtain its identity.

A regression invokes the actual management CLI with a deliberately different
recorded bootstrap and non-executable synthetic launcher bytes. It failed before
the repair by returning the executing binary's build. It also checks that changing
one installed copy refuses the saved identity. Native signed acceptance above
belongs to published v0.1.7; it does not qualify subsequently repaired binaries.

Local repair validation: all ten serial update-manager integration tests, all 37
library tests, all-target Clippy with warnings denied, format and diff checks pass.
The CLI regression explicitly failed on the original implementation before passing
with the repair. No MCP schema or authoring instructions changed; discovery
manifests do not require regeneration. Fresh native CI and signed acceptance
remain separate gates.
