//! Frozen wire contracts captured from c50a924 through the real STDIO entry point.
//! These contain schemas/defaults/annotations/instructions, never implementations.

use serde_json::Value;

pub fn contract() -> Value {
    let live = flag("INKSCAPE_MCP_LIVE_ENABLED", true);
    let raw = flag("INKSCAPE_MCP_RAW_ACTION_ENABLED", false);
    let core = choice("INKSCAPE_MCP_TOOL_PROFILE", "core");
    let short = choice("INKSCAPE_MCP_TOOL_DESC", "short");
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
    serde_json::from_str(source).expect("checked-in reference contract must be valid JSON")
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
