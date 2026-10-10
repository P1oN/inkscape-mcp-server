#!/bin/bash
# Synthetic GitHub protocol only; never invokes the real gh executable.
set -euo pipefail
root=${SIGNED_FIXTURE_ROOT:?}
mode=${SIGNED_FIXTURE_MODE:?}
printf '%s\n' "$*" >> "$root/calls"
if [ "$1" = api ]; then
    if [ "${2:-}" = --method ]; then
        [ "$3" = PATCH ] && [ "$4" = repos/fixture/repository/releases/42 ] && [ "$5" = -F ] && [ "$6" = draft=false ]
        printf 'publish\n' >> "$root/calls"
        printf '{}\n'
    else
        case "${*: -1}" in
            */releases/tags/*) exit 1;;
            */releases/42)
                printf '%s\n' "$SIGNED_FIXTURE_INVENTORY" | jq --arg mode "$mode" --arg uploaded "$([ ! -e "$root/uploaded" ] || printf yes)" '
                    to_entries | if $mode == "recover-complete" or $uploaded == "yes" then . else .[:2] end |
                    map({name:.key,size:.value.bytes,digest:("sha256:"+.value.sha256)}) |
                    if $mode == "remote-duplicate" then .[-1]=.[0] elif $mode == "recover-corrupt" then .[0].digest="sha256:wrong" elif $mode == "recover-foreign" then .+[{name:"foreign",size:1,digest:"sha256:wrong"}] elif $mode == "recover-duplicate" then .+[.[0]] else . end |
                    {id:42,draft:($mode!="recover-published"),tag_name:"v9.0.0",target_commitish:(if $mode=="recover-source" then "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" else "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" end),prerelease:true,assets:.}'
                ;;
            */releases\?*)
                if [ -e "$root/created" ] && [ "$mode" = missing-created-draft ]; then printf '[]\n'
                elif [ -e "$root/created" ] && [ "$mode" = duplicate-created-draft ]; then printf '[{"id":42,"draft":true,"tag_name":"v9.0.0"},{"id":43,"draft":true,"tag_name":"v9.0.0"}]\n'
                elif [[ "$mode" = recover-* ]] || [ -e "$root/created" ]; then
                    if [ "$mode" = second-page ]; then printf '[]\n'; fi
                    printf '[{"id":42,"draft":true,"tag_name":"v9.0.0"}]\n'
                elif [ "$mode" = draft ]; then printf '[{"tag_name":"v9.0.0"}]\n'
                else printf '[]\n'; fi;;
            *) printf '[]\n';;
        esac
    fi
elif [ "$1 $2" = 'release create' ]; then
    printf 'create\n' >> "$root/calls"; touch "$root/created"
elif [ "$1 $2" = 'release upload' ]; then
    printf 'upload\n' >> "$root/calls"
    [ "$mode" != upload-failure ] || exit 1
    shift 5
    : > "$root/uploaded-names"
    for path in "$@"; do [ "$path" != --clobber ]; basename "$path" >> "$root/uploaded-names"; done
    touch "$root/uploaded"
else exit 99; fi
