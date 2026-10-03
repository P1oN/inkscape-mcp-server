//! Read-only AND filtering with the reference cascade and one bounded CLI query.
use crate::{
    arguments, css,
    document::{Registry, elements},
    inspect, process, render, style,
    workspace::Workspace,
    xml,
};
use serde_json::{Value, json};
use std::{collections::HashMap, path::Path};
type Box4 = [f64; 4];
fn boxed(b: Box4) -> Value {
    json!({"x":b[0],"y":b[1],"width":b[2],"height":b[3]})
}
fn intersects(a: Box4, b: Box4) -> bool {
    a[0] <= b[0] + b[2] && b[0] <= a[0] + a[2] && a[1] <= b[1] + b[3] && b[1] <= a[1] + a[3]
}
fn parse_rows(raw: &[u8]) -> HashMap<String, Box4> {
    let mut result = HashMap::new();
    for line in String::from_utf8_lossy(raw).lines() {
        let fields = line.trim().split(',').collect::<Vec<_>>();
        if fields.len() != 5 || fields[0].trim().is_empty() {
            continue;
        }
        let numbers = fields[1..]
            .iter()
            .map(|s| s.trim().parse::<f64>())
            .collect::<Result<Vec<_>, _>>();
        if let Ok(n) = numbers
            && !(n[2] < 0. || n[3] < 0.)
        {
            result.insert(fields[0].trim().to_owned(), [n[0], n[1], n[2], n[3]]);
        }
    }
    result
}
fn query(registry: &Registry, id: &str) -> Result<HashMap<String, Box4>, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let binary = process::inkscape_binary().ok_or("inkscape engine unavailable")?;
    let input = render::input(&registry.workspace, entry)?;
    let temporary = tempfile::Builder::new()
        .prefix("imcp-query-")
        .tempdir()
        .map_err(|_| "query staging unavailable")?;
    let stage = Workspace {
        roots: vec![
            temporary
                .path()
                .canonicalize()
                .map_err(|_| "query staging unavailable")?,
        ],
        max_input: registry.workspace.max_input,
        max_output: registry.workspace.max_output,
    };
    stage.write_new(0, Path::new("input.svg"), &input.bytes)?;
    let outcome = process::run_bounded(
        &binary,
        &[
            stage.roots[0]
                .join("input.svg")
                .to_string_lossy()
                .into_owned(),
            "--query-all".into(),
        ],
        process::timeout(),
        registry.workspace.max_output,
    )?;
    if !outcome.success
        || outcome.timed_out
        || outcome.stdout.len() >= registry.workspace.max_output
    {
        return Ok(HashMap::new());
    }
    Ok(parse_rows(&outcome.stdout))
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let tag = arguments::string(args, "tag")?;
    let fill = arguments::string(args, "fill")?.map(style::color_key);
    let stroke = arguments::string(args, "stroke")?.map(style::color_key);
    let text = arguments::string(args, "text")?.map(str::to_lowercase);
    let prefix = arguments::string(args, "id_prefix")?;
    let accurate = arguments::boolean(args, "accurate_bbox", false)?;
    let region = if let Some(value) = args.get("bbox").filter(|v| !v.is_null()) {
        let mut b = [0.; 4];
        for (i, key) in ["x", "y", "width", "height"].iter().enumerate() {
            b[i] = arguments::optional_number(value, key)?
                .ok_or_else(|| format!("bbox requires {key}"))?;
        }
        Some(b)
    } else {
        None
    };
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let document = xml::parse(&bytes, registry.workspace.max_input)?;
    let nodes = elements(
        document
            .get_root_element()
            .ok_or("document could not be parsed safely")?,
    );
    let rules = if fill.is_some() || stroke.is_some() {
        css::rules(&nodes)
    } else {
        vec![]
    };
    // The reference paint matcher treats only the exact token "inherit" as
    // inherited. Memoize the preorder walk rather than rewalking every parent.
    let mut paints: HashMap<usize, [Option<String>; 2]> = HashMap::new();
    if fill.is_some() || stroke.is_some() {
        for node in &nodes {
            let mut inherited = node
                .get_parent()
                .and_then(|p| paints.get(&(p.node_ptr() as usize)))
                .cloned()
                .unwrap_or([None, None]);
            for (i, key) in ["fill", "stroke"].iter().enumerate() {
                if let Some(value) = css::own(node, &rules, key).filter(|v| v != "inherit") {
                    inherited[i] = Some(value);
                }
            }
            paints.insert(node.node_ptr() as usize, inherited);
        }
    }
    let boxes = if accurate {
        query(registry, id).unwrap_or_default()
    } else {
        HashMap::new()
    };
    let mut matched = Vec::new();
    for node in &nodes {
        let Some(mut object) = inspect::object_ref(node) else {
            continue;
        };
        let oid = object["object_id"].as_str().unwrap();
        let engine = boxes.get(oid).copied();
        if tag.is_some_and(|tag| tag != object["tag"].as_str().unwrap())
            || prefix.is_some_and(|prefix| !oid.starts_with(prefix))
        {
            continue;
        }
        let effective = paints.get(&(node.node_ptr() as usize));
        if [fill.as_ref(), stroke.as_ref()]
            .iter()
            .enumerate()
            .any(|(i, needle)| {
                needle.is_some_and(|needle| {
                    effective
                        .and_then(|p| p[i].as_ref())
                        .is_none_or(|v| style::color_key(v) != *needle)
                })
            })
        {
            continue;
        }
        if text.as_ref().is_some_and(|needle| {
            object["text"]
                .as_str()
                .is_none_or(|v| !v.to_lowercase().contains(needle))
        }) {
            continue;
        }
        if let Some(b) = region {
            let bounds = engine.or_else(|| {
                let v = &object["bbox"];
                Some([
                    v["x"].as_f64()?,
                    v["y"].as_f64()?,
                    v["width"].as_f64()?,
                    v["height"].as_f64()?,
                ])
            });
            if bounds.is_none_or(|bounds| !intersects(bounds, b)) {
                continue;
            }
        }
        if let Some(engine) = engine {
            object["bbox"] = boxed(engine);
        }
        matched.push(object);
    }
    Ok(json!({"doc_id":id,"count":matched.len(),"objects":matched}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_skips_bad_rows_keeps_zero_extent_and_last_duplicate() {
        let rows = parse_rows(
            b"a,1,2,3,4\nb,bad,2,3,4\nc,1,2,-3,4\na,5,6,0,0\n,0,0,1,1\nstray\nnonfinite,inf,0,1,1",
        );
        assert_eq!(rows.len(), 2);
        assert!(rows["nonfinite"][0].is_infinite());
        assert_eq!(rows["a"], [5., 6., 0., 0.]);
        assert!(intersects([0., 0., 1., 1.], [1., 1., 1., 1.]));
        assert!(!intersects([0., 0., 1., 1.], [1.01, 1., 1., 1.]));
        assert_eq!("ΟΣ".to_lowercase(), "ος");
    }
}
