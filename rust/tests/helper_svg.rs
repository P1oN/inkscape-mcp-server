use inkscape_mcp_rust::{
    helper_svg::{
        affine,
        edit::{self, Request, Step},
        fingerprint, fragment,
    },
    xml,
};
use serde_json::json;
const NONCE: &str = "mcp_1234567890abcdef1234567890abcdef";
fn request(op: &str, ids: &[&str]) -> Request {
    serde_json::from_value(json!({"nonce":NONCE,"operation":op,"selection":ids})).unwrap()
}
fn plan(svg: &str, op: &str, ids: &[&str]) -> Result<edit::Plan, &'static str> {
    edit::plan(svg, &request(op, ids), 1_048_576)
}
#[test]
fn fingerprint_wire_is_frozen_and_prefix_independent() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../migration/contracts/effect-data-cases.json"
    ))
    .unwrap();
    for c in cases["fingerprints"].as_array().unwrap() {
        assert_eq!(
            fingerprint::fingerprint(c["svg"].as_str().unwrap(), 1_048_576).unwrap(),
            c["expected"].as_str().unwrap()
        );
    }
    let a = "<svg xmlns='urn:svg'><text id='t'> hi <tspan>世界</tspan> tail </text></svg>";
    let b = "<s:svg xmlns:s='urn:svg' id='ui' version='1'><s:metadata><s:path/></s:metadata><s:text id='t'> hi <s:tspan>世界</s:tspan> tail </s:text></s:svg>";
    assert_eq!(
        fingerprint::fingerprint(a, 4096),
        fingerprint::fingerprint(b, 4096)
    );
    assert_ne!(
        fingerprint::fingerprint(a, 4096),
        fingerprint::fingerprint(&a.replace("tail", "changed"), 4096)
    );
    assert!(fingerprint::fingerprint(a, 4).is_err());
}
#[test]
fn prepared_fragment_remaps_qualified_references_and_preserves_mixed_content() {
    let source = r##"<defs><linearGradient id="p"/></defs><text id="t" fill="url('#p')">one<tspan>two</tspan>tail</text><use xmlns:x="http://www.w3.org/1999/xlink" x:href="#t"/>"##;
    let (bytes, ids) = fragment::prepare(source, NONCE).unwrap();
    assert_eq!(ids, fragment::plan(source, NONCE).unwrap());
    let doc = xml::parse(&bytes, 4096).unwrap();
    let root = doc.get_root_element().unwrap();
    assert_eq!(root.get_name(), "g");
    assert_eq!(root.get_property_no_ns("id").as_deref(), Some(NONCE));
    let children = root.get_child_elements();
    assert_eq!(children[1].get_content(), "onetwotail");
    assert_eq!(
        children[1].get_property_no_ns("fill").unwrap(),
        format!("url(#{NONCE}_0)")
    );
    assert_eq!(
        children[2]
            .get_property_ns("href", "http://www.w3.org/1999/xlink")
            .unwrap(),
        format!("#{NONCE}_1")
    );
    for bad in [
        "<script/>",
        "<rect onclick='x'/>",
        "<use href='http://x'/>",
        "<rect fill='url(#missing)'/>",
        "<g id='x'/><g id='x'/>",
        "<!--comment--><rect/>",
        "<rect id=''/>",
        "<!DOCTYPE svg><rect/>",
        "<g xmlns='urn:foreign'/>",
    ] {
        assert!(fragment::prepare(bad, NONCE).is_err(), "{bad}");
    }
}
#[test]
fn selection_refusals_are_pure_and_bounded() {
    for (svg, ids, reason) in [
        (
            "<svg><rect id='r'/><rect id='r'/></svg>",
            vec!["r"],
            "document contains duplicate ids",
        ),
        (
            "<svg><rect id='r'/></svg>",
            vec!["missing"],
            "invalid selection",
        ),
        (
            "<svg><defs><rect id='r'/></defs></svg>",
            vec!["r"],
            "definitions are not editable selections",
        ),
        (
            "<svg xmlns:s='http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd'><g s:insensitive='true'><rect id='r'/></g></svg>",
            vec!["r"],
            "selection contains locked objects or belongs to a locked layer",
        ),
        (
            "<svg xmlns:i='http://www.inkscape.org/namespaces/inkscape'><g id='r' i:groupmode='layer'/></svg>",
            vec!["r"],
            "select objects inside the layer, not the layer itself",
        ),
    ] {
        let before = svg.to_string();
        assert_eq!(plan(svg, "delete", &ids).unwrap_err(), reason);
        assert_eq!(svg, before);
    }
    let mut r = request("delete", &["r", "r"]);
    assert!(edit::plan("<svg/>", &r, 4096).is_err());
    r.nonce = "mcp_BAD".into();
    assert!(edit::plan("<svg/>", &r, 4096).is_err());
    assert!(
        serde_json::from_value::<Request>(
            json!({"nonce":NONCE,"operation":"exec","selection":["r"]})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Request>(
            json!({"nonce":NONCE,"operation":"delete","selection":["r"],"script":"x"})
        )
        .is_err()
    );
    assert!(
        plan(
            &format!("<svg>{}</svg>", "<rect/>".repeat(10_000)),
            "delete",
            &["r"]
        )
        .is_err()
    );
}
#[test]
fn deletion_includes_descendants_and_conservative_reference_boundaries() {
    for attr in [
        "href='#child'",
        "style='fill:url(#child)'",
        "connection-start='#child'",
    ] {
        let svg = format!("<svg><g id='g'><rect id='child'/></g><use id='u' {attr}/></svg>");
        assert!(plan(&svg, "delete", &["g"]).is_err());
        assert!(plan(&svg, "delete", &["g", "child", "u"]).is_ok());
    }
    assert!(
        plan(
            "<svg><rect id='r'/><use href='#rx'/></svg>",
            "delete",
            &["r"]
        )
        .is_ok()
    );
    assert!(
        plan(
            "<svg><style>#r {fill:red}</style><rect id='r'/></svg>",
            "delete",
            &["r"]
        )
        .is_err()
    );
}
#[test]
fn plans_preserve_parent_child_selection_paint_order_and_noops() {
    let svg = "<svg><defs/><rect id='a'/><circle id='b'/><path id='c'/></svg>";
    assert!(!plan(svg, "back", &["a"]).unwrap().changed());
    assert!(!plan(svg, "front", &["c"]).unwrap().changed());
    assert_eq!(
        plan(svg, "front", &["b", "a"]).unwrap().steps,
        vec![Step::Order {
            parent_path: vec![],
            indices: vec![0, 3, 1, 2]
        }]
    );
    assert_eq!(
        plan(svg, "raise", &["a", "b"]).unwrap().steps,
        vec![Step::Order {
            parent_path: vec![],
            indices: vec![0, 3, 1, 2]
        }]
    );
    assert!(plan(svg, "group", &["a", "c"]).is_err());
    assert!(plan(svg, "group", &["a", "b"]).unwrap().changed());
    assert_eq!(
        plan(
            "<svg><g id='g'><rect id='r'/></g></svg>",
            "delete",
            &["g", "r"]
        )
        .unwrap()
        .steps,
        vec![Step::Delete {
            ids: vec!["g".into()]
        }]
    );
    assert!(
        plan(
            "<svg><g id='g' opacity='.5'><rect/></g></svg>",
            "ungroup",
            &["g"]
        )
        .is_err()
    );
    assert!(plan("<svg><style/><rect id='r'/></svg>", "duplicate", &["r"]).is_err());
}
#[test]
fn duplicate_mapping_is_deterministic_and_collision_safe() {
    let svg = "<svg><g id='g'><rect id='r'/></g><use id='u' href='#r'/></svg>";
    let p = plan(svg, "duplicate", &["g", "r", "u"]).unwrap();
    assert_eq!(
        p.affected_ids,
        vec![
            format!("{NONCE}_0"),
            format!("{NONCE}_1"),
            format!("{NONCE}_2")
        ]
    );
    let Step::Duplicate { remap, .. } = &p.steps[0] else {
        panic!()
    };
    assert_eq!(remap["r"], format!("{NONCE}_1"));
    assert!(
        plan(
            &format!("<svg><rect id='r'/><rect id='{NONCE}_0'/></svg>"),
            "duplicate",
            &["r"]
        )
        .is_err()
    );
}
#[test]
fn text_and_style_noops_keep_live_explicit_semantics() {
    let svg = "<svg><text id='t'><tspan>hello</tspan></text><rect id='r' style='fill:red'/></svg>";
    let mut r = request("text", &["t"]);
    r.text = Some("hello".into());
    assert!(!edit::plan(svg, &r, 4096).unwrap().changed());
    r.text = Some("世界".into());
    assert!(edit::plan(svg, &r, 4096).unwrap().changed());
    assert!(edit::plan("<svg><text id='t'>a<tspan>b</tspan></text></svg>", &r, 4096).is_err());
    r.text = Some("bad\ntext".into());
    assert!(edit::plan(svg, &r, 4096).is_err());
    let mut r = request("style", &["r"]);
    r.style.insert("fill".into(), "red".into());
    assert!(!edit::plan(svg, &r, 4096).unwrap().changed());
    assert!(
        edit::plan("<svg><rect id='r' fill='red'/></svg>", &r, 4096)
            .unwrap()
            .changed()
    );
    r.style.insert("fill".into(), "red;display:none".into());
    assert!(edit::plan(svg, &r, 4096).is_err());
}
#[test]
fn document_space_transform_uses_shared_affine_conjugation() {
    let mut r = request("style", &["r"]);
    r.transform = Some("translate(10,6)".into());
    let p = edit::plan(
        "<svg><g transform='scale(2)'><rect id='r'/></g></svg>",
        &r,
        4096,
    )
    .unwrap();
    let Step::Style {
        transform: Some(t), ..
    } = &p.steps[0]
    else {
        panic!()
    };
    assert_eq!(affine::parse(t).unwrap(), [1., 0., 0., 1., 5., 3.]);
    for svg in [
        "<svg><svg><rect id='r'/></svg></svg>",
        "<svg><g transform='scale(0)'><rect id='r'/></g></svg>",
        "<svg><rect id='r' style='transform:scale(2)'/></svg>",
        "<svg><style>rect{transform:scale(2)}</style><rect id='r'/></svg>",
    ] {
        assert!(edit::plan(svg, &r, 4096).is_err());
    }
    r.transform = Some("matrix(1e999,0,0,1,0,0)".into());
    assert!(edit::plan("<svg><rect id='r'/></svg>", &r, 4096).is_err());
}

#[test]
fn captured_state_guard_refuses_stale_content_ids_and_native_selection() {
    let svg = "<svg><rect id='r' fill='red'/><circle id='c'/></svg>";
    let fingerprint = fingerprint::fingerprint(svg, 4096).unwrap();
    let ids = vec!["r".into(), "c".into()];
    let selected = vec!["r".into()];
    assert!(edit::guard(svg, &fingerprint, &ids, &selected, &selected, 4096).is_ok());
    assert_eq!(
        edit::guard(
            &svg.replace("red", "blue"),
            &fingerprint,
            &ids,
            &selected,
            &selected,
            4096
        ),
        Err("drawing content changed before insertion")
    );
    assert_eq!(
        edit::guard(
            &svg.replace("id='c'", "id='x'"),
            &fingerprint,
            &ids,
            &selected,
            &selected,
            4096
        ),
        Err("document changed before insertion")
    );
    assert_eq!(
        edit::guard(svg, &fingerprint, &ids, &selected, &["c".into()], 4096),
        Err("selection changed before edit")
    );
    assert_eq!(
        edit::guard(
            &svg.replace("id='c'", "id='r'"),
            &fingerprint,
            &ids,
            &selected,
            &selected,
            4096
        ),
        Err("document contains duplicate ids")
    );
}
#[test]
fn ungroup_compensates_each_child_and_refuses_references() {
    let svg =
        "<svg><g id='g' transform='scale(2)'><rect id='r' transform='translate(3,4)'/></g></svg>";
    let p = plan(svg, "ungroup", &["g"]).unwrap();
    let Step::Ungroup { children, .. } = &p.steps[0] else {
        panic!()
    };
    assert_eq!(
        affine::parse(&children[0].1).unwrap(),
        [2., 0., 0., 2., 6., 8.]
    );
    assert!(
        plan(
            &svg.replace("</svg>", "<use href='#g'/></svg>"),
            "ungroup",
            &["g"]
        )
        .is_err()
    );
    assert!(plan("<svg><g id='g' xmlns:s='http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd'><rect s:insensitive='true'/></g></svg>","delete",&["g"]).is_err());
}

#[test]
fn multiplied_style_output_and_reference_scans_refuse_with_bounded_work() {
    let svg = format!(
        "<svg>{}</svg>",
        (0..100)
            .map(|i| format!("<rect id='r{i}'/>"))
            .collect::<String>()
    );
    let mut r = request("style", &["r0"]);
    r.selection = (0..100).map(|i| format!("r{i}")).collect();
    r.style.insert("fill".into(), "x".repeat(8000));
    assert_eq!(
        edit::plan(&svg, &r, 16384).unwrap_err(),
        "edit plan exceeds size cap"
    );
    let svg = format!(
        "<svg><g id='g'>{}</g><rect title='{}'/></svg>",
        (0..1000)
            .map(|i| format!("<rect id='r{i}'/>"))
            .collect::<String>(),
        "x".repeat(70_000)
    );
    assert_eq!(
        plan(&svg, "delete", &["g"]).unwrap_err(),
        "edit reference scan exceeds work cap"
    );
}
