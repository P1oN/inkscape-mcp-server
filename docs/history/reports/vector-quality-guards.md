# Closure, duplicate path nodes and vector-only delivery

Requested and implemented locally on **2026-10-05**, on `codex/live-drawing-workflow`
from `d16083a`. This is an uncommitted candidate, not a published or installed runtime.
The final release server SHA-256 is
`3b64b463cfc2d494c1d6a5e8da313d9c9423ee1cc8ab9d57ce2a1cafd8a91008`.

## Behavior

- `create_path` parses bounded SVG path commands and rejects malformed/nonfinite geometry,
  including relative arithmetic overflow. Optional `require_closed` and `reject_zero_length`
  apply identically to standalone creation and typed batches, before transaction writes.
  Each subpath needs explicit `Z` when closure is requested. Intentional dots/open strokes
  remain allowed by default. Coincident-endpoint curves with distinct controls remain loops;
  a zero-length closing edge is not removed or rejected.
- `quality_report` supports explicit `closed_shape` roles and read-only path findings,
  optional path-local `node_tolerance`, and separate whole-document `vector_content` status.
  Advice does not alter SVG validity or score. Missing/ambiguous/unsupported targets and
  exhausted budgets remain unknown. Cross-object overlap is not inferred.
- `save_document_as(vector_only=true)` refuses recognized embedded raster or unknown image,
  resource, stylesheet or dynamic content before destination preparation/history writes.
  Hidden/defs images, clones, `feImage`, opaque embedded SVG and linked images are included.
  References cannot be exempted from the final gate. Default saving remains compatible.
- `replace_svg_fragment(dry_run=true)` uses the existing replacement/reference kernel on a
  disposable document and returns a bounded structural `candidate_svg`/`would_change`.
  No working bytes, snapshots or records change; no approval is needed for this branch.
  Render review remains separate, and applying the explicit replacement retains the existing
  HIGH-risk gate. This is not automatic geometry cleanup or an approval bound to a candidate.
- Shared authoring guidance and all 16 discovery snapshots are synchronized. Both MCP catalogs
  were regenerated; `llms.txt` is byte-identical because its short summaries did not change.
  Tool counts are unchanged. The usage guide documents bounds and conservative refusals.

## Validation

| Check | Result and scope |
| --- | --- |
| Final sequential runtime suite | 308 passed, two opt-in ignored; full native/synthetic Rust graph. `migration/results/vector-quality/checks/runtime-serial.log` |
| Tooling graph | 15 passed, both fmt/Clippy graphs passed; final release build passed |
| Discovery | Exact schemas/instructions in all 16 configurations: `migration/results/vector-quality/final-discovery/` |
| Authoring acceptance | Final binary, real STDIO plus Inkscape CLI, per-call and shell modes: `final-authoring-cli/probe.json`, `final-authoring-shell/probe.json` under `migration/results/vector-quality/` |
| General STDIO suites | Security, frame, startup (128 sessions/four workers), responsiveness, defects, diagnostic, compare, special-file, renderer and engine routes; final results under `migration/results/vector-quality/final-*` |
| Regression invariants | Standalone/batch closure and zero-segment refusal, malformed/nonfinite paths, loops, smooth/relative curves, intentional dots, multiple subpaths and path budgets; read-only findings; dry-run candidate/no-op/refusal/output bounds; vector save refuses without creating parents/records or overwriting an existing destination |
| Native GUI | Not run: no live mutation/helper changes. User GUI/windows and installed runtime were preserved |

The first full run caught default-value fixtures that needed updating for the new optional
parameters; its retained failure is `checks/runtime-initial-defaults-failure.log`. After that
repair, the default-parallel run failed the already recorded
`live_launch::tests::private_session_and_locks_refuse_link_escape_and_parallel_launch`
with `managed Inkscape launch failed`; see `checks/runtime-parallel-live-lock-failure.log`.
This remains an unresolved pre-existing concurrency issue. The successful final sequential
run is diagnostic evidence, not a claim that default-parallel execution is clean.

The first authoring CLI invocation lacked Inkscape in PATH and failed at rendering; its trace
remains in `migration/results/vector-quality/authoring/`. Subsequent invocations explicitly
used `/Applications/Inkscape.app/Contents/MacOS` in PATH and fresh evidence directories.
The synthetic flower/folds/round outline PNG was visually inspected; automatic pixel equality
checks cover the existing approved covered-object repair, no-op and restore. These do not
establish artistic quality on real user artwork or GUI acceptance.

Activate the code by explicitly rebuilding the selected runtime and reconnecting MCP while
preserving Inkscape. This task did not replace configured binaries, restart/kill user processes,
commit or publish. Fresh isolated STDIO processes loaded and verified the changed instructions.

## Delivery follow-up

On 2026-10-05 the user authorized committing and pushing all local changes into a PR.
PR #10 was already merged; `codex/vector-quality-guards` starts from its merge commit
`f6dc569` on `origin/main`, whose source tree equals the validation base. The validation
results above retain their original uncommitted build identity. Ignored raw test outputs
remain local evidence; publication does not update the user's configured MCP runtime.
