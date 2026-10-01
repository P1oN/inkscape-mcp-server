"""Bounded reproducible placement; shares clone/ID remapping and the edit pipeline."""

from __future__ import annotations

import math
import random
from typing import Annotated, Literal

from lxml import etree
from pydantic import BaseModel, ConfigDict, Field, TypeAdapter, ValidationError

from inkscape_mcp.config import get_settings
from inkscape_mcp.document.inspect import XLINK_NS
from inkscape_mcp.edit.dom import (
    SAFE_ID_RE,
    SVG_NS,
    EditError,
    all_ids,
    deep_copy_with_new_ids,
    require_target,
)
from inkscape_mcp.edit.pipeline import MutateFn
from inkscape_mcp.edit.structure import (
    LABEL,
    composed,
    inverse,
    local,
    matrix_text,
    multiply,
    parse_transform,
    references,
    reject_stylesheets,
)

MAX_REPEAT = 1024


class Point(BaseModel):
    model_config = ConfigDict(allow_inf_nan=False, extra="forbid")
    x: float
    y: float


class PolylinePlacement(BaseModel):
    model_config = ConfigDict(allow_inf_nan=False, extra="forbid")
    kind: Literal["polyline"]
    points: list[Point] = Field(min_length=2, max_length=256)
    count: int | None = Field(default=None, ge=1, le=MAX_REPEAT)
    spacing: float | None = Field(default=None, gt=0)


class RectanglePlacement(BaseModel):
    model_config = ConfigDict(allow_inf_nan=False, extra="forbid")
    kind: Literal["rectangle"]
    x: float
    y: float
    width: float = Field(gt=0)
    height: float = Field(gt=0)
    count: int | None = Field(default=None, ge=1, le=MAX_REPEAT)
    columns: int | None = Field(default=None, ge=1, le=MAX_REPEAT)
    spacing_x: float | None = Field(default=None, gt=0)
    spacing_y: float | None = Field(default=None, gt=0)


Placement = Annotated[PolylinePlacement | RectanglePlacement, Field(discriminator="kind")]
_PLACEMENT: TypeAdapter[Placement] = TypeAdapter(Placement)


class RepeatVariation(BaseModel):
    model_config = ConfigDict(allow_inf_nan=False, extra="forbid")
    seed: int = 0
    offset: float = Field(default=0, ge=0, le=1000)
    scale: float = Field(default=0, ge=0, le=0.5)
    rotation_degrees: float = Field(default=0, ge=0, le=180)


class PlacementItem(BaseModel):
    x: float
    y: float
    rotation_degrees: float
    scale: float


