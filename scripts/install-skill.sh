#!/bin/bash
# Install only the bundled agent instructions; no runtime or client configuration changes.
set -euo pipefail
export PATH="/usr/bin:/bin${PATH:+:$PATH}"
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fail() { printf '%s\n' "$*" >&2; exit 1; }
client=codex destination= update=false replace=false
while [ "$#" -gt 0 ]; do
    case "$1" in
        --client|--destination)
            [ "$#" -ge 2 ] || fail "Missing value for $1."
            case "$1" in --client) client=$2;; --destination) destination=$2;; esac
            shift 2;;
        --update) update=true; shift;;
        --replace) replace=true; shift;;
        --help)
            printf '%s\n' 'Usage: scripts/install-skill.sh [--client codex|claude] [--destination SKILLS_DIRECTORY]' \
                'Default: ${CODEX_HOME:-$HOME/.codex}/skills or $HOME/.claude/skills.' \
                'Installs inkscape-mcp. --replace archives/replaces all skill files; --update merges customizations.'
            exit 0;;
        *) fail "Unknown option: $1";;
    esac
done
case "$client" in codex|claude) ;; *) fail 'Client must be codex or claude.';; esac
[ "$replace" = false ] || [ "$update" = false ] || fail 'Choose --replace or --update.'
if [ -z "$destination" ]; then
    case "$client" in
        codex) destination=${CODEX_HOME:-${HOME:?HOME is required}/.codex}/skills;;
        claude) destination=${HOME:?HOME is required}/.claude/skills;;
    esac
fi
case "$destination" in ''|*$'\n'*|*$'\r'*) fail 'Invalid skills directory.';; esac
source_dir=$repo/skills/inkscape-mcp
for file in SKILL.md agents/openai.yaml; do
    [ -f "$source_dir/$file" ] && [ ! -L "$source_dir/$file" ] || fail 'Bundled skill is missing or symlinked.'
