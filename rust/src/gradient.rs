//! Typed editable paint servers with eager bounded stop validation.
use crate::{arguments, create, document::Registry, style, transaction, transform};
use indexmap::IndexMap;
use libxml::tree::{Document, NodeType};
use serde_json::{Value, json};
fn coordinate(raw: &str, key: &str) -> Result<String, String> {
    let value = raw.trim();
    let percent = value.ends_with('%');
    let body = value.strip_suffix('%').unwrap_or(value);
    let number = body
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("invalid {key} value: {}", style::python_repr(raw)))?;
    let number = transform::number(number).map_err(|error| {
        if percent {
            error
        } else {
            format!("invalid {key} value: {}", style::python_repr(raw))
        }
    })?;
    Ok(if percent {
        format!("{number}%")
    } else {
        number
    })
}
fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        other => other.to_string(),
    }
}
fn offset(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    let percent = trimmed.ends_with('%');
    let body = trimmed.strip_suffix('%').unwrap_or(trimmed);
    let number = body
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("invalid stop offset: {}", style::python_repr(raw)))?;
    if !(0.0..=if percent { 100.0 } else { 1.0 }).contains(&number) {
        return Err(if percent {
            "stop offset percentage must be between 0% and 100%"
        } else {
            "stop offset must be between 0 and 1"
        }
        .into());
    }
    let number = transform::number(number)?;
    Ok(if percent {
        format!("{number}%")
    } else {
        number
    })
}
pub struct Mutation {
    local: &'static str,
    id: Option<String>,
    coordinates: IndexMap<String, String>,
    stops: Vec<(String, String)>,
    pub params: Value,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        let linear = tool == "add_linear_gradient";
        let local = if linear {
            "linearGradient"
        } else {
            "radialGradient"
        };
        let keys: &[(&str, &str)] = if linear {
            &[("x1", "0%"), ("y1", "0%"), ("x2", "100%"), ("y2", "0%")]
        } else {
            &[("cx", "50%"), ("cy", "50%"), ("r", "50%")]
        };
        let mut coordinates = IndexMap::new();
        let mut params = json!({});
        for (key, default) in keys {
            let raw = arguments::string(args, key)?.unwrap_or(default);
            coordinates.insert((*key).into(), coordinate(raw, key)?);
            params[*key] = json!(raw);
        }
        if !linear {
            for key in ["fx", "fy"] {
                if let Some(raw) = arguments::string(args, key)? {
                    coordinates.insert(key.into(), coordinate(raw, key)?);
                }
            }
        }
        let raw = args["stops"].as_array().ok_or("stops must be a list")?;
        params["stop_count"] = json!(raw.len());
        if raw.is_empty() {
            return Err("gradient requires at least one stop".into());
        }
        if raw.len() > 1000 {
            return Err(format!("too many stops: {} > 1000", raw.len()));
        }
        let mut stops = Vec::new();
        for stop in raw {
            let object = stop.as_object().ok_or("gradient stops must be objects")?;
            let o = object
                .get("offset")
                .filter(|v| !v.is_null())
                .ok_or("gradient stop missing 'offset'")?;
            let c = object
                .get("color")
                .filter(|v| !v.is_null())
                .ok_or("gradient stop missing 'color'")?;
            let offset = offset(&text(o))?;
            let color = style::color(&text(c))?;
            let mut style = format!("stop-color:{color}");
            if let Some(opacity) = object.get("opacity").filter(|v| !v.is_null()) {
                let value = arguments::number(opacity).ok_or("stop opacity must be a number")?;
                if !(0.0..=1.0).contains(&value) {
                    return Err("stop opacity must be between 0 and 1".into());
                }
                style.push_str(&format!(";stop-opacity:{}", transform::number(value)?));
            }
            stops.push((offset, style));
        }
        Ok(Self {
            local,
            id: arguments::string(args, "object_id")?.map(str::to_owned),
            coordinates,
            stops,
            params,
        })
    }
    pub fn create(&self, document: &mut Document) -> Result<(String, String), String> {
        let id = create::resolve_id(
            document,
            self.id.as_deref(),
            if self.local == "linearGradient" {
                "lg"
            } else {
                "rg"
            },
        )?;
        let mut root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let mut defs = if let Some(defs) = root
            .get_child_elements()
            .into_iter()
            .find(|n| n.get_name() == "defs")
        {
            defs
        } else {
            let mut defs = create::svg_node("defs", &root, document)?;
            if let Some(mut first) = root.get_child_nodes().into_iter().find(|n| {
                !matches!(
                    n.get_type(),
                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                )
            }) {
                first
                    .add_prev_sibling(&mut defs)
                    .map_err(|_| "vector creation failed")?;
            } else {
                root.add_child(&mut defs)
                    .map_err(|_| "vector creation failed")?;
            }
            defs
        };
        let mut gradient = create::svg_node(self.local, &defs, document)?;
        gradient
            .set_property("id", &id)
            .map_err(|_| "vector creation failed")?;
        for (key, value) in &self.coordinates {
            gradient
                .set_property(key, value)
                .map_err(|_| "vector creation failed")?;
        }
        for (offset, style) in &self.stops {
            let mut stop = create::svg_node("stop", &gradient, document)?;
            stop.set_property("offset", offset)
                .map_err(|_| "vector creation failed")?;
            stop.set_property("style", style)
                .map_err(|_| "vector creation failed")?;
            gradient
                .add_child(&mut stop)
                .map_err(|_| "vector creation failed")?;
        }
        defs.add_child(&mut gradient)
            .map_err(|_| "vector creation failed")?;
        Ok((
            id.clone(),
            format!("created <{}> {}", self.local, style::python_repr(&id)),
        ))
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        Ok(self.create(document)?.1)
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
    fn defs_precede_comment_preserving_leading_text_and_stop_namespace() {
        let mut document = crate::xml::parse(
            br#"<svg xmlns="http://www.w3.org/2000/svg">lead<!--keep--><g id="g"/></svg>"#,
            4096,
        )
        .unwrap();
        Mutation::build(
            "add_linear_gradient",
            &json!({"stops":[{"offset":0,"color":"red"}],"object_id":"paint"}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        let serialized = document.to_string();
        assert!(serialized.contains("lead<defs><linearGradient"));
        assert!(serialized.contains(
            "<stop offset=\"0\" style=\"stop-color:red\"/></linearGradient></defs><!--keep-->"
        ));
        let before = document.to_string();
        assert!(
            Mutation::build(
                "add_radial_gradient",
                &json!({"stops":[{"offset":0,"color":"red"}],"object_id":"g"})
            )
            .unwrap()
            .mutate(&mut document)
            .is_err()
        );
        assert_eq!(before, document.to_string());
    }
    #[test]
    fn coordinates_stops_and_bounds_validate_before_mutation() {
        assert_eq!(coordinate(" -2.5e1% ", "x1").unwrap(), "-25%");
        assert_eq!(offset("0.25").unwrap(), "0.25");
        for stops in [
            json!([]),
            json!([{"color":"red"}]),
            json!([{"offset":2,"color":"red"}]),
            json!([{"offset":0,"color":"red","opacity":"NaN"}]),
        ] {
            assert!(Mutation::build("add_linear_gradient", &json!({"stops":stops})).is_err());
        }
    }
}
