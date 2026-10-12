#!/bin/bash
# Maintainer-only, headless rendering of deliberately authored vector sources.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
artwork=$repo/runtime/manager/artwork
engine=${INKSCAPE_BINARY:-/Applications/Inkscape.app/Contents/MacOS/inkscape}
[ -x "$engine" ] || { printf '%s\n' 'Set INKSCAPE_BINARY to an Inkscape executable.' >&2; exit 1; }
render_tmp=$(mktemp -d)
trap 'rm -rf -- "$render_tmp"' EXIT
mkdir "$render_tmp/Manager.iconset"
for size in 16 32 128 256 512; do
    for scale in 1 2; do
        suffix=
        [ "$scale" = 1 ] || suffix=@2x
        "$engine" "$artwork/manager-icon.svg" --export-type=png --export-width="$((size * scale))" --export-filename="$render_tmp/Manager.iconset/icon_${size}x${size}${suffix}.png"
    done
done
/usr/bin/iconutil -c icns "$render_tmp/Manager.iconset" -o "$artwork/Manager.icns"
for scale in 1 2; do
    "$engine" "$artwork/installer-background.svg" --export-type=png --export-width="$((720 * scale))" --export-filename="$render_tmp/background-$scale.png"
    /usr/bin/sips -s dpiWidth "$((72 * scale))" -s dpiHeight "$((72 * scale))" "$render_tmp/background-$scale.png" >/dev/null
done
/usr/bin/tiffutil -cathidpicheck "$render_tmp/background-1.png" "$render_tmp/background-2.png" -out "$artwork/installer-background.tiff"
printf '%s\n' 'Updated Manager.icns and installer-background.tiff; inspect before committing.'
