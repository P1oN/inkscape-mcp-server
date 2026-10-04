//! Explicitly approved deletion preserves sequential target and mixed-content semantics.
use crate::{
    arguments, css_assets,
    document::{Registry, elements},
    reparent, structure, style, transaction, xml,
};
use libxml::tree::{Document, NodeType};
use serde_json::{Value, json};
use std::collections::HashSet;
pub struct Mutation {
    pub ids: Vec<String>,
}
impl Mutation {
    pub fn build(args: &Value) -> Result<Self, String> {
        let ids = args["object_ids"]
            .as_array()
            .ok_or("object_ids must be a list")?
            .iter()
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .ok_or("object_ids must contain strings")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if ids.is_empty() {
            return Err("no target object ids supplied".into());
        }
        if ids.len() > 4096 {
            return Err("delete target list exceeds 4096 ids".into());
        }
        Ok(Self { ids })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let nodes = elements(root.clone());
        let mut removed_nodes = HashSet::new();
        let mut removed_ids = HashSet::new();
        for target in nodes.iter().filter(|n| {
            n.get_property_no_ns("id")
                .is_some_and(|id| self.ids.contains(&id))
        }) {
            if *target == root {
                return Err("cannot delete the document root".into());
            }
            for node in elements(target.clone()) {
                removed_nodes.insert(node.node_ptr() as usize);
                if let Some(id) = node.get_property_no_ns("id") {
                    removed_ids.insert(id);
                }
            }
        }
        if !removed_ids.is_empty() {
            // A stylesheet can hide references in escaped CSS or external imports.
            structure::reject_stylesheets(document)?;
            for (mut sibling, before) in [
                (root.get_prev_sibling(), true),
                (root.get_next_sibling(), false),
            ] {
                while let Some(node) = sibling {
                    if node.get_name() == "xml-stylesheet" {
                        return Err(
                            "delete cannot safely inspect external stylesheet references".into(),
                        );
                    }
                    sibling = if before {
                        node.get_prev_sibling()
                    } else {
                        node.get_next_sibling()
                    };
                }
            }
            for node in nodes
                .iter()
                .filter(|n| !removed_nodes.contains(&(n.node_ptr() as usize)))
            {
                let mut refs = reparent::references(node);
                for ((key, _), value) in node.get_properties_ns() {
                    if matches!(key.as_str(), "aria-labelledby" | "aria-describedby") {
                        refs.extend(value.split_whitespace().map(str::to_owned));
                    } else if matches!(key.as_str(), "begin" | "end") {
                        // IDs and offsets can both contain dots. Consider each
                        // event-base prefix conservatively instead of splitting
                        // at a decimal offset's final dot.
                        for term in value.split(';').map(str::trim) {
                            refs.extend(term.match_indices('.').map(|(i, _)| term[..i].to_owned()));
                        }
                    } else if matches!(
                        key.as_str(),
                        "style"
                            | "fill"
                            | "stroke"
                            | "mask"
                            | "clip-path"
                            | "filter"
                            | "marker"
                            | "marker-start"
                            | "marker-mid"
                            | "marker-end"
                            | "cursor"
                    ) {
                        css_assets::rewrite(&value, |url| {
                            if let Some(id) = url.strip_prefix('#') {
                                refs.insert(id.to_owned());
                            }
                            Ok(None)
                        })
                        .map_err(|_| "delete cannot safely inspect CSS references")?;
                    }
                }
                for raw in refs.clone().into_iter().filter(|id| id.contains('%')) {
                    let decoded = crate::engine_input::uri_path(&raw)
                        .map_err(|_| "delete cannot safely inspect encoded references")?;
                    refs.insert(decoded.to_string_lossy().into_owned());
                }
                if !refs.is_disjoint(&removed_ids) {
                    return Err(
                        "delete refused: remaining objects reference IDs in the removed subtree"
                            .into(),
                    );
                }
            }
        }
        let mut removed = Vec::new();
        for id in &self.ids {
            let Some(mut target) = elements(root.clone())
                .into_iter()
                .find(|n| n.get_property_no_ns("id").as_ref() == Some(id))
            else {
                continue;
            };
            if !target.get_parent().is_some_and(|p| p.is_element_node()) {
                return Err(format!(
                    "cannot delete the document root ({})",
                    style::python_repr(id)
                ));
            }
            // lxml parent.remove removes the element and its tail. The Rust DOM stores tail as siblings.
            while let Some(mut tail) = target.get_next_sibling().filter(|n| {
                matches!(
                    n.get_type(),
                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                )
            }) {
                tail.unlink_node();
            }
            target.unlink_node();
            removed.push(style::python_repr(id));
        }
        Ok(if removed.is_empty() {
            "no matching objects to delete".into()
        } else {
            format!(
                "deleted {} object(s): {}",
                removed.len(),
                removed.join(", ")
            )
        })
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    if args["object_ids"].as_array().is_some_and(Vec::is_empty) {
        return Err("delete_object requires at least one object id".into());
    }
    let mutation = Mutation::build(args)?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let document = xml::parse(&bytes, registry.workspace.max_input)?;
    let nodes = elements(
        document
            .get_root_element()
            .ok_or("document could not be parsed safely")?,
    );
    let affected = mutation
        .ids
        .iter()
        .filter(|id| {
            nodes
                .iter()
                .any(|n| n.get_property_no_ns("id").as_ref() == Some(*id))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut result = transaction::apply_dom(
        registry,
        id,
        "delete_object",
        json!({"object_ids":mutation.ids}),
        "high",
        arguments::string(args, "approval_token")?,
        |document| mutation.mutate(document),
    )?;
    result["affected_ids"] = if result["changed"].as_bool() == Some(true) {
        json!(affected)
    } else {
        json!([])
    };
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn referenced_subtrees_refuse_before_mutation_and_joint_deletion_is_allowed() {
        for reference in [
            r##"<use href="#r"/>"##,
            r##"<use href="#%72"/>"##,
            r##"<use xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="#r"/>"##,
            r##"<rect fill="URL('#r')"/>"##,
            r##"<rect style="fill:u\72l('#r')"/>"##,
            r##"<animate begin="r.click+1s"/>"##,
            r##"<animate begin="r.click+1.5s"/>"##,
            r##"<text aria-labelledby="r other"/>"##,
        ] {
            let mut document = xml::parse(
                format!("<svg><g id=\"g\"><rect id=\"r\"/></g>{reference}</svg>").as_bytes(),
                4096,
            )
            .unwrap();
            let before = xml::serialize(&document);
            assert!(
                Mutation::build(&json!({"object_ids":["g"]}))
                    .unwrap()
                    .mutate(&mut document)
                    .is_err(),
                "{reference}"
            );
            assert_eq!(xml::serialize(&document), before);
        }
        let mut document = xml::parse(
            br##"<svg><g id="g"><rect id="r"/><use href="#r"/></g><use id="u" href="#r"/></svg>"##,
            4096,
        )
        .unwrap();
        assert!(
            Mutation::build(&json!({"object_ids":["g","u"]}))
                .unwrap()
                .mutate(&mut document)
                .is_ok()
        );
        assert!(elements(document.get_root_element().unwrap()).len() == 1);
        let mut document = xml::parse(
            br#"<?xml-stylesheet type="text/css" href="theme.css"?><svg><rect id="r"/></svg>"#,
            4096,
        )
        .unwrap();
        let before = xml::serialize(&document);
        assert!(
            Mutation::build(&json!({"object_ids":["r"]}))
                .unwrap()
                .mutate(&mut document)
                .is_err()
        );
        assert_eq!(xml::serialize(&document), before);
    }
    #[test]
    fn deletion_removes_tail_and_skips_detached_descendants() {
        let mut document=xml::parse(br#"<svg id="root"><text>A<tspan id="span">B</tspan>C<tspan>D</tspan>E</text><g id="g"><rect id="r"/></g>tail</svg>"#,4096).unwrap();
        let mutation = Mutation::build(&json!({"object_ids":["span","g","r","g"]})).unwrap();
        assert_eq!(
            mutation.mutate(&mut document).unwrap(),
            "deleted 2 object(s): 'span', 'g'"
        );
        assert!(
            document
                .to_string()
                .contains("<text>A<tspan>D</tspan>E</text></svg>")
        );
        assert!(
            Mutation::build(&json!({"object_ids":["root"]}))
                .unwrap()
                .mutate(&mut document)
                .is_err()
        );
    }
}
