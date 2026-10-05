use inkscape_mcp_rust::{
    helper_svg::{
        apply,
        edit::{Operation, Request},
        elements, fingerprint,
    },
    xml,
};
use std::{collections::BTreeMap, fs, process::Command};
const NONCE: &str = "mcp_12345678901234567890123456789012";
const SVG: &str = r##"<?xml version="1.0"?><!--before--><?task keep?><!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:m="urn:metadata" m:keep="yes"><metadata>opaque<!--inside--></metadata><defs><linearGradient id="paint"/></defs><g id="parent" transform="translate(2,3)"><rect id="a" width="10" height="10" fill="url(#paint)"/> tail <!--between--><rect id="b" x="12" width="5" height="5"/><text id="t"><tspan id="span">old</tspan></text></g></svg><!--after-->"##;
fn request(operation: Operation, selection: &[&str]) -> Request {
    Request {
        nonce: NONCE.into(),
        operation,
        selection: selection.iter().map(|s| s.to_string()).collect(),
        style: BTreeMap::new(),
        transform: None,
        text: None,
    }
}
fn svg_node(bytes: &[u8], id: &str) -> String {
    let d = xml::parse(bytes, 100_000).unwrap();
    let n = elements(d.get_root_element().unwrap())
        .into_iter()
        .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
        .unwrap();
    d.node_to_string(&n)
}
#[test]
fn applies_all_ten_operations_preserving_full_xml_and_unrelated_text() {
    for operation in [
        Operation::Style,
        Operation::Text,
        Operation::Duplicate,
        Operation::Delete,
        Operation::Group,
        Operation::Ungroup,
        Operation::Raise,
        Operation::Lower,
        Operation::Front,
        Operation::Back,
    ] {
        let target = match operation {
            Operation::Text => "t",
            Operation::Ungroup => "parent",
            Operation::Lower | Operation::Back => "b",
            _ => "a",
        };
        let mut r = request(operation, &[target]);
        r.style.insert("opacity".into(), "0.5".into());
        r.text = Some("new & <text>".into());
        let a = apply::edit(SVG, &r, 100_000).unwrap();
        assert!(!a.bytes.is_empty(), "{operation:?}");
        let text = String::from_utf8(a.bytes.clone()).unwrap();
        for kept in [
            "<!--before-->",
            "<?task keep?>",
            "<!DOCTYPE svg>",
            "<!--after-->",
            "m:keep=\"yes\"",
            "opaque<!--inside-->",
            "<!--between-->",
        ] {
            assert!(text.contains(kept), "{operation:?}: {kept}");
        }
        if operation != Operation::Text && operation != Operation::Ungroup {
            assert_eq!(svg_node(&a.bytes, "t"), svg_node(SVG.as_bytes(), "t"));
        }
        if operation == Operation::Text {
            assert!(text.contains("new &amp; &lt;text&gt;"));
        }
        if operation == Operation::Duplicate {
            assert!(text.contains(&format!("id=\"{NONCE}_0\"")));
            assert!(text.contains("url(#paint)"));
        }
    }
}
#[test]
fn duplicate_remaps_only_local_references_and_preserves_tails() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:m="urn:meta"><g id="source" m:id="metadata-id"><rect id="r"/><use id="u" xlink:href="#r" style="fill:url('#r')"/></g>tail<rect id="outside"/></svg>"##;
    let a = apply::edit(svg, &request(Operation::Duplicate, &["source"]), 100_000).unwrap();
    let s = String::from_utf8(a.bytes).unwrap();
    assert_eq!(s.matches("tail").count(), 2);
    assert_eq!(s.matches("m:id=\"metadata-id\"").count(), 2);
    assert!(s.contains(&format!("xlink:href=\"#{NONCE}_1\"")));
    assert!(s.contains("xlink:href=\"#r\""));
}
#[test]
fn group_ungroup_preserve_mixed_children_namespace_and_transform() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><g id="parent">leading<rect id="a"/>tail<rect id="b"/>end<!--comment--><?pi keep?></g></svg>"##;
    let grouped = apply::edit(svg, &request(Operation::Group, &["a", "b"]), 100_000).unwrap();
    let text = String::from_utf8(grouped.bytes).unwrap();
    assert!(text.contains(&format!(
        "<g id=\"{NONCE}\"><rect id=\"a\"/>tail<rect id=\"b\"/>end</g>"
    )));
    let ungrouped = apply::edit(svg, &request(Operation::Ungroup, &["parent"]), 100_000).unwrap();
    let text = String::from_utf8(ungrouped.bytes).unwrap();
    assert!(text.contains("leading"));
    assert!(text.contains("tail"));
    assert!(text.contains("end<!--comment--><?pi keep?>"));
}
#[test]
fn insertion_and_true_noop_keep_source_untouched() {
    let a = apply::insert(
        SVG,
        "<text id=\"word\">a<tspan>b</tspan>c</text>",
        NONCE,
        100_000,
    )
    .unwrap();
    let text = String::from_utf8(a.bytes).unwrap();
    assert!(text.contains("a<tspan>b</tspan>c"));
    assert!(text.contains("m:keep=\"yes\""));
    let mut r = request(Operation::Style, &["a"]);
    r.style.insert("opacity".into(), "0.5".into());
    let first = apply::edit(SVG, &r, 100_000).unwrap();
    let second = apply::edit(std::str::from_utf8(&first.bytes).unwrap(), &r, 100_000).unwrap();
    assert!(second.bytes.is_empty());
    assert!(
        apply::edit(SVG, &request(Operation::Back, &["a"]), 100_000)
            .unwrap()
            .bytes
            .is_empty()
    );
}
fn invoke(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_inkscape-mcp-inx"))
        .args(args)
        .env_clear()
        .env("INKSCAPE_MCP_MANAGED_DIR", root)
        .output()
        .unwrap()
}
#[test]
#[cfg(target_os = "macos")]
fn temporary_var_alias_is_accepted_but_other_links_are_refused() {
    use std::{os::unix::fs::symlink, path::Path};
    let link = fs::read_link("/var").unwrap();
    assert!(link == Path::new("/private/var") || link == Path::new("private/var"));
    let tmp = tempfile::tempdir_in("/private/var/tmp").unwrap();
    let root = tmp.path();
    let input = root.join("input.svg");
    fs::write(&input, SVG).unwrap();
    let d = xml::parse(SVG.as_bytes(), 100_000).unwrap();
    let ids: Vec<_> = elements(d.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    let req = serde_json::json!({"nonce":NONCE,"expected_ids":ids,"expected_fingerprint":fingerprint::fingerprint(SVG,100_000).unwrap(),"operation":"style","selection":["a"],"style":{"opacity":"0.5"}});
    fs::write(
        root.join("insert-request.json"),
        serde_json::to_vec(&req).unwrap(),
    )
    .unwrap();
    let alias = Path::new("/var").join(input.strip_prefix("/private/var").unwrap());
    let out = invoke(root, &["--id=a", alias.to_str().unwrap()]);
    assert!(out.status.success());
    assert!(!out.stdout.is_empty());
    let reply: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("insert-result.json")).unwrap()).unwrap();
    assert_eq!(reply["ok"], true);
    fs::remove_file(root.join("insert-result.json")).unwrap();
    symlink(&input, root.join("linked.svg")).unwrap();
    symlink(root, root.join("linked-dir")).unwrap();
    for path in [
        root.join("linked.svg"),
        root.join("linked-dir/input.svg"),
        Path::new("input.svg").to_owned(),
        root.join("../input.svg"),
    ] {
        let out = invoke(root, &["--id=a", path.to_str().unwrap()]);
        assert!(!out.status.success(), "{}", path.display());
        assert!(out.stdout.is_empty());
        assert!(!root.join("insert-result.json").exists());
    }
    assert_eq!(fs::read(input).unwrap(), SVG.as_bytes());
}

