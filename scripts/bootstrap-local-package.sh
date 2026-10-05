#!/bin/bash
# macOS Apple Silicon source build with disposable, private development tools.
set -euo pipefail
export PATH="/usr/bin:/bin${PATH:+:$PATH}"
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fail() { printf '%s\n' "$*" >&2; exit 1; }
case "$repo" in *:*|*$'\n'*|*$'\r'*) fail 'Source directory must not contain a colon or newline.';; esac
fresh=false
case "${1:-}" in
    '') ;;
    --fresh-tools) fresh=true; shift;;
    --help) printf '%s\n' 'Usage: scripts/bootstrap-local-package.sh [--fresh-tools]' \
        'Build on macOS 15+ Apple Silicon. Download missing tools privately; delete build tools on exit.' \
        'Existing installations remain untouched. Build and runtime use native tools.'; exit 0;;
    *) fail 'Unexpected bootstrap option.';;
esac
[ "$#" -eq 0 ] || fail 'Unexpected bootstrap arguments.'
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || fail 'Automatic setup supports Apple Silicon macOS only. Use setup.sh --local-tools with installed build prerequisites on other targets.'
major=$(/usr/bin/sw_vers -productVersion); major=${major%%.*}
[ "$major" -ge 15 ] || fail 'Pinned native build inputs require macOS 15 or newer.'
if ! sdk=$(/usr/bin/xcrun --show-sdk-path 2>/dev/null); then
    printf '%s\n' 'Apple Command Line Tools are required. Complete the Apple installation dialog, then rerun ./setup.sh.' >&2
    /usr/bin/xcode-select --install >&2 || true
    exit 1
fi
[ -f "$sdk/usr/lib/libxml2.tbd" ] || fail 'Apple SDK is missing libxml2.'
private=$repo/.inkscape-mcp-local
[ ! -L "$private" ] || fail 'Private build directory must not be a symlink.'
(umask 077; mkdir -p -- "$private")
bootstrap_root=$(mktemp -d "$private/bootstrap.XXXXXX")
cleanup() {
    # Only our mktemp directory is removed; never uninstall shared tools.
    case "$bootstrap_root" in "$private"/bootstrap.*) rm -rf -- "$bootstrap_root";; esac
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
printf 'Temporary build tools: %s\n' "$bootstrap_root" >&2
download() {
    /usr/bin/curl --proto '=https' --proto-redir '=https' --tlsv1.2 --fail --location \
        --silent --show-error --connect-timeout 30 --max-time 600 "$1" -o "$2"
    actual=$(/usr/bin/shasum -a 256 "$2"); actual=${actual%% *}
    [ "$actual" = "$3" ] || fail 'Downloaded build tool checksum differs.'
}
cd -- "$repo"
# Reuse an exact native toolchain; never install a toolchain into a user's rustup home.
cargo_candidate=$(command -v cargo || true)
if [ -z "$cargo_candidate" ] && [ -x "$HOME/.cargo/bin/cargo" ]; then cargo_candidate=$HOME/.cargo/bin/cargo; fi
native_rust=false
if [ "$fresh" = false ] && [ -n "$cargo_candidate" ]; then
    rustc_candidate=$(dirname -- "$cargo_candidate")/rustc
    if "$cargo_candidate" --version 2>/dev/null | /usr/bin/grep -q '^cargo 1.99.0 ' &&
        [ -x "$rustc_candidate" ] &&
        "$rustc_candidate" --version 2>/dev/null | /usr/bin/grep -q '^rustc 1.99.0 ' &&
        "$rustc_candidate" --version --verbose 2>/dev/null | /usr/bin/grep -Fx 'host: aarch64-apple-darwin' >/dev/null; then
        native_rust=true
    fi
fi
if [ "$native_rust" = true ]; then
    export PATH="$(dirname -- "$cargo_candidate"):$PATH"
else
    download https://static.rust-lang.org/rustup/archive/1.28.2/aarch64-apple-darwin/rustup-init \
        "$bootstrap_root/rustup-init" 20ef5516c31b1ac2290084199ba77dbbcaa1406c45c1d978ca68558ef5964ef5
    chmod 700 "$bootstrap_root/rustup-init"
    export CARGO_HOME="$bootstrap_root/cargo" RUSTUP_HOME="$bootstrap_root/rustup"
    export RUSTUP_INIT_SKIP_PATH_CHECK=yes
    "$bootstrap_root/rustup-init" -y --no-modify-path --profile minimal --default-toolchain 1.99.0 >&2
    export PATH="$CARGO_HOME/bin:$PATH"
fi
export RUSTUP_TOOLCHAIN=1.99.0
# Compile only the standalone build CLI before downloading native inputs.
export INKSCAPE_MCP_BUILD_TOOLING_TARGET_DIR="$bootstrap_root/tooling-target"
"$repo/scripts/dev-tools.sh" bootstrap-native --lock rust/package/bootstrap-native-macos-arm64.json \
    --output "$bootstrap_root/native" >&2
export INKSCAPE_MCP_BUILD_NATIVE_ROOT="$bootstrap_root/native"
export INKSCAPE_MCP_BUILD_GLIB_PREFIX="$bootstrap_root/native/glib/2.90.0"
export INKSCAPE_MCP_BUILD_TARGET_DIR="$bootstrap_root/target"
export PATH="$bootstrap_root/native/glib/2.90.0/bin:$bootstrap_root/native/dbus/1.16.2_1/bin:$PATH"
package=$("$repo/scripts/build-local-package.sh")
printf '%s\n' 'Local package built. Removing temporary build tools.' >&2
printf '%s\n' "$package"
