# Repository artwork

All artwork here was authored through the project's configured Inkscape MCP using
typed shape, path, text, gradient, group and composition tools. PNGs are Inkscape
exports. SVG masters retain text, named groups, IDs and editable geometry.
The artwork follows the repository's MIT license.

| Asset | Editable master | PNG | Intended use |
| --- | --- | --- | --- |
| Project cover | [SVG](github-hero.svg) | [1440 × 720](github-hero.png) | README hero |
| Request flow | [SVG](workflow.svg) | [1440 × 600](workflow.png) | Architecture overview |
| Editable vectors | [SVG](editable-vectors.svg) | [1440 × 760](editable-vectors.png) | Feature illustration |
| First installation | [SVG](quickstart.svg) | [1440 × 660](quickstart.png) | Quickstart overview |
| Social preview | [SVG](social-preview.svg) | [1280 × 640](social-preview.png) | Share card / social preview candidate |
| Bezier bloom | [SVG](bezier-bloom.svg) | [640 × 480](bezier-bloom.png) | Reusable transparent vector sample |

The bloom is illustrative artwork, not a replacement for the Inkscape application
logo or the existing Manager icon. Committing the social card does not set
GitHub's repository social-preview setting.

## Design and portability

The palette is warm paper `#F4F1EA`, dark ink `#142C2B`, muted green `#57706A`,
coral `#E86D4A`, teal `#196D67` and lime `#D8E9B8`. SVG text uses Arial;
the PNGs preserve the reviewed rendering independently of viewer fonts.
The graphics have opaque backgrounds for consistent GitHub light/dark display,
except the standalone bloom, which has a transparent background.

The [provenance record](provenance.json) retains tool document IDs, save operations,
quality results, server-relative artifact URIs and SHA-256/byte counts. Local copies
were checked against `stat_artifact`. All six passed the MCP vector-only save gate.

See the [sprint report](../history/reports/mcp-showcase-2026-11-15.md) for the five
tasks, repairs, visual review and validation limits.
