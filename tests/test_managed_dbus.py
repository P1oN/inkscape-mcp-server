"""Selection reply framing and fail-closed edits; no real GUI required."""

import sys
from pathlib import Path

import pytest

from inkscape_mcp.config import Settings
from inkscape_mcp.live import managed_dbus
from inkscape_mcp.live.macos_launcher import ensure_session, secure_directory
from inkscape_mcp.live.managed_dbus import (
    ENV_STDOUT,
    ManagedDBusTransport,
    parse_selection_reply,
)
from inkscape_mcp.live.transport import LiveConnectionError, LiveError, LiveSelection
from inkscape_mcp.workspace.subprocess_exec import ProcessResult


@pytest.mark.parametrize("enabled", [True, False])
def test_insert_capability_requires_enabled_gaction(
    enabled: bool, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv(managed_dbus.ENV_DIR, "/private/test-session")
    monkeypatch.setattr(ManagedDBusTransport, "_actions_call_argv", lambda *args: ["gdbus"])
    monkeypatch.setattr(
        managed_dbus,
        "run_process",
        lambda *args, **kwargs: ProcessResult(
            args=["gdbus"], returncode=0,
            stdout=f"(({str(enabled).lower()}, signature '', @av []),)\n", stderr="",
            duration_s=0.001, timed_out=False,
        ),
    )
    assert ManagedDBusTransport._insert_available(1) is enabled


def test_selection_requires_complete_fence() -> None:
    line = "rectangle cloned: false ref: 1 href: 0 total href: 0\n"
    assert parse_selection_reply(line) is None
    assert parse_selection_reply(line + "100") is None
    assert parse_selection_reply(line + "100\n") == ["rectangle"]
    assert parse_selection_reply("0\n") == []
    assert parse_selection_reply("-1.3e+2\n") == []


def test_selection_supports_multiple_and_unicode_ids() -> None:
    text = "山 cloned: false ref: 2 href: 1 total href: 1\n"
    text += "tree cloned: true ref: 1 href: 0 total href: 0\n"
    assert parse_selection_reply(text + "0\n") == ["山", "tree"]
    assert parse_selection_reply(text + "12.5,-1.3e+2\n") == ["山", "tree"]


def test_unrecognized_reply_is_never_an_empty_selection() -> None:
    with pytest.raises(LiveError):
        parse_selection_reply("new-format rectangle\n0\n")


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX file locking")
def test_get_selection_reads_only_new_output(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    stream = tmp_path / "stdout.log"
    stream.write_text("stale cloned: false ref: 1 href: 0 total href: 0\n0\n")
    monkeypatch.setenv(ENV_STDOUT, str(stream))
    transport = ManagedDBusTransport(Settings(process_timeout_s=1))

    def activate(action: str, param: str) -> None:
        with stream.open("a") as output:
            if action == "select-list":
                output.write("current cloned: false ref: 1 href: 0 total href: 0\n")
            elif action == "query-x":
                output.write("12\n")

    monkeypatch.setattr(transport, "_activate", activate)
    assert transport.get_selection().object_ids == ["current"]


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX file locking")
def test_missing_reply_times_out(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    stream = tmp_path / "stdout.log"
    stream.write_text("")
    monkeypatch.setenv(ENV_STDOUT, str(stream))
    transport = ManagedDBusTransport(Settings(process_timeout_s=0.02))
    monkeypatch.setattr(transport, "_activate", lambda *args: None)
    with pytest.raises(LiveConnectionError, match="complete"):
        transport.get_selection()


def test_unsupported_edits_and_empty_selection_do_not_activate(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    transport = ManagedDBusTransport(Settings())
    monkeypatch.setattr(transport, "_activate", lambda *args: pytest.fail("must not mutate"))
    with pytest.raises(LiveError, match="single fill"):
        transport.apply_to_selection(style={"fill": "red", "opacity": "0.5"}, transform=None)
    monkeypatch.setattr(transport, "get_selection", lambda: LiveSelection())
    with pytest.raises(LiveError, match="select an object"):
        transport.apply_to_selection(style={"fill": "red"}, transform=None)


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX directory ownership")
def test_private_session_directory_rejects_symlinks_and_public_permissions(tmp_path: Path) -> None:
    public = tmp_path / "public"
    public.mkdir(mode=0o755)
    with pytest.raises(RuntimeError, match="0700"):
        secure_directory(public)
    link = tmp_path / "link"
    link.symlink_to(public)
    with pytest.raises(RuntimeError, match="symlink"):
        secure_directory(link)


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX supervisor locking")
def test_running_supervisor_with_failed_bus_is_not_replaced(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    import fcntl

    from inkscape_mcp.live import macos_launcher

    monkeypatch.setattr(macos_launcher, "secure_directory", lambda path: path)
    monkeypatch.setattr(macos_launcher, "_read_session", lambda path: None)
    monkeypatch.setattr(
        macos_launcher.subprocess, "Popen", lambda *args, **kwargs: pytest.fail("no new GUI")
    )
    with (tmp_path / "supervisor.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        with pytest.raises(RuntimeError, match="save and close"):
            ensure_session(tmp_path)


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX supervisor locking")
def test_reconnect_reuses_existing_session_without_spawning(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    from inkscape_mcp.live import macos_launcher

    session = {"address": "existing", "inkscape_pid": 123}
    monkeypatch.setattr(macos_launcher, "secure_directory", lambda path: path)
    monkeypatch.setattr(macos_launcher, "_read_session", lambda path: session)
    monkeypatch.setattr(
        macos_launcher.subprocess, "Popen", lambda *args, **kwargs: pytest.fail("no new GUI")
    )
    assert ensure_session(tmp_path) == session
    with pytest.raises(RuntimeError, match="already running"):
        ensure_session(tmp_path, Path("drawing.svg"))
