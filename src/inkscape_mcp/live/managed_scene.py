"""Read the exported live hierarchy without inventing viewport or computed CSS values."""

from __future__ import annotations

import math
import re

from lxml import etree

from inkscape_mcp.document.inspect import (
    ObjectInfo,
    _bbox_of,
    _build_node,
    _has_style,
    _is_layer,
    _is_leaf,
    _label_of,
    _paint_of,
    _parse_style_decls,
)
from inkscape_mcp.live.transport import (
    BBox,
    LiveDocumentRef,
    LiveScene,
    SceneCanvas,
    SceneSelectionItem,
)
from inkscape_mcp.workspace.xml_safety import parse_svg_bytes

DRAWABLE = frozenset(
    {
        "g",
        "rect",
        "circle",
        "ellipse",
        "path",
        "line",
        "polygon",
        "polyline",
        "text",
        "tspan",
        "image",
        "use",
        "svg",
    }
)
NON_RENDERED = frozenset({"defs", "clipPath", "mask", "pattern", "symbol", "metadata"})


def document_ref(root: etree._Element) -> LiveDocumentRef:
    """Use Inkscape's actual sodipodi namespace; never infer an on-disk path."""
    return LiveDocumentRef(
        name=root.get("{http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd}docname"),
        object_count=sum(1 for e in root.iter() if isinstance(e.tag, str)) - 1,
    )


def object_info(elem: etree._Element) -> ObjectInfo:
    parents = list(elem.iterancestors())
    bbox = None if any(p.get("transform") for p in parents) else _bbox_of(elem)
    return ObjectInfo(
        id=elem.get("id"),
        tag=etree.QName(elem).localname,
        label=_label_of(elem),
        has_style=_has_style(elem),
        paint=_paint_of(elem),
        is_layer=_is_layer(elem),
        is_leaf=_is_leaf(elem),
        bbox=bbox,
    )


def _visible(elem: etree._Element) -> bool:
    nodes = [elem, *elem.iterancestors()]
    for node in nodes:
        if etree.QName(node).localname in NON_RENDERED:
            return False
        style = _parse_style_decls(node.get("style", ""))
        if style.get("display", node.get("display")) == "none":
            return False
        if style.get("opacity", node.get("opacity")) in {"0", "0.0"}:
            return False
    # visibility can be overridden on a descendant, unlike display:none.
    for node in nodes:
        value = _parse_style_decls(node.get("style", "")).get("visibility", node.get("visibility"))
        if value and value != "inherit":
            return value not in {"hidden", "collapse"}
    return True


def scene_from_svg(svg: str, selected: list[str]) -> LiveScene:
    root = parse_svg_bytes(svg.encode()).getroot()
    elems = [e for e in root.iter() if isinstance(e.tag, str)]
    if len(elems) > 10000:
        from inkscape_mcp.live.transport import LiveError

        raise LiveError("live scene exceeds 10000 elements; use a document snapshot instead")
    by_id = {e.get("id"): e for e in elems if e.get("id")}
    selection = []
    for oid in selected:
        info = object_info(by_id[oid]) if oid in by_id else None
        bbox = BBox(**info.bbox.model_dump()) if info and info.bbox else None
        selection.append(SceneSelectionItem(id=oid, bbox=bbox))
    objects = [
        object_info(e)
        for e in elems
        if e is not root and etree.QName(e).localname in DRAWABLE and _visible(e)
    ]
    vb = None
    try:
        values = [float(v) for v in re.split(r"[\s,]+", root.get("viewBox", "").strip())]
        if len(values) == 4 and all(math.isfinite(v) for v in values) and min(values[2:]) > 0:
            vb = values
    except ValueError:
        pass

    def pixels(name: str) -> float | None:
        raw = root.get(name, "")
        if not re.fullmatch(r"\d+(?:\.\d+)?(?:px)?", raw):
            return None
        value = float(raw.removesuffix("px"))
        return value if math.isfinite(value) else None

    canvas = SceneCanvas(
        width=vb[2] if vb else pixels("width"),
        height=vb[3] if vb else pixels("height"),
        units=None if vb else "px",
        viewbox=vb,
    )
    return LiveScene(
        active_document=document_ref(root),
        selection=selection,
        selection_count=len(selection),
        canvas=canvas,
        visible_objects=objects,
        object_count=len(objects),
        tree=_build_node(root),
        notes=[
            "Visibility uses inline/presentation attributes; stylesheets are not computed.",
            "Viewport is unavailable. Paint is explicit; geometry is attribute-derived.",
        ],
    )
