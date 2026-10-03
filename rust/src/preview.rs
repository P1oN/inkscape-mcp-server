//! Operation-linked previews generated in a private staging directory, then securely adopted.

use crate::{document::Entry, process, workspace::Workspace, xml};
use std::path::Path;

pub fn operation_preview(
    workspace: &Workspace,
    entry: &Entry,
    operation: &str,
    phase: &str,
) -> Option<String> {
    match operation_preview_inner(workspace, entry, operation, phase) {
        Ok(path) => Some(path),
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::json!({"event":"operation_preview_failed","error":error})
            );
            None
        }
    }
}

fn operation_preview_inner(
    workspace: &Workspace,
    entry: &Entry,
    operation: &str,
    phase: &str,
) -> Result<String, String> {
    let binary = process::inkscape_binary().ok_or("Inkscape is unavailable")?;
    let bytes = workspace.read(entry.root, &entry.working(), workspace.max_input)?;
    let document = xml::parse(&bytes, workspace.max_input)?;
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    ensure_raster_bounds(&root)?;
    let input_bytes = crate::render::input(workspace, entry)?;
    let temporary = tempfile::Builder::new()
        .prefix("imcp-render-")
        .tempdir()
        .map_err(|_| "render staging unavailable")?;
    let stage = Workspace {
        roots: vec![
            temporary
                .path()
                .canonicalize()
                .map_err(|_| "render staging unavailable")?,
        ],
        max_input: workspace.max_input,
        max_output: workspace.max_output,
    };
    stage.write_new(0, Path::new("input.svg"), &input_bytes.bytes)?;
    let output = stage.roots[0].join("output.png");
    let input = stage.roots[0].join("input.svg");
    let warm = crate::engine::export(
        &workspace.roots[entry.root].join(entry.working()),
        &binary,
        &input,
        &output,
        "png",
        None,
    )
    .is_ok()
        && stage
            .regular_info(0, Path::new("output.png"), false)?
            .is_some();
    if !warm {
        let _ = stage.remove_file(0, Path::new("output.png"));
        let outcome = process::run(
            &binary,
            &[
                "--export-type=png".into(),
                format!("--export-filename={}", output.display()),
                "--export-area-page".into(),
                input.to_string_lossy().into_owned(),
            ],
            process::timeout(),
        )?;
        eprintln!(
            "{}",
            serde_json::json!({"event":"operation_preview_process","duration_s":outcome.duration_s,
            "stdout_bytes":outcome.stdout.len(),"timed_out":outcome.timed_out})
        );
        if outcome.timed_out {
            return Err("render timed out".into());
        }
        if !outcome.success {
            return Err("render failed".into());
        }
    }
    let png = stage.read(0, Path::new("output.png"), workspace.max_output)?;
    if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("render failed".into());
    }
    let relative = format!("artifacts/preview/op-{operation}-{phase}.png");
    workspace.write_new(entry.root, &entry.directory().join(&relative), &png)?;
    Ok(relative)
}

fn ensure_raster_bounds(root: &libxml::tree::Node) -> Result<(), String> {
    let cap = std::env::var("INKSCAPE_MCP_MAX_EXPORT_PX")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|cap| *cap >= 1)
        .unwrap_or(8192) as f64;
    let pattern = regex::Regex::new(
        r"^\s*([+-]?(?:[0-9]*\.?[0-9]+)(?:[eE][+-]?[0-9]+)?)\s*(px|pt|pc|mm|cm|in)?\s*$",
    )
    .unwrap();
    let declarations = crate::style::declarations(root);
    for property in ["width", "height"] {
        if let Some(value) = declarations
            .get(property)
            .cloned()
            .or_else(|| root.get_property_no_ns(property))
        {
            if value.trim().ends_with('%') {
                continue;
            }
            let capture = pattern
                .captures(&value)
                .ok_or("render dimensions could not be bounded")?;
            let number = capture[1]
                .parse::<f64>()
                .map_err(|_| "render dimensions could not be bounded")?;
            let factor = match capture.get(2).map(|unit| unit.as_str()) {
                Some("pt") => 96.0 / 72.0,
                Some("pc") => 16.0,
                Some("mm") => 96.0 / 25.4,
                Some("cm") => 96.0 / 2.54,
                Some("in") => 96.0,
                _ => 1.0,
            };
            if !number.is_finite() || number * factor > cap {
                return Err("export dimensions exceed the configured pixel limit".into());
            }
        }
    }
    if let Some(viewbox) = root.get_property_no_ns("viewBox") {
        let values: Result<Vec<f64>, _> = viewbox
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .map(str::parse)
            .collect();
        if let Ok(values) = values
            && values.len() == 4
            && values[2..].iter().any(|v| !v.is_finite() || *v > cap)
        {
            return Err("export dimensions exceed the configured pixel limit".into());
        }
    }
    Ok(())
}
