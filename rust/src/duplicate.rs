//! Bounded native subtree copies share the ID/reference edit kernel.
use crate::{
    arguments, create,
    document::{Registry, elements},
    identity, structure, style, transaction,
};
use libxml::{
    bindings,
    tree::{Document, Node, NodeType},
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};
static QUOTED: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"['"]#([^'"]+)['"]"#).unwrap());

/// Raw copies remain uniquely owned until attachment transfers them to the DOM.
struct Copy(bindings::xmlNodePtr);
impl Drop for Copy {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: this unattached copy is owned only by this guard.
            unsafe { bindings::xmlFreeNode(self.0) };
        }
    }
}
fn copy(source: &Node, document: &Document) -> Result<Copy, String> {
    // SAFETY: source and destination document remain live for this synchronous copy.
    let ptr = unsafe { bindings::xmlDocCopyNode(source.node_ptr(), document.doc_ptr(), 1) };
    if ptr.is_null() {
        Err("subtree copy failed".into())
    } else {
        Ok(Copy(ptr))
    }
}
fn append_after(anchor: &Node, copy: &mut Copy) -> Result<Node, String> {
    // SAFETY: anchor is live and attached; the guard owns an unattached node in
    // the same document. On success the document assumes ownership exactly once.
    let ptr = unsafe { bindings::xmlAddNextSibling(anchor.node_ptr(), copy.0) };
    if ptr.is_null() {
        return Err("subtree copy insertion failed".into());
    }
    copy.0 = std::ptr::null_mut();
    anchor
        .get_next_sibling()
        .ok_or("subtree copy insertion failed".into())
}
fn suffix() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..6].to_owned()
}
fn allocate(
    prefix: &str,
    supplied: Option<&str>,
    existing: &mut HashSet<String>,
    mut next: impl FnMut() -> String,
) -> Result<(String, String), String> {
    for _ in 0..128 {
        let token = next();
        let top = supplied
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{prefix}-{token}"));
        if existing.insert(top.clone()) {
            return Ok((token, top));
        }
        if supplied.is_some() {
            return Err(format!("id already in use: {}", style::python_repr(&top)));
        }
    }
    Err("unable to allocate an unused clone id".into())
}
pub(crate) fn validate_copy(document: &Document, target: &Node) -> Result<(), String> {
    structure::reject_stylesheets(document)?;
    let subtree = elements(target.clone());
    let old_ids = subtree
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect::<HashSet<_>>();
    for node in &subtree {
        for key in ["begin", "end", "aria-labelledby", "aria-describedby"] {
            if node.get_property_no_ns(key).is_some_and(|value| {
                if key.starts_with("aria-") {
                    value.split_whitespace().any(|id| old_ids.contains(id))
                } else {
                    value
                        .split(';')
                        .filter_map(|term| term.trim().rsplit_once('.').map(|(id, _)| id))
                        .any(|id| old_ids.contains(id))
                }
            }) {
                return Err(
                    "duplicate cannot safely preserve timing or accessibility references".into(),
                );
            }
        }
        for (key, value) in node.get_properties() {
            if QUOTED
                .captures_iter(&value)
                .any(|c| old_ids.contains(&c[1]))
            {
                return Err("duplicate cannot safely preserve quoted paint references".into());
            }
            if matches!(key.as_str(), "fill" | "stroke")
                && value.starts_with('#')
                && old_ids.contains(&value[1..])
                && style::color(&value).is_ok()
            {
                return Err(
                    "duplicate cannot safely distinguish an ID reference from a color".into(),
                );
            }
        }
    }
    Ok(())
}
pub(crate) fn insert_copy(
    document: &Document,
    target: &Node,
    after: &Node,
    new_id: Option<&str>,
    existing: &mut HashSet<String>,
) -> Result<Node, String> {
    let source_id = target
        .get_property_no_ns("id")
        .ok_or("copy source has no id")?;
    let (token, top) = allocate(&source_id, new_id, existing, suffix)?;
    let mapping = HashMap::from([(source_id, top.clone())]);
    // Copy and attach only on the pipeline's disposable candidate. All
    // public state still waits for validated serialization and atomic commit.
    let mut clone = copy(target, document)?;
    let mut tails = Vec::new();
    let mut tail_source = target.clone();
    while let Some(tail) = tail_source.get_next_sibling().filter(|n| {
        matches!(
            n.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode | NodeType::EntityRefNode)
        )
    }) {
        tails.push(copy(&tail, document)?);
        tail_source = tail;
    }
    let mut anchor = after.clone();
    while let Some(tail) = anchor.get_next_sibling().filter(|n| {
        matches!(
            n.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode | NodeType::EntityRefNode)
        )
    }) {
        anchor = tail;
    }
    let mut cloned = append_after(&anchor, &mut clone)?;
    // SAFETY: the copied subtree is now attached to this document. Remove
    // redundant inherited declarations, matching lxml's adoption behavior.
    if unsafe {
        bindings::xmlDOMWrapReconcileNamespaces(std::ptr::null_mut(), cloned.node_ptr(), 1)
    } < 0
    {
        return Err("subtree namespace reconciliation failed".into());
    }
    remint(document, &mut cloned, &top, &token, existing, mapping)?;
    let mut anchor = cloned.clone();
    for tail in &mut tails {
        anchor = append_after(&anchor, tail)?;
    }
    Ok(cloned)
}
pub(crate) fn append_asset(
    document: &Document,
    source: &Node,
    parent: &mut Node,
    existing: &mut HashSet<String>,
) -> Result<(Node, String), String> {
    let old = source.get_property_no_ns("id").filter(|s| !s.is_empty());
    let (token, top) = allocate(old.as_deref().unwrap_or("copy"), None, existing, suffix)?;
    let mapping = old.into_iter().map(|old| (old, top.clone())).collect();
    let mut asset_copy = copy(source, document)?;
    let pointer = unsafe { bindings::xmlAddChild(parent.node_ptr(), asset_copy.0) };
    if pointer.is_null() {
        return Err("subtree copy insertion failed".into());
    }
    asset_copy.0 = std::ptr::null_mut();
    if unsafe { bindings::xmlDOMWrapReconcileNamespaces(std::ptr::null_mut(), pointer, 1) } < 0 {
        return Err("subtree namespace reconciliation failed".into());
    }
    let mut cloned = parent
        .get_last_child()
        .ok_or("subtree copy insertion failed")?;
    remint(document, &mut cloned, &top, &token, existing, mapping)?;
    // Preserve the source subtree's tail, as copy.deepcopy/lxml append do.
    let mut next = source.get_next_sibling();
    while let Some(tail) = next.filter(|n| {
        matches!(
            n.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode)
        )
    }) {
        next = tail.get_next_sibling();
        let mut tail_copy = copy(&tail, document)?;
        if unsafe { bindings::xmlAddChild(parent.node_ptr(), tail_copy.0) }.is_null() {
            return Err("subtree tail insertion failed".into());
        }
        tail_copy.0 = std::ptr::null_mut();
    }
    Ok((cloned, top))
}
fn remint(
    document: &Document,
    cloned: &mut Node,
    top: &str,
    token: &str,
    existing: &mut HashSet<String>,
    mut mapping: HashMap<String, String>,
) -> Result<(), String> {
    let mut clone_nodes = elements(cloned.clone());
    cloned
        .set_property("id", top)
        .map_err(|_| "identity edit failed")?;
    for node in clone_nodes.iter_mut().skip(1) {
        let Some(old) = node.get_property_no_ns("id").filter(|id| !id.is_empty()) else {
            continue;
        };
        let mut candidate = format!("{old}-{token}");
        let mut attempts = 0;
        while existing.contains(&candidate) {
            if attempts >= 128 {
                return Err("unable to allocate an unused clone id".into());
            }
            candidate = format!("{old}-{}", suffix());
            attempts += 1;
        }
        existing.insert(candidate.clone());
        mapping.insert(old, candidate.clone());
        node.set_property("id", &candidate)
            .map_err(|_| "identity edit failed")?;
    }
    identity::rewrite_references(document, &mut clone_nodes, &mapping)?;
    Ok(())
}
pub struct Mutation {
    id: String,
    new: Option<String>,
    pub params: Value,
}
impl Mutation {
    pub fn build(args: &Value) -> Result<Self, String> {
        let id = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?
            .to_owned();
        let new = arguments::string(args, "new_id")?.map(str::to_owned);
        if let Some(new) = &new
            && !create::valid_id(new)
        {
            return Err(format!("invalid object id: {}", style::python_repr(new)));
        }
        let params = json!({"object_id":id,"new_id":new});
        Ok(Self { id, new, params })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let nodes = elements(root);
        let target = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_ref() == Some(&self.id))
            .cloned()
            .ok_or("object id not found in document")?;
        if !target.get_parent().is_some_and(|p| p.is_element_node()) {
            return Err(format!(
                "object {} cannot be duplicated (it is the document root)",
                style::python_repr(&self.id)
            ));
        }
        let mut existing = nodes
            .iter()
            .filter_map(|n| n.get_property_no_ns("id"))
            .collect::<HashSet<_>>();
        if let Some(new) = &self.new
            && existing.contains(new)
        {
            return Err(format!("id already in use: {}", style::python_repr(new)));
        }
        validate_copy(document, &target)?;
        let cloned = insert_copy(
            document,
            &target,
            &target,
            self.new.as_deref(),
            &mut existing,
        )?;
        let top = cloned
            .get_property_no_ns("id")
            .ok_or("copy source has no id")?;
        Ok(format!(
            "duplicated {} as {}",
            style::python_repr(&self.id),
            style::python_repr(&top)
        ))
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let m = Mutation::build(args)?;
    transaction::apply_dom(
        registry,
        id,
        "duplicate_object",
        m.params.clone(),
        "medium",
        None,
        |d| m.mutate(d),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn root_id_collisions_retry_boundedly_and_explicit_ids_still_refuse() {
        let mut existing = HashSet::from(["r-000001".into()]);
        let mut tokens = ["000001", "000002"].into_iter();
        assert_eq!(
            allocate("r", None, &mut existing, || tokens.next().unwrap().into()).unwrap(),
            ("000002".into(), "r-000002".into())
        );
        assert_eq!(
            allocate("r", Some("r-000001"), &mut existing, || "000003".into()).unwrap_err(),
            "id already in use: 'r-000001'"
        );
        let mut attempts = 0;
        assert_eq!(
            allocate("r", None, &mut existing, || {
                attempts += 1;
                "000001".into()
            })
            .unwrap_err(),
            "unable to allocate an unused clone id"
        );
        assert_eq!(attempts, 128);
    }

    #[test]
    fn cloned_ids_and_internal_references_are_unique_original_refs_stay_intact() {
        let mut d=crate::xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg"><g id="g"><rect id="r"/><use href="#r"/></g><use href="#r"/></svg>"##,4096).unwrap();
        Mutation::build(&json!({"object_id":"g","new_id":"copy"}))
            .unwrap()
            .mutate(&mut d)
            .unwrap();
        let nodes = elements(d.get_root_element().unwrap());
        let clone = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some("copy"))
            .unwrap();
        let copied = elements(clone.clone());
        let rect = copied
            .iter()
            .find(|n| n.get_name() == "rect")
            .unwrap()
            .get_property_no_ns("id")
            .unwrap();
        assert!(rect.starts_with("r-"));
        assert_eq!(rect.len(), 8);
        assert_eq!(
            copied
                .iter()
                .find(|n| n.get_name() == "use")
                .unwrap()
                .get_property_no_ns("href")
                .unwrap(),
            format!("#{rect}")
        );
        assert_eq!(
            nodes.last().unwrap().get_property_no_ns("href").as_deref(),
            Some("#r")
        );
        crate::xml::parse(&crate::xml::serialize(&d), 4096).unwrap();
    }
    #[test]
    fn cloned_tail_and_sibling_order_match_element_insertion() {
        let mut d =
            crate::xml::parse(br#"<svg><rect id="r"/>tail<!--c--><circle/></svg>"#, 4096).unwrap();
        Mutation::build(&json!({"object_id":"r","new_id":"copy"}))
            .unwrap()
            .mutate(&mut d)
            .unwrap();
        assert!(
            d.to_string()
                .contains("<rect id=\"r\"/>tail<rect id=\"copy\"/>tail<!--c--><circle/>")
        );
    }
}
