//! Fixed live insertion allowlist; headless adoption has a different policy.
use libxml::tree::NodeType;
use std::collections::HashMap;
use std::sync::LazyLock;
const SVG: &str = "http://www.w3.org/2000/svg";
const CAP: usize = 1024 * 1024;
static URL: LazyLock<regex::Regex> = LazyLock::new(|| {
    // Python \w means Unicode alphanumeric plus underscore; Rust's \w also
    // includes combining marks, so use explicit letter/number classes.
    regex::Regex::new(r##"(?i:url)\([\s\x1c-\x1f]*['"]?(#[\p{L}\p{N}_.:-]+)['"]?[\s\x1c-\x1f]*\)"##)
        .unwrap()
});
fn fail(message: &'static str) -> &'static str {
    message
}
type Validated = (libxml::tree::Document, HashMap<String, String>, Vec<String>);
fn validated(fragment: &str, prefix: &str) -> Result<Validated, &'static str> {
    let token = prefix.strip_prefix("mcp_").unwrap_or("");
    if token.len() != 32
        || !token
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(fail("invalid insertion id"));
    }
    if fragment
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
        .is_empty()
        || fragment.len() > CAP
    {
        return Err(fail("insertion fragment is empty or too large"));
    }
    let source = format!("<svg xmlns=\"{SVG}\">{fragment}</svg>");
    let document = crate::xml::parse(source.as_bytes(), CAP + 100)
        .map_err(|_| fail("malformed insertion XML"))?;
    let root = document
        .get_root_element()
        .ok_or(fail("malformed insertion XML"))?;
    let mut nodes = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(node) = stack.pop() {
        if matches!(
            node.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode)
        ) {
            continue;
        }
        nodes.push(node.clone());
        if nodes.len() > 10_000 {
            return Err(fail("too many insertion elements"));
        }
        stack.extend(node.get_child_nodes().into_iter().rev());
    }
    let mut ids = HashMap::new();
    let mut ordered = vec![prefix.to_string()];
    for node in &nodes {
        if !node.is_element_node() {
            return Err(fail("insertion contains non-element content"));
        }
        if node.get_namespace().map(|ns| ns.get_href()).as_deref() != Some(SVG)
            || !matches!(
                node.get_name().as_str(),
                "svg"
                    | "g"
                    | "defs"
                    | "rect"
                    | "circle"
                    | "ellipse"
                    | "path"
                    | "line"
                    | "polygon"
                    | "polyline"
                    | "text"
                    | "tspan"
                    | "use"
                    | "linearGradient"
                    | "radialGradient"
                    | "stop"
                    | "clipPath"
                    | "mask"
                    | "pattern"
                    | "title"
                    | "desc"
            )
        {
            return Err(fail("unsupported SVG insertion element"));
        }
        if let Some(id) = node.get_property_no_ns("id").filter(|id| !id.is_empty()) {
            if ids.contains_key(&id) {
                return Err(fail("duplicate insertion id"));
            }
            let new = format!("{prefix}_{}", ids.len());
            ordered.push(new.clone());
            ids.insert(id, new);
        }
    }
    for node in &nodes {
        for ((name, _), value) in node.get_properties_ns() {
            if name.to_lowercase().starts_with("on") || name == "base" {
                return Err(fail("event attributes are unsupported"));
            }
            if name == "id" {
                // Python raises an uncaught KeyError for an empty ID. Refuse it
                // deterministically before any request, preserving the document.
                if !ids.contains_key(&value) {
                    return Err(fail("''"));
                }
            } else if name == "href" {
                if !value
                    .strip_prefix('#')
                    .is_some_and(|id| ids.contains_key(id))
                {
                    return Err(fail("href must reference an id in the fragment"));
                }
            } else if value.to_lowercase().contains("url") || name == "style" {
                if value.contains(['\\', '@']) {
                    return Err(fail("unsupported CSS insertion value"));
                }
                for found in URL.captures_iter(&value) {
                    if !ids.contains_key(&found[1][1..]) {
                        return Err(fail("paint reference must be internal to the fragment"));
                    }
                }
                if URL.replace_all(&value, "").to_lowercase().contains("url") {
                    return Err(fail("external or malformed CSS URL"));
                }
            }
        }
    }
    if root.get_child_elements().is_empty() {
        return Err(fail("insertion has no objects"));
    }
    Ok((document, ids, ordered))
}

pub fn plan(fragment: &str, prefix: &str) -> Result<Vec<String>, &'static str> {
    validated(fragment, prefix).map(|(_, _, ids)| ids)
}
/// Build one ordinary root group on an owned candidate, never in a user's document.
pub fn prepare(fragment: &str, prefix: &str) -> Result<(Vec<u8>, Vec<String>), &'static str> {
    let (document, ids, ordered) = validated(fragment, prefix)?;
    let mut root = document
        .get_root_element()
        .ok_or("malformed insertion XML")?;
    for mut node in super::elements(root.clone()) {
        for ((name, ns), value) in node.get_properties_ns() {
            let value = if name == "id" {
                ids[&value].clone()
            } else if name == "href" {
                format!("#{}", ids[&value[1..]])
            } else if value.to_lowercase().contains("url") || name == "style" {
                URL.replace_all(&value, |c: &regex::Captures<'_>| {
                    format!("url(#{})", ids[&c[1][1..]])
                })
                .into_owned()
            } else {
                continue;
            };
            match ns {
                Some(ns) => node.set_property_ns(&name, &value, &ns),
                None => node.set_property(&name, &value),
            }
            .map_err(|_| "fragment rewrite failed")?;
        }
    }
    root.set_name("g").map_err(|_| "fragment rewrite failed")?;
    root.set_property("id", prefix)
        .map_err(|_| "fragment rewrite failed")?;
    // The Python wrapper discards leading wrapper text (child tails survive).
    let mut child = root.get_first_child();
    while let Some(mut node) = child {
        if node.is_element_node() {
            break;
        }
        child = node.get_next_sibling();
        node.unlink();
    }
    let bytes = document.node_to_string(&root).into_bytes();
    if bytes.len() > 8 * CAP {
        return Err("prepared fragment exceeds size cap");
    }
    Ok((bytes, ordered))
}

/// Rewrite only supported local references; external definition references survive.
pub fn remap_reference(key: &str, value: &str, ids: &HashMap<String, String>) -> String {
    let result = URL
        .replace_all(value, |c: &regex::Captures<'_>| {
            ids.get(&c[1][1..])
                .map(|id| format!("url(#{id})"))
                .unwrap_or_else(|| c[0].to_string())
        })
        .into_owned();
    if matches!(key, "href" | "connector-start" | "connector-end")
        && let Some(id) = result.strip_prefix('#').and_then(|id| ids.get(id))
    {
        return format!("#{id}");
    }
    result
}
