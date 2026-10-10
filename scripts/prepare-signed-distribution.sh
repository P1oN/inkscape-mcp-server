#!/bin/bash
# Protected CI only. Transform exact verified inputs; never rebuild shipped application code.
set -euo pipefail
set +x
fail() { printf '%s\n' "$*" >&2; exit 1; }
[ "${GITHUB_ACTIONS:-false}" = true ] || fail 'Signing preparation is restricted to protected GitHub CI.'
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || fail 'Native Apple Silicon signing runner required.'
[ "$#" = 2 ] || fail 'Usage: prepare-signed-distribution.sh CANDIDATE_DIRECTORY NEW_OUTPUT_DIRECTORY'
signing_candidate=$(cd -- "$1" && pwd -P)
signing_output=$2
[ ! -e "$signing_output" ] && [ ! -L "$signing_output" ] || fail 'Signing output must be new.'
signing_sha=$(jq -er .source_revision "$signing_candidate/CANDIDATE.json")
[ "$(git rev-parse HEAD)" = "$signing_sha" ] || fail 'Packaging checkout differs from verified candidate revision.'
(cd "$signing_candidate" && jq -r .input_sha256 CANDIDATE.json | shasum -a 256 -c -) >/dev/null
signing_private=$(mktemp -d "$RUNNER_TEMP/inkscape-signing.XXXXXX")
signing_keychain=$signing_private/release.keychain-db
signing_keychains=()
signing_search_changed=false
cleanup_signing() {
    local signing_cleanup_status=0
    if [ "$signing_search_changed" = true ]; then
        if ! security list-keychains -d user -s ${signing_keychains[@]+"${signing_keychains[@]}"} >/dev/null; then
            printf '%s\n' 'Failed to restore keychain search list; signing preparation cannot continue.' >&2
            signing_cleanup_status=1
        fi
    fi
    # Destroy credentials even if restoration fails; this isolated CI job must fail closed.
    if ! security delete-keychain "$signing_keychain" >/dev/null 2>&1 && [ -e "$signing_keychain" ]; then
        printf '%s\n' 'Failed to unregister signing keychain; removing its private files.' >&2
        signing_cleanup_status=1
    fi
    rm -rf -- "$signing_private"
    unset SIGNING_CERTIFICATE_BASE64 SIGNING_CERTIFICATE_PASSWORD NOTARY_KEY_BASE64
    return "$signing_cleanup_status"
}
trap cleanup_signing EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
# Keep raw credentials in unexported shell variables, including during Cargo build.rs.
signing_certificate=${SIGNING_CERTIFICATE_BASE64:?Signing certificate required}
signing_notary_key=${NOTARY_KEY_BASE64:?Notarization API key required}
signing_certificate_password=${SIGNING_CERTIFICATE_PASSWORD:?Certificate password required}
unset SIGNING_CERTIFICATE_BASE64 SIGNING_CERTIFICATE_PASSWORD NOTARY_KEY_BASE64
export -n signing_certificate signing_notary_key signing_certificate_password
RUSTUP_TOOLCHAIN=1.99.0 scripts/dev-tools.sh help >/dev/null
security list-keychains -d user > "$signing_private/search-list"
while IFS= read -r signing_path; do
    case "$signing_path" in
        *\"*\")
            # security prints literal contents inside framing quotes, not shell escapes.
            signing_path=${signing_path#*\"}
            signing_path=${signing_path%\"}
            signing_keychains+=("$signing_path") ;;
        '') ;;
        *) fail 'Unexpected keychain search-list format; nothing changed.' ;;
    esac