#[test]
fn many_targets_keep_ids_and_structure_after_indexed_application() {
    let count = 2000;
    let mut svg = String::from("<svg xmlns='http://www.w3.org/2000/svg'>");
    let ids: Vec<_> = (0..count).map(|i| format!("r{i}")).collect();
    for id in &ids {
        svg.push_str(&format!("<rect id='{id}' width='1' height='1'/>"));
    }
    svg.push_str("</svg>");
    for operation in [
        Operation::Style,
        Operation::Group,
        Operation::Duplicate,
        Operation::Delete,
    ] {
        let mut r = request(operation, &[]);
        r.selection = ids.clone();
        r.style.insert("opacity".into(), "0.5".into());
        let applied = apply::edit(&svg, &r, 2_000_000).unwrap();
        let doc = xml::parse(&applied.bytes, 2_000_000).unwrap();
        let root = doc.get_root_element().unwrap();
        let all = elements(root.clone());
        match operation {
            Operation::Style => {
                assert_eq!(root.get_child_elements().len(), count);
                assert!(
                    all.iter()
                        .skip(1)
                        .all(|n| n.get_property_no_ns("style").as_deref() == Some("opacity:0.5"))
                );
            }
            Operation::Group => {
                assert_eq!(root.get_child_elements().len(), 1);
                assert_eq!(
                    root.get_child_elements()[0].get_child_elements().len(),
                    count
                );
            }
            Operation::Duplicate => {
                assert_eq!(root.get_child_elements().len(), count * 2);
                assert_eq!(applied.ids.len(), count);
            }
            Operation::Delete => assert!(root.get_child_elements().is_empty()),
            _ => unreachable!(),
        }
        if operation != Operation::Delete {
            let remaining: std::collections::HashSet<_> = all
                .iter()
                .filter_map(|n| n.get_property_no_ns("id"))
                .collect();
            assert!(ids.iter().all(|id| remaining.contains(id)));
            assert!(applied.ids.iter().all(|id| remaining.contains(id)));
        }
    }
}

