#!/bin/bash
# Fixed native probes only, after certificate and notarization key deletion.
set -euo pipefail
[ "$#" = 2 ] || { printf '%s\n' 'Usage: accept-signed-distribution.sh PREPARED_DIRECTORY DISTINCT_BASELINE_DIRECTORY' >&2; exit 1; }
acceptance_root=$(cd "$1" && pwd -P)
acceptance_baseline=$(cd "$2" && pwd -P)
acceptance_team=${EXPECTED_SIGNING_TEAM:?Expected Team ID required}
[[ "$acceptance_team" =~ ^[A-Z0-9]{10}$ ]] || exit 1
acceptance_requirement="=anchor apple generic and certificate leaf[subject.OU] = \"$acceptance_team\""
acceptance_tool=$acceptance_root/packaging-tools
codesign --verify --strict -R "$acceptance_requirement" "$acceptance_tool"
codesign --display --verbose=4 "$acceptance_tool" 2>&1 | python3 scripts/verify-signature-info.py native
acceptance_package=$acceptance_root/inkscape-mcp-macos-arm64
codesign --verify --deep --strict -R "$acceptance_requirement" "$acceptance_package/Inkscape MCP Runtime.app"
codesign --verify --deep --strict -R "$acceptance_requirement" "$acceptance_root/Inkscape MCP Manager.app"
xcrun stapler validate "$acceptance_package/Inkscape MCP Runtime.app"
xcrun stapler validate "$acceptance_root/Inkscape MCP Manager.app"
xcrun stapler validate "$acceptance_root/Inkscape-MCP-Manager.dmg"
for acceptance_app in "$acceptance_package/Inkscape MCP Runtime.app" "$acceptance_root/Inkscape MCP Manager.app"; do
    codesign --display --verbose=4 "$acceptance_app" 2>&1 | python3 scripts/verify-signature-info.py native
done
acceptance_helper="$acceptance_root/Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher"
codesign --verify --strict -R "$acceptance_requirement" "$acceptance_helper"
codesign --display --verbose=4 "$acceptance_helper" 2>&1 | python3 scripts/verify-signature-info.py native
codesign --verify --strict -R "$acceptance_requirement" "$acceptance_root/Inkscape-MCP-Manager.dmg"
codesign --display --verbose=4 "$acceptance_root/Inkscape-MCP-Manager.dmg" 2>&1 | python3 scripts/verify-signature-info.py container

while IFS= read -r native_path; do
    codesign --verify --strict -R "$acceptance_requirement" "$acceptance_package/$native_path"
    codesign --display --verbose=4 "$acceptance_package/$native_path" 2>&1 | python3 scripts/verify-signature-info.py native
done < <(jq -er '.code[].path' "$acceptance_root/distribution-evidence.json")
"$acceptance_tool" package-acceptance --package "$acceptance_package" --output "$acceptance_root/acceptance/package" --engine-mode per_call
"$acceptance_tool" installer-acceptance --manager-app "$acceptance_root/Inkscape MCP Manager.app" --package "$acceptance_package" --output "$acceptance_root/acceptance/installer"
"$acceptance_tool" update-acceptance --package "$acceptance_root/acceptance-input/inkscape-mcp-macos-arm64" --runtime-package "$acceptance_package" --previous-package "$acceptance_baseline" --runtime-archive "$acceptance_root/inkscape-mcp-macos-arm64.tar.gz" --output "$acceptance_root/acceptance/updates"
python3 scripts/record-signed-acceptance.py "$acceptance_root"
python3 scripts/verify-signed-distribution.py "$acceptance_root" >/dev/null
