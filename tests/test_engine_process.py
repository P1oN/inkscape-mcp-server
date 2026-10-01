"""`EngineProcess` framing + lifecycle, exercised against a fake shell (no Inkscape).

The fake (``fake_inkscape_shell.py``) reproduces the real shell's framing, so these tests drive the
PRODUCTION framing / threading / timeout / crash / shutdown code without a real Inkscape binary.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

from inkscape_mcp.config import Settings
from inkscape_mcp.engine.process import (
    EngineActionError,
    EngineCrash,
    EngineProcess,
    EngineTimeout,
)

FAKE = [sys.executable, str(Path(__file__).parent / "fake_inkscape_shell.py")]


def _proc(timeout_s: float = 10.0) -> EngineProcess:
    return EngineProcess(argv=FAKE, settings=Settings(process_timeout_s=timeout_s))


def test_banner_drained_and_first_command_frames_cleanly() -> None:
    p = _proc()
    p.start()
    try:
        resp = p.execute("select-by-id:r1")
        # No-output command: echo + prompt stripped leaves nothing.
        assert resp.output_lines == []
        assert resp.command == "select-by-id:r1"
    finally:
        p.shutdown()


def test_single_line_output_stripped_of_echo_and_prompt() -> None:
    p = _proc()
    p.start()
    try:
        assert p.execute("query-x").output_lines == ["10"]
        assert p.execute("query-width").output_lines == ["30"]
    finally:
        p.shutdown()


def test_multiline_output_framed_without_bleeding_into_next_command() -> None:
    p = _proc()
    p.start()
    try:
        resp = p.execute("query-all")
        assert resp.output_lines == ["svg1,10,10,70,20", "r1,10,10,30,20"]
        # The next command must frame cleanly (no leftover from the multi-line response).
        assert p.execute("query-x").output_lines == ["10"]
    finally:
        p.shutdown()


def test_unknown_action_surfaces_engine_action_error() -> None:
    p = _proc()
    p.start()
    try:
        with pytest.raises(EngineActionError) as ei:
            p.execute("definitely-not-an-action")
        assert "definitely-not-an-action" in str(ei.value)
    finally:
        p.shutdown()


def test_newline_in_command_is_refused() -> None:
    p = _proc()
    p.start()
    try:
        with pytest.raises(EngineActionError):
            p.execute("query-x\nquery-width")
    finally:
        p.shutdown()


def test_per_command_timeout_kills_worker() -> None:
    p = _proc(timeout_s=0.5)
    p.start()
    try:
        with pytest.raises(EngineTimeout):
            p.execute("__sleep__:3")
        # The worker was killed on timeout, so it is no longer alive.
        assert not p.is_alive()
    finally:
        p.shutdown()


def test_crash_mid_session_raises_engine_crash() -> None:
    p = _proc()
    p.start()
    try:
        with pytest.raises(EngineCrash):
            p.execute("__crash__")
        assert not p.is_alive()
        # A subsequent command on the dead worker also raises crash (not a hang).
        with pytest.raises(EngineCrash):
            p.execute("query-x")
    finally:
        p.shutdown()


def test_clean_shutdown_terminates_worker() -> None:
    p = _proc()
    p.start()
    assert p.is_alive()
    p.shutdown()
    assert not p.is_alive()


def test_execute_before_start_raises_crash() -> None:
    p = _proc()
    with pytest.raises(EngineCrash):
        p.execute("query-x")


@pytest.mark.parametrize("polling", [False, True])
def test_action_error_never_bleeds_into_next_command(
    monkeypatch: pytest.MonkeyPatch, polling: bool
) -> None:
    from inkscape_mcp.engine import process

    if sys.platform == "win32" and not polling:
        pytest.skip("Windows selectors cannot monitor pipes")
    monkeypatch.setattr(process, "_POLL_PIPES", polling)
    p = _proc()
    p.start()
    try:
        for i in range(100):
            with pytest.raises(EngineActionError, match=f"unknown-{i}"):
                p.execute(f"unknown-{i}")
            assert p.execute("query-x").output_lines == ["10"]
    finally:
        p.shutdown()


@pytest.mark.parametrize("polling", [False, True])
def test_reader_preserves_split_utf8_and_flushes_incomplete_eof(
    monkeypatch: pytest.MonkeyPatch, polling: bool
) -> None:
    import os
    import subprocess

    from inkscape_mcp.engine import process

    if sys.platform == "win32" and not polling:
        pytest.skip("Windows selectors cannot monitor pipes")
    monkeypatch.setattr(process, "_POLL_PIPES", polling)
    data = "Кириллица 😀".encode() + b"\xe2"
    p = _proc()
    child = subprocess.Popen(
        [sys.executable, "-c", f"import os; os.write(1,{data!r}); os.write(2,{data!r})"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    assert child.stderr is not None
    stderr_fd = child.stderr.fileno()
    original_read = os.read
    pause_stderr = False

    def read_one(fd: int, size: int) -> bytes:
        nonlocal pause_stderr
        # Force a publication boundary after each stderr byte as well as stdout.
        if fd == stderr_fd:
            pause_stderr = not pause_stderr
            if not pause_stderr:
                raise BlockingIOError
        return original_read(fd, 1)

    monkeypatch.setattr(process.os, "read", read_one)
    p._proc = child
    try:
        p._read_output()
        assert p._out == p._err == "Кириллица 😀\ufffd"
    finally:
        child.wait(timeout=5)
        p.shutdown()


@pytest.mark.parametrize("newline", ["\n", "\r\n"])
def test_shell_frame_normalizes_platform_newlines(newline: str) -> None:
    frame = newline.join(["query-all", "svg1,10,10,70,20", "r1,10,10,30,20", "> "])
    assert EngineProcess._strip_frame(frame, "query-all") == ["svg1,10,10,70,20", "r1,10,10,30,20"]
