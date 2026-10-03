//! Web, icon and print exports compose the protected render pipeline with fixed typed options.
use crate::{arguments, document::Registry, render};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn integers(args: &Value, key: &str) -> Result<Option<Vec<i64>>, String> {
    if args.get(key).is_none_or(Value::is_null) {
        return Ok(None);
    }
    let list = args[key]
        .as_array()
        .ok_or_else(|| format!("{key} must be a list"))?;
    if list.len() > 32 {
        return Err("export profile exceeds the item cap (32)".into());
    }
    list.iter()
        .map(|v| {
            render::integer(&json!({"value":v}), "value")?
                .ok_or_else(|| format!("{key} must contain integers"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
fn prefix(caller: Option<&str>, key: &str) -> String {
    caller
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s}-{key}"))
        .unwrap_or(key.into())
}

fn inner(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let out = arguments::string(args, "out_dir")?;
    let caller = arguments::string(args, "name_prefix")?;
    let mut requests: Vec<(i64, Option<i64>, Option<i64>)> = Vec::new();
    let profile = match tool {
        "create_icon_set" => {
            let sizes = integers(args, "sizes")?.unwrap_or(vec![16, 32, 48, 64, 128, 256]);
            for size in sizes {
                if size <= 0 {
                    return Err("icon size must be a positive integer".into());
                }
                if size > i64::from(render::cap()) {
                    return Err("icon size exceeds the configured pixel cap".into());
                }
                requests.push((size, None, Some(size)));
            }
            "icon"
        }
        "export_web_profile" => {
            let widths = integers(args, "widths")?;
            let scales = integers(args, "scales")?;
            let base = render::integer(args, "width_px")?.unwrap_or(1024);
            let mut resolved = BTreeMap::new();
            if let Some(widths) = widths {
                if widths.is_empty() {
                    return Err("web profile requires at least one width".into());
                }
                for width in widths {
                    if width <= 0 {
                        return Err("web width must be a positive integer".into());
                    }
                    resolved.insert(width, None);
                }
            } else if let Some(scales) = scales {
                if scales.is_empty() {
                    return Err("web profile requires at least one scale".into());
                }
                if base <= 0 {
                    return Err("web width must be a positive integer".into());
                }
                for scale in scales {
                    if scale <= 0 {
                        return Err("web scale must be a positive integer".into());
                    }
                    resolved.insert(
                        base.checked_mul(scale)
                            .ok_or("export exceeds the configured size or dimension limit")?,
                        Some(scale),
                    );
                }
            } else {
                if base <= 0 {
                    return Err("web width must be a positive integer".into());
                }
                resolved.insert(base, None);
            }
            requests.extend(
                resolved
                    .into_iter()
                    .map(|(width, scale)| (width, scale, None)),
            );
            "web"
        }
        "export_print_profile" => "print",
        _ => return Err("unknown export profile".into()),
    };
    let mut artifacts = Vec::new();
    for (width, scale, size) in requests {
        let key = if profile == "icon" {
            format!("icon-{width}")
        } else {
            format!("web-{width}w")
        };
        let result = render::call(
            registry,
            "export_document",
            &json!({"doc_id":id,"format":"png","width_px":width,"name_prefix":prefix(caller,&key),"out_dir":out,"inline":false}),
        )?;
        artifacts.push(artifact(
            &result["structuredContent"],
            size,
            scale,
            if profile == "web" { Some(width) } else { None },
        ));
    }
    if profile != "icon" {
        let params = json!({"doc_id":id,"format":if profile=="print" {"pdf"}else{"svg"},"out_dir":out,"name_prefix":prefix(caller,profile),"inline":false});
        let result = if profile == "print" {
            render::print_profile(registry, &params)
        } else {
            render::call(registry, "export_document", &params)
        }?;
        artifacts.push(artifact(&result["structuredContent"], None, None, None));
    }
    let settings = if profile == "print" {
        json!({"pdf_version":"1.4","text_to_path":"true"})
    } else {
        json!({})
    };
    Ok(json!({"doc_id":id,"profile":profile,"artifacts":artifacts,"applied_settings":settings}))
}
fn artifact(result: &Value, size: Option<i64>, scale: Option<i64>, width: Option<i64>) -> Value {
    let mut value = json!({"path":result["artifact_path"],"workspace_relative_path":result["workspace_relative_path"],"artifact":result["artifact"],"requested_size_px":size,"scale":scale,"requested_width_px":width});
    for key in [
        "format",
        "width_px",
        "height_px",
        "opaque_px",
        "all_blank",
        "is_vector",
        "fonts_outlined",
    ] {
        value[key] = result[key].clone();
    }
    value
}

pub fn call(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    inner(registry, tool, args).map_err(|error| {
        if error.starts_with("path rejected: outside workspace") {
            "path rejected: outside workspace".into()
        } else {
            error
        }
    })
}
