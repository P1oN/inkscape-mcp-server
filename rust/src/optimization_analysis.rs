//! Shared nonmutating optimization signals for quality and the later optimizer.
use crate::document::elements;
use libxml::tree::{Node, NodeType};
use regex::Regex;
use std::{collections::HashSet, sync::LazyLock};
static REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"url\(\s*['"]?#([^'")\s]+)['"]?\s*\)"#).unwrap());
static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-?\d+\.\d+(?:[eE][-+]?\d+)?").unwrap());
pub(crate) fn references(root: Node) -> HashSet<String> {
    let nodes = elements(root);
    let mut refs = HashSet::new();
    for node in &nodes {
        if let Some(href) = node
            .get_property_ns("href", "http://www.w3.org/1999/xlink")
            .filter(|s| !s.is_empty())
            .or_else(|| node.get_property_no_ns("href"))
            && let Some(id) = href
                .strip_prefix('#')
                .map(str::trim)
                .filter(|s| !s.is_empty())
        {
            refs.insert(id.to_owned());
        }
        for attr in [
            "fill",
            "stroke",
            "mask",
            "clip-path",
            "filter",
            "style",
            "marker-start",
            "marker-mid",
            "marker-end",
        ] {
            if let Some(v) = node.get_property_no_ns(attr) {
                for c in REF.captures_iter(&v) {
                    refs.insert(c[1].trim().to_owned());
                }
            }
        }
    }
    refs
}
pub fn counts(root: Node) -> [usize; 5] {
    counts_with(root, 2, &HashSet::new())
}
pub(crate) fn counts_with(root: Node, precision: usize, keep: &HashSet<String>) -> [usize; 5] {
    let nodes = elements(root.clone());
    let mut refs = references(root.clone());
    refs.extend(keep.iter().cloned());
    let mut count = [0; 5];
    let mut stack = vec![root.clone()];
    while let Some(n) = stack.pop() {
        stack.extend(n.get_child_nodes());
        if matches!(n.get_type(), Some(NodeType::CommentNode | NodeType::PiNode)) {
            count[0] += 1;
        }
    }
    for node in &nodes {
        let name = node.get_name();
        if *node != root && ["namedview", "metadata"].contains(&name.as_str()) {
            count[0] += 1;
        } else {
            count[0] += node
                .get_properties_ns()
                .keys()
                .filter(|(_, ns)| {
                    ns.as_ref().is_some_and(|ns| {
                        [
                            crate::document::INKSCAPE_NS,
                            "http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd",
                        ]
                        .contains(&ns.get_href().as_str())
                    })
                })
                .count();
        }
        if name == "defs" {
            count[1] += node
                .get_child_elements()
                .iter()
                .filter(|n| {
                    !["style", "script"].contains(&n.get_name().as_str())
                        && n.get_property_no_ns("id")
                            .is_none_or(|id| !refs.contains(&id))
                })
                .count();
        }
        if node
            .get_property_no_ns("id")
            .is_some_and(|id| !refs.contains(&id))
        {
            count[2] += 1;
        }
        if *node != root
            && ["g", "defs"].contains(&name.as_str())
            && node.get_property_no_ns("id").is_none()
            && node.get_child_elements().is_empty()
        {
            count[3] += 1;
        }
        for ((attr, _), value) in node.get_properties_ns() {
            if !coordinate(&attr) {
                continue;
            }
            let reduced = rounded(&value, precision);
            if reduced != value {
                count[4] += 1;
            }
        }
    }
    count
}

pub(crate) fn coordinate(attr: &str) -> bool {
    [
        "d",
        "points",
        "transform",
        "gradientTransform",
        "patternTransform",
        "x",
        "y",
        "x1",
        "y1",
        "x2",
        "y2",
        "cx",
        "cy",
        "r",
        "rx",
        "ry",
        "dx",
        "dy",
        "width",
        "height",
        "offset",
        "fx",
        "fy",
    ]
    .contains(&attr)
}
pub(crate) fn rounded(value: &str, precision: usize) -> String {
    NUMBER
        .replace_all(value, |c: &regex::Captures| {
            let Ok(n) = c[0].parse::<f64>() else {
                return c[0].to_owned();
            };
            let text = format!("{n:.precision$}");
            let text = if text.contains('.') {
                text.trim_end_matches('0').trim_end_matches('.')
            } else {
                &text
            };
            if text.is_empty() || text == "-0" {
                "0".into()
            } else {
                text.to_owned()
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_metadata_rounding_and_empty_containers_are_readonly() {
        let doc = crate::xml::parse(
            br##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:i="http://www.inkscape.org/namespaces/inkscape"><!--kept--><metadata i:label="skip-own"><g i:label="nested"/></metadata><defs><linearGradient id="used"/><linearGradient id="dead"/><style/></defs><rect id="r" fill="url(#used)" x="2.675" y="-0.001"/><use href="#r"/><g/><!--kept--></svg>"##,
            4096,
        ).unwrap();
        let before = crate::xml::serialize(&doc);
        assert_eq!(counts(doc.get_root_element().unwrap()), [4, 1, 1, 2, 2]);
        assert_eq!(crate::xml::serialize(&doc), before);
    }
}
