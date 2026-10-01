"""Windows no-follow writes and mkdir with directory handles held against renames.

Keep every ancestor open without FILE_SHARE_DELETE while accessing a pathname. Reparse
points are rejected before writing bytes or descending into a newly created directory.
"""

from __future__ import annotations

import errno
import os
import sys
from collections.abc import Iterator
from contextlib import ExitStack, contextmanager
from pathlib import Path


def _open_handle(
    path: Path, *, directory: bool, exclusive: bool = False, read_only: bool = False
) -> int:
    if sys.platform != "win32":
        raise RuntimeError("Windows file handles require Windows")
    import ctypes
    from ctypes import wintypes

    class AttributeTag(ctypes.Structure):
        _fields_ = [("attributes", wintypes.DWORD), ("tag", wintypes.DWORD)]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    create = kernel.CreateFileW
    create.argtypes = [
        wintypes.LPCWSTR,
        wintypes.DWORD,
        wintypes.DWORD,
        ctypes.c_void_p,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.HANDLE,
    ]
    create.restype = wintypes.HANDLE
    close = kernel.CloseHandle
    close.argtypes = [wintypes.HANDLE]
    close.restype = wintypes.BOOL
    info = kernel.GetFileInformationByHandleEx
    info.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    info.restype = wintypes.BOOL
    # OPEN_REPARSE_POINT, BACKUP_SEMANTICS for directories; OPEN_EXISTING / CREATE_NEW /
    # OPEN_ALWAYS. Never truncate until the opened handle has passed the reparse check.
    handle = create(
        str(path),
        0x80000000 if directory or read_only else 0x40000000,
        0x00000003 if directory else 0,
        None,
        3 if directory or read_only else (1 if exclusive else 4),
        0x00200000 | (0x02000000 if directory else 0),
        None,
    )
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        attributes = AttributeTag()
        if not info(handle, 9, ctypes.byref(attributes), ctypes.sizeof(attributes)):
            raise ctypes.WinError(ctypes.get_last_error())
        if attributes.attributes & 0x00000400:  # FILE_ATTRIBUTE_REPARSE_POINT
            raise OSError(errno.ELOOP, "reparse point refused")
        if directory and not attributes.attributes & 0x00000010:
            raise OSError(errno.ENOTDIR, "not a directory")
        return int(handle)
    except BaseException:
        close(handle)
        raise


def _close_handle(handle: int) -> None:
    if sys.platform != "win32":
        raise RuntimeError("Windows file handles require Windows")
    import ctypes
    from ctypes import wintypes

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    close = kernel.CloseHandle
    close.argtypes = [wintypes.HANDLE]
    close.restype = wintypes.BOOL
    if not close(handle):
        raise ctypes.WinError(ctypes.get_last_error())


@contextmanager
def _locked_directory(path: Path) -> Iterator[None]:
    handle = _open_handle(path, directory=True)
    try:
        yield
    finally:
        _close_handle(handle)


def _lock_ancestors(stack: ExitStack, path: Path) -> None:
    for ancestor in [*reversed(path.parents), path]:
        stack.enter_context(_locked_directory(ancestor))


def mkdir_chain(base: Path, components: tuple[str, ...]) -> None:
    with ExitStack() as stack:
        _lock_ancestors(stack, base)
        current = base
        for part in components:
            if part in ("", ".", "..") or Path(part).name != part:
                raise OSError(errno.EINVAL, "invalid directory component")
            current = current / part
            try:
                current.mkdir()
            except FileExistsError:
                pass
            stack.enter_context(_locked_directory(current))


@contextmanager
def write_fd(path: Path, *, exclusive: bool = False) -> Iterator[int]:
    if sys.platform != "win32":
        raise RuntimeError("Windows file handles require Windows")
    import msvcrt

    with ExitStack() as stack:
        _lock_ancestors(stack, path.parent)
        handle = _open_handle(path, directory=False, exclusive=exclusive)
        try:
            fd = msvcrt.open_osfhandle(handle, os.O_WRONLY | os.O_BINARY)
        except BaseException:
            _close_handle(handle)
            raise
        try:
            os.ftruncate(fd, 0)
            yield fd
        finally:
            os.close(fd)


@contextmanager
def read_fd(path: Path) -> Iterator[int]:
    """Open an existing non-reparse file while ancestors are held against renames."""
    if sys.platform != "win32":
        raise RuntimeError("Windows file handles require Windows")
    import msvcrt

    with ExitStack() as stack:
        _lock_ancestors(stack, path.parent)
        handle = _open_handle(path, directory=False, read_only=True)
        try:
            fd = msvcrt.open_osfhandle(handle, os.O_RDONLY | os.O_BINARY)
        except BaseException:
            _close_handle(handle)
            raise
        try:
            yield fd
        finally:
            os.close(fd)
