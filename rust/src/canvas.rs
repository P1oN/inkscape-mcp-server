//! Canvas mutations preserve child geometry and use the shared staged transaction.
use crate::{
    arguments,
    document::{Registry, elements},
    style, transaction, transform,
};
use libxml::tree::{Document, Namespace, Node, NodeType};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;
static LENGTH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]*\.?[0-9]+(?:px|pt|pc|mm|cm|in|em|ex|rem|%)?$").unwrap());
fn length(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if !LENGTH.is_match(value) {
        return Err(format!(
            "invalid length value: {}",
            style::python_repr(value)
        ));
    }
    Ok(value.into())
}
fn numeric(raw: Option<String>) -> Option<f64> {
    let raw = raw?;
    let mut text = raw.trim();
    if text.ends_with('%') {
        return None;
    }
    for unit in ["px", "pt", "pc", "mm", "cm", "in", "em", "ex", "rem"] {
        if let Some(value) = text.strip_suffix(unit) {
            text = value;
            break;
        }
    }
    text.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
}
pub(crate) fn viewbox(raw: Option<String>) -> Option<[f64; 4]> {
    let raw = raw?;
    let values = raw
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    values.try_into().ok()
}
fn valid(vb: &[f64; 4]) -> bool {
    vb.iter().all(|v| v.is_finite()) && vb[2] > 0.0 && vb[3] > 0.0
}
pub(crate) fn synthesized(width: Option<String>, height: Option<String>) -> Option<String> {
    Some(format!(
        "0 0 {} {}",
        transform::number(numeric(width)?).ok()?,
        transform::number(numeric(height)?).ok()?
    ))
}
pub struct Mutation {
    resize: bool,
    width: String,
    height: String,
    adjust: bool,
    bleed: Option<f64>,
    color: String,
    pub params: Value,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        if tool == "normalize_viewbox" {
            return Ok(Self {
                resize: false,
                width: String::new(),
                height: String::new(),
                adjust: false,
                bleed: None,
                color: String::new(),
                params: json!({}),
            });
        }
        let width = args["width"]
            .as_str()
            .ok_or("width must be a string")?
            .to_owned();
        let height = args["height"]
            .as_str()
            .ok_or("height must be a string")?
            .to_owned();
        let adjust = arguments::boolean(args, "adjust_viewbox", false)?;
        let bleed = arguments::optional_number(args, "bleed")?;
        let color = arguments::string(args, "bleed_color")?
            .unwrap_or("#ffffff")
            .to_owned();
        let params = json!({"width":width,"height":height,"adjust_viewbox":adjust,"bleed":bleed,"bleed_color":color});
        Ok(Self {
            resize: true,
            width,
            height,
            adjust,
            bleed,
            color,
            params,
        })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let mut root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        if !self.resize {
            let raw = root.get_property_no_ns("viewBox");
            if viewbox(raw.clone()).is_some_and(|vb| valid(&vb)) {
                return Ok("viewBox already normalized".into());
            }
            let value = synthesized(
                root.get_property_no_ns("width"),
                root.get_property_no_ns("height"),
            )
            .ok_or("cannot normalize viewBox: no valid width/height to derive it from")?;
            root.set_property("viewBox", &value)
                .map_err(|_| "canvas edit failed")?;
            return Ok(if raw.is_none() {
                format!(
                    "synthesized viewBox {} from width/height",
                    style::python_repr(&value)
                )
            } else {
                format!(
                    "repaired malformed viewBox to {}",
                    style::python_repr(&value)
                )
            });
        }
        let width = length(&self.width)?;
        let height = length(&self.height)?;
        if self.bleed.is_some() && self.adjust {
            return Err("resize_canvas bleed cannot be combined with adjust_viewbox".into());
        }
        if let Some(bleed) = self.bleed {
            if !bleed.is_finite() {
                return Err("bleed must be finite".into());
            }
            if bleed < 0.0 {
                return Err("resize_canvas bleed must be 0 or greater".into());
            }
        }
        let bleed = self.bleed.unwrap_or(0.0);
        let color = if bleed != 0.0 {
            style::color(&self.color)?
        } else {
            self.color.clone()
        };
        root.set_property("width", &width)
            .map_err(|_| "canvas edit failed")?;
        root.set_property("height", &height)
            .map_err(|_| "canvas edit failed")?;
        let base = format!("resized canvas to {width} x {height}");
        if bleed != 0.0 {
            let vb = viewbox(root.get_property_no_ns("viewBox"))
                .filter(valid)
                .or_else(|| viewbox(synthesized(Some(width.clone()), Some(height.clone()))))
                .ok_or(
                    "resize_canvas bleed needs a valid viewBox (or numeric width/height) to extend",
                )?;
            let extended = [
                vb[0] - bleed,
                vb[1] - bleed,
                vb[2] + 2.0 * bleed,
                vb[3] + 2.0 * bleed,
            ];
            let values = extended
                .into_iter()
                .map(transform::number)
                .collect::<Result<Vec<_>, _>>()?;
            let nodes = elements(root.clone());
            let mut index = 0usize;
            let id = loop {
                let candidate = if index == 0 {
                    "bleed-bg".into()
                } else {
                    format!("bleed-bg-{index}")
                };
                if !nodes
                    .iter()
                    .any(|n| n.get_property_no_ns("id").as_ref() == Some(&candidate))
                {
                    break candidate;
                }
                index += 1;
            };
            let namespace = root
                .get_namespace()
                .filter(|ns| ns.get_href() == "http://www.w3.org/2000/svg");
            let mut rect =
                Node::new("rect", namespace, document).map_err(|_| "canvas edit failed")?;
            if rect.get_namespace().is_none() {
                let ns = Namespace::new("ns0", "http://www.w3.org/2000/svg", &mut rect)
                    .map_err(|_| "canvas edit failed")?;
                rect.set_namespace(&ns).map_err(|_| "canvas edit failed")?;
            }
            for (key, value) in [
                ("id", id.as_str()),
                ("x", &values[0]),
                ("y", &values[1]),
                ("width", &values[2]),
                ("height", &values[3]),
                ("fill", &color),
            ] {
                rect.set_property(key, value)
                    .map_err(|_| "canvas edit failed")?;
            }
            // lxml insertion respects leading text and the preceding element's tail.
            let children = root.get_child_nodes();
            let start = children
                .iter()
                .position(|n| n.is_element_node() && n.get_name() == "defs")
                .map(|i| i + 1)
                .unwrap_or(0);
            if let Some(mut next) = children.into_iter().skip(start).find(|n| {
                !matches!(
                    n.get_type(),
                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                )
            }) {
                next.add_prev_sibling(&mut rect)
                    .map_err(|_| "canvas edit failed")?;
            } else {
                root.add_child(&mut rect)
                    .map_err(|_| "canvas edit failed")?;
            }
            root.set_property("viewBox", &values.join(" "))
                .map_err(|_| "canvas edit failed")?;
            return Ok(format!(
                "{base} and added a {}-unit bleed border painted {color} (background {})",
                transform::number(bleed)?,
                style::python_repr(&id)
            ));
        }
        let value = synthesized(Some(width), Some(height));
        if self.adjust {
            if let Some(value) = value {
                root.set_property("viewBox", &value)
                    .map_err(|_| "canvas edit failed")?;
                return Ok(format!(
                    "{base} and adjusted viewBox to {}",
                    style::python_repr(&value)
                ));
            }
            return Ok(format!(
                "{base} (viewBox not adjusted: width/height are not numeric)"
            ));
        }
        if viewbox(root.get_property_no_ns("viewBox")).is_none()
            && let Some(value) = value
        {
            root.set_property("viewBox", &value)
                .map_err(|_| "canvas edit failed")?;
            return Ok(format!(
                "{base} and synthesized viewBox {}",
                style::python_repr(&value)
            ));
        }
        Ok(base)
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
    fn repair_preserves_valid_tokens_and_bleed_preserves_geometry_order() {
        let mut document=crate::xml::parse(br#"<svg xmlns="http://www.w3.org/2000/svg" width="100px" height="50mm" viewBox="0,0,100,50"><defs/>tail<rect id="bleed-bg" x="1"/><text>A<tspan>B</tspan>C</text></svg>"#,4096).unwrap();
        let before = document.to_string();
        Mutation::build("normalize_viewbox", &json!({}))
            .unwrap()
            .mutate(&mut document)
            .unwrap();
        assert_eq!(before, document.to_string());
        Mutation::build(
            "resize_canvas",
            &json!({"width":"100px","height":"50mm","bleed":2}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        let after = document.to_string();
        assert!(after.contains("<defs/>tail<rect id=\"bleed-bg-1\""));
        assert!(after.contains("viewBox=\"-2 -2 104 54\""));
        assert!(after.contains("A<tspan>B</tspan>C"));
        assert_eq!(numeric(Some("1rem".into())), None);
    }
}
