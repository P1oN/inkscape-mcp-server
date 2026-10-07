#!/bin/bash
# Isolated filesystem/process/build fixtures; no installed clients or GUI are touched.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
fixture=$(cd -- "$fixture" && pwd -P)
trap 'rm -rf -- "$fixture"' EXIT
root=$fixture/source
mkdir -p "$root/scripts" "$root/rust/tooling" "$root/.inkscape-mcp-local" "$fixture/bin"
cp "$repo/scripts/cleanup-development.sh" "$repo/scripts/build-local-package.sh" \
    "$repo/scripts/dev-tools.sh" "$root/scripts/"
export FIXTURE_PROCESS_FILE=$fixture/processes
: > "$FIXTURE_PROCESS_FILE"
cat > "$fixture/bin/ps" <<'MOCK'
#!/bin/bash
cat "$FIXTURE_PROCESS_FILE"
MOCK
chmod +x "$fixture/bin/ps"
export PATH="$fixture/bin:$PATH"
private=$root/.inkscape-mcp-local
"$root/scripts/cleanup-development.sh" --apply
for number in 1 2 3 4 5 6 7 8; do
    path=$private/build.00000$number
    mkdir -p "$path/package"
    (cd -- "$path" && find package -print | LC_ALL=C sort) > "$path/.managed-files"
    printf '%s\n' inkscape-mcp-managed-build-v1 > "$path/.managed-build"
    touch -t "2020010${number}0000" "$path/.managed-build"
done
printf '%s\n' inkscape-mcp-setup-v1 "$private/build.000001/package/bin/inkscape-mcp" \
    /engine "$private/build.000003/package/drawings" > "$private/setup.conf"
printf '%s\n' "$private/build.000002/package/bin/inkscape-mcp" > "$FIXTURE_PROCESS_FILE"
touch "$private/build.000004/user-added.svg"
mkdir -p "$private/build.000006/package/drawings"
touch "$private/build.000006/package/drawings/user.svg"
mkdir -p "$private/build.legacy" "$fixture/outside"
touch "$fixture/outside/sentinel"
ln -s "$fixture/outside" "$private/build.aaaaaa"
"$root/scripts/cleanup-development.sh" > "$fixture/preview"
[ -d "$private/build.000005" ]
grep -F 'Would-remove' "$fixture/preview" >/dev/null
"$root/scripts/cleanup-development.sh" --apply
[ ! -e "$private/build.000005" ]
for number in 1 2 3 4 6 7 8; do [ -d "$private/build.00000$number" ]; done
[ -d "$private/build.legacy" ] && [ -f "$fixture/outside/sentinel" ]
for number in 9 10 11 12 13; do
    path=$private/build.$(printf '%06d' "$number")
    mkdir -p "$path/package"
    (cd -- "$path" && find package -print | LC_ALL=C sort) > "$path/.managed-files"
    printf '%s\n' inkscape-mcp-managed-build-v1 > "$path/.managed-build"
done
"$root/scripts/cleanup-development.sh" --apply > "$fixture/recent"
for number in 9 10 11 12 13; do [ -d "$private/build.$(printf '%06d' "$number")" ]; done
[ -f "$private/build.000006/package/drawings/user.svg" ]
printf '%s\n' 'Retention: dry-run, selected/running/workspace/recent/user-added/unmarked/symlink protections passed.'
mkdir -p "$root/rust/target" "$root/rust/tooling/target"
for path in "$root/rust/target" "$root/rust/tooling/target"; do
    printf '%s\n' 'Signature: 8a477f597d28d172789f06886806bc55' > "$path/CACHEDIR.TAG"
