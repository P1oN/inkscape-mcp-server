//! Read-only per-element inspection; never expand entity references into text.
use crate::document::{INKSCAPE_NS, Registry, elements};
use libxml::tree::{Node, NodeType};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;

static DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*([^:;]+?)\s*:\s*([^;]+?)\s*(?:;|$)").unwrap());
static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*([+-]?\d*\.?\d+(?:[eE][+-]?\d+)?)").unwrap());
static COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());
static URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"url\(\s*['"]?([^'")]+)['"]?\s*\)"#).unwrap());
static FONT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)font-family\s*:\s*([^;}\n]+)").unwrap());

fn declarations(node: &Node) -> indexmap::IndexMap<String, String> {
    parse_declarations(&node.get_property_no_ns("style").unwrap_or_default())
}

pub fn parse_declarations(raw: &str) -> indexmap::IndexMap<String, String> {
    DECL.captures_iter(raw)
        .map(|c| (c[1].trim().to_lowercase(), c[2].trim().to_owned()))
        .collect()
}

pub(crate) fn layer(node: &Node) -> bool {
    node.get_name() == "g"
        && node.get_property_ns("groupmode", INKSCAPE_NS).as_deref() == Some("layer")
}

pub(crate) fn paint(node: &Node) -> Value {
    let decls = declarations(node);
    let prop = |name: &str| {
        decls
            .get(name)
            .cloned()
            .or_else(|| node.get_property_no_ns(name))
            .map(|v| v.trim().to_owned())
    };
    let fill = prop("fill");
    let stroke = prop("stroke");
    let only = stroke
        .as_ref()
        .is_some_and(|s| !s.eq_ignore_ascii_case("none"))
        && fill
            .as_ref()
            .is_some_and(|s| s.eq_ignore_ascii_case("none"));
    json!({"fill":fill,"stroke":stroke,"stroke_width":prop("stroke-width"),"stroke_only":only})
}

fn num(node: &Node, name: &str) -> Option<f64> {
    let raw = node.get_property_no_ns(name)?;
    crate::decimal::float(&NUMBER.captures(&raw)?[1])
}

pub(crate) fn bbox(node: &Node) -> Option<Value> {
    if node
        .get_property_no_ns("transform")
        .is_some_and(|s| !s.is_empty())
    {
        return None;
    }
    let positive = |name: &str| num(node, name).filter(|n| *n >= 0.0);
    let (x, y, w, h) = match node.get_name().as_str() {
        "rect" | "image" | "use" => (
            num(node, "x").unwrap_or(0.0),
            num(node, "y").unwrap_or(0.0),
            positive("width")?,
            positive("height")?,
        ),
        "circle" => {
            let r = positive("r")?;
            (
                num(node, "cx").unwrap_or(0.0) - r,
                num(node, "cy").unwrap_or(0.0) - r,
                2.0 * r,
                2.0 * r,
            )
        }
        "ellipse" => {
            let rx = positive("rx")?;
            let ry = positive("ry")?;
            (
                num(node, "cx").unwrap_or(0.0) - rx,
                num(node, "cy").unwrap_or(0.0) - ry,
                2.0 * rx,
                2.0 * ry,
            )
        }
        "line" => {
            let a = num(node, "x1")?;
            let b = num(node, "y1")?;
            let c = num(node, "x2")?;
            let d = num(node, "y2")?;
            (a.min(c), b.min(d), (c - a).abs(), (d - b).abs())
        }
        "polygon" | "polyline" => {
            let raw = node.get_property_no_ns("points")?;
            let values = raw
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            let points = values.as_chunks::<2>().0.iter().collect::<Vec<_>>();
            let first = points.first()?;
            let (mut x, mut y, mut maxx, mut maxy) = (first[0], first[1], first[0], first[1]);
            for p in points {
                x = x.min(p[0]);
                y = y.min(p[1]);
                maxx = maxx.max(p[0]);
                maxy = maxy.max(p[1]);
            }
            (x, y, maxx - x, maxy - y)
        }
        _ => return None,
    };
    Some(json!({"x":x,"y":y,"width":w,"height":h}))
}

pub fn text(node: &Node) -> String {
    let mut raw = String::new();
    let mut stack = node.get_child_nodes();
    stack.reverse();
    while let Some(child) = stack.pop() {
        match child.get_type() {
            Some(NodeType::TextNode | NodeType::CDataSectionNode) => {
                raw.push_str(&child.get_content())
            }
            Some(NodeType::EntityRefNode) => {
                raw.push('&');
                raw.push_str(&child.get_name());
                raw.push(';');
            }
            Some(NodeType::ElementNode) => stack.extend(child.get_child_nodes().into_iter().rev()),
            _ => (),
        }
    }
    raw
}

