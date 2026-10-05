//! Read-only live discovery from one guarded, self-contained SVG snapshot.
use crate::{
    arguments,
    document::{INKSCAPE_NS, elements},
    inspect,
    live::Live,
    live_scene, process, render,
    workspace::Workspace,
};
use libxml::tree::{Document, Node, NodeType};
use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::LazyLock,
};
static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)url\s*\((.*?)\)").unwrap());
static UNTERMINATED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)url\s*\(").unwrap());
static FOLD: LazyLock<HashMap<char, String>> = LazyLock::new(|| {
    let value: Value =
        serde_json::from_str(include_str!("../../migration/contracts/casefold.json")).unwrap();
    value["mapping"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.chars().next().unwrap(), v.as_str().unwrap().into()))
        .collect()
});
fn fold(value: &str) -> String {
    let mut result = String::new();
    for c in value.chars() {
        if let Some(s) = FOLD.get(&c) {
            result.push_str(s);
        } else {
            result.push(c);
        }
    }
    result
}
fn drawable(node: &Node) -> bool {
    matches!(
        node.get_name().as_str(),
        "g" | "rect"
            | "circle"
            | "ellipse"
            | "path"
            | "line"
            | "polygon"
            | "polyline"
            | "text"
            | "tspan"
            | "image"
            | "use"
            | "svg"
    )
}
pub(crate) fn snapshot(svg: &str, workspace: &Workspace) -> Result<Document, String> {
    if svg.len() > workspace.max_input {
        return Err(format!(
            "input exceeds max size: {} > {} bytes",
            svg.len(),
            workspace.max_input
        ));
    }
    let document = crate::xml::parse(svg.as_bytes(), workspace.max_input)
        .map_err(|_| "live discovery snapshot could not be inspected safely")?;
    let root = document
        .get_root_element()
        .ok_or("live discovery snapshot could not be inspected safely")?;
    let mut stack = vec![root];
    let mut count = 0;
    let mut ids = HashSet::new();
    while let Some(node) = stack.pop() {
        if matches!(
            node.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode)
        ) {
            continue;
        }
        count += 1;
        if count > 10_000 {
            return Err("live discovery exceeds 10000 elements".into());
        }
        stack.extend(node.get_child_nodes().into_iter().rev());
        if !node.is_element_node() {
            if matches!(
                node.get_type(),
                Some(NodeType::EntityRefNode | NodeType::PiNode)
            ) {
                return Err(
                    "entities and processing instructions are unsupported in discovery".into(),
                );
            }
            continue;
        }
        if matches!(
            node.get_name().as_str(),
            "script" | "foreignObject" | "style"
        ) {
            return Err(
                "scripts, foreignObject and stylesheets require snapshot preparation".into(),
            );
        }
        if let Some(id) = node.get_property_no_ns("id").filter(|s| !s.is_empty())
            && !ids.insert(id)
        {
            return Err("duplicate object ids make discovery ambiguous".into());
        }
        for ((name, ns), value) in node.get_properties_ns() {
            if name == "absref" && !value.is_empty() {
                return Err("image fallback paths require a self-contained snapshot".into());
            }
            if name == "base" || name.to_lowercase().starts_with("on") {
                return Err("active content is unsupported in discovery snapshots".into());
            }
            if name == "href"
                && !value.starts_with('#')
                && !(node.get_name() == "image"
                    && (value.starts_with("data:image/png;base64,")
                        || value.starts_with("data:image/jpeg;base64,")))
            {
                return Err("external assets require a self-contained discovery snapshot".into());
            }
            let css = ns.is_none() && !matches!(name.as_str(), "id" | "d" | "points" | "href");
            if css && (value.contains('\\') || value.contains("/*")) {
                return Err("escaped CSS is unsupported in discovery snapshots".into());
            }
            for c in URL.captures_iter(&value) {
                if !c[1]
                    .trim_matches([' ', '\t', '\r', '\n', '\"', '\''])
                    .starts_with('#')
                {
                    return Err("external paint references are unsupported in discovery".into());
                }
            }
            if UNTERMINATED.is_match(&URL.replace_all(&value, "")) {
                return Err("unterminated paint references are unsupported in discovery".into());
            }
        }
    }
    Ok(document)
}
fn query(document: &Document, workspace: &Workspace) -> Result<HashMap<String, [f64; 4]>, String> {
    let root = document.get_root_element().unwrap();
    let (sx, sy, tx, ty) = crate::geometry::root_mapping(&root)?;
    let binary =
        process::inkscape_binary().ok_or("Inkscape CLI is required for accurate live geometry")?;
    let temp = tempfile::Builder::new()
        .prefix("imcp-discovery-")
        .tempdir()
        .map_err(|_| "live discovery snapshot could not be inspected safely")?;
    let stage = Workspace {
        roots: vec![
            temp.path()
                .canonicalize()
                .map_err(|_| "live discovery snapshot could not be inspected safely")?,
        ],
        max_input: workspace.max_input,
        max_output: workspace.max_output,
    };
    stage.write_new(
        0,
        Path::new("snapshot.svg"),
        document.node_to_string(&root).as_bytes(),
    )?;
    let result = process::run_bounded(
        &binary,
        &[
            stage.roots[0]
                .join("snapshot.svg")
                .to_string_lossy()
                .into_owned(),
            "--query-all".into(),
        ],
        process::timeout(),
        workspace.max_output,
    )
    .map_err(|_| "Inkscape CLI is required for accurate live geometry")?;
    if result.timed_out || !result.success || result.stdout.len() >= workspace.max_output {
        return Err("accurate live geometry query failed or timed out".into());
    }
    let stdout = String::from_utf8_lossy(&result.stdout);
    let mut boxes = HashMap::new();
    for line in stdout.lines() {
        let fields = line.rsplitn(5, ',').collect::<Vec<_>>();
        if fields.len() != 5 || fields[4].is_empty() {
            continue;
        }
        let Some(n) = fields[..4]
            .iter()
            .rev()
            .map(|s| crate::decimal::float(s.trim()))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let mapped = [(n[0] - tx) / sx, (n[1] - ty) / sy, n[2] / sx, n[3] / sy];
        if mapped.iter().any(|n| !n.is_finite()) || n[2] < 0. || n[3] < 0. {
            continue;
        }
        boxes.insert(fields[4].into(), mapped);
    }
    if boxes.is_empty() && !stdout.trim().is_empty() {
        return Err("Inkscape returned no usable object geometry".into());
    }
    Ok(boxes)
}
pub fn find(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let (active, svg) = live
        .session
        .operation(|t| Ok((t.active_document()?, t.document_svg()?)))
        .map_err(|e| {
            crate::live_events::protocol_message(&e)
                .map(|m| format!("Error calling tool 'live_find_objects': {m}"))
                .unwrap_or_else(|| e.public_message().into())
        })?;
    let limit = render::integer(args, "limit")?.unwrap_or(100);
    if !(1..=1000).contains(&limit) {
        return Err("discovery limit must be between 1 and 1000".into());
    }
    let label = arguments::string(args, "label")?.map(fold);
    let layer = arguments::string(args, "layer")?;
    let text = arguments::string(args, "text")?.map(fold);
    let tag = arguments::string(args, "tag")?;
    let prefix = arguments::string(args, "id_prefix")?;
    let doc = snapshot(&svg, workspace)?;
    let root = doc.get_root_element().unwrap();
    let boxes = query(&doc, workspace)?;
    let mut matches = Vec::new();
    for node in elements(root).into_iter().skip(1) {
        let Some(id) = node.get_property_no_ns("id").filter(|s| !s.is_empty()) else {
            continue;
        };
        if !drawable(&node) || !live_scene::visible(&node) {
            continue;
        }
        let mut ancestors = vec![node.clone()];
        let mut parent = node.get_parent();
        while let Some(p) = parent {
            if p.is_element_node() {
                ancestors.push(p.clone());
            }
            parent = p.get_parent();
        }
        let owner = ancestors.iter().find(|n| inspect::layer(n));
        let layer_id = owner.and_then(|n| n.get_property_no_ns("id"));
        let layer_label = owner.and_then(|n| n.get_property_ns("label", INKSCAPE_NS));
        let own_label = node.get_property_ns("label", INKSCAPE_NS);
        let words = matches!(node.get_name().as_str(), "text" | "tspan").then(|| {
            inspect::text(&node)
                .split(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        });
        if tag.is_some_and(|t| t != node.get_name())
            || prefix.is_some_and(|p| !id.starts_with(p))
            || label
                .as_ref()
                .is_some_and(|n| !fold(own_label.as_deref().unwrap_or("")).contains(n))
            || layer.is_some_and(|n| {
                layer_id.as_deref() != Some(n)
                    && !fold(layer_label.as_deref().unwrap_or("")).contains(&fold(n))
            })
            || text
                .as_ref()
                .is_some_and(|n| !fold(words.as_deref().unwrap_or("")).contains(n))
        {
            continue;
        }
        let mut info = live_scene::object_info(&node);
        info["bbox"] = boxes
            .get(&id)
            .map(|b| json!({"x":b[0],"y":b[1],"width":b[2],"height":b[3]}))
            .unwrap_or(Value::Null);
        info["layer_id"] = json!(layer_id);
        info["layer_label"] = json!(layer_label);
        info["text"] = json!(words);
        info["locked"] = json!(ancestors.iter().any(|n| {
            n.get_property_ns(
                "insensitive",
                "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
            )
            .as_deref()
                == Some("true")
        }));
        matches.push(info);
    }
    let total = matches.len();
    matches.truncate(limit as usize);
    let fingerprint = crate::live_effect::fingerprint(&svg, workspace.max_input)
        .map_err(|_| "live discovery snapshot could not be inspected safely")?;
    Ok(
        json!({"active_document":active,"fingerprint":fingerprint,"coordinate_space":"document_user_units","count":matches.len(),"objects":matches,"total_matches":total,"truncated":total>limit as usize,"notes":["Bounds are Inkscape engine bounds converted to document user units.","Paint remains explicit; semantic identity must be confirmed from a preview."]}),
    )
}
/// Isolated export preserves ancestors/defs in the immutable snapshot, without GUI selection.
pub fn preview(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let svg = live.session.operation(|t| t.document_svg()).map_err(|e| {
        crate::live_events::protocol_message(&e)
            .map(|m| format!("Error calling tool 'live_preview_object': {m}"))
            .unwrap_or_else(|| e.public_message().into())
    })?;
    let width = render::integer(args, "width")?.unwrap_or(512);
    if width < 1 || width > i64::from(render::cap()) {
        return Err("preview width is outside the export dimension cap".into());
    }
    let id = args["object_id"]
        .as_str()
        .ok_or("object_id must be a string")?;
    if id.is_empty()
        || id
            .chars()
            .any(|c| matches!(c, ',' | ';') || c.is_whitespace() || c <= '\u{1f}' || c == '\u{7f}')
    {
        return Err("object id is unsupported by isolated export".into());
    }
    let expected = args["expected_fingerprint"]
        .as_str()
        .ok_or("expected_fingerprint must be a string")?;
    let document = snapshot(&svg, workspace).map_err(|e| {
        if e == "live discovery snapshot could not be inspected safely" {
            "live object preview could not be rendered safely".into()
        } else {
            e
        }
    })?;
    let fingerprint = crate::live_effect::fingerprint(&svg, workspace.max_input)
        .map_err(|_| "live object preview could not be rendered safely")?;
    if fingerprint != expected {
        return Err("drawing changed since discovery; find the object again before preview".into());
    }
    let root = document.get_root_element().unwrap();
    let candidate = elements(root.clone())
        .into_iter()
        .find(|n| n.get_property_no_ns("id").as_deref() == Some(id));
    if candidate
        .as_ref()
        .is_none_or(|n| !drawable(n) || !live_scene::visible(n))
    {
        return Err("visible object id not found".into());
    }
    if workspace.roots.is_empty() {
        return Err("no workspace root configured for object previews".into());
    }
    for directory in [
        ".inkscape-mcp/live/artifacts",
        ".inkscape-mcp/live/operations",
    ] {
        workspace
            .ensure_directory(0, Path::new(directory))
            .map_err(|_| "live object preview could not be rendered safely")?;
    }
    let b = query(&document, workspace)?
        .get(id)
        .copied()
        .ok_or("object has no nonempty engine bounds")?;
    if b[2] <= 0. || b[3] <= 0. {
        return Err("object has no nonempty engine bounds".into());
    }
    let (sx, sy, _, _) = crate::geometry::root_mapping(&root)?;
    let (pw, ph) = (b[2] * sx, b[3] * sy);
    if !pw.is_finite() || !ph.is_finite() || pw <= 0. || ph <= 0. {
        return Err("object bounds cannot produce a finite preview size".into());
    }
    let requested_height = width as f64 * (ph / pw);
    if !requested_height.is_finite() {
        return Err("preview height exceeds the export cap; reduce preview width".into());
    }
    let height = requested_height.ceil();
    if height > f64::from(render::cap()) {
        return Err(format!(
            "export dimensions exceed cap: {width}x{height:.0} > {}px per side",
            render::cap()
        ));
    }
    let binary =
        process::inkscape_binary().ok_or("Inkscape CLI is required for object previews")?;
    let temp = tempfile::Builder::new()
        .prefix("imcp-preview-")
        .tempdir()
        .map_err(|_| "live object preview could not be rendered safely")?;
    let stage = Workspace {
        roots: vec![
            temp.path()
                .canonicalize()
                .map_err(|_| "live object preview could not be rendered safely")?,
        ],
        max_input: workspace.max_input,
        max_output: workspace.max_output,
    };
    stage.write_new(
        0,
        Path::new("snapshot.svg"),
        document.node_to_string(&root).as_bytes(),
    )?;
    let result = process::run_bounded(
        &binary,
        &[
            stage.roots[0]
                .join("snapshot.svg")
                .to_string_lossy()
                .into_owned(),
            format!("--export-id={id}"),
            "--export-id-only".into(),
            "--export-type=png".into(),
            format!("--export-width={width}"),
            format!(
                "--export-filename={}",
                stage.roots[0].join("object.png").display()
            ),
        ],
        process::timeout(),
        workspace.max_output,
    )
    .map_err(|_| "Inkscape CLI is required for object previews")?;
    if result.timed_out
        || !result.success
        || stage.file_kind(0, Path::new("object.png"))? != Some(libc::S_IFREG)
    {
        return Err("object preview failed or timed out".into());
    }
    let bytes = stage
        .read(0, Path::new("object.png"), workspace.max_output)
        .map_err(|_| "live object preview could not be rendered safely")?;
    let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
    let image = decoder
        .read_info()
        .map_err(|_| "object renderer did not produce PNG")?;
    if image.info().width > render::cap() || image.info().height > render::cap() {
        return Err(format!(
            "export dimensions exceed cap: {}x{} > {}px per side",
            image.info().width,
            image.info().height,
            render::cap()
        ));
    }
    let mut path = None;
    for _ in 0..4 {
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let name = format!(
            ".inkscape-mcp/live/artifacts/live-view-object-{}.png",
            &nonce[..24]
        );
        if workspace.file_kind(0, Path::new(&name))?.is_none() {
            path = Some(name);
            break;
        }
    }
    let path = path.ok_or("live object preview could not be rendered safely")?;
    workspace
        .atomic_write(0, Path::new(&path), &bytes)
        .map_err(|_| "live object preview could not be rendered safely")?;
    Ok(
        json!({"artifact_path":path,"format":"png","size_bytes":bytes.len(),"region":false,"scale":null}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_caps_entities_and_unterminated_css_refuse_before_any_cli_work() {
        let workspace = Workspace {
            roots: vec![],
            max_input: 1024 * 1024,
            max_output: 4096,
        };
        let source =
            "<!DOCTYPE svg [<!ENTITY secret SYSTEM 'file:///never-read'>]><svg>&secret;</svg>";
        assert_eq!(
            snapshot(source, &workspace).err().unwrap(),
            "entities and processing instructions are unsupported in discovery"
        );
        assert_eq!(
            snapshot(
                "<svg><rect fill='URL( file:///never-read'/></svg>",
                &workspace
            )
            .err()
            .unwrap(),
            "unterminated paint references are unsupported in discovery"
        );
        let source = format!("<svg>{}</svg>", "<!--comment-->".repeat(10_000));
        assert_eq!(
            snapshot(&source, &workspace).err().unwrap(),
            "live discovery exceeds 10000 elements"
        );
        let small = Workspace {
            max_input: 6,
            ..workspace
        };
        assert_eq!(
            snapshot("<svg>é</svg>", &small).err().unwrap(),
            "input exceeds max size: 13 > 6 bytes"
        );
        assert!(small.roots.is_empty());
        assert_eq!(fold("Straße Σς ﬃ İ"), "strasse σσ ffi i\u{307}");
    }
}
