//! Fixed high-risk geometry Actions, private bounded CLI staging and shared reversible edits.
use crate::{
    arguments,
    document::{INKSCAPE_NS, Registry, elements},
    identity, placement, process, render, style, transaction,
    workspace::Workspace,
    xml,
};
use libxml::tree::{Document, Node, NodeType};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

fn action(op: &str) -> &'static str {
    match op {
        "simplify_path" | "cleanup_paths" => "path-simplify",
        "boolean_union" => "path-union",
        "boolean_difference" => "path-difference",
        "combine_paths" => "path-combine",
        "break_apart" => "path-break-apart",
        "stroke_to_path" => "object-stroke-to-path",
        _ => unreachable!("only fixed dispatches reach the geometry kernel"),
    }
}
fn merge(op: &str) -> bool {
    matches!(op, "boolean_union" | "boolean_difference" | "combine_paths")
}
fn targets(document: &Document, raw: &[String], op: &str) -> Result<Vec<String>, String> {
    if raw.is_empty() {
        return Err("no target object ids supplied".into());
    }
    let mut seen = HashSet::new();
    let ids: Vec<_> = raw
        .iter()
        .filter(|s| seen.insert((*s).clone()))
        .cloned()
        .collect();
    for id in &ids {
        if !render::safe_object_id(id) {
            return Err("object id is not a safe svg id".into());
        }
        if id.contains(':') {
            return Err("object id must not contain ':'".into());
        }
    }
    let present: HashSet<_> = elements(document.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    if ids.iter().any(|id| !present.contains(id)) {
        return Err("object id not found in document".into());
    }
    if merge(op) && ids.len() < 2 {
        return Err(format!("{op} requires at least two distinct object ids"));
    }
    Ok(ids)
}
fn find(document: &Document, id: &str) -> Option<Node> {
    elements(document.get_root_element().unwrap())
        .into_iter()
        .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
}
fn bottom(document: &Document, ids: &[String], op: &str) -> Option<String> {
    if !merge(op) {
        return None;
    }
    elements(document.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .find(|id| ids.contains(id))
}
fn effective(n: &Node, key: &str) -> Option<String> {
    style::declarations(n)
        .get(key)
        .cloned()
        .or_else(|| n.get_property_no_ns(key))
}
const SODIPODI_NS: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd";
const LEGACY_SODIPODI_NS: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd";

const SHAPE_GEOMETRY: &[&str] = &[
    "d", "x", "y", "width", "height", "rx", "ry", "cx", "cy", "r", "points",
];

fn is_svg(node: &Node) -> bool {
    node.get_namespace()
        .is_some_and(|ns| ns.get_href() == "http://www.w3.org/2000/svg")
}

fn union_fill_rule(node: &Node) -> Option<String> {
    let mut declarations =
        crate::css::analysis_declarations(&node.get_property_no_ns("style").unwrap_or_default())?;
    if declarations.keys().any(|key| {
        SHAPE_GEOMETRY.contains(&key.as_str())
            || matches!(
                key.as_str(),
                "all" | "transform" | "rotate" | "scale" | "translate" | "offset"
            )
            || ["offset-", "animation", "transition"]
                .iter()
                .any(|prefix| key.starts_with(prefix))
    }) {
        return None;
    }
    Some(
        declarations
            .swap_remove("fill-rule")
            .or_else(|| node.get_property_no_ns("fill-rule"))
            .unwrap_or_else(|| "inherit".into()),
    )
}

fn editable_union_shape(node: &Node) -> bool {
    matches!(
        node.get_name().as_str(),
        "path" | "rect" | "circle" | "ellipse" | "polygon" | "polyline"
    ) && is_svg(node)
        && node.get_child_elements().iter().all(|child| {
            is_svg(child)
                && matches!(child.get_name().as_str(), "title" | "desc")
                && child.get_child_elements().is_empty()
        })
        && union_fill_rule(node).is_some()
        && !node.get_properties_ns().keys().any(|(key, ns)| {
            matches!(key.as_str(), "path-effect" | "original-d")
                || (ns.as_ref().is_some_and(|ns| ns.get_href() == INKSCAPE_NS)
                    && matches!(
                        key.as_str(),
                        "connector-type"
                            | "connector-curvature"
                            | "connection-start"
                            | "connection-end"
                    ))
                || (key == "type"
                    && ns.as_ref().is_some_and(|ns| {
                        matches!(ns.get_href().as_str(), SODIPODI_NS | LEGACY_SODIPODI_NS)
                    }))
        })
}

// Document order determines the survivor and the order given to Inkscape.
fn union_operands(document: &Document, ids: &[String]) -> Result<Vec<(String, Node)>, String> {
    let mut selected: HashMap<_, _> = ids.iter().map(|id| (id.as_str(), false)).collect();
    let mut operands = Vec::with_capacity(ids.len());
    for node in elements(document.get_root_element().unwrap()) {
        if let Some(id) = node.get_property_no_ns("id")
            && let Some(seen) = selected.get_mut(id.as_str())
        {
            if *seen {
                return Err(format!("boolean_union operand ID is ambiguous: {id}"));
            }
            *seen = true;
            operands.push((id, node));
        }
    }
    let parent = operands.first().and_then(|(_, node)| node.get_parent());
    if operands.len() != ids.len()
        || parent.is_none()
        || operands
            .iter()
            .any(|(_, node)| node.get_parent() != parent || !editable_union_shape(node))
    {
        return Err("boolean_union requires sibling shapes with directly editable geometry".into());
    }
    crate::delete::Mutation {
        ids: operands[1..].iter().map(|(id, _)| id.clone()).collect(),
    }
    .check_references(document)?;
    Ok(operands)
}

fn same_merge_frame(body: &Node, result: &Node) -> bool {
    let (Some(body_rule), Some(result_rule)) = (union_fill_rule(body), union_fill_rule(result))
    else {
        return false;
    };
    if body.get_property_no_ns("transform") != result.get_property_no_ns("transform")
        || body_rule != result_rule
    {
        return false;
    }
    let mut old = body.get_parent().filter(Node::is_element_node);
    let mut new = result.get_parent().filter(Node::is_element_node);
    while let (Some(a), Some(b)) = (&old, &new) {
        if !is_svg(a)
            || !is_svg(b)
            || !matches!(a.get_name().as_str(), "svg" | "g" | "a")
            || a.get_name() != b.get_name()
        {
            return false;
        }
        // The plain SVG export omits editor metadata. It stays on
        // the original nodes and do not change the coordinate frame.
        let frame_attributes = |node: &Node| {
            let mut attrs = node.get_properties_ns();
            attrs.retain(|(key, ns), _| {
                !ns.as_ref().is_some_and(|ns| match ns.get_href().as_str() {
                    INKSCAPE_NS => matches!(key.as_str(), "label" | "groupmode" | "version"),
                    SODIPODI_NS | LEGACY_SODIPODI_NS => key == "docname",
                    _ => false,
                })
            });
            attrs
        };
        let original = frame_attributes(a);
        let mut incoming = frame_attributes(b);
        // Inkscape fills in missing IDs and a root version during export.
        for key in ["id", "version"] {
            let key = (key.to_owned(), None);
            if !original.contains_key(&key) {
                incoming.remove(&key);
            }
        }
        if original != incoming {
            return false;
        }
        old = a.get_parent().filter(Node::is_element_node);
        new = b.get_parent().filter(Node::is_element_node);
    }
    old.is_none() && new.is_none()
}

fn apply_union_geometry(operands: Vec<(String, Node)>, incoming: &Document) -> Result<(), String> {
    let mut operands = operands.into_iter();
    let (body_id, mut body) = operands.next().ok_or("union has no body")?;
    let result = find(incoming, &body_id)
        .filter(|n| n.get_name() == "path" && is_svg(n))
        .ok_or("boolean_union produced no identifiable editable body path")?;
    if !same_merge_frame(&body, &result) {
        return Err("boolean_union result changed the body coordinate frame or fill rule".into());
    }
    let consumed: HashSet<_> = operands.as_slice().iter().map(|(id, _)| id).collect();
    if elements(incoming.get_root_element().unwrap())
        .iter()
        .any(|node| {
            node.get_property_no_ns("id")
                .is_some_and(|id| consumed.contains(&id))
        })
    {
        return Err("boolean_union engine result retained a consumed operand".into());
    }
    let d = result
        .get_property_no_ns("d")
        .filter(|d| !d.trim().is_empty())
        .ok_or("boolean_union produced no body geometry")?;
    if !crate::geometry_quality::path(&d, 0.)
        .is_ok_and(|review| review.open_subpaths == 0 && review.segments > 2)
    {
        return Err("boolean_union produced no valid closed path geometry".into());
    }
    body.set_name("path")
        .map_err(|_| "cannot update union body")?;
    for key in SHAPE_GEOMETRY {
        if *key != "d" && body.get_property_no_ns(key).is_some() {
            body.remove_property_no_ns(key)
                .map_err(|_| "cannot replace union geometry")?;
        }
    }
    body.set_property("d", &d)
        .map_err(|_| "cannot write union geometry")?;
    for (_, mut operand) in operands {
        operand.unlink_node();
    }
    Ok(())
}

fn finish_outline(old: &Document, new: &mut Document, ids: &[String]) -> Result<(), String> {
    for id in ids {
        if let Some(mut node) = find(new, id)
            && node.get_name() == "path"
            && effective(&node, "fill").is_none()
        {
            let fill = find(old, id)
                .and_then(|n| effective(&n, "stroke"))
                .filter(|v| !v.trim().is_empty() && v.trim().to_lowercase() != "none")
                .map(|v| v.trim().to_owned())
                .unwrap_or_else(|| "#000000".into());
            style::write_property(&mut node, "fill", &fill)?;
        }
    }
    let before: HashSet<_> = elements(old.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    for mut n in elements(new.get_root_element().unwrap()) {
        if n.get_property_no_ns("id")
            .is_some_and(|id| before.contains(&id) || ids.contains(&id))
        {
            continue;
        }
        let empty = match n.get_name().as_str() {
            "g" => n.get_child_elements().is_empty(),
            "path" => n
                .get_property_no_ns("d")
                .is_none_or(|v| v.trim().is_empty()),
            _ => false,
        };
        if empty && n.get_parent().is_some_and(|n| n.is_element_node()) {
            // lxml remove also removes the target's tail.
            while let Some(mut tail) = n.get_next_sibling().filter(|n| {
                matches!(
                    n.get_type(),
                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                )
            }) {
                tail.unlink_node();
            }
            n.unlink_node();
        }
    }
    Ok(())
}
fn run(registry: &Registry, id: &str, op: &str, ids: &[String]) -> Result<Document, String> {
    run_actions(
        registry,
        id,
        &format!("select-by-id:{};{}", ids.join(","), action(op)),
        "path operation",
    )
}
pub(crate) fn run_actions(
    registry: &Registry,
    id: &str,
    actions: &str,
    label: &str,
) -> Result<Document, String> {
    let binary = process::inkscape_binary().ok_or("inkscape engine unavailable on this runtime; call list_capabilities to see what this runtime supports")?;
    let temp = tempfile::Builder::new()
        .prefix("imcp-path-")
        .tempdir()
        .map_err(|_| format!("{label} failed"))?;
    let stage = Workspace {
        roots: vec![
            temp.path()
                .canonicalize()
                .map_err(|_| format!("{label} failed"))?,
        ],
        max_input: registry
            .workspace
            .max_input
            .max(registry.workspace.max_output),
        max_output: registry.workspace.max_output,
    };
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    // Pin and copy the current working bytes before giving the owned engine an input path.
    let input =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let prepared = crate::engine_input::prepare(
        &registry.workspace,
        entry.root,
        Path::new(&entry.source),
        &input,
    )?;
    stage.write_new(0, Path::new("input.svg"), &prepared.bytes)?;
    let output = stage.roots[0].join("output.svg");
    if crate::engine::actions(
        &registry.workspace.roots[entry.root].join(entry.working()),
        &binary,
        &stage.roots[0].join("input.svg"),
        &output,
        actions,
    )
    .is_ok()
    {
        // Validate the bounded staged result before accepting the warm path.
        if let Ok(bytes) = stage.read(0, Path::new("output.svg"), registry.workspace.max_output)
            && !bytes.iter().all(u8::is_ascii_whitespace)
            && let Ok(document) = xml::parse(&bytes, registry.workspace.max_output)
        {
            prepared.restore(&document, false, Some(&stage.roots[0]))?;
            return Ok(document);
        }
    }
    let _ = stage.remove_file(0, Path::new("output.svg"));
    let outcome = process::run(&binary, &[
        stage.roots[0].join("input.svg").to_string_lossy().into_owned(),
        format!("--actions={actions}"),
        "--export-type=svg".into(), "--export-plain-svg".into(),
        format!("--export-filename={}",stage.roots[0].join("output.svg").display()),
    ], process::timeout()).map_err(|_| "inkscape engine unavailable on this runtime; call list_capabilities to see what this runtime supports")?;
    if outcome.timed_out {
        return Err(format!("{label} timed out"));
    }
    if !outcome.success {
        return Err(format!("{label} failed"));
    }
    let bytes = stage
        .read(0, Path::new("output.svg"), registry.workspace.max_output)
        .map_err(|error| {
            if error.contains("size limit") {
                format!("{label} output exceeds size limit")
            } else {
                format!("{label} failed")
            }
        })?;
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(format!("{label} produced empty output"));
    }
    let result = xml::parse(&bytes, registry.workspace.max_output)
        .map_err(|_| format!("{label} produced unsafe output"))?;
    prepared.restore(&result, false, Some(&stage.roots[0]))?;
    Ok(result)
}
pub(crate) fn validate_result(old: &Document, new: &Document) -> Result<(), String> {
    let root = new
        .get_root_element()
        .ok_or("path operation produced unsafe output")?;
    if root.get_name() != "svg"
        || root
            .get_namespace()
            .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
    {
        return Err("path operation produced a non-SVG root".into());
    }
    let before: HashSet<_> = elements(old.get_root_element().unwrap())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    let nodes = elements(root);
    let mut present = HashSet::new();
    for node in &nodes {
        if let Some(id) = node.get_property_no_ns("id")
            && !present.insert(id)
        {
            return Err("path operation produced duplicate SVG ids".into());
        }
    }
    if nodes
        .iter()
        .flat_map(crate::reparent::references)
        .any(|id| before.contains(&id) && !present.contains(&id))
    {
        return Err("path operation would leave a reference to a removed SVG id".into());
    }
    Ok(())
}
pub(crate) fn replace_root(
    old: &Document,
    incoming: &Document,
    max: usize,
) -> Result<Document, String> {
    // lxml keeps original comments/PIs but serializes the incoming root document
    // DTD. Inkscape plain SVG normally omits that DTD.
    let root = old.get_root_element().unwrap();
    let mut first = root.clone();
    while let Some(n) = first.get_prev_sibling() {
        first = n;
    }
    let mut serialized = String::from("<?xml version='1.0' encoding='UTF-8'?>\n");
    let mut incoming_previous = incoming.get_root_element().unwrap().get_prev_sibling();
    while let Some(n) = incoming_previous {
        incoming_previous = n.get_prev_sibling();
        if n.get_type() == Some(NodeType::DTDNode) {
            serialized.push_str(&incoming.node_to_string(&n));
            serialized.push('\n');
        }
    }
    let mut next = Some(first);
    while let Some(n) = next {
        next = n.get_next_sibling();
        if n.get_type() == Some(NodeType::DTDNode) {
            continue;
        }
        if n == root {
            serialized.push_str(&incoming.node_to_string(&incoming.get_root_element().unwrap()));
        } else {
            serialized.push_str(&old.node_to_string(&n));
        }
        if n.get_type() == Some(NodeType::DTDNode) {
            serialized.push('\n');
        }
    }
    xml::parse(serialized.as_bytes(), max).map_err(str::to_owned)
}
pub fn apply(registry: &Registry, op: &str, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let raw = args["object_ids"]
        .as_array()
        .ok_or("object_ids must be a list")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or("object_ids must contain strings")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if raw.len() > 4096 {
        return Err("path target list exceeds 4096 ids".into());
    }
    let dry = arguments::boolean(args, "dry_run", true)?;
    let approval = arguments::string(args, "approval_token")?;
    let mut result = json!({"doc_id":id,"op":op,"dry_run":dry,"changed":false,"affected_ids":[],"result_id":null,"summary":null,"operation_id":null,"snapshot_id":null,"preview_before":null,"preview_after":null});
    if dry {
        let document = placement::load(registry, id)?;
        let ids = targets(&document, &raw, op)?;
        let bottom = if op == "boolean_union" {
            union_operands(&document, &ids)?
                .into_iter()
                .next()
                .map(|(id, _)| id)
        } else {
            bottom(&document, &ids, op)
        };
        result["result_id"] = json!(bottom);
        result["summary"] = json!(format!(
            "dry-run: would apply {op} (Inkscape {}) to {} object(s): {} — no change written",
            action(op),
            ids.len(),
            ids.join(", ")
        ));
        result["affected_ids"] = json!(ids);
        return Ok(result);
    }
    if approval.is_none_or(str::is_empty) {
        return Err("high-risk path operation requires an explicit approval_token".into());
    }
    let mut captured = Vec::new();
    let mut survivor = None;
    let applied = transaction::apply(
        registry,
        id,
        op,
        json!({"object_ids":raw,"dry_run":false}),
        "high",
        approval,
        |document| {
            let ids = targets(document, &raw, op)?;
            captured = ids.clone();
            if op == "boolean_union" {
                let operands = union_operands(document, &ids)?;
                let engine_ids: Vec<_> = operands.iter().map(|(id, _)| id.clone()).collect();
                let new = run(registry, id, op, &engine_ids)?;
                validate_result(document, &new)?;
                apply_union_geometry(operands, &new)?;
                survivor = engine_ids.into_iter().next();
                return Ok(format!("applied {op} to {} objects", ids.len()));
            }
            let bottom = bottom(document, &ids, op);
            let mut new = run(registry, id, op, &ids)?;
            if let Some(bottom) = bottom {
                let existing: HashSet<_> = elements(new.get_root_element().unwrap())
                    .iter()
                    .filter_map(|n| n.get_property_no_ns("id"))
                    .collect();
                let found = if existing.contains(&bottom) {
                    Some(bottom.clone())
                } else {
                    ids.iter().find(|id| existing.contains(*id)).cloned()
                };
                if let Some(found) = found {
                    if found != bottom && !existing.contains(&bottom) {
                        find(&new, &found)
                            .unwrap()
                            .set_property("id", &bottom)
                            .map_err(|_| "path operation failed")?;
                        identity::rewrite_references(
                            &new,
                            &mut elements(new.get_root_element().unwrap()),
                            &HashMap::from([(found, bottom.clone())]),
                        )?;
                        survivor = Some(bottom);
                    } else {
                        survivor = Some(found);
                    }
                }
            }
            if op == "stroke_to_path" {
                finish_outline(document, &mut new, &ids)?;
            }
            validate_result(document, &new)?;
            *document = replace_root(document, &new, registry.workspace.max_input)?;
            Ok(format!(
                "applied {op} (Inkscape {}) to {} object(s): {}",
                action(op),
                ids.len(),
                ids.join(", ")
            ))
        },
    )?;
    for key in [
        "changed",
        "summary",
        "operation_id",
        "snapshot_id",
        "preview_before",
        "preview_after",
    ] {
        result[key] = applied[key].clone();
    }
    result["affected_ids"] = json!(captured);
    result["result_id"] = json!(survivor);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn action_grammar_distinct_order_and_bottom_prediction_are_bounded() {
        let doc = xml::parse(
            b"<svg xmlns='http://www.w3.org/2000/svg'><path id='a'/><path id='b'/></svg>",
            4096,
        )
        .unwrap();
        let ids = targets(&doc, &["b".into(), "a".into(), "b".into()], "combine_paths").unwrap();
        assert_eq!(ids, ["b", "a"]);
        assert_eq!(bottom(&doc, &ids, "combine_paths"), Some("a".into()));
        for id in ["a;path-union", "a:b", "a,b", "-option"] {
            assert!(targets(&doc, &[id.into()], "simplify_path").is_err());
        }
        assert!(targets(&doc, &["a".into(), "a".into()], "boolean_union").is_err());
        assert_eq!(action("cleanup_paths"), "path-simplify");
    }
    #[test]
    fn outlines_restore_stroke_color_and_remove_only_new_empty_stubs() {
        let old = xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg"><path id="a" style="stroke:green"/><g id="kept"/><path id="b" stroke="none"/></svg>"##,4096).unwrap();
        let mut new = xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg"><path id="a" d="M0 0 L1 1"/><g id="kept"/><path id="b" d="M0 0 L2 2"/><g id="stub"/><g id="marker"><path d="M1 1 L3 3"/></g></svg>"##,4096).unwrap();
        finish_outline(&old, &mut new, &["a".into(), "b".into()]).unwrap();
        assert_eq!(
            effective(&find(&new, "a").unwrap(), "fill"),
            Some("green".into())
        );
        assert_eq!(
            effective(&find(&new, "b").unwrap(), "fill"),
            Some("#000000".into())
        );
        assert!(find(&new, "stub").is_none());
        assert!(find(&new, "kept").is_some());
        assert!(find(&new, "marker").is_some());
    }
    #[test]
    fn unsafe_engine_structure_and_broken_references_refuse() {
        let old = xml::parse(b"<svg><path id='a'/><path id='b'/></svg>", 4096).unwrap();
        for source in [
            "<html/>",
            "<svg><path id='a'/><path id='a'/></svg>",
            "<svg><path id='a'/><use href='#b'/></svg>",
        ] {
            let new = xml::parse(source.as_bytes(), 4096).unwrap();
            assert!(validate_result(&old, &new).is_err());
        }
        assert!(find(&old, "b").is_some());
    }
    #[test]
    fn union_refuses_unsafe_engine_geometry_before_editing_the_tree() {
        let original = xml::parse(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><path id="body" fill-rule="evenodd" d="M0 0H4V4Z"/><circle id="other" cx="3" cy="3" r="2"/></svg>"#,
            4096,
        )
        .unwrap();
        let before = xml::serialize(&original);
        let ids = ["body".into(), "other".into()];
        for (result, reason) in [
            (
                r#"<path id="body" fill-rule="nonzero" d="M0 0H5V5Z"/>"#,
                "coordinate frame or fill rule",
            ),
            (
                r#"<path id="body" fill-rule="evenodd" d="M0 0H5V5"/>"#,
                "no valid closed path geometry",
            ),
            (
                r#"<path id="body" fill-rule="evenodd" d="M0 0H5V5Z"/><circle id="other"/>"#,
                "retained a consumed operand",
            ),
        ] {
            let incoming = xml::parse(
                format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{result}</svg>"#).as_bytes(),
                4096,
            )
            .unwrap();
            let operands = union_operands(&original, &ids).unwrap();
            let error = apply_union_geometry(operands, &incoming).unwrap_err();
            assert!(error.contains(reason), "{error}");
            assert_eq!(xml::serialize(&original), before);
        }
    }
    #[test]
    fn union_converts_a_primitive_and_keeps_its_descriptive_children() {
        let original = xml::parse(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="body" x="1" y="1" width="4" height="4" rx="1" fill="red"><title id="label">Union body</title></rect><circle id="other" cx="3" cy="3" r="2"/></svg>"#,
            4096,
        )
        .unwrap();
        let incoming = xml::parse(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><path id="body" d="M0 0H5V5Z"/></svg>"#,
            4096,
        )
        .unwrap();
        let operands = union_operands(&original, &["other".into(), "body".into()]).unwrap();
        apply_union_geometry(operands, &incoming).unwrap();
        let body = find(&original, "body").unwrap();
        assert_eq!(body.get_name(), "path");
        assert_eq!(body.get_property_no_ns("d").as_deref(), Some("M0 0H5V5Z"));
        assert_eq!(body.get_property_no_ns("fill").as_deref(), Some("red"));
        for attr in ["x", "y", "width", "height", "rx"] {
            assert!(body.get_property_no_ns(attr).is_none(), "{attr}");
        }
        assert_eq!(
            find(&original, "label").unwrap().get_content(),
            "Union body"
        );
        assert!(find(&original, "other").is_none());
    }
    #[test]
    fn root_replacement_preserves_original_document_siblings() {
        let old = xml::parse(b"<!DOCTYPE svg><!--before--><svg xmlns='http://www.w3.org/2000/svg'><path id='a'/></svg><?after ok?>",4096).unwrap();
        let new = xml::parse(
            b"<!--engine--><svg xmlns='http://www.w3.org/2000/svg'><path id='b'/></svg>",
            4096,
        )
        .unwrap();
        let result = replace_root(&old, &new, 4096).unwrap();
        let text = String::from_utf8(xml::serialize(&result)).unwrap();
        assert!(text.contains("<!--before-->"));
        assert!(!text.contains("<!DOCTYPE"));
        assert!(text.ends_with("<?after ok?>"));
        assert!(!text.contains("engine"));
        assert!(find(&result, "b").is_some());
        assert!(find(&old, "a").is_some());
        let incoming_dtd = xml::parse(b"<!DOCTYPE svg><svg><path id='c'/></svg>", 4096).unwrap();
        let with_dtd = replace_root(&old, &incoming_dtd, 4096).unwrap();
        assert!(
            String::from_utf8(xml::serialize(&with_dtd))
                .unwrap()
                .contains("<!DOCTYPE svg>\n<!--before-->")
        );
    }
}
