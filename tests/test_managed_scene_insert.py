"""Scene semantics and insertion boundaries; no GUI required."""

import json
import runpy
import sys
from pathlib import Path
from types import ModuleType

import pytest
from lxml import etree

from inkscape_mcp.live import insert_payload
from inkscape_mcp.live.insert_payload import prepare_fragment
from inkscape_mcp.live.managed_scene import scene_from_svg

PREFIX = "mcp_" + "a" * 32


def test_insertion_remaps_ids_and_both_paint_and_use_references() -> None:
    fragment = (
        '<defs><linearGradient id="p"><stop offset="0"/></linearGradient></defs>'
        '<rect id="r" width="10" height="20" fill="url(#p)"/>'
        '<use href="#r"/>'
    )
    data, ids = prepare_fragment(fragment, PREFIX)
    root = etree.fromstring(data)
    assert root.get("id") == PREFIX
    assert root[1].get("fill") == f"url(#{ids[1]})"
    assert root[2].get("href") == "#" + ids[2]
    assert len(ids) == len(set(ids))


@pytest.mark.parametrize(
    "fragment",
    [
        "<script>alert(1)</script>",
        "<foreignObject/>",
        '<image href="file:///secret"/>',
        '<use href="https://example.com/x.svg#p"/>',
        '<rect onload="x"/>',
        '<rect style="fill:url(file:///secret)"/>',
        '<rect style="fill:u\\72l(file:///x)"/>',
        '<rect xml:base="https://example.com"/>',
        '<rect id="x"/><circle id="x"/>',
        '<rect fill="url(#missing)"/>',
        "<rect",
        "<!-- empty -->",
    ],
)
def test_unsafe_or_ambiguous_insertions_are_refused(fragment: str) -> None:
    with pytest.raises(ValueError):
        prepare_fragment(fragment, PREFIX)


def test_scene_preserves_hierarchy_excludes_defs_and_hidden_ancestors() -> None:
    svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" '
        'xmlns:i="http://www.inkscape.org/namespaces/inkscape" viewBox="10 20 640 400">'
        '<defs><rect id="definition" width="5" height="5"/></defs>'
        '<g id="hidden" style="display:none"><rect id="child"/></g>'
        '<g id="layer" i:groupmode="layer" i:label="Hills" transform="translate(5,6)">'
        '<rect id="hill" x="1" y="2" width="30" height="40" fill="green"/></g></svg>'
    )
    scene = scene_from_svg(svg, ["hill"])
    assert scene.canvas.viewbox == [10, 20, 640, 400]
    assert scene.canvas.width == 640
    assert {o.id for o in scene.visible_objects} == {"layer", "hill"}
    assert scene.selection[0].bbox is None  # ancestor transform makes local geometry misleading
    assert scene.tree is not None
    layer = scene.tree.children[2]
    assert layer.is_layer and layer.label == "Hills" and layer.transform == "translate(5,6)"
    assert layer.children[0].paint.fill == "green"
    assert scene.viewport.zoom is None and scene.notes


def test_scene_unknown_physical_size_is_not_invented() -> None:
    scene = scene_from_svg('<svg xmlns="http://www.w3.org/2000/svg" width="10mm"/>', [])
    assert scene.canvas.width is None


def test_scene_reads_filename_from_real_inkscape_namespace() -> None:
    scene = scene_from_svg(
        '<svg xmlns="http://www.w3.org/2000/svg" '
        'xmlns:s="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" '
        's:docname="landscape.svg"><rect id="hill"/></svg>', []
    )
    assert scene.active_document is not None
    assert scene.active_document.name == "landscape.svg"
    assert scene.active_document.path is None


@pytest.mark.parametrize("change", ["ids", "paint", "none"])
def test_one_shot_helper_checks_document_before_mutation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, change: str
) -> None:
    fake = ModuleType("inkex")
    fake.EffectExtension = object  # type: ignore[attr-defined]
    fake.AbortExtension = RuntimeError  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "inkex", fake)
    monkeypatch.setitem(sys.modules, "inkscape_mcp_insert_payload", insert_payload)
    monkeypatch.setenv("INKSCAPE_MCP_MANAGED_DIR", str(tmp_path))
    helper = Path(insert_payload.__file__).parent / "helper_extension" / "inkscape_mcp_insert.py"
    cls = runpy.run_path(str(helper))["InsertOnce"]
    effect = cls()
    effect.svg = etree.fromstring(
        '<svg xmlns="http://www.w3.org/2000/svg" id="drawing">'
        '<rect id="original" fill="blue"/></svg>'
    )
    fingerprint = insert_payload.document_fingerprint(effect.svg)
    if change == "paint":
        effect.svg[0].set("fill", "red")
    (tmp_path / "insert-request.json").write_text(
        json.dumps(
            {
                "nonce": PREFIX,
                "fragment": '<rect width="10" height="20"/>',
                "expected_ids": ["different"] if change == "ids" else ["drawing", "original"],
                "expected_fingerprint": fingerprint,
            }
        )
    )
    if change == "none":
        effect.effect()
        assert len(effect.svg) == 2
        assert effect.svg[1].get("id") == PREFIX
        assert len(effect.svg[1]) == 1
    else:
        with pytest.raises(RuntimeError, match="refused"):
            effect.effect()
        assert len(effect.svg) == 1
    reply = json.loads((tmp_path / "insert-result.json").read_text())
    assert reply["nonce"] == PREFIX and reply["ok"] is (change == "none")
