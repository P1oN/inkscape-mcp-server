#!/usr/bin/env python3
"""Fixed private managed supervisor. Contains no MCP server or runtime compiler."""

import fcntl
import json
import os
import plistlib
import shlex
import stat
import subprocess
import sys
import time
import uuid
from pathlib import Path

LIMIT = 128 * 1024 * 1024
ASSETS = (
    "inkscape_mcp_insert.py",
    "inkscape_mcp_insert.inx",
    "inkscape_mcp_edit.py",
    "inkscape_mcp_edit.inx",
    "inkscape_mcp_insert_payload.py",
    "inkscape_mcp_edit_errors.py",
)


def directory(path, create=False):
    """Pin every absolute component without following links."""
    path = Path(path)
    if not path.is_absolute() or ".." in path.parts:
        raise RuntimeError("unsafe directory")
    fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY)
    try:
        for name in path.parts[1:]:
            if create:
                try:
                    os.mkdir(name, 0o700, dir_fd=fd)
                except FileExistsError:
                    pass
            child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = child
        return fd
    except BaseException:
        os.close(fd)
        raise


def read(path, cap=LIMIT):
    parent = directory(Path(path).parent)
    try:
        fd = os.open(Path(path).name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        with os.fdopen(fd, "rb") as stream:
            info = os.fstat(stream.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_size > cap:
                raise RuntimeError("invalid fixed asset")
            raw = stream.read(cap + 1)
            if len(raw) > cap:
                raise RuntimeError("oversized fixed asset")
            return raw
    finally:
        os.close(parent)


def write(fd, name, raw, mode=0o600):
    if "/" in name or name in (".", ".."):
        raise RuntimeError("invalid output name")
    try:
        info = os.stat(name, dir_fd=fd, follow_symlinks=False)
        if not stat.S_ISREG(info.st_mode):
            raise RuntimeError("nonregular output")
    except FileNotFoundError:
        pass
    token = ".mcp-" + uuid.uuid4().hex
    handle = os.open(token, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, mode, dir_fd=fd)
    try:
        with os.fdopen(handle, "wb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(token, name, src_dir_fd=fd, dst_dir_fd=fd)
        os.fsync(fd)
    finally:
        try:
            os.unlink(token, dir_fd=fd)
        except FileNotFoundError:
            pass


def lock(fd, name):
    handle = os.open(name, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK, 0o600, dir_fd=fd)
    if not stat.S_ISREG(os.fstat(handle).st_mode):
        os.close(handle)
        raise RuntimeError("invalid lock")
    return os.fdopen(handle, "r+b")


def prepare(library, root, binary):
    # All executable/file inputs originate from fixed package paths and the Inkscape probe.
    contents = binary.parent.parent
    resources = contents / "Resources"
    if binary.parent.name != "MacOS" or not (resources / "lib/libgtk-3.0.dylib").is_file():
        raise RuntimeError("official GTK 3 Inkscape bundle required")
    vendor = resources / "share/inkscape/extensions"
    if not (vendor / "inkex").is_dir():
        raise RuntimeError("vendor inkex unavailable")
    result = subprocess.run(
        [str(binary), "--user-data-directory"], capture_output=True, timeout=10, check=True
    )
    user = Path(result.stdout.decode().strip())
    if not user.is_absolute() or len(result.stdout) > 8192:
        raise RuntimeError("user data directory unavailable")
    target = user / "extensions"
    extensions = directory(target, create=True)
    try:
        payloads = [(name, read(library / "helpers" / name, 2 * 1024 * 1024)) for name in ASSETS]
        for name, _ in payloads:
            try:
                info = os.stat(name, dir_fd=extensions, follow_symlinks=False)
                if not stat.S_ISREG(info.st_mode):
                    raise RuntimeError("unsafe extension destination")
            except FileNotFoundError:
                pass
        for name, raw in payloads:
            write(extensions, name, raw)
        wrapper = (
            "#!/bin/sh\nunset PYTHONHOME PYTHONPATH\n"
            + "export PYTHONPATH="
            + shlex.quote(str(vendor))
            + "\n"
            + "exec "
            + shlex.quote(str(library / "python/bin/python3"))
            + " "
            + shlex.quote(str(target / "inkscape_mcp_insert.py"))
            + ' "$@"\n'
        ).encode()
        write(extensions, "inkscape_mcp_insert_run.sh", wrapper, 0o700)
    finally:
        os.close(extensions)
    macos = root / "context-bridge/Inkscape.app/Contents/MacOS"
    executable = macos / "inkscape"
    fd = directory(macos, create=True)
    try:
        write(fd, "inkscape", read(binary), 0o700)
    finally:
        os.close(fd)
    contents_fd = directory(macos.parent)
    try:
        try:
            existing = os.readlink("Resources", dir_fd=contents_fd)
            if existing != str(resources):
                raise RuntimeError("unexpected resource link")
        except FileNotFoundError:
            os.symlink(resources, "Resources", dir_fd=contents_fd)
        info = plistlib.loads(read(contents / "Info.plist", 1024 * 1024))
        info["CFBundleIdentifier"] = "org.inkscape.Inkscape.MCPManaged"
        write(contents_fd, "Info.plist", plistlib.dumps(info))
    finally:
        os.close(contents_fd)
    bridge_fd = directory(root / "context-bridge")
    try:
        write(bridge_fd, "context.so", read(library / "context.so", 16 * 1024 * 1024))
    finally:
        os.close(bridge_fd)
    # Sign only the private executable copy. Never alter the vendor app or security settings.
    subprocess.run(
        ["/usr/bin/codesign", "--force", "--sign", "-", str(executable)],
        check=True,
        capture_output=True,
        timeout=30,
    )
    return executable, root / "context-bridge/context.so"


def supervise(root, binary):
    library = Path(__file__).resolve().parent
    root_fd = directory(root)
    bus = None
    gui = None
    try:
        info = os.fstat(root_fd)
        if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
            raise RuntimeError("private session permissions required")
        with lock(root_fd, "supervisor.lock") as held:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
            executable, bridge = prepare(library, root, binary)
            try:
                os.unlink("bus.sock", dir_fd=root_fd)
            except FileNotFoundError:
                pass
            for name in ("bus.log", "inkscape.stdout.log", "inkscape.stderr.log"):
                write(root_fd, name, b"")
            with (
                lock(root_fd, "bus.log") as log,
                lock(root_fd, "inkscape.stdout.log") as stdout,
                lock(root_fd, "inkscape.stderr.log") as stderr,
            ):
                address = "unix:path=" + str(root / "bus.sock")
                bus = subprocess.Popen(
                    [
                        str(library / "dbus/bin/dbus-daemon"),
                        "--config-file=" + str(library / "dbus/session.conf"),
                        "--address=" + address,
                        "--nofork",
                        "--print-address=1",
                    ],
                    stdin=subprocess.DEVNULL,
                    stdout=log,
                    stderr=log,
                )
                # Pin the sole bus endpoint; no stdout-derived arbitrary address is trusted.
                deadline = time.monotonic() + 5
                while time.monotonic() < deadline:
                    try:
                        if stat.S_ISSOCK(
                            os.stat("bus.sock", dir_fd=root_fd, follow_symlinks=False).st_mode
                        ):
                            break
                    except FileNotFoundError:
                        pass
                    if bus.poll() is not None:
                        raise RuntimeError("private bus exited")
                    time.sleep(0.05)
                else:
                    raise RuntimeError("private bus did not become ready")
                env = dict(os.environ)
                env.update(
                    DBUS_SESSION_BUS_ADDRESS=address,
                    INKSCAPE_MCP_MANAGED_DIR=str(root),
                    INKSCAPE_MCP_CONTEXT_BRIDGE="1",
                )
                env["GTK_MODULES"] = os.pathsep.join(
                    filter(None, (env.get("GTK_MODULES", ""), str(bridge)))
                )
                gui = subprocess.Popen(
                    [str(executable), "--with-gui"],
                    env=env,
                    stdin=subprocess.DEVNULL,
                    stdout=stdout,
                    stderr=stderr,
                )
                manifest = dict(
                    address=address,
                    inkscape_pid=gui.pid,
                    supervisor_pid=os.getpid(),
                    context_bridge=True,
                )
                write(root_fd, "session.json", json.dumps(manifest).encode())
                gui.wait()
    finally:
        # Only the exact owned bus is terminated, and only once the GUI exits or never starts.
        if gui is None or gui.poll() is not None:
            if bus is not None and bus.poll() is None:
                bus.terminate()
                try:
                    bus.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    bus.kill()
                    bus.wait()
            try:
                os.unlink("session.json", dir_fd=root_fd)
            except FileNotFoundError:
                pass
        os.close(root_fd)


if __name__ == "__main__":
    if sys.platform != "darwin" or len(sys.argv) != 3:
        raise SystemExit("fixed managed supervisor requires macOS and owned session inputs")
    supervise(Path(sys.argv[1]), Path(sys.argv[2]).resolve())
