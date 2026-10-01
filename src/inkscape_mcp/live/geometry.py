"""Root SVG user-unit to Inkscape export-pixel mapping, including aspect alignment."""

from __future__ import annotations

import math
import re

from lxml import etree

from inkscape_mcp.document.inspect import _parse_style_decls
from inkscape_mcp.live.transport import LiveError


class LiveGeometryError(LiveError):
    """Stable refusal to infer an unsupported root coordinate mapping."""


_UNITS = {
    "": 1.0,
    "px": 1.0,
    "mm": 96 / 25.4,
    "cm": 96 / 2.54,
    "in": 96.0,
    "pt": 96 / 72,
    "pc": 16.0,
    "q": 96 / 101.6,
}
_LENGTH = re.compile(r"([+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)\s*([a-z]*)")


def _length(raw: str) -> float:
    match = _LENGTH.fullmatch(raw.strip())
    if not match or match[2] not in _UNITS:
        raise LiveGeometryError("accurate geometry requires absolute root width and height")
    value = float(match[1]) * _UNITS[match[2]]
    if not math.isfinite(value) or value <= 0:
        raise LiveGeometryError("invalid document dimensions for accurate geometry")
    return value


def root_mapping(root: etree._Element) -> tuple[float, float, float, float]:
    """Return document -> output pixel scale and translation, including letterboxing."""
    if {"width", "height"} & _parse_style_decls(root.get("style", "")).keys():
        raise LiveGeometryError("root CSS sizing requires snapshot preparation")
    if not root.get("viewBox"):
        return 1.0, 1.0, 0.0, 0.0
    try:
        x, y, w, h = [float(v) for v in re.split(r"[\s,]+", root.get("viewBox", "").strip())]
    except ValueError as exc:
        raise LiveGeometryError("invalid document viewBox") from exc
    if not all(math.isfinite(v) for v in (x, y, w, h)) or min(w, h) <= 0:
        raise LiveGeometryError("invalid document viewBox")
    width, height = _length(root.get("width", "")), _length(root.get("height", ""))
    sx, sy = width / w, height / h
    if not all(math.isfinite(v) and v > 0 for v in (sx, sy)):
        raise LiveGeometryError("invalid root coordinate mapping")
    parts = root.get("preserveAspectRatio", "xMidYMid meet").split()
    if parts and parts[0] == "defer":
        parts = parts[1:]
    if parts == ["none"]:
        tx, ty = -x * sx, -y * sy
        if not all(math.isfinite(v) for v in (tx, ty)):
            raise LiveGeometryError("invalid root coordinate mapping")
        return sx, sy, tx, ty
    if not parts or not re.fullmatch(r"x(Min|Mid|Max)Y(Min|Mid|Max)", parts[0]):
        raise LiveGeometryError("unsupported preserveAspectRatio")
    if len(parts) > 2 or (len(parts) == 2 and parts[1] not in {"meet", "slice"}):
        raise LiveGeometryError("unsupported preserveAspectRatio")
    scale = max(sx, sy) if parts[-1] == "slice" else min(sx, sy)
    align = {"Min": 0.0, "Mid": 0.5, "Max": 1.0}
    tx = (width - w * scale) * align[parts[0][1:4]] - x * scale
    ty = (height - h * scale) * align[parts[0][5:8]] - y * scale
    if not all(math.isfinite(v) for v in (scale, tx, ty)) or scale <= 0:
        raise LiveGeometryError("invalid root coordinate mapping")
    return scale, scale, tx, ty
