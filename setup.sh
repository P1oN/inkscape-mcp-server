#!/bin/bash
# Build/configure a local Rust package without editing environment variables.
set -euo pipefail
export PATH="/usr/bin:/bin${PATH:+:$PATH}"
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
fail() { printf '%s\n' "$*" >&2; exit 1; }
ask() {
    [ -t 0 ] || fail "Missing $1; pass $2 for noninteractive setup."
    printf '%s: ' "$1" >&2
    IFS= read -r answer || fail "Setup cancelled."
}
single_line() {
    case "$1" in *$'\n'*|*$'\r'*) fail "Paths cannot contain newline characters.";; esac
}
package= workspace= inkscape= engine=per_call live=true build=false check=false bootstrap=false
sentry= sentry_dsn_file= sentry_environment=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --package|--workspace|--inkscape|--engine|--live|--sentry|--sentry-dsn-file|--sentry-environment)
            [ "$#" -ge 2 ] || fail "Missing value for $1."
            case "$1" in
                --package) package=$2;; --workspace) workspace=$2;;
                --inkscape) inkscape=$2;; --engine) engine=$2;; --live) live=$2;;
                --sentry) sentry=$2;; --sentry-dsn-file) sentry_dsn_file=$2;;
                --sentry-environment) sentry_environment=$2;;
            esac
            shift 2;;
        --build) build=true; shift;;
        --check) check=true; shift;;
        --bootstrap) bootstrap=true; build=true; shift;;
        --help)
            printf '%s\n' 'Usage: ./setup.sh [--package DIRECTORY] [--workspace DIRECTORY]' \
                '                  [--inkscape BINARY_OR_APP] [--live true|false]' \
                '                  [--engine per_call|shell] [--build|--bootstrap] [--check]' \
                '                  [--sentry true|false] [--sentry-dsn-file FILE]' \
                '                  [--sentry-environment LABEL]' \
                'Interactive setup asks about optional error reporting (default off).' \
                'DSN input is hidden; use a file rather than putting it in shell history.' \
                'Saves settings without launching the server, Python, libraries or Inkscape.' \
                '--check runs doctor; --build builds with existing tools; --bootstrap downloads missing tools privately.' \
                'Source builds need native developer prerequisites; archives need only Inkscape.' \
                'Does not launch Inkscape or change MCP client settings.'
            exit 0;;
        *) fail "Unknown option: $1";;
    esac
done
case "$engine" in per_call|shell) ;; *) fail "Engine must be per_call or shell.";; esac
case "$live" in true|false) ;; *) fail "Live must be true or false.";; esac
[ "$build" = false ] || [ -z "$package" ] || fail '--build and --package cannot be combined.'
[ "$build" = false ] || [ -f "$repo/rust/Cargo.toml" ] || fail '--build requires a source checkout.'
if [ -z "$package" ] && [ -f "$repo/libexec/inkscape-mcp/package.json" ]; then
    package=$repo
fi
if [ -z "$workspace" ]; then
    ask 'Existing directory of SVG files the server may access' --workspace
    workspace=$answer
fi
single_line "$workspace"
workspace=$(cd -- "$workspace" && pwd -P) || fail "Workspace directory does not exist."
single_line "$workspace"
case "$workspace" in *:*) fail "A workspace path cannot contain the POSIX path-list separator (:).";; esac
if [ -z "$inkscape" ]; then
    if [ -x /Applications/Inkscape.app/Contents/MacOS/inkscape ]; then
        inkscape=/Applications/Inkscape.app/Contents/MacOS/inkscape
    else
        inkscape=$(command -v inkscape || true)
    fi
fi
if [ -z "$inkscape" ]; then
    ask 'Path to the Inkscape executable or .app' --inkscape
    inkscape=$answer
fi
case "$inkscape" in *.app) inkscape=$inkscape/Contents/MacOS/inkscape;; esac
single_line "$inkscape"
[ -x "$inkscape" ] && [ "$(basename -- "$inkscape")" = inkscape ] || \
    fail "Inkscape must be an executable named inkscape."