fn color(raw: &str) -> Option<String> {
    let value = raw.trim().to_lowercase();
    if value.is_empty()
        || matches!(
            value.as_str(),
            "none" | "inherit" | "transparent" | "currentcolor" | "context-fill" | "context-stroke"
        )
        || value.starts_with("url(")
    {
        return None;
    }
    if let Some(hex) = value.strip_prefix('#') {
        if !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        return match hex.len() {
            3 => Some(format!(
                "#{}",
                hex.chars().flat_map(|c| [c, c]).collect::<String>()
            )),
            6 | 8 => Some(value),
            _ => None,
        };
    }
    Some(value)
}

pub(crate) fn object_ref(node: &Node) -> Option<Value> {
    if matches!(
        node.get_name().as_str(),
        "svg" | "defs" | "metadata" | "style" | "title" | "desc" | "namedview"
    ) {
        return None;
    }
    let id = node.get_property_no_ns("id").filter(|s| !s.is_empty())?;
    let p = paint(node);
    let collapsed = text(node).split_whitespace().collect::<Vec<_>>().join(" ");
    Some(
        json!({"object_id":id,"tag":node.get_name(),"bbox":bbox(node),
        "fill":p["fill"],"stroke":p["stroke"],"text":if collapsed.is_empty(){None}else{Some(collapsed)}}),
    )
}

pub(crate) fn object_info(node: &Node) -> Value {
    json!({"id":node.get_property_no_ns("id"),"tag":node.get_name(),"label":node.get_property_ns("label",INKSCAPE_NS),"has_style":(["style","fill","stroke"].iter().any(|a|node.get_property_no_ns(a).is_some_and(|s| !s.is_empty()))),"paint":paint(node),"is_layer":layer(node),"is_leaf":node.get_child_elements().is_empty(),"bbox":bbox(node)})
}
pub(crate) fn tree(root: Node) -> Value {
    let nodes = elements(root);
    let mut built = std::collections::HashMap::new();
    for node in nodes.iter().rev() {
        let children = node
            .get_child_elements()
            .iter()
            .map(|n| built.remove(&n.node_ptr()).unwrap())
            .collect::<Vec<Value>>();
        built.insert(node.node_ptr(),json!({"tag":node.get_name(),"id":node.get_property_no_ns("id"),"label":node.get_property_ns("label",INKSCAPE_NS),"paint":paint(node),"is_layer":layer(node),"is_leaf":children.is_empty(),"bbox":bbox(node),"children":children,"transform":node.get_property_no_ns("transform")}));
    }
    built.remove(&nodes[0].node_ptr()).unwrap()
}

