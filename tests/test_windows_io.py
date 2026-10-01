"""Native Windows handle safety; exercised by the Windows headless CI job."""

from __future__ import annotations

import errno
import os
import sys
from pathlib import Path

import pytest

from inkscape_mcp.workspace.windows_io import mkdir_chain, write_fd

pytestmark = pytest.mark.skipif(sys.platform != "win32", reason="requires Windows handles")


def test_nested_directories_and_overwrite(tmp_path: Path) -> None:
    mkdir_chain(tmp_path, ("nested", "child"))
    dest = tmp_path / "nested" / "child" / "out.svg"
    dest.write_bytes(b"original longer content")
    with write_fd(dest) as fd:
        os.write(fd, b"new")
    assert dest.read_bytes() == b"new"


def test_exclusive_write_preserves_existing_file(tmp_path: Path) -> None:
    dest = tmp_path / "out.svg"
    dest.write_bytes(b"original")
    with pytest.raises(OSError), write_fd(dest, exclusive=True):
        pytest.fail("existing destination was opened")
    assert dest.read_bytes() == b"original"


def test_symlink_write_refused_before_truncation(tmp_path: Path) -> None:
    target = tmp_path / "target.svg"
    target.write_bytes(b"original")
    link = tmp_path / "link.svg"
    link.symlink_to(target)
    with pytest.raises(OSError) as error, write_fd(link):
        pytest.fail("symlink was opened for writing")
    assert error.value.errno == errno.ELOOP
    assert target.read_bytes() == b"original"


def test_directory_symlink_refused_before_descending(tmp_path: Path) -> None:
    outside = tmp_path / "outside"
    outside.mkdir()
    base = tmp_path / "base"
    base.mkdir()
    (base / "link").symlink_to(outside, target_is_directory=True)
    with pytest.raises(OSError):
        mkdir_chain(base, ("link", "child"))
    assert not (outside / "child").exists()


def test_open_parent_cannot_be_renamed_during_write(tmp_path: Path) -> None:
    parent = tmp_path / "parent"
    parent.mkdir()
    with write_fd(parent / "out.svg") as fd:
        with pytest.raises(OSError):
            parent.rename(tmp_path / "moved")
        os.write(fd, b"safe")
    assert (parent / "out.svg").read_bytes() == b"safe"
