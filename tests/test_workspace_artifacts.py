import base64
import json
import os
from pathlib import Path

import pytest

from inkscape_mcp.config import ENV_WORKSPACE_ROOTS, get_settings
from inkscape_mcp.workspace.artifacts import artifact_link, read_artifact, root_id, workspace_info
from inkscape_mcp.workspace.paths import SandboxViolation


def test_multiple_roots_and_portable_read(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    roots = [tmp_path / "first", tmp_path / "second"]
    for root in roots:
        root.mkdir()
        (root / "same.svg").write_bytes(root.name.encode())
    monkeypatch.setenv(ENV_WORKSPACE_ROOTS, os.pathsep.join(map(str, roots)))
    get_settings.cache_clear()
    try:
        info = workspace_info()
        assert info["relative_path_root_id"] == root_id(roots[0])
        assert str(tmp_path) not in json.dumps(info)
        links = [artifact_link(root / "same.svg") for root in roots]
        assert links[0].uri != links[1].uri
        for link, root in zip(links, roots, strict=True):
            assert read_artifact(link.root_id, link.uri.rsplit("/", 1)[1]) == root.name.encode()
        for raw in ("../secret", str(tmp_path / "secret")):
            token = base64.urlsafe_b64encode(raw.encode()).decode().rstrip("=")
            with pytest.raises(ValueError):
                read_artifact(links[0].root_id, token)
        outside = tmp_path / "outside"
        outside.write_bytes(b"secret")
        (roots[0] / "escape").symlink_to(outside)
        token = base64.urlsafe_b64encode(b"escape").decode().rstrip("=")
        with pytest.raises(SandboxViolation):
            read_artifact(links[0].root_id, token)
    finally:
        get_settings.cache_clear()


def test_mcp_resource_save_and_root_selection(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    import asyncio

    from fastmcp import Client

    from inkscape_mcp.registry import reset_registry
    from inkscape_mcp.server import mcp, register_tools

    roots = [tmp_path / "one", tmp_path / "two"]
    for root in roots:
        root.mkdir()
        (root / "source.svg").write_bytes(
            b'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"/>'
        )
    monkeypatch.setenv(ENV_WORKSPACE_ROOTS, os.pathsep.join(map(str, roots)))
    get_settings.cache_clear()
    reset_registry()
    register_tools()

    async def run():
        async with Client(mcp) as client:
            info = await client.call_tool("get_workspace_info")
            second = info.structured_content["roots"][1]["root_id"]
            opened = await client.call_tool(
                "open_document", {"path": "source.svg", "root_id": second}
            )
            saved = await client.call_tool(
                "save_document_as",
                {
                    "doc_id": opened.structured_content["doc_id"],
                    "dest_path": "output/final.svg",
                    "root_id": second,
                },
            )
            link = saved.structured_content["artifact"]
            contents = await client.read_resource(link["uri"])
            assert base64.b64decode(contents[0].blob) == (roots[1] / "source.svg").read_bytes()
            assert not (roots[0] / "output/final.svg").exists()
            assert str(tmp_path) not in json.dumps(saved.structured_content)
            (roots[1] / "output/final.svg").unlink()
            with pytest.raises(Exception, match="unavailable"):
                await client.read_resource(link["uri"])

    try:
        asyncio.run(run())
    finally:
        get_settings.cache_clear()