inkscape_dir=$(cd -- "$(dirname -- "$inkscape")" && pwd -P)
single_line "$inkscape_dir"
case "$inkscape_dir" in *:*) fail "The Inkscape directory cannot contain (:).";; esac
# Reuse the configured package; rerunning setup must not create a new runtime path.
previous_config=$repo/.inkscape-mcp-local/setup.conf
if [ -z "$package" ] && [ "$build" = false ] && [ -f "$previous_config" ] &&
    [ ! -L "$previous_config" ] && [ ! -L "$repo/.inkscape-mcp-local" ]; then
    {
        IFS= read -r previous_version && IFS= read -r previous_binary || true
    } < "$previous_config"
    if [ "${previous_version:-}" = inkscape-mcp-setup-v1 ]; then
        case "${previous_binary:-}" in
            /*/bin/inkscape-mcp)
                if [ -x "$previous_binary" ]; then
                    package=${previous_binary%/bin/inkscape-mcp}
                fi;;
        esac
    fi
fi
if [ "$build" = true ]; then
    # The build recipe contains fixed commands; its stdout contains only one package path.
    builder=$repo/scripts/build-local-package.sh
    [ "$bootstrap" = false ] || builder=$repo/scripts/bootstrap-local-package.sh
    package=$(PATH="$inkscape_dir:${PATH:-/usr/bin:/bin}" "$builder")
fi
if [ -z "$package" ]; then
    ask 'Directory of the unpacked Rust package' --package
    package=$answer
fi
single_line "$package"
package=$(cd -- "$package" && pwd -P) || fail "Package directory does not exist."
single_line "$package"
binary=$package/bin/inkscape-mcp
[ -x "$binary" ] && [ -f "$package/libexec/inkscape-mcp/package.json" ] || \
    fail "Expected a complete package containing bin/inkscape-mcp and libexec/inkscape-mcp/package.json."
if [ "$check" = true ]; then
    printf '%s\n' 'Checking the package and Inkscape (no GUI launch)...' >&2
    PATH="$inkscape_dir:${PATH:-/usr/bin:/bin}" INKSCAPE_MCP_WORKSPACE_ROOTS="$workspace" \
        "$binary" --doctor >&2 || fail "Doctor failed; configuration was not saved."
fi
config_dir=$repo/.inkscape-mcp-local
[ ! -L "$config_dir" ] || fail "Configuration directory must not be a symlink."
(umask 077; mkdir -p -- "$config_dir")
[ -d "$config_dir" ] || fail "Configuration directory is unavailable."
# Also protect unpacked packages placed inside a Git checkout without a root ignore rule.
local_ignore=$config_dir/.gitignore
[ ! -L "$local_ignore" ] || fail "Local ignore file must not be a symlink."
if [ ! -e "$local_ignore" ]; then
    (umask 077; set -o noclobber; printf '*\n' > "$local_ignore") || fail "Could not protect local settings from Git."
fi
[ -f "$local_ignore" ] && [ "$(cat -- "$local_ignore")" = '*' ] || fail "Local settings ignore file must contain only *."

config=$config_dir/setup.conf
[ ! -L "$config" ] || fail "Configuration must not be a symlink."
# Telemetry is a separate data-only file, never sourced or embedded in a package.
telemetry=$config_dir/sentry.conf
[ ! -L "$telemetry" ] || fail "Sentry configuration must not be a symlink."
[ ! -e "$telemetry" ] || [ -f "$telemetry" ] || fail "Sentry configuration must be a regular file."
previous_dsn= previous_environment=production previous_enabled=false
if [ -f "$telemetry" ]; then
    {
        IFS= read -r telemetry_version && IFS= read -r previous_enabled &&
        IFS= read -r previous_dsn && IFS= read -r previous_environment ||
            fail "Incomplete Sentry configuration."
        if IFS= read -r extra || [ -n "$extra" ]; then fail "Unexpected Sentry configuration data."; fi
    } < "$telemetry"
    [ "$telemetry_version" = inkscape-mcp-sentry-v1 ] || fail "Unsupported Sentry configuration."
fi
if [ -z "$sentry" ] && { [ -n "$sentry_dsn_file" ] || [ -n "$sentry_environment" ]; }; then
    sentry=true
fi
if [ -z "$sentry" ] && [ -t 0 ]; then
    printf '%s\n' 'Optional error reporting sends technical failures, not drawings or tool arguments.' >&2
    ask "Enable Sentry error reporting? [y/N; blank keeps existing setting]" '--sentry true|false'
    case "$answer" in y|Y|yes|YES) sentry=true;; n|N|no|NO) sentry=false;; '') sentry=$previous_enabled;; *) fail "Answer yes or no.";; esac
fi
case "$sentry" in ''|true|false) ;; *) fail "Sentry must be true or false.";; esac
[ "$sentry" != false ] || { [ -z "$sentry_dsn_file" ] && [ -z "$sentry_environment" ]; } ||
    fail "Disabled Sentry cannot have DSN or environment options."
sentry_dsn=$previous_dsn
if [ "$sentry" = true ]; then
    if [ -n "$sentry_dsn_file" ]; then
        [ -f "$sentry_dsn_file" ] && [ ! -L "$sentry_dsn_file" ] || fail "DSN input must be a regular, non-symlink file."
        {
            IFS= read -r sentry_dsn || [ -n "$sentry_dsn" ] || fail "Empty DSN file."
            if IFS= read -r extra || [ -n "$extra" ]; then fail "DSN file must contain exactly one line."; fi
        } < "$sentry_dsn_file"
    elif [ -t 0 ]; then
        printf 'Sentry DSN (hidden; blank keeps saved value): ' >&2
        IFS= read -r -s answer || fail "Setup cancelled."
        printf '\n' >&2
        [ -z "$answer" ] || sentry_dsn=$answer
    fi
    [[ "$sentry_dsn" =~ ^https://[A-Za-z0-9]+@([A-Za-z0-9-]+\.)+[A-Za-z0-9-]+(:[0-9]+)?/[0-9]+$ ]] ||
        fail "Invalid Sentry DSN; expected an HTTPS ingestion DSN (not an API token)."
    [ "${#sentry_dsn}" -le 2048 ] || fail "Sentry DSN exceeds length limit."
    if [ -z "$sentry_environment" ]; then
        sentry_environment=$previous_environment
        if [ -t 0 ]; then
            ask "Sentry environment label [$previous_environment; e.g. wife]" --sentry-environment
            [ -z "$answer" ] || sentry_environment=$answer
        fi
    fi
    [[ "$sentry_environment" =~ ^[A-Za-z0-9_.-]{1,64}$ ]] || fail "Environment label must be 1–64 letters, digits, dots, underscores or hyphens."
elif [ "$sentry" = false ]; then
    sentry_dsn= sentry_environment=production
fi
temporary=$(mktemp "$config_dir/setup.XXXXXX")
telemetry_temporary=
trap 'rm -f -- "$temporary"; [ -z "$telemetry_temporary" ] || rm -f -- "$telemetry_temporary"' EXIT
if [ -n "$sentry" ]; then
    telemetry_temporary=$(mktemp "$config_dir/sentry.XXXXXX")
    printf '%s\n' 'inkscape-mcp-sentry-v1' "$sentry" "$sentry_dsn" "$sentry_environment" > "$telemetry_temporary"
    mv -f -- "$telemetry_temporary" "$telemetry"
    printf 'Saved private Sentry settings (enabled=%s); DSN not displayed.\n' "$sentry" >&2
fi
printf '%s\n' 'inkscape-mcp-setup-v1' "$binary" "$inkscape_dir" "$workspace" "$live" "$engine" > "$temporary"
mv -f -- "$temporary" "$config"
printf 'Saved settings in %s\nMCP command: %s/run-mcp.sh\n' "$config" "$repo" >&2
