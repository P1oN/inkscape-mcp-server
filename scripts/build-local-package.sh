#!/bin/bash
# Build a private native package from this checkout; stdout is the resulting path only.
set -euo pipefail
export PATH="/usr/bin:/bin${PATH:+:$PATH}"
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fail() { printf '%s\n' "$*" >&2; exit 1; }
if [ "$#" -gt 0 ]; then
    if [ "$#" -eq 1 ] && [ "$1" = --help ]; then
        printf '%s\n' 'Usage: scripts/build-local-package.sh' \
            'Builds a new private native package and prints its absolute directory.' \
            'Requires native Rust/SDK/GLib/D-Bus build prerequisites and pinned helper Python or uv.'
        exit 0
    fi
    fail 'Unexpected build argument; use setup.sh for configuration options.'
fi
case "$(uname -s)" in
    Darwin) platform=macos;; Linux) platform=linux;;
    *) fail 'Source packaging currently supports native macOS and GNU/Linux. Windows is pending.';;
esac
case "$(uname -m)" in arm64|aarch64|x86_64) ;; *) fail 'Unsupported native architecture.';; esac
architecture=$(uname -m)
case "$architecture" in arm64) architecture=aarch64;; esac
if [ "$platform" = macos ]; then
    native_target=$architecture-apple-darwin
else
    native_target=$architecture-unknown-linux-gnu
fi
if command -v cargo >/dev/null 2>&1; then
    cargo=$(command -v cargo)
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    cargo=$HOME/.cargo/bin/cargo
else
    fail 'Source builds need Rust 1.99.0. Install its toolchain, or use setup.sh --package with a ready archive (no compiler needed).'
fi
case "$("$cargo" --version)" in 'cargo 1.99.0 '*) ;; *) fail 'Source builds require the pinned Rust 1.99.0 toolchain.';; esac
export PATH="$(dirname -- "$cargo"):${PATH:-/usr/bin:/bin}"
case "$(rustc --version)" in 'rustc 1.99.0 '*) ;; *) fail 'Source builds require rustc 1.99.0.';; esac
rustc --version --verbose | /usr/bin/grep -Fx "host: $native_target" >/dev/null || \
    fail 'Rust compiler host differs from the native package target; cross-compilation is not supported.'
if [ "$platform" = macos ]; then
    sdk=$(/usr/bin/xcrun --show-sdk-path) || fail 'Source builds need the Apple command-line developer tools.'
    [ -f "$sdk/usr/lib/libxml2.tbd" ] || fail 'The Apple SDK is missing libxml2.'
    export LIBXML2="$sdk/usr/lib/libxml2.tbd"
    headers=false
    for prefix in /opt/homebrew /usr/local; do
        if [ -f "$prefix/include/glib-2.0/gio/gio.h" ]; then headers=true; fi
        if [ -d "$prefix/bin" ]; then export PATH="$PATH:$prefix/bin"; fi
    done
    [ "$headers" = true ] || fail 'Source packaging needs GLib development headers. A ready archive avoids this build dependency.'
else
    [ "$(getconf GNU_LIBC_VERSION)" != '' ] || fail 'Source packaging requires GNU/glibc Linux.'
    command -v patchelf >/dev/null 2>&1 || fail 'Source packaging needs patchelf.'
fi
for dependency in gdbus dbus-daemon; do
    command -v "$dependency" >/dev/null 2>&1 || fail "Source packaging needs $dependency. A ready archive includes it."
done
cd -- "$repo"
private=$repo/.inkscape-mcp-local
[ ! -L "$private" ] || fail 'Private build directory must not be a symlink.'
(umask 077; mkdir -p -- "$private")
build_root=$(mktemp -d "$private/build.XXXXXX")
printf 'Preparing native source build in %s\n' "$build_root" >&2
# Reuse only a runtime whose exact version and six wheel versions match the pinned recipe.
check='import sys,platform,importlib.metadata as m; from pathlib import Path; assert sys.version_info[:3] == (3,12,14); cpu=platform.machine(); assert {"arm64":"aarch64"}.get(cpu,cpu)==sys.argv[1]; pairs=[line.strip().split("==") for line in Path("rust/package/helper-requirements.txt").read_text().splitlines() if line.strip() and not line.lstrip().startswith("#")]; assert all(m.version(name)==version for name,version in pairs)'
python=
for candidate in "$repo/.packaging-venv/bin/python" "$repo/.venv/bin/python"; do
    if [ -x "$candidate" ] && "$candidate" -I -c "$check" "$architecture" >/dev/null 2>&1; then
        python=$candidate
        break
    fi
done
if [ -z "$python" ]; then
    # uv manages Python/wheels locally. Never install a system package manager or alter shell profiles.
    if command -v uv >/dev/null 2>&1; then
        uv=$(command -v uv)
    elif [ -x "$HOME/.cargo/bin/uv" ]; then
        uv=$HOME/.cargo/bin/uv
    else
        fail 'Source packaging needs uv or an existing pinned Python 3.12.14 helper environment. A ready archive needs neither.'
    fi
    "$uv" python install 3.12.14 >&2
    "$uv" venv --managed-python --python 3.12.14 "$build_root/helper-build" >&2
    python=$build_root/helper-build/bin/python
    "$uv" pip install --python "$python" --only-binary :all: -r rust/package/helper-requirements.txt >&2
    "$python" -I -c "$check" "$architecture" || fail 'The prepared Python helper runtime differs from the pinned native recipe.'
fi
printf '%s\n' 'Building the locked Rust server and private helper/native package...' >&2
"$cargo" build --locked --release --manifest-path rust/Cargo.toml \
    --target "$native_target" --target-dir "$repo/rust/target" >&2
"$python" scripts/migration_build_posix_package.py --output "$build_root/package" \
    --archive "$build_root/package.tar.gz" \
    --binary "$repo/rust/target/$native_target/release/inkscape-mcp-rust" >&2
printf '%s\n' "$build_root/package"
