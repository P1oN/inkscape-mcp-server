#!/usr/bin/env python3
"""Verify the owned macOS CPython runtime against a pinned cached Astral release."""

import argparse
import ast
import hashlib
import json
import re
import subprocess
import tarfile
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_probe import require


def sha(content):
    return hashlib.sha256(content).hexdigest()


def config_values(content):
    tree = ast.parse(content)
    require(
        len(tree.body) == 1
        and isinstance(tree.body[0], ast.Assign)
        and len(tree.body[0].targets) == 1
        and isinstance(tree.body[0].targets[0], ast.Name)
        and tree.body[0].targets[0].id == "build_time_vars",
        "unexpected executable sysconfig statements",
    )
    return ast.literal_eval(tree.body[0].value)


def patched_config(original, prefix, mappings):
    """Reproduce the published uv sysconfig algorithm, without executing upstream code."""
    mapping = {}
    for key, block in re.findall(r'\("([^"]+)"\.to_string\(\), vec!\[(.*?)\]\)', mappings, re.S):
        partial = re.findall(
            r'ReplacementMode::Partial \{ from: "([^"]*)"\.to_string\(\) \}, '
            r'to: "([^"]*)"\.to_string\(\)',
            block,
        )
        full = re.findall(r'ReplacementMode::Full,\s*to: "([^"]*)"\.to_string\(\)', block)
        mapping[key] = (partial, full)
    expected = {}
    for key, value in original.items():
        if isinstance(value, str):
            words = [
                str(prefix)
                if p == "/install"
                else str(prefix / p[9:])
                if p.startswith("/install/")
                else p
                for p in value.split()
            ]
            result = []
            iterator = iter(words)
            for word in iterator:
                if word == "-isysroot":
                    next(iterator, None)
                else:
                    result.append(word)
            value = " ".join(result)
            partial, full = mapping.get(key, ([], []))
            for source, target in partial:
                value = " ".join(target if p == source else p for p in value.split())
            for target in full:
                value = target
        expected[key] = value
    expected["PYTHON_BUILD_STANDALONE"] = 1
    return expected


def verify(package, cache, runtime_origin, report):
    vendor = Path("migration/vendor-notices/cpython/3.12.14-20260929-macos-arm64")
    provenance = json.loads((vendor / "provenance.json").read_text())
    artifact = next(a for a in provenance["archives"] if "install_only_stripped" in a["name"])
    archive = cache / artifact["name"]
    require(archive.stat().st_size == artifact["size"], "CPython provenance check failed")
    require(
        "sha256:" + sha(archive.read_bytes()) == artifact["digest"],
        "CPython provenance check failed",
    )
    runtime = package / "libexec/inkscape-mcp/python"
    sysconfig = "lib/python3.12/_sysconfigdata__darwin_darwin.py"
    exact, transformed = [], []
    with TemporaryDirectory(prefix="imcp-cpython-verify-") as temporary:
        with tarfile.open(archive) as stream:
            for path in sorted(runtime.rglob("*")):
                if not path.is_file() or path.is_symlink() or "site-packages" in path.parts:
                    continue
                relative = str(path.relative_to(runtime))
                content = path.read_bytes()
                if relative == "lib/python3.12/EXTERNALLY-MANAGED":
                    require(
                        content
                        == (
                            b"[externally-managed]\nError=This Python installation is managed "
                            b"by uv "
                            b"and should not be modified.\n"
                        ),
                        "CPython provenance check failed",
                    )
                    transformed.append(relative)
                    continue
                member = stream.getmember(
                    "python/" + ("bin/python3.12" if relative == "bin/python3" else relative)
                )
                require(
                    member.isfile() and member.size <= 64 * 1024 * 1024,
                    "CPython provenance check failed",
                )
                upstream = stream.extractfile(member).read()
                if relative == "lib/libpython3.12.dylib":
                    owned = Path(temporary) / "libpython3.12.dylib"
                    owned.write_bytes(upstream)
                    subprocess.run(
                        [
                            "/usr/bin/install_name_tool",
                            "-id",
                            str(runtime_origin / relative),
                            str(owned),
                        ],
                        check=True,
                    )
                    require(content == owned.read_bytes(), "libpython relocation differs")
                    transformed.append(relative)
                elif relative == sysconfig:
                    original = config_values(upstream)
                    actual = config_values(content)
                    require(
                        actual
                        == patched_config(
                            original,
                            runtime_origin,
                            (cache / "uv-generated_mappings.rs").read_text(),
                        ),
                        "sysconfig differs beyond published uv transformation",
                    )
                    transformed.append(relative)
                else:
                    require(content == upstream, "runtime member differs: " + relative)
                    exact.append(relative)
    result = {
        "passed": True,
        "package": str(package.resolve()),
        "executable_sha256": sha((runtime / "bin/python3").read_bytes()),
        "release_archive": artifact,
        "exact_members": exact,
        "explicit_transformations": transformed,
        "scope": "Runtime outside site-packages; exact byte comparison, exact libpython "
        "install-name reproduction and all sysconfig values against published uv rules. "
        "No foreign target or wheel provenance claim.",
    }
    require(
        result["executable_sha256"] == provenance["executable_sha256"],
        "CPython provenance check failed",
    )
    report.parent.mkdir(parents=True, exist_ok=True)
    if report.exists():
        raise RuntimeError("refuse to overwrite evidence report")
    report.write_text(json.dumps(result, indent=2) + "\n")
    print(f"CPython provenance passed: {len(exact)} exact members, {len(transformed)} transforms")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--cache", required=True, type=Path)
    parser.add_argument("--runtime-origin", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    verify(args.package, args.cache, args.runtime_origin, args.report)
