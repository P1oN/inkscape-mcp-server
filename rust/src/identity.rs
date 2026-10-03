//! Typed ID/label edits preserve supported reference syntax; ambiguous forms refuse staging.
use crate::{
    arguments,
    document::{Registry, elements},
    inspect, style, transaction,
};
use libxml::tree::{Document, Namespace};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;
const INK: &str = "http://www.inkscape.org/namespaces/inkscape";
const XLINK: &str = "http://www.w3.org/1999/xlink";
static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_.:-]*$").unwrap());
static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"url\(\s*#([^)\s]+)\s*\)").unwrap());
const REFS: &[&str] = &[
    "fill",
    "stroke",
    "mask",
    "clip-path",
    "filter",
    "style",
    "marker-start",
    "marker-mid",
    "marker-end",
];
pub(crate) fn set_namespaced(
    node: &mut libxml::tree::Node,
    document: &Document,
    uri: &str,
    key: &str,
    value: &str,
) -> Result<(), String> {
    set_namespaced_from(node, document, uri, key, value, 0)
}
fn set_namespaced_from(
    node: &mut libxml::tree::Node,
    document: &Document,
    uri: &str,
    key: &str,
    value: &str,
    first_prefix: usize,
) -> Result<(), String> {
    let namespace = if let Some(ns) = node
        .get_namespaces(document)
        .into_iter()
        .find(|ns| ns.get_href() == uri && !ns.get_prefix().is_empty())
    {
        ns
    } else {
        let declared = node.get_namespaces(document);
        let mut index = first_prefix;
        let prefix = loop {
            let prefix = format!("ns{index}");
            if !declared.iter().any(|ns| ns.get_prefix() == prefix) {
                break prefix;
            }
            index += 1;
        };
        Namespace::new(&prefix, uri, node).map_err(|_| "identity namespace unavailable")?
    };
    node.set_property_ns(key, value, &namespace)
        .map_err(|_| "identity edit failed".into())
}
pub(crate) fn set_inkscape(
    node: &mut libxml::tree::Node,
    document: &Document,
    key: &str,
    value: &str,
) -> Result<(), String> {
    set_namespaced(node, document, INK, key, value)
}
/// Newly qualified SVG objects consume ns0 in the reference's detached builder.
/// Reuse any in-scope Inkscape prefix, otherwise begin at ns1. Avoid shadowing
/// existing prefixes so serialized element namespaces remain unchanged.
pub(crate) fn set_inkscape_new_svg(
    node: &mut libxml::tree::Node,
    document: &Document,
    key: &str,
    value: &str,
) -> Result<(), String> {
    set_namespaced_from(node, document, INK, key, value, 1)
}
pub(crate) fn rewrite_references(
    document: &Document,
    nodes: &mut [libxml::tree::Node],
    mapping: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    for node in nodes {
        for (key, ns) in [
            ("href", Some(XLINK)),
            ("href", None),
            ("connection-start", Some(INK)),
            ("connection-end", Some(INK)),
        ] {
            let value = if let Some(ns) = ns {
                node.get_property_ns(key, ns)
            } else {
                node.get_property_no_ns(key)
            };
            if let Some(new) = value
                .as_deref()
                .and_then(|v| v.strip_prefix('#'))
                .and_then(|id| mapping.get(id))
            {
                if let Some(ns) = ns {
                    let namespace = node
                        .get_namespaces(document)
                        .into_iter()
                        .find(|n| n.get_href() == ns && !n.get_prefix().is_empty())
                        .ok_or("identity namespace unavailable")?;
                    node.set_property_ns(key, &format!("#{new}"), &namespace)
                        .map_err(|_| "identity edit failed")?;
                } else {
                    node.set_property(key, &format!("#{new}"))
                        .map_err(|_| "identity edit failed")?;
                }
            }
        }
        for key in REFS {
            if let Some(value) = node.get_property_no_ns(key)
                && (value.contains("url(") || value.starts_with('#'))
            {
                let mut updated = URL
                    .replace_all(&value, |c: &regex::Captures| {
                        let old = c.get(1).unwrap().as_str();
                        format!("url(#{})", mapping.get(old).map_or(old, String::as_str))
                    })
                    .into_owned();
                if let Some(new) = value.strip_prefix('#').and_then(|id| mapping.get(id)) {
                    updated = format!("#{new}");
                }
                node.set_property(key, &updated)
                    .map_err(|_| "identity edit failed")?;
            }
        }
    }
    Ok(())
}
pub struct Mutation {
    id: String,
    new_id: Option<String>,
    label: Option<String>,
    pub params: Value,
}
impl Mutation {
    pub fn build(args: &Value) -> Result<Self, String> {
        let id = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?
            .to_owned();
        let new_id = arguments::string(args, "new_id")?.map(str::to_owned);
        let label = arguments::string(args, "label")?.map(str::to_owned);
        if new_id.is_none() && label.is_none() {
            return Err("rename_object requires at least one of new_id / label".into());
        }
        if let Some(value) = &new_id
            && !ID.is_match(value)
        {
            return Err(format!("invalid object id: {}", style::python_repr(value)));
        }
        if let Some(value) = &label {
            let length = value.chars().count();
            if length > 256 {
                return Err(format!("label too long: {length} > 256 characters"));
            }
            if value
                .chars()
                .any(|c| matches!(c as u32,0..=8|11..=12|14..=31|127..=159))
            {
                return Err("label contains forbidden control characters".into());
            }
        }
        let params = json!({"object_id":id,"new_id":new_id,"label":label});
        Ok(Self {
            id,
            new_id,
            label,
            params,
        })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let mut nodes = elements(
            document
                .get_root_element()
                .ok_or("document could not be parsed safely")?,
        );
        let mut target = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_ref() == Some(&self.id))
            .cloned()
            .ok_or("object id not found in document")?;
        let mut changes = Vec::new();
        if let Some(new) = &self.new_id {
            if new == &self.id {
                return Err("new_id is the same as the current id".into());
            }
            if nodes
                .iter()
                .any(|n| n.get_property_no_ns("id").as_ref() == Some(new))
            {
                return Err(format!("id already in use: {}", style::python_repr(new)));
            }
            for node in &nodes {
                if node.get_name() == "style"
                    && inspect::text(node).contains(&format!("#{}", self.id))
                {
                    return Err("rename cannot safely preserve stylesheet references".into());
                }
                for key in ["begin", "end", "aria-labelledby", "aria-describedby"] {
                    if node
                        .get_property_no_ns(key)
                        .is_some_and(|v| v.contains(&self.id))
                    {
                        return Err(
                            "rename cannot safely preserve timing or accessibility references"
                                .into(),
                        );
                    }
                }
            }
            for node in &nodes {
                for key in ["fill", "stroke"] {
                    if node.get_property_no_ns(key).is_some_and(|value| {
                        value == format!("#{}", self.id) && style::color(&value).is_ok()
                    }) {
                        return Err(
                            "rename cannot safely distinguish an ID reference from a color".into(),
                        );
                    }
                }
            }
            target
                .set_property("id", new)
                .map_err(|_| "identity edit failed")?;
            rewrite_references(
                document,
                &mut nodes,
                &std::collections::HashMap::from([(self.id.clone(), new.clone())]),
            )?;
            changes.push(format!(
                "id {} -> {}",
                style::python_repr(&self.id),
                style::python_repr(new)
            ));
        }
        if let Some(label) = &self.label {
            set_inkscape(&mut target, document, "label", label)?;
            changes.push(format!("label -> {}", style::python_repr(label)));
        }
        Ok(format!("renamed object: {}", changes.join("; ")))
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(args)?;
    transaction::apply_dom(
        registry,
        id,
        "rename_object",
        mutation.params.clone(),
        "medium",
        None,
        |document| mutation.mutate(document),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn label_attribute_keeps_its_namespace_when_default_namespace_matches() {
        let mut document = crate::xml::parse(
            br#"<svg xmlns="http://www.inkscape.org/namespaces/inkscape" id="r"/>"#,
            4096,
        )
        .unwrap();
        Mutation::build(&json!({"object_id":"r","label":"Object"}))
            .unwrap()
            .mutate(&mut document)
            .unwrap();
        let root = document.get_root_element().unwrap();
        assert_eq!(
            root.get_property_ns("label", INK).as_deref(),
            Some("Object")
        );
        assert_eq!(root.get_property_no_ns("label"), None);
        assert!(document.to_string().contains("ns0:label=\"Object\""));
    }
    #[test]
    fn hex_id_color_ambiguity_refuses_before_mutation() {
        let mut document =
            crate::xml::parse(br##"<svg><rect id="abc" fill="#abc"/></svg>"##, 4096).unwrap();
        let before = document.to_string();
        assert!(
            Mutation::build(&json!({"object_id":"abc","new_id":"object"}))
                .unwrap()
                .mutate(&mut document)
                .is_err()
        );
        assert_eq!(before, document.to_string());
    }
    #[test]
    fn unsupported_stylesheet_refuses_before_id_change() {
        let mut document = crate::xml::parse(
            br##"<svg><style>#r{fill:red}</style><rect id="r"/></svg>"##,
            4096,
        )
        .unwrap();
        let before = document.to_string();
        assert!(
            Mutation::build(&json!({"object_id":"r","new_id":"new"}))
                .unwrap()
                .mutate(&mut document)
                .is_err()
        );
        assert_eq!(before, document.to_string());
    }
}
