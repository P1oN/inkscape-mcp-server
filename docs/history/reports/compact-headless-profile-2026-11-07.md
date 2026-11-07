# Compact headless profile pilot

The compact profile from [#49](https://github.com/P1oN/inkscape-mcp-server/issues/49)
selects ten tools from the existing headless short contracts. Full and compact discovery
both expose typed Boolean Difference with raw Actions disabled. They share the
initialization text, schemas of the common tools, approval gate and transaction kernel.
The pilot build included the prototype Difference preservation fix. The production
port reuses the geometry preservation and verification already merged in #52 and #53.

## Matched client runs

The runs used Codex CLI 0.157.0, model alias gpt-6-sol at low reasoning effort,
Inkscape 1.4.3 in per-call mode and macOS 27.0 on arm64. The client did not report
the provider model snapshot. Both profiles ran the same brief on fresh copies of the
same SVG. The first pair ran full then compact. The second reversed the order.
Each CLI process used an ephemeral session. Provider caches were uncontrolled.

| Pair | Profile | Input tokens | Cached input | Uncached input | Output tokens | MCP calls | Wall time | Completed |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 1 | full/short | 302,024 | 286,976 | 15,048 | 855 | 10 | 43.48 s | yes |
| 1 | compact | 299,017 | 284,672 | 14,345 | 898 | 10 | 35.90 s | yes |
| 2 | compact | 300,835 | 272,128 | 28,707 | 907 | 10 | 44.28 s | yes |
| 2 | full/short | 301,340 | 286,080 | 15,260 | 887 | 10 | 36.90 s | yes |

Every run used three Difference calls. After the planned dry run, the model first
supplied the approval token without setting dry_run=false, which kept it in dry-run
mode. It then applied with the flag set explicitly. Validation, visual comparison,
save and restore succeeded in all four runs. The saved outputs were byte-identical
(SHA-256 d81cd89239427ee14fe8edc8178f73f699fc72f382066307edb3a3831764c484).
Every source and restored working copy matched the original fixture bytes.

The serialized STDIO tools array fell from 211,719 bytes (73 tools) to 20,761 bytes
(10 tools), a 90.2% reduction. Total input tokens fell by 1.0% and 0.17% in the two
pairs. Uncached input fell by 4.7% in the first pair and rose by 88.1% in the second.
The CLI JSON events exposed aggregate usage and tool calls. The provider-projected
schemas were unavailable, so schema token attribution remains open under
[#48](https://github.com/P1oN/inkscape-mcp-server/issues/48). These pairs leave the
effect on uncached input cost and end-to-end latency inconclusive.

Two earlier setup attempts were excluded from the matched table. The CLI account
rejected the first model alias. The next pair reached the dry run and was stopped
by the CLI's confirmation policy before the operation reached MCP. The measured
runs granted client approval per tool for the owned synthetic workspace, using
[the Codex MCP configuration](https://learn.chatgpt.com/docs/config-file/config-reference).
The server still required the explicit apply token.

## Reproduce the pilot

Use a local built server binary. Start from
[the preservation fixture](../../../rust/tooling/tests/fixtures/boolean-difference.svg)
and insert the cutter immediately after the body element, inside the same group.
Name the result source.svg. The resulting SHA-256 is
e64e68308ff45fac77b45a20422875dbc43ecd0bbc054126703a4edfea5d131e.

~~~svg
  <circle id="cutter" cx="38" cy="30" r="11"/>
~~~

Use the same brief in each fresh workspace.

~~~text
Use only the Inkscape MCP tools for this synthetic fixture. Run these steps in order:
1. Open source.svg. Find body and cutter with two find_objects calls using id_prefix for each. Match the exact IDs.
2. Dry-run boolean_difference for body and cutter. If the planned surviving ID is body, apply the same operation. I explicitly approve this one apply on this owned synthetic fixture. Use approval_token "issue-49-fixture-approval".
3. Run validate_document. Use compare_region with the returned pre-edit snapshot ID and region {x:0,y:0,width:128,height:96}, width_px 128, inline true. Check that the cut is visible and surrounding artwork looks preserved.
4. Save the edited copy to result.svg with vector_only true. Then restore the working copy using the pre-edit snapshot ID. Keep the saved result.svg.
Briefly report the result and whether each step completed. Do not use shell, direct file reads or writes, other tools, or launch a GUI. Stop and report if a required step fails.
~~~

With absolute paths in server, workspace and engine_dir, and the brief in task.txt,
run once per profile. Use full and compact in the first pair and reverse the order
in the second pair. Each workspace contains only its own source.svg at startup.

~~~bash
settings=$(jq -nr --arg root "$workspace" --arg profile "$profile" \
  --arg path "$engine_dir:/usr/bin:/bin" '{
    INKSCAPE_MCP_WORKSPACE_ROOTS:$root, INKSCAPE_MCP_TOOL_PROFILE:$profile,
    INKSCAPE_MCP_TOOL_DESC:"short", INKSCAPE_MCP_LIVE_ENABLED:"false",
    INKSCAPE_MCP_RAW_ACTION_ENABLED:"false", INKSCAPE_MCP_ENGINE_MODE:"per_call",
    PATH:$path, SENTRY_DSN:""
  } | to_entries | map(.key + "=" + (.value|@json)) | "{" + join(",") + "}"')
/usr/bin/time -p codex exec --ignore-user-config --ignore-rules \
  --skip-git-repo-check --ephemeral --disable plugins --disable apps \
  --enable skip_host_skill_discovery --sandbox read-only -C "$workspace" \
  -m gpt-6-sol -c 'model_reasoning_effort="low"' \
  -c "mcp_servers.inkscape.command=\"$server\"" \
  -c "mcp_servers.inkscape.env=$settings" -c 'mcp_servers.inkscape.required=true' \
  -c 'mcp_servers.inkscape.tools.boolean_difference.approval_mode="approve"' \
  -c 'mcp_servers.inkscape.tools.save_document_as.approval_mode="approve"' \
  -c 'mcp_servers.inkscape.tools.restore_snapshot.approval_mode="approve"' \
  -c 'mcp_servers.inkscape.startup_timeout_sec=30' \
  -c 'mcp_servers.inkscape.tool_timeout_sec=30' \
  --json - < task.txt > "$workspace/events.jsonl" 2> "$workspace/client.log"
~~~

Extract usage from turn.completed and count completed MCP call events, including
any failed calls. Subtract cached_input_tokens from input_tokens for uncached
input. Compare saved SVG bytes between lanes and verify each source and restored
working copy against the fixture hash.

## Automated acceptance

The discovery matrix matched the 16 full/core reference contracts.
Both Rust Cargo graphs passed format, Clippy and their default test suites.
The engine-routes acceptance exercised full/short and compact in both per-call
and shell mode with raw Actions disabled. It checked discovery and direct-call raw
refusals, default dry run, explicit approval, validation, region comparison, vector
save, byte-exact restore, the referenced-cutter refusal and equivalence of saved
SVGs. The SVG structure and rendered pixels were checked independently against the
expected shape. This evidence covers headless execution on synthetic drawings.

## Direct region verification follow-up

[#50](https://github.com/P1oN/inkscape-mcp-server/issues/50) reuses the four
successful #49 client traces. Each used the applied response's snapshot ID.
The 128 × 96 region came from the task brief. The two ID lookups were before
geometry, and there were zero post-edit inspection calls in all four runs.

A new compact-profile Codex CLI run used the same fixture, model, and client
configuration. The brief explicitly set dry_run=false for apply and replaced the
verification instruction with:

~~~text
3. Immediately call compare_region using doc_id, snapshot_id, and changed_region from the applied operation response. Copy changed_region verbatim into region. Use width_px 128 and inline true. Check that the cut is visible and surrounding artwork looks preserved. Then run validate_document.
~~~

The applied response reported result_id body and changed_region
{"x":30,"y":20,"width":74,"height":49}. The next tool call was compare_region
with the same document, snapshot and region. Its crop covers about 30% of the
page by area. The client then validated and saved the edit, and restored the
pre-edit snapshot. The saved SVG and restored working copy matched their
respective #49 hashes.

| Client trace | Geometry to visual comparison | Validation calls | Post-edit inspection calls |
| --- | --- | ---: | ---: |
| #49, each of four runs | One compare_region using coordinates from the brief | 1 | 0 |
| #50, compact | One compare_region using only the applied response | 1 | 0 |

The verification call count held at two. Total MCP calls went from ten to nine
because the updated brief made the apply flag explicit. The traces contain no
whole-SVG read for a single object. The focused workflow uses the existing
find_objects and compare_region tools.

The optional field combines the visual bounds of both pre-edit operands with the
committed survivor in document coordinates. It uses two bounded Inkscape queries
after the transaction commits. Each query has a two-second ceiling. A missing or
unsafe bound leaves the applied response usable without the field. Automated
per-call and shell runs also checked a scaled, letterboxed fixture whose consumed
cutter extends beyond the survivor, and omission when a clone of the affected
group can repaint elsewhere.
