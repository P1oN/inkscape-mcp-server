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
package= workspace= inkscape= engine= live= build=false check=false bootstrap=false
sentry= sentry_dsn_file= sentry_environment=
skill_client=
connect_client=
skill_update=false
while [ "$#" -gt 0 ]; do
    case "$1" in
        --package|--workspace|--inkscape|--engine|--live|--sentry|--sentry-dsn-file|--sentry-environment|--install-skill|--connect-client)
            [ "$#" -ge 2 ] || fail "Missing value for $1."
            [ -n "$2" ] || fail "Empty value for $1."
            case "$1" in
                --package) package=$2;; --workspace) workspace=$2;;
                --inkscape) inkscape=$2;; --engine) engine=$2;; --live) live=$2;;
                --sentry) sentry=$2;; --sentry-dsn-file) sentry_dsn_file=$2;;
                --sentry-environment) sentry_environment=$2;;
                --install-skill) skill_client=$2;; --connect-client) connect_client=$2;;
            esac
            shift 2;;
        --local-tools|--build|--bootstrap|--rebuild)
            [ "$build" = false ] || fail 'Choose only one build mode.'
            build=true
            case "$1" in --bootstrap|--rebuild) bootstrap=true;; esac
            shift;;
        --update-skill) skill_update=true; shift;;
        --version)
            manifest=$repo/.inkscape-mcp-local/setup.conf
            [ -f "$manifest" ] && [ ! -L "$manifest" ] || fail 'No configured installation.'
            installed=$(sed -n '2p' "$manifest")
            case "$installed" in /*/bin/inkscape-mcp) ;; *) fail 'Invalid saved package.';; esac
            metadata=${installed%/bin/inkscape-mcp}/libexec/inkscape-mcp/package.json
            [ -f "$metadata" ] || fail 'Installed package metadata missing.'
            sed -n '/"build_info"/,/}/p; /"source_head"/p' "$metadata"
            exit 0;;
        --check) check=true; shift;;
        --help)
            printf '%s\n' 'Usage: ./setup.sh [--package DIRECTORY] [--workspace DIRECTORY]' \
                '                  [--inkscape BINARY_OR_APP] [--live true|false]' \
                '                  [--engine per_call|shell] [--local-tools] [--check]' \
                '                  [--sentry true|false] [--sentry-dsn-file FILE]' \
                '                  [--sentry-environment LABEL] [--install-skill codex|claude]' \
                '--install-skill optionally installs bundled agent guidance without changing MCP client settings.' \
                'Interactive setup asks about optional error reporting (default off).' \
                'DSN input is hidden; use a file rather than putting it in shell history.' \
                'Source setup automatically downloads missing build tools privately and builds on first use.' \
                'Subsequent runs reuse matching source builds and refresh existing clients/skills.' \
                '--local-tools builds with existing developer tools, without automatic tool downloads.' \
                '--package selects a ready package; --check runs doctor before saving.' \
                'Compatibility: --build aliases --local-tools; --bootstrap explicitly repeats automatic provisioning/build.' \
                'Automatic provisioning supports Apple Silicon macOS 15+; Apple developer tools are required.' \
                '--rebuild explicitly rebuilds; --version displays installed revision/build metadata.' \
                '--connect-client codex|claude verifies handshake and atomically updates the client configuration.' \
                '--install-skill replaces bundled guidance, archiving the previous skill; --update-skill is a compatibility alias.' \
                'Existing MCP registrations and installed skills are refreshed automatically; new clients require --connect-client.' \
                'Preserves client preferences and saved MCP settings. Does not launch Inkscape.'
            exit 0;;
        *) fail "Unknown option: $1";;
    esac
done
config_dir=$repo/.inkscape-mcp-local
config=$config_dir/setup.conf
[ ! -L "$config_dir" ] || fail "Configuration directory must not be a symlink."
[ ! -L "$config" ] || fail "Configuration must not be a symlink."
[ ! -e "$config" ] || [ -f "$config" ] || fail "Configuration must be a regular file."
previous_binary=
if [ -f "$config" ]; then
    {
        IFS= read -r previous_version && IFS= read -r previous_binary &&
        IFS= read -r previous_inkscape_dir && IFS= read -r previous_workspace || fail "Incomplete setup configuration."
        IFS= read -r previous_live || [ -n "$previous_live" ] || previous_live=true
        IFS= read -r previous_engine || [ -n "$previous_engine" ] || previous_engine=per_call
        if IFS= read -r extra || [ -n "$extra" ]; then fail "Unexpected setup configuration data."; fi
    } < "$config"
    [ "$previous_version" = inkscape-mcp-setup-v1 ] || fail "Unsupported setup configuration."
    case "$previous_binary" in /*/bin/inkscape-mcp) ;; *) fail "Invalid saved package path.";; esac
    case "$previous_inkscape_dir" in /*) ;; *) fail "Invalid saved Inkscape directory.";; esac
    case "$previous_workspace" in /*) ;; *) fail "Invalid saved workspace.";; esac
    case "$previous_live" in true|false) ;; *) fail "Invalid saved live setting.";; esac
    case "$previous_engine" in per_call|shell) ;; *) fail "Invalid saved engine setting.";; esac
    [ -n "$workspace" ] || workspace=$previous_workspace
    [ -n "$inkscape" ] || inkscape=$previous_inkscape_dir/inkscape
    [ -n "$live" ] || live=$previous_live
    [ -n "$engine" ] || engine=$previous_engine
fi
live=${live:-true}
engine=${engine:-per_call}
case "$engine" in per_call|shell) ;; *) fail "Engine must be per_call or shell.";; esac
case "$live" in true|false) ;; *) fail "Live must be true or false.";; esac
case "$skill_client" in ''|codex|claude) ;; *) fail 'Skill client must be codex or claude.';; esac
case "$connect_client" in ''|codex|claude) ;; *) fail 'Client must be codex or claude.';; esac
[ "$skill_update" = false ] || [ -n "$skill_client" ] || fail '--update-skill requires --install-skill.'
case "$sentry" in ''|true|false) ;; *) fail "Sentry must be true or false.";; esac
[ "$sentry" != false ] || { [ -z "$sentry_dsn_file" ] && [ -z "$sentry_environment" ]; } ||
    fail "Disabled Sentry cannot have DSN or environment options."
[ "$build" = false ] || [ -z "$package" ] || fail 'Build options and --package cannot be combined.'
[ "$build" = false ] || [ -f "$repo/rust/Cargo.toml" ] || fail 'Build options require a source checkout.'
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
# Reuse only a complete configured runtime. Explicit --package always wins.
if [ -z "$package" ] && [ "$build" = false ] && [ -n "$previous_binary" ] &&
    [ -x "$previous_binary" ] && [ -x "${previous_binary%/bin/inkscape-mcp}/bin/inkscape-mcp-client" ] && [ -x "${previous_binary%/bin/inkscape-mcp}/bin/inkscape-mcp-supervisor" ] && [ -x "${previous_binary%/bin/inkscape-mcp}/bin/inkscape-mcp-inx" ] && [ -x "${previous_binary%/bin/inkscape-mcp}/bin/inkscape-mcp-live" ] && [ -f "${previous_binary%/bin/inkscape-mcp}/libexec/inkscape-mcp/package.json" ]; then
    package=${previous_binary%/bin/inkscape-mcp}
    if [ -f "$repo/rust/Cargo.toml" ]; then
        source_revision=
        source_dirty=false
        if [ -e "$repo/.git" ]; then
            source_revision=$(git -C "$repo" rev-parse --verify HEAD 2>/dev/null || true)
            if [ -n "$source_revision" ] && ! git -C "$repo" diff --quiet HEAD -- rust runtime scripts skills setup.sh run-mcp.sh migration/contracts; then
                source_dirty=true
                printf '%s\n' 'Source edits are present; rebuilding automatically.' >&2
            fi
        elif [ -f "$repo/SOURCE_REVISION" ] && [ ! -L "$repo/SOURCE_REVISION" ]; then
            { IFS= read -r marker_version && IFS= read -r source_revision || true; } < "$repo/SOURCE_REVISION"
            [ "${marker_version:-}" = inkscape-mcp-source-v1 ] || source_revision=
        fi
        # Manifest is generated locally with a fixed pretty-printed source_head field.
        # This read uses system text tools, never imports the packaged Python runtime.
        installed_revision=$(sed -n 's/^[ ]*"source_head":[ ]*"\([0-9a-f]\{40\}\)",\{0,1\}[ ]*$/\1/p' "$package/libexec/inkscape-mcp/package.json")
        if [[ "$source_revision" =~ ^[0-9a-f]{40}$ ]] && [[ "$installed_revision" =~ ^[0-9a-f]{40}$ ]]; then
            if [ "$source_revision" != "$installed_revision" ]; then
                printf '%s\n' 'Sources changed since the installed build; rebuilding automatically.' >&2
                package=
            fi
        else
            printf '%s\n' 'Source revisions cannot be compared; rebuilding automatically.' >&2
            package=
        fi
        [ "$source_dirty" = false ] || package=
    fi
fi
if [ -z "$package" ] && [ "$build" = false ] && [ -f "$repo/rust/Cargo.toml" ]; then
    build=true
    bootstrap=true
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
        IFS= read -r telemetry_version && IFS= read -r previous_enabled || fail "Incomplete Sentry configuration."
        IFS= read -r previous_dsn || [ -n "$previous_dsn" ] || previous_dsn=
        IFS= read -r previous_environment || [ -n "$previous_environment" ] || previous_environment=production
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
if [ -z "$sentry" ] && [ -f "$telemetry" ]; then sentry=$previous_enabled; fi
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

if [ "$build" = true ]; then
    # The build recipe contains fixed commands; its stdout contains only one package path.
    builder=$repo/scripts/build-local-package.sh
    [ "$bootstrap" = false ] || builder=$repo/scripts/bootstrap-local-package.sh
    local_tools_only=false
    [ "$bootstrap" = true ] || local_tools_only=true
    package=$(PATH="$inkscape_dir:${PATH:-/usr/bin:/bin}" \
        INKSCAPE_MCP_BUILD_LOCAL_TOOLS_ONLY="$local_tools_only" "$builder")
fi
if [ -z "$package" ]; then
    ask 'Directory of the unpacked Rust package' --package
    package=$answer
fi
single_line "$package"
package=$(cd -- "$package" && pwd -P) || fail "Package directory does not exist."
single_line "$package"
binary=$package/bin/inkscape-mcp
[ -x "$binary" ] && [ -x "$package/bin/inkscape-mcp-client" ] && [ -x "$package/bin/inkscape-mcp-supervisor" ] && [ -x "$package/bin/inkscape-mcp-inx" ] && [ -x "$package/bin/inkscape-mcp-live" ] && [ -f "$package/libexec/inkscape-mcp/package.json" ] || \
    fail "Expected a complete package containing bin/inkscape-mcp, bin/inkscape-mcp-client, bin/inkscape-mcp-supervisor, bin/inkscape-mcp-inx, bin/inkscape-mcp-live and libexec/inkscape-mcp/package.json."
if [ "$check" = true ]; then
    printf '%s\n' 'Checking the package and Inkscape (no GUI launch)...' >&2
    PATH="$inkscape_dir:${PATH:-/usr/bin:/bin}" INKSCAPE_MCP_WORKSPACE_ROOTS="$workspace" \
        "$binary" --doctor >&2 || fail "Doctor failed; configuration was not saved."
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
if [ -n "$skill_client" ]; then
    skill_options=(--client "$skill_client" --replace)
    "$repo/scripts/install-skill.sh" "${skill_options[@]}" ||
        fail 'MCP settings were saved, but skill installation failed. See the message above and retry scripts/install-skill.sh.'
fi

if [ -n "$connect_client" ]; then
    "$repo/scripts/mcp-client.sh" --client "$connect_client" upgrade ||
        fail 'MCP settings were saved, but client connection failed. Retry scripts/mcp-client.sh --client '"$connect_client"' upgrade.'
fi
# Refresh existing registrations and installed guidance even on an ordinary rerun.
for existing_client in codex claude; do
    [ "$existing_client" != "$connect_client" ] || continue
    case "$existing_client" in
        codex) existing_config=${CODEX_HOME:-${HOME:?HOME is required}/.codex}/config.toml;;
        claude) existing_config=${HOME:?HOME is required}/.claude.json;;
    esac
    [ -e "$existing_config" ] || [ -L "$existing_config" ] || continue
    "$package/bin/inkscape-mcp-client" --repo "$repo" --client "$existing_client" upgrade-existing ||
        fail "Could not update the existing $existing_client registration."
done
for existing_client in codex claude; do
    [ "$existing_client" != "$skill_client" ] || continue
    case "$existing_client" in
        codex) existing_skill=${CODEX_HOME:-${HOME:?HOME is required}/.codex}/skills/inkscape-mcp;;
        claude) existing_skill=${HOME:?HOME is required}/.claude/skills/inkscape-mcp;;
    esac
    if [ -e "$existing_skill" ] || [ -L "$existing_skill" ]; then
        "$repo/scripts/install-skill.sh" --client "$existing_client" --replace ||
            fail "MCP settings were saved, but the existing $existing_client skill refresh failed. Retry \"$repo/scripts/install-skill.sh\" --client $existing_client --replace."
    fi
done