done < "$signing_private/search-list"
rm -f -- "$signing_private/search-list"
(umask 077; printf '%s' "$signing_certificate" | base64 -D > "$signing_private/certificate.p12")
(umask 077; printf '%s' "$signing_notary_key" | base64 -D > "$signing_private/notary.p8")
unset signing_certificate signing_notary_key
signing_password=$(openssl rand -hex 32)
security create-keychain -p "$signing_password" "$signing_keychain"
security set-keychain-settings -lut 21600 "$signing_keychain"
security unlock-keychain -p "$signing_password" "$signing_keychain"
signing_search_changed=true
security list-keychains -d user -s "$signing_keychain" ${signing_keychains[@]+"${signing_keychains[@]}"}
# Public Apple intermediates; preserve normal system trust evaluation (no trust override).
(cd scripts/certificates && printf '%s\n' \
    '7afc9d01a62f03a2de9637936d4afe68090d2de18d03f29c88cfb0b1ba63587f  DeveloperIDCA.cer' \
    'f16cd3c54c7f83cea4bf1a3e6a0819c8aaa8e4a1528fd144715f350643d2df3a  DeveloperIDG2CA.cer' | shasum -a 256 -c -) >/dev/null
security import scripts/certificates/DeveloperIDCA.cer -k "$signing_keychain" >/dev/null
security import scripts/certificates/DeveloperIDG2CA.cer -k "$signing_keychain" >/dev/null
security import "$signing_private/certificate.p12" -k "$signing_keychain" -P "$signing_certificate_password" -T /usr/bin/codesign >/dev/null
unset signing_certificate_password
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$signing_password" "$signing_keychain" >/dev/null
# Public certificate names/fingerprints and trust errors only; never dump key material.
security find-identity -p codesigning "$signing_keychain"
signing_identities=$(security find-identity -v -p codesigning "$signing_keychain")
signing_identity=${SIGNING_IDENTITY:?Developer ID Application identity required}
signing_identity_error() {
    security find-certificate -a -Z "$signing_keychain" || true
    fail 'Imported keychain has no valid matching signing identity; check the P12 certificate/private key and certificate trust chain.'
}
if [[ "$signing_identity" =~ ^[[:xdigit:]]{40}$ ]]; then
    signing_identity=$(printf '%s\n' "$signing_identities" | \
        grep -Ei "^[[:space:]]*[0-9]+\)[[:space:]]+$signing_identity[[:space:]]+\"" | \
        sed -n 's/^[^"]*"\([^"]*\)"[[:space:]]*$/\1/p') || signing_identity_error
else
    printf '%s\n' "$signing_identities" | grep -F -- "\"$signing_identity\"" >/dev/null || signing_identity_error
fi
case "$signing_identity" in
    'Developer ID Application: '*) ;;
    *) signing_identity_error ;;
esac
# The Rust distribution parser requires the full Developer ID name, not its SHA-1.
export SIGNING_IDENTITY="$signing_identity"
unset signing_identities signing_identity
xcrun notarytool store-credentials inkscape-release --keychain "$signing_keychain" --key "$signing_private/notary.p8" --key-id "${NOTARY_KEY_ID:?Key ID required}" --issuer "${NOTARY_ISSUER_ID:?Issuer ID required}" >/dev/null
rm -f -- "$signing_private/certificate.p12" "$signing_private/notary.p8"
unset signing_password
signing_tool=$PWD/rust/tooling/target/debug/inkscape-mcp-tools
"$signing_tool" unpack-runtime --archive "$signing_candidate/inkscape-mcp-macos-arm64.tar.gz" --output "$signing_private/extracted"
"$signing_tool" unpack-instructions --archive "$signing_candidate/inkscape-mcp-instructions.tar.gz" --output "$signing_private/text"
signing_package=$signing_private/extracted/inkscape-mcp-macos-arm64
codesign --force --sign "${SIGNING_IDENTITY:?Developer ID Application identity required}" --keychain "$signing_keychain" --timestamp --options runtime "$signing_tool"
signing_reference_options=()
if jq -e '.runtime_reference != null' "$signing_candidate/CANDIDATE.json" >/dev/null; then
    (cd "$signing_candidate/runtime-reference" && jq -r .runtime_reference_input_sha256 ../CANDIDATE.json | shasum -a 256 -c -) >/dev/null
    signing_reference_options=(--runtime-reference "$signing_candidate/runtime-reference")
