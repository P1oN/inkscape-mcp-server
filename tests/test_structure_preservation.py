from pathlib import Path

import pytest
from fastmcp.exceptions import ToolError
from lxml import etree

from inkscape_mcp.config import ENV_WORKSPACE_ROOTS, get_settings
from inkscape_mcp.edit.dom import EditError
from inkscape_mcp.edit.structure import MODE, composed, reparent_preserving
from inkscape_mcp.registry import get_registry, reset_registry
from inkscape_mcp.tools.create import create_group, reparent_object, set_group_mode
from inkscape_mcp.tools.text_object import rename_object


@pytest.fixture
def doc(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv(ENV_WORKSPACE_ROOTS, str(tmp_path))
    get_settings.cache_clear()
    reset_registry()
    source = tmp_path / "scene.svg"
    source.write_text(
        '<svg xmlns="http://www.w3.org/2000/svg"><g id="old" '
        'transform="translate(20,30)"><rect id="r" width="2" height="3" '
        'fill="red" transform="rotate(15)"/></g>'
        '<g id="new" transform="scale(2)"/></svg>'
    )
    entry = get_registry().open_document(str(source))
    yield entry, Path(entry.working_path)
    get_settings.cache_clear()


def test_named_creation_conversion_and_label(doc):
    entry, working = doc
    made = create_group(entry.doc_id, object_id="cat", label="Cat", mode="layer")
    assert made.changed
    rename_object(entry.doc_id, "cat", label="Sleeping cat")
    before = etree.parse(str(working))
    node = before.xpath('//*[@id="cat"]')[0]
    attrs = dict(node.attrib)
    result = set_group_mode(entry.doc_id, "cat", "group")
    after = etree.parse(str(working)).xpath('//*[@id="cat"]')[0]
    assert {k: v for k, v in attrs.items() if k != MODE} == dict(after.attrib)
    assert result.changed
    assert not set_group_mode(entry.doc_id, "cat", "group").changed


def test_reparent_preserves_matrix_style_and_snapshot(doc):
    entry, working = doc
    tree = etree.parse(str(working))
    before = composed(tree.xpath('//*[@id="r"]')[0])
    original = working.read_bytes()
    result = reparent_object(entry.doc_id, "r", "new", preserve_appearance=True)
    node = etree.parse(str(working)).xpath('//*[@id="r"]')[0]
    assert composed(node) == pytest.approx(before)
    assert node.get("fill") == "red"
    assert node.getparent().get("id") == "new"
    from inkscape_mcp.snapshots import restore_snapshot

    restore_snapshot(entry.doc_id, result.snapshot_id)
    assert working.read_bytes() == original


@pytest.mark.parametrize(
    "change,reason",
    [
        ('fill="blue"', "style"),
        ('transform="scale(0)"', "singular"),
        ('style="transform:scale(2)"', "style"),
    ],
)
def test_refusal_does_not_write(doc, change, reason):
    entry, working = doc
    tree = etree.parse(str(working))
    node = tree.xpath('//*[@id="new"]')[0]
    node.attrib.clear()
    node.set("id", "new")
    key, value = change.split("=", 1)
    node.set(key, value.strip('"'))
    working.write_bytes(etree.tostring(tree))
    original = working.read_bytes()
    with pytest.raises(ToolError, match=reason):
        reparent_object(entry.doc_id, "r", "new", preserve_appearance=True)
    assert working.read_bytes() == original


def test_paint_order_and_external_reference_refusals():
    tree = etree.ElementTree(
        etree.fromstring(b'<svg><g id="a"><rect id="r"/></g><rect id="middle"/><g id="b"/></svg>')
    )
    with pytest.raises(EditError, match="paint order"):
        reparent_preserving(tree, "r", "b")
    tree = etree.ElementTree(
        etree.fromstring(b'<svg><g id="a"><rect id="r"/></g><g id="b"/><use href="#r"/></svg>')
    )
    with pytest.raises(EditError, match="external references"):
        reparent_preserving(tree, "r", "b")


@pytest.mark.inkscape
def test_real_cli_reparent_and_conversion_preserve_pixels(doc):
    from PIL import Image, ImageChops

    from inkscape_mcp.render.cli import render_preview

    entry, working = doc
    tree = etree.parse(str(working))
    for key, value in {"width": "100", "height": "100", "viewBox": "0 0 100 100"}.items():
        tree.getroot().set(key, value)
    working.write_bytes(etree.tostring(tree))
    before = render_preview(entry.doc_id, width_px=400)
    reparent_object(entry.doc_id, "r", "new", preserve_appearance=True)
    set_group_mode(entry.doc_id, "new", "layer")
    after = render_preview(entry.doc_id, width_px=400)
    images = [
        Image.open(Path(entry.root) / frame.workspace_relative_path).convert("RGBA")
        for frame in (before, after)
    ]
    assert images[0].getchannel("A").getbbox() is not None
    assert all(band.getbbox() is None for band in ImageChops.difference(*images).split())


def test_css_transform_on_moved_object_refuses_without_write(doc):
    entry, working = doc
    tree = etree.parse(str(working))
    tree.xpath('//*[@id="r"]')[0].set("style", "transform:translate(1px, 2px)")
    working.write_bytes(etree.tostring(tree))
    before = working.read_bytes()
    with pytest.raises(ToolError, match="CSS transforms"):
        reparent_object(entry.doc_id, "r", "new", preserve_appearance=True)
    assert working.read_bytes() == before
