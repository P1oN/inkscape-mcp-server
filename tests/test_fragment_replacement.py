from pathlib import Path

import pytest
from fastmcp.exceptions import ToolError
from lxml import etree

from inkscape_mcp.config import ENV_WORKSPACE_ROOTS, get_settings
from inkscape_mcp.registry import get_registry, reset_registry
from inkscape_mcp.snapshots import list_snapshots, restore_snapshot
from inkscape_mcp.tools.compose import replace_svg_fragment

NS = 'xmlns="http://www.w3.org/2000/svg"'


@pytest.fixture
def doc(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv(ENV_WORKSPACE_ROOTS, str(tmp_path))
    get_settings.cache_clear()
    reset_registry()
    source = tmp_path / "scene.svg"
    source.write_text(
        f'<svg {NS} viewBox="0 0 10 10"><g id="paw">'
        '<rect id="toe" width="2" height="2"/></g>'
        '<circle id="outside" r="3"/></svg>'
    )
    entry = get_registry().open_document(str(source))
    yield entry, Path(entry.working_path)
    get_settings.cache_clear()


def test_replace_preserves_scene_ids_and_one_snapshot(doc):
    entry, path = doc
    before = path.read_bytes()
    outside = etree.parse(str(path)).xpath('//*[@id="outside"]')[0]
    result = replace_svg_fragment(
        entry.doc_id,
        "paw",
        f'<g {NS}><rect id="toe" width="4" height="2"/></g>',
        approval_token="test",
    )
    tree = etree.parse(str(path))
    assert tree.xpath('//*[@id="paw"]/rect') == []  # namespace-qualified element
    assert tree.xpath('//*[@id="toe"]')[0].get("width") == "4"
    assert etree.tostring(tree.xpath('//*[@id="outside"]')[0]) == etree.tostring(outside)
    assert len(list_snapshots(entry.doc_id)) == 1
    restore_snapshot(entry.doc_id, result.snapshot_id)
    assert path.read_bytes() == before


@pytest.mark.parametrize(
    "fragment,reason",
    [
        (f'<g {NS} id="different"/>', "root ID"),
        (f'<g {NS}><rect id="outside"/></g>', "conflicts"),
        (f'<g {NS}><rect id="a"/><rect id="a"/></g>', "duplicate"),
        (f'<g {NS}><use href="#missing"/></g>', "unresolved"),
        (f"<g {NS}><script/></g>", "disallowed"),
        (f'<g {NS}><use href="https://example.org/x"/></g>', "disallowed"),
        (f'<g {NS} onload="evil()"/>', "disallowed"),
    ],
)
def test_failure_is_byte_identical_without_snapshot(doc, fragment, reason):
    entry, path = doc
    before = path.read_bytes()
    with pytest.raises(ToolError, match=reason):
        replace_svg_fragment(entry.doc_id, "paw", fragment, approval_token="test")
    assert path.read_bytes() == before
    assert not list_snapshots(entry.doc_id)


def test_external_references_require_retention_and_explicit_policy(doc):
    entry, path = doc
    path.write_bytes(path.read_bytes().replace(b"</svg>", b'<use href="#toe"/></svg>'))
    before = path.read_bytes()
    for policy in ("reject_changes", "allow_retained"):
        with pytest.raises(ToolError, match="removed"):
            replace_svg_fragment(
                entry.doc_id, "paw", f"<g {NS}/>", reference_policy=policy, approval_token="test"
            )
        assert path.read_bytes() == before
    fragment = f'<g {NS}><rect id="toe" width="3" height="2"/></g>'
    with pytest.raises(ToolError, match="externally referenced"):
        replace_svg_fragment(entry.doc_id, "paw", fragment, approval_token="test")
    assert path.read_bytes() == before
    assert replace_svg_fragment(
        entry.doc_id, "paw", fragment, reference_policy="allow_retained", approval_token="test"
    ).changed


def test_approval_required_and_identical_fragment_noop(doc):
    entry, path = doc
    fragment = f'<g {NS} id="paw"><rect id="toe" width="2" height="2"/></g>'
    with pytest.raises(ToolError, match="approval"):
        replace_svg_fragment(entry.doc_id, "paw", fragment)
    before = path.read_bytes()
    assert not replace_svg_fragment(entry.doc_id, "paw", fragment, approval_token="test").changed
    assert path.read_bytes() == before
    assert not list_snapshots(entry.doc_id)