pub fn resource(registry: &Registry, id: &str, leaf: &str) -> Result<Value, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let document = crate::xml::parse(&bytes, registry.workspace.max_input)?;
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let nodes = elements(root);
    match leaf {
        "tree" => Ok(json!({"doc_id":id,"root":tree(nodes[0].clone())})),
        "layers" => Ok(
            json!({"doc_id":id,"layers":nodes.iter().filter(|n|layer(n)).map(|n| {
            let d=declarations(n);
            json!({"id":n.get_property_no_ns("id"),"label":n.get_property_ns("label",INKSCAPE_NS),"visible":d.get("display").map(String::as_str)!=Some("none") && d.get("visibility").map(String::as_str)!=Some("hidden"),"locked":n.get_property_ns("insensitive","http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd").as_deref()==Some("true"),"num_children":n.get_child_elements().len()})
        }).collect::<Vec<_>>()}),
        ),
        "objects" | "refs" => {
            let objects = nodes
                .iter()
                .filter(|n| {
                    !matches!(
                        n.get_name().as_str(),
                        "svg" | "defs" | "metadata" | "style" | "title" | "desc" | "namedview"
                    )
                })
                .filter_map(|n| {
                    if leaf == "refs" {
                        object_ref(n)
                    } else {
                        Some(object_info(n))
                    }
                })
                .collect::<Vec<_>>();
            if leaf == "refs" {
                Ok(json!(objects))
            } else {
                Ok(json!({"doc_id":id,"objects":objects}))
            }
        }
        "styles" => {
            let mut colors = Vec::new();
            let mut inline = 0;
            let mut rules = 0;
            for n in &nodes {
                if n.get_name() == "style" {
                    rules += COMMENT.replace_all(&own_text(n), "").matches('{').count();
                    continue;
                }
                if n.get_property_no_ns("style").is_some_and(|s| !s.is_empty()) {
                    inline += 1;
                }
                let d = declarations(n);
                for v in [
                    d.get("fill").cloned(),
                    d.get("stroke").cloned(),
                    n.get_property_no_ns("fill"),
                    n.get_property_no_ns("stroke"),
                ]
                .into_iter()
                .flatten()
                {
                    if let Some(c) = color(&v)
                        && !colors.contains(&c)
                    {
                        colors.push(c);
                    }
                }
            }
            Ok(
                json!({"doc_id":id,"colors":colors,"inline_style_count":inline,"css_rule_count":rules}),
            )
        }
        "assets" => {
            let mut assets = Vec::new();
            let mut seen = std::collections::HashSet::new();
            let external = |s: &str| !s.starts_with('#') && !s.to_lowercase().starts_with("data:");
            for n in &nodes {
                let mut add = |kind: &str, href: &str| {
                    let href = href.trim();
                    if !href.is_empty() && seen.insert((kind.to_owned(), href.to_owned())) {
                        assets.push(json!({"kind":kind,"href":href,"external":external(href),"used_by":n.get_property_no_ns("id")}));
                    }
                };
                if let Some(href) = n
                    .get_property_ns("href", "http://www.w3.org/1999/xlink")
                    .filter(|s| !s.is_empty())
                    .or_else(|| n.get_property_no_ns("href"))
                {
                    let name = n.get_name();
                    add(
                        if matches!(name.as_str(), "image" | "use") {
                            &name
                        } else {
                            "other"
                        },
                        &href,
                    );
                }
                for a in ["style", "fill", "stroke", "mask", "clip-path", "filter"] {
                    if let Some(v) = n.get_property_no_ns(a) {
                        for c in URL.captures_iter(&v) {
                            if external(c[1].trim()) {
                                add("other", &c[1]);
                            }
                        }
                    }
                }
            }
            Ok(json!({"doc_id":id,"assets":assets}))
        }
        "fonts" => {
            let mut families: indexmap::IndexMap<String, (usize, Option<String>)> =
                indexmap::IndexMap::new();
            for n in &nodes {
                let values = if n.get_name() == "style" {
                    FONT.captures_iter(&own_text(n))
                        .map(|c| c[1].to_owned())
                        .collect::<Vec<_>>()
                } else {
                    [
                        declarations(n).get("font-family").cloned(),
                        n.get_property_no_ns("font-family"),
                    ]
                    .into_iter()
                    .flatten()
                    .collect()
                };
                for value in values {
                    for part in value.split(',') {
                        let family = part.trim().trim_matches(['\'', '"']).trim();
                        if !family.is_empty() {
                            families
                                .entry(family.to_owned())
                                .or_insert((0, n.get_property_no_ns("id")))
                                .0 += 1;
                        }
                    }
                }
            }
            let installed = if families.is_empty() {
                None
            } else {
                crate::fonts::installed(registry.workspace.max_output)
            };
            let fonts = families
                .into_iter()
                .map(|(family, (count, used_by))| {
                    let low = family.trim().to_lowercase();
                    let available = if crate::fonts::generic(&low) {
                        Some(true)
                    } else {
                        installed.as_ref().map(|set| set.contains(&low))
                    };
                    json!({"family":family,"count":count,"available":available,"used_by":used_by})
                })
                .collect::<Vec<_>>();
            Ok(json!({"doc_id":id,"fonts":fonts}))
        }
        _ => Err("Rust migration pending: resource not ported".into()),
    }
}

pub fn document(registry: &Registry, id: &str) -> Result<Value, String> {
    Ok(
        json!({"summary":registry.summary(id)?,"tree":resource(registry,id,"tree")?,
        "layers":resource(registry,id,"layers")?,"styles":resource(registry,id,"styles")?,
        "fonts":resource(registry,id,"fonts")?,"assets":resource(registry,id,"assets")?,
        "objects":resource(registry,id,"refs")?}),
    )
}

pub fn own_text(node: &Node) -> String {
    node.get_child_nodes()
        .into_iter()
        .take_while(|n| {
            matches!(
                n.get_type(),
                Some(NodeType::TextNode | NodeType::CDataSectionNode)
            )
        })
        .map(|n| n.get_content())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symbolic_entity_text_and_mixed_content() {
        let d=crate::xml::parse(br#"<!DOCTYPE svg [<!ENTITY e "secret">]><svg> a&e;b<tspan>c</tspan>d<!--ignored-->z</svg>"#,4096).unwrap();
        assert_eq!(text(&d.get_root_element().unwrap()), " a&e;bcdz");
    }
}
