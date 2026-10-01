"""Read-only discovery and previews from one exported live SVG snapshot.

Query geometry in Inkscape's output pixels, then invert the root viewBox mapping.
Never use the DOM's local attribute boxes for paths or transformed objects.
"""

from __future__ import annotations

import math
import re
import secrets
import tempfile
from pathlib import Path

from lxml import etree
from PIL import Image
from pydantic import BaseModel, Field

from inkscape_mcp.config import Settings, get_settings
from inkscape_mcp.document.inspect import BBox, ObjectInfo, _is_layer, _label_of
from inkscape_mcp.live.geometry import root_mapping
from inkscape_mcp.live.insert_payload import document_fingerprint
from inkscape_mcp.live.managed_scene import DRAWABLE, _visible, object_info
from inkscape_mcp.live.render import LiveRenderResult
from inkscape_mcp.live.session import get_session_manager
from inkscape_mcp.live.transport import LiveDocumentRef, LiveError
from inkscape_mcp.workspace import sandbox
from inkscape_mcp.workspace.limits import (
    check_export_dimensions,
    check_input_bytes_size,
    check_output_size,
)
from inkscape_mcp.workspace.subprocess_exec import ProcessError, run_inkscape
from inkscape_mcp.workspace.xml_safety import parse_svg_bytes


class LiveDiscoveryError(LiveError):
    """Stable, path-free discovery refusal with an actionable client message."""


class LiveObjectMatch(ObjectInfo):
    """Addressable visible object; bbox is engine geometry in document user units."""

    layer_id: str | None = None
    layer_label: str | None = None
    text: str | None = None
    locked: bool = False


class LiveFindResult(BaseModel):
    active_document: LiveDocumentRef | None = None
    fingerprint: str
    coordinate_space: str = "document_user_units"
    objects: list[LiveObjectMatch]
    count: int
    total_matches: int
    truncated: bool
    notes: list[str] = Field(default_factory=list)


def _snapshot(svg: str, settings: Settings) -> etree._Element:
    data = svg.encode("utf-8")
    check_input_bytes_size(data, settings)
    root = parse_svg_bytes(data).getroot()
    elems = list(root.iter())
    if len(elems) > 10000:
        raise LiveDiscoveryError("live discovery exceeds 10000 elements")
    ids: set[str] = set()
    for elem in elems:
        if not isinstance(elem.tag, str):
            if isinstance(elem, (etree._Entity, etree._ProcessingInstruction)):
                raise LiveDiscoveryError(
                    "entities and processing instructions are unsupported in discovery"
                )
            continue
        if etree.QName(elem).localname in {"script", "foreignObject", "style"}:
            raise LiveDiscoveryError(
                "scripts, foreignObject and stylesheets require snapshot preparation"
            )
        oid = elem.get("id")
        if oid:
            if oid in ids:
                raise LiveDiscoveryError("duplicate object ids make discovery ambiguous")
            ids.add(oid)
        for key, value in elem.attrib.items():
            value = str(value)
            name = etree.QName(key).localname
            if name == "base" or name.lower().startswith("on"):
                raise LiveDiscoveryError("active content is unsupported in discovery snapshots")
            if name == "href" and not value.startswith("#"):
                if not (
                    etree.QName(elem).localname == "image"
                    and value.startswith(("data:image/png;base64,", "data:image/jpeg;base64,"))
                ):
                    raise LiveDiscoveryError(
                        "external assets require a self-contained discovery snapshot"
                    )
            if name in {
                "style",
                "fill",
                "stroke",
                "filter",
                "clip-path",
                "mask",
                "cursor",
                "marker",
                "marker-start",
                "marker-mid",
                "marker-end",
            } and ("\\" in value or "/*" in value):
                raise LiveDiscoveryError("escaped CSS is unsupported in discovery snapshots")
            for target in re.findall(r"url\s*\((.*?)\)", value, re.IGNORECASE | re.DOTALL):
                if not target.strip(" \t\r\n\"'").startswith("#"):
                    raise LiveDiscoveryError(
                        "external paint references are unsupported in discovery"
                    )
    return root


