//! Controlled typed action plans: operator allowlist AND detected version map AND fixed grammar.
use crate::{arguments, document::Registry, paths, process, style, transaction};
use regex::Regex;
use serde_json::{Value, json};
use std::{collections::HashSet, path::PathBuf, sync::LazyLock};
static ACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._-]*(:[A-Za-z0-9][A-Za-z0-9._-]*)?$").unwrap()
});
static ARG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._#-]*$").unwrap());
static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._:-]*$").unwrap());
static VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Inkscape\s+(\d+)\.(\d+)(?:\.(\d+))?").unwrap());
static KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+\.\d+(?:\.\d+)?)").unwrap());
const DEFAULTS: &[&str] = &[
    "select-by-id",
    "select-all",
    "select-clear",
    "selection-group",
    "selection-ungroup",
    "object-to-path",
    "path-union",
    "path-difference",
    "path-intersection",
    "path-combine",
    "path-break-apart",
    "path-simplify",
];
pub(crate) fn allowlist() -> HashSet<String> {
    let mut result: HashSet<_> = DEFAULTS.iter().map(|s| s.to_string()).collect();
    if let Some(raw) = std::env::var_os("INKSCAPE_MCP_ACTION_ALLOWLIST") {
        for s in std::env::split_paths(&raw) {
            let s = s.to_string_lossy();
            let s = s.trim();
            if TOKEN.is_match(s) {
                result.insert(s.into());
            }
        }
    }
    result
}
fn version_key(raw: &str) -> String {
    let candidate = KEY.find(raw).map_or(raw.trim(), |m| m.as_str());
    if ARG.is_match(candidate) && !candidate.contains('#') {
        candidate.into()
    } else {
        "unknown".into()
    }
}
fn probe(args: &[&str], limit: usize) -> Option<String> {
    let binary = process::inkscape_binary()?;
    let outcome = process::run_bounded(
        &binary,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        process::timeout(),
        limit,
    )
    .ok()?;
    if !outcome.success || outcome.timed_out || outcome.stdout.len() >= limit {
        return None;
    }
    Some(String::from_utf8_lossy(&outcome.stdout).into_owned())
}
pub(crate) fn parse_actions(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    text.lines()
        .filter_map(|line| line.trim().split_once(':').map(|(first, _)| first.trim()))
        .filter(|id| !id.is_empty() && seen.insert(id.to_string()))
        .map(str::to_owned)
        .collect()
}
fn map(registry: &Registry) -> Value {
    map_from(registry, None)
}
pub(crate) fn map_from(registry: &Registry, detected: Option<&Value>) -> Value {
    let version = detected
        .and_then(|v| v["inkscape_version"].as_str())
        .map(str::to_owned)
        .or_else(|| {
            if detected.is_none() {
                probe(&["--version"], registry.workspace.max_output)
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_owned())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".into());
    let actions = if let Some(v) = detected {
        v["actions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()
    } else {
        probe(&["--action-list"], registry.workspace.max_output)
            .map(|s| parse_actions(&s))
            .unwrap_or_default()
    };
    let tuple = VERSION.captures(&version).and_then(|c| {
        Some(vec![
            c[1].parse::<u64>().ok()?,
            c[2].parse::<u64>().ok()?,
            c.get(3).map_or(Some(0), |m| m.as_str().parse().ok())?,
        ])
    });
    let path =
        PathBuf::from(".inkscape-mcp/action-maps").join(format!("{}.json", version_key(&version)));
    if !registry.workspace.roots.is_empty()
        && let Ok(Some(bytes)) =
            registry
                .workspace
                .read_optional(0, &path, registry.workspace.max_output)
        && let Ok(cached) = serde_json::from_slice::<Value>(&bytes)
        && cached["inkscape_version"].is_string()
        && cached["probed_at"].is_string()
        && cached["actions"]
            .as_array()
            .is_some_and(|a| a.iter().all(Value::is_string))
    {
        return cached;
    }
    let fresh = json!({"inkscape_version":version,"inkscape_version_tuple":tuple,"actions":actions,"action_count":actions.len(),"probed_at":detected.map(|v|v["probed_at"].clone()).unwrap_or_else(||json!(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros,true))),"source":"probe"});
    if !registry.workspace.roots.is_empty()
        && registry
            .workspace
            .ensure_directory(0, path.parent().unwrap())
            .is_ok()
    {
        let _ =
            registry
                .workspace
                .atomic_write(0, &path, &serde_json::to_vec_pretty(&fresh).unwrap());
    }
    fresh
}
fn validate(steps: &Value, cmap: &Value, allow: &HashSet<String>) -> Result<Value, String> {
    let steps = steps.as_array().ok_or("steps must be a list")?;
    if steps.is_empty() {
        return Err("empty_chain: action chain has no steps".into());
    }
    if steps.len() > 32 {
        return Err("chain_too_long: action chain exceeds 32 steps".into());
    }
    let available: HashSet<_> = cmap["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let mut normalized = Vec::new();
    let mut parts = Vec::new();
    for step in steps {
        let action = step["action"]
            .as_str()
            .ok_or("action must be a string")?
            .trim();
        let repr = style::python_repr(action);
        if !ACTION.is_match(action) {
            return Err(format!("malformed_action: malformed action id: {repr}"));
        }
        if !allow.contains(action) {
            return Err(format!(
                "not_allowlisted: action {repr} is not allowlisted for execution"
            ));
        }
        if !available.contains(action) {
            return Err(format!(
                "action_absent: action {repr} is not available on Inkscape {}; call list_actions to see which Actions this runtime exposes",
                cmap["inkscape_version"].as_str().unwrap()
            ));
        }
        let empty = Vec::new();
        let args = match step.get("args") {
            None => &empty,
            Some(v) => v.as_array().ok_or("args must be a list")?,
        };
        if args.len() > 16 {
            return Err(format!(
                "too_many_args: action {repr} has too many arguments"
            ));
        }
        let mut tokens = Vec::new();
        for arg in args {
            let raw = arg.as_str().ok_or("args must contain strings")?;
            let token = raw.trim();
            if !ARG.is_match(token) {
                let mut message = format!(
                    "malformed_arg: malformed argument token: {}",
                    style::python_repr(raw)
                );
                if raw.contains(',') {
                    let ids = raw
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(style::python_repr)
                        .collect::<Vec<_>>()
                        .join(", ");
                    message.push_str(&format!(". HINT: pass each id as a SEPARATE arg token, not a comma-joined string (e.g. args=[{ids}] for select-by-id); the engine joins them with commas itself."));
                }
                return Err(message);
            }
            tokens.push(token.to_owned());
        }
        parts.push(if tokens.is_empty() {
            action.into()
        } else {
            format!("{action}:{}", tokens.join(","))
        });
        normalized.push(json!({"action":action,"args":tokens}));
    }
    let assembled = parts.join(";");
    Ok(
        json!({"steps":normalized,"actions_argument":assembled,"argv_preview":["<working-copy>",format!("--actions={assembled}"),"--export-type=svg","--export-plain-svg","--export-filename=<export-output>"],"inkscape_version":cmap["inkscape_version"],"valid":true}),
    )
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let raw = tool == "run_raw_action";
    let validate_only = tool == "validate_action_chain";
    let id = if validate_only {
        ""
    } else {
        args["doc_id"].as_str().ok_or("doc_id must be a string")?
    };
    let dry = raw && arguments::boolean(args, "dry_run", true)?;
    let approval = arguments::string(args, "approval_token")?;
    if raw && !registry.entries.contains_key(id) {
        return Err("document id not found".into());
    }
    if !validate_only && !dry && approval.is_none_or(str::is_empty) {
        return Err(if raw {
            "high-risk raw action requires an explicit approval_token"
        } else {
            "high-risk action chain requires an explicit approval_token"
        }
        .into());
    }
    let mut steps = if raw {
        json!([{"action":args["action"],"args":args.get("args").filter(|v|!v.is_null()).cloned().unwrap_or(json!([]))}])
    } else {
        args["steps"].clone()
    };
    if let Some(entries) = steps.as_array_mut() {
        for step in entries {
            let empty = json!([]);
            *step = json!({"action":step["action"],"args":step.get("args").unwrap_or(&empty)});
        }
    }
    let cmap = map(registry);
    let allow = allowlist();
    let plan = validate(&steps, &cmap, &allow)?;
    if validate_only {
        return Ok(plan);
    }
    let mut result = json!({"doc_id":id,"changed":false,"plan":plan,"summary":null,"operation_id":null,"snapshot_id":null,"preview_before":null,"preview_after":null});
    if raw {
        result["dry_run"] = json!(dry);
    }
    if dry {
        return Ok(result);
    }
    let params = if raw {
        json!({"action":plan["steps"][0]["action"],"args":plan["steps"][0]["args"],"actions":plan["actions_argument"]})
    } else {
        json!({"steps":steps,"actions":plan["actions_argument"]})
    };
    let applied = transaction::apply(registry, id, tool, params, "high", approval, |doc| {
        let checked = validate(&steps, &cmap, &allow)?;
        let chain = checked["actions_argument"].as_str().unwrap();
        let new = paths::run_actions(registry, id, chain, "action chain")?;
        paths::validate_result(doc, &new)?;
        *doc = paths::replace_root(doc, &new, registry.workspace.max_input)?;
        Ok(format!(
            "applied action chain ({} step(s)): {chain}",
            checked["steps"].as_array().unwrap().len()
        ))
    })?;
    for key in [
        "changed",
        "summary",
        "operation_id",
        "snapshot_id",
        "preview_before",
        "preview_after",
    ] {
        result[key] = applied[key].clone();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plans_have_fixed_argv_and_two_independent_gates_before_arguments() {
        let caps = json!({"inkscape_version":"Inkscape 1.4.3","actions":["select-by-id","path-union","export-do"]});
        let allow = HashSet::from(["select-by-id".into(), "path-union".into(), "absent".into()]);
        let plan = validate(
            &json!([{"action":" select-by-id ","args":[" a ","b"]},{"action":"path-union"}]),
            &caps,
            &allow,
        )
        .unwrap();
        assert_eq!(plan["actions_argument"], "select-by-id:a,b;path-union");
        assert_eq!(plan["argv_preview"][0], "<working-copy>");
        assert!(
            validate(&json!([{"action":"export-do"}]), &caps, &allow)
                .unwrap_err()
                .starts_with("not_allowlisted:")
        );
        assert!(
            validate(
                &json!([{"action":"absent","args":["bad;arg"]}]),
                &caps,
                &allow
            )
            .unwrap_err()
            .starts_with("action_absent:")
        );
        let hint = validate(
            &json!([{"action":"select-by-id","args":["a,b"]}]),
            &caps,
            &allow,
        )
        .unwrap_err();
        assert!(hint.contains("args=['a', 'b']"));
        for action in ["ns::a", "a:b:c", "select-all;quit", "", "--flag"] {
            assert!(
                validate(&json!([{"action":action}]), &caps, &allow)
                    .unwrap_err()
                    .starts_with("malformed_action:")
            );
        }
        assert!(
            validate(&json!([]), &caps, &allow)
                .unwrap_err()
                .starts_with("empty_chain:")
        );
        assert!(
            validate(
                &json!(vec![json!({"action":"path-union"}); 33]),
                &caps,
                &allow
            )
            .unwrap_err()
            .starts_with("chain_too_long:")
        );
        assert!(
            validate(
                &json!([{"action":"select-by-id","args":vec!["a";17]}]),
                &caps,
                &allow
            )
            .unwrap_err()
            .starts_with("too_many_args:")
        );
    }
    #[test]
    fn maps_keep_probe_order_and_version_keys_cannot_escape() {
        assert_eq!(
            parse_actions(" b : B\na : A\nb : duplicate\nno delimiter\n : blank"),
            ["b", "a"]
        );
        assert_eq!(version_key("Inkscape 1.4.3 (build)"), "1.4.3");
        for unsafe_key in ["../../escape", "/absolute", "../1.4.3/../../"] {
            let key = version_key(unsafe_key);
            assert!(!key.contains('/') && !key.contains('\\'));
        }
        assert_eq!(version_key(" unparsed "), "unparsed");
        assert_eq!(version_key(""), "unknown");
    }
}
