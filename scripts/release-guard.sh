#!/bin/bash
# Portable development/release validation. No native SDK or application code needed.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
export RUSTUP_TOOLCHAIN=1.99.0
if command -v cargo >/dev/null 2>&1; then
    guard_cargo=$(command -v cargo)
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    guard_cargo=$HOME/.cargo/bin/cargo
else
    printf '%s\n' 'Release guards require Rust 1.99.0.' >&2
    exit 1
fi
"$guard_cargo" build --locked --manifest-path "$repo/rust/tooling/Cargo.toml" --target-dir "$repo/rust/tooling/target" --no-default-features --bin inkscape-mcp-release-guard >&2
exec "$repo/rust/tooling/target/debug/inkscape-mcp-release-guard" "$@"
