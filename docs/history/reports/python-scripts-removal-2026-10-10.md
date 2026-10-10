# Remaining Python removal — 2026-10-10

The user explicitly requested removal of all Python scripts, including the retired
historical sources, after the release workflow introduced Python standard-library guards.
The shipped Rust runtime already required no Python. Six active scripts handled signed
release validation/tests; two obsolete eval runners still imported the removed Python
server, and 54 historical Python source files remained archived in the checkout.

Active guards are now the portable `inkscape-mcp-release-guard` Rust binary, built through
`scripts/release-guard.sh` from the existing locked tooling graph with no default native
features. It validates signatures, final asset hashes/inventory/CI provenance, Accepted
notarization records, final native receipts and immutable runtime references; records
acceptance only after all required gates; and extracts bounded CI tar archives after
preflight traversal/link/duplicate/parent/special-file/resource checks. Publication still
uses exact remote digests and numeric draft IDs. No runtime or MCP tool was added.

`scripts/test-signed-distribution.sh` runs Rust fixtures and isolated Bash signing-secret,
pinned-toolchain, identity and empty/populated-argument checks. The GitHub mock is Bash/jq,
never the real GitHub CLI. Linux CI verifies the guard without native SDK dependencies;
macOS retains Bash 3.2 checks. The signed workflow installs pinned Rust for its portable
guard jobs. Shipping application binaries still come from verified candidate artifacts.

All 62 tracked `.py` files, the historical Python tree, `.python-version` and obsolete
helper dependency list were removed. Original source remains recoverable from Git.
Historical reports and JSON/third-party license evidence remain unchanged in scope;
references to old commands describe their original revisions, not runnable current tools.
The unused eval dataset remains historical data; its Python-backed runners were removed
rather than claiming they evaluated the current Rust server.

Validation results are recorded after the applicable checks finish. This source cleanup
is independent of the unresolved protected signing-identity failure and is not release,
notarization or another-Mac acceptance evidence.

Local validation passes: eight portable Rust guard tests, sixteen native tooling tests,
four source/bootstrap tests, the signed-distribution shell fixtures, legacy publication
fixtures, tooling formatting/Clippy, Bash syntax and actionlint. Tracked source inventory
contains no Python sources, bytecode, wheels, Python project manifest or version file;
active workflows/scripts contain no Python invocations. Linux execution is covered by the
new CI job and remains pending its actual run. Native GUI acceptance was not repeated.
