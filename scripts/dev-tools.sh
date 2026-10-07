#!/bin/bash
# Development-only Rust CLI. Never copied to a ready runtime package or exposed through MCP.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd -- "$repo"
temporary_output=false
if [ "${1:-}" = --temporary-output ]; then
    temporary_output=true
    shift
    # GUI phases and update/installer profiles have deliberate persistent ownership.
    case "${1:-}" in
        package-acceptance|doctor-acceptance|notices-acceptance|launcher-acceptance|\
        socket-acceptance|security-acceptance|frame-acceptance|startup-acceptance|\
        responsiveness-acceptance|defects-acceptance|diagnostic-acceptance|\
        compare-acceptance|special-file-acceptance|renderer-acceptance|\
        engine-routes-acceptance|authoring-acceptance|live-workflow-acceptance) ;;
        *) printf '%s\n' '--temporary-output requires a supported automated acceptance command.' >&2; exit 1;;
    esac
    for option in "$@"; do
        case "$option" in --output|--output=*) printf '%s\n' '--temporary-output cannot be combined with --output.' >&2; exit 1;; esac
    done
fi
if [ "$(uname -s)" = Darwin ] && [ -z "${LIBXML2:-}" ]; then
    export LIBXML2="$(xcrun --show-sdk-path)/usr/lib/libxml2.tbd"
fi
if command -v cargo >/dev/null 2>&1; then
    cargo=$(command -v cargo)
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    cargo=$HOME/.cargo/bin/cargo
else
    printf '%s\n' 'Development tools require Rust 1.99.0.' >&2
    exit 1
fi
case "$("$cargo" --version)" in 'cargo 1.99.0 '*) ;; *) printf '%s\n' 'Development tools require Rust 1.99.0.' >&2; exit 1;; esac
options=(--locked)
if [ "${INKSCAPE_MCP_BUILD_LOCAL_TOOLS_ONLY:-false}" = true ]; then options+=(--offline); fi
target_dir=${INKSCAPE_MCP_BUILD_TOOLING_TARGET_DIR:-$repo/rust/tooling/target}
"$cargo" build "${options[@]}" --manifest-path "$repo/rust/tooling/Cargo.toml" --target-dir "$target_dir" >&2
if [ "$temporary_output" = false ]; then
    exec "$target_dir/debug/inkscape-mcp-tools" "$@"
fi
private=$repo/.inkscape-mcp-local
[ ! -L "$private" ] || { printf '%s\n' 'Private build directory must not be a symlink.' >&2; exit 1; }
(umask 077; mkdir -p -- "$private")
output=$(mktemp -d "$private/check.XXXXXX")
passed=false
cleanup_check() {
    if [ "$passed" = true ] && [ ! -L "$private" ] && [ ! -L "$output" ]; then
        rm -rf -- "$output"
    else
        printf 'Acceptance evidence retained in %s\n' "$output" >&2
    fi
}
trap cleanup_check EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
"$target_dir/debug/inkscape-mcp-tools" "$@" --output "$output/result"
passed=true
