#!/usr/bin/env python3
"""Check native target selection and owned ELF relocation logic using synthetic tool replies."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import migration_build_macos_package as builder
from migration_probe import require, write


def exercise(root, scenario):
    output = root / "package"
    library = output / "libexec/inkscape-mcp"
    library.mkdir(parents=True)
    server = output / "bin/inkscape-mcp"
    server.parent.mkdir()
    server.write_bytes(b"\x7fELFserver")
    sources = root / "source"
    sources.mkdir()
    for name in ("dbus-daemon", "gdbus", "libfoo.so.1", "collision.so"):
        (sources / name).write_bytes(b"\x7fELF" + name.encode())
    before = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sources.iterdir()}
    rpaths = {}
    calls = []

    def command(*argv):
        calls.append(list(argv))
        if argv[0] == "getconf":
            return "glibc 2.39\n"
        if argv[0] == "dpkg-query":
            return "migration-fixture: " + argv[-1] + "\n"
        dest = Path(argv[-1])
        if argv[:2] == ("patchelf", "--print-needed"):
            return "libc.so.6\n" if dest.parent.name == "native-lib" else "libfoo.so.1\n"
        if argv[:2] == ("patchelf", "--print-rpath"):
            return "$ORIGIN:$ORIGIN/../../../../../../../../outside:/build/external\n"
        if argv[:2] == ("patchelf", "--set-rpath"):
            rpaths[dest] = argv[2]
            return ""
        if argv[0] == "ldd":
            if dest.parent.name == "native-lib":
                return "libc.so.6 => /lib/libc.so.6 (0x1)\n"
            if scenario == "missing":
                return "libfoo.so.1 => not found\n"
            dependency = (
                library / "native-lib/libfoo.so.1" if dest in rpaths else sources / "libfoo.so.1"
            )
            if scenario == "collision" and dest.name == "gdbus":
                dependency = sources / "collision.so"
            return f"libfoo.so.1 => {dependency} (0x2)\nlibc.so.6 => /lib/libc.so.6 (0x1)\n"
        raise RuntimeError("unexpected fixture build command: " + repr(argv))

    manifest = {}
    error = None
    with (
        patch.object(builder, "command", command),
        patch.object(builder.shutil, "which", lambda name: str(sources / name)),
    ):
        try:
            builder.bundle_elf(output, library, manifest)
        except RuntimeError as exception:
            error = str(exception)
    after = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sources.iterdir()}
    require(before == after, "builder changed original dependency bytes")
    if scenario == "valid":
        require(error is None, "valid synthetic ELF closure failed: " + str(error))
        require(len(manifest["elf_inputs"]) == 1, "dependency was not deduplicated")
        require(len(rpaths) == 4, "owned ELF entries were not all relocated")
        for path, rpath in rpaths.items():
            require("outside" not in rpath and "/build" not in rpath, "external RPATH retained")
            for part in rpath.split(":"):
                resolved = (path.parent / part.removeprefix("$ORIGIN").lstrip("/")).resolve()
                require(resolved.is_relative_to(output), "RPATH escaped owned package")
    elif scenario == "missing":
        require(error == "unresolved ELF dependency: libfoo.so.1", "missing dependency accepted")
    else:
        require(error == "colliding ELF dependency names: libfoo.so.1", "collision accepted")
    return {
        "scenario": scenario,
        "passed": True,
        "error": error,
        "source_bytes_preserved": True,
        "mock_commands": calls,
        "manifest": manifest,
    }


def macho_paths(root):
    source = root / "bin/dbus-daemon"
    source.parent.mkdir(parents=True)
    source.write_bytes(b"fixture")
    library = root / "lib/libdbus.dylib"
    library.parent.mkdir()
    library.write_bytes(b"fixture")
    alternate = root / "other/libdbus.dylib"
    alternate.parent.mkdir()
    alternate.write_bytes(b"other")
    for rpath in (str(library.parent), "@loader_path/../lib"):
        with patch.object(
            builder, "command", return_value=f"cmd LC_RPATH\ncmdsize 48\npath {rpath} (offset 12)\n"
        ):
            require(
                builder.resolve_macho_dependency(source, "@rpath/libdbus.dylib") == library,
                "RPATH resolved incorrectly",
            )
    require(
        builder.resolve_macho_dependency(source, "@loader_path/../lib/libdbus.dylib") == library,
        "loader path resolved incorrectly",
    )
    for reply in (
        "",
        (
            f"cmd LC_RPATH\ncmdsize 48\npath {library.parent} (offset 12)\n"
            f"cmd LC_RPATH\ncmdsize 48\npath {alternate.parent} (offset 12)\n"
        ),
    ):
        with patch.object(builder, "command", return_value=reply):
            try:
                builder.resolve_macho_dependency(source, "@rpath/libdbus.dylib")
            except RuntimeError:
                continue
            raise RuntimeError("missing/ambiguous RPATH accepted")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    selections = []
    for host, arch, expected in [
        ("darwin", "arm64", "macos-arm64"),
        ("darwin", "x86_64", "macos-x86_64"),
        ("linux", "aarch64", "linux-arm64"),
        ("linux", "x86_64", "linux-x86_64"),
    ]:
        require(builder.native_target(host, arch) == expected, "wrong native target")
        selections.append(expected)
    for host, arch in [("win32", "x86_64"), ("linux", "riscv64")]:
        try:
            builder.native_target(host, arch)
        except RuntimeError:
            continue
        raise RuntimeError("unsupported native target accepted")
    results = []
    with TemporaryDirectory(prefix="imcp-elf-logic-") as temp:
        macho_paths(Path(temp).resolve() / "macho")
        for scenario in ("valid", "missing", "collision"):
            root = Path(temp).resolve() / scenario
            root.mkdir()
            results.append(exercise(root, scenario))
    write(args.output / "raw.json", results)
    write(
        args.report,
        {
            "passed": True,
            "target_selections": selections,
            "unsupported_targets_refused": 2,
            "synthetic_ELF_cases": len(results),
            "synthetic_MachO_path_cases": 5,
            "actual_Linux_execution": False,
            "native_GUI_launched": False,
            "scope": "Synthetic command replies only; not Linux ABI/loader acceptance",
            "builder_sha256": hashlib.sha256(Path(builder.__file__).read_bytes()).hexdigest(),
            "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        },
    )
    print("POSIX builder logic: four targets, two refusals, three synthetic ELF cases passed")


if __name__ == "__main__":
    main()
