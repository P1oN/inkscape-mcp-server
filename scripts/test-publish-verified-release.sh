#!/bin/bash
# Isolated release guard checks: fake GitHub CLI; no publication or archive code execution.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$repo"
fixture_root=$(mktemp -d)
trap 'rm -rf -- "$fixture_root"' EXIT
export RELEASE_FIXTURE_ROOT=$fixture_root
export GITHUB_REPOSITORY=example/repo RELEASE_TAG=v9.8.7 RELEASE_RUN_ID=123 RELEASE_PRERELEASE=true
fixture_sha=$(git rev-parse HEAD)
mkdir -p "$fixture_root/bin" "$fixture_root/artifacts" "$fixture_root/runtime/inkscape-mcp-macos-arm64/libexec/inkscape-mcp" "$fixture_root/source/inkscape-mcp-source-bootstrap"
jq -n --arg sha "$fixture_sha" '{source_head:$sha,build_info:{revision:$sha,build_id:"fixture"}}' > "$fixture_root/runtime/inkscape-mcp-macos-arm64/libexec/inkscape-mcp/package.json"
printf 'inkscape-mcp-source-v1\n%s\n' "$fixture_sha" > "$fixture_root/source/inkscape-mcp-source-bootstrap/SOURCE_REVISION"
tar -czf "$fixture_root/artifacts/inkscape-mcp-macos-arm64.tar.gz" -C "$fixture_root/runtime" inkscape-mcp-macos-arm64
tar -czf "$fixture_root/artifacts/inkscape-mcp-source-bootstrap.tar.gz" -C "$fixture_root/source" inkscape-mcp-source-bootstrap
for fixture_archive in inkscape-mcp-macos-arm64.tar.gz inkscape-mcp-source-bootstrap.tar.gz; do
    (cd "$fixture_root/artifacts" && shasum -a 256 "$fixture_archive" > "$fixture_archive.sha256")
done
mkdir -p "$fixture_root/instructions/inkscape-mcp-instructions" "$fixture_root/launcher/inkscape-mcp-launcher"
printf 'fixture launcher\n' > "$fixture_root/launcher/inkscape-mcp-launcher/launcher"
jq -n --arg sha "$fixture_sha" '{format:1,version:$sha,content_id:$sha,text_interface:1,launcher_minimum:1,files:{}}' > "$fixture_root/instructions/inkscape-mcp-instructions/manifest.json"
tar -czf "$fixture_root/artifacts/inkscape-mcp-instructions.tar.gz" -C "$fixture_root/instructions" inkscape-mcp-instructions
tar -czf "$fixture_root/artifacts/inkscape-mcp-launcher.tar.gz" -C "$fixture_root/launcher" inkscape-mcp-launcher
for fixture_archive in inkscape-mcp-instructions.tar.gz inkscape-mcp-launcher.tar.gz; do
    (cd "$fixture_root/artifacts" && shasum -a 256 "$fixture_archive" > "$fixture_archive.sha256")
done
fixture_asset() {
    local file=$fixture_root/artifacts/$1
    jq -n --arg name "$1" --arg hash "$(shasum -a 256 "$file" | cut -d ' ' -f 1)" --argjson bytes "$(wc -c < "$file" | tr -d ' ')" '{name:$name,bytes:$bytes,sha256:$hash}'
}
jq -n --arg sha "$fixture_sha" --argjson runtime "$(fixture_asset inkscape-mcp-macos-arm64.tar.gz)" --argjson instructions "$(fixture_asset inkscape-mcp-instructions.tar.gz)" --argjson launcher "$(fixture_asset inkscape-mcp-launcher.tar.gz)" --argjson bundle "$(cat "$fixture_root/instructions/inkscape-mcp-instructions/manifest.json")" '{format:1,tag:$sha,prerelease:true,runtime:{format:1,distribution_tag:$sha,build_id:"fixture",source_revision:$sha,os:"macos",architecture:"aarch64",minimum_os_major:15,text_interface:1,helper_protocol:5,launcher_minimum:1,asset:$runtime},instructions:$bundle,instruction_asset:$instructions,launcher_asset:$launcher}' > "$fixture_root/artifacts/inkscape-mcp-update-template.json"
jq -n --arg sha "$fixture_sha" '{repository:{full_name:"example/repo"},head_repository:{full_name:"example/repo"},workflow_id:42,event:"push",head_branch:"main",status:"completed",conclusion:"success",head_sha:$sha}' > "$fixture_root/run.json"
printf '[]\n' > "$fixture_root/tags.json"
printf '[]\n' > "$fixture_root/releases.json"
cat > "$fixture_root/bin/gh" <<'MOCK'
#!/bin/bash
set -euo pipefail
root=${RELEASE_FIXTURE_ROOT:?}
if [ "$1" = api ]; then
    shift
    [ "$1" != --paginate ] || shift
    case "$1" in
        */actions/workflows/rust-migration.yml) printf '42\n';;
        */actions/runs/123) cat "$root/run.json";;
        */git/matching-refs/tags/*) cat "$root/tags.json";;
        */releases\?per_page=100) cat "$root/releases.json";;
        */releases/tags/v1.0.0) printf '{"draft":false,"immutable":true}
