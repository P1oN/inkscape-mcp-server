"""Read-only live object search and isolated preview tools."""

from __future__ import annotations

from fastmcp.exceptions import ToolError

from inkscape_mcp.live.discovery import (
    LiveDiscoveryError,
    LiveFindResult,
    find_live_objects,
    preview_live_object,
)
from inkscape_mcp.live.geometry import LiveGeometryError
from inkscape_mcp.live.render import LiveRenderResult
from inkscape_mcp.live.transport import LiveError
from inkscape_mcp.server import mcp
from inkscape_mcp.tools.live import _map_live_error
from inkscape_mcp.workspace.limits import LimitExceeded
from inkscape_mcp.workspace.xml_safety import UnsafeXMLError


@mcp.tool
def live_find_objects(
    label: str | None = None,
    layer: str | None = None,
    text: str | None = None,
    tag: str | None = None,
    id_prefix: str | None = None,
    limit: int = 100,
) -> LiveFindResult:
    """Find visible live objects by layer, label, text, tag or id prefix, with engine bounds.

    When to use: identify candidate elements before an artistic edit. Combine filters with AND;
    label, layer label and text use case-insensitive substring matching. Layer also matches an
    exact layer id; tag and id_prefix are case-sensitive. Omitted filters list visible objects.
    Confirm ambiguous candidates with live_preview_object rather than guessing from ids.

    Key params: limit is 1..1000, default 100. Requires a connected live session and Inkscape CLI.
    Works on one exported snapshot; it never changes the selection or drawing. Self-contained
    PNG/JPEG images are supported; external assets and stylesheets require preparation.

    Return shape: active_document, fingerprint, objects (ids, labels, layer, text, locked, bbox
    in DOCUMENT USER UNITS), total_matches and truncated. Paint remains explicit, not computed.
    A null bbox means the engine did not report this object; no approximate bounds are invented.

    Example: live_find_objects(layer="Hills", tag="path")

    Risk class: low (read-only snapshot search; no Undo step or approval).
    """
    try:
        return find_live_objects(
            label=label, layer=layer, text=text, tag=tag, id_prefix=id_prefix, limit=limit
        )
    except (LiveDiscoveryError, LiveGeometryError, LimitExceeded) as exc:
        raise ToolError(str(exc)) from exc
    except LiveError as exc:
        raise _map_live_error(exc) from exc
    except (UnsafeXMLError, OSError) as exc:
        raise ToolError("live discovery snapshot could not be inspected safely") from exc


@mcp.tool
def live_preview_object(
    object_id: str,
    expected_fingerprint: str,
    width: int = 512,
) -> LiveRenderResult:
    """Render one found live object in isolation without changing the GUI selection.

    When to use: confirm which hill, tree or other candidate live_find_objects found before editing.
    Uses a fresh exported snapshot and refuses if its drawing fingerprint differs from discovery.
    The preview preserves the object's ancestors, styles and referenced definitions through
    Inkscape's isolated export. The GUI selection and Undo history are untouched.

    Key params: object_id and expected_fingerprint from live_find_objects; width is a positive
    pixel width within the configured export cap. Requires Inkscape CLI and a workspace root.
    Comma, semicolon, whitespace or control characters in ids are unsupported by isolated export.

    Return shape: LiveRenderResult with workspace-relative PNG artifact_path and size_bytes.
    If the drawing changed, find again. This preview verifies appearance, not permission to edit.

    Example: live_preview_object(object_id="hill", expected_fingerprint="<discovery fingerprint>")

    Risk class: low (read-only snapshot render; no Undo step or approval).
    """
    try:
        return preview_live_object(object_id, expected_fingerprint, width)
    except (LiveDiscoveryError, LiveGeometryError, LimitExceeded) as exc:
        raise ToolError(str(exc)) from exc
    except LiveError as exc:
        raise _map_live_error(exc) from exc
    except (UnsafeXMLError, OSError) as exc:
        raise ToolError("live object preview could not be rendered safely") from exc
