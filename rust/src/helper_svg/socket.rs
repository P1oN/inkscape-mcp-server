//! Snapshot semantics for the fixed v5 socket extension. No IO or GUI control.
use super::{apply, edit, elements};
use libxml::tree::Node;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
pub const CAP: usize = 16 * 1024 * 1024;
const INK: &str = "http://www.inkscape.org/namespaces/inkscape";
pub struct Snapshot {
    pub svg: String,
    pub selection: Vec<String>,
    pub path: Option<String>,
    original: String,
}
impl Snapshot {
    pub fn new(
        svg: String,
        selection: Vec<String>,
        path: Option<String>,
    ) -> Result<Self, &'static str> {
        let doc = crate::xml::parse(svg.as_bytes(), CAP)?;
        let root = doc.get_root_element().ok_or("invalid SVG")?;
        if root.get_name() != "svg"
            || root.get_namespace().map(|n| n.get_href()).as_deref()
                != Some("http://www.w3.org/2000/svg")
        {
            return Err("invalid SVG");
        }
        let nodes = elements(root);
        if nodes.len() > 10_000 || selection.len() > 10_000 {
            return Err("snapshot exceeds element cap");
        }
        let mut ids = HashSet::new();
        for n in nodes {
            if let Some(id) = n.get_property_no_ns("id")
                && (!ids.insert(id.clone()) || id.len() > 4096)
            {
                return Err("ambiguous IDs");
            }
        }
        let mut selected = HashSet::new();
        if selection
            .iter()
            .any(|id| id.is_empty() || id.len() > 4096 || !ids.contains(id) || !selected.insert(id))
        {
            return Err("invalid selection");
        }
        Ok(Self {
            original: svg.clone(),
            svg,
            selection,
            path,
        })
    }
    pub fn changed(&self) -> bool {
        self.svg != self.original
    }
    pub fn objects(&self) -> Result<Vec<Value>, &'static str> {
        let doc = crate::xml::parse(self.svg.as_bytes(), CAP)?;
        Ok(elements(doc.get_root_element().ok_or("invalid SVG")?)
            .into_iter()
            .filter(|n| {
                !matches!(
                    n.get_name().as_str(),
                    "svg" | "defs" | "metadata" | "style" | "title" | "desc" | "namedview"
                )
            })
            .map(|n| Self::info(&n))
            .collect())
    }
    pub fn info(n: &Node) -> Value {
        json!({"id":n.get_property_no_ns("id"),"tag":n.get_name(),"label":n.get_property_ns("label", INK),"has_style":(["style","fill","stroke"].iter().any(|k| n.get_property_no_ns(k).is_some_and(|v| !v.is_empty())))})
    }
    pub fn read(&self, cmd: &str, params: &Value) -> Result<Value, &'static str> {
        match cmd {
            "ping" => Ok(json!({"pong":true})),
            "get_active_document" => Ok(
                json!({"name":self.path.as_ref().and_then(|p|std::path::Path::new(p).file_name()).and_then(|n|n.to_str()),"path":self.path,"object_count":self.objects()?.len()}),
            ),
            "get_selection" => Ok(json!({"object_ids":self.selection})),
            "inspect_selection" => Ok(
                json!({"objects":self.objects()?.iter().filter(|n|n["id"].as_str().is_some_and(|id|self.selection.iter().any(|s|s==id))).collect::<Vec<_>>()}),
            ),
            "get_document_svg" => Ok(json!({"svg":self.svg})),
            "get_state_token" => Ok(
                json!({"revision":format!("{:x}",Sha256::digest(self.svg.as_bytes())),"selection":self.selection,"viewport":{"zoom":null,"center":null}}),
            ),
            "set_viewport" => {
                let mode = params["mode"].as_str().ok_or("invalid mode")?;
                if !matches!(mode, "zoom" | "pan" | "fit_page" | "fit_selection") {
                    return Err("invalid mode");
                }
                Ok(
                    json!({"mode":mode,"applied":false,"detail":"The modal extension cannot control the live viewport."}),
                )
            }
            _ => Err("unknown command"),
        }
    }
    pub fn canvas(&self) -> Result<Value, &'static str> {
        let doc = crate::xml::parse(self.svg.as_bytes(), CAP)?;
        let root = doc.get_root_element().ok_or("invalid SVG")?;
        // Retain the existing socket contract: leading numeric lengths, viewBox fallback.
        let leading = |name: &str| {
            root.get_property_no_ns(name).and_then(|s| {
                static NUMBER: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
                    regex::Regex::new(r"^\s*([-+]?[0-9]*\.?[0-9]+)").unwrap()
                });
                NUMBER
                    .captures(&s)
                    .and_then(|c| c[1].parse::<f64>().ok())
                    .filter(|n| n.is_finite())
            })
        };
        let vb = root
            .get_property_no_ns("viewBox")
            .and_then(|s| {
                s.replace(',', " ")
                    .split_whitespace()
                    .map(str::parse::<f64>)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()
            })
            .filter(|v| v.len() == 4 && v.iter().all(|n| n.is_finite()));
        Ok(
            json!({"width":leading("width").or_else(||vb.as_ref().map(|v|v[2])),"height":leading("height").or_else(||vb.as_ref().map(|v|v[3])),"units":root.get_property_ns("document-units",INK)}),
        )
    }
    pub fn mutate(&mut self, cmd: &str, params: &Value) -> Result<Value, &'static str> {
        let nonce = format!("mcp_{}", uuid::Uuid::new_v4().simple());
        let applied = match cmd {
            "apply_to_selection" | "set_selected_text" => {
                let style = if params["style"].is_null() {
                    BTreeMap::new()
                } else {
                    serde_json::from_value(params["style"].clone()).map_err(|_| "invalid style")?
                };
                let optional = |key: &str| {
                    if params[key].is_null() {
                        Ok(None)
                    } else {
                        params[key]
                            .as_str()
                            .map(|s| Some(s.to_string()))
                            .ok_or("invalid edit")
                    }
                };
                let request = edit::Request {
                    nonce,
                    operation: if cmd == "set_selected_text" {
                        edit::Operation::Text
                    } else {
                        edit::Operation::Style
                    },
                    selection: self.selection.clone(),
                    style,
                    transform: optional("transform")?,
                    text: optional("text")?,
                };
                apply::edit(&self.svg, &request, CAP)?
            }
            "insert_svg" => apply::insert(
                &self.svg,
                params["svg"].as_str().ok_or("invalid fragment")?,
                &nonce,
                CAP,
            )?,
            _ => return Err("unknown command"),
        };
        if !applied.bytes.is_empty() {
            let candidate = String::from_utf8(applied.bytes).map_err(|_| "invalid SVG")?;
            // Enforce the snapshot's element/ID limits after every growth operation.
            Self::new(candidate.clone(), self.selection.clone(), self.path.clone())?;
            self.svg = candidate;
        }
        Ok(
            json!({"affected_ids":applied.ids,"detail":match cmd {"set_selected_text"=>"set selected text","insert_svg"=>"inserted SVG",_=>"applied style/transform"},"undo_friendly":true}),
        )
    }
}
