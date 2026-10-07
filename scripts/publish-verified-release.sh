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
release_assets=(inkscape-mcp-macos-arm64.tar.gz inkscape-mcp-macos-arm64.tar.gz.sha256 inkscape-mcp-source-bootstrap.tar.gz inkscape-mcp-source-bootstrap.tar.gz.sha256 inkscape-mcp-instructions.tar.gz inkscape-mcp-instructions.tar.gz.sha256 inkscape-mcp-launcher.tar.gz inkscape-mcp-launcher.tar.gz.sha256 inkscape-mcp-update-template.json)
[ "$(find "$release_dir" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" = 9 ] || fail 'Unexpected CI artifact files.'
for release_asset in "${release_assets[@]}"; do
    [ -f "$release_dir/$release_asset" ] && [ ! -L "$release_dir/$release_asset" ] || fail "Missing or unsafe artifact: $release_asset"
done
for release_archive in inkscape-mcp-macos-arm64.tar.gz inkscape-mcp-source-bootstrap.tar.gz inkscape-mcp-instructions.tar.gz inkscape-mcp-launcher.tar.gz; do
    release_expected=$(cat "$release_dir/$release_archive.sha256")
    release_actual=$(cd "$release_dir" && shasum -a 256 "$release_archive")
    [ "$release_expected" = "$release_actual" ] || fail 'Archive checksum or filename differs from CI checksum.'
done
release_metadata=$(tar -xOf "$release_dir/inkscape-mcp-macos-arm64.tar.gz" inkscape-mcp-macos-arm64/libexec/inkscape-mcp/package.json)
jq -e --arg sha "$release_sha" '.source_head == $sha and .build_info.revision == $sha' <<< "$release_metadata" >/dev/null || fail 'Runtime source revision differs from CI.'
release_source=$(tar -xOf "$release_dir/inkscape-mcp-source-bootstrap.tar.gz" inkscape-mcp-source-bootstrap/SOURCE_REVISION)
[ "$release_source" = "$(printf 'inkscape-mcp-source-v1\n%s' "$release_sha")" ] || fail 'Source archive revision differs from CI.'
release_template=$release_dir/inkscape-mcp-update-template.json
jq -e --arg sha "$release_sha" --argjson package "$release_metadata" '
    .format == 1 and .runtime.format == 1 and .instructions.format == 1 and
    .runtime.source_revision == $sha and .runtime.build_id == $package.build_info.build_id and
    .runtime.os == "macos" and .runtime.architecture == "aarch64" and .runtime.minimum_os_major == 15 and
    .runtime.text_interface == 1 and .runtime.helper_protocol == 5 and .runtime.launcher_minimum == 1 and
    .instructions.text_interface == 1 and .instructions.launcher_minimum == 1
' "$release_template" >/dev/null || fail 'Unsupported update contract or CI/runtime identity mismatch.'
for release_component in runtime instruction_asset launcher_asset; do
    if [ "$release_component" = runtime ]; then release_asset_path='.runtime.asset'; else release_asset_path=".$release_component"; fi
    release_asset_name=$(jq -r "$release_asset_path.name" "$release_template")
    case "$release_asset_name" in inkscape-mcp-macos-arm64.tar.gz|inkscape-mcp-instructions.tar.gz|inkscape-mcp-launcher.tar.gz) ;; *) fail 'Unsafe update asset identity.';; esac
    release_hash=$(shasum -a 256 "$release_dir/$release_asset_name" | cut -d ' ' -f 1)
    release_bytes=$(wc -c < "$release_dir/$release_asset_name" | tr -d ' ')
    jq -e --arg hash "$release_hash" --argjson bytes "$release_bytes" "$release_asset_path | .sha256 == \$hash and .bytes == \$bytes" "$release_template" >/dev/null || fail 'Update asset checksum/length differs from verified CI evidence.'