def placement_plan(
    placement: Placement,
    orientation: Literal["fixed", "tangent"] = "fixed",
    variation: RepeatVariation | None = None,
) -> list[PlacementItem]:
    try:
        geometry = _PLACEMENT.validate_python(placement)
        jitter = RepeatVariation.model_validate(variation or RepeatVariation())
    except ValidationError as exc:
        raise EditError("invalid bounded placement/variation parameters") from exc
    if orientation not in {"fixed", "tangent"}:
        raise EditError("orientation must be fixed or tangent")
    positions: list[tuple[float, float, float]] = []
    if isinstance(geometry, PolylinePlacement):
        if (geometry.count is None) == (geometry.spacing is None):
            raise EditError("polyline requires exactly one of count or spacing")
        segments: list[tuple[Point, Point, float]] = []
        for start, end in zip(geometry.points[:-1], geometry.points[1:], strict=True):
            distance = math.hypot(end.x - start.x, end.y - start.y)
            if not math.isfinite(distance):
                raise EditError("non-finite polyline length")
            if distance > 0:
                segments.append((start, end, distance))
        length = sum(segment[2] for segment in segments)
        if not segments or not math.isfinite(length):
            raise EditError("polyline must have a finite nonzero length")
        if geometry.spacing is not None:
            ratio = length / geometry.spacing
            if not math.isfinite(ratio) or ratio >= MAX_REPEAT:
                raise EditError("placement exceeds 1024 instances")
            count = math.floor(ratio) + 1
            distances = [index * geometry.spacing for index in range(count)]
        else:
            count = geometry.count or 1
            distances = [index * length / (count - 1) if count > 1 else 0 for index in range(count)]
        cursor, accumulated = 0, 0.0
        for distance in distances:
            while cursor < len(segments) - 1 and distance > accumulated + segments[cursor][2]:
                accumulated += segments[cursor][2]
                cursor += 1
            start, end, segment_length = segments[cursor]
            t = min(1.0, (distance - accumulated) / segment_length)
            angle = math.degrees(math.atan2(end.y - start.y, end.x - start.x))
            positions.append(
                (start.x + (end.x - start.x) * t, start.y + (end.y - start.y) * t, angle)
            )
    else:
        if orientation == "tangent":
            raise EditError("rectangle placement supports fixed orientation only")
        if geometry.count is not None:
            if geometry.spacing_x is not None or geometry.spacing_y is not None:
                raise EditError("rectangle requires count or spacing, not both")
            columns = geometry.columns or math.ceil(math.sqrt(geometry.count))
            if columns > geometry.count:
                raise EditError("columns must not exceed count")
            rows = math.ceil(geometry.count / columns)
            dx, dy = geometry.width / columns, geometry.height / rows
            count = geometry.count
        else:
            if (
                geometry.spacing_x is None
                or geometry.spacing_y is None
                or geometry.columns is not None
            ):
                raise EditError("rectangle requires count or both spacings (without columns)")
            dx, dy = geometry.spacing_x, geometry.spacing_y
            ratios = (geometry.width / dx, geometry.height / dy)
            if any(not math.isfinite(v) or v > MAX_REPEAT for v in ratios):
                raise EditError("placement exceeds 1024 instances")
            columns, rows = math.floor(ratios[0]), math.floor(ratios[1])
            count = columns * rows
        if count < 1 or count > MAX_REPEAT:
            raise EditError("placement requires 1..1024 instances")
        positions = [
            (
                geometry.x + (index % columns + 0.5) * dx,
                geometry.y + (index // columns + 0.5) * dy,
                0,
            )
            for index in range(count)
        ]
    rng = random.Random(jitter.seed)  # noqa: S311 - reproducible artwork, not security
    plan = [
        PlacementItem(
            x=x + rng.uniform(-jitter.offset, jitter.offset),
            y=y + rng.uniform(-jitter.offset, jitter.offset),
            rotation_degrees=(angle if orientation == "tangent" else 0)
            + rng.uniform(-jitter.rotation_degrees, jitter.rotation_degrees),
            scale=1 + rng.uniform(-jitter.scale, jitter.scale),
        )
        for x, y, angle in positions
    ]
    if not all(
        math.isfinite(v)
        for item in plan
        for v in (item.x, item.y, item.rotation_degrees, item.scale)
    ):
        raise EditError("placement coordinates overflow")
    return plan


def make_repeat(
    object_id: str,
    plan: list[PlacementItem],
    mode: Literal["linked", "copies"],
    group_id: str,
    label: str,
    anchor: Point | None = None,
) -> MutateFn:
    if mode not in {"linked", "copies"}:
        raise EditError("mode must be linked or copies")
    if not SAFE_ID_RE.fullmatch(group_id) or len(label) > 1024:
        raise EditError("invalid group ID or label")
    if len(plan) < 1 or len(plan) > MAX_REPEAT:
        raise EditError("repeat requires 1..1024 instances")
    from inkscape_mcp.edit.text_object import make_rename_object

    make_rename_object(group_id, label=label)  # reuse bounded label validation
    origin = Point.model_validate(anchor or Point(x=0, y=0))

    def mutate(tree: etree._ElementTree) -> str:
        root = tree.getroot()
        source = require_target(root, object_id)
        parent = source.getparent()
        if parent is None or local(source) not in {
            "g",
            "rect",
            "circle",
            "ellipse",
            "path",
            "polygon",
            "polyline",
            "line",
            "text",
            "use",
        }:
            raise EditError("repeat source must be a graphical object with a parent")
        if local(parent) not in {"g", "svg"}:
            raise EditError("repeat source must be in a group/layer or document root")
        reject_stylesheets(root)
        if any(
            n.get("{http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd}insensitive") == "true"
            for n in [source, *source.iterancestors()]
        ):
            raise EditError("repeat source belongs to a locked object/layer")
        existing_ids = all_ids(root)
        if any(references(n) - existing_ids for n in source.iter()):
            raise EditError("source contains unresolved references")
        source_ids = [n.get("id") for n in source.iter() if n.get("id")]
        if len(source_ids) != len(set(source_ids)):
            raise EditError("source contains duplicate IDs")
        if group_id in existing_ids:
            raise EditError("repeat group ID already exists")
        if any(local(n) in {"animate", "animateTransform", "set", "svg"} for n in source.iter()):
            raise EditError("animated or nested viewport sources require preparation")
        current_size = len(etree.tostring(tree))
        instance_size = (
            len(etree.tostring(source)) + 16 * len(all_ids(source)) + 1024
            if mode == "copies"
            else 1024
        )
        if current_size + instance_size * len(plan) > get_settings().max_input_bytes:
            raise EditError("repeat projected document exceeds input size limit")
        parent_matrix = composed(parent)
        inv_parent = inverse(parent_matrix)
        original = parse_transform(source.get("transform", ""))
        source_matrix = composed(source)
        a, b, c, d, e, f = source_matrix
        ax, ay = a * origin.x + c * origin.y + e, b * origin.x + d * origin.y + f
        group = etree.Element(f"{{{SVG_NS}}}g", id=group_id)
        group.set(LABEL, label)
        parent.insert(parent.index(source) + 1, group)
        for index, item in enumerate(plan):
            delta = multiply(
                parse_transform(
                    f"translate({item.x},{item.y}) rotate({item.rotation_degrees}) "
                    f"scale({item.scale})"
                ),
                (1, 0, 0, 1, -ax, -ay),
            )
            compensation = multiply(multiply(inv_parent, delta), parent_matrix)
            if mode == "copies":
                clone, _ = deep_copy_with_new_ids(source, root, None)
                clone.set("transform", matrix_text(multiply(compensation, original)))
            else:
                new_id = f"{group_id}-instance-{index + 1}"
                if new_id in existing_ids:
                    raise EditError("generated linked instance ID conflicts with document")
                existing_ids.add(new_id)
                clone = etree.Element(f"{{{SVG_NS}}}use", id=new_id, href=f"#{object_id}")
                clone.set(f"{{{XLINK_NS}}}href", f"#{object_id}")
                clone.set("transform", matrix_text(compensation))
            group.append(clone)
        # Validate the complete disposable tree before any snapshot/write by the pipeline.
        if len(etree.tostring(tree)) > get_settings().max_input_bytes:
            raise EditError("repeated document exceeds the configured input size limit")
        return f"created {len(plan)} {mode} instances in named group {group_id!r}"

    return mutate
