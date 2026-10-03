#!/usr/bin/env python3
"""Build an owned native POSIX bundle from pinned development inputs; never publish."""

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
from importlib.metadata import distribution
from pathlib import Path

if __package__:
    from .migration_package_notices import collect_notices
else:
    from migration_package_notices import collect_notices

DEPS = ("lxml", "numpy", "cssselect", "tinycss2", "webencodings", "pillow")
HELPERS = {
    "helper_extension/inkscape_mcp_insert.py": "inkscape_mcp_insert.py",
    "helper_extension/inkscape_mcp_insert.inx": "inkscape_mcp_insert.inx",
    "helper_extension/inkscape_mcp_edit.py": "inkscape_mcp_edit.py",
    "helper_extension/inkscape_mcp_edit.inx": "inkscape_mcp_edit.inx",
    "helper_extension/inkscape_mcp_live.py": "inkscape_mcp_live.py",
    "helper_extension/inkscape_mcp_live.inx": "inkscape_mcp_live.inx",
    "insert_payload.py": "inkscape_mcp_insert_payload.py",
    "edit_errors.py": "inkscape_mcp_edit_errors.py",
}


def command(*args):
    return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT, timeout=120)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_context_bridge(source, headers, destination):
    # A stable install name avoids embedding the developer's output directory.
    argv = (
        "/usr/bin/clang",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-dynamiclib",
        "-framework",
        "Cocoa",
        "-undefined",
        "dynamic_lookup",
        "-Wl,-install_name,@rpath/inkscape-mcp-context.so",
        "-I" + str(headers / "include/glib-2.0"),
        "-I" + str(headers / "lib/glib-2.0/include"),
        str(source),
        "-o",
        str(destination),
    )
    command(*argv)
    return {
        "source_sha256": sha(source),
        "compiler_command": list(argv),
        "install_name": "@rpath/inkscape-mcp-context.so",
        "sha256": sha(destination),
    }


def dependencies(path):
    return [
        line.strip().split(" (compatibility", 1)[0]
        for line in command("/usr/bin/otool", "-L", str(path)).splitlines()[1:]
    ]


def system(dep):
    return dep.startswith(("/usr/lib/", "/System/Library/"))


def copy(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target, follow_symlinks=True)


def bundle_dbus(library, manifest):
    root = library / "dbus"
    mapping, queue = {}, []
    for name in ("dbus-daemon", "gdbus"):
        source = Path(shutil.which(name) or "").resolve()
        if not source.is_file():
            raise RuntimeError("development D-Bus executable missing: " + name)
        dest = root / "bin" / name
        copy(source, dest)
        mapping[source] = dest
        queue.append(source)
    graph = {}
    while queue:
        source = queue.pop(0)
        deps = dependencies(source)
        graph[source] = deps
        for dep in deps:
            if system(dep):
                continue
            if not dep.startswith("/"):
                raise RuntimeError("unresolved development D-Bus dependency: " + dep)
            target = Path(dep).resolve()
            if target == source or target in mapping:
                continue
            if (
                len(mapping) >= 64
                or not target.is_file()
                or target.stat().st_size > 64 * 1024 * 1024
            ):
                raise RuntimeError("invalid D-Bus dependency graph")
            dest = root / "lib" / target.name
            if dest in mapping.values():
                raise RuntimeError("colliding D-Bus library names")
            copy(target, dest)
            mapping[target] = dest
            queue.append(target)
    for source, dest in mapping.items():
        if dest.suffix == ".dylib":
            command("/usr/bin/install_name_tool", "-id", "@loader_path/" + dest.name, str(dest))
        for dep in graph[source]:
            if system(dep):
                continue
            target = Path(dep).resolve()
            relative = os.path.relpath(mapping[target], dest.parent)
            command(
                "/usr/bin/install_name_tool", "-change", dep, "@loader_path/" + relative, str(dest)
            )
        command("/usr/bin/codesign", "--force", "--sign", "-", str(dest))
        deps = dependencies(dest)
        if any(not system(dep) and not dep.startswith("@loader_path/") for dep in deps):
            raise RuntimeError("D-Bus relocation left external dependency")
        manifest["dbus_inputs"].append(
            {
                "source": str(source),
                "source_sha256": sha(source),
                "output": str(dest.relative_to(library)),
                "dependencies": deps,
            }
        )
    (root / "lib/gio/modules").mkdir(parents=True)
    copy(Path("rust/package/session.conf"), root / "session.conf")
    licenses = library / "licenses/dbus"
    for source in mapping:
        # Homebrew Cellar metadata/licenses are development provenance only.
        formula = next(
            (parent for parent in source.parents if (parent / "INSTALL_RECEIPT.json").is_file()),
            None,
        )
        if formula:
            for name in ("COPYING", "COPYING.LIB", "LICENSE", "AUTHORS", "sbom.spdx.json"):
                if (formula / name).is_file():
                    copy(formula / name, licenses / formula.parent.name / name)
            for name in ("INSTALL_RECEIPT.json", ".brew/" + formula.parent.name + ".rb"):
                if (formula / name).is_file():
                    copy(formula / name, licenses / formula.parent.name / "build-metadata" / name)


