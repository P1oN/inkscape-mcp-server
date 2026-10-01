"""MCP startup/reconnect must not open or resurrect a GUI."""

import sys
from collections.abc import Iterator
from pathlib import Path

import pytest
from fastmcp.exceptions import ToolError

from inkscape_mcp.config import ENV_LIVE_ENABLED, get_settings
from inkscape_mcp.live import macos_launcher as launcher
from inkscape_mcp.live.context_bridge import ENV_BRIDGE
from inkscape_mcp.live.managed_dbus import ENV_DIR, ENV_STDOUT
from inkscape_mcp.tools import live


@pytest.fixture
def isolated(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[Path]:
    root = tmp_path / "session"
    monkeypatch.setenv(ENV_DIR, str(root))
    for key in (ENV_STDOUT, ENV_BRIDGE, "DBUS_SESSION_BUS_ADDRESS"):
        monkeypatch.setenv(key, "")
        monkeypatch.delenv(key)
    monkeypatch.setenv("PATH", launcher.os.environ.get("PATH", ""))
    monkeypatch.setattr(launcher, "secure_directory", lambda path: path)
    monkeypatch.setattr(launcher.sys, "platform", "darwin")
    monkeypatch.setattr(live.platform, "system", lambda: "Darwin")
    monkeypatch.setenv(ENV_LIVE_ENABLED, "1")
    get_settings.cache_clear()
    yield root
    get_settings.cache_clear()


def test_boot_without_gui_runs_server_without_creating_session(
    isolated: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    from inkscape_mcp import server

    monkeypatch.setattr(sys, "argv", ["inkscape-mcp-macos", "--session-dir", str(isolated)])
    monkeypatch.setattr(launcher, "ensure_session", lambda *a: pytest.fail("no launch at boot"))
    called = []
    monkeypatch.setattr(server, "main", lambda: called.append(True))
    launcher.main()
    assert called == [True]
    assert not isolated.exists()


def test_attach_reuses_running_session_then_clears_closed_session(
    isolated: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    isolated.mkdir()
    data = {"address": f"unix:path={isolated}/bus.sock", "context_bridge": True}
    monkeypatch.setattr(launcher, "_read_session", lambda root: data)
    monkeypatch.setattr(launcher, "ensure_session", lambda *a: pytest.fail("no launch on attach"))
    assert launcher.attach_session(isolated)
    assert launcher.os.environ[ENV_STDOUT] == str(isolated / "inkscape.stdout.log")
    assert launcher.os.environ[ENV_BRIDGE] == "1"
    monkeypatch.setattr(launcher, "_read_session", lambda root: None)
    assert not launcher.attach_session(isolated)
    assert ENV_STDOUT not in launcher.os.environ
    assert "DBUS_SESSION_BUS_ADDRESS" not in launcher.os.environ


def test_explicit_launch_prepares_managed_environment(
    isolated: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    calls = []

    def launch(root: Path) -> dict[str, object]:
        calls.append(root)
        return {"address": "private-bus", "context_bridge": True}

    monkeypatch.setattr(launcher, "ensure_session", launch)
    assert live.live_launch()
    assert calls == [isolated]
    assert launcher.os.environ["DBUS_SESSION_BUS_ADDRESS"] == "private-bus"


def test_launch_gate_prevents_launch(isolated: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(ENV_LIVE_ENABLED, "0")
    get_settings.cache_clear()
    monkeypatch.setattr(launcher, "ensure_session", lambda *a: pytest.fail("disabled"))
    with pytest.raises(ToolError):
        live.live_launch()


def test_connect_refreshes_session_without_launching(
    isolated: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    from inkscape_mcp.live.session import LiveSession

    monkeypatch.setattr(launcher, "ensure_session", lambda *a: pytest.fail("connect cannot launch"))

    class Manager:
        def connect(self, *, prefer: str) -> LiveSession:
            return LiveSession(enabled=True, connected=False)

    monkeypatch.setattr(live, "get_session_manager", Manager)
    assert not live.live_connect().connected
    assert not isolated.exists()


def test_document_requires_explicit_launch(isolated: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(sys, "argv", ["inkscape-mcp-macos", "--document", "drawing.svg"])
    monkeypatch.setattr(launcher, "ensure_session", lambda *a: pytest.fail("no implicit launch"))
    with pytest.raises(SystemExit) as result:
        launcher.main()
    assert result.value.code == 2


@pytest.mark.parametrize("configured", [True, False])
def test_session_directory_without_getuid(
    isolated: Path, monkeypatch: pytest.MonkeyPatch, configured: bool
) -> None:
    monkeypatch.delattr(launcher.os, "getuid", raising=False)
    if not configured:
        monkeypatch.delenv(ENV_DIR)
    expected = isolated if configured else Path("/tmp/inkscape-mcp-0")
    assert launcher.session_directory() == expected


def test_configured_directory_does_not_evaluate_uid(
    isolated: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(
        launcher.os,
        "getuid",
        lambda: pytest.fail("configured path must not query uid"),
        raising=False,
    )
    assert launcher.session_directory() == isolated