done
release_instructions=$(tar -xOf "$release_dir/inkscape-mcp-instructions.tar.gz" inkscape-mcp-instructions/manifest.json)
jq -e --argjson instructions "$release_instructions" '.instructions == $instructions' "$release_template" >/dev/null || fail 'Instruction manifest differs from verified CI evidence.'
release_runtime_tag=${RELEASE_RUNTIME_TAG:-}
if [ -n "$release_runtime_tag" ]; then
    [[ "$release_runtime_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9]+([.-][A-Za-z0-9]+)*)?$ ]] || fail 'Invalid referenced runtime tag.'
    release_reference=$release_dir/reference
    mkdir "$release_reference"
    gh api "repos/$release_repo/releases/tags/$release_runtime_tag" | jq -e '.draft == false and .immutable == true' >/dev/null || fail 'Referenced runtime release must already be published and immutable.'
    gh release download "$release_runtime_tag" --repo "$release_repo" --pattern inkscape-mcp-update.json --dir "$release_reference"
    release_original_tag=$(jq -r .runtime.distribution_tag "$release_reference/inkscape-mcp-update.json")
    [[ "$release_original_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9]+([.-][A-Za-z0-9]+)*)?$ ]] || fail 'Unsafe runtime distribution reference.'
    gh api "repos/$release_repo/releases/tags/$release_original_tag" | jq -e '.draft == false and .immutable == true' >/dev/null || fail 'Original runtime distribution is not immutable.'
    gh release download "$release_original_tag" --repo "$release_repo" --pattern inkscape-mcp-macos-arm64.tar.gz --pattern RELEASE-METADATA.json --dir "$release_reference"
    release_reference_hash=$(shasum -a 256 "$release_reference/inkscape-mcp-macos-arm64.tar.gz" | cut -d ' ' -f 1)
    release_reference_bytes=$(wc -c < "$release_reference/inkscape-mcp-macos-arm64.tar.gz" | tr -d ' ')
    release_reference_metadata=$(tar -xOf "$release_reference/inkscape-mcp-macos-arm64.tar.gz" inkscape-mcp-macos-arm64/libexec/inkscape-mcp/package.json)
    jq -e --arg tag "$release_runtime_tag" --arg hash "$release_reference_hash" --argjson bytes "$release_reference_bytes" --argjson package "$release_reference_metadata" '
        .tag == $tag and .format == 1 and .runtime.format == 1 and
        .runtime.asset.name == "inkscape-mcp-macos-arm64.tar.gz" and .runtime.asset.sha256 == $hash and .runtime.asset.bytes == $bytes and
        .runtime.source_revision == $package.build_info.revision and .runtime.build_id == $package.build_info.build_id and
        .runtime.os == "macos" and .runtime.architecture == "aarch64" and .runtime.minimum_os_major == 15 and
        .runtime.text_interface == 1 and .runtime.helper_protocol == 5 and .runtime.launcher_minimum == 1
    ' "$release_reference/inkscape-mcp-update.json" >/dev/null || fail 'Referenced runtime identity/compatibility failed.'
    jq -e --argjson package "$release_reference_metadata" '.package_build == $package.build_info and (.ci_run | contains("/actions/runs/"))' "$release_reference/RELEASE-METADATA.json" >/dev/null || fail 'Referenced runtime lacks publication evidence.'
    release_metadata=$release_reference_metadata
    jq --arg tag "$release_tag" --argjson prerelease "$release_prerelease" --slurpfile reference "$release_reference/inkscape-mcp-update.json" '.tag=$tag | .runtime=$reference[0].runtime | .prerelease=$prerelease' "$release_template" > "$release_dir/inkscape-mcp-update.json"
    # Referenced archives stay in their original immutable release; do not upload them again.
    release_assets=("${release_assets[@]:2}")
else
    jq --arg tag "$release_tag" --argjson prerelease "$release_prerelease" '.tag=$tag | .runtime.distribution_tag=$tag | .prerelease=$prerelease' "$release_template" > "$release_dir/inkscape-mcp-update.json"
fi
# The template is CI evidence; the published contract is bound to the immutable distribution tag.
release_assets[${#release_assets[@]}-1]=inkscape-mcp-update.json
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
