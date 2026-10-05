# Editable vector authoring quality — implementation and acceptance

Implemented locally on `4aacfae` in `codex/python-removal-stages-1-3` on 2026-10-05.
The preceding Python-removal work was committed and pushed to PR #9 by user request.
This authoring follow-up changes guidance and read-only reporting, not live mutations.
No installed runtime, user configuration, published asset or existing GUI was replaced.

## Delivered behavior

- `migration/contracts/authoring-guidance.txt` is the shared English policy source.
  Initialization and `compose_artwork` expand it at runtime. The 16 discovery snapshots
  and compose template contain its generated text; `scripts/dev-tools.sh
  sync-authoring-guidance` refreshes them. Regression checks exercise compose rendering
  against every configuration and enforce identical policy delivery.
- `quality_report.editability.object_roles` accepts up to 200 unique explicit object IDs
  (1–256 UTF-8 bytes) with `stroke_only` or `independent_strokes` roles. Unknown/ambiguous
  SVG IDs produce unknown findings; malformed/repeated role entries are rejected.
  Roles are never inferred from names, primitive counts or open paths.
- `editability.authoring` contains bounded `findings`, `truncated` and `scope`. Each
  finding has `object_id`, `code`, `certainty` and `reason`. Stroke fill includes default,
  inherited and transparent paint; `fill-opacity=0` does not substitute for `fill=none`.
  The strict advisory cascade reuses the existing CSS selector/declaration infrastructure
  with shared declaration storage and a byte-weighted work budget, while correctly
  resolving presentation/style priority, importance, inheritance and
  source order. Legacy inspector behavior is unchanged. Unsupported syntax, selectors,
  external/conditional stylesheets, invalid selectors, foreign presentation styling,
  animation, context paint and exhausted limits are
  explicit uncertainty. Advice never alters validity or the existing quality score.
- Pinned `svgtypes 0.16.1` parses actual path segments and implicit repeated commands.
  Multiple subpaths are advised only for designated independent strokes. Undesignated
  compound filled paths are not errors. Parsing and traversal have explicit byte,
  segment, element, depth and work limits documented in the usage guide.
- Hidden/transparent scene observations account for ancestry, visibility overrides,
  static styles, paint opacity, resource containers and local references. A container
  with hidden visibility is uncertain because a descendant can override visibility. Required
  `defs`, `use` source subtrees, masks and clipping geometry are distinguished; uncertain
  external/encoded references remain uncertain. General occlusion is deferred. Partial
  overlap is allowed; bounding boxes are not proof of coverage.
- No cleanup tool or automatic geometry conversion was added. Reviewed repairs use
  existing approval-gated deletion/path operations, snapshots, Operation Records,
  original-file protection, reference refusals and no-op behavior.

## Reproducible checks

```sh
cargo fmt --check --manifest-path rust/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/Cargo.toml
cargo fmt --check --manifest-path rust/tooling/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/tooling/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/tooling/Cargo.toml
cargo build --locked --release --manifest-path rust/Cargo.toml
scripts/dev-tools.sh build-package --output migration/results/authoring-accepted-package --archive migration/results/authoring-accepted-package.tar.gz
scripts/dev-tools.sh discovery --binary migration/results/authoring-accepted-package/bin/inkscape-mcp --matrix --output migration/results/authoring-accepted-discovery
scripts/dev-tools.sh manifests --binary migration/results/authoring-accepted-package/bin/inkscape-mcp --output .
scripts/dev-tools.sh authoring-acceptance --binary migration/results/authoring-accepted-package/bin/inkscape-mcp --output migration/results/authoring-accepted
scripts/dev-tools.sh authoring-acceptance --binary migration/results/authoring-accepted-package/bin/inkscape-mcp --engine-mode shell --output migration/results/authoring-accepted-shell
scripts/dev-tools.sh notices-acceptance --package migration/results/authoring-accepted-package --output migration/results/authoring-accepted-notices
```

Use pinned Cargo on PATH and the macOS libxml SDK setting from CONTRIBUTING. Archive
paths must be new; retained evidence is never overwritten by a package rebuild.
Development-only binaries require Inkscape's CLI directory on PATH; the macOS ready
package uses its fixed engine fallback. `authoring-acceptance` starts fresh isolated
STDIO servers, loading the changed instructions without interacting with a GUI.

Fresh automated evidence: runtime **295 passed / 2 standard opt-in ignored**; tooling
**15 passed**; both fmt/Clippy graphs passed. Logs are in
`migration/results/authoring-checks`. All 16 discovery configurations are checked against
frozen snapshots. `llms.txt` and `llms-full.txt` are regenerated from the rebuilt server.
The accepted package has **443 inventory files / 194,912,958 inventory bytes**; archive
**49,206,796 bytes**, SHA-256
`f3bbd9803f7fa21f60f0a4f82388da652e8f9dd76e119851cbe03d5403800781`.
Relocated package acceptance passes both per-call and shell modes. Notices passes 11 checks
with 196 crates, including the new locked parser graph. Ten fresh STDIO suites pass against
that package: security (35), frame, startup (128 sessions/four workers), responsiveness
(both modes/cancellation), diagnostics (3), defects (5), comparison (3), special files (12),
renderer (22) and engine routes (8). Their `accepted-*.log` files and raw
`migration/results/authoring-accepted-*` directories identify the actual candidate.
Earlier candidates/results are superseded; the accepted package is the current evidence.

Fresh CLI authoring evidence is retained in `migration/results/authoring-accepted` and
`migration/results/authoring-accepted-shell`: trace, report, deliberately authored SVG,
before/after/restored PNGs and operation records. Structural assertions independently
check one continuous petal outline, a separate flower center, two separate fold objects,
a circular snowball and no embedded raster artwork. Reports preserve workspace bytes,
snapshots and records and keep validity/score unchanged. An unapproved synthetic deletion
is refused; explicit approved removal of the reviewed fully covered rectangle preserves
all decoded pixels. Repeating it is a history-free no-op, snapshot restoration preserves
pixels, and deletion of a source subtree referenced from outside that subtree is refused
before mutation/snapshot. Partially visible shapes and required resources survive.

The retained rendered fixture is also inspected visually for the rounded snowball,
smooth flower silhouette/separate center and unfilled folds. This is separate from
structural assertions and does not prove general model compliance or reference matching.
No native GUI/Undo/Redo acceptance is claimed; live behavior is unchanged. Foreign targets,
clean-machine installation and general occlusion remain outside this workstream.

During development an edge regression caught `opacity="0%"` being mistaken for an encoded
reference. Reference uncertainty now applies to reference URLs, not arbitrary percentage
values; the regression is retained. A candidate retained the previous tool description because the packaging command copies
prebuilt binaries; release was explicitly rebuilt and the verified package/discovery
supersedes that candidate. Initial direct-development rendering lacked an
Inkscape PATH entry; the explicit CLI run and final packaged runs provide fresh evidence.

## PR state

Stage 7 was pushed as `4aacfae`; PR #9's title/description were updated and its native
candidate CI passed. CodeRabbit full and incremental review commands were sent, but the
service skipped both: 161 and 118 files respectively exceed its 100-file limit. A skipped
review is not a completed review or a clean-code result. No paid on-demand review or
review-scope exclusions were introduced. The user subsequently authorized committing
and pushing this authoring implementation in the same PR, then merging into `main`
after checking the new candidate's CI. The acceptance evidence above predates that
commit and remains tied to its recorded package/build identity.
