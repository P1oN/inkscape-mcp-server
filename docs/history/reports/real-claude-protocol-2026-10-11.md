# Real Claude Code protocol acceptance — 2026-10-11

An official, pinned `@anthropic-ai/claude-code@2.1.296` npm installation was placed
only in the owned acceptance directory. The documented npm postinstall placed its
native Darwin arm64 binary; no global user installation was changed. Commands used
an isolated HOME/CFFIXED_USER_HOME and working directory, an explicit environment
without account/API credentials, and disabled telemetry/error reporting/auto-update.
No model session, login or paid API request was performed.

[Official installation](https://code.claude.com/docs/en/setup#install-with-npm) and
[MCP diagnostics](https://code.claude.com/docs/en/mcp#server-status) describe the
installation and `mcp get` interface used here. Native CLI SHA-256:
`c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937`.

## Reproduced published-runtime failure

The published v0.1.6 signed installer prepared/activated both Codex and Claude in
this separate profile, preserving a TOML sentinel, unrelated Claude preferences
and an unrelated MCP registration. Real `claude mcp get inkscape` returned exit zero
but reported `Connected · tools fetch failed`: protocol revision 2026-07-28 requires
`resultType`, and the frozen tools/list response omitted it. Exit zero or the word
Connected alone therefore does not prove client acceptance.

Restricting the advertised versions was insufficient in this SDK's discovery-first
connection: after fallback, the SDK still required per-request metadata. That trial
was superseded by implementing complete current-protocol response envelopes.

## Implemented behavior and scoped acceptance

The server now emits `resultType: complete` for tool/prompt/resource completion,
including error, render and wait paths. List/resource responses use zero TTL and
private cache scope for 2026-07-28 requests. The SDK removes the discriminator for
older negotiated peers, and the server leaves their cache fields absent. The
explicit supported-version ceiling prevents a future SDK update from silently
advertising a newer, untested protocol. See the
[2026 tool response contract](https://modelcontextprotocol.io/specification/2026-07-28/server/tools).

Real Claude 2.1.296 then reported `Status: ✔ Connected` with no discovery failure or
Issue line against the local repaired development server, build `b8dbcdc944e58037`.
That test server SHA-256 was
`643da4a4b6ae430938dfefe4e5f31d26e2ba607e02564f07324f05098c815be1`;
it is a development binary, not a new signed distribution. Its temporary extra
registration was removed through the real CLI, preserving the installer binding,
unrelated registration and preference sentinel.

Validation: all 344 Rust tests and all-target Clippy pass. A bounded real STDIO
regression exercises discovery-first 2026-07-28 and initialized 2025-03-26 sessions:
tool/resource/template/prompt lists, prompt rendering, workspace/intents reads,
workspace tool calls, render refusal and unknown-tool refusal. It requires modern
discriminators/cache fields and equal legacy payloads after those envelope fields
are removed. The 16-mode discovery matrix matches every frozen contract exactly.
MCP manifests were regenerated and remain unchanged.

Review found an omitted discriminator on the asynchronous live-wait error branch.
The STDIO regression now exposes live tools without connecting or launching
Inkscape, calls a zero-timeout wait with no session, and verifies that the actual
no-session refusal has a modern discriminator and unchanged legacy content. It
failed on the omitted field before the one-line repair and passes afterward.

This proves local real-CLI discovery and protocol behavior. Published v0.1.6 retains
the reproduced incompatibility with this Claude version. Qualification on new signed
bytes, authenticated interactive client/tool use, live Inkscape reconnect and the
other-Mac/browser-quarantine matrix remain separate gates.
