//! Bounded selector fan-out through the existing read-only matcher and atomic batch kernel.
use crate::{arguments, batch, document::Registry, find};
use serde_json::{Value, json};

fn operation(raw: &Value) -> Result<(Value, bool), String> {
    let name = raw["op"].as_str().ok_or("operation requires an op field")?;
    let (strings, numbers, required, single): (&[&str], &[&str], &[&str], bool) = match name {
        "set_fill" => (&["color"], &["opacity"], &["color"], false),
        "set_stroke" => (&["color", "width"], &["opacity"], &[], false),
        "set_opacity" => (&[], &["opacity"], &["opacity"], false),
        "set_font" => (&["family", "size", "weight"], &[], &[], false),
        "delete_object" => (&[], &[], &[], false),
        "move_object" => (&[], &["dx", "dy"], &["dx", "dy"], true),
        "scale_object" => (&[], &["sx", "sy"], &["sx"], true),
        "rotate_object" => (&[], &["degrees", "cx", "cy"], &["degrees"], true),
        _ => return Err("operation is outside the targeted operation set".into()),
    };
    let mut result = json!({"op":name});
    for key in strings {
        let value = arguments::string(raw, key)?;
        if required.contains(key) && value.is_none() {
            return Err(format!("operation requires {key}"));
        }
        result[key] = json!(value);
    }
    for key in numbers {
        let value = arguments::optional_number(raw, key)?;
        if required.contains(key) && value.is_none() {
            return Err(format!("operation requires {key}"));
        }
        // Preserve the original JSON token for nonfinite strings so the common
        // typed mutation builder refuses it rather than treating it as absent.
        result[key] = match value {
            Some(v) if !v.is_finite() => raw[key].clone(),
            _ => json!(value),
        };
    }
    Ok((result, single))
}

pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let max = match args.get("max_matches") {
        None => 64.,
        Some(v) => arguments::number(v).ok_or("max_matches must be an integer")?,
    };
    if !max.is_finite() || max.fract() != 0. {
        return Err("max_matches must be an integer".into());
    }
    if max < 1. {
        return Err("max_matches must be at least 1".into());
    }
    let selector = args["selector"]
        .as_object()
        .ok_or("selector must be an object")?;
    let (op, single) = operation(&args["operation"])?;
    let name = op["op"].as_str().unwrap();
    let risk = if name == "delete_object" {
        "high"
    } else {
        "medium"
    };
    let mut query = Value::Object(selector.clone());
    query["doc_id"] = json!(id);
    let found = find::apply(registry, &query)?;
    let objects = found["objects"].as_array().unwrap();
    let ids: Vec<_> = objects.iter().map(|o| o["object_id"].clone()).collect();
    if ids.len() as f64 > max {
        return Err(format!(
            "selector matched {} objects, exceeding max_matches={max:.0}; narrow the selector or raise max_matches",
            ids.len()
        ));
    }
    let mut edits = Vec::new();
    let mut plan = Vec::new();
    if !ids.is_empty() {
        if single {
            for id in &ids {
                let mut edit = op.clone();
                edit["object_id"] = id.clone();
                edits.push(edit);
                plan.push(json!({"op":name,"object_ids":[id]}));
            }
        } else {
            let mut edit = op.clone();
            edit["object_ids"] = json!(ids);
            edits.push(edit);
            plan.push(json!({"op":name,"object_ids":ids}));
        }
    }
    let dry = arguments::boolean(args, "dry_run", true)?;
    let mut result = json!({"doc_id":id,"matched_ids":ids,"match_count":ids.len(),
        "risk_class":risk,"dry_run":dry,"plan":plan,"applied":false,"changed":false,
        "operation_id":"","snapshot_id":"","summary":null});
    if dry {
        return Ok(result);
    }
    if edits.is_empty() {
        result["summary"] = json!("no change: selector matched no objects");
        return Ok(result);
    }
    let applied = batch::apply_named(
        registry,
        &json!({"doc_id":id,"edits":edits,"approval_token":args.get("approval_token")}),
        "transform_objects",
        Some(ids.len()),
    )?;
    for key in ["changed", "operation_id", "snapshot_id", "summary"] {
        result[key] = applied[key].clone();
    }
    result["applied"] = applied["changed"].clone();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broad_selector_and_batch_cap_preserve_bytes_and_history() {
        let dir = tempfile::tempdir().unwrap();
        let mut source = String::from("<svg xmlns='http://www.w3.org/2000/svg'>");
        for i in 0..65 {
            source.push_str(&format!("<rect id='r{i}' width='1' height='1'/>"));
        }
        source.push_str("</svg>");
        std::fs::write(dir.path().join("source.svg"), &source).unwrap();
        let mut registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![dir.path().canonicalize().unwrap()],
                max_input: 16384,
                max_output: 16384,
            },
            entries: indexmap::IndexMap::new(),
        };
        let opened = registry.open(&json!({"path":"source.svg"})).unwrap();
        let id = opened["doc_id"].as_str().unwrap();
        let mut args = json!({"doc_id":id,"selector":{"tag":"rect"},
            "operation":{"op":"move_object","dx":1,"dy":2}});
        assert!(apply(&registry, &args).unwrap_err().contains("matched 65"));
        args["max_matches"] = json!(65);
        let dry = apply(&registry, &args).unwrap();
        assert_eq!(dry["plan"].as_array().unwrap().len(), 65);
        assert_eq!(dry["matched_ids"].as_array().unwrap().len(), 65);
        args["dry_run"] = json!(false);
        assert_eq!(
            apply(&registry, &args).unwrap_err(),
            "apply_edits batch exceeds 64 edits"
        );
        args["operation"] = json!({"op":"create_rect"});
        assert!(apply(&registry, &args).is_err());
        let entry = &registry.entries[id];
        assert_eq!(
            std::fs::read(dir.path().join(entry.working())).unwrap(),
            source.as_bytes()
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
