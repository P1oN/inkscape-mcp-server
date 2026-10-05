use inkscape_mcp_rust::helper_svg::{
    fingerprint, oneshot,
    package::{self, Change},
};
use serde_json::json;
use std::collections::BTreeMap;
const NONCE: &str = "mcp_12345678901234567890123456789012";
const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg"><defs><mask id="mask"><rect id="mask-r" fill="white" width="20" height="20"/></mask><pattern id="pattern" width="5" height="5" patternUnits="userSpaceOnUse"><circle id="dot" r="2"/></pattern></defs><text id="text" mask="url(#mask)" fill="url(#pattern)"><tspan id="run">old</tspan></text><use id="clone" href="#text"/></svg>"##;
fn style(color: &str) -> Change {
    Change::Style {
        style: BTreeMap::from([("fill".into(), color.into())]),
        transform: None,
    }
}
#[test]
fn package_validates_all_before_publication_preserves_resources_and_net_noop() {
    let edits = vec![
        style("red"),
        Change::Text {
            text: "new <&>".into(),
        },
    ];
    let applied = package::prepare(SVG, &["text".into()], &edits, NONCE, 65536).unwrap();
    let candidate = String::from_utf8(applied.bytes).unwrap();
    assert!(candidate.contains("new &lt;&amp;&gt;"));
    for s in [
        "id=\"mask\"",
        "id=\"pattern\"",
        "mask=\"url(#mask)\"",
        "href=\"#text\"",
        "id=\"run\"",
    ] {
        assert!(candidate.contains(s));
    }
    assert!(package::prepare(SVG, &["clone".into()], &edits, NONCE, 65536).is_err());
    let styled = SVG.replace(r##"fill="url(#pattern)""##, r#"style="fill:red""#);
    assert!(
        package::prepare(
            &styled,
            &["text".into()],
            &[style("blue"), style("red")],
            NONCE,
            65536
        )
        .unwrap()
        .bytes
        .is_empty()
    );
    assert!(
        package::prepare(SVG, &["text".into()], &vec![style("red"); 17], NONCE, 65536).is_err()
    );
}
#[test]
fn native_package_guard_checks_contents_ids_selection_and_rejects_mixed_payloads() {
    let doc = inkscape_mcp_rust::xml::parse(SVG.as_bytes(), 65536).unwrap();
    let ids = inkscape_mcp_rust::helper_svg::elements(doc.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect::<Vec<_>>();
    let request = json!({"nonce":NONCE,"expected_ids":ids,"expected_fingerprint":fingerprint::fingerprint(SVG,65536).unwrap(),"selection":["text"],"edits":[{"op":"style","style":{"fill":"red"}},{"op":"text","text":"new"}]});
    let prepare = |v| {
        oneshot::prepare(
            SVG,
            serde_json::from_value(v).unwrap(),
            &["text".into()],
            65536,
        )
    };
    assert!(!prepare(request.clone()).unwrap().0.bytes.is_empty());
    for (key, value) in [
        ("expected_fingerprint", json!("stale")),
        ("expected_ids", json!(["text"])),
        ("selection", json!(["clone"])),
        ("fragment", json!("<rect/>")),
        ("operation", json!("style")),
    ] {
        let mut bad = request.clone();
        bad[key] = value;
        assert!(prepare(bad).is_err(), "{key}");
    }
}
