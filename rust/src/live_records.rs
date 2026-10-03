//! Bounded root-scoped live records; approvals precede persistence and every live mutation.
use crate::{transaction, workspace::Workspace};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};
#[derive(Serialize, Deserialize)]
struct Record {
    operation_id: String,
    tool: String,
    risk_class: String,
    params: serde_json::Map<String, Value>,
    #[serde(default)]
    transport: Option<String>,
    #[serde(default)]
    document: Option<Value>,
    #[serde(default)]
    selection: Vec<String>,
    #[serde(default)]
    policy_decision: serde_json::Map<String, Value>,
    #[serde(default)]
    affected_ids: Vec<String>,
    #[serde(default)]
    undo_friendly: bool,
    #[serde(default)]
    completion_uncertain: bool,
    #[serde(default)]
    previews: serde_json::Map<String, Value>,
    #[serde(default)]
    diff_artifacts: Vec<String>,
    #[serde(default = "proposed")]
    status: String,
    created_at: String,
    updated_at: String,
}
fn proposed() -> String {
    "proposed".into()
}
pub(crate) fn id(id: &str) -> bool {
    id.len() == 11
        && id.starts_with("op_")
        && id[3..]
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn path(id_value: &str) -> Result<PathBuf, String> {
    if !id(id_value) {
        return Err("invalid live operation id".into());
    }
    Ok(Path::new(".inkscape-mcp/live/operations").join(format!("{id_value}.json")))
}
fn resolved(path: &Path) -> Option<PathBuf> {
    let mut prefix = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let mut tail = Vec::new();
    while !prefix.exists() {
        if tail.len() >= 256 {
            return None;
        }
        tail.push(prefix.file_name()?.to_owned());
        if !prefix.pop() {
            return None;
        }
    }
    let mut result = prefix.canonicalize().ok()?;
    for part in tail.into_iter().rev() {
        result.push(part);
    }
    let mut normalized = PathBuf::new();
    for part in result.components() {
        match part {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => (),
            _ => normalized.push(part),
        }
    }
    Some(normalized)
}
pub fn sanitize(document: &Value, workspace: &Workspace) -> Value {
    let mut document = document.clone();
    if let Some(raw) = document["path"].as_str() {
        if raw.len() > 8192 {
            document["path"] = json!("<external>");
            return document;
        }
        let relative = resolved(Path::new(raw)).and_then(|resolved| {
            workspace.roots.iter().find_map(|root| {
                resolved
                    .strip_prefix(root)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            })
        });
        document["path"] = json!(relative.unwrap_or("<external>".into()));
    }
    document
}
fn validate(value: Value) -> Result<Value, String> {
    let mut record: Record =
        serde_json::from_value(value).map_err(|_| "invalid live operation record")?;
    if !id(&record.operation_id)
        || !matches!(
            record.risk_class.as_str(),
            "low" | "medium" | "high" | "restricted"
        )
        || !matches!(
            record.status.as_str(),
            "proposed" | "applied" | "discarded" | "reverted"
        )
        || record.previews.values().any(|v| !v.is_string())
    {
        return Err("invalid live operation record".into());
    }
    if let Some(document) = record.document.as_mut() {
        if !document.is_object() {
            return Err("invalid live operation document".into());
        }
        for field in ["window_id", "document_id", "name", "path"] {
            if !document[field].is_null() && !document[field].is_string() {
                return Err("invalid live operation document".into());
            }
        }
        if !document["object_count"].is_null()
            && !(document["object_count"].is_number()
                && !document["object_count"]
                    .to_string()
                    .contains(['.', 'e', 'E']))
        {
            return Err("invalid live operation document".into());
        }
        let fields = ["window_id", "document_id", "name", "path", "object_count"];
        *document = Value::Object(
            fields
                .into_iter()
                .map(|key| (key.into(), document[key].clone()))
                .collect(),
        );
    }
    serde_json::to_value(record).map_err(|_| "invalid live operation record".into())
}
pub fn persist(workspace: &Workspace, record: &Value) -> Result<(), String> {
    let record = validate(record.clone())?;
    let bytes =
        serde_json::to_vec_pretty(&record).map_err(|_| "live record serialization failed")?;
    if bytes.len() > workspace.max_output {
        return Err("live operation record exceeds size limit".into());
    }
    for directory in [
        ".inkscape-mcp/live/artifacts",
        ".inkscape-mcp/live/operations",
    ] {
        workspace
            .ensure_directory(0, Path::new(directory))
            .map_err(|_| "could not store live operation record")?;
    }
    workspace
        .atomic_write(0, &path(record["operation_id"].as_str().unwrap())?, &bytes)
        .map_err(|_| "could not store live operation record".into())
}
pub fn new(
    workspace: &Workspace,
    tool: &str,
    params: Value,
    transport: &str,
    document: &Value,
    selection: Vec<String>,
    approval: Option<&str>,
) -> Result<Value, String> {
    transaction::policy("high", approval)?;
    if workspace.roots.is_empty() {
        return Err("live operation failed".into());
    }
    let now = transaction::now();
    for _ in 0..4 {
        let operation_id = transaction::token("op_");
        let directory = Path::new(".inkscape-mcp/live/operations");
        workspace
            .ensure_directory(0, directory)
            .map_err(|_| "could not store live operation record")?;
        if workspace
            .file_kind(0, &path(&operation_id)?)
            .map_err(|_| "could not store live operation record")?
            .is_some()
        {
            continue;
        }
        let record = json!({"operation_id":operation_id,"tool":tool,"risk_class":"high","params":params,"transport":transport,"document":sanitize(document,workspace),"selection":selection,"policy_decision":{"risk_class":"high","permitted":true,"approval_required":true,"approved":true},"affected_ids":[],"undo_friendly":false,"completion_uncertain":false,"previews":{},"diff_artifacts":[],"status":"proposed","created_at":now,"updated_at":now});
        persist(workspace, &record)?;
        return Ok(record);
    }
    Err("could not store live operation record".into())
}
pub fn update(workspace: &Workspace, record: &mut Value, changes: Value) -> Result<(), String> {
    let mut updated = record.clone();
    for (key, value) in changes.as_object().ok_or("invalid live operation update")? {
        updated[key] = value.clone();
    }
    updated["updated_at"] = json!(transaction::now());
    updated["document"] = sanitize(&updated["document"], workspace);
    persist(workspace, &updated)?;
    *record = updated;
    Ok(())
}
pub fn get(workspace: &Workspace, operation: &str) -> Result<Value, String> {
    let path = path(operation).map_err(|_| "no live operation with that id")?;
    let bytes = workspace
        .read(0, &path, workspace.max_output)
        .map_err(|_| "no live operation with that id")?;
    validate(serde_json::from_slice(&bytes).map_err(|_| "invalid live operation record")?)
}
pub fn list(workspace: &Workspace) -> Value {
    let directory = Path::new(".inkscape-mcp/live/operations");
    let mut files = Vec::new();
    if let Ok(entries) = workspace.directory_entries(0, directory) {
        for (name, mode) in entries {
            if mode != libc::S_IFREG
                || !name.ends_with(".json")
                || !id(name.trim_end_matches(".json"))
            {
                continue;
            }
            let path = directory.join(&name);
            if let Ok(Some((_, mtime))) = workspace.regular_info(0, &path, false) {
                files.push((path, mtime));
            }
        }
    }
    files.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut operations = Vec::new();
    let mut used = 64usize;
    for (path, _) in files.into_iter().take(50) {
        let value = workspace
            .read(0, &path, workspace.max_output)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .and_then(|value| validate(value).ok());
        if let Some(value) = value {
            let size = serde_json::to_vec(&value)
                .map(|b| b.len() + 1)
                .unwrap_or(usize::MAX);
            if size > workspace.max_output.saturating_sub(used) {
                break;
            }
            used += size;
            operations.push(value);
        }
    }
    json!({"count":operations.len(),"operations":operations})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approvals_precede_writes_and_paths_are_redacted_before_persistence() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let original = root.path().join("original.svg");
        std::fs::write(&original, b"original").unwrap();
        let document = json!({"name":"Drawing.svg","path":original,"object_count":1});
        for approval in [
            None,
            Some(""),
            Some(" \t\n"),
            Some("\u{1c}\u{1d}\u{1e}\u{1f}"),
        ] {
            assert!(
                new(
                    &workspace,
                    "live_apply_to_selection",
                    json!({}),
                    "test",
                    &document,
                    vec![],
                    approval
                )
                .is_err()
            );
            assert!(!root.path().join(".inkscape-mcp").exists());
        }
        let mut record = new(
            &workspace,
            "live_apply_to_selection",
            json!({}),
            "test",
            &document,
            vec!["r".into()],
            Some("approved"),
        )
        .unwrap();
        assert_eq!(record["document"]["path"], "original.svg");
        let created = record["created_at"].clone();
        update(
            &workspace,
            &mut record,
            json!({"status":"discarded","completion_uncertain":true}),
        )
        .unwrap();
        let log = list(&workspace);
        assert_eq!(log["count"], 1);
        assert_eq!(log["operations"][0]["created_at"], created);
        assert_eq!(log["operations"][0]["completion_uncertain"], true);
        assert_eq!(
            sanitize(&json!({"path":"/unowned/drawing.svg"}), &workspace)["path"],
            "<external>"
        );
        assert_eq!(
            sanitize(&json!({"path":root.path().join("missing.svg")}), &workspace)["path"],
            "missing.svg"
        );
        assert_eq!(std::fs::read(&original).unwrap(), b"original");
    }
    #[test]
    fn no_follow_reads_and_record_output_budget_preserve_originals() {
        let root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let record = new(
            &workspace,
            "live_apply_to_selection",
            json!({"content":"x".repeat(500)}),
            "test",
            &Value::Null,
            vec![],
            Some("approved"),
        )
        .unwrap();
        let original = root.path().join("external.json");
        std::fs::write(&original, serde_json::to_vec(&record).unwrap()).unwrap();
        std::os::unix::fs::symlink(
            &original,
            root.path()
                .join(".inkscape-mcp/live/operations/op_deadbeef.json"),
        )
        .unwrap();
        assert_eq!(list(&workspace)["count"], 1);
        workspace.max_output = 400;
        assert_eq!(list(&workspace)["count"], 0);
        assert!(persist(&workspace, &record).is_err());
        assert!(original.exists());
        assert!(path("../external").is_err());
    }
}
