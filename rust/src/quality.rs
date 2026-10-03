//! Validation, quantitative metrics and shared read-only optimization/structure analysis.
use crate::{
    collection,
    document::{Registry, elements},
    editability, fonts, inspect, optimization_analysis, validate, xml,
};
use serde_json::{Value, json};
fn one(registry: &Registry, id: &str, options: &Value) -> Result<Value, String> {
    let report = validate::document(registry, id)?;
    let summary = registry.summary(id)?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let doc = xml::parse(&bytes, registry.workspace.max_input)?;
    let root = doc
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let nodes = elements(root.clone());
    let mut raster_count = 0;
    let mut raster_bytes = 0;
    for n in &nodes {
        if n.get_name() == "image"
            && let Some(href) = n
                .get_property_ns("href", "http://www.w3.org/1999/xlink")
                .filter(|s| !s.is_empty())
                .or_else(|| n.get_property_no_ns("href"))
            && let Some(size) = validate::data_size(&href)
        {
            raster_count += 1;
            raster_bytes += size;
        }
    }
    let font_report = inspect::resource(registry, id, "fonts")?;
    let referenced = font_report["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["family"].as_str())
        .map(str::trim)
        .filter(|f| !f.is_empty() && !fonts::generic(f))
        .collect::<Vec<_>>();
    let installed = fonts::installed(registry.workspace.max_input);
    let missing = installed
        .as_ref()
        .map(|s| {
            referenced
                .iter()
                .filter(|f| !s.contains(&f.to_lowercase()))
                .copied()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let coverage = json!({"referenced":referenced.len(),"installed":if installed.is_some(){referenced.len()-missing.len()}else{0},"missing":missing.iter().take(64).map(|s|s.chars().take(120).collect::<String>()).collect::<Vec<_>>(),"checked":installed.is_some()});
    let vb = summary["viewbox"].as_array();
    let width = vb.map(|v| v[2].clone()).unwrap_or(Value::Null);
    let height = vb.map(|v| v[3].clone()).unwrap_or(Value::Null);
    let viewbox = json!({"present":vb.is_some(),"valid":width.as_f64().is_some_and(|w|w>0.)&&height.as_f64().is_some_and(|h|h>0.),"width":width,"height":height});
    let counts = optimization_analysis::counts(root.clone());
    let messages = [
        (
            "editor_metadata",
            "editor-only metadata/attributes/comments can be stripped",
        ),
        (
            "unused_defs",
            "unreferenced <defs> templates can be removed",
        ),
        (
            "unreferenced_ids",
            "id attributes nothing references can be dropped",
        ),
        ("empty_groups", "empty groups can be removed"),
        ("reducible_coords", "coordinate precision can be reduced"),
    ];
    let opportunities = counts
        .into_iter()
        .zip(messages)
        .filter(|(n, _)| *n > 0)
        .map(|(n, (code, message))| json!({"code":code,"count":n,"message":message}))
        .collect::<Vec<_>>();
    let penalty = report["error_count"].as_u64().unwrap() * 15
        + report["warning_count"].as_u64().unwrap() * 5
        + counts.iter().map(|n| (*n).min(5) as u64).sum::<u64>();
    Ok(
        json!({"doc_id":id,"ok":report["ok"],"score":100u64.saturating_sub(penalty),"findings":report["findings"],"error_count":report["error_count"],"warning_count":report["warning_count"],"metrics":{"object_count":summary["num_objects"],"node_count":nodes.len(),"layer_count":summary["num_layers"],"embedded_raster_count":raster_count,"embedded_raster_bytes":raster_bytes,"font_coverage":coverage,"viewbox":viewbox},"opportunities":opportunities,"editability":editability::analyze(root,options)?}),
    )
}
fn report(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    if tool == "quality_report" {
        return one(
            registry,
            args["doc_id"].as_str().ok_or("doc_id must be a string")?,
            &args["editability"],
        );
    }
    let ids = collection::ids(args)?;
    let verdict = collection::verdict(registry, &ids)?;
    let mut reports = vec![];
    for id in ids {
        reports.push(one(registry, id, &Value::Null)?);
    }
    let scores = reports
        .iter()
        .map(|r| r["score"].as_u64().unwrap())
        .collect::<Vec<_>>();
    let sum = scores.iter().sum::<u64>();
    let mean = (sum as f64 / scores.len() as f64).round_ties_even() as u64;
    Ok(
        json!({"all_ok":reports.iter().all(|r|r["ok"]==true),"worst_score":scores.iter().min(),"mean_score":mean,"total_opportunities":reports.iter().map(|r|r["opportunities"].as_array().unwrap().len()).sum::<usize>(),"per_doc":reports,"consistency":verdict}),
    )
}

pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let value = report(registry, tool, args)?;
    if value.to_string().len() > registry.workspace.max_output {
        return Err("quality report exceeds the configured output size limit".into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_and_set_output_cap_refuse_without_mutation_or_history() {
        let dir = tempfile::tempdir().unwrap();
        let source = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><rect/></svg>";
        std::fs::write(dir.path().join("source.svg"), source).unwrap();
        let mut registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![dir.path().canonicalize().unwrap()],
                max_input: 4096,
                max_output: 32,
            },
            entries: indexmap::IndexMap::new(),
        };
        let opened = registry.open(&json!({"path":"source.svg"})).unwrap();
        let id = opened["doc_id"].as_str().unwrap();
        for (tool, args) in [
            ("quality_report", json!({"doc_id":id})),
            ("quality_report_set", json!({"doc_ids":[id]})),
        ] {
            assert_eq!(
                apply(&registry, tool, &args).unwrap_err(),
                "quality report exceeds the configured output size limit"
            );
        }
        let entry = &registry.entries[id];
        assert_eq!(
            std::fs::read(dir.path().join(entry.working())).unwrap(),
            source
        );
        assert_eq!(
            std::fs::read_dir(dir.path().join(entry.directory()).join("snapshots"))
                .unwrap()
                .count(),
            0
        );
        assert_eq!(
            std::fs::read_dir(dir.path().join(entry.directory()).join("operations"))
                .unwrap()
                .count(),
            0
        );
    }
}
