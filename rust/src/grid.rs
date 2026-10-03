//! Compose different assets into one reversible document using shared placement copies.
use crate::{
    arguments, create,
    document::{Registry, elements},
    duplicate,
    grid_plan::Plan,
    placement, structure, style, transaction, xml,
};
use serde_json::{Value, json};
use std::collections::HashSet;
fn list<'a>(args: &'a Value, key: &str) -> Result<Vec<&'a str>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(vec![]),
        Some(v) => {
            let values = v.as_array().ok_or(format!("{key} must be a list"))?;
            if values.len() > 1024 {
                return Err("compose_grid source list exceeds 1024 assets".into());
            }
            values
                .iter()
                .map(|v| v.as_str().ok_or(format!("{key} must contain strings")))
                .collect()
        }
    }
}
pub fn apply(registry: &mut Registry, args: &Value) -> Result<Value, String> {
    let doc_ids = list(args, "doc_ids")?;
    let object_ids = list(args, "object_ids")?;
    let source = arguments::string(args, "source_doc_id")?;
    if doc_ids.is_empty() == object_ids.is_empty() {
        return Err(
            "compose_grid requires EXACTLY ONE of doc_ids OR object_ids (with source_doc_id)"
                .into(),
        );
    }
    if !doc_ids.is_empty() && source.is_some() {
        return Err("source_doc_id is only valid with object_ids, not doc_ids".into());
    }
    if !object_ids.is_empty() && source.is_none() {
        return Err("object_ids requires source_doc_id (the document the objects live in)".into());
    }
    let mut documents = vec![];
    let mut assets = vec![];
    let mut total = 0usize;
    if doc_ids.is_empty() {
        let document = placement::load(registry, source.unwrap())?;
        let nodes = elements(document.get_root_element().unwrap());
        for object in &object_ids {
            let node = nodes
                .iter()
                .find(|n| n.get_property_no_ns("id").as_deref() == Some(*object))
                .cloned()
                .ok_or("object id not found in document")?;
            assets.push((node, (*object).to_owned()));
        }
        documents.push(document);
    } else {
        for doc in &doc_ids {
            let document = placement::load(registry, doc)?;
            total = total
                .checked_add(xml::serialize(&document).len())
                .ok_or("grid sources exceed the configured size limit")?;
            if total > registry.workspace.max_input {
                return Err("grid sources exceed the configured size limit".into());
            }
            assets.push((document.get_root_element().unwrap(), (*doc).to_owned()));
            documents.push(document);
        }
    }
    let plan = Plan::build(args, assets.len())?;
    for (i, (node, _)) in assets.iter().enumerate() {
        placement::guard(&documents[if doc_ids.is_empty() { 0 } else { i }], node)?;
    }
    let target = if let Some(target) = arguments::string(args, "target_doc_id")? {
        let doc = placement::load(registry, target)?;
        structure::reject_stylesheets(&doc)?;
        if doc
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
        target.to_owned()
    } else {
        let [width, height] = plan.canvas();
        registry.create(&json!({"width":width,"height":height}))?["doc_id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let mut cells = vec![];
    let mut result = transaction::apply_dom(
        registry,
        &target,
        "compose_grid",
        json!({"rows":plan.rows,"cols":plan.cols,"cell":plan.cell,"assets":assets.len()}),
        "medium",
        None,
        |document| {
            let mut root = document.get_root_element().unwrap();
            let mut existing: HashSet<_> = elements(root.clone())
                .iter()
                .filter_map(|n| n.get_property_no_ns("id"))
                .collect();
            for (i, (source, label)) in assets.iter().enumerate() {
                let mut wrapper = create::svg_node("g", &root, document)?;
                root.add_child(&mut wrapper)
                    .map_err(|_| "grid wrapper creation failed")?;
                let (clone, top) =
                    duplicate::append_asset(document, source, &mut wrapper, &mut existing)?;
                let id = format!("cell-{}-{}-{top}", i / plan.cols, i % plan.cols);
                if !existing.insert(id.clone()) {
                    return Err("grid wrapper ID already in use".into());
                }
                wrapper
                    .set_property("id", &id)
                    .map_err(|_| "grid identity failed")?;
                let [x, y] = plan.origin(i);
                let factor = plan.factor(clone);
                let mut transform = format!(
                    "translate({},{})",
                    style::format_num(x)?,
                    style::format_num(y)?
                );
                if factor < 1. {
                    transform.push_str(&format!(" scale({})", style::format_num(factor)?));
                }
                wrapper
                    .set_property("transform", &transform)
                    .map_err(|_| "grid transform failed")?;
                cells.push(
                    json!({"row":i/plan.cols,"col":i%plan.cols,"group_id":id,"source":label}),
                );
            }
            Ok(format!(
                "composed {} asset(s) into a {}x{} grid",
                assets.len(),
                plan.rows,
                plan.cols
            ))
        },
    )?;
    result["target_doc_id"] = json!(target);
    result["rows"] = json!(plan.rows);
    result["cols"] = json!(plan.cols);
    result["cells"] = json!(cells);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_caps_and_invalid_plan_precede_new_document_and_history() {
        let dir = tempfile::tempdir().unwrap();
        let source=b"<svg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 10 10'><rect id='r' width='10' height='10'/></svg>";
        std::fs::write(dir.path().join("source.svg"), source).unwrap();
        let mut registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![dir.path().canonicalize().unwrap()],
                max_input: 256,
                max_output: 4096,
            },
            entries: indexmap::IndexMap::new(),
        };
        let opened = registry.open(&json!({"path":"source.svg"})).unwrap();
        let id = opened["doc_id"].as_str().unwrap();
        assert_eq!(
            apply(
                &mut registry,
                &json!({"rows":1,"cols":2,"cell":20,"doc_ids":[id,id]})
            )
            .unwrap_err(),
            "grid sources exceed the configured size limit"
        );
        assert_eq!(
            apply(
                &mut registry,
                &json!({"rows":0,"cols":1,"cell":20,"doc_ids":[id]})
            )
            .unwrap_err(),
            "compose_grid rows and cols must each be >= 1"
        );
        assert!(
            apply(
                &mut registry,
                &json!({"rows":1,"cols":1,"cell":20,"doc_ids":vec![id;1025]})
            )
            .unwrap_err()
            .contains("1024 assets")
        );
        assert_eq!(registry.entries.len(), 1);
        let entry = &registry.entries[id];
        assert_eq!(
            std::fs::read(dir.path().join(entry.working())).unwrap(),
            source
        );
        for name in ["snapshots", "operations"] {
            assert_eq!(
                std::fs::read_dir(dir.path().join(entry.directory()).join(name))
                    .unwrap()
                    .count(),
                0
            );
        }
    }
}
