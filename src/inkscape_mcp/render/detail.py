"""Fixed document-space PNG regions, including read-only historical snapshot rendering."""

from __future__ import annotations

import math
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field

from inkscape_mcp.config import get_settings
from inkscape_mcp.edit.dom import EditError, normalize_color
from inkscape_mcp.live.geometry import LiveGeometryError, root_mapping
from inkscape_mcp.registry import get_registry
from inkscape_mcp.render.cli import RenderResult, _run_and_finalize, _unique_token
from inkscape_mcp.snapshots import SnapshotNotFound, list_snapshots
from inkscape_mcp.workspace import sandbox
from inkscape_mcp.workspace.limits import check_export_dimensions, check_input_size
from inkscape_mcp.workspace.paths import resolve_read_path
from inkscape_mcp.workspace.xml_safety import parse_svg_file


class PreviewRegion(BaseModel):
    model_config = ConfigDict(allow_inf_nan=False, extra="forbid")
    x: float
    y: float
    width: float = Field(gt=0)
    height: float = Field(gt=0)


def source_path(doc_id: str, snapshot_id: str | None) -> Path:
    entry = get_registry().get(doc_id)
    if snapshot_id is None:
        path = Path(entry.working_path)
    else:
        matches = [s for s in list_snapshots(doc_id) if s.snapshot_id == snapshot_id]
        if not matches:
            raise SnapshotNotFound("snapshot id not found")
        path = sandbox.snapshots_dir(Path(entry.root), doc_id) / matches[0].file
    resolved = resolve_read_path(path)
    check_input_size(resolved)
    return resolved


def render_region(
    doc_id: str,
    region: PreviewRegion,
    width_px: int,
    background: str = "transparent",
    snapshot_id: str | None = None,
) -> RenderResult:
    settings = get_settings()
    source = source_path(doc_id, snapshot_id)
    root = parse_svg_file(source).getroot()
    if root.get("transform"):
        raise EditError("root transforms require preparation for document-space region export")
    try:
        sx, sy, tx, ty = root_mapping(root)
    except LiveGeometryError as exc:
        raise EditError(str(exc)) from exc
    bounds = (
        region.x * sx + tx,
        region.y * sy + ty,
        (region.x + region.width) * sx + tx,
        (region.y + region.height) * sy + ty,
    )
    projected_height = width_px * region.height * sy / (region.width * sx)
    if not all(math.isfinite(v) for v in (*bounds, projected_height)):
        raise EditError("region bounds or dimensions are non-finite")
    if not isinstance(width_px, int) or width_px <= 0:
        raise EditError("width_px must be a positive integer")
    height_px = max(1, math.ceil(projected_height))
    check_export_dimensions(width_px, height_px, settings)
    color = normalize_color(background)
    entry = get_registry().get(doc_id)
    sandbox.ensure_doc_dirs(Path(entry.root), doc_id)
    out = (
        sandbox.artifacts_dir(Path(entry.root), doc_id)
        / "preview"
        / f"detail-{_unique_token()}.png"
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    args = [
        str(source),
        "--export-type=png",
        f"--export-filename={out}",
        "--export-area=" + ":".join(format(v, ".17g") for v in bounds),
        f"--export-width={width_px}",
        f"--export-background={color}",
        "--export-background-opacity=" + ("0" if color == "transparent" else "1"),
    ]
    return _run_and_finalize(
        doc_id=doc_id,
        args=args,
        out=out,
        fmt="png",
        width_px=width_px,
        height_px=height_px,
        entry=entry,
        settings=settings,
        event="preview",
    )
