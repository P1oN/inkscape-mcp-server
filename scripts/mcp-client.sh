#!/bin/bash
# Use the installed native client manager independently of the helper runtime.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
config=$repo/.inkscape-mcp-local/setup.conf
[ -f "$config" ] && [ ! -L "$config" ] && [ ! -L "$repo/.inkscape-mcp-local" ] || { echo 'Run setup.sh first.' >&2; exit 1; }
binary=$(sed -n '2p' "$config")
case "$binary" in /*/bin/inkscape-mcp) ;; *) echo 'Invalid saved package.' >&2; exit 1;; esac
manager=${binary%/bin/inkscape-mcp}/bin/inkscape-mcp-client
[ -x "$manager" ] || { echo 'Configured Rust client CLI is missing; run setup.sh --rebuild.' >&2; exit 1; }
exec "$manager" --repo "$repo" "$@"