fi
signing_channel=prerelease
[ "$(jq -r .prerelease "$signing_candidate/CANDIDATE.json")" = true ] || signing_channel=stable
"$signing_tool" build-distribution --package "$signing_package" --output "$signing_output" --tag "$(jq -r .tag "$signing_candidate/CANDIDATE.json")" --channel "$signing_channel" \
    ${signing_reference_options[@]+"${signing_reference_options[@]}"} --instructions "$signing_private/text/inkscape-mcp-instructions" --identity "$SIGNING_IDENTITY" --team-id "${SIGNING_TEAM_ID:?Team ID required}" --notary-profile inkscape-release --keychain "$signing_keychain"
signing_output=$(cd "$signing_output" && pwd -P)
# Remove credentials before executing any final packaged probe.
cleanup_signing
trap - EXIT INT TERM
mkdir "$signing_output/inkscape-mcp-launcher" "$signing_output/inkscape-mcp-launcher/bin"
cp "$signing_output/Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher" "$signing_output/inkscape-mcp-launcher/bin/"
cp -R "$signing_output/Inkscape MCP Manager.app" "$signing_output/inkscape-mcp-launcher/"
COPYFILE_DISABLE=1 tar -czf "$signing_output/inkscape-mcp-launcher.tar.gz" -C "$signing_output" inkscape-mcp-launcher
cp "$signing_candidate/inkscape-mcp-instructions.tar.gz" "$signing_output/"
"$signing_tool" build-update-manifest --runtime "$signing_output/inkscape-mcp-macos-arm64.tar.gz" --instructions "$signing_output/inkscape-mcp-instructions.tar.gz" --launcher "$signing_output/inkscape-mcp-launcher.tar.gz" --tag "$(jq -r .tag "$signing_candidate/CANDIDATE.json")" --output "$signing_output/inkscape-mcp-update.json"
jq --argjson prerelease "$(jq .prerelease "$signing_candidate/CANDIDATE.json")" --slurpfile candidate "$signing_candidate/CANDIDATE.json" '.prerelease=$prerelease | if $candidate[0].runtime_reference != null then .runtime=$candidate[0].runtime_reference else . end' "$signing_output/inkscape-mcp-update.json" > "$signing_output/update-final.json"
mv "$signing_output/update-final.json" "$signing_output/inkscape-mcp-update.json"
cp "$signing_tool" "$signing_output/packaging-tools"
cp "$signing_candidate/CANDIDATE.json" "$signing_output/"
cp "$signing_candidate/inkscape-mcp-source-bootstrap.tar.gz" "$signing_output/"
# Keep the legacy input only for isolated source-migration fixtures; it is not published.
"$signing_tool" unpack-runtime --archive "$signing_candidate/inkscape-mcp-macos-arm64.tar.gz" --output "$signing_output/acceptance-input"

for signing_asset in Inkscape-MCP-Manager.dmg inkscape-mcp-macos-arm64.tar.gz inkscape-mcp-instructions.tar.gz inkscape-mcp-launcher.tar.gz inkscape-mcp-source-bootstrap.tar.gz; do
    (cd "$signing_output" && shasum -a 256 "$signing_asset" > "$signing_asset.sha256")
done
jq --arg run "$GITHUB_RUN_ID" --arg revision "$signing_sha" --arg tool "$(shasum -a 256 "$signing_tool" | cut -d ' ' -f 1)" '. + {ci_run:("https://github.com/"+.repository+"/actions/runs/"+.run_id),packaging_revision:$revision,preparation_run:$run,packaging_tool_sha256:$tool}' "$signing_candidate/CANDIDATE.json" > "$signing_output/RELEASE-METADATA.json"
