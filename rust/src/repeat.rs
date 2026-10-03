//! Named ordinary groups of bounded, root-space linked/copy repetitions.
use crate::{
    arguments, create,
    document::{Registry, elements},
    duplicate, identity, reparent as affine, repeat_plan, structure, style, transaction, xml,
};
use libxml::{
    bindings,
    tree::{Document, Node, NodeType},
};
use serde_json::{Value, json};
use std::collections::HashSet;
fn tails(node: &Node) -> Vec<Node> {
    let mut result = vec![];
    let mut next = node.get_next_sibling();
    while let Some(n) = next.filter(|n| {
        matches!(
            n.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode | NodeType::EntityRefNode)
        )
    }) {
        next = n.get_next_sibling();
        result.push(n);
    }
    result
}
fn matrix(m: [f64; 6]) -> String {
    format!(
        "matrix({})",
        m.into_iter()
            .map(affine::general)
            .collect::<Vec<_>>()
            .join(",")
    )
}
pub(crate) struct Mutation {
    object: String,
    group: String,
    label: String,
    mode: String,
    plan: Value,
    anchor: [f64; 2],
}
impl Mutation {
    pub(crate) fn build(args: &Value) -> Result<Self, String> {
        if let Some(value) = args.get("mode").and_then(Value::as_str)
            && !["linked", "copies"].contains(&value)
        {
            return Err(format!(
                "1 validation error for call[repeat_objects]\nmode\n  Input should be 'linked' or 'copies' [type=literal_error, input_value={}, input_type=str]\n    For further information visit https://errors.pydantic.dev/2.13/v/literal_error",
                style::python_repr(value)
            ));
        }
        let plan = repeat_plan::plan(args)?;
        let object = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?
            .to_owned();
        let group = args["group_id"]
            .as_str()
            .ok_or("group_id must be a string")?
            .to_owned();
        let label = arguments::string(args, "label")?
            .unwrap_or("Repeated objects")
            .to_owned();
        let mode = arguments::string(args, "mode")?
            .unwrap_or("linked")
            .to_owned();
        if !["linked", "copies"].contains(&mode.as_str()) {
            return Err("mode must be linked or copies".into());
        }
        if !create::valid_id(&group) || label.chars().count() > 1024 {
            return Err("invalid group ID or label".into());
        }
        identity::Mutation::build(&json!({"object_id":group,"label":label}))?;
        let mut anchor = [0.; 2];
        if let Some(point) = args.get("anchor").filter(|v| !v.is_null()) {
            if point
                .as_object()
                .is_none_or(|p| p.keys().any(|k| k != "x" && k != "y"))
            {
                return Err("invalid bounded placement/variation parameters".into());
            }
            for (i, k) in ["x", "y"].into_iter().enumerate() {
                anchor[i] = arguments::number(&point[k])
                    .filter(|n| n.is_finite())
                    .ok_or("invalid bounded placement/variation parameters")?;
            }
        }
        Ok(Self {
            object,
            group,
            label,
            mode,
            plan,
            anchor,
        })
    }
    pub(crate) fn mutate(&self, doc: &mut Document, cap: usize) -> Result<String, String> {
        let root = doc
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let nodes = elements(root.clone());
        let source = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some(&self.object))
            .cloned()
            .ok_or("object id not found in document")?;
        let parent = source
            .get_parent()
            .filter(Node::is_element_node)
            .ok_or("repeat source must be a graphical object with a parent")?;
        if ![
            "g", "rect", "circle", "ellipse", "path", "polygon", "polyline", "line", "text", "use",
        ]
        .contains(&source.get_name().as_str())
        {
            return Err("repeat source must be a graphical object with a parent".into());
        }
        if !["g", "svg"].contains(&parent.get_name().as_str()) {
            return Err("repeat source must be in a group/layer or document root".into());
        }
        structure::reject_stylesheets(doc)?;
        let mut ancestor = Some(source.clone());
        while let Some(n) = ancestor {
            if n.get_property_ns(
                "insensitive",
                "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
            )
            .as_deref()
                == Some("true")
            {
                return Err("repeat source belongs to a locked object/layer".into());
            }
            ancestor = n.get_parent().filter(Node::is_element_node);
        }
        let mut ids = nodes
            .iter()
            .filter_map(|n| n.get_property_no_ns("id"))
            .collect::<HashSet<_>>();
        let subtree = elements(source.clone());
        if subtree
            .iter()
            .any(|n| !affine::references(n).is_subset(&ids))
        {
            return Err("source contains unresolved references".into());
        }
        let source_ids = subtree
            .iter()
            .filter_map(|n| n.get_property_no_ns("id").filter(|id| !id.is_empty()))
            .collect::<Vec<_>>();
        if source_ids.len() != source_ids.iter().collect::<HashSet<_>>().len() {
            return Err("source contains duplicate IDs".into());
        }
        if ids.contains(&self.group) {
            return Err("repeat group ID already exists".into());
        }
        if subtree
            .iter()
            .any(|n| ["animate", "animateTransform", "set", "svg"].contains(&n.get_name().as_str()))
        {
            return Err("animated or nested viewport sources require preparation".into());
        }
        let count = self.plan.as_array().unwrap().len();
        let copied_tail = tails(&source)
            .iter()
            .map(|n| doc.node_to_string(n).len())
            .sum::<usize>();
        let estimate = if self.mode == "copies" {
            doc.node_to_string(&source).len()
                + copied_tail
                + 16 * source_ids.iter().collect::<HashSet<_>>().len()
                + 1024
        } else {
            1024
        };
        if estimate
            .checked_mul(count)
            .and_then(|n| n.checked_add(xml::serialize(doc).len()))
            .is_none_or(|n| n > cap)
        {
            return Err("repeat projected document exceeds input size limit".into());
        }
        if self.mode == "copies" {
            duplicate::validate_copy(doc, &source)?;
        }
        let parent_matrix = affine::composed(parent.clone())?;
        let inv_parent = affine::inverse(parent_matrix)?;
        let original = affine::parse(&source.get_property_no_ns("transform").unwrap_or_default())?;
        let [a, b, c, d, e, f] = affine::composed(source.clone())?;
        let [x, y] = self.anchor;
        let (ax, ay) = (a * x + c * y + e, b * x + d * y + f);
        let mut group = create::svg_node("g", &parent, doc)?;
        group
            .set_property("id", &self.group)
            .map_err(|_| "vector creation failed")?;
        let mut after = tails(&source).last().cloned().unwrap_or(source.clone());
        after
            .add_next_sibling(&mut group)
            .map_err(|_| "vector creation failed")?;
        identity::set_inkscape_new_svg(&mut group, doc, "label", &self.label)?;
        ids.insert(self.group.clone());
        for (i, item) in self.plan.as_array().unwrap().iter().enumerate() {
            let translation = affine::parse(&format!(
                "translate({},{}) rotate({}) scale({})",
                item["x"], item["y"], item["rotation_degrees"], item["scale"]
            ))?;
            let delta = affine::multiply(translation, [1., 0., 0., 1., -ax, -ay])?;
            let compensation =
                affine::multiply(affine::multiply(inv_parent, delta)?, parent_matrix)?;
            if self.mode == "copies" {
                let mut clone = duplicate::insert_copy(doc, &source, &group, None, &mut ids)?;
                let mut tail = tails(&clone);
                clone.unlink_node();
                for n in &mut tail {
                    n.unlink_node();
                }
                clone
                    .set_property(
                        "transform",
                        &matrix(affine::multiply(compensation, original)?),
                    )
                    .map_err(|_| "repeat transform failed")?;
                group
                    .add_child(&mut clone)
                    .map_err(|_| "repeat copy adoption failed")?;
                for n in &mut tail {
                    group
                        .add_child(n)
                        .map_err(|_| "repeat tail adoption failed")?;
                }
                // SAFETY: adopted clone is attached to this live disposable document.
                if unsafe {
                    bindings::xmlDOMWrapReconcileNamespaces(
                        std::ptr::null_mut(),
                        clone.node_ptr(),
                        1,
                    )
                } < 0
                {
                    return Err("subtree namespace reconciliation failed".into());
                }
            } else {
                let id = format!("{}-instance-{}", self.group, i + 1);
                if !ids.insert(id.clone()) {
                    return Err("generated linked instance ID conflicts with document".into());
                }
                let mut instance = create::svg_node("use", &group, doc)?;
                instance
                    .set_property("id", &id)
                    .map_err(|_| "vector creation failed")?;
                group
                    .add_child(&mut instance)
                    .map_err(|_| "vector creation failed")?;
                let href = format!("#{}", self.object);
                instance
                    .set_property("href", &href)
                    .map_err(|_| "vector creation failed")?;
                identity::set_namespaced(
                    &mut instance,
                    doc,
                    "http://www.w3.org/1999/xlink",
                    "href",
                    &href,
                )?;
                instance
                    .set_property("transform", &matrix(compensation))
                    .map_err(|_| "repeat transform failed")?;
            }
        }
        if xml::serialize(doc).len() > cap {
            return Err("repeated document exceeds the configured input size limit".into());
        }
        Ok(format!(
            "created {count} {} instances in named group {}",
            self.mode,
            style::python_repr(&self.group)
        ))
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let dry = arguments::boolean(args, "dry_run", true)?;
    let m = Mutation::build(args)?;
    let edit = if dry {
        let entry = registry.entries.get(id).ok_or("document id not found")?;
        let bytes =
            registry
                .workspace
                .read(entry.root, &entry.working(), registry.workspace.max_input)?;
        let mut doc = xml::parse(&bytes, registry.workspace.max_input)?;
        m.mutate(&mut doc, registry.workspace.max_input)?;
        Value::Null
    } else {
        transaction::apply_dom(
            registry,
            id,
            "repeat_objects",
            json!({"object_id":m.object,"mode":m.mode,"group_id":m.group,"count":m.plan.as_array().unwrap().len()}),
            "medium",
            None,
            |d| m.mutate(d, registry.workspace.max_input),
        )?
    };
    Ok(
        json!({"doc_id":id,"dry_run":dry,"mode":m.mode,"group_id":m.group,"plan":m.plan,"edit":edit}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detached_group_label_reconciles_default_and_existing_document_prefixes() {
        let args = json!({"object_id":"r","group_id":"repeat","placement":{"kind":"polyline",
            "points":[{"x":0,"y":0},{"x":10,"y":0}],"count":2}});
        for (source, prefix) in [
            (
                "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect id=\"r\"/></svg>",
                "ns1",
            ),
            (
                "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:i=\"http://www.inkscape.org/namespaces/inkscape\"><rect id=\"r\"/></svg>",
                "i",
            ),
            // Do not shadow the document's SVG prefix with an Inkscape label.
            (
                "<ns1:svg xmlns:ns1=\"http://www.w3.org/2000/svg\"><ns1:rect id=\"r\"/></ns1:svg>",
                "ns2",
            ),
        ] {
            let mut doc = xml::parse(source.as_bytes(), 4096).unwrap();
            Mutation::build(&args)
                .unwrap()
                .mutate(&mut doc, 4096)
                .unwrap();
            let bytes = xml::serialize(&doc);
            assert!(
                String::from_utf8_lossy(&bytes)
                    .contains(&format!("{prefix}:label=\"Repeated objects\""))
            );
            let checked = xml::parse(&bytes, 4096).unwrap();
            let group = elements(checked.get_root_element().unwrap())
                .into_iter()
                .find(|n| n.get_property_no_ns("id").as_deref() == Some("repeat"))
                .unwrap();
            assert_eq!(
                group.get_namespace().unwrap().get_href(),
                "http://www.w3.org/2000/svg"
            );
            assert_eq!(
                group
                    .get_property_ns("label", crate::document::INKSCAPE_NS)
                    .as_deref(),
                Some("Repeated objects")
            );
        }
    }
    #[test]
    fn maximum_linked_plan_and_projected_size_refusal_are_bounded() {
        let args = json!({"object_id":"r","group_id":"repeat","placement":{"kind":"rectangle","x":0,"y":0,"width":100,"height":100,"count":1024}});
        let m = Mutation::build(&args).unwrap();
        let mut d = xml::parse(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r" width="1" height="1"/></svg>"#,
            4096,
        )
        .unwrap();
        let before = xml::serialize(&d);
        assert_eq!(
            m.mutate(&mut d, 1024).unwrap_err(),
            "repeat projected document exceeds input size limit"
        );
        assert_eq!(before, xml::serialize(&d));
        m.mutate(&mut d, 2_000_000).unwrap();
        let group = elements(d.get_root_element().unwrap())
            .into_iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some("repeat"))
            .unwrap();
        assert_eq!(group.get_child_elements().len(), 1024);
        assert!(
            group
                .get_property_ns("groupmode", crate::document::INKSCAPE_NS)
                .is_none()
        );
        assert_eq!(
            group.get_child_elements()[1023]
                .get_property_no_ns("href")
                .as_deref(),
            Some("#r")
        );
    }
    #[test]
    fn locked_ancestor_and_unsupported_copy_references_refuse() {
        let args = json!({"object_id":"r","group_id":"repeat","placement":{"kind":"polyline","points":[{"x":0,"y":0},{"x":10,"y":0}],"count":2}});
        let mut d=xml::parse(br#"<svg xmlns:s="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"><g s:insensitive="true"><rect id="r"/></g></svg>"#,4096).unwrap();
        let before = xml::serialize(&d);
        assert_eq!(
            Mutation::build(&args)
                .unwrap()
                .mutate(&mut d, 4096)
                .unwrap_err(),
            "repeat source belongs to a locked object/layer"
        );
        assert_eq!(before, xml::serialize(&d));
        let mut d = xml::parse(
            br#"<svg><g id="r"><rect id="shape" aria-labelledby="shape"/></g></svg>"#,
            4096,
        )
        .unwrap();
        let before = xml::serialize(&d);
        let mut copies = args;
        copies["mode"] = json!("copies");
        assert!(
            Mutation::build(&copies)
                .unwrap()
                .mutate(&mut d, 4096)
                .unwrap_err()
                .contains("accessibility")
        );
        assert_eq!(before, xml::serialize(&d));
    }
}
