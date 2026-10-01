"""Discovery geometry uses the renderer, retaining user units and snapshot consistency."""

from __future__ import annotations

import shutil
from contextlib import nullcontext
from pathlib import Path

import pytest
from fastmcp.exceptions import ToolError
from lxml import etree
from PIL import Image

from inkscape_mcp.config import Settings
from inkscape_mcp.live.discovery import (
    LiveDiscoveryError,
    _snapshot,
    find_in_snapshot,
    preview_snapshot_object,
    query_boxes,
    root_mapping,
)
from inkscape_mcp.live.transport import LiveError, RenderRegion
from inkscape_mcp.workspace.limits import LimitExceeded
from inkscape_mcp.workspace.subprocess_exec import ProcessResult

SVG = (
    '<svg xmlns="http://www.w3.org/2000/svg" '
    'xmlns:i="http://www.inkscape.org/namespaces/inkscape" '
    'xmlns:s="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" '
    'width="400" height="200" viewBox="10 20 100 50">'
    '<defs><rect id="definition" width="1" height="1"/></defs>'
    '<g id="hidden" display="none"><path id="hidden-path" d="M0 0h10v10z"/></g>'
    '<g id="layer" i:groupmode="layer" i:label="Дальние холмы" transform="translate(20,30)">'
    '<path id="hill" i:label="Зелёный холм" d="M0 0h10v5h-10z" fill="green"/>'
    '<text id="caption" x="5" y="10">Летний пейзаж</text>'
    '<g id="locked" s:insensitive="true"><rect id="locked-rect" width="2" height="2"/></g>'
    "</g></svg>"
)


@pytest.mark.parametrize(
    ("aspect", "expected"),
    [
        ("none", (4, 2, -40, -40)),
        ("xMidYMid meet", (2, 2, 80, -40)),
        ("xMaxYMax slice", (4, 4, -40, -280)),
        ("xMinYMin meet", (2, 2, -20, -40)),
    ],
)
def test_root_mapping_accounts_for_origin_letterbox_and_nonuniform_scale(
    aspect: str,
    expected: tuple[float, float, float, float],
) -> None:
    root = etree.fromstring(
        f'<svg width="400" height="200" viewBox="10 20 100 100" preserveAspectRatio="{aspect}"/>'
    )
    assert root_mapping(root) == expected


def test_physical_units_are_converted_to_pixels() -> None:
    root = etree.fromstring('<svg width="25.4mm" height="1in" viewBox="0 0 24 24"/>')
    assert root_mapping(root) == pytest.approx((4, 4, 0, 0))


def test_viewbox_only_root_uses_unit_scale_and_origin_translation() -> None:
    """Inkscape's standalone default preserves the viewBox's user-unit dimensions."""
    root = etree.fromstring('<svg viewBox="10 20 100 50"/>')
    assert root_mapping(root) == (1, 1, -10, -20)


@pytest.mark.parametrize("dimension", ["width", "height"])
def test_single_missing_root_dimension_still_refuses(dimension: str) -> None:
    root = etree.fromstring('<svg width="100" height="50" viewBox="0 0 100 50"/>')
    del root.attrib[dimension]
    with pytest.raises(LiveError, match="absolute root width and height"):
        root_mapping(root)


