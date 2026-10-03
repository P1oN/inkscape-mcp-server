"""Exercise the shipped helper's request failure boundary without a native GUI."""

import json
import runpy
import sys
from pathlib import Path
from types import ModuleType

import insert_payload
import pytest
from lxml import etree

ENV_DIR = "INKSCAPE_MCP_MANAGED_DIR"


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


@pytest.mark.parametrize("fill", ["blue", "red"])
def test_edit_preserves_document_prolog_and_epilog_for_inkex_change_detection(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, fill: str
) -> None:
    from copy import deepcopy

    class SelectedElement(etree.ElementBase):
        @property
        def selection(self):
            return [self[0]]

    parser = etree.XMLParser()
    parser.set_element_class_lookup(etree.ElementDefaultClassLookup(element=SelectedElement))
    svg = etree.fromstring(
        b"<!-- Created with Inkscape --><?acceptance test?><!DOCTYPE svg>"
        b'<svg xmlns="http://www.w3.org/2000/svg"><rect id="selected" fill="blue"/></svg>'
        b"<?after first?><!-- after second -->",
        parser,
    )
    document = svg.getroottree()
    original = etree.tostring(document)
    inkex = ModuleType("inkex")
    inkex.EffectExtension = object  # type: ignore[attr-defined]
    inkex.AbortExtension = RuntimeError  # type: ignore[attr-defined]
    edit = ModuleType("inkscape_mcp_edit")

    def plan_edit(root, request):
        working = deepcopy(root)
        working[0].set("fill", request["style"]["fill"])
        return working, ["selected"]

    edit.plan_edit = plan_edit  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "inkex", inkex)
    monkeypatch.setitem(sys.modules, "inkscape_mcp_edit", edit)
    monkeypatch.setitem(sys.modules, "inkscape_mcp_insert_payload", insert_payload)
    monkeypatch.setenv(ENV_DIR, str(tmp_path))
    helper_path = Path(insert_payload.__file__).parent / "helper_extension/inkscape_mcp_insert.py"
    helper = runpy.run_path(str(helper_path))["InsertOnce"]()
    helper.svg, helper.document = svg, document
    (tmp_path / "insert-request.json").write_text(
        json.dumps(
            {
                "nonce": "mcp_" + "a" * 32,
                "operation": "style",
                "selection": ["selected"],
                "expected_ids": ["selected"],
                "expected_fingerprint": insert_payload.document_fingerprint(svg),
                "style": {"fill": fill},
            }
        )
    )
    helper.effect()
    reply = json.loads((tmp_path / "insert-result.json").read_text())
    assert reply["ok"] is True and reply["ids"] == ["selected"]
    assert (etree.tostring(helper.document) == original) is (fill == "blue")
    assert helper.document.getroot() is svg
    expected = original if fill == "blue" else original.replace(b'fill="blue"', b'fill="red"')
    assert etree.tostring(helper.document) == expected
    assert helper.document.docinfo.doctype == "<!DOCTYPE svg>"
    if fill == "blue":
        assert helper.document.getroot() is svg
        assert b"<!-- Created with Inkscape --><?acceptance test?>" in original
    else:
        assert helper.document.getroot()[0].get("fill") == "red"