def query_boxes(path: Path, root: etree._Element, settings: Settings) -> dict[str, BBox]:
    sx, sy, tx, ty = root_mapping(root)
    try:
        result = run_inkscape([str(path), "--query-all"], settings)
    except ProcessError as exc:
        raise LiveDiscoveryError("Inkscape CLI is required for accurate live geometry") from exc
    if result.timed_out or result.returncode:
        raise LiveDiscoveryError("accurate live geometry query failed or timed out")
    boxes: dict[str, BBox] = {}
    for line in result.stdout.splitlines():
        parts = line.rsplit(",", 4)
        if len(parts) != 5 or not parts[0]:
            continue
        try:
            x, y, w, h = (float(v) for v in parts[1:])
        except ValueError:
            continue
        values = ((x - tx) / sx, (y - ty) / sy, w / sx, h / sy)
        if not all(math.isfinite(v) for v in values) or min(w, h) < 0:
            continue
        boxes[parts[0]] = BBox(x=values[0], y=values[1], width=values[2], height=values[3])
    if not boxes:
        raise LiveDiscoveryError("Inkscape returned no usable object geometry")
    return boxes


def find_in_snapshot(
    svg: str,
    *,
    label: str | None = None,
    layer: str | None = None,
    text: str | None = None,
    tag: str | None = None,
    id_prefix: str | None = None,
    limit: int = 100,
    settings: Settings | None = None,
) -> LiveFindResult:
    """AND filters; labels/layer labels/text match case-insensitive substrings."""
    s = settings or get_settings()
    if not 1 <= limit <= 1000:
        raise LiveDiscoveryError("discovery limit must be between 1 and 1000")
    root = _snapshot(svg, s)
    with tempfile.TemporaryDirectory(prefix="imcp-discovery-") as directory:
        path = Path(directory) / "snapshot.svg"
        path.write_bytes(etree.tostring(root))
        boxes = query_boxes(path, root, s)
    matches = []
    for elem in root.iter():
        if not isinstance(elem.tag, str) or not elem.get("id"):
            continue
        name = etree.QName(elem).localname
        if elem is root or name not in DRAWABLE or not _visible(elem):
            continue
        ancestors = [elem, *elem.iterancestors()]
        owner = next((node for node in ancestors if _is_layer(node)), None)
        owner_id = owner.get("id") if owner is not None else None
        owner_label = _label_of(owner) if owner is not None else None
        own_label = _label_of(elem)
        words = (
            " ".join("".join(str(part) for part in elem.itertext()).split())
            if name in {"text", "tspan"}
            else None
        )
        if tag is not None and tag != name:
            continue
        if label is not None and label.casefold() not in (own_label or "").casefold():
            continue
        if layer is not None and not (
            layer == owner_id or layer.casefold() in (owner_label or "").casefold()
        ):
            continue
        if text is not None and text.casefold() not in (words or "").casefold():
            continue
        if id_prefix is not None and not elem.get("id", "").startswith(id_prefix):
            continue
        info = object_info(elem)
        info.bbox = boxes.get(info.id or "")
        matches.append(
            LiveObjectMatch(
                **info.model_dump(),
                layer_id=owner_id,
                layer_label=owner_label,
                text=words,
                locked=any(
                    node.get("{http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd}insensitive")
                    == "true"
                    for node in ancestors
                ),
            )
        )
    return LiveFindResult(
        fingerprint=document_fingerprint(root),
        objects=matches[:limit],
        count=min(limit, len(matches)),
        total_matches=len(matches),
        truncated=len(matches) > limit,
        notes=[
            "Bounds are Inkscape engine bounds converted to document user units.",
            "Paint remains explicit; semantic identity must be confirmed from a preview.",
        ],
    )