def test_managed_region_export_accepts_the_document_bounds_returned_by_discovery(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from inkscape_mcp.live.managed_dbus import ManagedDBusTransport

    transport = ManagedDBusTransport(Settings())
    monkeypatch.setattr(transport, "_operation", lambda: nullcontext())
    monkeypatch.setattr(transport, "get_document_svg", lambda: SVG)
    exports = []

    def export(fmt: str, **kwargs: object) -> bytes:
        exports.append((fmt, kwargs))
        return b"PNG"

    monkeypatch.setattr(transport, "_export_document_bytes", export)
    assert transport.render_view(RenderRegion(x=20, y=30, width=10, height=5), 0.5) == b"PNG"
    assert exports[0][1]["region"] == RenderRegion(x=40, y=40, width=40, height=20)
    assert exports[0][1]["dpi"] == 48


@pytest.mark.parametrize(
    "attrs",
    [
        'width="100%"',
        'style="width:400px"',
        'preserveAspectRatio="invalid"',
        'viewBox="0 0 nan 50"',
    ],
)
def test_unsupported_root_mapping_refuses(attrs: str) -> None:
    root = etree.fromstring('<svg width="400" height="200" viewBox="0 0 100 50"/>')
    other = etree.fromstring(f"<svg {attrs}/>")
    root.attrib.update(other.attrib)
    with pytest.raises(LiveError):
        root_mapping(root)


@pytest.mark.parametrize(
    "content",
    [
        '<image href="file:///secret"/>',
        '<rect fill="url(https://example.com/x)"/>',
        '<rect fill="u\\72l(file:///secret)"/>',
        '<rect style="fill:url(\nfile:///secret\n)"/>',
        '<rect fill="url(file:///secret"/>',
        '<rect style="fill:url(#valid);stroke:URL(\nfile:///secret"/>',
        '<text shape-inside="u\\72l(file:///secret)"/>',
        '<text shape-inside="u/**/rl(file:///secret)"/>',
        "<style>rect {fill:red}</style>",
        "<script/>",
        '<rect id="same"/><rect id="same"/>',
        '<image href="data:image/svg+xml;base64,PHN2Zy8+"/>',
        '<image xmlns:s="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" '
        'href="#missing" s:absref="/outside/secret.png"/>',
    ],
)
def test_snapshot_refuses_ambiguous_or_external_content(content: str) -> None:
    with pytest.raises(LiveError):
        _snapshot(f'<svg xmlns="http://www.w3.org/2000/svg">{content}</svg>', Settings())


def test_invalid_engine_rows_are_not_reported_as_geometry(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    from inkscape_mcp.live import discovery

    monkeypatch.setattr(
        discovery,
        "run_inkscape",
        lambda *args: ProcessResult(
            args=[],
            returncode=0,
            stdout="valid,40,40,80,20\nbad,nan,0,10,10\nnegative,0,0,-1,2\n",
            stderr="",
            duration_s=0,
            timed_out=False,
        ),
    )
    boxes = query_boxes(tmp_path / "unused.svg", etree.fromstring(SVG), Settings())
    assert set(boxes) == {"valid"}
    assert boxes["valid"].model_dump() == {"x": 20, "y": 30, "width": 20, "height": 5}


def test_tool_preserves_actionable_stale_snapshot_refusal(monkeypatch: pytest.MonkeyPatch) -> None:
    from inkscape_mcp.tools import live_discovery

    def refuse(*args: object) -> None:
        raise LiveDiscoveryError(
            "drawing changed since discovery; find the object again before preview"
        )

    monkeypatch.setattr(live_discovery, "preview_live_object", refuse)
    with pytest.raises(ToolError, match=r"drawing changed.*find the object again"):
        live_discovery.live_preview_object("hill", "old-fingerprint")


@pytest.mark.parametrize("tool", ["find", "preview"])
def test_tool_preserves_limit_reason_without_exposing_os_errors(
    monkeypatch: pytest.MonkeyPatch,
    tool: str,
) -> None:
    """Dimension/input caps are actionable; filesystem failures stay path-free."""
    from inkscape_mcp.tools import live_discovery

    name = "find_live_objects" if tool == "find" else "preview_live_object"

    def limited(*args: object, **kwargs: object) -> None:
        raise LimitExceeded("export dimensions exceed cap: 512x16384 > 8192px per side")

    monkeypatch.setattr(live_discovery, name, limited)
    invoke = (
        (lambda: live_discovery.live_find_objects())
        if tool == "find"
        else (lambda: live_discovery.live_preview_object("hill", "fingerprint"))
    )
    with pytest.raises(ToolError, match="export dimensions exceed cap"):
        invoke()

    def os_error(*args: object, **kwargs: object) -> None:
        raise OSError("private path /some/private/workspace")

    monkeypatch.setattr(live_discovery, name, os_error)
    with pytest.raises(ToolError) as error:
        invoke()
    assert "private" not in str(error.value)


@pytest.mark.inkscape
@pytest.mark.skipif(shutil.which("inkscape") is None, reason="requires Inkscape CLI")
def test_viewbox_only_geometry_preview_and_retention(tmp_path: Path) -> None:
    """Exercise Inkscape's missing-size default and the generated artifact's actual lifecycle."""
    from inkscape_mcp.retention import prune_live_frames_at

    svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="10 20 100 50">'
        '<rect id="r" x="20" y="30" width="10" height="5" fill="red"/></svg>'
    )
    settings = Settings(workspace_roots=[tmp_path], live_frame_max_bytes=1)
    found = find_in_snapshot(svg, settings=settings)
    assert found.objects[0].bbox is not None
    assert found.objects[0].bbox.model_dump() == pytest.approx(
        {"x": 20, "y": 30, "width": 10, "height": 5}
    )
    preview = preview_snapshot_object(svg, "r", found.fingerprint, width=200, settings=settings)
    artifact = tmp_path / preview.artifact_path
    with Image.open(artifact) as image:
        assert image.size == (200, 100)
        assert image.convert("RGBA").getpixel((100, 50)) == (255, 0, 0, 255)
    unrelated = artifact.with_name("unrelated.png")
    unrelated.write_bytes(b"keep")
    pruned = prune_live_frames_at(tmp_path, settings)
    assert pruned.pruned_frames == 1 and not artifact.exists()
    assert unrelated.exists()


@pytest.mark.inkscape
@pytest.mark.skipif(shutil.which("inkscape") is None, reason="requires Inkscape CLI")
def test_empty_drawing_discovery_returns_zero_matches() -> None:
    result = find_in_snapshot('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"/>')
    assert result.objects == [] and result.total_matches == 0


@pytest.mark.inkscape
@pytest.mark.skipif(shutil.which("inkscape") is None, reason="requires Inkscape CLI")
def test_tall_object_limit_can_be_recovered_by_reducing_width(tmp_path: Path) -> None:
    """Reject oversized aspect-ratio exports before rendering; a smaller width succeeds."""
    svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 100">'
        '<rect id="r" width="1" height="100"/></svg>'
    )
    settings = Settings(workspace_roots=[tmp_path])
    found = find_in_snapshot(svg, settings=settings)
    with pytest.raises(LimitExceeded, match=r"512x51200.*8192px per side"):
        preview_snapshot_object(svg, "r", found.fingerprint, width=512, settings=settings)
    preview = preview_snapshot_object(svg, "r", found.fingerprint, width=8, settings=settings)
    with Image.open(tmp_path / preview.artifact_path) as image:
        assert image.size == (8, 800)


