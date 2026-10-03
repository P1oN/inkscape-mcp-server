//! Container-stable allowlisted replacement with explicit external reference policy.
use crate::{
    adopt, arguments, create,
    document::{Registry, elements},
    reparent, structure, style, transaction, validate, xml,
};
use libxml::{
    bindings,
    tree::{Document, Node, c14n::CanonicalizationOptions},
};
use serde_json::{Value, json};
use std::collections::HashSet;
fn ids(nodes: &[Node]) -> HashSet<String> {
    nodes
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .filter(|id| !id.is_empty())
        .collect()
}
fn canonical(node: &Node) -> Result<String, String> {
    Document::dup_node_into_new_doc(node)
        .map_err(|_| "fragment copy failed")?
        .canonicalize(
            CanonicalizationOptions {
                with_comments: true,
                ..Default::default()
            },
            None,
        )
        .map_err(|_| "fragment canonicalization failed".into())
}
struct Copy(bindings::xmlNodePtr);
impl Drop for Copy {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { bindings::xmlFreeNode(self.0) };
        }
    }
}
fn mutate(
    document: &mut Document,
    replacement: &Node,
    object_id: &str,
    policy: &str,
) -> Result<String, String> {
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let nodes = elements(root.clone());
    let mut target = nodes
        .iter()
        .find(|n| n.get_property_no_ns("id").as_deref() == Some(object_id))
        .cloned()
        .ok_or("object id not found in document")?;
    if target == root {
        return Err("use set_document_svg for the document root".into());
    }
    if target.get_name() != replacement.get_name()
        || target.get_namespace().map(|n| n.get_href())
            != replacement.get_namespace().map(|n| n.get_href())
    {
        return Err("replacement must keep the selected element's qualified tag".into());
    }
    structure::reject_stylesheets(document)?;
    let subtree = elements(target.clone());
    let inside: HashSet<_> = subtree.iter().map(|n| n.node_ptr() as usize).collect();
    let outside: Vec<_> = nodes
        .into_iter()
        .filter(|n| !inside.contains(&(n.node_ptr() as usize)))
        .collect();
    let old_ids = ids(&subtree);
    let new_nodes = elements(replacement.clone());
    let new_ids = ids(&new_nodes);
    let outside_ids = ids(&outside);
    if !new_ids.is_disjoint(&outside_ids) {
        return Err("fragment ID conflicts with the rest of the document".into());
    }
    let known: HashSet<_> = new_ids.union(&outside_ids).cloned().collect();
    if new_nodes
        .iter()
        .any(|n| !reparent::references(n).is_subset(&known))
    {
        return Err("fragment contains unresolved references".into());
    }
    let removed: HashSet<_> = old_ids.difference(&new_ids).cloned().collect();
    let external: HashSet<_> = outside.iter().flat_map(reparent::references).collect();
    if !external.is_disjoint(&removed) {
        return Err("rest of document references IDs removed by fragment replacement".into());
    }
    for n in &outside {
        let extra: HashSet<String> = ["aria-labelledby", "aria-describedby"]
            .iter()
            .filter_map(|key| n.get_property_no_ns(key))
            .flat_map(|v| v.split_whitespace().map(str::to_owned).collect::<Vec<_>>())
            .chain(
                ["begin", "end"]
                    .iter()
                    .filter_map(|key| n.get_property_no_ns(key))
                    .flat_map(|v| {
                        v.split(';')
                            .filter_map(|term| {
                                term.trim().rsplit_once('.').map(|(id, _)| id.to_owned())
                            })
                            .collect::<Vec<_>>()
                    }),
            )
            .collect();
        if !extra.is_disjoint(&removed) {
            return Err("fragment cannot remove timing or accessibility reference targets".into());
        }
    }
    if canonical(&target)? == canonical(replacement)? {
        return Ok("fragment already matches".into());
    }
    if policy == "reject_changes" && !external.is_disjoint(&old_ids) {
        return Err("retained subtree IDs are externally referenced; explicitly allow_retained to update their appearance".into());
    }
    // Own the detached recursive copy until the destination takes ownership.
    let mut copy =
        Copy(unsafe { bindings::xmlDocCopyNode(replacement.node_ptr(), document.doc_ptr(), 1) });
    if copy.0.is_null() {
        return Err("fragment copy failed".into());
    }
    let attached = unsafe { bindings::xmlAddPrevSibling(target.node_ptr(), copy.0) };
    if attached.is_null() {
        return Err("fragment replacement failed".into());
    }
    copy.0 = std::ptr::null_mut();
    if unsafe { bindings::xmlDOMWrapReconcileNamespaces(std::ptr::null_mut(), attached, 1) } < 0 {
        return Err("fragment namespace reconciliation failed".into());
    }
    // Tail is represented by following text siblings; leave them after the new element.
    target.unlink_node();
    Ok(format!(
        "replaced fragment {}; retained {} IDs, removed {}; external reference policy {policy}",
        style::python_repr(object_id),
        old_ids.intersection(&new_ids).count(),
        removed.len()
    ))
}
pub(crate) struct Mutation {
    incoming: Document,
    object: String,
    policy: String,
}
impl Mutation {
    pub(crate) fn build(args: &Value, max_bytes: usize) -> Result<Self, String> {
        let object = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?;
        let raw = args["svg"].as_str().ok_or("svg must be a string")?;
        let policy = arguments::string(args, "reference_policy")?.unwrap_or("reject_changes");
        if raw.len() > max_bytes {
            return Err("input svg exceeds the configured size limit".into());
        }
        if !["reject_changes", "allow_retained"].contains(&policy) {
            return Err("invalid external reference policy".into());
        }
        let incoming = xml::parse(raw.as_bytes(), max_bytes)?;
        adopt::scrub(&incoming)?;
        let mut replacement = incoming.get_root_element().unwrap();
        if replacement
            .get_property_no_ns("id")
            .is_some_and(|id| id != object)
        {
            return Err("fragment root ID must match selected container ID (or be omitted)".into());
        }
        replacement
            .set_property("id", object)
            .map_err(|_| "fragment identity update failed")?;
        let nodes = elements(replacement.clone());
        let raw_ids: Vec<_> = nodes
            .iter()
            .filter_map(|n| n.get_property_no_ns("id"))
            .filter(|id| !id.is_empty())
            .collect();
        if raw_ids.len() != raw_ids.iter().collect::<HashSet<_>>().len() {
            return Err("duplicate IDs inside fragment".into());
        }
        if raw_ids.iter().any(|id| !create::valid_id(id)) {
            return Err("fragment contains an invalid ID".into());
        }
        Ok(Self {
            incoming,
            object: object.to_owned(),
            policy: policy.to_owned(),
        })
    }
    pub(crate) fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let replacement = self
            .incoming
            .get_root_element()
            .ok_or("fragment root is missing")?;
        mutate(document, &replacement, &self.object, &self.policy)
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let object = args["object_id"]
        .as_str()
        .ok_or("object_id must be a string")?;
    let _raw = args["svg"].as_str().ok_or("svg must be a string")?;
    let policy = arguments::string(args, "reference_policy")?.unwrap_or("reject_changes");
    let approval = arguments::string(args, "approval_token")?;
    if approval.is_none_or(str::is_empty) {
        return Err("high-risk compose operation requires an explicit approval_token".into());
    }
    let mutation = Mutation::build(args, registry.workspace.max_input)?;
    let mut result = transaction::apply_dom(
        registry,
        id,
        "replace_svg_fragment",
        json!({"object_id":object,"reference_policy":policy}),
        "high",
        approval,
        |doc| mutation.mutate(doc),
    )?;
    result["validation"] = validate::document(registry, id)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_limit_and_approval_precede_fragment_parse() {
        let registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![],
                max_input: 4,
                max_output: 4096,
            },
            entries: indexmap::IndexMap::new(),
        };
        assert_eq!(
            apply(
                &registry,
                &json!({"doc_id":"missing","object_id":"g","svg":"<broken"})
            )
            .unwrap_err(),
            "high-risk compose operation requires an explicit approval_token"
        );
        assert_eq!(apply(&registry,&json!({"doc_id":"missing","object_id":"g","svg":"<g/>","approval_token":"approved"})).unwrap_err(),"document id not found");
        assert_eq!(apply(&registry,&json!({"doc_id":"missing","object_id":"g","svg":"<g />","approval_token":"approved"})).unwrap_err(),"input svg exceeds the configured size limit");
    }
    #[test]
    fn canonical_noop_preserves_original_attribute_order_and_tail() {
        let mut doc=xml::parse(br#"<svg xmlns="http://www.w3.org/2000/svg"><g id="g"><rect id="r" width="2" height="3"/></g>tail</svg>"#,4096).unwrap();
        let incoming=xml::parse(br#"<g xmlns="http://www.w3.org/2000/svg" id="g"><rect height="3" width="2" id="r"/></g>"#,4096).unwrap();
        let before = xml::serialize(&doc);
        assert_eq!(
            mutate(
                &mut doc,
                &incoming.get_root_element().unwrap(),
                "g",
                "reject_changes"
            )
            .unwrap(),
            "fragment already matches"
        );
        assert_eq!(xml::serialize(&doc), before);
    }
    #[test]
    fn replacement_keeps_slot_tail_and_external_policy() {
        let mut doc=xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg"><g id="g"><rect id="r" width="2"/></g>tail<use href="#r"/></svg>"##,4096).unwrap();
        let incoming = xml::parse(
            br#"<g xmlns="http://www.w3.org/2000/svg" id="g"><rect id="r" width="3"/></g>"#,
            4096,
        )
        .unwrap();
        let replacement = incoming.get_root_element().unwrap();
        let before = xml::serialize(&doc);
        assert!(mutate(&mut doc, &replacement, "g", "reject_changes").is_err());
        assert_eq!(xml::serialize(&doc), before);
        mutate(&mut doc, &replacement, "g", "allow_retained").unwrap();
        let bytes = xml::serialize(&doc);
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("width=\"3\""));
        assert!(text.contains("</g>tail<use href=\"#r\""));
        assert!(xml::parse(&bytes, 4096).is_ok());
    }
}
