//! Ordinary groups and explicitly requested layers preserve existing editor metadata.
use crate::{
    arguments, create,
    document::{Registry, elements},
    identity, style, transaction,
};
use libxml::tree::Document;
use serde_json::{Value, json};
const INK: &str = "http://www.inkscape.org/namespaces/inkscape";
pub struct Mutation {
    create: bool,
    id: Option<String>,
    parent: Option<String>,
    label: Option<String>,
    mode: String,
    pub params: Value,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        let create = tool == "create_group";
        let id = arguments::string(args, "object_id")?.map(str::to_owned);
        if !create && id.is_none() {
            return Err("object_id must be a string".into());
        }
        let parent = arguments::string(args, "parent_id")?.map(str::to_owned);
        let label = arguments::string(args, "label")?.map(str::to_owned);
        let mode = arguments::string(args, "mode")?
            .unwrap_or(if create { "group" } else { "" })
            .to_owned();
        if !create && !matches!(mode.as_str(), "group" | "layer") {
            return Err("mode must be group or layer".into());
        }
        let params = if create {
            json!({"parent_id":parent,"object_id":id,"label":label,"mode":mode})
        } else {
            json!({"object_id":id,"mode":mode})
        };
        Ok(Self {
            create,
            id,
            parent,
            label,
            mode,
            params,
        })
    }
    fn mode(document: &Document, id: &str, mode: &str) -> Result<String, String> {
        if !matches!(mode, "group" | "layer") {
            return Err("mode must be group or layer".into());
        }
        let nodes = elements(
            document
                .get_root_element()
                .ok_or("document could not be parsed safely")?,
        );
        let mut target = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
            .cloned()
            .ok_or("object id not found in document")?;
        if target.get_name() != "g" {
            return Err("group/layer conversion requires a g container".into());
        }
        if (target.get_property_ns("groupmode", INK).as_deref() == Some("layer"))
            == (mode == "layer")
        {
            return Ok("container mode already matches".into());
        }
        crate::structure::reject_stylesheets(document)?;
        if mode == "layer" {
            identity::set_inkscape(&mut target, document, "groupmode", "layer")?;
        } else if target.get_property_ns("groupmode", INK).is_some() {
            target
                .remove_property_ns("groupmode", INK)
                .map_err(|_| "group mode edit failed")?;
        }
        Ok(format!("converted {} to {mode}", style::python_repr(id)))
    }
    pub fn mutate_created(&self, document: &mut Document) -> Result<(String, String), String> {
        if !self.create {
            let id = self.id.as_deref().ok_or("object_id must be a string")?;
            return Ok((id.into(), Self::mode(document, id, &self.mode)?));
        }
        let mut parent = create::resolve_parent(document, self.parent.as_deref())?;
        let id = create::resolve_id(document, self.id.as_deref(), "g")?;
        let mut group = create::svg_node("g", &parent, document)?;
        group
            .set_property("id", &id)
            .map_err(|_| "vector creation failed")?;
        parent
            .add_child(&mut group)
            .map_err(|_| "vector creation failed")?;
        if let Some(label) = &self.label {
            identity::Mutation::build(&json!({"object_id":id,"label":label}))?.mutate(document)?;
        }
        Self::mode(document, &id, &self.mode)?;
        Ok((
            id.clone(),
            format!("created <g> {}", style::python_repr(&id)),
        ))
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        Ok(self.mutate_created(document)?.1)
    }
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(tool, args)?;
    let mut created = String::new();
    let mut result = transaction::apply_dom(
        registry,
        id,
        tool,
        mutation.params.clone(),
        "medium",
        None,
        |document| {
            let (id, summary) = mutation.mutate_created(document)?;
            created = id;
            Ok(summary)
        },
    )?;
    if mutation.create {
        result["object_id"] = json!(created);
        result["bbox"] = Value::Null;
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stylesheet_guard_noop_and_metadata_preservation() {
        let mut document=crate::xml::parse(br#"<svg xmlns="http://www.w3.org/2000/svg"><style>g{fill:red}</style><g id="g" transform="translate(1)" style="opacity:0.5"><rect id="r"/></g></svg>"#,4096).unwrap();
        let before = document.to_string();
        assert!(
            Mutation::build("set_group_mode", &json!({"object_id":"g","mode":"layer"}))
                .unwrap()
                .mutate(&mut document)
                .is_err()
        );
        assert_eq!(before, document.to_string());
        assert_eq!(
            Mutation::build("set_group_mode", &json!({"object_id":"g","mode":"group"}))
                .unwrap()
                .mutate(&mut document)
                .unwrap(),
            "container mode already matches"
        );
        assert_eq!(before, document.to_string());
    }
}
