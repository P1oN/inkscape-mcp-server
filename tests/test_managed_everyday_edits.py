"""Exercise edit planning with the installed vendor inkex, including coordinate semantics."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from typing import Any
from uuid import uuid4

import pytest
from lxml import etree

from inkscape_mcp.live.insert_payload import document_fingerprint

VENDOR = Path("/Applications/Inkscape.app/Contents/Resources/share/inkscape/extensions")
HELPER = Path(__file__).parents[1] / "src/inkscape_mcp/live/helper_extension/inkscape_mcp_edit.py"


@pytest.fixture
def planner(monkeypatch: pytest.MonkeyPatch) -> Any:
    if not (VENDOR / "inkex").is_dir():
        pytest.skip("requires vendor inkex; native acceptance supplies end-to-end coverage")
    monkeypatch.syspath_prepend(str(VENDOR))
    spec = importlib.util.spec_from_file_location("everyday_edits", HELPER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def drawing(xml: str) -> Any:
    import inkex

    return inkex.load_svg(
        f'<svg xmlns="http://www.w3.org/2000/svg" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd">{xml}</svg>'
    ).getroot()


def edit(planner: Any, root: Any, operation: str, selection: list[str], **params: Any) -> Any:
    before = etree.tostring(root)
    result = planner.plan_edit(
        root, dict(operation=operation, selection=selection, nonce="mcp_" + uuid4().hex, **params)
    )
    assert etree.tostring(root) == before  # planning never changes its input
    return result


def test_style_and_transform_in_document_coordinates(planner: Any) -> None:
    root = drawing(
        '<g id="layer" transform="translate(10,20) scale(2)">'
        '<rect id="a" width="10" height="10" transform="translate(3,4)"/>'
        '<rect id="other"/></g>'
    )
    other = etree.tostring(root.getElementById("other"))
    result, ids = edit(
        planner,
        root,
        "style",
        ["a"],
        style={"fill": "#123456", "stroke": "#654321", "stroke-width": "2", "opacity": "0.5"},
        transform="translate(6,8)",
    )
    original_point = root.getElementById("a").composed_transform().apply_to_point((0, 0))
    point = result.getElementById("a").composed_transform().apply_to_point((0, 0))
    assert tuple(point) == pytest.approx((original_point.x + 6, original_point.y + 8))
    assert result.getElementById("a").style["opacity"] == "0.5"
    assert etree.tostring(result.getElementById("other")) == other
    assert ids == ["a"]


@pytest.mark.parametrize("operation", ["style", "duplicate", "delete", "group"])
def test_locked_layer_refuses_without_changes(planner: Any, operation: str) -> None:
    root = drawing('<g sodipodi:insensitive="true"><rect id="a"/></g>')
    before = document_fingerprint(root)
    with pytest.raises(ValueError, match="locked"):
        edit(planner, root, operation, ["a"], style={"fill": "red"})
    assert document_fingerprint(root) == before


def test_parent_child_selection_is_transformed_once(planner: Any) -> None:
    root = drawing('<g id="a"><rect id="b" width="10" height="10"/></g>')
    result, ids = edit(planner, root, "style", ["a", "b"], transform="translate(5,0)")
    assert ids == ["a"]
    assert result.getElementById("b").get("transform") is None
    assert result.getElementById("b").composed_transform().apply_to_point((0, 0)).x == 5


def test_duplicate_remaps_descendant_references(planner: Any) -> None:
    root = drawing(
        '<g id="a"><rect id="b"/><use id="c" href="#b"/></g><use id="external" href="#b"/>'
    )
    result, ids = edit(planner, root, "duplicate", ["a"])
    assert len(ids) == 3 and len({e.get("id") for e in result.iter() if e.get("id")}) == 7
    assert result.getElementById(ids[2]).get("href") == "#" + ids[1]
    assert result.getElementById("external").get("href") == "#b"


def test_delete_referenced_objects_refuses(planner: Any) -> None:
    root = drawing('<rect id="a"/><use id="b" href="#a"/>')
    with pytest.raises(ValueError, match="referenced"):
        edit(planner, root, "delete", ["a"])
    result, ids = edit(planner, root, "delete", ["a", "b"])
    assert len(result) == 0 and ids == ["a", "b"]


def test_group_and_ungroup_preserve_geometry_and_sibling_order(planner: Any) -> None:
    root = drawing('<g transform="scale(2)"><rect id="a"/><rect id="b"/><rect id="c"/></g>')
    grouped, ids = edit(planner, root, "group", ["b", "a"])
    assert [e.get("id") for e in grouped[0]] == [ids[0], "c"]
    ungrouped, ids = edit(planner, grouped, "ungroup", [ids[0]])
    assert [e.get("id") for e in ungrouped[0]] == ["a", "b", "c"]
    assert tuple(ungrouped.getElementById("a").composed_transform().apply_to_point((1, 1))) == (
        2,
        2,
    )
    with pytest.raises(ValueError, match="consecutive"):
        edit(planner, root, "group", ["a", "c"])


def test_ungroup_preserves_group_transform_and_refuses_effects(planner: Any) -> None:
    root = drawing('<g id="a" transform="translate(10,20)"><rect id="b" transform="scale(2)"/></g>')
    result, _ = edit(planner, root, "ungroup", ["a"])
    assert tuple(result[0].composed_transform().apply_to_point((1, 1))) == (12, 22)
    root[0].set("opacity", ".5")
    with pytest.raises(ValueError, match="effects"):
        edit(planner, root, "ungroup", ["a"])


@pytest.mark.parametrize(
    "operation,expected",
    [
        ("front", ["a", "c", "b", "d"]),
        ("back", ["b", "d", "a", "c"]),
        ("raise", ["a", "c", "b", "d"]),
        ("lower", ["b", "a", "d", "c"]),
    ],
)
def test_stacking_preserves_selected_order(
    planner: Any, operation: str, expected: list[str]
) -> None:
    root = drawing("".join(f'<rect id="{i}"/>' for i in "abcd"))
    result, _ = edit(planner, root, operation, ["b", "d"])
    assert [e.get("id") for e in result] == expected


def test_text_preserves_single_run_formatting_and_refuses_mixed_runs(planner: Any) -> None:
    root = drawing(
        '<text id="a" style="font-size:20px">'
        '<tspan id="b" x="10" y="20" style="font-weight:bold">Old</tspan></text>'
    )
    result, _ = edit(planner, root, "text", ["a"], text="Новый & текст")
    assert result[0][0].text == "Новый & текст"
    assert result[0][0].attrib == root[0][0].attrib
    import inkex

    root[0].append(inkex.Tspan("another run"))
    with pytest.raises(ValueError, match="multiple text runs"):
        edit(planner, root, "text", ["a"], text="new")


def test_singular_parent_transform_refuses_without_changes(planner: Any) -> None:
    root = drawing('<g transform="scale(0)"><rect id="a"/></g>')
    before = document_fingerprint(root)
    with pytest.raises(ZeroDivisionError):
        edit(planner, root, "style", ["a"], transform="translate(5,0)")
    assert document_fingerprint(root) == before


def test_stylesheet_grouping_and_nested_viewport_transform_refuse(planner: Any) -> None:
    root = drawing('<style>g {fill: red}</style><rect id="a"/><rect id="b"/>')
    with pytest.raises(ValueError, match="stylesheets"):
        edit(planner, root, "group", ["a", "b"])
    root = drawing('<svg viewBox="0 0 10 10" width="100"><rect id="a"/></svg>')
    with pytest.raises(ValueError, match="nested SVG"):
        edit(planner, root, "style", ["a"], transform="translate(5,0)")


@pytest.mark.parametrize("container", ["a", "svg", "switch", "foreignObject", "flowRoot"])
@pytest.mark.parametrize("operation", ["front", "back", "raise", "lower"])
def test_stacking_counts_other_rendered_containers(
    planner: Any, container: str, operation: str
) -> None:
    sibling = f'<{container} id="b"><rect id="inside"/></{container}>'
    rect = '<rect id="a"/>'
    forward = operation in {"front", "raise"}
    root = drawing(rect + sibling if forward else sibling + rect)
    result, _ = edit(planner, root, operation, ["a"])
    assert [e.get("id") for e in result] == (["b", "a"] if forward else ["a", "b"])


@pytest.mark.parametrize("target", ["node", "parent"])
def test_stylesheet_transform_refuses_without_mutating_document(planner: Any, target: str) -> None:
    root = drawing(
        "<style>.moved {transform: translate(10px, 0)}</style>"
        f'<g class="{"moved" if target == "parent" else ""}">'
        f'<rect id="a" class="{"moved" if target == "node" else ""}"/></g>'
    )
    before = document_fingerprint(root)
    with pytest.raises(ValueError, match="CSS transforms"):
        edit(planner, root, "style", ["a"], transform="translate(5,0)")
    assert document_fingerprint(root) == before
