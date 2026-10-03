//! Explicit bounded retention; never part of a mutating tool's implicit flow.
use crate::{
    document::{Entry, Registry},
    transaction,
    workspace::Workspace,
};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
fn setting(key: &str, default: i64) -> i64 {
    std::env::var(format!("INKSCAPE_MCP_{key}"))
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}
fn clock(text: &str) -> Option<f64> {
    chrono::DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|d| d.timestamp() as f64 + d.timestamp_subsec_nanos() as f64 / 1e9)
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|d| {
                    d.and_utc().timestamp() as f64
                        + d.and_utc().timestamp_subsec_nanos() as f64 / 1e9
                })
        })
}
fn selected(
    entries: &[Value],
    now: f64,
    keep_n: i64,
    days: i64,
    hard_n: i64,
    bytes: i64,
) -> HashSet<String> {
    let mut ordered = entries.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|e| e["seq"].as_i64().unwrap_or(0));
    let cut = if keep_n > 0 {
        ordered.len().saturating_sub(keep_n as usize)
    } else {
        ordered.len()
    };
    let mut kept = ordered
        .iter()
        .enumerate()
        .filter(|(i, e)| {
            *i >= cut
                || (days > 0
                    && e["created_at"]
                        .as_str()
                        .and_then(clock)
                        .is_none_or(|stamp| stamp >= now - days as f64 * 86400.))
        })
        .map(|(_, e)| *e)
        .collect::<Vec<_>>();
    if hard_n >= 0 && kept.len() > hard_n as usize {
        kept.drain(..kept.len() - hard_n as usize);
    }
    if bytes >= 0 {
        let mut total = kept
            .iter()
            .map(|e| e["size_bytes"].as_u64().unwrap_or(0) as u128)
            .sum::<u128>();
        let mut cut = 0;
        while total > bytes as u128 && cut < kept.len() {
            total -= kept[cut]["size_bytes"].as_u64().unwrap_or(0) as u128;
            cut += 1;
        }
        kept.drain(..cut);
    }
    kept.into_iter()
        .filter_map(|e| e["snapshot_id"].as_str().map(str::to_owned))
        .collect()
}
fn files(ws: &Workspace, root: usize, dir: &Path) -> Result<Vec<String>, String> {
    match ws.file_kind(root, dir) {
        Ok(Some(kind)) if kind == libc::S_IFDIR => (),
        _ => return Ok(vec![]),
    }
    let mut files = ws
        .directory_entries(root, dir)?
        .into_iter()
        .filter(|(_, kind)| *kind == libc::S_IFREG)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
fn record_valid(record: &Value, live: bool) -> bool {
    let required = if live {
        vec![
            "operation_id",
            "tool",
            "risk_class",
            "created_at",
            "updated_at",
        ]
    } else {
        vec![
            "operation_id",
            "doc_id",
            "tool",
            "risk_class",
            "created_at",
            "updated_at",
        ]
    };
    let maps = ["policy_decision", "previews"]
        .iter()
        .all(|key| record.get(*key).is_none_or(Value::is_object));
    let arrays = if live {
        vec!["selection", "affected_ids", "diff_artifacts"]
    } else {
        vec!["artifacts", "logs"]
    };
    let lists = arrays.iter().all(|key| {
        record.get(*key).is_none_or(|v| {
            v.as_array()
                .is_some_and(|items| items.iter().all(Value::is_string))
        })
    });
    let previews = record.get("previews").is_none_or(|v| {
        v.as_object()
            .is_some_and(|map| map.values().all(Value::is_string))
    });
    let invocation = live
        || record
            .get("invocation")
            .is_none_or(|v| v.is_null() || v.is_object());
    maps && lists
        && previews
        && invocation
        && required.iter().all(|k| record[*k].is_string())
        && record["params"].is_object()
        && record["risk_class"]
            .as_str()
            .is_some_and(|s| ["low", "medium", "high", "restricted"].contains(&s))
        && record.get("status").is_none_or(|s| {
            s.as_str()
                .is_some_and(|s| ["proposed", "applied", "discarded", "reverted"].contains(&s))
        })
        && record
            .get("snapshot_id")
            .is_none_or(|v| v.is_null() || v.is_string())
}
fn read_record(ws: &Workspace, root: usize, path: &Path) -> Option<Value> {
    ws.read(root, path, ws.max_output.min(4 * 1024 * 1024))
        .ok()
        .and_then(|v| serde_json::from_slice(&v).ok())
}
fn snapshots(registry: &Registry, entry: &Entry, now: f64) -> Result<Value, String> {
    let entries = transaction::list_snapshots(registry, entry)?;
    if entries.len() > 10000 {
        return Err("snapshot entry limit exceeded".into());
    }
    if entries.iter().any(|e| {
        !e["snapshot_id"].is_string()
            || !e["file"].is_string()
            || !e["created_at"].is_string()
            || e["seq"].as_i64().is_none()
            || e["size_bytes"].as_u64().is_none()
    }) {
        return Err("snapshot index could not be parsed".into());
    }
    let keep = selected(
        &entries,
        now,
        setting("SNAPSHOT_KEEP_N", 50),
        setting("SNAPSHOT_KEEP_DAYS", 30),
        setting("SNAPSHOT_HARD_MAX_N", 500),
        setting("SNAPSHOT_HARD_MAX_BYTES", 5 * 1024 * 1024 * 1024),
    );
    let mut kept = vec![];
    let mut removed = vec![];
    let mut freed = 0u64;
    for e in entries {
        let Some(id) = e["snapshot_id"].as_str() else {
            return Err("snapshot index could not be parsed".into());
        };
        let Some(file) = e["file"].as_str() else {
            return Err("snapshot index could not be parsed".into());
        };
        if keep.contains(id)
            || file.is_empty()
            || Path::new(file).file_name().and_then(|v| v.to_str()) != Some(file)
            || file.contains('\0')
        {
            kept.push(e);
            continue;
        }
        match registry.workspace.regular_info(
            entry.root,
            &entry.directory().join("snapshots").join(file),
            true,
        ) {
            Ok(info) => {
                freed = freed.saturating_add(info.map(|s| s.0).unwrap_or(0));
                removed.push(id.to_owned());
            }
            Err(_) => kept.push(e),
        }
    }
    if !removed.is_empty() {
        registry.workspace.atomic_write(
            entry.root,
            &entry.directory().join("snapshots/index.json"),
            &serde_json::to_vec_pretty(&json!({"snapshots":kept}))
                .map_err(|_| "snapshot index serialization failed")?,
        )?;
    }
    let retained = kept
        .iter()
        .filter_map(|e| e["snapshot_id"].as_str())
        .collect::<HashSet<_>>();
    let mut operations = vec![];
    if !kept.is_empty() || !removed.is_empty() {
        let directory = entry.directory().join("operations");
        for name in files(&registry.workspace, entry.root, &directory)? {
            if !name.starts_with("op_") || !name.ends_with(".json") {
                continue;
            }
            let path = directory.join(name);
            let Some(record) = read_record(&registry.workspace, entry.root, &path)
                .filter(|v| record_valid(v, false))
            else {
                continue;
            };
            if let Some(snap) = record["snapshot_id"].as_str()
                && !retained.contains(snap)
                && registry
                    .workspace
                    .regular_info(entry.root, &path, true)
                    .is_ok()
            {
                operations.push(record["operation_id"].as_str().unwrap().to_owned());
            }
        }
    }
    Ok(
        json!({"doc_id":entry.id,"pruned_snapshot_ids":removed,"pruned_operation_ids":operations,"retained_count":kept.len(),"freed_bytes":freed,"live_frames":null}),
    )
}
fn live(ws: &Workspace, root: usize, now: f64) -> Result<Value, String> {
    let dir = PathBuf::from(".inkscape-mcp/live/artifacts");
    let ops = PathBuf::from(".inkscape-mcp/live/operations");
    let mut protected = HashSet::new();
    for name in files(ws, root, &ops)? {
        if !name.starts_with("op_") || !name.ends_with(".json") {
            continue;
        }
        let Some(record) = read_record(ws, root, &ops.join(name)).filter(|r| record_valid(r, true))
        else {
            continue;
        };
        let mut refs = record["previews"]
            .as_object()
            .map(|p| p.values().collect::<Vec<_>>())
            .unwrap_or_default();
        if let Some(diff) = record["diff_artifacts"].as_array() {
            refs.extend(diff);
        }
        for val in refs {
            if let Some(name) = val
                .as_str()
                .and_then(|s| Path::new(s).file_name())
                .and_then(|n| n.to_str())
            {
                protected.insert(name.to_owned());
            }
        }
    }
    let mut frames = vec![];
    for name in files(ws, root, &dir)? {
        if !name.starts_with("live-view-") || !name.ends_with(".png") || protected.contains(&name) {
            continue;
        }
        if let Ok(Some((size, mtime))) = ws.regular_info(root, &dir.join(&name), false) {
            frames.push((name, size, mtime));
        }
    }
    frames.sort_by(|a, b| b.2.total_cmp(&a.2));
    let days = setting("LIVE_FRAME_KEEP_DAYS", 7);
    let max = setting("LIVE_FRAME_MAX_BYTES", 512 * 1024 * 1024);
    let (mut pruned, mut freed, mut retained, mut running) = (0u64, 0u64, 0u64, 0u128);
    for (name, size, mtime) in frames {
        let old = days > 0 && mtime < now - days as f64 * 86400.;
        let over = max > 0 && running + size as u128 > max as u128;
        if old || over {
            match ws.regular_info(root, &dir.join(name), true) {
                Ok(info) => {
                    pruned += 1;
                    freed = freed.saturating_add(info.map(|v| v.0).unwrap_or(0));
                }
                Err(_) => {
                    retained += 1;
                    running += size as u128;
                }
            }
        } else {
            retained += 1;
            running += size as u128;
        }
    }
    Ok(json!({"pruned_frames":pruned,"freed_bytes":freed,"retained_frames":retained}))
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    crate::recovery::recover_document(registry, entry)?;
    let now = chrono::Utc::now().timestamp_micros() as f64 / 1e6;
    let mut result = snapshots(registry, entry, now)?;
    result["live_frames"] = live(&registry.workspace, entry.root, now)?;
    Ok(result)
}

/// Explicit startup maintenance over persisted documents, independent of this
/// process's registry. Each root/document is isolated; no GUI or engine is used.
pub fn sweep(registry: &Registry) -> Value {
    sweep_at(registry, chrono::Utc::now().timestamp_micros() as f64 / 1e6)
}

fn sweep_at(registry: &Registry, now: f64) -> Value {
    let mut documents = vec![];
    let mut frames = vec![];
    let mut failures = 0usize;
    for root in 0..registry.workspace.roots.len() {
        match registry
            .workspace
            .file_kind(root, Path::new(".inkscape-mcp"))
        {
            Ok(None) => continue,
            Ok(Some(kind)) if kind == libc::S_IFDIR => (),
            _ => {
                failures += 1;
                continue;
            }
        }
        let directory = Path::new(".inkscape-mcp/documents");
        let mut children = match registry.workspace.file_kind(root, directory) {
            Ok(None) => vec![],
            Ok(Some(kind)) if kind == libc::S_IFDIR => {
                match registry.workspace.directory_entries(root, directory) {
                    Ok(entries) => entries,
                    Err(_) => {
                        failures += 1;
                        vec![]
                    }
                }
            }
            _ => {
                failures += 1;
                vec![]
            }
        };
        children.sort_by(|a, b| a.0.cmp(&b.0));
        for (id, kind) in children {
            if kind != libc::S_IFDIR {
                continue;
            }
            let entry = Entry {
                id,
                root,
                source: String::new(),
                opened_at: String::new(),
            };
            if crate::recovery::recover_document(registry, &entry).is_err() {
                failures += 1;
                continue;
            }
            let index = entry.directory().join("snapshots/index.json");
            // Skip missing and nonregular indexes; descriptor descent refuses
            // linked/swapped ancestors before any snapshot or record mutation.
            if !matches!(
                registry.workspace.file_kind(root, &index),
                Ok(Some(libc::S_IFREG))
            ) {
                continue;
            }
            match snapshots(registry, &entry, now) {
                Ok(result) => documents.push(result),
                Err(_) => failures += 1,
            }
        }
        match live(&registry.workspace, root, now) {
            Ok(result) => frames.push(result),
            Err(_) => failures += 1,
        }
    }
    json!({"documents": documents, "live_frames": frames, "failures": failures})
}
#[cfg(test)]
mod tests {
    use super::*;

    fn persisted_fixture() -> (tempfile::TempDir, Registry, Vec<Value>) {
        let root = tempfile::tempdir().unwrap();
        let registry = Registry {
            workspace: Workspace {
                roots: vec![root.path().canonicalize().unwrap()],
                max_input: 52428800,
                max_output: 104857600,
            },
            entries: indexmap::IndexMap::new(),
        };
        let directory = Path::new(".inkscape-mcp/documents/d_prior");
        for name in ["original.svg", "working/document.svg"] {
            registry
                .workspace
                .write_new(0, &directory.join(name), b"preserved drawing")
                .unwrap();
        }
        let entries = (0..60).map(|seq| {
            let name = format!("snap_{seq:08x}.svg");
            registry.workspace.write_new(0, &directory.join("snapshots").join(&name), b"snapshot").unwrap();
            json!({"snapshot_id":format!("snap_{seq:08x}"),"file":name,"seq":seq,"created_at":"2000-01-01T00:00:00+00:00","size_bytes":8})
        }).collect::<Vec<_>>();
        registry
            .workspace
            .write_new(
                0,
                &directory.join("snapshots/index.json"),
                &serde_json::to_vec(&json!({"snapshots":entries})).unwrap(),
            )
            .unwrap();
        registry.workspace.write_new(0, &directory.join("operations/op_00000001.json"), &serde_json::to_vec(&json!({"operation_id":"op_00000001","doc_id":"d_prior","tool":"set_fill","risk_class":"medium","params":{},"snapshot_id":"snap_00000000","created_at":"2000-01-01T00:00:00+00:00","updated_at":"2000-01-01T00:00:00+00:00"})).unwrap()).unwrap();
        (root, registry, entries)
    }

    #[test]
    fn startup_empty_root_is_read_only_and_not_a_failure() {
        let root = tempfile::tempdir().unwrap();
        let registry = Registry {
            workspace: Workspace {
                roots: vec![root.path().canonicalize().unwrap()],
                max_input: 100,
                max_output: 100,
            },
            entries: indexmap::IndexMap::new(),
        };
        let result = sweep_at(&registry, 2e9);
        assert_eq!(
            result,
            json!({"documents":[], "live_frames":[], "failures":0})
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn startup_prunes_prior_session_without_registering_or_touching_drawings() {
        let (_root, registry, _) = persisted_fixture();
        let directory = Path::new(".inkscape-mcp/documents/d_prior");
        let result = sweep_at(&registry, 2e9);
        assert_eq!(result["failures"], 0);
        assert_eq!(result["documents"][0]["retained_count"], 50);
        assert_eq!(
            result["documents"][0]["pruned_snapshot_ids"]
                .as_array()
                .unwrap()
                .len(),
            10
        );
        assert_eq!(
            result["documents"][0]["pruned_operation_ids"],
            json!(["op_00000001"])
        );
        assert!(registry.entries.is_empty());
        for name in ["original.svg", "working/document.svg"] {
            assert_eq!(
                registry
                    .workspace
                    .read(0, &directory.join(name), 100)
                    .unwrap(),
                b"preserved drawing"
            );
        }
        let repeated = sweep_at(&registry, 2e9);
        assert_eq!(repeated["documents"][0]["pruned_snapshot_ids"], json!([]));
    }

    #[test]
    fn startup_skips_bad_and_linked_documents_but_continues_other_roots_and_frames() {
        use std::os::unix::fs::symlink;
        let (root, mut registry, _) = persisted_fixture();
        let external = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        std::fs::write(external.path().join("sentinel"), b"external drawing").unwrap();
        symlink(
            external.path(),
            root.path().join(".inkscape-mcp/documents/a_link"),
        )
        .unwrap();
        registry
            .workspace
            .write_new(
                0,
                Path::new(".inkscape-mcp/documents/a_bad/snapshots/index.json"),
                b"{broken",
            )
            .unwrap();
        registry
            .workspace
            .roots
            .push(other.path().canonicalize().unwrap());
        registry
            .workspace
            .write_new(
                1,
                Path::new(".inkscape-mcp/live/artifacts/live-view-old.png"),
                b"old frame",
            )
            .unwrap();
        registry
            .workspace
            .write_new(
                1,
                Path::new(".inkscape-mcp/live/artifacts/selection-export.png"),
                b"keep",
            )
            .unwrap();
        let result = sweep_at(&registry, 2e9);
        assert_eq!(result["failures"], 1);
        assert_eq!(result["documents"].as_array().unwrap().len(), 1);
        assert_eq!(result["documents"][0]["retained_count"], 50);
        assert_eq!(result["live_frames"][1]["pruned_frames"], 1);
        assert_eq!(
            std::fs::read(external.path().join("sentinel")).unwrap(),
            b"external drawing"
        );
        assert_eq!(
            registry
                .workspace
                .read(
                    1,
                    Path::new(".inkscape-mcp/live/artifacts/selection-export.png"),
                    100
                )
                .unwrap(),
            b"keep"
        );
        assert!(
            root.path()
                .join(".inkscape-mcp/documents/a_bad/snapshots/index.json")
                .is_file()
        );
    }

    #[test]
    fn keep_union_caps_and_bad_clock_match_reference() {
        let entries=(0..5).map(|i|json!({"snapshot_id":format!("snap_{i}"),"seq":i,"created_at":if i==0 {"bad"}else{"2000-01-01T00:00:00"},"size_bytes":10})).collect::<Vec<_>>();
        assert_eq!(
            selected(&entries, 2e9, 2, 1, -1, -1),
            HashSet::from(["snap_0".into(), "snap_3".into(), "snap_4".into()])
        );
        assert_eq!(
            selected(&entries, 2e9, 5, 0, 2, 10),
            HashSet::from(["snap_4".into()])
        );
        assert!(selected(&entries, 2e9, 5, 0, 0, -1).is_empty());
    }
}
