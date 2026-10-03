//! Eagerly validated palette/color mutations, staged by the shared reversible transaction.
use crate::{
    arguments,
    document::{Registry, elements},
    style, transaction,
};
use libxml::tree::Document;
use serde_json::{Value, json};
use std::{collections::HashSet, sync::LazyLock};
static COLORS: LazyLock<HashSet<String>> =
    LazyLock::new(|| serde_json::from_str(include_str!("palette_colors.json")).unwrap());
const PROPERTIES: [&str; 6] = [
    "fill",
    "stroke",
    "stop-color",
    "flood-color",
    "lighting-color",
    "color",
];
fn palette_color(raw: &str) -> Result<String, String> {
    let color = style::color(raw)?;
    if color.chars().all(char::is_alphabetic) && !COLORS.contains(&color) {
        return Err(format!("invalid colour value: {}", style::python_repr(raw)));
    }
    Ok(color)
}
pub struct Mutation {
    pub params: Value,
    pairs: Vec<(String, String)>,
    scopes: Option<Vec<String>>,
    palette: bool,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        let scopes = match args.get("scope_ids") {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                value
                    .as_array()
                    .ok_or("scope_ids must be a list")?
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .ok_or("scope_ids must contain strings")
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        };
        let palette = tool == "apply_palette";
        let mut pairs = Vec::new();
        let params = if palette {
            let mapping = args["mapping"]
                .as_object()
                .ok_or("mapping must be an object")?;
            if mapping.is_empty() {
                return Err("apply_palette requires a non-empty colour mapping".into());
            }
            if mapping.len() > 256 {
                return Err("palette exceeds 256 mappings".into());
            }
            for (from, to) in mapping {
                pairs.push((
                    palette_color(from)?,
                    palette_color(to.as_str().ok_or("palette values must be strings")?)?,
                ));
            }
            json!({"mapping":mapping,"scope_ids":scopes})
        } else {
            let from =
                arguments::string(args, "from_color")?.ok_or("from_color must be a string")?;
            let to = arguments::string(args, "to_color")?.ok_or("to_color must be a string")?;
            pairs.push((style::color(from)?, style::color(to)?));
            json!({"from_color":from,"to_color":to,"scope_ids":scopes})
        };
        Ok(Self {
            params,
            pairs,
            scopes,
            palette,
        })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let all = elements(root);
        let mut selected = Vec::new();
        if let Some(scopes) = &self.scopes {
            if scopes.is_empty() {
                return Err("no target object ids supplied".into());
            }
            let targets = scopes
                .iter()
                .map(|id| {
                    all.iter()
                        .find(|node| node.get_property_no_ns("id").as_ref() == Some(id))
                        .cloned()
                        .ok_or("object id not found in document")
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut seen = HashSet::new();
            for target in targets {
                for node in elements(target) {
                    if seen.insert(node.node_ptr() as usize) {
                        selected.push(node);
                    }
                }
            }
        } else {
            selected = all;
        }
        let mut count = 0;
        for (from, to) in &self.pairs {
            let from = style::color_key(from);
            for node in &mut selected {
                let decls = style::declarations(node);
                for prop in PROPERTIES {
                    if let Some(current) = decls.get(prop)
                        && style::color_key(current) == from
                    {
                        style::write_property(node, prop, to)?;
                        count += 1;
                    }
                }
                for prop in PROPERTIES {
                    if decls.contains_key(prop) {
                        continue;
                    }
                    if let Some(current) = node.get_property_no_ns(prop)
                        && style::color_key(&current) == from
                    {
                        node.set_property(prop, to)
                            .map_err(|_| "style edit failed")?;
                        count += 1;
                    }
                }
            }
        }
        let plural = if count == 1 { "object" } else { "objects" };
        Ok(if self.palette {
            format!(
                "applied palette ({} mappings, {count} {plural})",
                self.pairs.len()
            )
        } else {
            format!(
                "replaced {} with {} ({count} {plural})",
                self.pairs[0].0, self.pairs[0].1
            )
        })
    }
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(tool, args)?;
    transaction::apply_dom(
        registry,
        id,
        tool,
        mutation.params.clone(),
        "medium",
        None,
        |document| mutation.mutate(document),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_validation_and_scope_resolution_precede_any_staged_change() {
        assert!(Mutation::build("apply_palette", &json!({"mapping":{"red":"notacolor"}})).is_err());
        assert!(Mutation::build("apply_palette", &json!({"mapping":{"red":"url(#g)"}})).is_err());
        let bytes=br##"<svg><g id="scope"><rect id="r" fill="#abc" style="fill:#abc;stroke:#abc"/></g></svg>"##;
        let mut document = crate::xml::parse(bytes, 4096).unwrap();
        let before = crate::xml::serialize(&document);
        let invalid = Mutation::build(
            "replace_color",
            &json!({"from_color":"#abc","to_color":"red","scope_ids":["scope","absent"]}),
        )
        .unwrap();
        assert!(invalid.mutate(&mut document).is_err());
        assert_eq!(crate::xml::serialize(&document), before);
        let replacement = Mutation::build(
            "replace_color",
            &json!({"from_color":"#aabbcc","to_color":"red","scope_ids":["scope","r"]}),
        )
        .unwrap();
        assert_eq!(
            replacement.mutate(&mut document).unwrap(),
            "replaced #aabbcc with red (2 objects)"
        );
        let nodes = elements(document.get_root_element().unwrap());
        assert!(nodes[2].get_property_no_ns("fill").is_none());
        assert_eq!(
            nodes[2].get_property_no_ns("style").unwrap(),
            "fill:red;stroke:red"
        );
        assert!(palette_color("RebeccaPurple").is_ok());
    }
}
