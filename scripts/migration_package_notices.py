"""Collect bounded, hash-verified notices for a native developer-built package."""

import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
import tomllib
from pathlib import Path, PurePosixPath

PREFIXES = ("license", "copying", "copyright", "notice", "unlicense")
TRIPLES = {
    "macos-arm64": "aarch64-apple-darwin",
    "macos-x86_64": "x86_64-apple-darwin",
    "linux-arm64": "aarch64-unknown-linux-gnu",
    "linux-x86_64": "x86_64-unknown-linux-gnu",
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def archive_notices(archive, checksum, name, version):
    if archive.stat().st_size > 64 * 1024 * 1024:
        raise RuntimeError("crate archive exceeds notice input cap")
    if sha(archive.read_bytes()) != checksum:
        raise RuntimeError("crate archive differs from Cargo.lock: " + name)
    prefix = f"{name}-{version}/"
    notices = {}
    total = 0
    with tarfile.open(archive, "r:gz") as stream:
        for index, member in enumerate(stream):
            if index >= 100000:
                raise RuntimeError("crate archive member cap exceeded")
            if not member.name.startswith(prefix):
                raise RuntimeError("unexpected crate archive root")
            relative = PurePosixPath(member.name[len(prefix) :])
            if relative.is_absolute() or ".." in relative.parts:
                raise RuntimeError("unsafe crate archive path")
            selected = any(part.lower().startswith(PREFIXES) for part in relative.parts)
            if not selected and str(relative) not in ("Cargo.toml", ".cargo_vcs_info.json"):
                continue
            if not member.isfile() or member.size > 4 * 1024 * 1024:
                raise RuntimeError("unsafe or excessive crate notice")
            total += member.size
            if total > 16 * 1024 * 1024:
                raise RuntimeError("crate notices exceed total cap")
            source = stream.extractfile(member)
            if source is None:
                raise RuntimeError("crate notice unavailable")
            content = source.read(member.size + 1)
            if len(content) != member.size or str(relative) in notices:
                raise RuntimeError("incomplete or duplicate crate notice")
            notices[str(relative)] = content
    metadata = tomllib.loads(notices.pop("Cargo.toml").decode())["package"]
    vcs = json.loads(notices.pop(".cargo_vcs_info.json", b"{}"))
    return metadata, vcs, notices


def native_source_notices(output, copy_notice, vendor=Path("migration/vendor-notices/native")):
    """Supplement installed SBOMs only from the exact reviewed source version/hash."""
    rows, gaps = [], []
    native = output / "libexec/inkscape-mcp/licenses/dbus"
    for sbom_path in sorted(native.glob("*/sbom.spdx.json")):
        name = sbom_path.parent.name
        sbom = json.loads(sbom_path.read_text())
        source = next(
            (p for p in sbom["packages"] if p["SPDXID"] == f"SPDXRef-Archive-{name}-src"),
            None,
        )
        if source is None:
            gaps.append("missing native source SBOM: " + name)
            continue
        version = source["versionInfo"]
        root = vendor / name / version
        if not (root / "provenance.json").is_file():
            gaps.append("missing exact native source notice supplement: " + name + " " + version)
            continue
        provenance = json.loads((root / "provenance.json").read_text())
        checksums = [c["checksumValue"] for c in source["checksums"] if c["algorithm"] == "SHA256"]
        if (
            provenance["name"] != name
            or provenance["version"] != version
            or checksums != [provenance["sha256"]]
            or provenance["url"] != source["downloadLocation"]
        ):
            raise RuntimeError("native source notice provenance differs: " + name)
        notices = []
        for notice in provenance["notices"]:
            relative = PurePosixPath(notice["path"])
            if relative.is_absolute() or ".." in relative.parts:
                raise RuntimeError("unsafe native source notice path")
            path = root / str(relative)
            if path.is_symlink() or not path.is_file() or path.stat().st_size > 4 * 1024 * 1024:
                raise RuntimeError("missing or unsafe native source notice")
            content = path.read_bytes()
            if len(content) != notice["bytes"] or sha(content) != notice["sha256"]:
                raise RuntimeError("native source notice hash differs: " + name)
            notices.append(copy_notice(content, Path("native-source") / name / version / relative))
        if not notices:
            raise RuntimeError("empty native source notice supplement")
        notices.append(
            copy_notice(
                (root / "provenance.json").read_bytes(),
                Path("native-source") / name / version / "provenance.json",
            )
        )
        rows.append({"source": provenance, "notices": notices})
    return rows, gaps


def cpython_notices(output, target, copy_notice):
    root = Path("migration/vendor-notices/cpython/3.12.14-20260929-macos-arm64")
    if target != "macos-arm64":
        return None, ["Exact private CPython release attribution unavailable for " + target]
    provenance = json.loads((root / "provenance.json").read_text())
    executable = output / "libexec/inkscape-mcp/python/bin/python3"
    if sha(executable.read_bytes()) != provenance["executable_sha256"]:
        return None, ["Private CPython binary does not match reviewed Astral release"]
    notices = []
    for notice in provenance["notices"]:
        relative = PurePosixPath(notice["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise RuntimeError("unsafe CPython source notice path")
        path = root / str(relative)
        if path.is_symlink() or not path.is_file() or path.stat().st_size > 4 * 1024 * 1024:
            raise RuntimeError("unsafe or missing CPython source notice")
        content = path.read_bytes()
        if len(content) != notice["bytes"] or sha(content) != notice["sha256"]:
            raise RuntimeError("CPython source notice hash differs")
        notices.append(copy_notice(content, Path("cpython-source") / relative))
    notices.append(
        copy_notice((root / "provenance.json").read_bytes(), Path("cpython-source/provenance.json"))
    )
    return {"source": provenance, "notices": notices}, []


def wheel_provenance(output, target):
    if target != "macos-arm64":
        return None, ["Exact helper wheel attribution unavailable for " + target]
    lock = json.loads(Path("rust/package/helper-wheel-provenance-macos-arm64.json").read_text())
    site = output / "libexec/inkscape-mcp/python/lib/python3.12/site-packages"
    for row in lock["wheels"]:
        for member in row["members"]:
            relative = PurePosixPath(member["path"])
            if relative.is_absolute() or ".." in relative.parts:
                raise RuntimeError("unsafe locked wheel member path")
            path = site / str(relative)
            if path.is_symlink() or not path.is_file() or path.stat().st_size > 64 * 1024 * 1024:
                raise RuntimeError("missing or unsafe locked wheel member")
            content = path.read_bytes()
            if len(content) != member["bytes"] or sha(content) != member["sha256"]:
                raise RuntimeError("helper wheel member differs from reviewed archive")
    return lock, []


def glib_build_notices(output, target, copy_notice):
    root = Path("migration/vendor-notices/native-build/glib/2.90.0")
    recipe = output / "libexec/inkscape-mcp/licenses/dbus/glib/build-metadata/.brew/glib.rb"
    if not target.startswith("macos") or not recipe.is_file():
        return None, ["Exact native GLib build metadata unavailable for " + target]
    provenance = json.loads((root / "provenance.json").read_text())
    if sha(recipe.read_bytes()) != provenance["formula_sha256"]:
        return None, ["Native GLib recipe differs from reviewed bottle"]
    notices = []
    for row in provenance["files"]:
        path = root / row["path"]
        content = path.read_bytes()
        if len(content) != row["bytes"] or sha(content) != row["sha256"]:
            raise RuntimeError("native build source metadata differs")
        notices.append(copy_notice(content, Path("native-build/glib/2.90.0") / row["path"]))
    notices.append(
        copy_notice(
            (root / "provenance.json").read_bytes(),
            Path("native-build/glib/2.90.0/provenance.json"),
        )
    )
    return {"source": provenance, "notices": notices}, []


def collect_notices(output, target):
    cargo = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")
    tree_args = [
        cargo,
        "tree",
        "--locked",
        "--offline",
        "--manifest-path",
        "rust/Cargo.toml",
        "--target",
        TRIPLES[target],
        "--prefix",
        "none",
        "--edges",
        "normal,build",
        "--format",
        "{p}",
    ]
    graph = subprocess.check_output(tree_args, text=True, timeout=60)
    pairs = sorted(
        {
            match.groups()
            for line in graph.splitlines()
            if (match := re.match(r"([\w-]+) v([\w.+-]+)", line))
            and match[1] != "inkscape-mcp-rust"
        }
    )
    if not pairs or len(pairs) > 512:
        raise RuntimeError("unexpected native dependency graph size")
    lock = tomllib.loads(Path("rust/Cargo.lock").read_text())
    locked = {(p["name"], p["version"]): p for p in lock["package"]}
    cargo_root = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    destination = output / "libexec/inkscape-mcp/licenses"
    rows, gaps = [], []

    def copy_notice(content, relative):
        path = destination / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.exists():
            raise RuntimeError("notice output collision")
        path.write_bytes(content)
        return {
            "path": str(path.relative_to(output)),
            "bytes": len(content),
            "sha256": sha(content),
        }

    for name, version in pairs:
        package = locked[(name, version)]
        if package.get("source") != "registry+https://github.com/rust-lang/crates.io-index":
            raise RuntimeError("unsupported crate notice source")
        archives = list((cargo_root / "registry/cache").glob(f"*/{name}-{version}.crate"))
        if len(archives) != 1:
            raise RuntimeError("missing/ambiguous locked crate archive: " + name)
        metadata, vcs, notices = archive_notices(archives[0], package["checksum"], name, version)
        upstream = None
        if not notices and (name, version) == ("rmcp", "3.5.0"):
            root = Path("migration/vendor-notices/rmcp-3.5.0")
            upstream = json.loads((root / "provenance.json").read_text())
            content = (root / "LICENSE").read_bytes()
            if (
                vcs.get("git", {}).get("sha1") != upstream["commit"]
                or sha(content) != upstream["sha256"]
            ):
                raise RuntimeError("rmcp upstream notice provenance differs")
            notices["LICENSE"] = content
        if not notices:
            gaps.append("missing crate license text: " + name + " " + version)
        rows.append(
            {
                "name": name,
                "version": version,
                "source": package["source"],
                "archive_sha256": package["checksum"],
                "license_expression": metadata.get("license"),
                "repository": metadata.get("repository"),
                "upstream_notice": upstream,
                "notices": [
                    copy_notice(content, Path("rust") / f"{name}-{version}" / path)
                    for path, content in sorted(notices.items())
                ],
            }
        )
    rustc = shutil.which("rustc") or str(Path.home() / ".cargo/bin/rustc")
    sysroot = Path(subprocess.check_output([rustc, "--print", "sysroot"], text=True).strip())
    rust_docs = sysroot / "share/doc/rust"
    standard = [
        rust_docs / "COPYRIGHT-library.html",
        *sorted((rust_docs / "licenses").glob("*.txt")),
    ]
    std_rows = []
    for path in standard:
        if not path.is_file() or path.stat().st_size > 4 * 1024 * 1024:
            raise RuntimeError("Rust standard-library notice unavailable or excessive")
        std_rows.append(copy_notice(path.read_bytes(), Path("rust-standard-library") / path.name))
    site = output / "libexec/inkscape-mcp/python/lib/python3.12/site-packages"
    wheel_notices = [
        str(p.relative_to(output))
        for p in sorted(site.rglob("*"))
        if p.is_file()
        and any(part.lower().startswith(PREFIXES) for part in p.relative_to(site).parts)
    ]
    native_sources, native_gaps = native_source_notices(output, copy_notice)
    gaps.extend(native_gaps)
    python_source, python_gaps = cpython_notices(output, target, copy_notice)
    gaps.extend(python_gaps)
    wheels, wheel_gaps = wheel_provenance(output, target)
    gaps.extend(wheel_gaps)
    glib_build, glib_gaps = glib_build_notices(output, target, copy_notice)
    gaps.extend(glib_gaps)
    native_notices = [
        str(p.relative_to(output))
        for p in sorted(destination.rglob("*"))
        if p.is_file()
        and p.relative_to(destination).parts[0] not in ("rust", "rust-standard-library")
    ]
    # Inventory evidence must not be mistaken for complete binary-source/license clearance.
    gaps.extend(
        [
            "Private CPython native dependency/source provenance needs audit.",
            "Native bus/GLib/gettext source obligations and referenced license texts need audit.",
            "Foreign target and exact linked-vs-build dependency attribution are not established.",
        ]
    )
    inventory = {
        "target": target,
        "cargo_tree_command": tree_args,
        "cargo_tree": graph,
        "cargo_lock_sha256": sha(Path("rust/Cargo.lock").read_bytes()),
        "rust_crates": rows,
        "rust_standard_library_notices": std_rows,
        "python_wheel_notice_paths": wheel_notices,
        "native_notice_paths": native_notices,
        "native_source_supplements": native_sources,
        "cpython_source_supplement": python_source,
        "helper_wheel_provenance": wheels,
        "glib_build_source_supplement": glib_build,
        "redistribution_audit_complete": False,
        "remaining_gaps": gaps,
    }
    (output / "LICENSE-INVENTORY.json").write_text(json.dumps(inventory, indent=2) + "\n")
    lines = [
        "# Third-party notices",
        "",
        "The server's MIT license is in LICENSE. Third-party terms remain separate.",
        "Native-target Cargo normal/build dependency notices are included conservatively;",
        "this inventory does not claim every listed crate is linked into the binary.",
        "Rust standard-library notices are in libexec/inkscape-mcp/licenses/rust-standard-library.",
        "Python's license is in libexec/inkscape-mcp/python/lib/python3.12/LICENSE.txt.",
        "Wheel notices remain beside their dist-info metadata. Native bus notices and SBOMs",
        "are under libexec/inkscape-mcp/licenses. LICENSE-INVENTORY.json records source hashes",
        "and remaining audit gaps. No license alternatives are silently reselected.",
        "",
        "| Crate | Version | Declared license |",
        "|---|---|---|",
    ]
    lines.extend(f"| {r['name']} | {r['version']} | {r['license_expression']} |" for r in rows)
    (output / "THIRD_PARTY_NOTICES.md").write_text("\n".join(lines) + "\n")
    return {
        "inventory": "LICENSE-INVENTORY.json",
        "rust_crates": len(rows),
        "rust_notice_files": sum(len(r["notices"]) for r in rows),
        "redistribution_audit_complete": False,
    }
