#!/bin/bash
# Minimal publication job. Native verification runs earlier, without write permissions.
set -euo pipefail
[ "$#" = 1 ] || [ "$#" = 3 ] || { printf '%s\n' 'Usage: publish-signed-distribution.sh VERIFIED_ASSET_DIRECTORY [--recover-draft RELEASE_ID]' >&2; exit 1; }
publication_recovery=
if [ "$#" = 3 ]; then
    [ "$2" = --recover-draft ] && [[ "$3" =~ ^[1-9][0-9]{0,19}$ ]] || { printf '%s\n' 'Recovery requires an explicit numeric draft release ID.' >&2; exit 1; }
    publication_recovery=$3
fi
publication_assets=$(cd "$1" && pwd -P)
publication_inventory=$(scripts/release-guard.sh verify-signed-distribution "$publication_assets")
publication_tag=$(jq -er .tag "$publication_assets/CANDIDATE.json")
publication_sha=$(jq -er .source_revision "$publication_assets/CANDIDATE.json")
publication_repo=${GITHUB_REPOSITORY:?}
publication_tags=$(gh api --paginate "repos/$publication_repo/git/matching-refs/tags/$publication_tag")
jq -e -s --arg ref "refs/tags/$publication_tag" --arg sha "$publication_sha" --arg recovery "$publication_recovery" 'add | all(.[]; .ref != $ref or ($recovery != "" and .object.type == "commit" and .object.sha == $sha))' <<< "$publication_tags" >/dev/null
publication_releases=$(gh api --paginate "repos/$publication_repo/releases?per_page=100")
jq -e -s --arg tag "$publication_tag" --arg recovery "$publication_recovery" 'add | all(.[]; .tag_name != $tag or ($recovery != "" and (.id|tostring) == $recovery and .draft == true))' <<< "$publication_releases" >/dev/null || { printf '%s\n' 'Tag/release/draft exists; explicit recovery is required.' >&2; exit 1; }
publication_existing='[]'
if [ -n "$publication_recovery" ]; then
    publication_draft=$(gh api "repos/$publication_repo/releases/$publication_recovery")
    jq -e --arg id "$publication_recovery" --arg tag "$publication_tag" --arg sha "$publication_sha" --argjson preview "$(jq -r .prerelease "$publication_assets/CANDIDATE.json")" --argjson local "$publication_inventory" '
        (.id|tostring) == $id and .draft == true and .tag_name == $tag and
        .target_commitish == $sha and .prerelease == $preview and
        ((.assets|map(.name)|unique|length) == (.assets|length)) and
        all(.assets[]; $local[.name] != null and $local[.name].bytes == .size and
            ("sha256:"+$local[.name].sha256) == .digest)
    ' <<< "$publication_draft" >/dev/null || { printf '%s\n' 'Draft identity or existing asset digests differ; nothing changed.' >&2; exit 1; }
    publication_existing=$(jq -c '[.assets[].name]' <<< "$publication_draft")
fi
printf '%s\n' 'Download Inkscape-MCP-Manager.dmg for Apple Silicon macOS 15+ and Inkscape 1.4+.' \
    'Open the Manager, choose clients and an SVG workspace, review installation, then restart Codex/Claude.' \
    'Existing settings and artwork are preserved. Component/source archives are advanced downloads.' > "$RUNNER_TEMP/signed-release-notes.md"
publication_options=(--repo "$publication_repo" --target "$publication_sha" --title "$publication_tag" --draft --notes-file "$RUNNER_TEMP/signed-release-notes.md")
[ "$(jq -r .prerelease "$publication_assets/CANDIDATE.json")" = false ] || publication_options+=(--prerelease)
if [ -z "$publication_recovery" ]; then gh release create "$publication_tag" "${publication_options[@]}"; fi
publication_files=()
while IFS= read -r name; do publication_files+=("$publication_assets/$name"); done < <(jq -r --argjson existing "$publication_existing" 'keys[] | select(. as $name | $existing | index($name) | not)' <<< "$publication_inventory")
if [ "${#publication_files[@]}" -gt 0 ]; then gh release upload "$publication_tag" --repo "$publication_repo" "${publication_files[@]}"; fi
publication_id=$publication_recovery
if [ -z "$publication_id" ]; then
    publication_created=$(gh api --paginate "repos/$publication_repo/releases?per_page=100")
    publication_id=$(jq -er -s --arg tag "$publication_tag" 'add | map(select(.tag_name == $tag and .draft == true)) | if length == 1 then .[0].id else error("Expected one matching draft") end' <<< "$publication_created")
fi
[[ "$publication_id" =~ ^[1-9][0-9]{0,19}$ ]] || exit 1
publication_remote=$(gh api "repos/$publication_repo/releases/$publication_id")
jq -e --arg id "$publication_id" --arg tag "$publication_tag" --arg sha "$publication_sha" --argjson preview "$(jq -r .prerelease "$publication_assets/CANDIDATE.json")" --argjson local "$publication_inventory" '(.id|tostring) == $id and .prerelease == $preview and .draft == true and .tag_name == $tag and .target_commitish == $sha and (.assets|length) == ($local|length) and (.assets|map(.name)|unique|length) == ($local|length) and all(.assets[]; $local[.name].bytes == .size and ("sha256:"+$local[.name].sha256) == .digest)' <<< "$publication_remote" >/dev/null || { printf '%s\n' 'Remote draft identity or asset digests/count differ; draft retained.' >&2; exit 1; }
gh api --method PATCH "repos/$publication_repo/releases/$publication_id" -F draft=false >/dev/null