done
[ ! -L "$source_dir" ] && [ ! -L "$source_dir/agents" ] || fail 'Bundled skill must not be symlinked.'
[ ! -L "$destination" ] || fail 'Skills directory must not be a symlink.'
(umask 077; mkdir -p -- "$destination")
destination=$(cd -- "$destination" && pwd -P)
target=$destination/inkscape-mcp
[ ! -L "$target" ] || fail 'Existing skill must not be a symlink.'
lock=$destination/.inkscape-mcp-install-lock
(umask 077; mkdir -- "$lock") || fail 'Another skill installation is active; retry later.'
stage= backup=
cleanup_update() {
    [ -z "$stage" ] || rm -rf -- "$stage"
    rmdir -- "$lock" 2>/dev/null || true
}
trap cleanup_update EXIT
if [ -e "$target" ]; then
    [ -d "$target" ] && [ ! -L "$target/agents" ] || fail 'Existing skill has a different layout; preserved.'
    [ -z "$(find "$target" -type l -print)" ] || fail 'Existing skill contains symlinks; preserved.'
    if [ "$replace" = true ]; then
        stage=$(mktemp -d "$destination/.inkscape-mcp-update.XXXXXX")
        (umask 077; mkdir -p -- "$stage/agents" "$stage/.inkscape-mcp-upstream/agents";
            cp -- "$source_dir/SKILL.md" "$stage/SKILL.md";
            cp -- "$source_dir/agents/openai.yaml" "$stage/agents/openai.yaml";
            cp -- "$source_dir/SKILL.md" "$stage/.inkscape-mcp-upstream/SKILL.md";
            cp -- "$source_dir/agents/openai.yaml" "$stage/.inkscape-mcp-upstream/agents/openai.yaml";
            printf '%s\n' "$repo" > "$stage/.inkscape-mcp-owner")
        backup_root=$(dirname -- "$destination")/inkscape-mcp-backups
        [ ! -L "$backup_root" ] || fail 'Skill backup directory must not be symlinked.'
        (umask 077; mkdir -p -- "$backup_root")
        backup=$(mktemp -d "$backup_root/skill.XXXXXX")
        rmdir -- "$backup"
        mv -- "$target" "$backup"
        if ! mv -- "$stage" "$target"; then
            mv -- "$backup" "$target"
            fail 'Skill replacement failed; previous version restored.'
        fi
        stage=
        printf 'Replaced skill: %s\nPrevious version archived at %s\n' "$target" "$backup" >&2
        exit 0
    fi
    identical=true
    for file in SKILL.md agents/openai.yaml; do
        [ -f "$target/$file" ] || fail 'Existing skill has a different layout; preserved.'
        cmp -s -- "$source_dir/$file" "$target/$file" || identical=false
    done
    if [ "$identical" = true ]; then
        # Adopt legacy identical installs so future updates can preserve custom edits.
        baseline=$target/.inkscape-mcp-upstream
        [ ! -L "$baseline" ] || fail 'Skill baseline must not be a symlink.'
        if [ ! -e "$baseline" ]; then
            (umask 077; mkdir -p -- "$baseline/agents"; cp -- "$source_dir/SKILL.md" "$baseline/SKILL.md"; cp -- "$source_dir/agents/openai.yaml" "$baseline/agents/openai.yaml")
        fi
        printf 'Skill already installed: %s\n' "$target" >&2
        exit 0
    fi
    [ "$update" = true ] || fail "Existing skill differs; preserved at $target. Use --update for a managed update."
    baseline=$target/.inkscape-mcp-upstream
    for file in SKILL.md agents/openai.yaml; do
        [ -f "$baseline/$file" ] && [ ! -L "$baseline/$file" ] || fail "No upstream baseline; existing edits preserved. Compare with $source_dir manually."
    done
    stage=$(mktemp -d "$destination/.inkscape-mcp-update.XXXXXX")
    cp -R -- "$target/." "$stage/"
    conflicts=false
    for file in SKILL.md agents/openai.yaml; do
        if cmp -s -- "$target/$file" "$baseline/$file"; then
            cp -- "$source_dir/$file" "$stage/$file"
        elif cmp -s -- "$source_dir/$file" "$baseline/$file"; then
            : # only the user changed this file
        else
            status=0
            diff3 -m -- "$target/$file" "$baseline/$file" "$source_dir/$file" > "$stage/$file" || status=$?
            [ "$status" -le 1 ] || fail 'Could not merge skill; existing files preserved.'
            [ "$status" -eq 0 ] || conflicts=true
        fi
    done
    if [ "$conflicts" = true ]; then
        proposal=$(mktemp -d "${TMPDIR:-/tmp}/inkscape-mcp-proposal.XXXXXX")
        cp -R -- "$stage/." "$proposal/"
        fail "Skill merge conflicts; installed files preserved. Review proposed files at $proposal."
    fi
    printf '%s\n' "$repo" > "$stage/.inkscape-mcp-owner"
    cp -- "$source_dir/SKILL.md" "$stage/.inkscape-mcp-upstream/SKILL.md"
    cp -- "$source_dir/agents/openai.yaml" "$stage/.inkscape-mcp-upstream/agents/openai.yaml"
    backup=$(mktemp -d "$destination/.inkscape-mcp-backup.XXXXXX")
    rmdir -- "$backup"
    mv -- "$target" "$backup"
    if ! mv -- "$stage" "$target"; then
        mv -- "$backup" "$target"
        fail 'Skill update failed; previous version restored.'
    fi
    stage=
    printf 'Updated skill: %s\nPrevious version preserved at %s\n' "$target" "$backup" >&2
    exit 0
fi
# Exclusive directory creation protects existing content, including concurrent installs.
(umask 077; mkdir -- "$target") || fail 'Skill destination appeared during installation; preserved.'
complete=false
cleanup() {
    if [ "$complete" = false ]; then
        rm -f -- "$target/SKILL.md" "$target/agents/openai.yaml" "$target/.inkscape-mcp-owner"
        rm -f -- "$target/.inkscape-mcp-upstream/SKILL.md" "$target/.inkscape-mcp-upstream/agents/openai.yaml"
        rmdir -- "$target/.inkscape-mcp-upstream/agents" "$target/.inkscape-mcp-upstream" "$target/agents" "$target" 2>/dev/null || true
    fi
    cleanup_update
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
(umask 077; mkdir -- "$target/agents"; cp -- "$source_dir/SKILL.md" "$target/SKILL.md"; \
    cp -- "$source_dir/agents/openai.yaml" "$target/agents/openai.yaml")
(umask 077; mkdir -p -- "$target/.inkscape-mcp-upstream/agents"; cp -- "$source_dir/SKILL.md" "$target/.inkscape-mcp-upstream/SKILL.md"; cp -- "$source_dir/agents/openai.yaml" "$target/.inkscape-mcp-upstream/agents/openai.yaml")
(umask 077; printf '%s\n' "$repo" > "$target/.inkscape-mcp-owner")
complete=true
printf 'Installed skill: %s\nRestart your client to discover it; invoke with $inkscape-mcp.\n' "$target" >&2
