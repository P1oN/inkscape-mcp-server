#!/bin/bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
if [ "$#" -ne 2 ] || [ "$1" != --client ]; then
    echo 'Usage: ./uninstall.sh --client codex|claude' >&2
    echo 'Disconnects this installation and archives local settings/builds. Drawings and skills are preserved.' >&2
    exit 1
fi
exec "$repo/scripts/mcp-client.sh" --client "$2" uninstall
