//! Approval-gated, allowlisted SVG adoption into the staged working tree.
use crate::{
    arguments,
    document::{INKSCAPE_NS, Registry, elements},
    style, transaction, validate, xml,
};
use libxml::{
    bindings,
    tree::{Document, Node, NodeType},
};
use serde_json::{Value, json};
use std::collections::HashSet;
const SVG: &str = "http://www.w3.org/2000/svg";
const XLINK: &str = "http://www.w3.org/1999/xlink";
const SODI: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd";
const ALLOWED_ELEMENTS: &[&str] = &[
    "circle",
    "clipPath",
    "defs",
    "desc",
    "ellipse",
    "g",
    "line",
    "linearGradient",
    "marker",
    "mask",
    "metadata",
    "path",
    "pattern",
    "polygon",
    "polyline",
    "radialGradient",
    "rect",
    "stop",
    "svg",
    "symbol",
    "text",
    "textPath",
    "title",
    "tspan",
    "use",
];
const ALLOWED_ATTRS: &[&str] = &[
    "alignment-baseline",
    "class",
    "clip-path",
    "clip-rule",
    "clipPathUnits",
    "color",
    "cx",
    "cy",
    "d",
    "display",
    "dominant-baseline",
    "dx",
    "dy",
    "fill",
    "fill-opacity",
    "fill-rule",
    "filter",
    "font-family",
    "font-size",
    "font-style",
    "font-weight",
    "fx",
    "fy",
    "gradientTransform",
    "gradientUnits",
    "height",
    "id",
    "letter-spacing",
    "marker-end",
    "marker-mid",
    "marker-start",
    "markerHeight",
    "markerUnits",
    "markerWidth",
    "mask",
    "maskContentUnits",
    "maskUnits",
    "offset",
    "opacity",
    "orient",
    "patternContentUnits",
    "patternTransform",
    "patternUnits",
    "points",
    "preserveAspectRatio",
    "r",
    "refX",
    "refY",
    "rotate",
    "rx",
    "ry",
    "spreadMethod",
    "stop-color",
    "stop-opacity",
    "stroke",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-opacity",
    "stroke-width",
    "style",
    "text-anchor",
    "text-decoration",
    "transform",
    "version",
    "viewBox",
    "visibility",
    "white-space",
    "width",
    "word-spacing",
    "x",
    "x1",
    "x2",
    "xml:space",
    "xmlns",
    "y",
    "y1",
    "y2",
];
const _URL_REF_ATTRS: &[&str] = &[
    "clip-path",
    "fill",
    "filter",
    "marker-end",
    "marker-mid",
    "marker-start",
    "mask",
    "stroke",
    "style",
];
fn allowed(ns: Option<&str>) -> bool {
    ns.is_none_or(|ns| [SVG, XLINK, INKSCAPE_NS, SODI].contains(&ns))
}
pub(crate) fn scrub(document: &Document) -> Result<(), String> {
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        stack.extend(n.get_child_nodes().into_iter().rev().filter(|n| {
            !matches!(
                n.get_type(),
                Some(NodeType::TextNode | NodeType::CDataSectionNode)
            )
        }));
        if !n.is_element_node() {
            return Err("svg contains a disallowed node (comment/processing-instruction)".into());
        }
        let ns = n.get_namespace().map(|ns| ns.get_href());
        if !allowed(ns.as_deref()) {
            return Err(format!(
                "svg contains disallowed element namespace: {}",
                style::python_repr(ns.as_deref().unwrap())
            ));
        }
        let name = n.get_name();
        if !ALLOWED_ELEMENTS.contains(&name.as_str()) {
            return Err(format!(
                "svg contains disallowed element: {}",
                style::python_repr(&name)
            ));
        }
        for ((name, ns), value) in n.get_properties_ns() {
            let ns = ns.map(|ns| ns.get_href());
            let attr = style::python_repr(&name);
            if name.to_lowercase().starts_with("on") {
                return Err(format!(
                    "svg contains a disallowed event-handler attribute: {attr}"
                ));
            }
            if !allowed(ns.as_deref()) {
                return Err(format!(
                    "svg contains disallowed attribute namespace: {}",
                    style::python_repr(ns.as_deref().unwrap())
                ));
            }
            if name == "href" {
                if !value.trim().is_empty() && !value.trim().starts_with('#') {
                    return Err(format!(
                        "svg contains disallowed external/active reference in {attr}: only same-document '#id' references are allowed"
                    ));
                }
                continue;
            }
            if ns
                .as_deref()
                .is_some_and(|s| [INKSCAPE_NS, SODI].contains(&s))
            {
                continue;
            }
            if !ALLOWED_ATTRS.contains(&name.as_str()) {
                return Err(format!("svg contains a disallowed attribute: {attr}"));
            }
            if _URL_REF_ATTRS.contains(&name.as_str()) {
                let low = value.to_lowercase();
                if low.contains("javascript:") {
                    return Err(format!(
                        "svg contains disallowed 'javascript:' reference in {attr}"
                    ));
                }
                let mut rest = low.as_str();
                while let Some(i) = rest.find("url(") {
                    rest = &rest[i + 4..];
                    if !rest
                        .trim_start()
                        .trim_start_matches(['\'', '"'])
                        .trim_start()
                        .starts_with('#')
                    {
                        return Err(format!(
                            "svg contains disallowed external reference in a url(...) in {attr}: only same-document 'url(#id)' references are allowed"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
struct Copy(bindings::xmlNodePtr);
impl Drop for Copy {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { bindings::xmlFreeNode(self.0) };
        }
    }
}
fn graft(source: &Node, parent: &Node, document: &Document) -> Result<(), String> {
    // Both documents stay alive; this guard owns the detached recursive copy until attachment.
    let mut copy =
        Copy(unsafe { bindings::xmlDocCopyNode(source.node_ptr(), document.doc_ptr(), 1) });
    if copy.0.is_null() {
        return Err("fragment copy failed".into());
    }
    let attached = unsafe { bindings::xmlAddChild(parent.node_ptr(), copy.0) };
    if attached.is_null() {
        return Err("fragment insertion failed".into());
    }
    copy.0 = std::ptr::null_mut();
    // xmlAddChild may merge and free a text copy; use the returned live pointer.
    if source.is_element_node()
        && unsafe { bindings::xmlDOMWrapReconcileNamespaces(std::ptr::null_mut(), attached, 1) } < 0
    {
        return Err("fragment namespace reconciliation failed".into());
    }
    Ok(())
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let raw = args["svg"].as_str().ok_or("svg must be a string")?;
    let approval = arguments::string(args, "approval_token")?;
    if approval.is_none_or(str::is_empty) {
        return Err("high-risk compose operation requires an explicit approval_token".into());
    }
    if raw.len() > registry.workspace.max_input {
        return Err("input svg exceeds the configured size limit".into());
    }
    let incoming = xml::parse(raw.as_bytes(), registry.workspace.max_input)?;
    scrub(&incoming)?;
    let incoming_root = incoming.get_root_element().unwrap();
    let replace = tool == "set_document_svg";
    if replace && incoming_root.get_name() != "svg" {
        return Err("set_document_svg requires the SVG root element to be <svg>".into());
    }
    let parent_id = arguments::string(args, "parent_id")?;
    let unwrap = arguments::boolean(args, "unwrap", true)?;
    let params = if replace {
        json!({"svg_length":raw.chars().count()})
    } else {
        json!({"svg_length":raw.chars().count(),"parent_id":parent_id,"unwrap":unwrap})
    };
    let mut result =
        transaction::apply_dom(registry, id, tool, params, "high", approval, |document| {
            if replace {
                *document = xml::parse(raw.as_bytes(), registry.workspace.max_input)?;
                return Ok("replaced document with composed svg".into());
            }
            let root = document
                .get_root_element()
                .ok_or("document could not be parsed safely")?;
            let parent = if let Some(id) = parent_id {
                elements(root.clone())
                    .into_iter()
                    .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
                    .ok_or("object id not found in document")?
            } else {
                root.clone()
            };
            let children = if unwrap && incoming_root.get_name() == "svg" {
                incoming_root.get_child_elements()
            } else {
                vec![incoming_root.clone()]
            };
            if children.is_empty() {
                return Err("composed fragment <svg> wrapper is empty".into());
            }
            let existing: HashSet<_> = elements(root)
                .iter()
                .filter_map(|n| n.get_property_no_ns("id"))
                .collect();
            let mut added = HashSet::new();
            for child in &children {
                for n in elements(child.clone()) {
                    if let Some(id) = n.get_property_no_ns("id")
                        && (existing.contains(&id) || !added.insert(id))
                    {
                        return Err(
                            "fragment ID conflicts with the document or repeats inside fragment"
                                .into(),
                        );
                    }
                }
            }
            for child in &children {
                graft(child, &parent, document)?;
                // lxml append moves the element with its tail; wrapper text itself is not inserted.
                let mut sibling = child.get_next_sibling();
                while let Some(n) = sibling.filter(|n| {
                    matches!(
                        n.get_type(),
                        Some(NodeType::TextNode | NodeType::CDataSectionNode)
                    )
                }) {
                    sibling = n.get_next_sibling();
                    graft(&n, &parent, document)?;
                }
            }
            Ok(if unwrap && incoming_root.get_name() == "svg" {
                format!(
                    "inserted {} element(s) from composed fragment",
                    children.len()
                )
            } else {
                format!(
                    "inserted <{}> from composed fragment",
                    incoming_root.get_name()
                )
            })
        })?;
    result["validation"] = validate::document(registry, id)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approval_and_input_caps_precede_parse_registry_and_history() {
        let registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![],
                max_input: 32,
                max_output: 4096,
            },
            entries: indexmap::IndexMap::new(),
        };
        assert_eq!(
            apply(
                &registry,
                "set_document_svg",
                &json!({"doc_id":"missing","svg":"<broken"})
            )
            .unwrap_err(),
            "high-risk compose operation requires an explicit approval_token"
        );
        assert_eq!(
            apply(
                &registry,
                "set_document_svg",
                &json!({"doc_id":"missing","svg":"x".repeat(33),"approval_token":"approved"})
            )
            .unwrap_err(),
            "input svg exceeds the configured size limit"
        );
        assert!(registry.entries.is_empty());
    }
    #[test]
    fn allowlist_blocks_active_external_entity_and_unknown_inputs() {
        for source in [
            "<svg><script/></svg>",
            "<g onload='test'/>",
            "<use href='data:text/plain,a'/>",
            "<g fill='url(https://example.com/a)'/>",
            "<g><!--comment--></g>",
            "<g strange='1'/>",
            "<!DOCTYPE g [<!ENTITY e SYSTEM 'file:///etc/passwd'>]><g>&e;</g>",
        ] {
            assert!(scrub(&xml::parse(source.as_bytes(), 4096).unwrap()).is_err());
        }
        assert!(
            scrub(
                &xml::parse(
                    b"<g><text>A<tspan>middle</tspan>tail</text><use href='#r'/></g>",
                    4096
                )
                .unwrap()
            )
            .is_ok()
        );
    }
    #[test]
    fn independent_copy_reconciles_namespace_and_preserves_mixed_tail() {
        let source=xml::parse(b"<svg xmlns='http://www.w3.org/2000/svg'><text>first<tspan>middle</tspan>last</text>tail</svg>",4096).unwrap();
        let target = xml::parse(b"<svg xmlns='http://www.w3.org/2000/svg'/>", 4096).unwrap();
        let parent = target.get_root_element().unwrap();
        for n in source.get_root_element().unwrap().get_child_nodes() {
            graft(&n, &parent, &target).unwrap();
        }
        drop(source);
        let bytes = xml::serialize(&target);
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("<text>first<tspan>middle</tspan>last</text>tail"));
        assert!(xml::parse(&bytes, 4096).is_ok());
    }
}
