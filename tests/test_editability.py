from lxml import etree

from inkscape_mcp.editability import EditabilityOptions, analyze_editability


def test_advice_is_configurable_and_semantics_explicit():
    root = etree.fromstring(
        b'<svg xmlns:i="http://www.inkscape.org/namespaces/inkscape">'
        b'<g id="cat" i:groupmode="layer"><g id="child" style="opacity:.5"/>'
        b'</g><g id="scene" i:groupmode="layer" i:label="Scene"/></svg>'
    )
    report = analyze_editability(
        root,
        EditabilityOptions(
            semantic_group_ids=["cat"], layer_advisory_threshold=1, max_group_depth=1
        ),
    )
    assert report.layer_count == 2
    assert any(a.code == "semantic_layer" and a.object_id == "cat" for a in report.advice)
    assert not any(a.code == "semantic_layer" and a.object_id == "scene" for a in report.advice)
    assert any(a.code == "structural_style_risk" and a.kind == "observation" for a in report.advice)
    assert not analyze_editability(root, EditabilityOptions(enabled=False)).advice
    default = analyze_editability(root, EditabilityOptions(check_labels=False))
    assert not any(
        a.code in {"missing_group_label", "many_layers", "semantic_layer"} for a in default.advice
    )
