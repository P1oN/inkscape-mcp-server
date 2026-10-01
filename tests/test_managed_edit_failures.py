"""Edit reply reconciliation: no blind retry after an uncertain completion."""

import json
import sys
from pathlib import Path

import pytest
from lxml import etree

from inkscape_mcp.config import Settings
from inkscape_mcp.live.insert_payload import document_fingerprint
from inkscape_mcp.live.managed_dbus import ENV_DIR, ENV_STDOUT, ManagedDBusTransport
from inkscape_mcp.live.transport import (
    LiveConnectionError,
    LiveEditRefused,
    LiveError,
    LiveMutationUncertain,
    LiveSelection,
)
from inkscape_mcp.tools.live import _map_live_error


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX managed operations")
@pytest.mark.parametrize(
    "failure", ["timeout", "malformed", "nonce", "refused", "activation", "mismatch"]
)
def test_edit_failure_cleanup_and_recovery(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    failure: str,
) -> None:
    stream = tmp_path / "stdout.log"
    stream.touch()
    monkeypatch.setenv(ENV_DIR, str(tmp_path))
    monkeypatch.setenv(ENV_STDOUT, str(stream))
    transport = ManagedDBusTransport(Settings(process_timeout_s=0.01))
    monkeypatch.setattr(transport, "_effect_available", lambda *args: True)
    monkeypatch.setattr(
        transport, "get_selection", lambda: LiveSelection(object_ids=["a"], count=1)
    )
    document = '<svg xmlns="http://www.w3.org/2000/svg"><rect id="a"/></svg>'
    monkeypatch.setattr(transport, "get_document_svg", lambda: document)
    request, reply = tmp_path / "insert-request.json", tmp_path / "insert-result.json"
    recovery = False

    def activate(*args: object) -> None:
        nonlocal document
        data = json.loads(request.read_text())
        if recovery or failure == "mismatch":
            root = etree.fromstring(document.encode())
            root[0].set("fill", "blue")
            document = etree.tostring(root).decode()
            reply.write_text(
                json.dumps(
                    dict(
                        nonce=data["nonce"],
                        ok=True,
                        ids=["a"],
                        fingerprint=document_fingerprint(root)
                        if recovery
                        else "not-applied-content",
                    )
                )
            )
        elif failure == "activation":
            raise LiveConnectionError("timed out")
        elif failure == "malformed":
            reply.write_text("{")
        elif failure == "nonce":
            reply.write_text(json.dumps(dict(nonce="old", ok=True)))
        elif failure == "refused":
            reply.write_text(
                json.dumps(
                    dict(
                        nonce=data["nonce"],
                        ok=False,
                        error="selection contains locked objects or belongs to a locked layer",
                    )
                )
            )

    monkeypatch.setattr(transport, "_activate", activate)
    expected = LiveEditRefused if failure == "refused" else LiveMutationUncertain
    with pytest.raises(expected) as error:
        transport._edit_selection("style", style={"fill": "blue"})
    assert not request.exists() and not reply.exists()
    if failure == "refused":
        assert "locked" in str(_map_live_error(error.value))
    recovery = True
    result = transport._edit_selection("style", style={"fill": "blue"})
    assert result.affected_ids == ["a"] and result.undo_friendly
    assert not request.exists() and not reply.exists()


def test_refusal_mapping_does_not_expose_generic_internal_errors() -> None:
    assert str(_map_live_error(LiveError("/private/secret"))) == "live operation failed"
