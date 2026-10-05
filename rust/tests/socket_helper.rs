//! New Rust snapshot invariants, without a Python parity oracle or GUI.
use inkscape_mcp_rust::helper_svg::socket::Snapshot;
use serde_json::json;
fn drawing() -> String {
    "<svg xmlns='http://www.w3.org/2000/svg' width='100mm' viewBox='0 0 100 80'><g id='g'><rect id='r' width='20' height='10' style='fill:red'/></g><text id='t'>Hi</text></svg>".into()
}
#[test]
fn snapshot_reads_keep_metadata_selection_and_null_viewport() {
    let s = Snapshot::new(drawing(), vec!["r".into()], Some("drawing.svg".into())).unwrap();
    assert_eq!(
        s.canvas().unwrap(),
        json!({"width":100.,"height":80.,"units":null})
    );
    assert_eq!(
        s.read("get_selection", &json!({})).unwrap(),
        json!({"object_ids":["r"]})
    );
    assert_eq!(
        s.read("get_active_document", &json!({})).unwrap()["object_count"],
        3
    );
    assert_eq!(
        s.read("inspect_selection", &json!({})).unwrap()["objects"][0]["tag"],
        "rect"
    );
    let token = s.read("get_state_token", &json!({})).unwrap();
    assert_eq!(token["revision"].as_str().unwrap().len(), 64);
    assert!(token["viewport"]["zoom"].is_null());
    assert!(
        !s.read("set_viewport", &json!({"mode":"fit_page"})).unwrap()["applied"]
            .as_bool()
            .unwrap()
    );
    assert!(!s.changed());
}
#[test]
fn refused_edits_and_matching_values_preserve_exact_input() {
    let mut s = Snapshot::new(drawing(), vec!["r".into()], None).unwrap();
    let before = s.svg.clone();
    s.mutate("apply_to_selection", &json!({"style":{"fill":"red"}}))
        .unwrap();
    assert_eq!(s.svg, before);
    assert!(!s.changed());
    for params in [
        json!({"style":{"made-up":"x"}}),
        json!({"transform":"translate(NaN)"}),
        json!({"style":{"fill":42}}),
    ] {
        assert!(s.mutate("apply_to_selection", &params).is_err());
        assert_eq!(s.svg, before);
    }
    assert!(
        s.mutate("set_selected_text", &json!({"text":"wrong element"}))
            .is_err()
    );
    assert_eq!(s.svg, before);
    s.mutate(
        "apply_to_selection",
        &json!({"style":{"fill":"blue"},"transform":"translate(3,4)"}),
    )
    .unwrap();
    assert!(s.changed());
    assert!(s.svg.contains("blue"));
    let after = s.svg.clone();
    s.mutate("apply_to_selection", &json!({"style":{"fill":"blue"}}))
        .unwrap();
    assert_eq!(s.svg, after);
}
#[test]
fn text_is_literal_and_insertion_has_remapped_editable_geometry() {
    let mut s = Snapshot::new(drawing(), vec!["t".into()], None).unwrap();
    s.mutate("set_selected_text", &json!({"text":"A & <B>"}))
        .unwrap();
    let d = inkscape_mcp_rust::xml::parse(s.svg.as_bytes(), 10000).unwrap();
    let t = inkscape_mcp_rust::helper_svg::elements(d.get_root_element().unwrap())
        .into_iter()
        .find(|n| n.get_property_no_ns("id").as_deref() == Some("t"))
        .unwrap();
    assert_eq!(t.get_content(), "A & <B>");
    let after = s.svg.clone();
    s.mutate("set_selected_text", &json!({"text":"A & <B>"}))
        .unwrap();
    assert_eq!(s.svg, after);
    let result = s
        .mutate(
            "insert_svg",
            &json!({"svg":"<g id='a'><rect id='b' width='2' height='3'/><use href='#b'/></g>"}),
        )
        .unwrap();
    assert!(result["affected_ids"].as_array().unwrap().len() >= 3);
    assert!(s.svg.contains("href=\"#mcp_"));
    let before = s.svg.clone();
    assert!(
        s.mutate(
            "insert_svg",
            &json!({"svg":"<image href='file:///secret'/>"})
        )
        .is_err()
    );
    assert_eq!(s.svg, before);
}
#[test]
fn snapshot_refuses_ambiguous_selection_and_growth() {
    assert!(Snapshot::new(drawing(), vec!["missing".into()], None).is_err());
    assert!(Snapshot::new(drawing(), vec!["r".into(), "r".into()], None).is_err());
    assert!(
        Snapshot::new(
            "<svg xmlns='http://www.w3.org/2000/svg'><rect id='x'/><rect id='x'/></svg>".into(),
            vec![],
            None
        )
        .is_err()
    );
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg'>{}</svg>",
        "<rect/>".repeat(9999)
    );
    let mut s = Snapshot::new(svg.clone(), vec![], None).unwrap();
    assert!(
        s.mutate("insert_svg", &json!({"svg":"<rect width='1' height='1'/>"}))
            .is_err()
    );
    assert_eq!(s.svg, svg);
}