@pytest.mark.inkscape
@pytest.mark.skipif(shutil.which("inkscape") is None, reason="requires Inkscape CLI")
def test_real_engine_finds_transformed_paths_text_layers_and_isolated_preview(
    tmp_path: Path,
) -> None:
    settings = Settings(workspace_roots=[tmp_path])
    result = find_in_snapshot(SVG, layer="ХОЛМЫ", tag="path", settings=settings)
    assert result.count == result.total_matches == 1
    hill = result.objects[0]
    assert hill.id == "hill" and hill.layer_id == "layer"
    assert hill.bbox is not None
    assert hill.bbox.model_dump() == pytest.approx({"x": 20, "y": 30, "width": 10, "height": 5})
    words = find_in_snapshot(SVG, text="ЛЕТНИЙ", settings=settings)
    assert [o.id for o in words.objects] == ["caption"]
    assert words.objects[0].bbox is not None
    locked = find_in_snapshot(SVG, id_prefix="locked-", settings=settings)
    assert locked.objects[0].locked
    limited = find_in_snapshot(SVG, limit=1, settings=settings)
    assert limited.truncated and limited.total_matches > limited.count == 1
    preview = preview_snapshot_object(SVG, "hill", result.fingerprint, width=200, settings=settings)
    with Image.open(tmp_path / preview.artifact_path) as image:
        assert image.size == (200, 100)
        assert image.convert("RGBA").getpixel((100, 50)) == (0, 128, 0, 255)
    with pytest.raises(LiveError, match="drawing changed"):
        preview_snapshot_object(
            SVG.replace('fill="green"', 'fill="red"'), "hill", result.fingerprint, settings=settings
        )


@pytest.mark.inkscape
@pytest.mark.skipif(shutil.which("inkscape") is None, reason="requires Inkscape CLI")
@pytest.mark.parametrize("aspect", ["none", "xMidYMid meet", "xMaxYMax slice"])
def test_real_engine_letterbox_bounds_are_in_document_coordinates(aspect: str) -> None:
    svg = (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="400" height="200" '
        f'viewBox="10 20 100 100" preserveAspectRatio="{aspect}">'
        '<rect id="r" x="20" y="30" width="10" height="5"/></svg>'
    )
    result = find_in_snapshot(svg)
    assert result.objects[0].bbox is not None
    assert result.objects[0].bbox.model_dump() == pytest.approx(
        {"x": 20, "y": 30, "width": 10, "height": 5}
    )
