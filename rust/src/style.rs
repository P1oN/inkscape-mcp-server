//! Native style kernel; validation is eager and mutation stays in the shared transaction.

use crate::{
    document::{Registry, elements},
    transaction,
};
use indexmap::IndexMap;
use libxml::tree::{Document, Node};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;

static COLOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:#[0-9a-fA-F]{3,4}|#[0-9a-fA-F]{6}|#[0-9a-fA-F]{8}|[A-Za-z]{1,32}|(?:rgb|rgba|hsl|hsla)\([0-9.,%/ \t]+\))$").unwrap()
});
static PAINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^url\(\s*#([A-Za-z_][A-Za-z0-9_.:-]*)\s*\)(?:\s+(\S.*))?$").unwrap()
});
static LENGTH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]*\.?[0-9]+(?:px|pt|pc|mm|cm|in|em|ex|rem|%)?$").unwrap());
static STYLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*([^:;]+?)\s*:\s*([^;]+?)\s*(?:;|$)").unwrap());

pub fn python_repr(value: &str) -> String {
    let quote = if value.contains('\'') && !value.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut result = String::from(quote);
    for character in value.chars() {
        match character {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            character if character == quote => {
                result.push('\\');
                result.push(character);
            }
            character if character.is_control() => {
                result.push_str(&format!("\\x{:02x}", character as u32))
            }
            character => result.push(character),
        }
    }
    result.push(quote);
    result
}

pub fn color(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("colour value is empty".into());
    }
    if !COLOR.is_match(value) {
        return Err(format!("invalid colour value: {}", python_repr(value)));
    }
    let lower = value.to_lowercase();
    Ok(if lower.contains('(') {
        lower.chars().filter(|c| !c.is_whitespace()).collect()
    } else {
        lower
    })
}

pub fn paint(raw: &str) -> Result<String, String> {
    if let Some(capture) = PAINT.captures(raw.trim()) {
        let reference = format!("url(#{})", &capture[1]);
        return match capture.get(2) {
            Some(fallback) => Ok(format!("{reference} {}", color(fallback.as_str())?)),
            None => Ok(reference),
        };
    }
    color(raw)
}

pub fn color_key(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if let Some(hex) = lower.strip_prefix('#')
        && matches!(hex.len(), 3 | 4)
        && hex.chars().all(|c| c.is_ascii_hexdigit())
    {
        return format!("#{}", hex.chars().flat_map(|c| [c, c]).collect::<String>());
    }
    if ["rgb(", "rgba(", "hsl(", "hsla("]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
    {
        lower.chars().filter(|c| !c.is_whitespace()).collect()
    } else {
        lower
    }
}

pub fn length(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if !LENGTH.is_match(value) {
        return Err(format!("invalid length value: {}", python_repr(value)));
    }
    Ok(value.into())
}
pub fn format_num(value: f64) -> Result<String, String> {
    if !value.is_finite() {
        return Err("number must be finite".into());
    }
    let formatted = format!("{value:.6}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    Ok(if trimmed.is_empty() || trimmed == "-0" {
        "0".into()
    } else {
        trimmed.to_owned()
    })
}

fn opacity(value: f64) -> Result<String, String> {
    if !(0.0..=1.0).contains(&value) {
        return Err("opacity must be between 0 and 1".into());
    }
    format_num(value)
}

pub fn declarations(node: &Node) -> IndexMap<String, String> {
    let mut declarations = IndexMap::new();
    for capture in STYLE.captures_iter(&node.get_property_no_ns("style").unwrap_or_default()) {
        declarations.insert(capture[1].trim().to_lowercase(), capture[2].trim().into());
    }
    declarations
}

fn set_property(
    node: &mut Node,
    property: &str,
    value: &str,
    compare_color: bool,
) -> Result<(), String> {
    let properties = declarations(node);
    let current = properties
        .get(property)
        .cloned()
        .or_else(|| node.get_property_no_ns(property));
    if current.as_ref().is_some_and(|current| {
        if compare_color {
            color_key(current) == color_key(value)
        } else {
            current == value
        }
    }) {
        return Ok(());
    }
    write_property(node, property, value)
}

