//! Cross-document placement preserves source bytes and uses the common clone/transaction kernel.
use crate::{
    arguments, create,
    document::{Registry, elements},
    duplicate, reparent, structure, style, transaction, xml,
};
use libxml::tree::{Document, Node, NodeType};
use serde_json::{Value, json};
use std::collections::HashSet;
pub(crate) fn load(registry: &Registry, id: &str) -> Result<Document, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    xml::parse(&bytes, registry.workspace.max_input).map_err(str::to_owned)
}
fn number(args: &Value, key: &str, default: Option<f64>) -> Result<f64, String> {
    match args.get(key) {
        Some(v) => arguments::number(v).ok_or(format!("{key} must be a number")),
        None => default.ok_or(format!("{key} must be a number")),
    }
}
pub(crate) fn guard(source_document: &Document, source: &Node) -> Result<(), String> {
    duplicate::validate_copy(source_document, source)?;
    let nodes = elements(source.clone());
    if source.get_name() != "svg"
        && nodes.iter().any(|n| {
            n.get_properties().iter().any(|(key, value)| {
                crate::optimization_analysis::coordinate(key) && value.contains('%')
            })
        })
    {
        return Err("placement of viewport-relative geometry requires preparation".into());
    }
    let ids: HashSet<_> = nodes
        .iter()
        .filter_map(|n| n.get_property_no_ns("id"))
        .collect();
    for n in &nodes {
        if !reparent::references(n).is_subset(&ids) {
            return Err(
                "placement requires referenced definitions inside the source subtree".into(),
            );
        }
        for key in ["href", "xlink:href"] {
            if n.get_property_no_ns(key)
                .is_some_and(|v| !v.is_empty() && !v.starts_with('#') && !v.starts_with("data:"))
            {
                return Err("placement of external assets requires preparation".into());
            }
        }
        if n.get_property_ns("href", "http://www.w3.org/1999/xlink")
            .is_some_and(|v| !v.is_empty() && !v.starts_with('#') && !v.starts_with("data:"))
        {
            return Err("placement of external assets requires preparation".into());
        }
        if n.get_name() == "script"
            || n.get_child_nodes()
                .iter()
                .any(|n| n.get_type() == Some(NodeType::EntityRefNode))
        {
            return Err(
                "placement of active or entity-dependent content requires preparation".into(),
            );
        }
    }
    let mut ancestor = source.get_parent().filter(|n| n.is_element_node());
    while let Some(parent) = ancestor {
        if parent.get_properties().keys().any(|k| {
            !matches!(
                k.as_str(),
                "id" | "width" | "height" | "viewBox" | "version"
            ) && !k.contains(':')
        }) {
            return Err("placement of inherited container properties requires preparation".into());
        }
        ancestor = parent.get_parent().filter(|n| n.is_element_node());
    }
    Ok(())
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let target = args["target_doc_id"]
        .as_str()
        .ok_or("target_doc_id must be a string")?;
    let source = arguments::string(args, "source_doc_id")?
        .filter(|s| !s.is_empty())
        .ok_or("place_document requires source_doc_id (the document to place from)")?;
    let source_document = load(registry, source)?;
    let root = source_document.get_root_element().unwrap();
    let object = arguments::string(args, "object_id")?;
    let source_node = if let Some(object) = object {
        elements(root)
            .into_iter()
            .find(|n| n.get_property_no_ns("id").as_deref() == Some(object))
            .ok_or("object id not found in document")?
    } else {
        root
    };
    let label = object.unwrap_or(source);
    let target_document = load(registry, target)?;
    let x = number(args, "x", None)?;
    let y = number(args, "y", None)?;
    let scale = number(args, "scale", Some(1.))?;
    for (name, value) in [("x", x), ("y", y)] {
        if !value.is_finite() {
            return Err(format!("{name} must be finite"));
        }
    }
    if !scale.is_finite() || scale <= 0. {
        return Err("place_document scale must be a finite number greater than 0".into());
    }
    guard(&source_document, &source_node)?;
    structure::reject_stylesheets(&target_document)?;
    if target_document
        .get_root_element()
        .unwrap()
        .get_properties()
        .keys()
        .any(|k| {
            !matches!(
                k.as_str(),
                "id" | "width" | "height" | "viewBox" | "version"
            ) && !k.contains(':')
        })
    {
        return Err("placement into inherited root properties requires preparation".into());
    }
    let mut placed = String::new();
    let mut result = transaction::apply_dom(
        registry,
        target,
        "place_document",
        json!({"source":label,"x":x,"y":y,"scale":scale}),
        "medium",
        None,
        |document| {
            let mut root = document.get_root_element().unwrap();
            let mut existing: HashSet<_> = elements(root.clone())
                .iter()
                .filter_map(|n| n.get_property_no_ns("id"))
                .collect();
            let mut wrapper = create::svg_node("g", &root, document)?;
            root.add_child(&mut wrapper)
                .map_err(|_| "placement wrapper failed")?;
            let (_, top) =
                duplicate::append_asset(document, &source_node, &mut wrapper, &mut existing)?;
            let base = format!("placed-{top}");
            let mut id = base.clone();
            let mut attempt = 1;
            while existing.contains(&id) {
                if attempt > 4096 {
                    return Err("unable to allocate a placement id".into());
                }
                id = format!("{base}-{attempt}");
                attempt += 1;
            }
            wrapper
                .set_property("id", &id)
                .map_err(|_| "placement identity failed")?;
            let translate = format!(
                "translate({},{})",
                style::format_num(x)?,
                style::format_num(y)?
            );
            let transform = if scale != 1. {
                format!("{translate} scale({})", style::format_num(scale)?)
            } else {
                translate
            };
            wrapper
                .set_property("transform", &transform)
                .map_err(|_| "placement transform failed")?;
            placed = id.clone();
            Ok(format!(
                "placed {} as {} at ({},{}) scale {}",
                style::python_repr(label),
                style::python_repr(&id),
                style::format_num(x)?,
                style::format_num(y)?,
                style::format_num(scale)?
            ))
        },
    )?;
    result["target_doc_id"] = json!(target);
    result["placed_id"] = json!(placed);
    result["source"] = json!(label);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracted_object_requires_self_contained_references_and_neutral_ancestry() {
        for source in [
            "<svg xmlns='http://www.w3.org/2000/svg'><defs><linearGradient id='p'/></defs><rect id='r' fill='url(#p)'/></svg>",
            "<svg xmlns='http://www.w3.org/2000/svg'><g fill='red'><rect id='r'/></g></svg>",
            "<svg xmlns='http://www.w3.org/2000/svg'><rect id='r' width='50%'/></svg>",
        ] {
            let doc = xml::parse(source.as_bytes(), 4096).unwrap();
            let root = doc.get_root_element().unwrap();
            let node = elements(root)
                .into_iter()
                .find(|n| n.get_property_no_ns("id").as_deref() == Some("r"))
                .unwrap();
            assert!(guard(&doc, &node).is_err());
        }
    }
    #[test]
    fn append_asset_remints_rootless_copy_and_rewrites_contained_refs() {
        let source = xml::parse(
            br##"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r"/><use href="#r"/></svg>"##,
            4096,
        )
        .unwrap();
        let target =
            xml::parse(b"<svg xmlns='http://www.w3.org/2000/svg'><g/></svg>", 4096).unwrap();
        let mut parent = target.get_root_element().unwrap().get_child_elements()[0].clone();
        let (node, id) = duplicate::append_asset(
            &target,
            &source.get_root_element().unwrap(),
            &mut parent,
            &mut HashSet::new(),
        )
        .unwrap();
        assert!(id.starts_with("copy-"));
        let children = node.get_child_elements();
        assert_eq!(
            children[1].get_property_no_ns("href"),
            Some(format!(
                "#{}",
                children[0].get_property_no_ns("id").unwrap()
            ))
        );
        assert_eq!(
            source.get_root_element().unwrap().get_child_elements()[0]
                .get_property_no_ns("id")
                .as_deref(),
            Some("r")
        );
    }
}
