Current package checkpoint: stage36 (2026-10-03). Actual cold/warm archive install,
doctor10/launcher11/notices17 pass.2532FILES entries verified. Exact comparison to35:
2506 existing files unchanged, including every executable/runtime/helper/native asset;
only inventory/package metadata changes and24 added CPython/source notices. Thus35
binary-specific security/discovery/CLI/diagnostic evidence applies to identical bytes,
without inferring new GUI/performance acceptance. Build index: package-stage36-build-comparison.json.

Provenance review: six exact PyPI wheels,1259 matching members. Every upstream RECORD
row retained; only hashed uv INSTALLER/empty REQUESTED metadata and two omitted NumPy
CLI records added. Installed GLib recipe matches exact SHA-verified SBOM bottle; current
API-cache recipe differs and was not substituted. Sole GLib patch history commit predates
the bottle; patch applies to exact upstream2.90.0. Native source kit includes four exact
upstream archives/recipes/receipts/SBOMs, patch and recipe's gobject-introspection resource.
Raw: wheel-source-audit-stage35-final and homebrew-source-audit. Initial wheel-report
serialization failure was a local audit variable-shadowing bug, fixed; earlier raw retained.

19CPython license texts/PYTHON.json/provenance and GLib recipe/patch now packaged in36;
wheel hashes enforced by builder. Homebrew recipe BSD2 license additionally retrieved
and prepared in collector, but not yet in36; include it and the source-kit license in the
next consolidated package. Do not claim complete redistribution clearance. Source kit
is a source-input artifact, not a reproduced binary. Current GUI/performance evidence
remains historical. Goal active: finish remaining publication/output scope and current
synthetic native assessment, then final requirement/documentation audit. Windows,
clean-machine installation and historical incident cause remain user-deferred.

CPython provenance checkpoint (development, 2026-10-03): actual stage35 runtime matches
Astral CPython3.12.14+20260929 macOS arm64 stripped release. Both downloaded release
archives match GitHub asset SHA-256.946 regular runtime members outside site-packages
match byte-for-byte; libpython matches exact install_name_tool reproduction with the
original basename; all sysconfig values match published uv transformations; the added
EXTERNALLY-MANAGED marker matches explicitly. Additional executable sysconfig statements
are refused. Earlier codesign simulation used a different basename and yielded a different
signature; that was an audit-fixture issue, not a runtime defect. Raw failures are retained.
Pinned build recipe b498734a5791d0e6786695a226fd398a41c6f7f6 records source URL/hash/version;
actual packaged OpenSSL3.5.9/SQLite3.53.1/Expat2.8.5/mpdecimal4.0.0 agree with the recipe.
Report: migration/cpython-stage35-provenance-comparison.json.

19 full-distribution license texts plus PYTHON.json/provenance are now checked in under
migration/vendor-notices/cpython. Collector selects them only for the exact reviewed
executable hash; unknown targets/binaries remain explicit gaps. Collection tests and
Python lint/format pass. Current archive remains35 and DOES NOT yet include these texts.
This is regular-file runtime attribution, not wheel provenance, complete source rebuild
or legal clearance; conservative licenses do not imply every named library is linked.
Next: exact Homebrew GLib patch/source metadata, wheel attribution and fresh packaged
notices, scoped remaining audit/current native assessment. Goal remains incomplete.

Current stage35 asset-staging package passes cold/warm archive installation, launcher11,
doctor10, notices15 and discovery16; see RUST_MIGRATION_REPORT.md. Helper source is runtime/,
not src/inkscape_mcp. Earlier package checkpoints below are historical.

# Native Rust candidate packaging

