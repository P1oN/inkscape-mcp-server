//! Read-only static computed paint and local resource relationships from a live snapshot.
use crate::{css, document::elements, live::Live, live_records, workspace::Workspace};
use libxml::tree::Node;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
const PROPS: &[&str] = &[
    "fill",
    "stroke",
    "color",
    "stroke-width",
    "fill-opacity",
    "stroke-opacity",
    "opacity",
    "display",
    "visibility",
    "mask",
    "clip-path",
    "filter",
    "marker-start",
    "marker-mid",
    "marker-end",
];
fn result(value: Result<String, String>) -> Value {
    match value {
        Ok(value) => json!({"certainty":"known","value":value}),
        Err(reason) => json!({"certainty":"unknown","value":null,"reason":reason}),
    }
}
fn computed(cascade: &css::Analysis, node: &Node, property: &str) -> Result<String, String> {
    let mut value = cascade.computed(node, property)?;
    if matches!(property, "fill" | "stroke") {
        match svgtypes::Paint::from_str(&value).map_err(|_| "Unsupported paint syntax.")? {
            svgtypes::Paint::CurrentColor => {
                return cascade.computed(node, "color").and_then(|v| {
                    svgtypes::Color::from_str(&v)
                        .map(|_| v)
                        .map_err(|_| "Unsupported currentColor value.".into())
                });
            }
            svgtypes::Paint::ContextFill | svgtypes::Paint::ContextStroke => {
                return Err("Context paint requires instance/marker rendered review.".into());
            }
            _ => {}
        }
    }
    if matches!(property, "opacity" | "fill-opacity" | "stroke-opacity") {
        let numeric = value
            .strip_suffix('%')
            .unwrap_or(&value)
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite());
        let numeric = numeric.ok_or("Unsupported opacity value.")?;
        let numeric = if value.ends_with('%') {
            numeric / 100.
        } else {
            numeric
        };
        value = numeric.clamp(0., 1.).to_string();
    }
    if property == "stroke-width" {
        let length = svgtypes::Length::from_str(&value).map_err(|_| "Unsupported stroke width.")?;
        if !length.number.is_finite()
            || length.number < 0.
            || !matches!(
                length.unit,
                svgtypes::LengthUnit::None | svgtypes::LengthUnit::Px
            )
        {
            return Err("Stroke width needs viewport/font/unit resolution.".into());
        }
    }
    match property {
        "color" => {
            svgtypes::Color::from_str(&value).map_err(|_| "Unsupported color value.")?;
        }
        "display"
            if !["none", "inline", "block", "contents", "inline-block"]
                .contains(&value.as_str()) =>
        {
            return Err("Unsupported display value.".into());
        }
        "visibility" if !["visible", "hidden", "collapse"].contains(&value.as_str()) => {
            return Err("Unsupported visibility value.".into());
        }
        "mask" | "clip-path" | "filter" | "marker-start" | "marker-mid" | "marker-end"
            if value != "none" =>
        {
            svgtypes::FuncIRI::from_str(&value)
                .map_err(|_| "Unsupported effect/reference value.")?;
        }
        _ => {}
    }
    Ok(value)
}
fn link(property: &str, value: &str) -> Result<Option<String>, String> {
    if property == "href" {
        return value
            .strip_prefix('#')
            .filter(|v| !v.is_empty())
            .map(|v| Some(v.into()))
            .ok_or_else(|| "External or empty reference is not inspected.".into());
    }
    if value == "none" {
        return Ok(None);
    }
    if matches!(property, "fill" | "stroke") {
        return match svgtypes::Paint::from_str(value) {
            Ok(svgtypes::Paint::FuncIRI(id, _)) => Ok(Some(id.into())),
            Ok(_) => Ok(None),
            Err(_) => Err("Unsupported paint reference syntax.".into()),
        };
    }
    svgtypes::FuncIRI::from_str(value)
        .map(|v| Some(v.0.into()))
        .map_err(|_| "Unsupported resource reference syntax.".into())
}
pub fn describe(node: &Node, cascade: &css::Analysis, by_id: &HashMap<String, Node>) -> Value {
    let mut paint = serde_json::Map::new();
    let mut relationships = Vec::new();
    let mut unknowns = Vec::new();
    let mut pending = Vec::new();
    for property in PROPS {
        let value = computed(cascade, node, property);
        if let Ok(value) = &value
            && [
                "fill",
                "stroke",
                "mask",
                "clip-path",
                "filter",
                "marker-start",
                "marker-mid",
                "marker-end",
            ]
            .contains(property)
        {
            match link(property, value) {
                Ok(Some(id)) => pending.push((node.clone(), property.to_string(), id)),
                Err(reason) => unknowns.push(json!({"property":property,"reason":reason})),
                _ => {}
            }
        }
        paint.insert((*property).into(), result(value));
    }
    // Ancestor effects composite the subtree; they are not inherited properties.
    let mut effects = Vec::new();
    let mut ancestor = node.get_parent().filter(Node::is_element_node);
    for _ in 0..128 {
        let Some(n) = ancestor else {
            break;
        };
        let mut effect = serde_json::Map::new();
        for property in ["opacity", "display", "mask", "clip-path", "filter"] {
            let value = computed(cascade, &n, property);
            if let Ok(value) = &value
                && ["mask", "clip-path", "filter"].contains(&property)
            {
                match link(property, value) {
                    Ok(Some(id)) => pending.push((n.clone(), property.into(), id)),
                    Err(reason) => unknowns.push(json!({"property":property,"reason":reason})),
                    _ => {}
                }
            }
            effect.insert(property.into(), result(value));
        }
        effects.push(json!({"object_id":n.get_property_no_ns("id"),"properties":effect}));
        ancestor = n.get_parent().filter(Node::is_element_node);
    }
    if ancestor.is_some() {
        unknowns.push(json!({"reason":"Ancestor relationship depth exceeded."}));
    }
    if let Some(href) = node
        .get_property_no_ns("href")
        .or_else(|| node.get_property_ns("href", "http://www.w3.org/1999/xlink"))
    {
        match link("href", &href) {
            Ok(Some(id)) => pending.push((node.clone(), "href".into(), id)),
            Err(reason) => unknowns.push(json!({"property":"href","reason":reason})),
            _ => {}
        }
    }
    let mut seen = HashSet::new();
    let mut truncated = false;
    while let Some((source, property, id)) = pending.pop() {
        let source_id = source.get_property_no_ns("id");
        if !seen.insert((source_id.clone(), property.clone(), id.clone())) {
            unknowns
                .push(json!({"reason":"Repeated/cyclic resource link; graph expansion stopped."}));
            continue;
        }
        if relationships.len() >= 128 {
            truncated = true;
            break;
        }
        let target = by_id.get(&id);
        relationships.push(json!({"source_id":source_id,"property":property,"target_id":id,"target_tag":target.map(Node::get_name),"certainty":if target.is_some(){"known"}else{"unknown"},"resolved":target.is_some()}));
        if let Some(target) = target
            && let Some(href) = target
                .get_property_no_ns("href")
                .or_else(|| target.get_property_ns("href", "http://www.w3.org/1999/xlink"))
        {
            match link("href", &href) {
                Ok(Some(next)) => pending.push((target.clone(), "href".into(), next)),
                Err(reason) => unknowns.push(json!({"property":"href","reason":reason})),
                _ => {}
            }
        }
    }
    json!({"properties":paint,"ancestor_effects":effects,"relationships":relationships,"relationship_unknowns":unknowns,"relationships_truncated":truncated,"instance_note":if node.get_name()=="use"{Some("Properties describe the use element; source shadow-tree paint and instance overrides require rendered review.")}else{None::<&str>}})
}
pub fn inspect(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let ids: Vec<String> = serde_json::from_value(args["object_ids"].clone())
        .map_err(|_| "object_ids must be strings")?;
    if ids.is_empty()
        || ids.len() > 100
        || ids.iter().any(|id| id.is_empty() || id.len() > 256)
        || ids.iter().collect::<HashSet<_>>().len() != ids.len()
    {
        return Err("supply 1..100 unique object IDs of 1..256 bytes".into());
    }
    let (active, svg) = live
        .session
        .operation(|t| Ok((t.active_document()?, t.document_svg()?)))
        .map_err(|e| e.public_message().to_string())?;
    inspect_snapshot(&svg, &ids, workspace).map(|mut result| {
        result["active_document"] = live_records::sanitize(&active, workspace);
        result
    })
}
pub fn inspect_snapshot(svg: &str, ids: &[String], workspace: &Workspace) -> Result<Value, String> {
    if ids.is_empty() || ids.len() > 100 {
        return Err("supply 1..100 object IDs".into());
    }
    let document = crate::xml::parse(svg.as_bytes(), workspace.max_input)?;
    let root = document.get_root_element().ok_or("invalid live SVG")?;
    // Bounded traversal before the reusable cascade/index are built.
    let mut stack = vec![root.clone()];
    let mut count = 0;
    while let Some(n) = stack.pop() {
        count += 1;
        if count > 10000 {
            return Err("live styles exceed 10000 elements".into());
        }
        stack.extend(n.get_child_elements());
    }
    let nodes = elements(root);
    let mut by_id = HashMap::new();
    for n in &nodes {
        if let Some(id) = n.get_property_no_ns("id")
            && by_id.insert(id, n.clone()).is_some()
        {
            return Err("duplicate object ids make inspection ambiguous".into());
        }
    }
    let cascade = css::Analysis::new(&nodes);
    let objects = ids
        .iter()
        .map(|id| match by_id.get(id) {
            Some(n) => {
                json!({"id":id,"tag":n.get_name(),"computed_style":describe(n,&cascade,&by_id)})
            }
            None => json!({"id":id,"certainty":"unknown","reason":"Object ID not found."}),
        })
        .collect::<Vec<_>>();
    Ok(
        json!({"fingerprint":crate::live_effect::fingerprint(svg,workspace.max_input).map_err(|e|e.public_message().to_string())?,"objects":objects,"count":objects.len(),"notes":["Static bounded CSS; known values are not a pixel appearance guarantee. No external assets are fetched.","Relationships report local links, not coverage or clone shadow-tree computed paint."]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inspect(svg: &str, ids: &[&str]) -> Value {
        inspect_snapshot(
            svg,
            &ids.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            &Workspace {
                roots: vec![],
                max_input: 1024 * 1024,
                max_output: 1024 * 1024,
            },
        )
        .unwrap()
    }
    #[test]
    fn computed_live_css_resources_ancestry_and_clone_uncertainty() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><style>.paint {fill:url(#gradient); stroke:currentColor} #r {fill:url(#pattern)!important}</style><defs><linearGradient id="base"/><linearGradient id="gradient" href="#base"/><pattern id="pattern" href="#pattern-base"/><pattern id="pattern-base"/><mask id="mask"/><clipPath id="clip"/></defs><g id="g" color="#123456" opacity="0.5" mask="url(#mask)"><rect id="r" class="paint" fill="red" style="fill:blue" clip-path="url(#clip)"/><use id="clone" href="#r"/><rect id="default"/><rect id="dynamic" fill="var(--paint)"/></g></svg>"##;
        let report = inspect(svg, &["r", "clone", "default", "dynamic"]);
        let r = &report["objects"][0]["computed_style"];
        assert_eq!(r["properties"]["fill"]["value"], "url(#pattern)");
        assert_eq!(r["properties"]["stroke"]["value"], "#123456");
        assert_eq!(r["properties"]["opacity"]["value"], "1");
        assert_eq!(
            r["ancestor_effects"][0]["properties"]["opacity"]["value"],
            "0.5"
        );
        for id in ["pattern", "pattern-base", "clip", "mask"] {
            assert!(
                r["relationships"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v["target_id"] == id && v["resolved"] == true)
            );
        }
        assert!(report["objects"][1]["computed_style"]["instance_note"].is_string());
        assert_eq!(
            report["objects"][2]["computed_style"]["properties"]["fill"]["value"],
            "black"
        );
        assert_eq!(
            report["objects"][3]["computed_style"]["properties"]["fill"]["certainty"],
            "unknown"
        );
    }
    #[test]
    fn unsupported_css_cycles_missing_targets_and_bounds_are_explicit() {
        let css = inspect(
            "<svg><style>g > rect {fill:red}</style><rect id='r'/></svg>",
            &["r"],
        );
        assert_eq!(
            css["objects"][0]["computed_style"]["properties"]["fill"]["certainty"],
            "unknown"
        );
        let cycle = inspect(
            "<svg><defs><pattern id='a' href='#b'/><pattern id='b' href='#a'/></defs><rect id='r' fill='url(#a)' mask='url(#missing)'/></svg>",
            &["r", "absent"],
        );
        let style = &cycle["objects"][0]["computed_style"];
        assert!(
            style["relationship_unknowns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["reason"].as_str().unwrap().contains("cyclic"))
        );
        assert!(
            style["relationships"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["target_id"] == "missing" && r["resolved"] == false)
        );
        assert_eq!(cycle["objects"][1]["certainty"], "unknown");
        let source = format!("<svg>{}</svg>", "<g/>".repeat(10000));
        assert!(
            inspect_snapshot(
                &source,
                &["x".into()],
                &Workspace {
                    roots: vec![],
                    max_input: 1024 * 1024,
                    max_output: 1024
                }
            )
            .is_err()
        );
    }
}
