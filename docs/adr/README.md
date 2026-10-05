# Architecture decision records

These records summarize decisions already implemented by 2026-10-05. They are retrospective
records, not new approval or acceptance claims. Detailed milestones and test evidence live in
[history](../history/README.md); current behavior and unfinished work live in the
[handoff](../AGENT_HANDOFF.md) and [backlog](../RUST_NEXT_PLAN.md).

| Decision | Status |
| --- | --- |
| [Native Rust runtime and tooling](native-rust-runtime.md) | Accepted; Python removal stages 1–7 merged in PR #9 |
| [Owned managed GUI and guarded edits](managed-gui-ownership.md) | Accepted; bounded integration remains experimental |
| [Shared vector guidance and explicit advisory roles](editable-vector-authoring.md) | Accepted; authoring merged in PR #9 |

For a new architectural change, record context, decision, consequences, validation limits and
links to the implementation. Keep execution logs and completed task checklists in history.
