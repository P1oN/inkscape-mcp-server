//! Conservative neutral grouping; appearance ambiguity fails on the disposable DOM.
use crate::{
    arguments, create,
    document::{Registry, elements},
    style, transaction,
};
use libxml::tree::{Document, Node, NodeType};
use serde_json::{Value, json};
pub(crate) fn paint_order(root: Node) -> Vec<usize> {
    let mut order = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match node.get_name().as_str() {
            "svg" | "g" => stack.extend(node.get_child_elements().into_iter().rev()),
            "defs" | "metadata" | "title" | "desc" | "namedview" | "" => (),
            _ => order.push(node.node_ptr() as usize),
        }
    }
    order
}
pub(crate) fn reject_stylesheets(document: &Document) -> Result<(), String> {
    if elements(
        document
            .get_root_element()
            .ok_or("document could not be parsed safely")?,
    )
    .iter()
    .any(|n| n.get_name() == "style")
    {
        return Err("structural preservation with stylesheets requires inline styles first".into());
    }
    Ok(())
}
pub struct Mutation {
    ids: Vec<String>,
    id: Option<String>,
    pub params: Value,
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
            return Err("group_objects requires at least one object id".into());
        }
        if ids.len() > 4096 {
            return Err("group target list exceeds 4096 ids".into());
        }
        let id = arguments::string(args, "object_id")?.map(str::to_owned);
        let params = json!({"object_ids":ids});
        Ok(Self { ids, id, params })
    }
    pub fn create(&self, document: &mut Document) -> Result<(String, String), String> {
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let nodes = elements(root.clone());
        let mut targets = self
            .ids
            .iter()
            .map(|id| {
                nodes
                    .iter()
                    .find(|n| n.get_property_no_ns("id").as_ref() == Some(id))
                    .cloned()
                    .ok_or("object id not found in document")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let parent = targets[0]
            .get_parent()
            .filter(|n| n.is_element_node())
            .ok_or("cannot group the document root")?;
        let id = create::resolve_id(document, self.id.as_deref(), "g")?;
        reject_stylesheets(document)?;
        if !matches!(parent.get_name().as_str(), "g" | "svg") {
            return Err("grouping this parent requires structural preparation".into());
        }
        for target in &targets {
            if target.get_parent().as_ref() != Some(&parent) {
                return Err(
                    "grouping across different parents requires structural preparation".into(),
                );
            }
            let mut next = target.get_next_sibling();
            while let Some(tail) = next.filter(|n| {
                matches!(
                    n.get_type(),
                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                )
            }) {
                if !tail.get_content().trim().is_empty() {
                    return Err("grouping with mixed text tails requires preparation".into());
                }
                next = tail.get_next_sibling();
            }
        }
        let before = paint_order(root.clone());
        let mut group = create::svg_node("g", &parent, document)?;
        group
            .set_property("id", &id)
            .map_err(|_| "vector creation failed")?;
        targets[0]
            .add_prev_sibling(&mut group)
            .map_err(|_| "vector creation failed")?;
        for target in &mut targets {
            while let Some(mut tail) = target.get_next_sibling().filter(|n| {
                matches!(
                    n.get_type(),
                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                )
            }) {
                tail.unlink_node();
            }
            target.unlink_node();
            group
                .add_child(target)
                .map_err(|_| "vector grouping failed")?;
        }
        if before != paint_order(root) {
            return Err(
                "grouping changes global paint order; overlap preservation is ambiguous".into(),
            );
        }
        Ok((
            id.clone(),
            format!(
                "grouped {} object(s) into {}",
                targets.len(),
                style::python_repr(&id)
            ),
        ))
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        Ok(self.create(document)?.1)
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(args)?;
    let mut created = String::new();
    let mut result = transaction::apply_dom(
        registry,
        id,
        "group_objects",
        mutation.params.clone(),
        "medium",
        None,
        |document| {
            let (id, summary) = mutation.create(document)?;
            created = id;
            Ok(summary)
        },
    )?;
    result["object_id"] = json!(created);
    result["bbox"] = Value::Null;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsafe_parent_stylesheet_and_mixed_tails_refuse_before_dom_changes() {
        for source in [
            br#"<svg><g><rect id="a"/></g><g><rect id="b"/></g></svg>"#.as_slice(),
            br#"<svg><style>g{fill:red}</style><rect id="a"/><rect id="b"/></svg>"#,
            br#"<svg><rect id="a"/>meaningful<rect id="b"/></svg>"#,
        ] {
            let mut document = crate::xml::parse(source, 4096).unwrap();
            let before = document.to_string();
            assert!(
                Mutation::build(&json!({"object_ids":["a","b"],"object_id":"group"}))
                    .unwrap()
                    .mutate(&mut document)
                    .is_err()
            );
            assert_eq!(before, document.to_string());
        }
    }
    #[test]
    fn adjacent_neutral_group_preserves_pointer_paint_order() {
        let mut document=crate::xml::parse(br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="a"/><circle id="b"/><rect id="c"/></svg>"#,4096).unwrap();
        let before = paint_order(document.get_root_element().unwrap());
        Mutation::build(&json!({"object_ids":["a","b"],"object_id":"group"}))
            .unwrap()
            .mutate(&mut document)
            .unwrap();
        assert_eq!(before, paint_order(document.get_root_element().unwrap()));
        assert!(
            document
                .to_string()
                .contains("<g id=\"group\"><rect id=\"a\"/><circle id=\"b\"/></g><rect id=\"c\"/>")
        );
    }
}
