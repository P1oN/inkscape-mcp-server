"""Exercise the shipped helper's request failure boundary without a native GUI."""

import json
import runpy
import sys
from pathlib import Path
from types import ModuleType

import pytest
from lxml import etree

from inkscape_mcp.live import insert_payload
from inkscape_mcp.live.managed_dbus import ENV_DIR


@pytest.mark.parametrize("failure", ["missing", "oversized", "malformed", "missing_nonce"])
def test_request_failure_writes_refusal_and_helper_can_recover(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, failure: str
) -> None:
    inkex = ModuleType("inkex")
    inkex.EffectExtension = object  # type: ignore[attr-defined]
    inkex.AbortExtension = RuntimeError  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "inkex", inkex)
    monkeypatch.setitem(sys.modules, "inkscape_mcp_insert_payload", insert_payload)
    monkeypatch.setenv(ENV_DIR, str(tmp_path))
    helper_path = Path(insert_payload.__file__).parent / "helper_extension/inkscape_mcp_insert.py"
    helper = runpy.run_path(str(helper_path))["InsertOnce"]()
    helper.svg = etree.fromstring(b'<svg xmlns="http://www.w3.org/2000/svg"><rect id="old"/></svg>')
    original = etree.tostring(helper.svg)
    request = tmp_path / "insert-request.json"
    if failure == "oversized":
        request.write_text(" " * (2 * 1024 * 1024 + 1))
    elif failure == "malformed":
        request.write_text("{")
    elif failure == "missing_nonce":
        request.write_text("{}")

    with pytest.raises(RuntimeError, match="Insertion refused"):
        helper.effect()
    reply = tmp_path / "insert-result.json"
    result = json.loads(reply.read_text())
    assert result["nonce"] is None and result["ok"] is False
    assert "error" in result
    assert etree.tostring(helper.svg) == original
    assert not (tmp_path / "insert-result.tmp").exists()

    nonce = "mcp_" + "a" * 32
    request.write_text(
        json.dumps(
            {
                "nonce": nonce,
                "fragment": '<circle id="new" r="5"/>',
                "expected_ids": ["old"],
                "expected_fingerprint": insert_payload.document_fingerprint(helper.svg),
            }
        )
    )
    helper.effect()
    result = json.loads(reply.read_text())
    assert result["nonce"] == nonce and result["ok"] is True
    assert helper.svg[-1].get("id") == nonce
