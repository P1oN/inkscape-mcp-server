//! Bounded web cleanup through the existing snapshot/audit transaction.
use crate::{
    arguments, collection,
    document::{INKSCAPE_NS, Registry, elements},
    optimization_analysis as analysis, transaction, xml,
};
use libxml::{
    bindings,
    tree::{Document, Node, NodeType},
};
use serde_json::{Value, json};
use std::collections::HashSet;
const SODI: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd";
const CODES: [&str; 5] = [
    "editor_metadata",
    "unused_defs",
    "unreferenced_ids",
    "empty_groups",
    "reducible_coords",
];
fn remove(mut node: Node) {
    while let Some(mut tail) = node.get_next_sibling().filter(|n| {
        matches!(
            n.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode)
        )
    }) {
        tail.unlink_node();
    }
    node.unlink_node();
}
// The freshly reparsed tree has no detached nodes or outstanding Namespace wrappers.
// Only declarations unused by any element/attribute pointer can be freed.
fn cleanup(document: &Document) {
    let nodes = elements(document.get_root_element().unwrap());
    let mut used = HashSet::new();
    for n in &nodes {
        unsafe {
            let ptr = n.node_ptr();
            used.insert((*ptr).ns as usize);
            let mut attr = (*ptr).properties;
            while !attr.is_null() {
                used.insert((*attr).ns as usize);
                attr = (*attr).next;
            }
        }
    }
    for n in nodes {
        unsafe {
            let mut link = &raw mut (*n.node_ptr()).nsDef;
            while !(*link).is_null() {
                let ns = *link;
                if used.contains(&(ns as usize)) {
                    link = &raw mut (*ns).next;
                } else {
                    *link = (*ns).next;
                    (*ns).next = std::ptr::null_mut();
                    bindings::xmlFreeNs(ns);
                }
            }
        }
    }
}
fn mutate(
    document: &mut Document,
    precision: usize,
    keep: &HashSet<String>,
    cap: usize,
) -> Result<(String, Value), String> {
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    // Legacy cleanup cannot track selectors, SMIL, accessibility references or scripts.
    // Refuse before any DOM change instead of silently dropping their referenced IDs.
    for node in elements(root.clone()) {
        if ["style", "script"].contains(&node.get_name().as_str())
            || ["begin", "end", "aria-labelledby", "aria-describedby"]
                .iter()
                .any(|k| node.get_property_no_ns(k).is_some())
        {
            return Err("optimize cannot safely preserve stylesheet, script, animation or accessibility references".into());
        }
    }
    let before = xml::serialize(document).len();
    let reported = analysis::counts_with(root.clone(), precision, keep);
    let mut refs = analysis::references(root.clone());
    refs.extend(keep.iter().cloned());
    let required: HashSet<String> = elements(root.clone())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .filter(|id| refs.contains(id))
        .collect();
    let mut actual = [0usize; 5];
    let mut nodes = vec![];
    let mut stack = vec![root.clone()];
    while let Some(n) = stack.pop() {
        let children = n.get_child_nodes();
        stack.extend(children.into_iter().rev());
        nodes.push(n);
    }
    for mut n in nodes {
        if matches!(n.get_type(), Some(NodeType::CommentNode | NodeType::PiNode)) {
            if n.get_parent().is_some() {
                remove(n);
                actual[0] += 1;
            }
            continue;
        }
        if !n.is_element_node() {
            continue;
        }
        if n != root && ["namedview", "metadata"].contains(&n.get_name().as_str()) {
            if n.get_parent().is_some() {
                remove(n);
                actual[0] += 1;
            }
            continue;
        }
        for ((name, ns), _) in n.get_properties_ns() {
            if let Some(ns) = ns
                && [INKSCAPE_NS, SODI].contains(&ns.get_href().as_str())
            {
                n.remove_property_ns(&name, &ns.get_href())
                    .map_err(|_| "optimize attribute removal failed")?;
                actual[0] += 1;
            }
        }
    }
    for defs in elements(root.clone())
        .into_iter()
        .filter(|n| n.get_name() == "defs")
    {
        for child in defs.get_child_elements() {
            if !["style", "script"].contains(&child.get_name().as_str())
                && child
                    .get_property_no_ns("id")
                    .is_none_or(|id| !refs.contains(&id))
            {
                remove(child);
                actual[1] += 1;
            }
        }
    }
    for mut n in elements(root.clone()) {
        if n.get_property_no_ns("id")
            .is_some_and(|id| !refs.contains(&id))
        {
            n.remove_property_no_ns("id")
                .map_err(|_| "optimize attribute removal failed")?;
            actual[2] += 1;
        }
    }
    loop {
        let mut changed = false;
        for n in elements(root.clone()) {
            if n != root
                && ["g", "defs"].contains(&n.get_name().as_str())
                && n.get_property_no_ns("id").is_none()
                && n.get_child_elements().is_empty()
            {
                remove(n);
                actual[3] += 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for mut n in elements(root) {
        for ((name, ns), old) in n.get_properties_ns() {
            if !analysis::coordinate(&name) {
                continue;
            }
            let new = analysis::rounded(&old, precision);
            if new != old {
                if let Some(ns) = ns {
                    n.set_property_ns(&name, &new, &ns)
                        .map_err(|_| "optimize coordinate update failed")?;
                } else {
                    n.set_property(&name, &new)
                        .map_err(|_| "optimize coordinate update failed")?;
                }
                actual[4] += 1;
            }
        }
    }
    let remaining: HashSet<String> = elements(document.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    if !required.is_subset(&remaining) {
        return Err("optimize cannot remove an existing referenced target".into());
    }
    *document = xml::parse(&xml::serialize(document), cap)?;
    cleanup(document);
    let removed: serde_json::Map<String, Value> = CODES
        .into_iter()
        .zip(reported)
        .filter(|(_, n)| *n > 0)
        .map(|(code, n)| (code.into(), json!(n)))
        .collect();
    let deltas = json!({"bytes_before":before,"bytes_after":xml::serialize(document).len(),"removed":removed});
    Ok((
        format!(
            "web-optimized: stripped {} editor nodes/attrs, removed {} unused defs, {} unreferenced ids, {} empty groups, rounded {} coordinate attrs (precision {precision})",
            actual[0], actual[1], actual[2], actual[3], actual[4]
        ),
        deltas,
    ))
}
fn options(args: &Value) -> Result<(usize, HashSet<String>), String> {
    let precision = match args.get("precision") {
        None => 2,
        Some(v) => arguments::number(v)
            .filter(|n| n.is_finite() && n.fract() == 0.)
            .ok_or("precision must be an integer")? as i64,
    };
    if !(0..=8).contains(&precision) {
        return Err("precision must be between 0 and 8".into());
    }
    let keep = match args.get("keep_ids") {
        None | Some(Value::Null) => HashSet::new(),
        Some(v) => {
            let list = v
                .as_array()
                .filter(|l| l.len() <= 4096)
                .ok_or("keep_ids must contain at most 4096 strings")?;
            list.iter()
                .filter(|v| v.as_str() != Some(""))
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or("keep_ids must contain strings")
                })
                .collect::<Result<HashSet<_>, _>>()?
        }
    };
    Ok((precision as usize, keep))
}
fn one(registry: &Registry, id: &str, args: &Value) -> Result<Value, String> {
    let (precision, keep) = options(args)?;
    let mut deltas = Value::Null;
    let mut result = transaction::apply_dom(
        registry,
        id,
        "svg_web_optimize",
        json!({"precision":precision,"keep_ids":args.get("keep_ids").unwrap_or(&Value::Null)}),
        "medium",
        None,
        |doc| {
            let (summary, values) = mutate(doc, precision, &keep, registry.workspace.max_input)?;
            deltas = values;
            Ok(summary)
        },
    )?;
    for (key, value) in deltas
        .as_object()
        .ok_or("optimize did not produce deltas")?
    {
        result[key] = value.clone();
    }
    Ok(result)
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    if tool == "svg_web_optimize" {
        return one(
            registry,
            args["doc_id"].as_str().ok_or("doc_id must be a string")?,
            args,
        );
    }
    let ids = collection::ids(args)?;
    let verdict = collection::verdict(registry, &ids)?;
    let mut entries = vec![];
    let mut before = 0i64;
    let mut after = 0i64;
    let mut changed = 0;
    for id in ids {
        let result = one(registry, id, args)?;
        before += result["bytes_before"].as_i64().unwrap();
        after += result["bytes_after"].as_i64().unwrap();
        changed += usize::from(result["changed"] == true);
        entries.push(json!({"doc_id":id,"result":result}));
    }
    Ok(
        json!({"per_doc":entries,"total_bytes_before":before,"total_bytes_after":after,"total_bytes_saved":before-after,"changed_count":changed,"consistency":verdict}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn namespace_cleanup_preserves_used_prefixes_references_and_idempotence() {
        let mut doc=xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:i="http://www.inkscape.org/namespaces/inkscape" xmlns:x="http://www.w3.org/1999/xlink" xmlns:a="urn:unused"><defs><g id="target"/></defs><use x:href="#target" x="2.675"/><g/><rect i:label="kept" width="10"/></svg>"##,4096).unwrap();
        let (_, delta) = mutate(&mut doc, 2, &HashSet::new(), 4096).unwrap();
        let bytes = xml::serialize(&doc);
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(!text.contains("xmlns:a") && !text.contains("xmlns:i"));
        assert!(
            text.contains("xmlns:x")
                && text.contains("x:href=\"#target\"")
                && text.contains("id=\"target\"")
        );
        assert_eq!(delta["removed"]["editor_metadata"], 1);
        let (_, delta) = mutate(&mut doc, 2, &HashSet::new(), 4096).unwrap();
        assert_eq!(delta["removed"], json!({}));
        assert_eq!(xml::serialize(&doc), bytes);
    }
    #[test]
    fn unsafe_references_refuse_before_mutation_and_referenced_metadata_is_not_committed() {
        let source=br#"<svg xmlns="http://www.w3.org/2000/svg"><style>#r{fill:red}</style><rect id="r"/></svg>"#;
        let mut doc = xml::parse(source, 4096).unwrap();
        let before = xml::serialize(&doc);
        assert!(mutate(&mut doc, 2, &HashSet::new(), 4096).is_err());
        assert_eq!(xml::serialize(&doc), before);
        let mut doc=xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg"><metadata id="target"/><use href="#target"/></svg>"##,4096).unwrap();
        assert_eq!(
            mutate(&mut doc, 2, &HashSet::new(), 4096).unwrap_err(),
            "optimize cannot remove an existing referenced target"
        );
    }
}
