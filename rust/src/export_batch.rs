//! Typed bounded export batches, with complete validation and projection before rendering.
use crate::{arguments, document::Registry, inspect, render};
use serde_json::{Value, json};
use std::{collections::HashSet, path::PathBuf};

pub fn call(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let specs = args["specs"].as_array().ok_or("specs must be a list")?;
    if specs.is_empty() {
        return Err("export batch requires at least one spec".into());
    }
    if specs.len() > 32 {
        return Err("export batch exceeds the item cap (32)".into());
    }
    let mut typed = Vec::new();
    for spec in specs {
        if !spec.is_object() {
            return Err("export spec must be an object".into());
        }
        let format = arguments::string(spec, "format")?
            .ok_or("format must be a string")?
            .trim()
            .to_lowercase();
        typed.push((
            format,
            render::integer(spec, "width_px")?,
            arguments::string(spec, "object_id")?,
        ));
    }
    let dry_run = arguments::boolean(args, "dry_run", true)?;
    let configured = std::env::var("INKSCAPE_MCP_ARTIFACT_MAX_BYTES_PER_DOC")
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .filter(|n| *n >= 0)
        .unwrap_or(512 * 1024 * 1024);
    let budget = render::integer(args, "byte_budget")?.unwrap_or(configured);
    if budget <= 0 {
        return Err("byte budget must be a positive integer".into());
    }
    let budget = budget.min(configured) as u64;
    let out = arguments::string(args, "out_dir")?;
    let prefix = arguments::string(args, "name_prefix")?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    if let Some(out) = out {
        let path = PathBuf::from(out);
        let directory = if path.is_absolute() {
            path
        } else {
            registry.workspace.roots[entry.root].join(path)
        };
        // Match the reference's safe directory validation (including empty-directory creation
        // on dry runs with an explicit out_dir), without creating an export artifact.
        registry
            .workspace
            .resolve_output(
                directory
                    .join(".export-batch-probe")
                    .to_str()
                    .ok_or("path rejected: invalid characters")?,
                None,
            )
            .map_err(|error| {
                if error.starts_with("path rejected: outside workspace") {
                    "path rejected: outside workspace".to_owned()
                } else {
                    error
                }
            })?;
    }
    let summary = registry.summary(id)?;
    let working_size = registry
        .workspace
        .read(entry.root, &entry.working(), registry.workspace.max_input)?
        .len() as u64;
    let ids = if typed.iter().any(|s| s.2.is_some()) {
        inspect::resource(registry, id, "objects")?["objects"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v["id"].as_str().map(str::to_owned))
            .collect::<HashSet<_>>()
    } else {
        HashSet::new()
    };
    let mut items = Vec::new();
    let mut projected_total = 0u64;
    for (index, (format, width, object)) in typed.iter().enumerate() {
        if !["png", "pdf", "svg"].contains(&format.as_str()) {
            return Err(format!(
                "spec {index}: unsupported format (expected one of: png, pdf, svg); call list_capabilities to see the supported export formats"
            ));
        }
        if let Some(w) = width {
            if *w <= 0 {
                return Err(format!("spec {index}: width_px must be a positive integer"));
            }
            if *w > i64::from(render::cap()) {
                return Err(format!(
                    "spec {index}: width_px exceeds the configured pixel cap"
                ));
            }
        }
        if let Some(object) = object {
            if !render::safe_object_id(object) {
                return Err(format!("spec {index}: object id is not valid"));
            }
            if !ids.contains(*object) {
                return Err(format!("spec {index}: object id not found in document"));
            }
        }
        let projected = if format == "png" {
            let (w, h) = render::estimated_dimensions(&summary, *width);
            (w as u64)
                .checked_mul(h as u64)
                .and_then(|n| n.checked_mul(4))
                .ok_or("export projection exceeds the supported range")?
        } else {
            working_size.max(4096)
        };
        projected_total = projected_total
            .checked_add(projected)
            .ok_or("export projection exceeds the supported range")?;
        items.push(json!({"index":index,"format":format,"width_px":width,"object_id":object,"projected_bytes":projected,"artifact_path":null,"artifact":null,"workspace_relative_path":null,"status":"planned"}));
    }
    let within = projected_total <= budget;
    if !dry_run && !within {
        return Err(format!(
            "export batch projected output exceeds the byte budget ({projected_total} > {budget} bytes)"
        ));
    }
    let mut actual_total = 0u64;
    if !dry_run {
        for item in &mut items {
            let export_args = json!({"doc_id":id,"format":item["format"],"width_px":item["width_px"],"object_id":item["object_id"],"out_dir":out,"name_prefix":prefix,"inline":false});
            let result = render::call(
                registry,
                if item["object_id"].is_null() {
                    "export_document"
                } else {
                    "export_object"
                },
                &export_args,
            )?;
            let output = &result["structuredContent"];
            let uri = output["artifact"]["uri"]
                .as_str()
                .ok_or("export artifact unavailable")?;
            let (root, token) = uri
                .strip_prefix("inkscape://artifact/")
                .and_then(|s| s.split_once('/'))
                .ok_or("export artifact unavailable")?;
            let bytes = registry.workspace.read_artifact(root, token)?;
            actual_total = actual_total
                .checked_add(bytes.len() as u64)
                .ok_or("export output exceeds the supported range")?;
            if actual_total > budget {
                return Err(format!(
                    "export batch output exceeded the byte budget mid-run ({actual_total} > {budget} bytes)"
                ));
            }
            for key in ["artifact_path", "artifact", "workspace_relative_path"] {
                item[key] = output[key].clone();
            }
            item["status"] = json!("exported");
        }
    }
    Ok(
        json!({"doc_id":id,"dry_run":dry_run,"item_count":items.len(),"byte_budget":budget,"projected_total_bytes":projected_total,"actual_total_bytes":if dry_run {None}else{Some(actual_total)},"within_budget":within,"items":items}),
    )
}
