//! Engine-derived content framing uses a private bounded identity probe.
use crate::{
    canvas, document::Registry, process, render, style, transaction, transform,
    workspace::Workspace, xml,
};
use libxml::tree::Document;
use serde_json::{Value, json};
use std::path::Path;
fn bounds(output: &[u8]) -> Result<[f64; 4], String> {
    let text = String::from_utf8_lossy(output);
    let first = text
        .trim()
        .lines()
        .next()
        .ok_or("document has no content to fit the viewBox to")?;
    let fields = first.split(',').collect::<Vec<_>>();
    if fields.len() != 5 {
        return Err("content bbox query returned an unexpected shape".into());
    }
    let mut result: [f64; 4] = [0.0; 4];
    for (value, raw) in result.iter_mut().zip(&fields[1..]) {
        *value = raw
            .trim()
            .parse()
            .map_err(|_| "content bbox query returned a non-numeric value")?;
    }
    if !result.iter().all(|v| v.is_finite()) || result[2] <= 0.0 || result[3] <= 0.0 {
        return Err("content bounding box is degenerate".into());
    }
    Ok(result)
}
fn mutate(registry: &Registry, id: &str, document: &mut Document) -> Result<String, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let mut root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let input = if let Some(identity) = canvas::synthesized(
        root.get_property_no_ns("width"),
        root.get_property_no_ns("height"),
    ) {
        // Copy only the root, matching the reference probe's omission of the outer DTD.
        let bytes = format!(
            "<?xml version='1.0' encoding='UTF-8'?>\n{}",
            document.node_to_string(&root)
        )
        .into_bytes();
        let probe = xml::parse(&bytes, registry.workspace.max_input)?;
        let mut probe_root = probe
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        probe_root
            .set_property("viewBox", &identity)
            .map_err(|_| "content bbox query failed")?;
        if probe_root
            .get_property_no_ns("preserveAspectRatio")
            .is_some()
        {
            probe_root
                .remove_property_no_ns("preserveAspectRatio")
                .map_err(|_| "content bbox query failed")?;
        }
        crate::engine_input::prepare(
            &registry.workspace,
            entry.root,
            Path::new(&entry.source),
            &xml::serialize(&probe),
        )?
    } else {
        // Pin/read the working copy and preserve its relative asset base in staging.
        render::input(&registry.workspace, entry)?
    };
    let binary = process::inkscape_binary().ok_or("inkscape engine unavailable")?;
    let temporary = tempfile::Builder::new()
        .prefix("imcp-fit-")
        .tempdir()
        .map_err(|_| "content bbox query failed")?;
    let stage = Workspace {
        roots: vec![
            temporary
                .path()
                .canonicalize()
                .map_err(|_| "content bbox query failed")?,
        ],
        max_input: registry.workspace.max_input,
        max_output: registry.workspace.max_output,
    };
    stage
        .write_new(0, Path::new("input.svg"), &input.bytes)
        .map_err(|_| "content bbox query failed")?;
    let outcome = process::run(
        &binary,
        &[
            "--query-all".into(),
            stage.roots[0]
                .join("input.svg")
                .to_string_lossy()
                .into_owned(),
        ],
        process::timeout(),
    )
    .map_err(|_| "inkscape engine unavailable")?;
    if outcome.timed_out {
        return Err("content bbox query timed out".into());
    }
    if !outcome.success {
        return Err("content bbox query failed".into());
    }
    let bbox = bounds(&outcome.stdout)?;
    if canvas::viewbox(root.get_property_no_ns("viewBox")).is_some_and(|current| {
        current
            .iter()
            .zip(bbox)
            .all(|(a, b)| (*a - b).abs() <= 1e-6)
    }) {
        return Ok("viewBox already fits content".into());
    }
    let value = bbox
        .into_iter()
        .map(transform::number)
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    root.set_property("viewBox", &value)
        .map_err(|_| "canvas edit failed")?;
    Ok(format!(
        "fit viewBox to content bbox {}",
        style::python_repr(&value)
    ))
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    transaction::apply(
        registry,
        id,
        "fit_to_content",
        json!({}),
        "medium",
        None,
        |document| mutate(registry, id, document),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_output_must_prove_finite_drawable_bounds() {
        assert_eq!(
            bounds(b"svg, -3,2,4.5,6\nr,0,0,1,1").unwrap(),
            [-3.0, 2.0, 4.5, 6.0]
        );
        for output in [
            b"".as_slice(),
            b"svg,0,0,0,10",
            b"svg,0,0,NaN,10",
            b"svg,0,0,1",
            b"svg,0,0,bad,10",
        ] {
            assert!(bounds(output).is_err());
        }
    }
}
