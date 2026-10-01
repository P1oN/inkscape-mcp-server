"""Managed insertion failure boundaries with a simulated native effect."""

import json
import sys
from pathlib import Path

import pytest
from lxml import etree

from inkscape_mcp.config import Settings
from inkscape_mcp.live.insert_payload import prepare_fragment
from inkscape_mcp.live.managed_dbus import ENV_DIR, ENV_STDOUT, ManagedDBusTransport
from inkscape_mcp.live.transport import LiveConnectionError, LiveError


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX file locking")
@pytest.mark.parametrize(
    "failure", ["timeout", "malformed", "array", "nonce", "refused", "action", "action_refused"]
)
def test_failed_insert_cleans_request_and_allows_next_operation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, failure: str
) -> None:
    stream = tmp_path / "stdout.log"
    stream.write_text("")
    monkeypatch.setenv(ENV_STDOUT, str(stream))
    monkeypatch.setenv(ENV_DIR, str(tmp_path))
    transport = ManagedDBusTransport(Settings(process_timeout_s=0.01))
    monkeypatch.setattr(transport, "supports", lambda command: True)
    original = '<svg xmlns="http://www.w3.org/2000/svg" id="root"><rect id="existing"/></svg>'
    document = original
    monkeypatch.setattr(transport, "get_document_svg", lambda: document)
    reply = tmp_path / "insert-result.json"
    request = tmp_path / "insert-request.json"
    recovery = False

    def activate(action: str, parameter: str) -> None:
        nonlocal document
        data = json.loads(request.read_text())
        if recovery:
            group, ids = prepare_fragment(data["fragment"], data["nonce"])
            root = etree.fromstring(document.encode())
            root.append(etree.fromstring(group))
            document = etree.tostring(root).decode()
            reply.write_text(json.dumps({"nonce": data["nonce"], "ok": True, "ids": ids}))
        elif failure == "action":
            raise LiveConnectionError("native action failed")
        elif failure == "action_refused":
            reply.write_text(json.dumps({"nonce": data["nonce"], "ok": False}))
            raise LiveConnectionError("native dialog outlived D-Bus call")
        elif failure == "malformed":
            reply.write_text("{")
        elif failure == "array":
            reply.write_text("[]")
        elif failure != "timeout":
            reply.write_text(
                json.dumps(
                    {
                        "nonce": "stale" if failure == "nonce" else data["nonce"],
                        "ok": failure != "refused",
                    }
                )
            )

    monkeypatch.setattr(transport, "_activate", activate)
    with pytest.raises(LiveError) as error:
        transport.insert_svg('<circle id="new" r="5"/>')
    if failure == "action_refused":
        assert "refused insertion" in str(error.value)
    elif failure == "action":
        assert "inspect Inkscape before retrying" in str(error.value)
    assert document == original
    assert not request.exists() and not reply.exists()
    recovery = True
    result = transport.insert_svg('<circle id="new" r="5"/>')
    assert result.count == 2 and result.affected_ids[0] in document
    assert "existing" in document
    assert not request.exists() and not reply.exists()
