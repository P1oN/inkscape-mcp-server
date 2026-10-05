//! Explicit vector primitives; no generic tag/attribute or raster conversion surface.
use crate::{
    arguments,
    document::{Registry, elements},
    style, transaction, transform,
};
use indexmap::IndexMap;
use libxml::tree::{Document, Namespace, Node};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;
const SVG: &str = "http://www.w3.org/2000/svg";
static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_.:-]*$").unwrap());
static LENGTH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]*\.?[0-9]+(?:px|pt|pc|mm|cm|in|em|ex|rem|%)?$").unwrap());
static PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\d\s,.+\-eEMmLlHhVvCcSsQqTtAaZz]+$").unwrap());
pub(crate) fn numeric(value: f64, key: &str, limit: u8) -> Result<String, String> {
    if limit == 1 && value <= 0.0 {
        return Err(format!("{key} must be greater than 0"));
    }
    if limit == 2 && value < 0.0 {
        return Err(format!("{key} must be 0 or greater"));
    }
    transform::number(value).map_err(|_| {
        format!(
            "invalid {key} value: {}",
            if value.is_nan() {
                "nan"
            } else if value.is_sign_negative() {
                "-inf"
            } else {
                "inf"
            }
        )
    })
}
fn bbox(points: &[[f64; 2]]) -> Value {
    let x = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let y = points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let maxx = points
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    let maxy = points
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max);
    json!({"x":x,"y":y,"width":maxx-x,"height":maxy-y})
}
pub(crate) fn valid_id(id: &str) -> bool {
    ID.is_match(id)
}
pub(crate) fn resolve_parent(document: &Document, id: Option<&str>) -> Result<Node, String> {
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    if let Some(id) = id {
        return elements(root)
            .into_iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
            .ok_or_else(|| "object id not found in document".into());
    }
    Ok(root
        .get_child_elements()
        .into_iter()
        .find(|n| {
            n.get_name() == "g"
                && n.get_property_ns("groupmode", "http://www.inkscape.org/namespaces/inkscape")
                    .as_deref()
                    == Some("layer")
        })
        .unwrap_or(root))
}
pub(crate) fn resolve_id(
    document: &Document,
    supplied: Option<&str>,
    prefix: &str,
) -> Result<String, String> {
    let nodes = elements(
        document
            .get_root_element()
            .ok_or("document could not be parsed safely")?,
    );
    if let Some(id) = supplied {
        if !ID.is_match(id) {
            return Err(format!("invalid object id: {}", style::python_repr(id)));
        }
        if nodes
            .iter()
            .any(|n| n.get_property_no_ns("id").as_deref() == Some(id))
        {
            return Err(format!("id already in use: {}", style::python_repr(id)));
        }
        return Ok(id.into());
    }
    loop {
        let candidate = format!(
            "{prefix}-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        );
        if !nodes
            .iter()
            .any(|n| n.get_property_no_ns("id").as_deref() == Some(&candidate))
        {
            return Ok(candidate);
        }
    }
}
pub(crate) fn svg_node(local: &str, parent: &Node, document: &Document) -> Result<Node, String> {
    let namespace = parent
        .get_namespace()
        .filter(|ns| ns.get_href() == SVG)
        .or_else(|| {
            parent
                .get_namespaces(document)
                .into_iter()
                .find(|ns| ns.get_href() == SVG)
        });
    let mut node = Node::new(local, namespace, document).map_err(|_| "vector creation failed")?;
    if node.get_namespace().is_none() {
        let ns = Namespace::new("ns0", SVG, &mut node).map_err(|_| "vector creation failed")?;
        node.set_namespace(&ns)
            .map_err(|_| "vector creation failed")?;
    }
    Ok(node)
}
pub struct Mutation {
    local: String,
    attrs: IndexMap<String, String>,
    parent: Option<String>,
    id: Option<String>,
    text: Option<String>,
    paint: Vec<(String, String)>,
    pub params: Value,
    pub bbox: Value,
}
impl Mutation {
    pub fn build(tool: &str, args: &Value) -> Result<Self, String> {
        let local = tool.strip_prefix("create_").ok_or("unknown primitive")?;
        let fields: &[(&str, u8)] = match local {
            "rect" => &[("x", 0), ("y", 0), ("width", 1), ("height", 1)],
            "circle" => &[("cx", 0), ("cy", 0), ("r", 1)],
            "ellipse" => &[("cx", 0), ("cy", 0), ("rx", 1), ("ry", 1)],
            "line" => &[("x1", 0), ("y1", 0), ("x2", 0), ("y2", 0)],
            "text" => &[("x", 0), ("y", 0)],
            "polygon" | "polyline" | "path" => &[],
            _ => return Err("unknown primitive".into()),
        };
        let parent = arguments::string(args, "parent_id")?.map(str::to_owned);
        let id = arguments::string(args, "object_id")?.map(str::to_owned);
        let mut params = json!({});
        let mut attrs = IndexMap::new();
        let mut values = IndexMap::new();
        for (key, limit) in fields {
            let value = arguments::optional_number(args, key)?
                .ok_or_else(|| format!("{key} must be a number"))?;
            attrs.insert((*key).into(), numeric(value, key, *limit)?);
            params[*key] = json!(value);
            values.insert(*key, value);
        }
        params["parent_id"] = json!(parent);
        if local == "rect" {
            for key in ["rx", "ry"] {
                if let Some(value) = arguments::optional_number(args, key)? {
                    attrs.insert(key.into(), numeric(value, key, 2)?);
                }
            }
        }
        let mut result_bbox = Value::Null;
        let mut text = None;
        match local {
            "rect" => {
                result_bbox = json!({"x":values["x"],"y":values["y"],"width":values["width"],"height":values["height"]});
            }
            "circle" | "ellipse" => {
                let rx = if local == "circle" {
                    values["r"]
                } else {
                    values["rx"]
                };
                let ry = if local == "circle" { rx } else { values["ry"] };
                result_bbox =
                    json!({"x":values["cx"]-rx,"y":values["cy"]-ry,"width":2.0*rx,"height":2.0*ry});
            }
            "line" => {
                result_bbox = bbox(&[[values["x1"], values["y1"]], [values["x2"], values["y2"]]]);
            }
            "polygon" | "polyline" => {
                let points = args["points"].as_array().ok_or("points must be a list")?;
                if points.is_empty() {
                    return Err(format!("{local} requires at least one point"));
                }
                if points.len() > 100_000 {
                    return Err(format!("too many points: {} > 100000", points.len()));
                }
                let mut pairs = Vec::new();
                let mut tokens = Vec::new();
                for point in points {
                    let pair = point
                        .as_array()
                        .filter(|p| p.len() == 2)
                        .ok_or("each point must contain two numbers")?;
                    let x = arguments::number(&pair[0]).ok_or("point x must be a number")?;
                    let y = arguments::number(&pair[1]).ok_or("point y must be a number")?;
                    tokens.push(format!(
                        "{},{}",
                        numeric(x, "point x", 0)?,
                        numeric(y, "point y", 0)?
                    ));
                    pairs.push([x, y]);
                }
                attrs.insert("points".into(), tokens.join(" "));
                params = json!({"point_count":points.len(),"parent_id":parent});
                result_bbox = bbox(&pairs);
            }
            "path" => {
                let raw = args["d"].as_str().ok_or("d must be a string")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("path d is empty".into());
                }
                let length = value.chars().count();
                if length > 200_000 {
                    return Err(format!("path d too long: {length} > 200000 characters"));
                }
                if !PATH.is_match(value) {
                    return Err("path d contains invalid characters".into());
                }
                let review = crate::geometry_quality::path(value, 0.)?;
                let require_closed = arguments::boolean(args, "require_closed", false)?;
                let reject_zero_length = arguments::boolean(args, "reject_zero_length", false)?;
                if require_closed && review.open_subpaths > 0 {
                    return Err("require_closed: every subpath must end with Z".into());
                }
                if reject_zero_length && review.zero_segments > 0 {
                    return Err(
                        "reject_zero_length: path contains exact zero-length segments".into(),
                    );
                }
                attrs.insert("d".into(), value.into());
                params = json!({"d_length":raw.chars().count(),"parent_id":parent,"require_closed":require_closed,"reject_zero_length":reject_zero_length});
            }
            "text" => {
                let value = args["text"].as_str().ok_or("text must be a string")?;
                let length = value.chars().count();
                if length > 100_000 {
                    return Err(format!("text too long: {length} > 100000 characters"));
                }
                if value
                    .chars()
                    .any(|c| matches!(c as u32,0..=8|11..=12|14..=31|127..=159))
                {
                    return Err("text contains forbidden control characters".into());
                }
                params["text_length"] = json!(length);
                text = Some(value.into());
            }
            _ => (),
        }
        let mut paint = Vec::new();
        for (key, property) in [
            ("fill", "fill"),
            ("stroke", "stroke"),
            ("stroke_width", "stroke-width"),
        ] {
            if key == "fill" && local == "line" {
                continue;
            }
            if let Some(raw) = arguments::string(args, key)? {
                let value = if key == "stroke_width" {
                    let value = raw.trim();
                    if !LENGTH.is_match(value) {
                        return Err(format!(
                            "invalid length value: {}",
                            style::python_repr(value)
                        ));
                    }
                    value.into()
                } else {
                    style::paint(raw)?
                };
                paint.push((property.into(), value));
            }
        }
        Ok(Self {
            local: local.into(),
            attrs,
            parent,
            id,
            text,
            paint,
            params,
            bbox: result_bbox,
        })
    }
    pub fn create(&self, document: &mut Document) -> Result<(String, String), String> {
        let mut parent = resolve_parent(document, self.parent.as_deref())?;
        let id = resolve_id(document, self.id.as_deref(), &self.local)?;
        let mut node = svg_node(&self.local, &parent, document)?;
        node.set_property("id", &id)
            .map_err(|_| "vector creation failed")?;
        for (key, value) in &self.attrs {
            node.set_property(key, value)
                .map_err(|_| "vector creation failed")?;
        }
        for (key, value) in &self.paint {
            style::write_property(&mut node, key, value)?;
        }
        if let Some(text) = &self.text {
            let mut literal =
                Node::new_text(text, document).map_err(|_| "vector creation failed")?;
            node.add_child(&mut literal)
                .map_err(|_| "vector creation failed")?;
        }
        parent
            .add_child(&mut node)
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
    result["bbox"] = mutation.bbox;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_guards_refuse_before_mutation_and_batch_history() {
        let temp = tempfile::tempdir().unwrap();
        let source = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'/>";
        std::fs::write(temp.path().join("source.svg"), source).unwrap();
        let mut registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![temp.path().canonicalize().unwrap()],
                max_input: 100000,
                max_output: 100000,
            },
            entries: indexmap::IndexMap::new(),
        };
        let id = registry.open(&json!({"path":"source.svg"})).unwrap()["doc_id"]
            .as_str()
            .unwrap()
            .to_owned();
        for spec in [
            json!({"d":"M0 0 L1 1","require_closed":true}),
            json!({"d":"M0 0 L0 0 Z","reject_zero_length":true}),
            json!({"d":"M0 0 L"}),
            json!({"d":"M1e999 0"}),
        ] {
            let mut args = spec.clone();
            args["doc_id"] = json!(id);
            assert!(apply(&registry, "create_path", &args).is_err());
            let mut edit = spec;
            edit["op"] = json!("create_path");
            assert!(crate::batch::apply(&registry,&json!({"doc_id":id,"edits":[{"op":"create_rect","x":0,"y":0,"width":1,"height":1},edit]})).is_err());
        }
        let entry = &registry.entries[&id];
        assert_eq!(
            std::fs::read(temp.path().join(entry.working())).unwrap(),
            source
        );
        for dir in ["operations", "snapshots"] {
            assert_eq!(
                std::fs::read_dir(temp.path().join(entry.directory()).join(dir))
                    .unwrap()
                    .count(),
                0
            );
        }
        assert!(
            Mutation::build(
                "create_path",
                &json!({"d":"M0 0 C1 0 1 1 0 0 Z","require_closed":true,"reject_zero_length":true})
            )
            .is_ok()
        );
        assert!(Mutation::build("create_path", &json!({"d":"M0 0 L0 0"})).is_ok());
    }
    #[test]
    fn default_layer_and_literal_empty_text() {
        let mut document=crate::xml::parse(br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:i="http://www.inkscape.org/namespaces/inkscape"><g id="group"/><g id="layer" i:groupmode="layer"/></svg>"#,4096).unwrap();
        Mutation::build(
            "create_text",
            &json!({"x":1,"y":2,"text":"","object_id":"label"}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        assert!(
            document
                .to_string()
                .contains("<text id=\"label\" x=\"1\" y=\"2\"></text>")
        );
        let target = elements(document.get_root_element().unwrap())
            .into_iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some("label"))
            .unwrap();
        assert_eq!(
            target
                .get_parent()
                .unwrap()
                .get_property_no_ns("id")
                .as_deref(),
            Some("layer")
        );
    }
}