#[test]
fn fixed_cli_empty_path_guards_refusals_noop_and_atomic_results() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let svg = root.join("input.svg");
    fs::write(&svg, SVG).unwrap();
    let d = xml::parse(SVG.as_bytes(), 100_000).unwrap();
    let ids: Vec<_> = elements(d.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    let mut req = serde_json::json!({"nonce":NONCE,"expected_ids":ids,"expected_fingerprint":fingerprint::fingerprint(SVG,100_000).unwrap(),"operation":"style","selection":["a"],"style":{"opacity":"0.5"}});
    for (label, change) in [
        ("ids", 1),
        ("fingerprint", 2),
        ("selection", 3),
        ("valid", 0),
    ] {
        let mut v = req.clone();
        match change {
            1 => v["expected_ids"] = serde_json::json!([]),
            2 => v["expected_fingerprint"] = serde_json::json!("stale"),
            3 => v["selection"] = serde_json::json!(["b"]),
            _ => {}
        }
        fs::write(
            root.join("insert-request.json"),
            serde_json::to_vec(&v).unwrap(),
        )
        .unwrap();
        let out = invoke(&root, &["--id=a", svg.to_str().unwrap()]);
        assert!(out.status.success());
        assert!(out.stderr.is_empty());
        let reply: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("insert-result.json")).unwrap()).unwrap();
        assert_eq!(reply["ok"], change == 0, "{label}");
        assert_eq!(out.stdout.is_empty(), change != 0);
        assert_eq!(fs::read(&svg).unwrap(), SVG.as_bytes());
        if change == 0 {
            fs::write(&svg, &out.stdout).unwrap();
            req["expected_fingerprint"] = reply["fingerprint"].clone();
        }
    }
    fs::write(
        root.join("insert-request.json"),
        serde_json::to_vec(&req).unwrap(),
    )
    .unwrap();
    assert!(
        invoke(&root, &["--id=a", svg.to_str().unwrap()])
            .stdout
            .is_empty()
    );
    // A linked result is never followed or replaced; no output is emitted on publication failure.
    fs::remove_file(root.join("insert-result.json")).unwrap();
    std::os::unix::fs::symlink(&svg, root.join("insert-result.json")).unwrap();
    let old = fs::read(&svg).unwrap();
    let out = invoke(&root, &["--id=a", svg.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert_eq!(fs::read(&svg).unwrap(), old);
}

#[test]
fn ungroup_refuses_css_transforms_and_viewports_before_application() {
    for child in [
        "<rect id=\"r\" style=\"transform:scale(2)\"/>",
        "<svg id=\"nested\"><rect/></svg>",
    ] {
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\"><g id=\"g\" transform=\"translate(10,20)\">{child}</g></svg>"
        );
        assert!(apply::edit(&svg, &request(Operation::Ungroup, &["g"]), 100_000).is_err());
    }
}
#[test]
#[ignore = "separate native CLI rendering gate; requires an explicitly supplied Inkscape executable"]
fn real_inkscape_render_preserves_group_and_ungroup_appearance() {
    let binary =
        std::env::var_os("INKSCAPE_MCP_ACCEPTANCE_INKSCAPE").expect("explicit native CLI required");
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><defs><linearGradient id="p"><stop stop-color="blue"/><stop offset="1" stop-color="red"/></linearGradient></defs><g id="g" transform="matrix(1,.2,.1,1,3,4)"><rect id="a" x="10" y="10" width="40" height="30" fill="url(#p)"/><circle id="b" cx="55" cy="40" r="20" fill="green"/></g></svg>"##;
    let mut pixels = vec![];
    for (name, source) in [
        ("before", svg.as_bytes().to_vec()),
        (
            "group",
            apply::edit(svg, &request(Operation::Group, &["a", "b"]), 100_000)
                .unwrap()
                .bytes,
        ),
        (
            "ungroup",
            apply::edit(svg, &request(Operation::Ungroup, &["g"]), 100_000)
                .unwrap()
                .bytes,
        ),
    ] {
        let input = root.join(format!("{name}.svg"));
        let output = root.join(format!("{name}.png"));
        fs::write(&input, source).unwrap();
        let status = Command::new(&binary)
            .arg(&input)
            .arg("--export-type=png")
            .arg(format!("--export-filename={}", output.display()))
            .arg("--export-width=400")
            .env("INKSCAPE_PROFILE_DIR", root.join("profile"))
            .output()
            .unwrap();
        assert!(
            status.status.success(),
            "{}",
            String::from_utf8_lossy(&status.stderr)
        );
        let decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(output).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut bytes).unwrap();
        pixels.push(bytes[..info.buffer_size()].to_vec());
    }
    assert_eq!(pixels[0], pixels[1], "group appearance");
    assert_eq!(pixels[0], pixels[2], "ungroup affine appearance");
}

#[test]
fn typed_wire_refuses_unknown_fields_and_cross_mode_payloads() {
    use inkscape_mcp_rust::helper_svg::oneshot;
    let d = xml::parse(SVG.as_bytes(), 100_000).unwrap();
    let ids: Vec<_> = elements(d.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    let mut v = serde_json::json!({"nonce":NONCE,"expected_ids":ids,"expected_fingerprint":fingerprint::fingerprint(SVG,100_000).unwrap(),"operation":"style","selection":["a"],"style":{"opacity":".5"}});
    v["execute"] = serde_json::json!("anything");
    assert!(serde_json::from_value::<oneshot::Request>(v.clone()).is_err());
    v.as_object_mut().unwrap().remove("execute");
    v["fragment"] = serde_json::json!("<rect/>");
    assert!(
        oneshot::prepare(
            SVG,
            serde_json::from_value(v).unwrap(),
            &["a".into()],
            100_000
        )
        .is_err()
    );
    let mut r = request(Operation::Style, &["a"]);
    r.style.insert("opacity".into(), "0".repeat(5000));
    assert!(apply::edit(SVG, &r, SVG.len() + 100).is_err());
}