done
printf '/compiler/cargo cargo test\n' > "$FIXTURE_PROCESS_FILE"
if "$root/scripts/cleanup-development.sh" --apply --caches > "$fixture/refusal" 2>&1; then exit 1; fi
[ -d "$root/rust/target" ]
printf '%s\n' "$root/rust/target/debug/server" > "$FIXTURE_PROCESS_FILE"
if "$root/scripts/cleanup-development.sh" --apply --caches > "$fixture/refusal" 2>&1; then exit 1; fi
[ -d "$root/rust/target" ]
: > "$FIXTURE_PROCESS_FILE"
mv "$root/rust/tooling" "$root/rust/tooling-real"
ln -s tooling-real "$root/rust/tooling"
if "$root/scripts/cleanup-development.sh" --apply --caches > "$fixture/refusal" 2>&1; then exit 1; fi
[ -d "$root/rust/target" ]
rm "$root/rust/tooling"
mv "$root/rust/tooling-real" "$root/rust/tooling"
"$root/scripts/cleanup-development.sh" --apply --caches
[ ! -e "$root/rust/target" ] && [ ! -e "$root/rust/tooling/target" ]
printf '%s\n' 'Cache cleanup: compiler/process/symlink refusal and idle removal passed.'
# Exercise disposable acceptance output without compiling a runtime.
cat > "$fixture/bin/cargo" <<'MOCK'
#!/bin/bash
if [ "$1" = --version ]; then printf 'cargo 1.99.0 (fixture)\n'; exit; fi
[ "${FIXTURE_BUILD_FAIL:-false}" = false ]
MOCK
chmod +x "$fixture/bin/cargo"
export INKSCAPE_MCP_BUILD_TOOLING_TARGET_DIR=$fixture/tools
mkdir -p "$fixture/tools/debug"
cat > "$fixture/tools/debug/inkscape-mcp-tools" <<'MOCK'
#!/bin/bash
output=${!#}
mkdir -p "$output"
printf 'fixture\n' > "$output/report.json"
[ "${FIXTURE_CHECK_FAIL:-false}" = false ]
MOCK
chmod +x "$fixture/tools/debug/inkscape-mcp-tools"
"$root/scripts/dev-tools.sh" --temporary-output authoring-acceptance
shopt -s nullglob
checks=("$private"/check.*); [ "${#checks[@]}" -eq 0 ]
if FIXTURE_CHECK_FAIL=true "$root/scripts/dev-tools.sh" --temporary-output authoring-acceptance; then exit 1; fi
checks=("$private"/check.*); [ "${#checks[@]}" -eq 1 ]
[ -f "${checks[0]}/result/report.json" ]
for command in native-gui update-acceptance build-package; do
    if "$root/scripts/dev-tools.sh" --temporary-output "$command" > "$fixture/refusal" 2>&1; then exit 1; fi
done
if "$root/scripts/dev-tools.sh" --temporary-output authoring-acceptance --output "$fixture/outside" > "$fixture/refusal" 2>&1; then exit 1; fi
printf '%s\n' 'Temporary checks: successful output removed, failure evidence retained, persistent commands refused.'
# Exercise actual build staging traps with synthetic Cargo and packaging commands.
cat > "$fixture/bin/rustc" <<'MOCK'
#!/bin/bash
printf 'rustc 1.99.0 (fixture)\n'
if [ "${2:-}" = --verbose ]; then
    case "$(uname -s)" in Darwin) suffix=apple-darwin;; *) suffix=unknown-linux-gnu;; esac
    arch=$(uname -m); [ "$arch" != arm64 ] || arch=aarch64
    printf 'host: %s-%s\n' "$arch" "$suffix"
fi
MOCK
for command in gdbus dbus-daemon patchelf; do
    printf '#!/bin/bash\nexit 0\n' > "$fixture/bin/$command"
    chmod +x "$fixture/bin/$command"
done
chmod +x "$fixture/bin/rustc"
export INKSCAPE_MCP_BUILD_GLIB_PREFIX=$fixture/glib
mkdir -p "$fixture/glib/include/glib-2.0/gio"
touch "$fixture/glib/include/glib-2.0/gio/gio.h"
cat > "$root/scripts/dev-tools.sh" <<'MOCK'
#!/bin/bash
while [ "$1" != --output ]; do shift; done
mkdir -p "$2/bin"
[ "${FIXTURE_PACKAGE_FAIL:-false}" = false ]
MOCK
chmod +x "$root/scripts/dev-tools.sh"
before=("$private"/build.*)
if FIXTURE_BUILD_FAIL=true "$root/scripts/build-local-package.sh" > "$fixture/refusal" 2>&1; then exit 1; fi
after=("$private"/build.*); [ "${#before[@]}" -eq "${#after[@]}" ]
if FIXTURE_PACKAGE_FAIL=true "$root/scripts/build-local-package.sh" > "$fixture/refusal" 2>&1; then exit 1; fi
after=("$private"/build.*); [ "${#before[@]}" -eq "${#after[@]}" ]
package=$("$root/scripts/build-local-package.sh")
[ -d "$package" ] && [ -f "${package%/package}/.managed-build" ]
[ ! -e "${package%/package}/package.tar.gz" ]
printf '%s\n' 'Build staging: compiler/package failures cleaned, successful package retained without duplicate archive.'
