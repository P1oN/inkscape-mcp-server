//! Bounded read-only runtime probes. Never imports helpers or starts a GUI/session.
use crate::{actions, arguments, document::Registry, process, workspace::Workspace};
use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::LazyLock,
};
static VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Inkscape\s+(\d+)\.(\d+)(?:\.(\d+))?").unwrap());
static EXPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)--export-type\S*.*?\[([^\]]+)\]").unwrap());
static INKEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"__version__\s*=\s*['"]([^'"]+)['"]"#).unwrap());
static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._:-]*$").unwrap());
pub(crate) fn owner_name(text: &str) -> Option<String> {
    static OWNER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\(\s*'(:[0-9]+\.[0-9]+)'\s*,\s*\)$").unwrap());
    OWNER.captures(text.trim()).map(|c| c[1].to_owned())
}
pub(crate) fn version(text: &str) -> Option<[u64; 3]> {
    let c = VERSION.captures(text)?;
    Some([
        c[1].parse().ok()?,
        c[2].parse().ok()?,
        c.get(3).map_or(Some(0), |m| m.as_str().parse().ok())?,
    ])
}
fn exports(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    EXPORT
        .captures(text)
        .map(|c| {
            c[1].split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty() && seen.insert(s.to_string()))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
fn probe(
    binary: &PathBuf,
    args: &[&str],
    label: &str,
    notes: &mut Vec<String>,
    limit: usize,
    errors: bool,
) -> Option<String> {
    let result = process::run_bounded(
        binary,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        process::timeout(),
        limit,
    );
    let Ok(result) = result else {
        notes.push(format!("{label} failed to launch"));
        return None;
    };
    if result.timed_out {
        notes.push(format!(
            "{label} timed out after {}s",
            json!(process::timeout().as_secs_f64())
        ));
        return None;
    }
    if !result.success {
        if errors {
            notes.push(format!("{label} exited {}", result.exit_code));
        }
        return None;
    }
    if result.stdout.len() >= limit {
        notes.push(format!("{label} output exceeds size limit"));
        return None;
    }
    Some(String::from_utf8_lossy(&result.stdout).into_owned())
}
fn addressed<'a>(args: &[&'a str], address: Option<&'a str>) -> Vec<&'a str> {
    let mut args = args.to_vec();
    if let Some(address) = address {
        args.splice(1..2, ["--address", address]);
    }
    args
}
fn cli(args: &[&str], notes: &mut Vec<String>, limit: usize) -> Option<String> {
    probe(
        &process::inkscape_binary()?,
        args,
        &format!("inkscape {}", args.join(" ")),
        notes,
        limit,
        true,
    )
}
fn clean(text: Option<String>) -> Option<String> {
    text.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}
/// Overlay the same gated tool surface exposed by tools/list on each cached read.
pub(crate) fn overlay(probe: &Value, contract: &Value) -> Value {
    static RISK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)Risk class:\s*(low|medium|high|restricted)").unwrap());
    let mut tools = contract["tools/list"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| {
            let description = tool["description"].as_str().unwrap_or("");
            let purpose = description
                .lines()
                .map(str::trim)
                .find(|s| !s.is_empty())
                .unwrap_or("(no description)");
            let risk = RISK
                .captures(description)
                .map(|c| c[1].to_lowercase())
                .unwrap_or_else(|| "unknown".into());
            json!({"name":tool["name"],"purpose":purpose,"risk":risk})
        })
        .collect::<Vec<_>>();
    tools.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let mut value = probe.clone();
    value["python_version"] = json!("not applicable (native Rust MCP)");
    value["intents"] = crate::intents::summary()["intents"].clone();
    value["tool_count"] = json!(tools.len());
    value["tools"] = json!(tools);
    value
}
pub(crate) fn detect(registry: &Registry, host: Option<&crate::live_probe::Inputs>) -> Value {
    let limit = registry.workspace.max_output;
    let mut notes = Vec::new();
    let binary = process::inkscape_binary();
    let mut ver = None;
    let mut tuple = None;
    let mut ids = Vec::new();
    let mut types = Vec::new();
    let mut system = None;
    let mut user = None;
    if binary.is_none() {
        notes.push("inkscape not found on PATH".into());
    } else {
        if let Some(text) = cli(&["--version"], &mut notes, limit) {
            tuple = version(&text);
            ver = clean(Some(text));
            if tuple.is_none() {
                notes.push("could not parse inkscape version string".into());
            }
        }
        if let Some(text) = cli(&["--action-list"], &mut notes, limit) {
            ids = actions::parse_actions(&text);
            if ids.is_empty() {
                notes.push("inkscape --action-list returned no parseable actions".into());
            }
        }
        if let Some(text) = cli(&["--help"], &mut notes, limit) {
            types = exports(&text);
            if types.is_empty() {
                notes.push("could not parse --export-type list from inkscape --help".into());
            }
        }
        system = clean(cli(&["--system-data-directory"], &mut notes, limit));
        user = clean(cli(&["--user-data-directory"], &mut notes, limit));
    }
    let data_dirs: Vec<_> = [system.as_ref(), user.as_ref()]
        .into_iter()
        .flatten()
        .collect();
    let mut inkex_path = None;
    let mut inkex_version = None;
    for dir in &data_dirs {
        let base = PathBuf::from(dir);
        let relative = Path::new("extensions/inkex/__init__.py");
        let candidate = base.join(relative);
        if !candidate.is_file() {
            continue;
        }
        inkex_path = Some(candidate.to_string_lossy().into_owned());
        let read = base
            .canonicalize()
            .map_err(|_| "inkex data directory unavailable".to_string())
            .and_then(|root| {
                Workspace {
                    roots: vec![root],
                    max_input: registry.workspace.max_input,
                    max_output: limit,
                }
                .read(0, relative, registry.workspace.max_input)
            });
        match read {
            Ok(bytes) => {
                inkex_version = INKEX
                    .captures(&String::from_utf8_lossy(&bytes))
                    .map(|c| c[1].to_string());
                if inkex_version.is_none() {
                    notes.push("inkex __version__ not found in sources".into());
                }
            }
            Err(_) => {
                notes.push("inkex __init__.py unreadable: bounded no-follow read refused".into())
            }
        }
        break;
    }
    if inkex_path.is_none() {
        notes.push("inkex not found under any data dir".into());
    }
    let bus = host.map(|h| h.session_bus).unwrap_or_else(|| {
        std::env::var("DBUS_SESSION_BUS_ADDRESS").is_ok_and(|s| !s.trim().is_empty())
    });
    let mut present = false;
    if !bus {
        notes.push("no DBUS_SESSION_BUS_ADDRESS; session bus unavailable".into());
    } else if let Some(gdbus) = host
        .and_then(|h| h.binary.clone())
        .or_else(|| process::binary("gdbus"))
    {
        let address = host.and_then(|h| h.address.as_deref());
        // Query the broker first. Do not send to an unowned service name: gdbus
        // otherwise permits D-Bus activation, contrary to the no-launch guarantee.
        let owner = probe(
            &gdbus,
            &addressed(
                &[
                    "call",
                    "--session",
                    "--dest",
                    "org.freedesktop.DBus",
                    "--object-path",
                    "/org/freedesktop/DBus",
                    "--method",
                    "org.freedesktop.DBus.GetNameOwner",
                    "org.inkscape.Inkscape",
                ],
                address,
            ),
            "gdbus org.freedesktop.DBus.GetNameOwner",
            &mut notes,
            limit,
            false,
        );
        let owner = owner.and_then(|s| owner_name(&s));
        if let Some(owner) = owner {
            present = probe(
                &gdbus,
                &addressed(
                    &[
                        "call",
                        "--session",
                        "--dest",
                        &owner,
                        "--object-path",
                        "/org/inkscape/Inkscape",
                        "--method",
                        "org.gtk.Actions.List",
                    ],
                    address,
                ),
                "gdbus org.gtk.Actions.List",
                &mut notes,
                limit,
                false,
            )
            .is_some();
        }
    } else {
        notes.push("gdbus unavailable; cannot probe live Inkscape on session bus".into());
    }
    let live = data_dirs.iter().any(|dir| {
        Path::new(dir)
            .join("extensions/inkscape_mcp_live_run.sh")
            .is_file()
    });
    let fonts = if let Some(fc) = process::binary("fc-list") {
        let count = probe(&fc, &[], "fc-list", &mut notes, limit, true)
            .map(|text| text.lines().filter(|s| !s.trim().is_empty()).count())
            .unwrap_or(0);
        if count == 0 {
            notes.push("font count is 0; fontconfig may be broken".into());
        }
        count
    } else {
        notes.push("fc-list unavailable; font count is 0".into());
        0
    };
    json!({"inkscape_available":binary.is_some(),"inkscape_binary":binary.as_ref().map(|p|p.to_string_lossy().into_owned()),"inkscape_version":ver,"inkscape_version_tuple":tuple,"meets_minimum":tuple.is_some_and(|t|t>=[1,3,0]),"has_export_actions":ids.iter().any(|s|s.starts_with("export-")),"has_object_actions":ids.iter().any(|s|s.starts_with("object-")),"has_path_actions":ids.iter().any(|s|s.starts_with("path-")),"has_select_actions":ids.iter().any(|s|s.starts_with("select")),"actions":ids,"export_types":types,"shell_mode_available":binary.is_some(),"system_data_dir":system,"user_data_dir":user,"inkex_path":inkex_path,"inkex_version":inkex_version,"dbus_session_bus":bus,"dbus_inkscape_present":present,"live_extension_socket_available":live,"font_count":fonts,"probed_at":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros,true),"notes":notes})
}
pub fn discovery(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let caps = detect(registry, None);
    let mut result = json!({"inkscape_available":caps["inkscape_available"],"inkscape_version":caps["inkscape_version"],"notes":caps["notes"]});
    if tool == "list_actions" {
        let mut allowed: Vec<_> = actions::allowlist().into_iter().collect();
        allowed.sort();
        let ids = caps["actions"].as_array().unwrap();
        let available: Vec<_> = allowed
            .iter()
            .filter(|s| ids.iter().any(|v| v.as_str() == Some(s)))
            .collect();
        result["allowlisted"] = json!(allowed);
        result["available"] = json!(available);
        result["action_count"] = json!(ids.len());
        result["actions"] = if arguments::boolean(args, "include_all_actions", true)? {
            caps["actions"].clone()
        } else {
            json!([])
        };
        let _ = actions::map_from(registry, Some(&caps));
    } else {
        let mut allowed: Vec<_> = std::env::var_os("INKSCAPE_MCP_EXTENSION_ALLOWLIST")
            .map(|s| {
                std::env::split_paths(&s)
                    .map(|p| p.to_string_lossy().trim().to_owned())
                    .filter(|s| TOKEN.is_match(s))
                    .collect()
            })
            .unwrap_or_default();
        allowed.sort();
        allowed.dedup();
        if allowed.is_empty() {
            result["notes"].as_array_mut().unwrap().push(json!("extension execution is opt-in and OFF by default: the allowlist is empty until an operator adds extension ids to `extension_allowlist`; no extension can execute while it is empty (sec.12)."));
        }
        result["allowlisted"] = json!(allowed);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_overlay_is_sorted_refreshed_and_does_not_change_cached_probe() {
        let probe = json!({"probed_at":"cached timestamp","notes":[],"actions":["select-all"]});
        let mut contract = json!({"tools/list":{"tools":[
            {"name":"z","description":"\n  Purpose one  \nRisk Class: HIGH (approval)"},
            {"name":"a","description":"   "}
        ]}});
        let first = overlay(&probe, &contract);
        assert_eq!(first["tool_count"], 2);
        assert_eq!(
            first["tools"][0],
            json!({"name":"a","purpose":"(no description)","risk":"unknown"})
        );
        assert_eq!(
            first["tools"][1],
            json!({"name":"z","purpose":"Purpose one","risk":"high"})
        );
        contract["tools/list"]["tools"]
            .as_array_mut()
            .unwrap()
            .pop();
        let next = overlay(&probe, &contract);
        assert_eq!(next["tool_count"], 1);
        assert_eq!(next["probed_at"], probe["probed_at"]);
        assert_eq!(next["intents"].as_array().unwrap().len(), 49);
        assert_eq!(next["python_version"], "not applicable (native Rust MCP)");
        assert!(probe.get("tools").is_none());
    }
    #[test]
    fn versions_exports_and_unique_owner_are_parsed_without_activation_targets() {
        assert_eq!(version("Inkscape 1.4 (fixture)"), Some([1, 4, 0]));
        assert_eq!(version("Inkscape 1.4.3 (fixture)"), Some([1, 4, 3]));
        assert_eq!(version("unparsed"), None);
        assert_eq!(
            exports("--export-type=TYPE\n types [svg,png,pdf,svg wmf]"),
            ["svg", "png", "pdf", "wmf"]
        );
        assert_eq!(owner_name("(':1.23',)"), Some(":1.23".into()));
        for invalid in [
            "(true,)",
            "('org.inkscape.Inkscape',)",
            "(':1.23;quit',)",
            "(':1.23', 'extra')",
            "(':1.23',) trailing",
        ] {
            assert!(owner_name(invalid).is_none());
        }
    }
}
