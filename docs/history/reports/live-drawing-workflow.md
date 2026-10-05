# Live drawing workflow candidate — 2026-10-05

Acceptance ran on the uncommitted `codex/live-drawing-workflow` candidate, based on `main`
`44e0e30`. The user subsequently authorized committing, pushing and opening a PR.
This records local development acceptance, not a release, remote CI or artist pilot.
The initial working tree was clean. The user authorized an isolated synthetic GUI run
on 2026-10-05. During acceptance no commit, PR, installed runtime change or operation on a
user's drawing was performed. The owned session was restored to blank and gracefully closed.

## Implemented scope

- Full-profile `live_inspect_objects`: bounded static effective paint, currentColor/defaults,
  ancestor compositing and local gradient/mask/clip/pattern/clone links. Unknown CSS, dynamic
  values, unresolved/cyclic references, instance limitations and bounds remain explicit.
- Full-profile `live_change_package`: 1–16 typed style/transform/text changes on a fixed
  selection of at most 1,000. Review uses immutable candidates and before/after page PNGs
  with portable artifact resources. Document identity, selection, content fingerprint and
  canonical edit digest guard application; the native helper independently verifies actual
  input IDs/content/selection before preparing one candidate. Existing approvals, audit,
  workspace/symlink boundaries and uncertain-result recovery remain in place.
- Pure package kernel reuses the native edit planner/applicator. Failed members never publish;
  net DOM no-ops produce no SVG bytes, native dispatch or Live Operation Records. Unsupported
  transports refuse before persistence/dispatch. Changed packages use one existing record.
- Owned synthetic mask/clip, linked-pattern, linked-gradient and clone study; real STDIO/
  native snapshot-helper/CLI acceptance. Prepared owned native `package`/`package-noop` phases.
- Tool schemas, intent guidance, user/developer guides and both generated manifests updated.
  Full surface: **112 tools, seven prompts, 18 resources**; advanced tools remain full-profile.

Supported operations, rendering gates, failure/recovery semantics and the deferred pilot
protocol are in [the workflow guide](../../live/reviewed-workflow.md).

## Fresh automated and CLI evidence

Host: Apple Silicon macOS; Cargo/Rust 1.99.0; Inkscape CLI **1.4.3 (0d15f75, 2025-12-25)**.
Use the Cargo/macOS SDK environment from [CONTRIBUTING](../../../CONTRIBUTING.md).

| Check | Result and limits |
| --- | --- |
| Locked runtime Cargo suite | **301 passed, zero failed, two existing opt-in ignored**; includes static CSS/resources, missing/cyclic refs, package validation/refusals/net no-op, native input guards, stale managed context/selection/content, one dispatch and one public record |
| Locked tooling suite | **15 passed, zero failed** |
| Both fmt and all-target Clippy graphs | Passed with `-D warnings`; final `git diff --check` passed |
| Release build | All five native binaries built; candidate hashes below |
| Discovery | All **16** live/raw/profile/description configurations exactly match updated contracts; startup did not create a GUI/session |
| Live workflow probe | Real STDIO, actual Rust socket snapshot helper and Inkscape CLI: static resource relationships, drawable IDs/nonempty engine bounds, isolated previews, masked opaque/transparent pixels, package before/after images and readable artifact resources. Stale content, changed edit digest, absent approval and unsupported socket apply refuse; original bytes and drawing fingerprint unchanged |
| Ten standard STDIO suites | Security, frames, 128-session/four-concurrent startup, responsiveness/cancellation in both engine modes, defects, diagnostics, comparison, special files, isolated renderer and engine routes passed on the final packaged server |
| Relocated package acceptance | Both per-call and shell modes passed: **443 files**, empty PATH, actual native INX/private-bus exchanges, headless approvals/no-ops/batch rollback/restore/resources and real CLI pixels. This is independent of native package GUI Undo/Redo |
| Visual inspection | Inspected before/after package PNGs: masked half-circle, linked dot panel, gradient flower and clone remain; only the caption text and color change as planned |

The first live-workflow probe failed because its harness expected black RGB channels in a
fully transparent pixel; the CLI returned transparent white (`[255,255,255,0]`). The harness
expectation was corrected, with the original failure retained under `p2-live-workflow/`;
no renderer/product workaround or timeout retry was used. `p2-live-workflow-corrected/`
then passed. The final probe additionally checks portable artifact reads.

### Reproduction

