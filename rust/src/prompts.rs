//! Native prompt rendering from the checked reference message templates.
use rmcp::{ErrorData, model::*};
use serde_json::{Value, json};
pub fn index(contract: &Value) -> Value {
    let mut prompts = contract["prompts/list"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    prompts.sort_by_key(|p| p["name"].as_str().unwrap());
    json!({"prompt_count":prompts.len(),"prompts":prompts.into_iter().map(|p| {
        let purpose=p["description"].as_str().unwrap_or("").lines().map(str::trim).find(|s| !s.is_empty()).unwrap_or("(no description)");
        let arguments=p["arguments"].as_array().map(|args|args.iter().map(|a|json!({"name":a["name"],"description":a["description"].as_str().unwrap_or(""),"required":a["required"].as_bool().unwrap_or(false)})).collect::<Vec<_>>()).unwrap_or_default();
        json!({"name":p["name"],"purpose":purpose,"arguments":arguments})
    }).collect::<Vec<_>>()})
}

pub fn render(
    contract: &Value,
    request: GetPromptRequestParams,
) -> Result<GetPromptResponse, ErrorData> {
    let args = request.arguments.unwrap_or_default();
    if args.values().any(|v| !v.is_string()) {
        return Err(ErrorData::invalid_params(
            "Invalid request parameters",
            Some(json!("")),
        ));
    }
    let known = contract["prompts/list"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["name"] == request.name);
    if !known {
        return Err(serde_json::from_value(
            json!({"code":0,"message":format!("Unknown prompt: '{}'",request.name)}),
        )
        .unwrap());
    }
    let instructions = inkscape_mcp_rust::update::instructions::active()
        .map_err(|e| ErrorData::internal_error(e, None))?;
    let mut rendered = instructions.prompts[&request.name].clone();
    if request.name == "compose_artwork" {
        for message in rendered["messages"].as_array_mut().unwrap() {
            message["content"]["text"] =
                json!(instructions.expand(message["content"]["text"].as_str().unwrap()));
        }
    }
    if matches!(
        request.name.as_str(),
        "compose_artwork" | "restyle_artwork" | "live_canvas_assist"
    ) {
        let goal=args.get("goal").and_then(Value::as_str).ok_or_else(||serde_json::from_value::<ErrorData>(json!({"code":0,"message":format!("Error rendering prompt '{}': Missing required arguments: {{'goal'}}",request.name)})).unwrap())?;
        let clean = goal
            .split(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(500)
            .collect::<String>();
        let clean = if clean.is_empty() {
            "(no goal provided)"
        } else {
            &clean
        };
        for message in rendered["messages"].as_array_mut().unwrap() {
            let text = message["content"]["text"].as_str().unwrap();
            message["content"]["text"] =
                json!(text.replace("__MIGRATION_GOAL_SLOT_9a6c0__", clean));
        }
    }
    let result: GetPromptResult = serde_json::from_value(rendered)
        .map_err(|_| ErrorData::internal_error("prompt serialization failed", None))?;
    Ok(result.into())
}
