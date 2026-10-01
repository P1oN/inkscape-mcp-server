"""Portable, root-qualified resource identifiers; never client-local filesystem links."""

from __future__ import annotations

import base64
import hashlib
from pathlib import Path

from pydantic import BaseModel

from inkscape_mcp.config import Settings, get_settings
from inkscape_mcp.workspace.limits import check_input_size
from inkscape_mcp.workspace.paths import SandboxViolation, owning_root, resolve_read_path


class ArtifactLink(BaseModel):
    root_id: str
    uri: str
    workspace_relative_path: str
    location: str = "server"


def root_id(root: Path) -> str:
    return hashlib.sha256(str(root).encode()).hexdigest()[:24]


def artifact_link(path: Path, settings: Settings | None = None) -> ArtifactLink:
    settings = settings if settings is not None else get_settings()
    resolved = resolve_read_path(path, settings)
    root = owning_root(resolved, settings.workspace_roots)
    if root is None:  # containment was checked above
        raise SandboxViolation("path rejected: outside workspace")
    relative = resolved.relative_to(root).as_posix()
    token = base64.urlsafe_b64encode(relative.encode()).decode().rstrip("=")
    return ArtifactLink(
        root_id=root_id(root),
        uri=f"inkscape://artifact/{root_id(root)}/{token}",
        workspace_relative_path=relative,
    )


def read_artifact(root_key: str, token: str) -> bytes:
    settings = get_settings()
    roots = {root_id(root): root for root in settings.workspace_roots}
    if root_key not in roots or len(token) > 8192:
        raise ValueError("artifact resource is not valid")
    try:
        relative = base64.b64decode(
            token + "=" * (-len(token) % 4), altchars=b"-_", validate=True
        ).decode()
    except (ValueError, UnicodeError) as exc:
        raise ValueError("artifact resource is not valid") from exc
    candidate = Path(relative)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise ValueError("artifact resource is not valid")
    resolved = resolve_read_path(roots[root_key] / candidate, settings)
    if owning_root(resolved, settings.workspace_roots) != roots[root_key]:
        raise ValueError("artifact root does not match")
    check_input_size(resolved, settings)
    return resolved.read_bytes()


def workspace_info() -> dict[str, object]:
    roots = get_settings().workspace_roots
    return {
        "roots": [
            {"root_id": root_id(root), "name": root.name, "location": "server"} for root in roots
        ],
        "relative_path_root_id": root_id(roots[0]) if roots else None,
        "relative_path_rule": ("Relative paths resolve against the first root, never client CWD."),
        "artifact_access": (
            "Read inkscape://artifact resource URIs through MCP; paths are server-side."
        ),
    }


def qualified_workspace_path(path: str, root_key: str) -> str:
    """Select a configured root explicitly without disclosing its absolute server path."""
    roots = {root_id(root): root for root in get_settings().workspace_roots}
    if root_key not in roots:
        raise SandboxViolation("workspace root ID not found; call get_workspace_info")
    candidate = Path(path)
    if candidate.is_absolute() or ".." in candidate.parts or not path.strip() or "\x00" in path:
        raise SandboxViolation("root-qualified path must be a relative path without '..'")
    return str(roots[root_key] / candidate)
