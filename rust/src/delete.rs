//! Explicitly approved deletion preserves sequential target and mixed-content semantics.
use crate::{
    arguments,
    document::{Registry, elements},
    style, transaction, xml,
};
use libxml::tree::{Document, NodeType};
use serde_json::{Value, json};
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
