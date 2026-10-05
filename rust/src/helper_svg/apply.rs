//! Apply a freshly computed plan only to an owned DOM; return no bytes for a no-op.
use super::{
    edit::{self, Request, Step},
    elements,
};
use libxml::tree::{Document, Node};
use std::collections::HashMap;
fn index(root: &Node) -> HashMap<String, Node> {
    elements(root.clone())
        .into_iter()
        .filter_map(|n| n.get_property_no_ns("id").map(|id| (id, n)))
        .collect()
}
fn node(by_id: &HashMap<String, Node>, id: &str) -> Result<Node, &'static str> {
    by_id.get(id).cloned().ok_or("invalid selection")
}
fn set(n: &mut Node, name: &str, value: &str) -> Result<(), &'static str> {
    n.set_property(name, value)
        .map_err(|_| "edit application failed")
}
fn append(parent: &mut Node, child: &mut Node) -> Result<(), &'static str> {
    child.unlink();
    parent
        .add_child(child)
        .map_err(|_| "edit application failed")
}
fn before(anchor: &mut Node, child: &mut Node) -> Result<(), &'static str> {
    child.unlink();
    anchor
        .add_prev_sibling(child)
        .map_err(|_| "edit application failed")
}
/// Elements own following XML text tails. Never move somebody else's comment/PI.
fn bundle(n: &Node) -> Vec<Node> {
    let mut result = vec![n.clone()];
    let mut next = n.get_next_sibling();
    while let Some(t) = next.filter(Node::is_text_node) {
        next = t.get_next_sibling();
        result.push(t);
    }
    result
}
fn clone_into(document: &mut Document, source: &Node) -> Result<Node, &'static str> {
    let copy = Document::dup_node_into_new_doc(source).map_err(|_| "edit application failed")?;
    let mut root = copy.get_root_element().ok_or("edit application failed")?;
    root.unlink();
    document
        .import_node(&mut root)
        .map_err(|_| "edit application failed")
}
pub struct Applied {
    pub bytes: Vec<u8>,
    pub ids: Vec<String>,
}
pub fn edit(svg: &str, request: &Request, cap: usize) -> Result<Applied, &'static str> {
    let plan = edit::plan(svg, request, cap)?;
    if !plan.changed() {
        return Ok(Applied {
            bytes: vec![],
            ids: plan.affected_ids,
        });
    }
    let mut document = crate::xml::parse(svg.as_bytes(), cap)?;
    let root = document
        .get_root_element()
        .ok_or("invalid document or selection")?;
    let by_id = index(&root);
    for step in plan.steps {
        match step {
            Step::Style {
                id,
                values,
                transform,
            } => {
                let mut n = node(&by_id, &id)?;
                if !values.is_empty() {
                    let raw = n.get_property_no_ns("style").unwrap_or_default();
                    let mut declarations: Vec<String> = raw
                        .split(';')
                        .filter(|part| {
                            !part.is_empty()
                                && !part
                                    .split_once(':')
                                    .is_some_and(|(key, _)| values.contains_key(key.trim()))
                        })
                        .map(str::to_string)
                        .collect();
                    declarations.extend(values.iter().map(|(k, v)| format!("{k}:{v}")));
                    set(&mut n, "style", &declarations.join(";"))?;
                }
                if let Some(t) = transform {
                    set(&mut n, "transform", &t)?;
                }
            }
            Step::Text { id, value } => {
                let n = node(&by_id, &id)?;
                let mut leaf = elements(n)
                    .into_iter()
                    .find(|n| n.get_child_elements().is_empty())
                    .ok_or("invalid selection")?;
                for mut n in leaf.get_child_nodes() {
                    n.unlink();
                }
                let mut text =
                    Node::new_text(&value, &document).map_err(|_| "edit application failed")?;
                append(&mut leaf, &mut text)?;
            }
            Step::Delete { ids } => {
                for id in ids {
                    for mut n in bundle(&node(&by_id, &id)?) {
                        n.unlink();
                    }
                }
            }
            Step::Duplicate { ids, remap } => {
                let remap: HashMap<_, _> = remap.into_iter().collect();
                for id in ids {
                    let source = node(&by_id, &id)?;
                    let mut clone = clone_into(&mut document, &source)?;
                    for mut n in elements(clone.clone()) {
                        for ((key, ns), value) in n.get_properties_ns() {
                            let changed = if key == "id" && ns.is_none() {
                                remap.get(&value).cloned().ok_or("invalid selection")?
                            } else {
                                super::fragment::remap_reference(&key, &value, &remap)
                            };
                            if changed != value {
                                match ns {
                                    Some(ns) => n.set_property_ns(&key, &changed, &ns),
                                    None => n.set_property(&key, &changed),
                                }
                                .map_err(|_| "edit application failed")?;
                            }
                        }
                    }
                    let tails = bundle(&source);
                    let mut anchor = tails.last().unwrap().clone();
                    anchor
                        .add_next_sibling(&mut clone)
                        .map_err(|_| "edit application failed")?;
                    // Clone the source tail without consuming it.
                    for t in tails.into_iter().skip(1) {
                        let mut text = Node::new_text(&t.get_content(), &document)
                            .map_err(|_| "edit application failed")?;
                        clone
                            .add_next_sibling(&mut text)
                            .map_err(|_| "edit application failed")?;
                    }
                }
            }
            Step::Group { ids, id } => {
                let mut first = node(&by_id, &ids[0])?;
                let mut group = Node::new("g", first.get_namespace(), &document)
                    .map_err(|_| "edit application failed")?;
                set(&mut group, "id", &id)?;
                before(&mut first, &mut group)?;
                for id in ids {
                    for mut n in bundle(&node(&by_id, &id)?) {
                        append(&mut group, &mut n)?;
                    }
                }
            }
            Step::Ungroup { id, children } => {
                let mut group = node(&by_id, &id)?;
                for (mut n, (_, transform)) in group.get_child_elements().into_iter().zip(children)
                {
                    set(&mut n, "transform", &transform)?;
                }
                for mut n in group.get_child_nodes() {
                    before(&mut group, &mut n)?;
                }
                group.unlink(); // The group's tail remains in the parent.
            }
            Step::Order {
                parent_path,
                indices,
            } => {
                let mut parent = root.clone();
                for i in parent_path {
                    parent = parent
                        .get_child_elements()
                        .get(i)
                        .cloned()
                        .ok_or("invalid selection")?;
                }
                let siblings = parent.get_child_elements();
                let bundles: Vec<_> = siblings.iter().map(bundle).collect();
                let mut ordered = indices.into_iter();
                let mut output = vec![];
                let tails: Vec<_> = bundles
                    .iter()
                    .flat_map(|b| b.iter().skip(1))
                    .cloned()
                    .collect();
                for n in parent.get_child_nodes() {
                    if n.is_element_node() {
                        output.extend(bundles[ordered.next().ok_or("invalid selection")?].clone());
                    } else if !tails.contains(&n) {
                        output.push(n);
                    }
                }
                for mut n in parent.get_child_nodes() {
                    n.unlink();
                }
                for mut n in output {
                    append(&mut parent, &mut n)?;
                }
            }
        }
    }
    let bytes = crate::xml::serialize(&document);
    if bytes.len() > cap {
        return Err("edit output exceeds size cap");
    }
    // Reparse before publication: IDs/namespaces/references must remain valid XML.
    crate::xml::parse(&bytes, cap)?;
    Ok(Applied {
        bytes,
        ids: plan.affected_ids,
    })
}
pub fn insert(svg: &str, fragment: &str, nonce: &str, cap: usize) -> Result<Applied, &'static str> {
    let (bytes, ids) = super::fragment::prepare(fragment, nonce)?;
    let mut document = crate::xml::parse(svg.as_bytes(), cap)?;
    let mut root = document
        .get_root_element()
        .ok_or("invalid document or selection")?;
    let existing: Vec<_> = elements(root.clone())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    if ids.iter().any(|id| existing.contains(id)) {
        return Err("insertion id collision");
    }
    let fragment = crate::xml::parse(&bytes, cap)?;
    let mut child = clone_into(&mut document, &fragment.get_root_element().unwrap())?;
    append(&mut root, &mut child)?;
    let bytes = crate::xml::serialize(&document);
    if bytes.len() > cap {
        return Err("edit output exceeds size cap");
    }
    crate::xml::parse(&bytes, cap)?;
    Ok(Applied { bytes, ids })
}
