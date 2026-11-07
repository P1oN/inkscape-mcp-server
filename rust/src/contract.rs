//! Frozen wire contracts captured from c50a924 through the real STDIO entry point.
//! These contain schemas/defaults/annotations/instructions, never implementations.

use serde_json::Value;

const COMPACT_TOOLS: &[&str] = &[
    // The workspace and create tools also support the installer's health probe.
    "get_workspace_info",
    "create_document",
    "open_document",
    "find_objects",
    "boolean_difference",
    "validate_document",
    "render_preview",
    "compare_region",
    "save_document_as",
    "restore_snapshot",
];

pub fn contract() -> Value {
    let compact = choice("INKSCAPE_MCP_TOOL_PROFILE", "compact");
    let live = !compact && flag("INKSCAPE_MCP_LIVE_ENABLED", true);
    let raw = !compact && flag("INKSCAPE_MCP_RAW_ACTION_ENABLED", false);
    let core = choice("INKSCAPE_MCP_TOOL_PROFILE", "core");
    let short = compact || choice("INKSCAPE_MCP_TOOL_DESC", "short");
    macro_rules! select {
        ($filename:literal) => {
            include_str!(concat!("../../migration/contracts/", $filename, ".json"))
        };
    }
    let source = match (live, raw, core, short) {
        (true, true, false, false) => select!("live-true_raw-true_full_full"),
        (true, true, false, true) => select!("live-true_raw-true_full_short"),
        (true, true, true, false) => select!("live-true_raw-true_core_full"),
        (true, true, true, true) => select!("live-true_raw-true_core_short"),
        (true, false, false, false) => select!("live-true_raw-false_full_full"),
        (true, false, false, true) => select!("live-true_raw-false_full_short"),
        (true, false, true, false) => select!("live-true_raw-false_core_full"),
        (true, false, true, true) => select!("live-true_raw-false_core_short"),
        (false, true, false, false) => select!("live-false_raw-true_full_full"),
        (false, true, false, true) => select!("live-false_raw-true_full_short"),
        (false, true, true, false) => select!("live-false_raw-true_core_full"),
        (false, true, true, true) => select!("live-false_raw-true_core_short"),
        (false, false, false, false) => select!("live-false_raw-false_full_full"),
        (false, false, false, true) => select!("live-false_raw-false_full_short"),
        (false, false, true, false) => select!("live-false_raw-false_core_full"),
        (false, false, true, true) => select!("live-false_raw-false_core_short"),
    };
    let mut value: Value =
        serde_json::from_str(source).expect("checked-in reference contract must be valid JSON");
    if compact {
        value["tools/list"]["tools"]
            .as_array_mut()
            .unwrap()
            .retain(|tool| COMPACT_TOOLS.contains(&tool["name"].as_str().unwrap()));
    }
    let instructions = inkscape_mcp_rust::update::instructions::active()
        .expect("instructions validated before server construction");
    value["initialize"]["instructions"] =
        Value::String(instructions.expand(&instructions.initialization));
    for prompt in value["prompts/list"]["prompts"].as_array_mut().unwrap() {
        let name = prompt["name"].as_str().unwrap().to_string();
        prompt["description"] = instructions.prompts[&name]["description"].clone();
    }
    value
}

pub(crate) fn flag(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => matches!(
            value.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Ok(_) => default,
        Err(_) => default,
    }
}

fn choice(name: &str, value: &str) -> bool {
    std::env::var(name).is_ok_and(|candidate| candidate.trim().to_lowercase() == value)
}
