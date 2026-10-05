#!/bin/bash
# Development-only Rust CLI. Never copied to a ready runtime package or exposed through MCP.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd -- "$repo"
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
exec "$target_dir/debug/inkscape-mcp-tools" "$@"