def native_target(system_name, machine):
    architectures = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x86_64"}
    if system_name not in ("darwin", "linux") or machine not in architectures:
        raise RuntimeError("native POSIX builder supports macOS/Linux x86_64 and arm64 only")
    return ("macos" if system_name == "darwin" else "linux") + "-" + architectures[machine]


def bundle_elf(output, library, manifest):
    """Relocate non-glibc ELF dependency closure; never patch original host files."""
    system_names = {
        "libc.so.6",
        "libm.so.6",
        "libdl.so.2",
        "libpthread.so.0",
        "librt.so.1",
        "libresolv.so.2",
        "libutil.so.1",
        "ld-linux-x86-64.so.2",
        "ld-linux-aarch64.so.1",
    }
    native = library / "native-lib"
    native.mkdir()
    for name in ("dbus-daemon", "gdbus"):
        source = shutil.which(name)
        if not source:
            raise RuntimeError("development D-Bus executable missing: " + name)
        copy(Path(source), library / "dbus/bin" / name)
    copy(Path("rust/package/session.conf"), library / "dbus/session.conf")
    (library / "dbus/lib/gio/modules").mkdir(parents=True)
    queue = []
    for path in sorted(output.rglob("*")):
        if path.is_file() and not path.is_symlink():
            with path.open("rb") as stream:
                if stream.read(4) == b"\x7fELF":
                    queue.append(path)
    inputs = {}
    processed = set()
    while queue:
        dest = queue.pop(0)
        if dest in processed:
            continue
        if len(processed) >= 512:
            raise RuntimeError("ELF dependency graph exceeds 512 files")
        processed.add(dest)
        needed = command("patchelf", "--print-needed", str(dest)).splitlines()
        resolved = {}
        for line in command("ldd", str(dest)).splitlines():
            match = re.match(r"\s*(\S+)\s+=>\s+(\S+)\s+\(", line)
            if match:
                resolved[match[1]] = Path(match[2]).resolve()
        for name in needed:
            if name in system_names:
                continue
            if "/" in name or name not in resolved:
                raise RuntimeError("unresolved ELF dependency: " + name)
            source = resolved[name]
            if source.is_relative_to(output):
                continue
            if not source.is_file() or source.stat().st_size > 64 * 1024 * 1024:
                raise RuntimeError("invalid ELF dependency")
            target = native / name
            if name in inputs and inputs[name]["source_sha256"] != sha(source):
                raise RuntimeError("colliding ELF dependency names: " + name)
            if name not in inputs:
                if len(inputs) >= 64:
                    raise RuntimeError("ELF external dependency graph exceeds 64 files")
                copy(source, target)
                inputs[name] = {
                    "source": str(source),
                    "source_sha256": sha(source),
                    "output": str(target.relative_to(library)),
                }
                queue.append(target)
                # Debian/Ubuntu license provenance. Source-package redistribution audit
                # remains a release gate; do not infer compliance merely from a file copy.
                owner = command("dpkg-query", "-S", str(source)).splitlines()
                for entry in owner:
                    package = entry.split(": ", 1)[0].split(":", 1)[0]
                    copyright_file = Path("/usr/share/doc") / package / "copyright"
                    if copyright_file.is_file():
                        copy(copyright_file, library / "licenses/linux" / package / "copyright")
        original = command("patchelf", "--print-rpath", str(dest)).strip()
        origin = "$ORIGIN/" + os.path.relpath(native, dest.parent)
        retained = []
        for part in original.split(":"):
            if part == "$ORIGIN" or part.startswith("$ORIGIN/"):
                relative = part.removeprefix("$ORIGIN").lstrip("/")
                if (dest.parent / relative).resolve().is_relative_to(output):
                    retained.append(part)
        command("patchelf", "--set-rpath", ":".join(dict.fromkeys([origin, *retained])), str(dest))
    # Verify actual loader resolution after relocation, not merely recorded DT_NEEDED names.
    for dest in processed:
        text = command("ldd", str(dest))
        if "not found" in text:
            raise RuntimeError("relocated ELF dependency is missing")
        for line in text.splitlines():
            match = re.match(r"\s*(\S+)\s+=>\s+(\S+)\s+\(", line)
            if match and match[1] not in system_names:
                if not Path(match[2]).resolve().is_relative_to(output):
                    raise RuntimeError("relocated ELF dependency escaped package: " + match[1])
    manifest["elf_inputs"] = list(inputs.values())
    manifest["system_elf_dependencies"] = sorted(system_names)
    manifest["glibc_build_version"] = command("getconf", "GNU_LIBC_VERSION").strip()


