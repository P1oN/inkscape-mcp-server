//! Shared read-only consistency signals for bounded sets of registered documents.
use crate::{
    arguments,
    document::{Registry, elements},
    style, xml,
};
use indexmap::IndexMap;
use libxml::tree::Node;
use regex::Regex;
use serde_json::{Value, json};
use std::{collections::HashSet, sync::LazyLock};
static STROKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"stroke-width\s*:\s*([^;]+)").unwrap());
static NAMING: LazyLock<Vec<(&str, Regex)>> = LazyLock::new(|| {
    [
        ("kebab", r"^[a-z0-9]+(?:-[a-z0-9]+)+$"),
        ("snake", r"^[a-z0-9]+(?:_[a-z0-9]+)+$"),
        ("camel", r"^[a-z]+(?:[A-Z][a-z0-9]*)+$"),
        ("pascal", r"^(?:[A-Z][a-z0-9]*){2,}$"),
        ("flat", r"^[a-z0-9]+$"),
    ]
    .into_iter()
    .map(|(name, pattern)| (name, Regex::new(pattern).unwrap()))
    .collect()
});
fn naming(id: &str) -> String {
    NAMING
        .iter()
        .find(|(_, r)| r.is_match(id))
        .map(|(s, _)| *s)
        .unwrap_or("other")
        .into()
}
fn dominant(values: Vec<String>) -> String {
    let mut counts = IndexMap::<String, usize>::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    let highest = counts.values().copied().max().unwrap_or(0);
    counts
        .into_iter()
        .find(|(_, n)| *n == highest)
        .map(|(s, _)| s)
        .unwrap_or_default()
}
fn dimension(raw: Option<String>) -> Option<f64> {
    let raw = raw?;
    let mut text = raw.trim();
    for suffix in ["px", "pt", "pc", "mm", "cm", "in", "em", "ex", "rem", "%"] {
        if let Some(number) = text.strip_suffix(suffix) {
            text = number;
            break;
        }
    }
    text.parse::<f64>().ok().filter(|n| n.is_finite())
}
fn signals(root: Node) -> Result<[String; 3], String> {
    let viewbox = root
        .get_property_no_ns("viewBox")
        .and_then(|s| {
            s.split(|c: char| c.is_whitespace() || c == ',')
                .filter(|s| !s.is_empty())
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
        .filter(|v| v.len() == 4);
    let dims = if let Some(v) = viewbox {
        Some((v[2], v[3]))
    } else {
        dimension(root.get_property_no_ns("width"))
            .zip(dimension(root.get_property_no_ns("height")))
    };
    let box_signature = if let Some((w, h)) = dims {
        format!("{}x{}", style::format_num(w)?, style::format_num(h)?)
    } else {
        String::new()
    };
    let mut widths = Vec::new();
    let mut ids = Vec::new();
    for node in elements(root) {
        if let Some(width) = node
            .get_property_no_ns("stroke-width")
            .filter(|s| !s.trim().is_empty())
        {
            widths.push(width.trim().into());
        }
        if let Some(css) = node.get_property_no_ns("style") {
            for capture in STROKE.captures_iter(&css) {
                let token = capture[1].trim();
                if !token.is_empty() {
                    widths.push(token.into());
                }
            }
        }
        if let Some(id) = node.get_property_no_ns("id").filter(|s| !s.is_empty()) {
            ids.push(naming(&id));
        }
    }
    Ok([box_signature, dominant(widths), dominant(ids)])
}
pub fn ids(args: &Value) -> Result<Vec<&str>, String> {
    let ids = args["doc_ids"].as_array().ok_or("doc_ids must be a list")?;
    if ids.is_empty() {
        return Err("doc_ids must contain at least one document id".into());
    }
    if ids.len() > 32 {
        return Err("document set exceeds the item cap (32)".into());
    }
    let ids = ids
        .iter()
        .map(|v| v.as_str().ok_or("doc_ids must contain strings"))
        .collect::<Result<Vec<_>, _>>()?;
    if ids.iter().copied().collect::<HashSet<_>>().len() != ids.len() {
        return Err("doc_ids must not contain duplicate document ids".into());
    }
    Ok(ids)
}
pub fn verdict(registry: &Registry, ids: &[&str]) -> Result<Value, String> {
    let mut all = Vec::new();
    for id in ids {
        let entry = registry.entries.get(*id).ok_or("document id not found")?;
        let bytes =
            registry
                .workspace
                .read(entry.root, &entry.working(), registry.workspace.max_input)?;
        let document = xml::parse(&bytes, registry.workspace.max_input)
            .map_err(|_| "document could not be parsed safely")?;
        all.push(signals(
            document
                .get_root_element()
                .ok_or("document could not be parsed safely")?,
        )?);
    }
    let mut properties = Vec::new();
    for (index, name) in ["viewBox", "stroke_width", "id_naming"].iter().enumerate() {
        let mut buckets = IndexMap::<String, Vec<&str>>::new();
        let mut unknown = Vec::new();
        for (id, signals) in ids.iter().zip(&all) {
            let value = &signals[index];
            if value.is_empty() {
                unknown.push(*id);
            } else {
                buckets.entry(value.clone()).or_default().push(*id);
            }
        }
        let highest = buckets.values().map(Vec::len).max().unwrap_or(0);
        let majority = buckets
            .iter()
            .find(|(_, v)| v.len() == highest)
            .map(|(key, _)| key.clone());
        let values = buckets
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect::<serde_json::Map<_, _>>();
        properties.push(json!({"property":name,"agree":buckets.len()<=1,"majority":majority,"values":values,"unknown_doc_ids":unknown}));
    }
    Ok(json!({"consistent":properties.iter().all(|v|v["agree"]==true),"properties":properties}))
}
pub fn export_set(registry: &Registry, args: &Value) -> Result<Value, String> {
    let ids = ids(args)?;
    let consistency = verdict(registry, &ids)?;
    let dry = arguments::boolean(args, "dry_run", true)?;
    let mut entries = Vec::new();
    let mut total_items = 0u64;
    let mut total_bytes = 0u64;
    for id in ids {
        let mut single = args.clone();
        single["doc_id"] = json!(id);
        let result = crate::export_batch::call(registry, &single)?;
        total_items = total_items
            .checked_add(result["item_count"].as_u64().unwrap())
            .ok_or("export set exceeds the supported range")?;
        total_bytes = total_bytes
            .checked_add(
                result["actual_total_bytes"]
                    .as_u64()
                    .unwrap_or(result["projected_total_bytes"].as_u64().unwrap()),
            )
            .ok_or("export set exceeds the supported range")?;
        entries.push(json!({"doc_id":id,"result":result}));
    }
    Ok(
        json!({"per_doc":entries,"total_items":total_items,"total_bytes":total_bytes,"dry_run":dry,"consistency":consistency}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signals_count_attributes_styles_and_ids_with_first_seen_ties() {
        let d=xml::parse(br#"<svg width="100px" height="80mm" id="root-id"><rect id="RedBox" stroke-width="2" style="stroke-width:3;stroke-width:3"/><g id="other-id" stroke-width="2"/></svg>"#,4096).unwrap();
        assert_eq!(
            signals(d.get_root_element().unwrap()).unwrap(),
            ["100x80", "2", "kebab"]
        );
        assert_eq!(dominant(vec!["first".into(), "second".into()]), "first");
        for (id, expected) in [
            ("red-box", "kebab"),
            ("red_box", "snake"),
            ("redBox", "camel"),
            ("RedBox", "pascal"),
            ("flat1", "flat"),
            ("X.Y", "other"),
        ] {
            assert_eq!(naming(id), expected);
        }
        assert!(dimension(Some("1rem".into())).is_none()); // Match suffix-order behavior in reference.
        let unknown = xml::parse(b"<svg width='auto' height='auto'/>", 4096).unwrap();
        assert_eq!(
            signals(unknown.get_root_element().unwrap()).unwrap(),
            ["", "", ""]
        );
        assert!(ids(&json!({"doc_ids":[]})).is_err());
        assert!(ids(&json!({"doc_ids":["a","a"]})).is_err());
        assert!(ids(&json!({"doc_ids":vec!["a";33]})).is_err());
    }
}
