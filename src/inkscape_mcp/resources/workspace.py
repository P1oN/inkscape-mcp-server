"""Workspace discovery and sandboxed artifact readback."""

import json

from fastmcp.exceptions import ResourceError

from inkscape_mcp.server import mcp
from inkscape_mcp.workspace.artifacts import read_artifact, workspace_info
from inkscape_mcp.workspace.limits import LimitExceeded
from inkscape_mcp.workspace.paths import SandboxViolation


@mcp.resource("inkscape://workspace", mime_type="application/json")
def workspace() -> str:
    """Configured server roots as opaque identifiers and the relative-path anchor rule."""
    return json.dumps(workspace_info())


@mcp.resource("inkscape://artifact/{root_key}/{token}", mime_type="application/octet-stream")
def artifact(root_key: str, token: str) -> bytes:
    """Read a root-qualified artifact; sandbox and size limits apply on every read."""
    try:
        return read_artifact(root_key, token)
    except (ValueError, OSError, SandboxViolation, LimitExceeded) as exc:
        raise ResourceError("artifact unavailable or rejected by workspace/size policy") from exc
