//! Native deterministic guidance over one captured, compiled reference table.
use crate::document::Registry;
use serde_json::{Value, json};
use std::sync::LazyLock;
static DATA: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../migration/contracts/intent-data.json"))
        .expect("captured intent guidance")
});
fn guidance(entry: &Value) -> Value {
    json!({"goal_pattern":entry["goal_pattern"],"tools":entry["tools"],"how_to":entry["how_to"],"group":entry["group"]})
}
pub fn summary() -> Value {
    json!({"intents":DATA["entries"].as_array().unwrap().iter().map(guidance).collect::<Vec<_>>()})
}
pub fn resource_text() -> String {
    let entries = summary()["intents"]
        .as_array()
        .unwrap()
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{\"intents\": [{entries}]}}")
}
fn matched(goal: &str) -> Value {
    let lower = goal.to_lowercase();
    for rule in DATA["out_of_scope"].as_array().unwrap() {
        if rule["keywords"]
            .as_array()
            .unwrap()
            .iter()
            .any(|kw| lower.contains(kw.as_str().unwrap()))
        {
            return json!({"goal":goal,"matches":[],"out_of_scope":true,"note":rule["reason"]});
        }
    }
    let mut scores = DATA["entries"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let score = e["keywords"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|kw| lower.contains(kw.as_str().unwrap()))
                .count();
            (score, i, e)
        })
        .filter(|(score, _, _)| *score > 0)
        .collect::<Vec<_>>();
    scores.sort_by_key(|(score, i, _)| (std::cmp::Reverse(*score), *i));
    let matches = scores
        .into_iter()
        .take(3)
        .map(|(_, _, e)| guidance(e))
        .collect::<Vec<_>>();
    let note = if matches.is_empty() {
        DATA["no_match_note"].clone()
    } else {
        json!("")
    };
    json!({"goal":goal,"matches":matches,"out_of_scope":false,"note":note})
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let goal = args["goal"].as_str().ok_or("goal must be a string")?;
    if goal.len() > registry.workspace.max_input {
        return Err("goal exceeds the configured size limit".into());
    }
    let value = matched(goal);
    if value.to_string().len() > registry.workspace.max_output {
        return Err("guidance exceeds the configured output size limit".into());
    }
    Ok(value)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_summary_keyword_visibility_and_ties() {
        let contract: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        for entry in DATA["entries"].as_array().unwrap() {
            for tool in entry["tools"].as_array().unwrap() {
                assert!(
                    contract["tools/list"]["tools"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|t| t["name"] == *tool)
                );
            }
        }
        assert_eq!(summary()["intents"].as_array().unwrap().len(), 49);
        assert!(!resource_text().contains("\"keywords\""));
        let r = matched("draw a circle and rectangle");
        assert!(!r["out_of_scope"].as_bool().unwrap());
        assert_eq!(r["matches"][0]["tools"][0], "create_rect");
        assert!(
            matched("download a photo and run python")["out_of_scope"]
                .as_bool()
                .unwrap()
        );
        assert_eq!(
            matched("unrecognised xyz")["matches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }
    #[test]
    fn guidance_input_and_output_limits_refuse_without_state() {
        let mut registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![],
                max_input: 4,
                max_output: 32,
            },
            entries: indexmap::IndexMap::new(),
        };
        assert_eq!(
            apply(&registry, &json!({"goal":"rectangle"})).unwrap_err(),
            "goal exceeds the configured size limit"
        );
        registry.workspace.max_input = 1024;
        assert_eq!(
            apply(&registry, &json!({"goal":"unknown"})).unwrap_err(),
            "guidance exceeds the configured output size limit"
        );
        assert!(registry.entries.is_empty());
    }
}
