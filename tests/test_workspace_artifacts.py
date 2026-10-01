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


def test_artifact_read_uses_output_limit_and_keeps_import_limit(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    from inkscape_mcp.config import Settings
    from inkscape_mcp.workspace import artifacts
    from inkscape_mcp.workspace.limits import LimitExceeded, check_input_size

    settings = Settings(workspace_roots=[tmp_path.resolve()], max_input_bytes=4, max_output_bytes=8)
    monkeypatch.setattr(artifacts, "get_settings", lambda: settings)
    file = tmp_path / "export.bin"
    file.write_bytes(b"12345678")
    link = artifact_link(file, settings)
    token = link.uri.rsplit("/", 1)[1]
    assert read_artifact(link.root_id, token) == b"12345678"
    with pytest.raises(LimitExceeded):
        check_input_size(file, settings)
    file.write_bytes(b"123456789")
    with pytest.raises(LimitExceeded):
        read_artifact(link.root_id, token)


@pytest.mark.parametrize("replacement", ["file", "parent", "root"])
def test_artifact_rejects_symlink_swap_after_path_validation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, replacement: str
) -> None:
    from inkscape_mcp.config import Settings
    from inkscape_mcp.workspace import artifacts

    root = tmp_path / "workspace"
    (root / "sub").mkdir(parents=True)
    file = root / "sub" / "image.bin"
    file.write_bytes(b"safe")
    outside = tmp_path / "outside"
    (outside / "sub").mkdir(parents=True)
    (outside / "image.bin").write_bytes(b"secret")
    (outside / "sub" / "image.bin").write_bytes(b"secret")
    settings = Settings(workspace_roots=[root.resolve()])
    monkeypatch.setattr(artifacts, "get_settings", lambda: settings)
    link = artifact_link(file, settings)
    resolve = artifacts.resolve_read_path

    def swapped(path, settings):
        resolved = resolve(path, settings)
        target = (
            file if replacement == "file" else (file.parent if replacement == "parent" else root)
        )
        target.rename(target.with_name(target.name + "-original"))
        target.symlink_to(
            outside / "image.bin" if replacement == "file" else outside,
            target_is_directory=replacement != "file",
        )
        return resolved

    monkeypatch.setattr(artifacts, "resolve_read_path", swapped)
    with pytest.raises(OSError):
        read_artifact(link.root_id, link.uri.rsplit("/", 1)[1])


@pytest.mark.skipif(os.name == "nt", reason="Windows read handle denies concurrent writes")
def test_artifact_growth_after_fstat_remains_bounded(tmp_path: Path, monkeypatch) -> None:
    from inkscape_mcp.config import Settings
    from inkscape_mcp.workspace import artifacts
    from inkscape_mcp.workspace.limits import LimitExceeded

    settings = Settings(workspace_roots=[tmp_path.resolve()], max_output_bytes=4)
    monkeypatch.setattr(artifacts, "get_settings", lambda: settings)
    file = tmp_path / "image.bin"
    file.write_bytes(b"safe")
    link = artifact_link(file, settings)
    fstat = os.fstat

    def grow(fd):
        info = fstat(fd)
        file.write_bytes(b"larger than the cap")
        return info

    monkeypatch.setattr(artifacts.os, "fstat", grow)
    with pytest.raises(LimitExceeded):
        read_artifact(link.root_id, link.uri.rsplit("/", 1)[1])


@pytest.mark.skipif(os.name == "nt", reason="Windows read handle prevents replacement")
def test_artifact_reads_the_opened_file_after_path_replacement(tmp_path: Path, monkeypatch) -> None:
    from inkscape_mcp.config import Settings
    from inkscape_mcp.workspace import artifacts

    root = tmp_path / "workspace"
    root.mkdir()
    file = root / "image.bin"
    file.write_bytes(b"safe")
    outside = tmp_path / "secret.bin"
    outside.write_bytes(b"secret")
    settings = Settings(workspace_roots=[root.resolve()])
    monkeypatch.setattr(artifacts, "get_settings", lambda: settings)
    link = artifact_link(file, settings)
    fstat = os.fstat

    def swap_opened_file(fd):
        info = fstat(fd)
        file.rename(root / "original.bin")
        file.symlink_to(outside)
        return info

    monkeypatch.setattr(artifacts.os, "fstat", swap_opened_file)
    assert read_artifact(link.root_id, link.uri.rsplit("/", 1)[1]) == b"safe"


@pytest.mark.skipif(os.name == "nt", reason="POSIX named pipe")
def test_artifact_rejects_named_pipe_without_blocking(tmp_path: Path, monkeypatch) -> None:
    from inkscape_mcp.config import Settings
    from inkscape_mcp.workspace import artifacts

    settings = Settings(workspace_roots=[tmp_path.resolve()])
    monkeypatch.setattr(artifacts, "get_settings", lambda: settings)
    pipe = tmp_path / "pipe"
    os.mkfifo(pipe)
    link = artifact_link(pipe, settings)
    with pytest.raises(ValueError, match="regular file"):
        read_artifact(link.root_id, link.uri.rsplit("/", 1)[1])