';;
        *) exit 99;;
    esac
elif [ "$1 $2" = 'run download' ]; then
    while [ "$1" != --dir ]; do shift; done
    cp "$root/artifacts/"* "$2/"
elif [ "$1 $2" = 'release download' ]; then
    pattern= destination=
    while [ "$#" -gt 0 ]; do
        case "$1" in --pattern) pattern=$2; shift 2;; --dir) destination=$2; shift 2;; *) shift;; esac
    done
    if [ "$pattern" = inkscape-mcp-update.json ]; then
        cp "$root/reference/inkscape-mcp-update.json" "$destination/"
    else
        cp "$root/artifacts/inkscape-mcp-macos-arm64.tar.gz" "$root/reference/RELEASE-METADATA.json" "$destination/"
    fi
elif [ "$1" = release ]; then

    printf '%s\n' "$2" >> "$root/publication.log"
    if [ "$2" = upload ]; then
        [ "$#" -eq 15 ] || [ "$#" -eq 13 ]
        for asset in "${@:6}"; do [ -f "$asset" ]; done
    fi
else
    exit 99
fi
MOCK
chmod +x "$fixture_root/bin/gh"
export PATH="$fixture_root/bin:$PATH"
expect_refusal() {
    rm -f "$fixture_root/publication.log"
    if bash "$repo/scripts/publish-verified-release.sh" > "$fixture_root/output" 2>&1; then
        cat "$fixture_root/output"; printf 'Unexpected publication: %s\n' "$1" >&2; exit 1
    fi
    [ ! -e "$fixture_root/publication.log" ]
    printf '%s: refused before publication\n' "$1"
}
RELEASE_TAG=invalid expect_refusal 'invalid tag'
jq '.event="pull_request"' "$fixture_root/run.json" > "$fixture_root/invalid.json"
cp "$fixture_root/run.json" "$fixture_root/valid.json"
mv "$fixture_root/invalid.json" "$fixture_root/run.json"
expect_refusal 'PR run'
cp "$fixture_root/valid.json" "$fixture_root/run.json"
jq '.conclusion="failure"' "$fixture_root/valid.json" > "$fixture_root/run.json"
expect_refusal 'failed run'
cp "$fixture_root/valid.json" "$fixture_root/run.json"
printf '[{"ref":"refs/tags/v9.8.7"}]\n[]\n' > "$fixture_root/tags.json"
expect_refusal 'existing tag on earlier API page'
printf '[]\n' > "$fixture_root/tags.json"
printf '[{"tag_name":"v9.8.7"}]\n[]\n' > "$fixture_root/releases.json"
expect_refusal 'existing draft on earlier API page'
printf '[]\n' > "$fixture_root/releases.json"
printf 'bad checksum\n' > "$fixture_root/artifacts/inkscape-mcp-macos-arm64.tar.gz.sha256"
expect_refusal 'corrupt artifact'
(cd "$fixture_root/artifacts" && shasum -a 256 inkscape-mcp-macos-arm64.tar.gz > inkscape-mcp-macos-arm64.tar.gz.sha256)
cp "$fixture_root/artifacts/inkscape-mcp-macos-arm64.tar.gz" "$fixture_root/valid-runtime.tar.gz"
jq '.build_info.revision="0000000000000000000000000000000000000000"' "$fixture_root/runtime/inkscape-mcp-macos-arm64/libexec/inkscape-mcp/package.json" > "$fixture_root/invalid.json"
mv "$fixture_root/invalid.json" "$fixture_root/runtime/inkscape-mcp-macos-arm64/libexec/inkscape-mcp/package.json"
tar -czf "$fixture_root/artifacts/inkscape-mcp-macos-arm64.tar.gz" -C "$fixture_root/runtime" inkscape-mcp-macos-arm64
(cd "$fixture_root/artifacts" && shasum -a 256 inkscape-mcp-macos-arm64.tar.gz > inkscape-mcp-macos-arm64.tar.gz.sha256)
expect_refusal 'runtime revision mismatch'
cp "$fixture_root/valid-runtime.tar.gz" "$fixture_root/artifacts/inkscape-mcp-macos-arm64.tar.gz"
(cd "$fixture_root/artifacts" && shasum -a 256 inkscape-mcp-macos-arm64.tar.gz > inkscape-mcp-macos-arm64.tar.gz.sha256)
cp "$fixture_root/artifacts/inkscape-mcp-source-bootstrap.tar.gz" "$fixture_root/valid-source.tar.gz"
printf 'inkscape-mcp-source-v1\n0000000000000000000000000000000000000000\n' > "$fixture_root/source/inkscape-mcp-source-bootstrap/SOURCE_REVISION"
tar -czf "$fixture_root/artifacts/inkscape-mcp-source-bootstrap.tar.gz" -C "$fixture_root/source" inkscape-mcp-source-bootstrap
(cd "$fixture_root/artifacts" && shasum -a 256 inkscape-mcp-source-bootstrap.tar.gz > inkscape-mcp-source-bootstrap.tar.gz.sha256)
expect_refusal 'source revision mismatch'
cp "$fixture_root/valid-source.tar.gz" "$fixture_root/artifacts/inkscape-mcp-source-bootstrap.tar.gz"
(cd "$fixture_root/artifacts" && shasum -a 256 inkscape-mcp-source-bootstrap.tar.gz > inkscape-mcp-source-bootstrap.tar.gz.sha256)
bash "$repo/scripts/publish-verified-release.sh" > "$fixture_root/output" 2>&1 || { cat "$fixture_root/output"; exit 1; }
[ "$(cat "$fixture_root/publication.log")" = "$(printf 'create\nupload\nedit')" ]
printf 'valid artifacts: draft, ten assets, publication in order passed\n'
mkdir "$fixture_root/reference"
jq '.tag="v1.0.0" | .runtime.distribution_tag="v1.0.0"' "$fixture_root/artifacts/inkscape-mcp-update-template.json" > "$fixture_root/reference/inkscape-mcp-update.json"
jq -n --arg sha "$fixture_sha" '{package_build:{revision:$sha,build_id:"fixture"},ci_run:"https://github.com/example/repo/actions/runs/122"}' > "$fixture_root/reference/RELEASE-METADATA.json"
rm "$fixture_root/publication.log"
RELEASE_RUNTIME_TAG=v1.0.0 bash "$repo/scripts/publish-verified-release.sh" > "$fixture_root/output" 2>&1 || { cat "$fixture_root/output"; exit 1; }
[ "$(cat "$fixture_root/publication.log")" = "$(printf 'create\nupload\nedit')" ]
printf 'reused immutable runtime: reference verified without execution or republication passed\n'
jq '.runtime.helper_protocol=99' "$fixture_root/reference/inkscape-mcp-update.json" > "$fixture_root/reference/invalid.json"
mv "$fixture_root/reference/invalid.json" "$fixture_root/reference/inkscape-mcp-update.json"
RELEASE_RUNTIME_TAG=v1.0.0 expect_refusal 'incompatible reused runtime'