pub(crate) fn write_property(node: &mut Node, property: &str, value: &str) -> Result<(), String> {
    let mut properties = declarations(node);
    properties.insert(property.into(), value.into());
    let serialized = properties
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join(";");
    node.set_property("style", &serialized)
        .map_err(|_| "style edit failed")?;
    if node.get_property_no_ns(property).is_some() {
        node.remove_property_no_ns(property)
            .map_err(|_| "style edit failed")?;
    }
    Ok(())
}

pub struct Mutation {
    ids: Vec<String>,
    properties: Vec<(String, String, bool)>,
    summary: String,
    pub params: Value,
}

impl Mutation {
    pub fn build(tool: &str, arguments: &Value) -> Result<Self, String> {
        let ids: Vec<String> = arguments["object_ids"]
            .as_array()
            .ok_or("object_ids must be a list")?
            .iter()
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .ok_or("object_ids must contain strings".to_string())
            })
            .collect::<Result<_, _>>()?;
        let mut properties = Vec::new();
        let mut parts = Vec::new();
        let mut params = json!({"object_ids":ids});
        if tool == "set_fill" || tool == "set_stroke" {
            if let Some(raw) = crate::arguments::string(arguments, "color")? {
                let value = paint(raw)?;
                let compare = !value.starts_with("url(");
                properties.push((
                    if tool == "set_fill" { "fill" } else { "stroke" }.into(),
                    value,
                    compare,
                ));
                parts.push("color");
                params["color"] = json!(raw);
            } else if tool == "set_fill" {
                return Err("color must be a string".into());
            } else {
                params["color"] = Value::Null;
            }
            if tool == "set_stroke" {
                params["width"] = arguments.get("width").cloned().unwrap_or(Value::Null);
                if let Some(raw) = crate::arguments::string(arguments, "width")? {
                    let value = length(raw)?;
                    properties.push(("stroke-width".into(), value, false));
                    parts.push("width");
                }
            }
        }
        params["opacity"] = arguments.get("opacity").cloned().unwrap_or(Value::Null);
        if let Some(value) = crate::arguments::optional_number(arguments, "opacity")? {
            params["opacity"] = json!(value);
            properties.push((
                match tool {
                    "set_fill" => "fill-opacity",
                    "set_stroke" => "stroke-opacity",
                    _ => "opacity",
                }
                .into(),
                opacity(value)?,
                false,
            ));
            parts.push("opacity");
        } else if tool == "set_opacity" {
            return Err("opacity must be a number between 0 and 1".into());
        }
        if tool == "set_stroke" && parts.is_empty() {
            return Err("set_stroke requires at least one of color, width, opacity".into());
        }
        let count = ids.len();
        let plural = if count == 1 { "object" } else { "objects" };
        let summary = if tool == "set_stroke" {
            format!("set stroke {} on {count} {plural}", parts.join("/"))
        } else {
            format!(
                "set {} on {count} {plural}",
                if tool == "set_fill" {
                    "fill"
                } else {
                    "opacity"
                }
            )
        };
        Ok(Self {
            ids,
            properties,
            summary,
            params,
        })
    }

    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        if self.ids.is_empty() {
            return Err("no target object ids supplied".into());
        }
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let nodes = elements(root);
        let mut targets = self
            .ids
            .iter()
            .map(|id| {
                nodes
                    .iter()
                    .find(|n| n.get_property_no_ns("id").as_ref() == Some(id))
                    .cloned()
                    .ok_or("object id not found in document".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        for target in &mut targets {
            for (name, value, compare) in &self.properties {
                set_property(target, name, value, *compare)?;
            }
        }
        Ok(self.summary.clone())
    }
}

pub fn apply(registry: &Registry, tool: &str, arguments: &Value) -> Result<Value, String> {
    let id = arguments["doc_id"]
        .as_str()
        .ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(tool, arguments)?;
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
    fn foreign_paint_and_style_attributes_do_not_affect_edits_or_noops() {
        for attrs in [
            r#"q:fill="red""#,
            r#"q:style="fill:red""#,
            r#"q:fill="red" fill="black" q:style="fill:green""#,
        ] {
            let source = format!(r#"<svg xmlns:q="urn:example"><rect id="r" {attrs}/></svg>"#);
            let mut document = crate::xml::parse(source.as_bytes(), 4096).unwrap();
            let before = crate::xml::serialize(&document);
            let foreign =
                document.get_root_element().unwrap().get_child_elements()[0].get_properties_ns();
            let mutation =
                Mutation::build("set_fill", &json!({"object_ids":["r"],"color":"red"})).unwrap();
            mutation.mutate(&mut document).unwrap();
            assert_ne!(crate::xml::serialize(&document), before);
            let target = &document.get_root_element().unwrap().get_child_elements()[0];
            assert_eq!(
                target.get_property_no_ns("style").as_deref(),
                Some("fill:red")
            );
            assert!(target.get_property_no_ns("fill").is_none());
            for ((key, ns), value) in foreign {
                if let Some(ns) = ns {
                    assert_eq!(target.get_property_ns(&key, &ns.get_href()), Some(value));
                }
            }
            let edited = crate::xml::serialize(&document);
            mutation.mutate(&mut document).unwrap();
            assert_eq!(crate::xml::serialize(&document), edited);
        }
    }

    #[test]
    fn foreign_ids_never_select_or_shadow_an_svg_target() {
        let mut document = crate::xml::parse(
            br#"<svg xmlns:q="urn:example"><rect q:id="r" id="s"/><rect id="r"/><rect q:id="foreign"/></svg>"#,
            4096,
        )
        .unwrap();
        Mutation::build("set_fill", &json!({"object_ids":["r"],"color":"red"}))
            .unwrap()
            .mutate(&mut document)
            .unwrap();
        let children = document.get_root_element().unwrap().get_child_elements();
        assert!(children[0].get_property_no_ns("style").is_none());
        assert_eq!(
            children[1].get_property_no_ns("style").as_deref(),
            Some("fill:red")
        );
        let before = crate::xml::serialize(&document);
        assert_eq!(
            Mutation::build(
                "set_fill",
                &json!({"object_ids":["foreign"],"color":"blue"})
            )
            .unwrap()
            .mutate(&mut document)
            .unwrap_err(),
            "object id not found in document"
        );
        assert_eq!(crate::xml::serialize(&document), before);
    }

    #[test]
    fn invalid_optional_types_cannot_be_ignored_by_a_partial_edit() {
        for arguments in [
            json!({"object_ids":["r"],"color":123,"width":"2"}),
            json!({"object_ids":["r"],"color":"red","width":2}),
            json!({"object_ids":["r"],"color":"red","opacity":"wrong"}),
        ] {
            assert!(Mutation::build("set_stroke", &arguments).is_err());
        }
        let mutation = Mutation::build(
            "set_fill",
            &json!({"object_ids":["r"],"color":"red","opacity":"0.5"}),
        )
        .unwrap();
        assert_eq!(mutation.params["opacity"], 0.5);
    }

    #[test]
    fn css_injection_external_paint_and_case_sensitive_references() {
        for value in [
            "red;display:none",
            "url(http://example.com/a)",
            "javascript:alert(1)",
            "rgb(1,\n2,3)",
        ] {
            assert!(paint(value).is_err());
        }
        assert_eq!(paint("url( #Gradient ) RED").unwrap(), "url(#Gradient) red");
        assert_eq!(color_key("#aBc"), "#aabbcc");
        let mut document = crate::xml::parse(
            br##"<svg><rect id="r" fill="#ABC"/><rect id="s" fill="url(#Gradient)"/></svg>"##,
            4096,
        )
        .unwrap();
        let first = document.to_string();
        Mutation::build("set_fill", &json!({"object_ids":["r"],"color":"#aabbcc"}))
            .unwrap()
            .mutate(&mut document)
            .unwrap();
        assert_eq!(document.to_string(), first);
        Mutation::build(
            "set_fill",
            &json!({"object_ids":["s"],"color":"url(#gradient)"}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        assert!(document.to_string().contains("url(#gradient)"));
    }
}
