# Rust completion checklist

Current candidate stage38, 2026-10-03. User decisions supersede the original executable
Python oracle requirement: rewritten Python MCP retired; Windows backlog; clean-machine
install user-owned; historical live investigation only on recurrence; timing refinement secondary.

| Requirement | State and scoped evidence |
|---|---|
| Separate branch and preserved work | codex/rust-migration; retirement backup and local uncommitted changes preserved |
| Native Rust server, official pinned SDK | rmcp 3.5.0/Cargo.lock; no Python MCP subprocess |
| External discovery contract | Actual38 16 configurations match frozen JSON exactly; full 110 tools / 7 prompts / 18 resources |
| Arguments/results/families | Historical full family/error review; current 215 Rust tests and affected actual38 STDIO regressions; finite coverage |
| Edit/XML/filesystem safeguards | Actual38 cold/warm transactions, security35/files12, diagnostics3, limits5, frame bound, compare publication, assets17/routes8 both engines; see security review for limits |
| Runtime/helpers/bridge | Private packaged CPython + six wheels, two fixed effect helpers, supervisor and prebuilt bridge/bus; helper tests 6 and investigation 20; 122 current native checks include Undo/Redo |
| Ready archive/setup/doctor | Actual38 empty-PATH temporary installations, launcher11, doctor10; source setup/run without manual env edits |
| Dependency attribution | Exact crate/runtime/wheel/native provenance and notices17; Homebrew BSD2 bound; source kit supplied; not legal certification/full binary reproduction |
| Native live behavior | Actual38 retry:122 fixed native checks +5 two-window guard checks +closed-session reconnect |
| Startup/RSS/read/edit/batch/CLI measurements | Historical results retained; current stage38 Rust-only headless COMPLETE (10 runs/250 timings); current live COMPLETE (5 reconnects/15 stable observations) |
| Documentation and manifests | Current docs consolidated, history retained; llms generated from actual38 and match |
| Other targets/signing | Only current Mac proved; foreign jobs prepared; Windows deferred; ad-hoc signing, no Developer ID/notarization |
| Final delivery / goal | COMPLETE within user-confirmed current Mac scope; no commits/PR/publication |

Final evidence index: [package38](../migration/package-stage38-final-comparison.json).
Final scoped audit: [requirements](../migration/requirement-audit-stage38-final.json).
A current headless pass does not transfer old native/performance results. No finite suite
proves all compatibility or security combinations. See [next plan](RUST_NEXT_PLAN.md).
