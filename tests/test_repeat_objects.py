from pathlib import Path

import pytest
from fastmcp.exceptions import ToolError
from lxml import etree

from inkscape_mcp.config import ENV_WORKSPACE_ROOTS, get_settings
from inkscape_mcp.edit.dom import EditError
from inkscape_mcp.edit.repeat import (
    PolylinePlacement,
    RectanglePlacement,
    RepeatVariation,
    placement_plan,
)
from inkscape_mcp.edit.structure import composed, references
from inkscape_mcp.registry import get_registry, reset_registry
from inkscape_mcp.snapshots import list_snapshots
from inkscape_mcp.tools.transform import repeat_objects


def test_polyline_spacing_tangent_seed_and_bounds():
    geometry = PolylinePlacement(
        kind="polyline", points=[{"x": 0, "y": 0}, {"x": 10, "y": 0}, {"x": 10, "y": 10}], spacing=5
    )
    plain = placement_plan(geometry, "tangent")
    assert [(p.x, p.y) for p in plain] == [(0, 0), (5, 0), (10, 0), (10, 5), (10, 10)]
    assert plain[-1].rotation_degrees == 90
    variation = RepeatVariation(seed=42, offset=1, scale=0.1, rotation_degrees=3)
    assert placement_plan(geometry, "tangent", variation) == placement_plan(
        geometry, "tangent", variation
    )
    other = placement_plan(geometry, "tangent", RepeatVariation(seed=43, offset=1))
    assert other != placement_plan(geometry, "tangent", variation)
    with pytest.raises(EditError, match="1024"):
        placement_plan(geometry.model_copy(update={"spacing": 0.001}))
    with pytest.raises(EditError, match="exactly one"):
        placement_plan(geometry.model_copy(update={"count": 2}))
    rect = RectanglePlacement(kind="rectangle", x=0, y=0, width=10, height=10, count=4, columns=2)
    assert [(p.x, p.y) for p in placement_plan(rect)] == [
        (2.5, 2.5),
        (7.5, 2.5),
        (2.5, 7.5),
        (7.5, 7.5),
    ]


@pytest.fixture
def doc(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv(ENV_WORKSPACE_ROOTS, str(tmp_path))
    get_settings.cache_clear()
    reset_registry()
    source = tmp_path / "scene.svg"
    source.write_text(
        '<svg xmlns="http://www.w3.org/2000/svg"><g id="parent" '
        'transform="translate(20,30) scale(2)" fill="red"><g id="leaf" '
        'transform="translate(3,4)"><defs><linearGradient id="grad"/></defs>'
        '<rect id="r" width="2" height="3" fill="url(#grad)"/></g></g></svg>'
    )
    entry = get_registry().open_document(str(source))
    yield entry, Path(entry.working_path)
    get_settings.cache_clear()


@pytest.mark.parametrize("mode", ["linked", "copies"])
def test_apply_dry_run_refs_coordinates_and_single_snapshot(doc, mode):
    entry, path = doc
    geometry = PolylinePlacement(
        kind="polyline", points=[{"x": 40, "y": 50}, {"x": 60, "y": 50}], count=2
    )
    before = path.read_bytes()
    result = repeat_objects(entry.doc_id, "leaf", geometry, "pattern", mode=mode)
    assert result.dry_run and not result.edit
    assert path.read_bytes() == before and not list_snapshots(entry.doc_id)
    applied = repeat_objects(entry.doc_id, "leaf", geometry, "pattern", mode=mode, dry_run=False)
    assert applied.plan == result.plan
    assert applied.edit.changed
    assert len(list_snapshots(entry.doc_id)) == 1
    root = etree.parse(str(path)).getroot()
    ids = [n.get("id") for n in root.iter() if n.get("id")]
    assert len(ids) == len(set(ids))
    group = root.xpath('//*[@id="pattern"]')[0]
    assert len(group) == 2
    for index, clone in enumerate(group):
        matrix = composed(clone)
        if mode == "linked":
            from inkscape_mcp.edit.structure import multiply, parse_transform

            matrix = multiply(
                matrix, parse_transform(root.xpath('//*[@id="leaf"]')[0].get("transform"))
            )
            assert clone.get("href") == "#leaf"
        else:
            assert clone.get("id") != "leaf"
            assert references(clone[-1]) == {clone[0][0].get("id")}
        assert (matrix[4], matrix[5]) == pytest.approx((40 + index * 20, 50))
    for node in root.iter():
        assert references(node) <= set(ids)
    before = path.read_bytes()
    with pytest.raises(ToolError, match="already exists"):
        repeat_objects(entry.doc_id, "leaf", geometry, "pattern", mode=mode, dry_run=False)
    assert path.read_bytes() == before


@pytest.mark.inkscape
def test_real_cli_linked_and_copies_render_identically(doc):
    from PIL import Image, ImageChops

    from inkscape_mcp.render.cli import render_preview
    from inkscape_mcp.snapshots import restore_snapshot

    entry, path = doc
    tree = etree.parse(str(path))
    for key, value in {"width": "100", "height": "100", "viewBox": "0 0 100 100"}.items():
        tree.getroot().set(key, value)
    tree.xpath('//*[@id="r"]')[0].set("fill", "blue")
    path.write_bytes(etree.tostring(tree))
    geometry = PolylinePlacement(
        kind="polyline", points=[{"x": 40, "y": 50}, {"x": 60, "y": 50}], count=2
    )
    first = repeat_objects(entry.doc_id, "leaf", geometry, "pattern", mode="linked", dry_run=False)
    linked = render_preview(entry.doc_id, width_px=400)
    restore_snapshot(entry.doc_id, first.edit.snapshot_id)
    repeat_objects(entry.doc_id, "leaf", geometry, "pattern", mode="copies", dry_run=False)
    copies = render_preview(entry.doc_id, width_px=400)
    images = [
        Image.open(Path(entry.root) / frame.workspace_relative_path).convert("RGBA")
        for frame in (linked, copies)
    ]
    assert images[0].getchannel("A").getbbox() is not None
    assert all(band.getbbox() is None for band in ImageChops.difference(*images).split())


def test_css_transform_source_and_size_projection_refuse_without_write(doc, monkeypatch):
    entry, path = doc
    geometry = PolylinePlacement(
        kind="polyline", points=[{"x": 0, "y": 0}, {"x": 10, "y": 10}], count=2
    )
    tree = etree.parse(str(path))
    tree.xpath('//*[@id="leaf"]')[0].set("style", "transform:scale(2)")
    path.write_bytes(etree.tostring(tree))
    before = path.read_bytes()
    with pytest.raises(ToolError, match="CSS transforms"):
        repeat_objects(entry.doc_id, "leaf", geometry, "pattern", dry_run=False)
    assert path.read_bytes() == before and not list_snapshots(entry.doc_id)
    monkeypatch.setenv("INKSCAPE_MCP_MAX_INPUT_BYTES", "1024")
    get_settings.cache_clear()
    with pytest.raises(ToolError, match="projected"):
        repeat_objects(entry.doc_id, "leaf", geometry, "pattern", mode="copies")
    assert path.read_bytes() == before and not list_snapshots(entry.doc_id)
