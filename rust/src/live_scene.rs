//! Managed scene reads reuse the headless ObjectInfo/tree model; no computed CSS or viewport.
use crate::{document::elements, inspect, live_socket::Error};
use libxml::tree::Node;
use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};
static PIXELS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+(?:\.\d+)?(?:px)?$").unwrap());
static SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s,\x1c-\x1f]+").unwrap());
fn ancestors(node: &Node) -> Vec<Node> {
    let mut result = vec![node.clone()];
    let mut parent = node.get_parent();
    while let Some(p) = parent {
        if p.is_element_node() {
            result.push(p.clone());
        }
        parent = p.get_parent();
    }
    result
}
pub fn object_info(node: &Node) -> Value {
    let mut info = inspect::object_info(node);
    if ancestors(node).iter().skip(1).any(|p| {
        p.get_property_no_ns("transform")
            .is_some_and(|s| !s.is_empty())
    }) {
        info["bbox"] = Value::Null;
    }
    info
}
pub(crate) fn visible(node: &Node) -> bool {
    let ancestors = ancestors(node);
    for node in &ancestors {
        if matches!(
            node.get_name().as_str(),
            "defs" | "clipPath" | "mask" | "pattern" | "symbol" | "metadata"
        ) {
            return false;
        }
        let decls =
            inspect::parse_declarations(&node.get_property_no_ns("style").unwrap_or_default());
        let prop = |key: &str| {
            decls
                .get(key)
                .cloned()
                .or_else(|| node.get_property_no_ns(key))
        };
        if prop("display").as_deref() == Some("none")
            || matches!(prop("opacity").as_deref(), Some("0" | "0.0"))
        {
            return false;
        }
    }
    for node in &ancestors {
        let decls =
            inspect::parse_declarations(&node.get_property_no_ns("style").unwrap_or_default());
        if let Some(value) = decls
            .get("visibility")
            .cloned()
            .or_else(|| node.get_property_no_ns("visibility"))
            && !value.is_empty()
            && value != "inherit"
        {
            return !matches!(value.as_str(), "hidden" | "collapse");
        }
    }
    true
}
pub fn document_ref(root: &Node) -> Value {
    json!({"window_id":null,"document_id":null,"name":root.get_property_ns("docname","http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"),"path":null,"object_count":elements(root.clone()).len()-1})
}
pub fn scene(svg: &str, selected: &[String], cap: usize) -> Result<Value, Error> {
    let document = crate::xml::parse(svg.as_bytes(), cap)
        .map_err(|_| Error::Protocol("active document export is invalid"))?;
    let root = document
        .get_root_element()
        .ok_or(Error::Protocol("active document export is invalid"))?;
    let nodes = elements(root.clone());
    if nodes.len() > 10_000 {
        return Err(Error::Protocol(
            "live scene exceeds 10000 elements; use a document snapshot instead",
        ));
    }
    if selected.len() > 10_000 {
        return Err(Error::Protocol("live scene selection exceeds size cap"));
    }
    let by_id: HashMap<_, _> = nodes
        .iter()
        .filter_map(|n| {
            n.get_property_no_ns("id")
                .filter(|s| !s.is_empty())
                .map(|id| (id, n))
        })
        .collect();
    let selection: Vec<_> = selected
        .iter()
        .map(
            |id| json!({"id":id,"bbox":by_id.get(id).map(|node|object_info(node)["bbox"].clone())}),
        )
        .collect();
    let objects: Vec<_> = nodes
        .iter()
        .skip(1)
        .filter(|n| {
            matches!(
                n.get_name().as_str(),
                "g" | "rect"
                    | "circle"
                    | "ellipse"
                    | "path"
                    | "line"
                    | "polygon"
                    | "polyline"
                    | "text"
                    | "tspan"
                    | "image"
                    | "use"
                    | "svg"
            ) && visible(n)
        })
        .map(object_info)
        .collect();
    let viewbox = root.get_property_no_ns("viewBox").and_then(|s| {
        let values = SPLIT
            .split(s.trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')))
            .map(crate::decimal::float)
            .collect::<Option<Vec<_>>>()?;
        (values.len() == 4
            && values.iter().all(|n| n.is_finite())
            && values[2] > 0.0
            && values[3] > 0.0)
            .then_some(values)
    });
    let pixels = |name: &str| {
        root.get_property_no_ns(name)
            .filter(|s| PIXELS.is_match(s))
            .and_then(|s| crate::decimal::float(s.trim_end_matches("px")))
            .filter(|n| n.is_finite())
    };
    let canvas = json!({"width":viewbox.as_ref().map(|v|v[2]).or_else(||pixels("width")),"height":viewbox.as_ref().map(|v|v[3]).or_else(||pixels("height")),"units":if viewbox.is_some(){None}else{Some("px")},"viewbox":viewbox});
    Ok(
        json!({"active_document":document_ref(&root),"selection_count":selection.len(),"selection":selection,"viewport":{"zoom":null,"center":null,"visible_region":null},"canvas":canvas,"object_count":objects.len(),"visible_objects":objects,"tree":inspect::tree(root),"notes":["Visibility uses inline/presentation attributes; stylesheets are not computed.","Viewport is unavailable. Paint is explicit; geometry is attribute-derived."]}),
    )
}
pub fn inspection(svg: &str, selected: &[String], cap: usize) -> Result<Value, Error> {
    let document = crate::xml::parse(svg.as_bytes(), cap)
        .map_err(|_| Error::Protocol("active document export is invalid"))?;
    let root = document
        .get_root_element()
        .ok_or(Error::Protocol("active document export is invalid"))?;
    let ids: HashSet<_> = selected.iter().collect();
    let objects: Vec<_> = elements(root)
        .iter()
        .filter(|n| {
            n.get_property_no_ns("id")
                .is_some_and(|id| ids.contains(&id))
        })
        .map(object_info)
        .collect();
    Ok(json!({"count":objects.len(),"objects":objects}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_python_scene_and_inspection_fixtures_match_without_normalization() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/managed-scene-cases.json"
        ))
        .unwrap();
        for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            let selected = case["selected"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            let svg = case["svg"].as_str().unwrap();
            assert_eq!(
                scene(svg, &selected, 1024 * 1024).unwrap(),
                case["scene"],
                "scene {index}"
            );
            assert_eq!(
                inspection(svg, &selected, 1024 * 1024).unwrap(),
                case["inspection"],
                "inspection {index}"
            );
        }
    }
    #[test]
    fn native_scene_element_selection_and_xml_input_bounds_refuse() {
        let svg = format!("<svg>{}</svg>", "<rect/>".repeat(10_000));
        assert_eq!(
            scene(&svg, &[], 1024 * 1024),
            Err(Error::Protocol(
                "live scene exceeds 10000 elements; use a document snapshot instead"
            ))
        );
        assert!(scene("<svg/>", &vec!["r".into(); 10_001], 1024).is_err());
        assert!(scene("<svg>&external;</svg>", &[], 1024).is_err());
        assert!(inspection("<svg/>", &[], 3).is_err());
    }
}
