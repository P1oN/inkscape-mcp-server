//! Literal text and validated font edits share the staged transaction kernel.
use crate::{
    css,
    document::{Registry, elements},
    fonts, inspect, style, transaction,
};
use libxml::tree::{Document, Node};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;
static FAMILY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^[A-Za-z0-9 ,._'"-]{1,128}$"#).unwrap());
static LENGTH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]*\.?[0-9]+(?:px|pt|pc|mm|cm|in|em|ex|rem|%)?$").unwrap());
pub struct Mutation {
    ids: Vec<String>,
    text: Option<String>,
    properties: Vec<(String, String)>,
    pub params: Value,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        if tool == "replace_text" {
            let id = args["object_id"]
                .as_str()
                .ok_or("object_id must be a string")?;
            let text = args["text"].as_str().ok_or("text must be a string")?;
            let length = text.chars().count();
            if length > 100_000 {
                return Err(format!("text too long: {length} > 100000 characters"));
            }
            if text
                .chars()
                .any(|c| matches!(c as u32, 0..=8 | 11..=12 | 14..=31 | 127..=159))
            {
                return Err("text contains forbidden control characters".into());
            }
            return Ok(Self {
                ids: vec![id.into()],
                text: Some(text.into()),
                properties: vec![],
                params: json!({"object_id":id,"text_length":length}),
            });
        }
        let ids = args["object_ids"]
            .as_array()
            .ok_or("object_ids must be a list")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or("object_ids must contain strings")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if ids.is_empty() {
            return Err("no target object ids supplied".into());
        }
        let mut params = json!({"object_ids":ids});
        let mut properties = Vec::new();
        for (key, prop) in [
            ("family", "font-family"),
            ("size", "font-size"),
            ("weight", "font-weight"),
        ] {
            let raw = crate::arguments::string(args, key)?;
            params[key] = json!(raw);
            if let Some(raw) = raw {
                let value = if key == "weight" {
                    raw.trim().to_lowercase()
                } else {
                    raw.trim().into()
                };
                let valid = match key {
                    "family" => FAMILY.is_match(&value),
                    "size" => LENGTH.is_match(&value),
                    _ => matches!(
                        value.as_str(),
                        "normal"
                            | "bold"
                            | "bolder"
                            | "lighter"
                            | "100"
                            | "200"
                            | "300"
                            | "400"
                            | "500"
                            | "600"
                            | "700"
                            | "800"
                            | "900"
                    ),
                };
                if !valid {
                    return Err(format!(
                        "invalid {} value: {}",
                        if key == "size" { "length" } else { prop },
                        style::python_repr(if key == "weight" { raw } else { &value })
                    ));
                }
                properties.push((prop.into(), value));
            }
        }
        if properties.is_empty() {
            return Err("set_font requires at least one of family / size / weight".into());
        }
        Ok(Self {
            ids,
            text: None,
            properties,
            params,
        })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let nodes = elements(
            document
                .get_root_element()
                .ok_or("document could not be parsed safely")?,
        );
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
        if let Some(text) = &self.text {
            let target = &mut targets[0];
            if !matches!(
                target.get_name().as_str(),
                "text" | "tspan" | "textPath" | "tref" | "flowRoot" | "flowPara" | "flowSpan"
            ) {
                return Err(format!(
                    "object {} is not a text element",
                    style::python_repr(&self.ids[0])
                ));
            }
            for mut child in target.get_child_nodes() {
                child.unlink_node();
            }
            // xmlNewDocText treats ampersands and markup literally; xmlNodeSetContent does not.
            let mut literal = Node::new_text(text, document).map_err(|_| "text edit failed")?;
            target
                .add_child(&mut literal)
                .map_err(|_| "text edit failed")?;
            return Ok(format!(
                "replaced text content of {}",
                style::python_repr(&self.ids[0])
            ));
        }
        for target in &mut targets {
            for (prop, value) in &self.properties {
                style::write_property(target, prop, value)?;
            }
        }
        Ok(format!(
            "set {} on {} object(s)",
            self.properties
                .iter()
                .map(|(p, v)| format!("{p}={v}"))
                .collect::<Vec<_>>()
                .join(", "),
            targets.len()
        ))
    }
}
fn coverage(registry: &Registry, id: &str, ids: &[String]) -> Result<Value, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let document = crate::xml::parse(&bytes, registry.workspace.max_input)?;
    let nodes = elements(
        document
            .get_root_element()
            .ok_or("document could not be parsed safely")?,
    );
    let rules = css::rules(&nodes);
    let mut details = Vec::new();
    let mut ok = true;
    for id in ids {
        let Some(target) = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_ref() == Some(id))
        else {
            continue;
        };
        let Some(families) = css::inherited(target, &rules, "font-family") else {
            continue;
        };
        let Some(family) = families
            .split(',')
            .map(|part| part.trim().trim_matches(['\'', '\"']).trim())
            .find(|part| !part.is_empty())
        else {
            continue;
        };
        let text = inspect::text(target);
        if family.is_empty() || text.trim().is_empty() {
            continue;
        }
        if let Some(missing) = fonts::uncovered(family, &text, registry.workspace.max_output) {
            let suggestion = if missing.is_empty() {
                None
            } else {
                ok = false;
                fonts::suggest(&missing, registry.workspace.max_output)
            };
            details.push(json!({"object_id":id,"family":family,"uncovered_chars":missing,"suggested_family":suggestion}));
        }
    }
    Ok(json!({"coverage_ok":ok,"font_coverage":details}))
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(tool, args)?;
    let mut result = transaction::apply_dom(
        registry,
        id,
        tool,
        mutation.params.clone(),
        "medium",
        None,
        |document| mutation.mutate(document),
    )?;
    if tool == "set_font" {
        let detail = coverage(registry, id, &mutation.ids)
            .unwrap_or(json!({"coverage_ok":true,"font_coverage":[]}));
        for key in ["coverage_ok", "font_coverage"] {
            result[key] = detail[key].clone();
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn literal_replacement_preserves_sibling_tail_and_empty_run() {
        let mut document = crate::xml::parse(
            br#"<svg><text id="t">before<tspan id="s">old</tspan>after</text></svg>"#,
            4096,
        )
        .unwrap();
        Mutation::build(
            "replace_text",
            &json!({"object_id":"s","text":"<&unknown;>"}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        assert!(
            document
                .to_string()
                .contains("before<tspan id=\"s\">&lt;&amp;unknown;&gt;</tspan>after")
        );
        Mutation::build("replace_text", &json!({"object_id":"t","text":""}))
            .unwrap()
            .mutate(&mut document)
            .unwrap();
        assert!(document.to_string().contains("<text id=\"t\"></text>"));
    }
}
