#!/bin/bash
# Use the configured private interpreter, never require a system Python installation.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
config=$repo/.inkscape-mcp-local/setup.conf
[ -f "$config" ] && [ ! -L "$config" ] && [ ! -L "$repo/.inkscape-mcp-local" ] || { echo 'Run setup.sh first.' >&2; exit 1; }
binary=$(sed -n '2p' "$config")
case "$binary" in /*/bin/inkscape-mcp) ;; *) echo 'Invalid saved package.' >&2; exit 1;; esac
python=${binary%/bin/inkscape-mcp}/libexec/inkscape-mcp/python/bin/python3
[ -x "$python" ] || { echo 'Configured private Python is missing; run setup.sh --rebuild.' >&2; exit 1; }
exec "$python" -I "$repo/scripts/mcp_client.py" --repo "$repo" "$@"