def find_live_objects(
    *,
    label: str | None = None,
    layer: str | None = None,
    text: str | None = None,
    tag: str | None = None,
    id_prefix: str | None = None,
    limit: int = 100,
) -> LiveFindResult:
    transport = get_session_manager().require_transport()
    with transport.operation_scope():
        doc = transport.get_active_document()
        svg = transport.get_document_svg()
    result = find_in_snapshot(
        svg, label=label, layer=layer, text=text, tag=tag, id_prefix=id_prefix, limit=limit
    )
    result.active_document = doc
    return result


def preview_snapshot_object(
    svg: str,
    object_id: str,
    expected_fingerprint: str,
    *,
    width: int = 512,
    settings: Settings | None = None,
) -> LiveRenderResult:
    """Isolated object export from the same fingerprinted snapshot, never GUI selection."""
    s = settings or get_settings()
    if not 1 <= width <= s.max_export_px:
        raise LiveDiscoveryError("preview width is outside the export dimension cap")
    if not re.fullmatch(r"[^,;\s\x00-\x1f\x7f]+", object_id):
        raise LiveDiscoveryError("object id is unsupported by isolated export")
    root = _snapshot(svg, s)
    if document_fingerprint(root) != expected_fingerprint:
        raise LiveDiscoveryError(
            "drawing changed since discovery; find the object again before preview"
        )
    candidates = [e for e in root.iter() if isinstance(e.tag, str) and e.get("id") == object_id]
    if (
        not candidates
        or etree.QName(candidates[0]).localname not in DRAWABLE
        or not _visible(candidates[0])
    ):
        raise LiveDiscoveryError("visible object id not found")
    if not s.workspace_roots:
        raise LiveDiscoveryError("no workspace root configured for object previews")
    workspace = s.workspace_roots[0].resolve()
    sandbox.ensure_live_dirs(workspace)
    out = sandbox.live_artifacts_dir(workspace) / f"live-object-{secrets.token_hex(12)}.png"
    if not out.resolve().parent.is_relative_to(workspace):
        raise LiveDiscoveryError("live artifacts path escaped the workspace")
    with tempfile.TemporaryDirectory(prefix="imcp-preview-") as directory:
        path, png = Path(directory) / "snapshot.svg", Path(directory) / "object.png"
        path.write_bytes(etree.tostring(root))
        box = query_boxes(path, root, s).get(object_id)
        if box is None or min(box.width, box.height) <= 0:
            raise LiveDiscoveryError("object has no nonempty engine bounds")
        sx, sy, _, _ = root_mapping(root)
        height = math.ceil(width * (box.height * sy) / (box.width * sx))
        check_export_dimensions(width, height, s)
        try:
            result = run_inkscape(
                [
                    str(path),
                    f"--export-id={object_id}",
                    "--export-id-only",
                    "--export-type=png",
                    f"--export-width={width}",
                    f"--export-filename={png}",
                ],
                s,
            )
        except ProcessError as exc:
            raise LiveDiscoveryError("Inkscape CLI is required for object previews") from exc
        if result.timed_out or result.returncode or not png.is_file():
            raise LiveDiscoveryError("object preview failed or timed out")
        check_output_size(png, s)
        with Image.open(png) as image:
            if image.format != "PNG":
                raise LiveDiscoveryError("object renderer did not produce PNG")
            check_export_dimensions(image.width, image.height, s)
        pending = out.with_suffix(".tmp")
        try:
            pending.write_bytes(png.read_bytes())
            pending.replace(out)
        finally:
            pending.unlink(missing_ok=True)
    return LiveRenderResult(
        artifact_path=out.relative_to(workspace).as_posix(),
        format="png",
        size_bytes=out.stat().st_size,
    )


def preview_live_object(
    object_id: str, expected_fingerprint: str, width: int = 512
) -> LiveRenderResult:
    transport = get_session_manager().require_transport()
    with transport.operation_scope():
        svg = transport.get_document_svg()
    return preview_snapshot_object(svg, object_id, expected_fingerprint, width=width)