This is a developer workflow. End users install Inkscape plus the archive and run its
`bin/inkscape-mcp`; they do not install Python, uv, Homebrew, Rust or a compiler.
The current verified local archive and installation commands are in
[RUST_MIGRATION_REPORT.md](RUST_MIGRATION_REPORT.md#install-and-check-the-current-local-candidate).

## Targets and evidence

| Target | Prepared native runner | Current evidence |
|---|---|---|
| macOS arm64 | `macos-15` | Stage35 archive cold/warm/CLI/STDIO/asset/route checks pass; historical native29 remains scoped to27 |
| macOS x86_64 | `macos-15-intel` | Build/package/acceptance job prepared; no Intel execution or archive claimed |
| Linux GNU x86_64 | `ubuntu-24.04` | ELF bundler and archive acceptance job prepared; synthetic relocation checks only |
| Linux GNU arm64 | `ubuntu-24.04-arm` | Same prepared job; no Linux execution or archive claimed |
| Windows | None yet | Native handle/reparse/rename protections and runtime port required before a package job can pass |

The labels follow [GitHub's hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
The workflow checks the actual Rust host triple; it cannot label a cross-compiled or wrong
architecture executable as native. Prepared jobs are local source, not completed CI runs.
No workflow, commit, PR or artifact was published. Older glibc/macOS compatibility and Linux
variants such as musl/NixOS are not established by these Ubuntu/macOS runner selections.

## Build inputs and output

Use Rust 1.99.0 with the locked Cargo dependencies and a native managed CPython 3.12.14.
The six private helper distributions are pinned in `rust/package/helper-requirements.txt`.
CI obtains the runtime with [uv's managed Python](https://docs.astral.sh/uv/concepts/python-versions/);
uv is a developer dependency and is omitted from the archive. Setup-uv is pinned to the
verified v6.8.0 commit and uv to 0.12.22. Wheel-only dependency installation fails if a
required native wheel is unavailable, rather than compiling on an end-user machine.

The native host needs Inkscape >=1.4 for acceptance. macOS builds need GLib headers, clang,
D-Bus tools and the official architecture-matching Inkscape app; the bridge is prebuilt.
GNU/Linux builds need libxml2 development files, patchelf, D-Bus/GLib command tools and
Debian/Ubuntu package metadata for dependency-license provenance. Linux's glibc version is
recorded in the package manifest; an older baseline is not inferred from the target name.

After the locked native release build, create a new output tree and archive:

```sh
.packaging-venv/bin/python scripts/migration_build_posix_package.py \
  --output dist/inkscape-mcp-<native-target> \
  --archive dist/inkscape-mcp-<native-target>.tar.gz
```

Targets use `macos-arm64`, `macos-x86_64`, `linux-arm64` or `linux-x86_64`. Existing output
folders/archives refuse. The original `migration_build_macos_package.py` CLI remains a
compatible entry point to the same implementation. Python tarfile creates one package root
without macOS AppleDouble sidecars; extraction uses the bounded data filter. Private runtime,
fixed helpers, bus and manifest are included, never the Python MCP server or its dependencies.

macOS non-system D-Bus dylibs use `@loader_path` and ad-hoc signing. Linux packages copy the
non-glibc ELF closure into `libexec/inkscape-mcp/native-lib` and set relative `$ORIGIN` RPATH
with [patchelf](https://github.com/NixOS/patchelf). Original host libraries are never patched.
Missing/colliding dependencies, excessive graphs and loader resolutions outside the package
refuse. Only safe existing origin-relative paths inside the package survive relocation;
glibc/its loader remain system dependencies. No global LD_LIBRARY_PATH or Docker/MCP/user
configuration is changed. License files/provenance are copied where available; complete
redistribution/source-obligation audit and reproducible source builds remain release gates.

## Acceptance and diagnosis

Jobs install the actual archive under an owned temporary root with empty PATH, verify
manifest hashes and contained symlinks, import the private helper modules, execute fixed
helper CLIs and private D-Bus exchange, enumerate 110 tools/7 prompts/18 resources and
render/export real SVG/PNG. They exercise snapshots/restore, no-op, approval refusal, atomic
batch rollback and original-file preservation in both per-call and warm-shell modes.

Doctor validates native thin/fat Mach-O or ELF64 architecture, the matching manifest,
private runtime imports, helper/bus assets and Inkscape >=1.4. macOS also checks the vendor
GTK3 architecture and prebuilt Cocoa context module; Linux omits those Cocoa requirements
and reports managed GUI support false. Linux's ordinary D-Bus/socket connections retain
their existing explicitly opened-session behavior; `live_launch` remains macOS-only.
Standard `/usr/local/bin/inkscape` and `/usr/bin/inkscape` are fallback locations only inside
a complete Linux package; PATH selection still takes precedence.

Doctor uses private diagnostic profiles, starts no GUI/bus/effect, installs nothing and
checks no compiler on the end-user host. Header matching proves architecture only; readiness
is not native GUI, ABI, Undo/Redo or signing/notarization acceptance. No security settings
are bypassed. Native acceptance must use separately owned synthetic documents.

Local evidence: `migration/posix-builder-logic-comparison.json` (synthetic ELF replies, not
Linux execution), `migration/posix-package-ci-prepared-comparison.json` (prepared workflow),
`migration/package-stage15-build-comparison.json` (actual current-Mac archive).

Stage15's current binary also passes 657 source/packaged/coerced real STDIO observations against
owned authenticated loopback peers (`migration/live-stage15-regressions-comparison.json`).
Those synthetic IPC checks include live guards, reconnect, events, mutation records, viewport
and render. Explicit lost-reply/PNG encoding differences remain in each report; this evidence
does not replace native GUI/Undo/Redo acceptance or any actual Linux/Intel execution.

## Current local stage27 candidate

Stage27 includes the stage26 DOM no-op fix in a normal Rust release, without diagnostic
profiling. The actual archive passes cold/warm installation with complete-tree no-op mtime
checks, ten doctor profiles, nine notice checks, eleven launcher checks and sixteen exact
frozen discovery configurations. Its executable matches the fresh release's 122 edit scenario
validation. Helper/native entries remain byte-identical to stage23. Stage29 checks this exact
package with 122 scoped native checks and five paired live read-only runs (80 requests,
40 scene/selection/PNG comparisons, zero sampler errors). Current headless measurements
cover 520 requests. Historical results remain tied to their own packages; current results
are separate evidence. See RUST_MIGRATION_REPORT.md for scope and unresolved history issues.

From a checkout, setup.sh detects/sets native SDK/library/tool paths and invokes
scripts/build-local-package.sh. Rust 1.99.0 and native build prerequisites must exist; an
exactly pinned Python helper environment is reused or installed uv obtains its runtime/wheels.
The script diagnoses missing tools without changing system package managers or shell profiles.
No manual environment configuration is needed. Existing source builds/settings are preserved.
A ready archive needs only Inkscape and auto-detects itself; source compilation is unnecessary.

Local evidence uses a fresh source copy/empty target directory with existing host tools;
it does not establish a fresh-OS/uv-download setup. Foreign jobs include source setup and
empty-PATH launcher checks but remain unexecuted. See the migration report for current archive
hash, installation and remaining history/platform/license/measurement limits.

## Notice collection

`migration_package_notices.py` copies notices from native-host normal/build crate archives
whose SHA-256 matches Cargo.lock. It performs bounded reads without tar extraction and
preserves upstream license alternatives. `THIRD_PARTY_NOTICES.md` and
`LICENSE-INVENTORY.json` are covered by FILES.json. The missing rmcp 3.5.0 license is
supplied from the exact crate-recorded upstream commit with checked-in URL/hash provenance.
Rust standard-library notices are copied from the selected compiler; Python/wheel notices
remain with their runtime metadata. Prepared CI runs `migration_notices_acceptance.py`.

Stage20 has 92 crates/178 notice texts and 13 standard-library notices; its actual cold/warm
archive installation, doctor and 9 notice checks pass. Native component equivalence to
stage19 failed for the rebuilt bridge, whose LC_ID_DYLIB contains the output path and UUID
changes. Stage20 never received native GUI acceptance. Stage21 fixes the bridge install name:
two actual same-host different-directory builds match exactly and verify their signatures.
Fresh stage21 native acceptance passes; installation instructions now point to stage21.
Whole archive reproducibility and other-host ABI compatibility remain unverified.

This is a conservative inventory, not complete license clearance. Stage35 includes exact hash-verified GLib/D-Bus/runtime-gettext/PCRE2 license texts
and Homebrew receipt/formula metadata. Standalone CPython native dependency provenance,
modified native corresponding source
and relink obligations and other-host attribution remain open. The inventory explicitly
records `redistribution_audit_complete: false`; no package was published.

## STDIO request bound

Development uses INKSCAPE_MCP_MAX_REQUEST_BYTES for a per-line UTF-8 byte cap including
CR/newline. The positive integer override defaults to six times INKSCAPE_MCP_MAX_INPUT_BYTES
plus 1MiB. Oversized or unfinished overlong input ends the MCP connection before JSON parsing.
Normal requests retain their contract. This is a transport bound, not an all-response/RSS cap.
Stage36 predates this change; a fresh archive must be checked before delivery.
