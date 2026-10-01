from pathlib import Path

import pytest
from PIL import Image

from inkscape_mcp.config import ENV_WORKSPACE_ROOTS, get_settings
from inkscape_mcp.edit.dom import EditError
from inkscape_mcp.registry import get_registry, reset_registry
from inkscape_mcp.render.detail import PreviewRegion, render_region, source_path
from inkscape_mcp.snapshots import SnapshotNotFound, create_snapshot
from inkscape_mcp.workspace.limits import LimitExceeded


@pytest.fixture
def scene(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv(ENV_WORKSPACE_ROOTS, str(tmp_path))
    get_settings.cache_clear()
    reset_registry()
    svg = tmp_path / "scene.svg"
    svg.write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" '
        'viewBox="0 0 10 10"><rect id="r" x="2" y="7" width="2" height="2" '
        'fill="red"/></svg>'
    )
    entry = get_registry().open_document(str(svg))
    yield entry
    get_settings.cache_clear()


def test_bad_snapshot_and_dimensions_do_not_edit(scene):
    path = Path(scene.working_path)
    original = path.read_bytes()
    with pytest.raises(SnapshotNotFound):
        source_path(scene.doc_id, "../../secret")
    with pytest.raises(LimitExceeded):
        render_region(scene.doc_id, PreviewRegion(x=0, y=0, width=1, height=1), 100000)
    with pytest.raises(EditError):
        render_region(scene.doc_id, PreviewRegion(x=0, y=0, width=1, height=1), -2)
    assert path.read_bytes() == original


@pytest.mark.inkscape
def test_real_cli_crop_and_snapshot_have_equal_scale_and_background(scene):
    snapshot = create_snapshot(scene.doc_id)
    path = Path(scene.working_path)
    path.write_bytes(path.read_bytes().replace(b'fill="red"', b'fill="blue"'))
    current = path.read_bytes()
    region = PreviewRegion(x=2, y=7, width=2, height=2)
    before = render_region(scene.doc_id, region, 80, "white", snapshot.snapshot_id)
    after = render_region(scene.doc_id, region, 80, "white")
    images = [
        Image.open(Path(scene.root) / frame.workspace_relative_path).convert("RGBA")
        for frame in (before, after)
    ]
    assert images[0].size == images[1].size == (80, 80)
    assert images[0].getpixel((40, 40)) == (255, 0, 0, 255)
    assert images[1].getpixel((40, 40)) == (0, 0, 255, 255)
    assert path.read_bytes() == current
