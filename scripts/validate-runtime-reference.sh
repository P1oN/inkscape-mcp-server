#!/bin/bash
# Read-only immutable signed-runtime reference validation; no application execution.
set -euo pipefail
[ "$#" = 2 ] || exit 1
reference_candidate=$(cd "$1" && pwd -P)
reference_requested=$2
[[ "$reference_requested" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9]+([.-][A-Za-z0-9]+)*)?$ ]] || exit 1
reference_repo=${GITHUB_REPOSITORY:?}
reference_root=$reference_candidate/runtime-reference
mkdir "$reference_root"
gh api "repos/$reference_repo/releases/tags/$reference_requested" | jq -e '.draft == false and .immutable == true' >/dev/null
gh release download "$reference_requested" --repo "$reference_repo" --pattern inkscape-mcp-update.json --dir "$reference_root"
reference_original=$(jq -er .runtime.distribution_tag "$reference_root/inkscape-mcp-update.json")
[[ "$reference_original" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9]+([.-][A-Za-z0-9]+)*)?$ ]] || exit 1
mv "$reference_root/inkscape-mcp-update.json" "$reference_root/requested-update.json"
gh api "repos/$reference_repo/releases/tags/$reference_original" | jq -e '.draft == false and .immutable == true' >/dev/null
gh release download "$reference_original" --repo "$reference_repo" --pattern inkscape-mcp-update.json --pattern CANDIDATE.json --pattern distribution-evidence.json --pattern FINAL-ACCEPTANCE.json --pattern inkscape-mcp-macos-arm64.tar.gz --dir "$reference_root"
scripts/release-guard.sh verify-runtime-reference "$reference_root"
reference_hashes=$(cd "$reference_root" && shasum -a 256 requested-update.json inkscape-mcp-update.json CANDIDATE.json distribution-evidence.json FINAL-ACCEPTANCE.json inkscape-mcp-macos-arm64.tar.gz)
jq --slurpfile original "$reference_root/inkscape-mcp-update.json" --arg hashes "$reference_hashes" '.runtime_reference=$original[0].runtime | .runtime_reference_input_sha256=$hashes' "$reference_candidate/CANDIDATE.json" > "$reference_candidate/reference-candidate.json"
mv "$reference_candidate/reference-candidate.json" "$reference_candidate/CANDIDATE.json"
