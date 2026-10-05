//! Fingerprint v1: SHA-256 of the documented UTF-8 record stream.
use super::elements;
use libxml::tree::{Node, NodeType};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn trim(text: &str) -> &str {
    text.trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
}
fn adjacent(mut node: Option<Node>) -> String {
    let mut text = String::new();
    while let Some(current) = node {
        if !matches!(
            current.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode)
        ) {
            break;
        }
        text.push_str(&current.get_content());
        node = current.get_next_sibling();
    }
    trim(&text).to_string()
}
fn dumps(value: &Value) -> String {
    let compact = serde_json::to_string(value).unwrap();
    let mut quoted = false;
    let mut escaped = false;
    let mut result = String::new();
    for c in compact.chars() {
        result.push(c);
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
        } else if c == ',' || c == ':' {
            result.push(' ');
        }
    }
    result
}
pub fn fingerprint(svg: &str, cap: usize) -> Result<String, &'static str> {
    let document =
        crate::xml::parse(svg.as_bytes(), cap).map_err(|_| "active document export is invalid")?;
    let root = document
        .get_root_element()
        .ok_or("active document export is invalid")?;
    let budget = cap.saturating_mul(8).min(64 * 1024 * 1024);
    let over = "document fingerprint exceeds size cap";
    let mut digest = Sha256::new();
    digest.update(b"[");
    let mut count = 2usize;
    let mut first = true;
    for node in elements(root.clone()) {
        let mut depth = 0;
        let mut skipped = matches!(node.get_name().as_str(), "metadata" | "namedview");
        let mut parent = node.get_parent();
        while let Some(p) = parent {
            if p.is_element_node() {
                depth += 1;
                skipped |= matches!(p.get_name().as_str(), "metadata" | "namedview");
            }
            parent = p.get_parent();
        }
        if skipped {
            continue;
        }
        let tag = node
            .get_namespace()
            .map(|ns| format!("{{{}}}{}", ns.get_href(), node.get_name()))
            .unwrap_or_else(|| node.get_name());
        let mut size = tag.len();
        if size > budget {
            return Err(over);
        }
        let mut attrs = Vec::new();
        for ((name, ns), value) in node.get_properties_ns() {
            if node == root && !(ns.is_none() && !matches!(name.as_str(), "id" | "version")) {
                continue;
            }
            let uri = ns.map(|ns| ns.get_href());
            size = size
                .saturating_add(name.len())
                .saturating_add(value.len())
                .saturating_add(uri.as_ref().map_or(0, String::len));
            if size > budget {
                return Err(over);
            }
            attrs.push((
                uri.map(|uri| format!("{{{uri}}}{name}")).unwrap_or(name),
                value,
            ));
        }
        attrs.sort();
        let record = dumps(&json!([
            depth,
            tag,
            attrs,
            adjacent(node.get_first_child()),
            adjacent(node.get_next_sibling())
        ]));
        count = count
            .saturating_add(record.len())
            .saturating_add(if first { 0 } else { 2 });
        if count > budget {
            return Err(over);
        }
        if !first {
            digest.update(b", ");
        }
        first = false;
        digest.update(record.as_bytes());
    }
    digest.update(b"]");
    Ok(format!("{:x}", digest.finalize()))
}
