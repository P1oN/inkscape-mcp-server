//! Bounded numeric transforms prepend in parent space without baking geometry.
use crate::{
    arguments,
    document::{Registry, elements},
    style, transaction,
};
use libxml::tree::{Document, Node};
use serde_json::{Value, json};
pub fn number(value: f64) -> Result<String, String> {
    if !value.is_finite() {
        return Err("number must be finite".into());
    }
    let text = format!("{value:.6}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    Ok(if text.is_empty() || text == "-0" {
        "0".into()
    } else {
        text.into()
    })
}
fn finite(key: &str, value: f64) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(format!("{key} must be finite"))
    }
}
fn target(document: &Document, id: &str) -> Result<Node, String> {
    elements(
        document
            .get_root_element()
            .ok_or("document could not be parsed safely")?,
    )
    .into_iter()
    .find(|node| node.get_property_no_ns("id").as_deref() == Some(id))
    .ok_or_else(|| "object id not found in document".into())
}
pub struct Mutation {
    id: String,
    tool: String,
    values: Vec<Option<f64>>,
    pub params: Value,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        let id = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?
            .to_owned();
        let mut params = json!({"object_id":id});
        let keys: &[(&str, bool)] = match tool {
            "move_object" => &[("dx", true), ("dy", true)],
            "scale_object" => &[("sx", true), ("sy", false)],
            _ => &[("degrees", true), ("cx", false), ("cy", false)],
        };
        let mut values = Vec::new();
        for (key, required) in keys {
            let value = arguments::optional_number(args, key)?;
            if *required && value.is_none() {
                return Err(format!("{key} must be a number"));
            }
            params[*key] = json!(value);
            values.push(value);
        }
        Ok(Self {
            id,
            tool: tool.into(),
            values,
            params,
        })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let a = self.values[0].ok_or("missing numeric value")?;
        let (transform, summary) = match self.tool.as_str() {
            "move_object" => {
                let b = self.values[1].ok_or("missing numeric value")?;
                finite("dx", a)?;
                finite("dy", b)?;
                let a = number(a)?;
                let b = number(b)?;
                (
                    format!("translate({a},{b})"),
                    format!("translated {} by ({a},{b})", style::python_repr(&self.id)),
                )
            }
            "scale_object" => {
                let b = self.values[1].unwrap_or(a);
                finite("sx", a)?;
                finite("sy", b)?;
                if a <= 0.0 || b <= 0.0 {
                    return Err("scale factors must be positive".into());
                }
                let a = number(a)?;
                let b = number(b)?;
                (
                    format!("scale({a},{b})"),
                    format!(
                        "scaled {} by ({a},{b}) about the origin",
                        style::python_repr(&self.id)
                    ),
                )
            }
            _ => {
                finite("degrees", a)?;
                if self.values[1].is_some() != self.values[2].is_some() {
                    return Err("rotation centre requires both cx and cy".into());
                }
                target(document, &self.id)?;
                let a = number(a)?;
                if let (Some(x), Some(y)) = (self.values[1], self.values[2]) {
                    finite("cx", x)?;
                    finite("cy", y)?;
                    let x = number(x)?;
                    let y = number(y)?;
                    (
                        format!("rotate({a},{x},{y})"),
                        format!(
                            "rotated {} by {a}deg about ({x},{y})",
                            style::python_repr(&self.id)
                        ),
                    )
                } else {
                    (
                        format!("rotate({a})"),
                        format!(
                            "rotated {} by {a}deg about the origin",
                            style::python_repr(&self.id)
                        ),
                    )
                }
            }
        };
        let mut target = target(document, &self.id)?;
        let existing = target.get_property_no_ns("transform").unwrap_or_default();
        let combined = if existing.is_empty() {
            transform
        } else {
            format!("{transform} {existing}").trim().to_owned()
        };
        target
            .set_property("transform", &combined)
            .map_err(|_| "transform edit failed")?;
        Ok(summary)
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
    fn parent_space_order_rounding_and_preserved_ids() {
        let mut document=crate::xml::parse(br##"<svg><g transform="scale(2)"><rect id="r" transform=" rotate(4) "/></g><use href="#r"/></svg>"##,4096).unwrap();
        Mutation::build(
            "move_object",
            &json!({"object_id":"r","dx":-0.0000001,"dy":1.2345678}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        let serialized = document.to_string();
        assert!(serialized.contains("translate(0,1.234568)  rotate(4)"));
        assert!(serialized.contains("href=\"#r\""));
        for args in [
            json!({"object_id":"r","sx":0}),
            json!({"object_id":"r","sx":-1}),
            json!({"object_id":"r","sx":"inf"}),
        ] {
            assert!(
                Mutation::build("scale_object", &args)
                    .unwrap()
                    .mutate(&mut document)
                    .is_err()
            );
            assert_eq!(serialized, document.to_string());
        }
    }
}