```sh
cargo fmt --check --manifest-path rust/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/Cargo.toml
cargo fmt --check --manifest-path rust/tooling/Cargo.toml
cargo clippy --locked --all-targets --manifest-path rust/tooling/Cargo.toml -- -D warnings
cargo test --locked --manifest-path rust/tooling/Cargo.toml
cargo build --locked --release --manifest-path rust/Cargo.toml
scripts/dev-tools.sh discovery --binary rust/target/release/inkscape-mcp-rust --matrix --output migration/results/p2-final-discovery
scripts/dev-tools.sh manifests --binary rust/target/release/inkscape-mcp-rust --output .
scripts/dev-tools.sh live-workflow-acceptance --binary rust/target/release/inkscape-mcp-rust --helper rust/target/release/inkscape-mcp-live --output migration/results/p2-final-live-workflow
scripts/dev-tools.sh build-package --output migration/results/p2-final-package --archive migration/results/p2-final-package.tar.gz
scripts/dev-tools.sh package-acceptance --archive migration/results/p2-final-package.tar.gz --output migration/results/p2-final-package-cold
scripts/dev-tools.sh package-acceptance --archive migration/results/p2-final-package.tar.gz --engine-mode shell --output migration/results/p2-final-package-shell
```

Each standard suite uses `scripts/dev-tools.sh NAME-acceptance --binary
migration/results/p2-final-package/bin/inkscape-mcp --output migration/results/p2-final-NAME`,
where NAME is security, frame, startup, responsiveness, defects, diagnostic, compare,
special-file, renderer or engine-routes. Renderer and package acceptance run sequentially.
Fresh server processes load the changed schemas/instructions while preserving existing GUIs.
No installed client/runtime is replaced.

Raw logs/JSON/wire traces/retained SVGs/PNGs are Git-ignored under `migration/results/p2-final-*`.
Runtime/tooling logs are in `p2-final-checks/`; new-tool fixture/report/images are in
`p2-final-live-workflow/`. Earlier candidate package evidence is retained under `p2-*` and
must not be substituted for final binary evidence.

| Candidate artifact | SHA-256 |
| --- | --- |
| Server | `ca91574f9298a95536456a30cd8a865337e2553b32f8df59f4b41f89b2024224` |
| INX helper | `ffb68aca6b8a419b456f8905a2e967981ce2ca9636e0cf9809afe1df9bbca294` |
| Socket helper | `bcac92b8ac4fc057ff8c7240b21d7385487413878afad60be1d25714761ca16e` |
| Synthetic resource SVG | `41f6fab651d671db15cce88cc6e2f864f7547057f392d922ff6762f484ced1ba` |
| Ready package archive | `1a94b30e42ef7aef278e2f34223cf1b91c5632551bbd08b4f06d535063850bd9` |

## Scoped native GUI acceptance

The user authorized this run on 2026-10-05. The final packaged binaries above launched
one private managed macOS session with its own HOME/profile/workspace/private bus. The
recorded context bridge app and document were selected explicitly; existing user sessions
were not operated on. Evidence is retained under `migration/results/p2-native-gui/`.

- Native insertion produced a named group containing editable `Hello` text. The text was
  selected through the native text tool, then the selector, rather than guessing the group.
- `reviewed` applied fill `#d95d72`, opacity `0.75` and text `Reviewed & editable` through
  one reviewed package. One matching applied Live Operation Record was verified.
- `repeated` reapplied the same package: unchanged SVG and operation history, no dispatch.
- One native **Edit → Undo** after the no-op restored the package-before XML. One native
  **Edit → Redo** restored package-after XML. Recursive XML comparison ignores only
  `sodipodi:namedview`; separately rendered 800px page PNGs match byte-for-byte for both
  before/Undo and after/Redo pairs. `gui-comparison.json` records checks and image hashes.
- `selected-stale-content`, `selected-stale-ids` and `selected-stale-selection` exercised
  the actual INX guard with deliberately stale request fields; each refused without SVG
  mutation. These are native request guard checks, not observed intervening human edits
  after a retained package review or a multiple-window race.
- The first stale-content invocation stopped before mutation because Undo/Redo had cleared
  the actual selection, despite the visible text handles. Its trace is retained. The text
  was selected again through GUI before the three fresh guard phases.
- Two final native Undo actions restored the owned blank. `native-gui close-owned` checked
  the original context, manifest, private executable/process ownership and blank capture,
  then gracefully closed only the owned session.

| Native page PNG pair | Matching SHA-256 |
| --- | --- |
| Before / native Undo | `78e9e6a4bd7583202ce254b584a810c95f7b99054cf36324f8bb63f410c9953a` |
| After / native Redo | `5e9b9461c8ab9e458a8dc2196e99f1919347205713ad664fbec1f3b9223f2247` |

This establishes one Undo/Redo transaction for this **two-member style/text package on a
single synthetic text object**, on the recorded build and Inkscape version. It does not
establish arbitrary package live atomicity, all transformations/selection sizes or race-free
human review. Runtime `native_undo_verified=false` and package `undo_friendly=false` remain
conservative per-call fields; no automatic environment capability is inferred from this run.

## Unverified acceptance

**Real artwork/artist pilot/separate Mac/terminal-free installation usability:** user-deferred
on 2026-10-05 until works and a separate Mac are available. No observations or terminal-free
product flow have been invented. Synthetic resource evidence is not realistic-artwork acceptance.
Only this unfinished acceptance remains in [the active backlog](../../RUST_NEXT_PLAN.md).
