#!/bin/bash
# Only MCP JSON-RPC reaches stdout. Configuration is data, never shell code.
set -euo pipefail
export PATH="/usr/bin:/bin${PATH:+:$PATH}"
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
fail() { printf '%s\n' "$*" >&2; exit 1; }
config=$repo/.inkscape-mcp-local/setup.conf
[ -f "$config" ] && [ ! -L "$config" ] && [ ! -L "$repo/.inkscape-mcp-local" ] || \
    fail "Run $repo/setup.sh first."
{
    IFS= read -r version && IFS= read -r binary && IFS= read -r inkscape_dir &&
    IFS= read -r workspace && IFS= read -r live && IFS= read -r engine ||
        fail "Incomplete setup configuration; run setup.sh again."
    if IFS= read -r extra || [ -n "$extra" ]; then
        fail "Unexpected setup configuration data; run setup.sh again."
    fi
} < "$config"
[ "$version" = inkscape-mcp-setup-v1 ] || fail "Unsupported setup configuration."
case "$binary" in /*/bin/inkscape-mcp) ;; *) fail "Invalid package executable path.";; esac
case "$workspace" in /*) ;; *) fail "Workspace must be absolute.";; esac
case "$workspace" in *:*) fail "Invalid workspace path-list separator.";; esac
case "$inkscape_dir" in /*) ;; *) fail "Inkscape directory must be absolute.";; esac
case "$inkscape_dir" in *:*) fail "Invalid Inkscape directory path-list separator.";; esac
case "$live" in true|false) ;; *) fail "Invalid live setting.";; esac
case "$engine" in per_call|shell) ;; *) fail "Invalid engine setting.";; esac
[ -x "$binary" ] && [ -d "$workspace" ] && [ -x "$inkscape_dir/inkscape" ] || \
    fail "Configured package, workspace or Inkscape is missing; run setup.sh again."
export PATH="$inkscape_dir:${PATH:-/usr/bin:/bin}"
export INKSCAPE_MCP_WORKSPACE_ROOTS="$workspace"
export INKSCAPE_MCP_LIVE_ENABLED="$live"
export INKSCAPE_MCP_ENGINE_MODE="$engine"
telemetry=$repo/.inkscape-mcp-local/sentry.conf
[ ! -L "$telemetry" ] || fail "Sentry configuration must not be a symlink."
if [ -e "$telemetry" ]; then
    [ -f "$telemetry" ] && [ -O "$telemetry" ] || fail "Sentry configuration must be an owned regular file."
    {
        IFS= read -r telemetry_version && IFS= read -r sentry_enabled &&
        IFS= read -r sentry_dsn && IFS= read -r sentry_environment || fail "Incomplete Sentry configuration."
        if IFS= read -r extra || [ -n "$extra" ]; then fail "Unexpected Sentry configuration data."; fi
    } < "$telemetry"
    [ "$telemetry_version" = inkscape-mcp-sentry-v1 ] || fail "Unsupported Sentry configuration."
    case "$sentry_enabled" in
        true)
            [[ "$sentry_dsn" =~ ^https://[A-Za-z0-9]+@([A-Za-z0-9-]+\.)+[A-Za-z0-9-]+(:[0-9]+)?/[0-9]+$ ]] &&
                [ "${#sentry_dsn}" -le 2048 ] || fail "Invalid saved Sentry DSN."
            [[ "$sentry_environment" =~ ^[A-Za-z0-9_.-]{1,64}$ ]] || fail "Invalid saved Sentry environment."
            export SENTRY_DSN="$sentry_dsn" SENTRY_ENVIRONMENT="$sentry_environment";;
        false)
            [ -z "$sentry_dsn" ] || fail "Unexpected DSN in disabled Sentry configuration."
            unset SENTRY_DSN
            export SENTRY_TRACES_SAMPLE_RATE=0;;
        *) fail "Invalid saved Sentry setting.";;
    esac
fi
exec "$binary" "$@"
