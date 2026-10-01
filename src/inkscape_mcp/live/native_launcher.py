"""Build a GTK context module and a private, ad-hoc signed Inkscape launcher copy.

The vendor application is read-only. Hardened runtime rejects user GTK modules;
only the session-local executable copy is re-signed. Resources stay in the vendor bundle.
"""

from __future__ import annotations

import hashlib
import plistlib
import shutil
import subprocess
from pathlib import Path


def glib_prefix() -> Path | None:
    """Find Homebrew's public GIO headers without installing anything."""
    return next(
        (
            base
            for base in (Path("/opt/homebrew"), Path("/usr/local"))
            if (base / "include/glib-2.0/gio/gio.h").is_file()
            and (base / "lib/glib-2.0/include/glibconfig.h").is_file()
        ),
        None,
    )


def build_dependencies() -> dict[str, bool]:
    """Read-only prerequisite probe for the launcher and doctor."""
    return {
        "clang": shutil.which("clang") is not None,
        "codesign": shutil.which("codesign") is not None,
        "glib_headers": glib_prefix() is not None,
    }


def prepare_context_launcher(binary: Path, root: Path) -> tuple[Path, Path]:
    binary = binary.resolve()
    contents = binary.parent.parent
    resources = contents / "Resources"
    if binary.parent.name != "MacOS" or not (resources / "lib/libgtk-3.0.dylib").is_file():
        raise RuntimeError("context bridge requires the official GTK 3 Inkscape macOS bundle")
    clang = shutil.which("clang")
    codesign = shutil.which("codesign")
    includes = glib_prefix()
    if clang is None or codesign is None or includes is None:
        raise RuntimeError(
            "context bridge needs Apple command line tools and Homebrew glib headers"
        )
    source = Path(__file__).parent / "native/context.m"
    digest = hashlib.sha256(binary.read_bytes() + source.read_bytes()).hexdigest()
    directory = root / "context-bridge"
    executable = directory / "Inkscape.app/Contents/MacOS/inkscape"
    module = directory / "context.so"
    stamp = directory / "build.sha256"
    if (
        stamp.is_file()
        and stamp.read_text() == digest
        and executable.is_file()
        and module.is_file()
    ):
        return executable, module
    executable.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    shutil.copyfile(binary, executable)
    executable.chmod(0o700)
    link = executable.parent.parent / "Resources"
    if link.is_symlink():
        link.unlink()
    link.symlink_to(resources, target_is_directory=True)
    info = plistlib.loads((contents / "Info.plist").read_bytes())
    info["CFBundleIdentifier"] = "org.inkscape.Inkscape.MCPManaged"
    (executable.parent.parent / "Info.plist").write_bytes(plistlib.dumps(info))
    try:
        subprocess.run(
            [
                clang,
                "-Wall",
                "-Wextra",
                "-Werror",
                "-dynamiclib",
                "-framework",
                "Cocoa",
                "-undefined",
                "dynamic_lookup",
                f"-I{includes / 'include/glib-2.0'}",
                f"-I{includes / 'lib/glib-2.0/include'}",
                str(source),
                "-o",
                str(module),
            ],
            check=True,
            capture_output=True,
            timeout=60,
        )
        subprocess.run(
            [codesign, "--force", "--sign", "-", str(executable)],
            check=True,
            capture_output=True,
            timeout=30,
        )
    except (subprocess.SubprocessError, OSError) as exc:
        stamp.unlink(missing_ok=True)
        raise RuntimeError(
            "could not build the document context bridge; see launcher setup"
        ) from exc
    stamp.write_text(digest)
    return executable, module
