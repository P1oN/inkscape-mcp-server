"""The module build changes only a private launcher copy and ships its C source."""

import plistlib
import sys
from pathlib import Path

import pytest

from inkscape_mcp.live import native_launcher


@pytest.mark.skipif(sys.platform == "win32", reason="macOS bundle symlink layout")
def test_private_copy_build_and_source_cache(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    contents = tmp_path / "vendor/Inkscape.app/Contents"
    binary = contents / "MacOS/inkscape"
    binary.parent.mkdir(parents=True)
    binary.write_bytes(b"vendor executable")
    (contents / "Info.plist").write_bytes(plistlib.dumps({"CFBundleIdentifier": "org.inkscape"}))
    resources = contents / "Resources"
    (resources / "lib").mkdir(parents=True)
    (resources / "lib/libgtk-3.0.dylib").touch()
    root = tmp_path / "session"
    root.mkdir(mode=0o700)
    monkeypatch.setattr(native_launcher.shutil, "which", lambda name: name)
    monkeypatch.setattr(native_launcher, "glib_prefix", lambda: tmp_path / "headers")
    calls: list[list[str]] = []

    def run(argv: list[str], **kwargs: object) -> None:
        calls.append(argv)
        if argv[0] == "clang":
            Path(argv[-1]).write_bytes(b"compiled module")
        else:
            assert argv[0] == "codesign"
            assert Path(argv[-1]).is_relative_to(root)
            Path(argv[-1]).write_bytes(b"private signed executable")

    monkeypatch.setattr(native_launcher.subprocess, "run", run)
    executable, module = native_launcher.prepare_context_launcher(binary, root)
    assert binary.read_bytes() == b"vendor executable"
    assert executable.read_bytes() == b"private signed executable"
    assert module.is_file()
    assert (executable.parent.parent / "Resources").resolve() == resources
    assert len(calls) == 2
    assert native_launcher.prepare_context_launcher(binary, root) == (executable, module)
    assert len(calls) == 2
    binary.write_bytes(b"updated vendor executable")
    native_launcher.prepare_context_launcher(binary, root)
    assert len(calls) == 4


def test_unknown_bundle_is_never_copied_or_signed(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    binary = tmp_path / "inkscape"
    binary.write_bytes(b"not a vendor bundle")
    monkeypatch.setattr(native_launcher.subprocess, "run", lambda *a, **k: pytest.fail("no build"))
    with pytest.raises(RuntimeError, match="official GTK 3"):
        native_launcher.prepare_context_launcher(binary, tmp_path / "session")
    assert not (tmp_path / "session").exists()
