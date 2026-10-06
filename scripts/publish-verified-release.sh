#!/bin/bash
# Publish only archives produced by a successful main CI run. Never execute archive contents.
set -euo pipefail
fail() { printf '%s\n' "$*" >&2; exit 1; }
release_repo=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}
release_tag=${RELEASE_TAG:?RELEASE_TAG is required}
release_run=${RELEASE_RUN_ID:?RELEASE_RUN_ID is required}
release_prerelease=${RELEASE_PRERELEASE:-true}
[[ "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9]+([.-][A-Za-z0-9]+)*)?$ ]] || fail 'Invalid release tag.'
[[ "$release_run" =~ ^[1-9][0-9]*$ ]] || fail 'Invalid CI run ID.'
case "$release_prerelease" in true|false) ;; *) fail 'Invalid prerelease flag.';; esac

release_workflow=$(gh api "repos/$release_repo/actions/workflows/rust-migration.yml" --jq .id)
release_info=$(gh api "repos/$release_repo/actions/runs/$release_run")
jq -e --arg repo "$release_repo" --argjson workflow "$release_workflow" '
    .repository.full_name == $repo and .head_repository.full_name == $repo and
    .workflow_id == $workflow and .event == "push" and .head_branch == "main" and
    .status == "completed" and .conclusion == "success"
' <<< "$release_info" >/dev/null || fail 'Release requires a successful main push run of the native workflow.'
release_sha=$(jq -r .head_sha <<< "$release_info")
[[ "$release_sha" =~ ^[0-9a-f]{40}$ ]] || fail 'Invalid CI source revision.'
git cat-file -e "$release_sha^{commit}"
git merge-base --is-ancestor "$release_sha" HEAD || fail 'CI revision is not in checked-out main history.'
# Read the complete tag list so a network/auth failure cannot masquerade as a missing tag.
release_tags=$(gh api --paginate "repos/$release_repo/git/matching-refs/tags/$release_tag")
jq -e -s --arg ref "refs/tags/$release_tag" 'add | all(.[]; .ref != $ref)' <<< "$release_tags" >/dev/null || fail 'Release tag already exists; published versions are never replaced.'
release_list=$(gh api --paginate "repos/$release_repo/releases?per_page=100")
jq -e -s --arg tag "$release_tag" 'add | all(.[]; .tag_name != $tag)' <<< "$release_list" >/dev/null || fail 'Release already exists, including a possible failed draft; inspect it before retrying.'

release_dir=$(mktemp -d)
trap 'rm -rf -- "$release_dir"' EXIT
gh run download "$release_run" --repo "$release_repo" --name inkscape-mcp-macos-arm64 --dir "$release_dir"
release_assets=(inkscape-mcp-macos-arm64.tar.gz inkscape-mcp-macos-arm64.tar.gz.sha256 inkscape-mcp-source-bootstrap.tar.gz inkscape-mcp-source-bootstrap.tar.gz.sha256)
[ "$(find "$release_dir" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" = 4 ] || fail 'Unexpected CI artifact files.'
for release_asset in "${release_assets[@]}"; do
    [ -f "$release_dir/$release_asset" ] && [ ! -L "$release_dir/$release_asset" ] || fail "Missing or unsafe artifact: $release_asset"
done
for release_archive in inkscape-mcp-macos-arm64.tar.gz inkscape-mcp-source-bootstrap.tar.gz; do
    release_expected=$(cat "$release_dir/$release_archive.sha256")
    release_actual=$(cd "$release_dir" && shasum -a 256 "$release_archive")
    [ "$release_expected" = "$release_actual" ] || fail 'Archive checksum or filename differs from CI checksum.'
done
release_metadata=$(tar -xOf "$release_dir/inkscape-mcp-macos-arm64.tar.gz" inkscape-mcp-macos-arm64/libexec/inkscape-mcp/package.json)
jq -e --arg sha "$release_sha" '.source_head == $sha and .build_info.revision == $sha' <<< "$release_metadata" >/dev/null || fail 'Runtime source revision differs from CI.'
release_source=$(tar -xOf "$release_dir/inkscape-mcp-source-bootstrap.tar.gz" inkscape-mcp-source-bootstrap/SOURCE_REVISION)
[ "$release_source" = "$(printf 'inkscape-mcp-source-v1\n%s' "$release_sha")" ] || fail 'Source archive revision differs from CI.'
jq -n --arg tag "$release_tag" --arg sha "$release_sha" --arg run "$release_run" --arg repo "$release_repo" --argjson package "$release_metadata" \
    '{tag:$tag,source_revision:$sha,ci_run:("https://github.com/"+$repo+"/actions/runs/"+$run),package_build:$package.build_info}' > "$release_dir/RELEASE-METADATA.json"
printf 'Source revision: `%s`\n\nVerified CI: https://github.com/%s/actions/runs/%s\n\nApple Silicon macOS 15+ with Inkscape 1.4+. Checksums and compiled build identity are included. Native GUI and clean-machine acceptance are separately scoped.\n' "$release_sha" "$release_repo" "$release_run" > "$release_dir/notes.md"
# Stage all assets before publishing an immutable release. Failure leaves a draft for inspection.
release_options=(--repo "$release_repo" --target "$release_sha" --title "$release_tag" --draft --generate-notes --notes-file "$release_dir/notes.md")
[ "$release_prerelease" = false ] || release_options+=(--prerelease)
gh release create "$release_tag" "${release_options[@]}"
release_uploads=()
for release_asset in "${release_assets[@]}" RELEASE-METADATA.json; do release_uploads+=("$release_dir/$release_asset"); done
gh release upload "$release_tag" --repo "$release_repo" "${release_uploads[@]}"
gh release edit "$release_tag" --repo "$release_repo" --draft=false
