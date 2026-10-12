# Manager distribution artwork

These are deliberately authored vector sources for the language-neutral DMG concept
approved on 2026-10-12. No tracing or embedded raster artwork is used in the SVGs.
The ink mountain/pen and connected nodes form the Manager icon; faint Bezier control
handles connect the installer presentation to vector editing. Project license applies.

- `manager-icon.svg`: transparent 1024-square master, with a rounded ivory tile.
- `Manager.icns`: native icon sizes 16/32/128/256/512 points with 1× and 2× variants.
- `installer-background.svg`: 720×480 logical points; product title, chevron and accents.
- `installer-background.tiff`: 720×480 and 1440×960 representations; AppKit reports
  an overall logical image size of 720×480 points.

Regenerate from the repository root with `bash scripts/render-manager-artwork.sh`.
Set `INKSCAPE_BINARY` if Inkscape is installed elsewhere. Rendering is headless.
Review regenerated files visually; font rendering/PNG metadata can vary by host.
The rendered concept is a visual reference, not a production asset or a traced input.
Finder supplies the draggable icons and filename labels; do not bake them into the
background. Applications uses the OS folder icon and a standard `/Applications` link.
The right and bottom edges fade fully to white to blend with Finder's canvas on resize.
Packaging seals the TIFF in the app's `Contents/Resources` before signing; the DMG
references that resource instead of adding a separate background folder at its root.
