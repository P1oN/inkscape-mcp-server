#!/bin/bash
# Rust release guards and synthetic shell fixtures; no signing or publication.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$repo"
scripts/release-guard.sh verify-signature-info native <<'SIGNATURE'
Authority=Developer ID Application: Fixture
Timestamp=Oct 8, 2026 at noon
CodeDirectory flags=0x10000(runtime)
SIGNATURE
export RUSTUP_TOOLCHAIN=1.99.0
fixture_cargo=$(command -v cargo || printf '%s/.cargo/bin/cargo' "$HOME")
"$fixture_cargo" test --locked --manifest-path rust/tooling/Cargo.toml --no-default-features --bin inkscape-mcp-release-guard
fixture_root=$(mktemp -d)
trap 'rm -rf -- "$fixture_root"' EXIT
mkdir "$fixture_root/scripts" "$fixture_root/mock" "$fixture_root/private"
cat > "$fixture_root/probe" <<'PROBE'
set -eu
for name in SIGNING_CERTIFICATE_BASE64 SIGNING_CERTIFICATE_PASSWORD NOTARY_KEY_BASE64 signing_certificate signing_certificate_password signing_notary_key; do
    ! /usr/bin/env | /usr/bin/grep -q "^$name=" || exit 9
done
PROBE
{
    printf '#!/bin/bash\n'
    cat "$fixture_root/probe"
    printf 'test "$RUSTUP_TOOLCHAIN" = 1.99.0\ntest ! -e "$signing_private/certificate.p12"\n'
} > "$fixture_root/scripts/dev-tools.sh"
{
    printf '#!/bin/bash\n'
    cat "$fixture_root/probe"
    cat <<'SECURITY'
if [ "$1" = find-identity ]; then
    if [ "${FIXTURE_IDENTITY:-valid}" = valid ]; then
        printf '%s\n' '  1) CC8B39272E03B790FCB94930A98CC6C4722B235B "Developer ID Application: Fixture (DN263AX69U)"' '     1 valid identities found'
    else printf '%s\n' '     0 valid identities found'; fi
fi
SECURITY
} > "$fixture_root/mock/security"
{
    printf '#!/bin/bash\n'
    cat "$fixture_root/probe"
    printf 'test -f "$signing_private/notary.p8"\n'
} > "$fixture_root/mock/xcrun"
{
    printf '#!/bin/bash\n'
    cat "$fixture_root/probe"
    printf "printf 'synthetic-keychain-password\\n'\n"
} > "$fixture_root/mock/openssl"
cat > "$fixture_root/mock/base64" <<'BASE64'
#!/bin/bash
set -eu
[ "$#" = 1 ] && [ "$1" = -D ]
exec /usr/bin/openssl base64 -d -A
BASE64
chmod +x "$fixture_root/scripts/dev-tools.sh" "$fixture_root/mock/"*
printf 'fail() { printf "%%s\\n" "$*" >&2; exit 1; };\n' > "$fixture_root/fragment"
sed -n '/^# Keep raw credentials/,/^signing_tool=/p' scripts/prepare-signed-distribution.sh | sed '$d' >> "$fixture_root/fragment"
(
    cd "$fixture_root"
    export PATH="$fixture_root/mock:$PATH" RUSTUP_TOOLCHAIN=stable
    export signing_private="$fixture_root/private" signing_keychain="$fixture_root/private/test.keychain"
    export SIGNING_CERTIFICATE_BASE64=Y2VydA== SIGNING_CERTIFICATE_PASSWORD=synthetic-password NOTARY_KEY_BASE64=a2V5
    export NOTARY_KEY_ID=fixture NOTARY_ISSUER_ID=fixture
    export signing_certificate=inherited signing_certificate_password=inherited signing_notary_key=inherited
    export SIGNING_IDENTITY='Developer ID Application: Fixture (DN263AX69U)'
    /bin/bash -eu fragment
    [ ! -e "$signing_private/certificate.p12" ] && [ ! -e "$signing_private/notary.p8" ]
    SIGNING_IDENTITY=cc8b39272e03b790fcb94930a98cc6c4722b235b /bin/bash -eu fragment
    for identity in 'Developer ID Application: Wrong (DN263AX69U)' 'Developer ID Application: Fixture'; do
        if SIGNING_IDENTITY="$identity" /bin/bash -eu fragment > output 2>&1; then exit 1; fi
        /usr/bin/grep -F 'no valid matching signing identity' output >/dev/null
        rm -f "$signing_private/certificate.p12" "$signing_private/notary.p8"
    done
    if FIXTURE_IDENTITY=missing /bin/bash -eu fragment > output 2>&1; then exit 1; fi
    /usr/bin/grep -F 'no valid matching signing identity' output >/dev/null
)
# Check the actual optional argument expansion, including Bash 3.2 nounset semantics.
grep -F '${signing_reference_options[@]+"${signing_reference_options[@]}"}' scripts/prepare-signed-distribution.sh >/dev/null
/bin/bash -eu <<'ARGS'
signing_reference_options=()
set -- ${signing_reference_options[@]+"${signing_reference_options[@]}"}
test "$#" = 0
signing_reference_options=(--runtime-reference 'path with spaces')
set -- ${signing_reference_options[@]+"${signing_reference_options[@]}"}
test "$#" = 2 && test "$2" = 'path with spaces'
ARGS
printf '%s\n' 'Pinned tooling, credential inheritance, identity preflight and Bash arguments: passed'
