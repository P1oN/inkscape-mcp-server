# Repository instructions

Start with [docs/AGENT_HANDOFF.md](docs/AGENT_HANDOFF.md), [README.md](README.md),
[CONTRIBUTING.md](CONTRIBUTING.md) and [docs/agent-usage-guide.md](docs/agent-usage-guide.md).
The handoff records the next six improvements and distinguishes pending work from shipped features.
Check current code and Git status before relying on recorded counts or validation results.

- Preserve existing uncommitted work. Do not reset, clean or restore files from HEAD to discard it.
- Keep tools typed and bounded; reuse the edit pipeline, snapshots, Operation Records, no-op
  handling and existing approval gates. Do not add arbitrary code, shell or extension execution.
- Preserve original files, workspace/symlink protections, SVG IDs and references. Structural
  changes must preserve appearance or fail before mutation; headless and live limits differ.
- For authoring through MCP, do not propose or perform bitmap tracing or automatic raster-to-vector
  conversion, including external preprocessing. Use bitmaps only as visual references and
  deliberately author editable vector geometry. Do not substitute embedded raster artwork.
- Use ordinary named groups for semantic objects by default; layers organize the scene.
  Respect user-requested structure and preserve existing artwork. See the shared guidance in
  the MCP instructions in `migration/contracts/` and `docs/agent-usage-guide.md`; do not create conflicting copies of the policy.
- MCP startup/reconnect must not launch Inkscape. Launch requires a user request. Do not close
  user windows or kill Inkscape processes; native acceptance uses separate synthetic documents.
- The legacy Python MCP server and paired parity workflow are retired by user instruction;
  evolve Rust with regression and invariant checks, not repeated Python comparisons.
- Run appropriate checks from CONTRIBUTING, update docs, and regenerate `llms.txt` and
  `llms-full.txt` when the exposed surface changes. Distinguish automated tests from native GUI
  acceptance. Restart the MCP server to load changed instructions, preserving the existing GUI.
- Do not commit, publish a PR or send messages unless the user asks.