def build(output, binary=None):
    target = native_target(sys.platform, platform.machine())
    if sys.platform == "linux" and not command("getconf", "GNU_LIBC_VERSION").startswith("glibc "):
        raise RuntimeError("Linux packages require a GNU/glibc build host")
    output.mkdir(parents=True, exist_ok=False)
    output = output.resolve()
    library = output / "libexec/inkscape-mcp"
    runtime = library / "python"
    runtime.mkdir(parents=True)
    base = Path(sys.base_prefix)
    if sys.version_info[:2] != (3, 12):
        raise RuntimeError("pinned helper Python 3.12 required on development host")
    copy(base / "bin/python3.12", runtime / "bin/python3")
    shutil.copytree(
        base / "lib",
        runtime / "lib",
        symlinks=True,
        ignore=shutil.ignore_patterns("site-packages", "__pycache__", "*.pyc"),
    )
    site = runtime / "lib/python3.12/site-packages"
    site.mkdir(parents=True)
    manifest = {
        "target": target,
        "python": sys.version,
        "runtime_source": str(base),
        "python_deps": [],
        "dbus_inputs": [],
        "release_signed": False,
        "notarized": False,
        "source_head": command("git", "rev-parse", "HEAD").strip(),
    }
    for name in DEPS:
        dist = distribution(name)
        for relative in dist.files or []:
            if (
                ".." in relative.parts
                or relative.suffix == ".pyc"
                or "__pycache__" in relative.parts
            ):
                continue
            source = Path(dist.locate_file(relative))
            if source.is_file():
                copy(source, site / str(relative))
        manifest["python_deps"].append({"name": name, "version": dist.version})
    source = Path("runtime")
    for relative, name in HELPERS.items():
        copy(source / relative, library / "helpers" / name)
    copy(Path("rust/package/supervise.py"), library / "supervise.py")
    server_binary = binary or Path("rust/target/release/inkscape-mcp-rust")
    copy(server_binary, output / "bin/inkscape-mcp")
    if sys.platform == "darwin":
        # Cargo consolidates DWARF before temporary LTO objects are removed.
        # Keep its UUID-matched bundle beside the relocated executable so the
        # backtrace integration can resolve lines without the build directory.
        symbols = output / "bin/inkscape-mcp.dSYM"
        source_symbols = Path(str(server_binary) + ".dSYM")
        if not source_symbols.is_dir():
            raise RuntimeError("Rust release dSYM missing; build with split-debuginfo=packed")
        shutil.copytree(source_symbols, symbols)
        manifest["server_debug_symbols"] = "bin/inkscape-mcp.dSYM"
    for launcher in ("setup.sh", "run-mcp.sh"):
        copy(Path(launcher), output / launcher)
    headers = next(
        (
            p
            for p in (Path("/opt/homebrew"), Path("/usr/local"))
            if (p / "include/glib-2.0/gio/gio.h").is_file()
        ),
        None,
    )
    if sys.platform == "darwin" and headers is None:
        raise RuntimeError("development GLib headers required to build bridge")
    if sys.platform == "darwin":
        manifest["context_bridge_build"] = build_context_bridge(
            source / "native/context.m", headers, library / "context.so"
        )
        bundle_dbus(library, manifest)
    else:
        bundle_elf(output, library, manifest)
    copy(Path("LICENSE"), output / "LICENSE")
    manifest["license_inventory"] = collect_notices(output, target)
    (library / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    files = {}
    for path in sorted(output.rglob("*")):
        if path.is_file() and not path.is_symlink():
            files[str(path.relative_to(output))] = {
                "sha256": sha(path),
                "bytes": path.stat().st_size,
            }
    (output / "FILES.json").write_text(json.dumps(files, indent=2) + "\n")
    print(
        json.dumps(
            {
                "package": str(output),
                "files": len(files),
                "bytes": sum(row["bytes"] for row in files.values()),
            }
        )
    )


def build_and_archive(output, archive=None, binary=None):
    if archive:
        archive = archive.absolute()
        if archive.exists() or output.absolute() in archive.parents:
            raise RuntimeError("archive must be new and outside the package tree")
    build(output.absolute(), binary)
    if archive:
        with tarfile.open(archive, "x:gz") as stream:
            stream.add(output.absolute(), arcname=output.name)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    build_and_archive(args.output, args.archive, args.binary)
