//! Shared edit transaction: policy, staged DOM, no-op, snapshots, preview and audit.

use crate::{
    document::{Entry, Registry},
    preview, xml,
};
use libxml::tree::Document;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

// Deterministic filesystem failure injection exists only in the test executable.
// Release builds have no hook, environment switch or executable callback surface.
#[cfg(test)]
thread_local! {
    static AFTER_WORKING_TEST: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
fn after_working_test() {
    AFTER_WORKING_TEST.with(|hook| {
        if let Some(action) = hook.borrow_mut().take() {
            action();
        }
    });
}

pub fn token(prefix: &str) -> String {
    format!(
        "{prefix}{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    )
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
}

fn record_path(entry: &Entry, operation: &str) -> std::path::PathBuf {
    entry
        .directory()
        .join("operations")
        .join(format!("{operation}.json"))
}

pub fn persist(registry: &Registry, entry: &Entry, record: &Value) -> Result<(), String> {
    let operation = record["operation_id"]
        .as_str()
        .ok_or("invalid operation record")?;
    registry.workspace.atomic_write(
        entry.root,
        &record_path(entry, operation),
        &serde_json::to_vec_pretty(record).map_err(|_| "operation record serialization failed")?,
    )
}

pub fn record(id: &str, tool: &str, risk: &str, params: Value, approved: bool) -> Value {
    let timestamp = now();
    json!({"operation_id":token("op_"),"doc_id":id,"tool":tool,"risk_class":risk,
        "params":params,"policy_decision":{"risk_class":risk,"permitted":true,
            "approval_required":risk=="high","approved":approved},
        "invocation":null,"snapshot_id":null,"artifacts":[],"previews":{},"logs":[],
        "status":"proposed","created_at":timestamp,"updated_at":timestamp})
}

pub fn policy(risk: &str, approval: Option<&str>) -> Result<(), String> {
    match risk {
        "restricted" => Err("restricted operations are not permitted".into()),
        "high"
            if !approval.is_some_and(|token| {
                !token
                    .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
                    .is_empty()
            }) =>
        {
            Err("high-risk operation requires explicit approval".into())
        }
        "low" | "medium" | "high" => Ok(()),
        _ => Err("invalid operation risk class".into()),
    }
}

pub fn list_snapshots(registry: &Registry, entry: &Entry) -> Result<Vec<Value>, String> {
    let path = entry.directory().join("snapshots/index.json");
    match registry
        .workspace
        .read_optional(entry.root, &path, registry.workspace.max_output)?
    {
        None => Ok(Vec::new()),
        Some(bytes) => {
            let value: Value =
                serde_json::from_slice(&bytes).map_err(|_| "snapshot index could not be parsed")?;
            value["snapshots"]
                .as_array()
                .cloned()
                .ok_or("snapshot index could not be parsed".into())
        }
    }
}

pub fn snapshot(
    registry: &Registry,
    entry: &Entry,
    label: Option<&str>,
    operation: Option<&str>,
) -> Result<Value, String> {
    if operation.is_some_and(|id| !valid_id(id, "op_")) {
        return Err("invalid operation id".into());
    }
    if label.is_some_and(|label| label.chars().count() > 256) {
        return Err("snapshot label too long".into());
    }
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let mut snapshots = list_snapshots(registry, entry)?;
    let index = entry.directory().join("snapshots/index.json");
    let old_index =
        registry
            .workspace
            .read_optional(entry.root, &index, registry.workspace.max_output)?;
    let sequence = snapshots
        .iter()
        .filter_map(|snapshot| snapshot["seq"].as_u64())
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("snapshot sequence exceeds limit")?;
    let id = token("snap_");
    let filename = format!(
        "{sequence:04}-{}-{}.svg",
        operation.unwrap_or(&id),
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    );
    let info = json!({"snapshot_id":id,"seq":sequence,"file":filename,"created_at":now(),
        "label":label,"operation_id":operation,"size_bytes":bytes.len()});
    let path = entry.directory().join("snapshots").join(&filename);
    registry.workspace.write_new(entry.root, &path, &bytes)?;
    snapshots.push(info.clone());
    if let Err(error) = registry.workspace.atomic_write(
        entry.root,
        &index,
        &serde_json::to_vec_pretty(&json!({"snapshots":snapshots})).unwrap(),
    ) {
        // An fsync error can follow a successful rename. Restore the index before
        // deleting its new SVG; otherwise it might retain a dangling snapshot link.
        let rollback = match old_index {
            Some(bytes) => registry.workspace.atomic_write(entry.root, &index, &bytes),
            None => registry
                .workspace
                .read_optional(entry.root, &index, registry.workspace.max_output)
                .and_then(|current| {
                    if current.is_some() {
                        registry.workspace.remove_file(entry.root, &index)
                    } else {
                        Ok(())
                    }
                }),
        };
        if rollback.is_ok() {
            let _ = registry.workspace.remove_file(entry.root, &path);
            return Err(error);
        }
        return Err(
            "snapshot index persistence failed; snapshot bytes retained for recovery".into(),
        );
    }
    Ok(info)
}

/// Staged operations that can run a fixed CLI retain the pre-dispatch audit gate.
pub fn apply<F>(
    registry: &Registry,
    id: &str,
    tool: &str,
    params: Value,
    risk: &str,
    approval: Option<&str>,
    mutate: F,
) -> Result<Value, String>
where
    F: FnOnce(&mut Document) -> Result<String, String>,
{
    apply_with_mode::<true, F>(registry, id, tool, params, risk, approval, mutate)
}

/// Pure in-memory DOM staging can decide no-op before any new audit write.
/// Callers must not dispatch a CLI or publish files from their callback.
pub fn apply_dom<F>(
    registry: &Registry,
    id: &str,
    tool: &str,
    params: Value,
    risk: &str,
    approval: Option<&str>,
    mutate: F,
) -> Result<Value, String>
where
    F: FnOnce(&mut Document) -> Result<String, String>,
{
    apply_with_mode::<false, F>(registry, id, tool, params, risk, approval, mutate)
}

fn apply_with_mode<const PRE_AUDIT: bool, F>(
    registry: &Registry,
    id: &str,
    tool: &str,
    params: Value,
    risk: &str,
    approval: Option<&str>,
    mutate: F,
) -> Result<Value, String>
where
    F: FnOnce(&mut Document) -> Result<String, String>,
{
    policy(risk, approval)?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    crate::recovery::recover_document(registry, entry)?;
    let mut record = record(id, tool, risk, params, true);
    if PRE_AUDIT {
        persist(registry, entry, &record)?;
    } else {
        // Keep descriptor/symlink checks before staging without creating a file.
        registry
            .workspace
            .directory_file(entry.root, &entry.directory().join("operations"))?;
    }
    let operation = record["operation_id"].as_str().unwrap().to_owned();
    let bytes = registry
        .workspace
        .read(entry.root, &entry.working(), registry.workspace.max_input)
        .inspect_err(|_| {
            if !PRE_AUDIT {
                let _ = persist(registry, entry, &record);
            }
        })?;
    let mut document = xml::parse(&bytes, registry.workspace.max_input).inspect_err(|_| {
        if !PRE_AUDIT {
            let _ = persist(registry, entry, &record);
        }
    })?;
    let before = xml::serialize(&document);
    let summary = match mutate(&mut document) {
        Ok(summary) => summary,
        Err(error) => {
            record["status"] = json!("discarded");
            record["updated_at"] = json!(now());
            let _ = persist(registry, entry, &record);
            return Err(error);
        }
    };
    let after = xml::serialize(&document);
    if before == after {
        if PRE_AUDIT {
            registry
                .workspace
                .remove_file(entry.root, &record_path(entry, &operation))?;
        }
        return Ok(
            json!({"doc_id":id,"operation_id":"","snapshot_id":"","changed":false,
            "summary":format!("no change: {summary}"),"preview_before":null,"preview_after":null}),
        );
    }
    if after.len() > registry.workspace.max_input || after.is_empty() {
        record["status"] = json!("discarded");
        record["updated_at"] = json!(now());
        let _ = persist(registry, entry, &record);
        return Err("input file exceeds the configured size limit".into());
    }
    // Parse staged bytes before any document mutation; malformed serializer output fails closed.
    xml::parse(&after, registry.workspace.max_input).inspect_err(|_| {
        if !PRE_AUDIT {
            let _ = persist(registry, entry, &record);
        }
    })?;
    if !PRE_AUDIT {
        // Durable intent still precedes snapshots, previews and working-copy publication.
        persist(registry, entry, &record)?;
    }
    let preview_before =
        preview::operation_preview(&registry.workspace, entry, &operation, "before");
    let pre = snapshot(
        registry,
        entry,
        Some(&format!("pre-{tool}")),
        Some(&operation),
    )?;
    // Publish the recovery link and journal before replacing the head. A crash
    // can then be distinguished from a completed edit without guessing.
    record["snapshot_id"] = pre["snapshot_id"].clone();
    record["updated_at"] = json!(now());
    persist(registry, entry, &record)?;
    crate::recovery::prepare(registry, entry, &record, &bytes, &after)?;
    if let Err(error) = registry
        .workspace
        .atomic_write(entry.root, &entry.working(), &after)
    {
        // A failed post-rename sync can report an error after replacement. Always restore bytes.
        let rollback = registry
            .workspace
            .atomic_write(entry.root, &entry.working(), &bytes);
        record["status"] = json!("discarded");
        record["snapshot_id"] = pre["snapshot_id"].clone();
        record["updated_at"] = json!(now());
        let _ = persist(registry, entry, &record);
        return Err(if rollback.is_ok() {
            error
        } else {
            "edit persistence failed; restore the recorded snapshot before retrying".into()
        });
    }
    #[cfg(test)]
    after_working_test();
    let preview_after = preview::operation_preview(&registry.workspace, entry, &operation, "after");
    record["snapshot_id"] = pre["snapshot_id"].clone();
    record["status"] = json!("applied");
    record["updated_at"] = json!(now());
    if let Some(path) = &preview_before {
        record["previews"]["before"] = json!(path);
    }
    if let Some(path) = &preview_after {
        record["previews"]["after"] = json!(path);
    }
    if persist(registry, entry, &record).is_err() {
        let rollback = registry
            .workspace
            .atomic_write(entry.root, &entry.working(), &bytes);
        record["status"] = json!("discarded");
        let _ = persist(registry, entry, &record);
        return Err(if rollback.is_ok() {
            "operation record could not be persisted; edit rolled back".into()
        } else {
            "edit persistence failed; restore the recorded snapshot before retrying".into()
        });
    }
    // A durable applied record commits the edit; a failed journal cleanup is
    // harmless and is retried at the next explicit startup maintenance pass.
    let _ = crate::recovery::finish(registry, entry, &operation);
    Ok(
        json!({"doc_id":id,"operation_id":operation,"snapshot_id":pre["snapshot_id"],
        "changed":true,"summary":summary,"preview_before":preview_before,"preview_after":preview_after}),
    )
}

pub(crate) fn snapshot_source(
    registry: &Registry,
    entry: &Entry,
    snapshot_id: &str,
) -> Result<std::path::PathBuf, String> {
    if !valid_id(snapshot_id, "snap_") {
        return Err("snapshot not found".into());
    }
    let snapshots = list_snapshots(registry, entry)?;
    let target = snapshots
        .iter()
        .find(|snapshot| snapshot["snapshot_id"].as_str() == Some(snapshot_id))
        .ok_or("snapshot not found")?;
    let filename = target["file"].as_str().ok_or("snapshot not found")?;
    if Path::new(filename).components().count() != 1
        || !matches!(
            Path::new(filename).components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err("snapshot not found".into());
    }
    Ok(entry.directory().join("snapshots").join(filename))
}

pub fn restore(registry: &Registry, id: &str, snapshot_id: &str) -> Result<Value, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    crate::recovery::recover_document(registry, entry)?;
    let source = snapshot_source(registry, entry, snapshot_id)?;
    let bytes = registry
        .workspace
        .read(entry.root, &source, registry.workspace.max_input)?;
    let before =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let mut record = record(
        id,
        "restore_snapshot",
        "medium",
        json!({"snapshot_id":snapshot_id}),
        true,
    );
    persist(registry, entry, &record)?;
    let operation = record["operation_id"].as_str().unwrap().to_owned();
    let pre = snapshot(
        registry,
        entry,
        Some(&format!("pre-restore before {snapshot_id}")),
        Some(&operation),
    )?;
    record["snapshot_id"] = pre["snapshot_id"].clone();
    record["updated_at"] = json!(now());
    persist(registry, entry, &record)?;
    crate::recovery::prepare(registry, entry, &record, &before, &bytes)?;
    if let Err(error) = registry
        .workspace
        .atomic_write(entry.root, &entry.working(), &bytes)
    {
        let rollback = registry
            .workspace
            .atomic_write(entry.root, &entry.working(), &before);
        record["status"] = json!("discarded");
        record["updated_at"] = json!(now());
        let _ = persist(registry, entry, &record);
        return Err(if rollback.is_ok() {
            error
        } else {
            "restore persistence failed; restore the recorded snapshot before retrying".into()
        });
    }
    #[cfg(test)]
    after_working_test();
    record["status"] = json!("applied");
    record["updated_at"] = json!(now());
    if persist(registry, entry, &record).is_err() {
        let rollback = registry
            .workspace
            .atomic_write(entry.root, &entry.working(), &before);
        record["status"] = json!("discarded");
        record["updated_at"] = json!(now());
        let _ = persist(registry, entry, &record);
        return Err(if rollback.is_ok() {
            "operation record could not be persisted; restore rolled back".into()
        } else {
            "restore persistence failed; restore the recorded snapshot before retrying".into()
        });
    }
    let _ = crate::recovery::finish(registry, entry, &operation);
    Ok(
        json!({"doc_id":id,"restored_from":snapshot_id,"operation_id":operation,
        "pre_restore_snapshot_id":pre["snapshot_id"],"restored_sha256":format!("{:x}", Sha256::digest(&bytes)),
        "restored_size_bytes":bytes.len()}),
    )
}

fn valid_id(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == 8
            && hex
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{batch, style, workspace::Workspace};

    fn fixture() -> (tempfile::TempDir, Registry, String, Vec<u8>) {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![directory.path().canonicalize().unwrap()],
            max_input: 52428800,
            max_output: 104857600,
        };
        let original = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><rect id="r" width="10" height="10" fill="#123"/></svg>"##.to_vec();
        workspace
            .write_new(0, Path::new("fixture.svg"), &original)
            .unwrap();
        let mut registry = Registry {
            workspace,
            entries: indexmap::IndexMap::new(),
        };
        let id = registry.open(&json!({"path":"fixture.svg"})).unwrap()["doc_id"]
            .as_str()
            .unwrap()
            .to_string();
        (directory, registry, id, original)
    }

    #[test]
    fn noop_does_not_even_create_then_remove_a_proposed_record() {
        use std::os::fd::AsRawFd;
        let (_directory, registry, id, _original) = fixture();
        let entry = &registry.entries[&id];
        let operations = registry.workspace.roots[0]
            .join(entry.directory())
            .join("operations");
        let directory = std::fs::File::open(&operations).unwrap();
        let times = [libc::timespec {
            tv_sec: 1,
            tv_nsec: 0,
        }; 2];
        // SAFETY: an owned synthetic directory FD and two valid timespecs.
        assert_eq!(
            unsafe { libc::futimens(directory.as_raw_fd(), times.as_ptr()) },
            0
        );
        let before = directory.metadata().unwrap().modified().unwrap();
        let result = style::apply(
            &registry,
            "set_fill",
            &json!({"doc_id":id,"object_ids":["r"],"color":"#112233"}),
        )
        .unwrap();
        assert_eq!(result["changed"], false);
        assert_eq!(directory.metadata().unwrap().modified().unwrap(), before);
        assert_eq!(std::fs::read_dir(&operations).unwrap().count(), 0);
    }

    #[test]
    fn noop_change_and_byte_exact_restore() {
        let (_directory, registry, id, original) = fixture();
        let entry = &registry.entries[&id];
        let noop = style::apply(
            &registry,
            "set_fill",
            &json!({"doc_id":id,"object_ids":["r"],"color":"#112233"}),
        )
        .unwrap();
        assert_eq!(noop["changed"], false);
        assert!(list_snapshots(&registry, entry).unwrap().is_empty());
        assert_eq!(
            std::fs::read_dir(
                registry.workspace.roots[0]
                    .join(entry.directory())
                    .join("operations")
            )
            .unwrap()
            .count(),
            0
        );
        let change = style::apply(
            &registry,
            "set_fill",
            &json!({"doc_id":id,"object_ids":["r"],"color":"blue"}),
        )
        .unwrap();
        assert_eq!(change["changed"], true);
        assert_eq!(list_snapshots(&registry, entry).unwrap().len(), 1);
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.directory().join("original.svg"), 52428800)
                .unwrap(),
            original
        );
        restore(&registry, &id, change["snapshot_id"].as_str().unwrap()).unwrap();
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap(),
            original
        );
        assert_eq!(list_snapshots(&registry, entry).unwrap().len(), 2);
    }

    #[test]
    fn batch_rollback_and_eager_validation() {
        let (_directory, registry, id, original) = fixture();
        let entry = &registry.entries[&id];
        let invalid = json!({"doc_id":id,"edits":[{"op":"set_fill","object_ids":["r"],"color":"red"},
            {"op":"set_fill","object_ids":["absent"],"color":"blue"}]});
        assert_eq!(
            batch::apply(&registry, &invalid).unwrap_err(),
            "object id not found in document"
        );
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap(),
            original
        );
        assert!(list_snapshots(&registry, entry).unwrap().is_empty());
        let records: Vec<_> = std::fs::read_dir(
            registry.workspace.roots[0]
                .join(entry.directory())
                .join("operations"),
        )
        .unwrap()
        .collect();
        assert_eq!(records.len(), 1);
        let record: Value =
            serde_json::from_slice(&std::fs::read(records[0].as_ref().unwrap().path()).unwrap())
                .unwrap();
        assert_eq!(record["status"], "discarded");
        let invalid = json!({"doc_id":id,"edits":[{"op":"set_fill","object_ids":["r"],"color":"red;display:none"}]});
        assert!(batch::apply(&registry, &invalid).is_err());
        assert_eq!(
            std::fs::read_dir(
                registry.workspace.roots[0]
                    .join(entry.directory())
                    .join("operations")
            )
            .unwrap()
            .count(),
            1
        );
        let result = batch::apply(
            &registry,
            &json!({"doc_id":id,"edits":[{"op":"set_fill","object_ids":["r"],"color":"blue"},
            {"op":"set_opacity","object_ids":["r"],"opacity":0.5}]}),
        )
        .unwrap();
        assert_eq!(result["edit_count"], 2);
        assert_eq!(list_snapshots(&registry, entry).unwrap().len(), 1);
    }

    #[test]
    fn repeat_and_fragment_batch_share_approval_rollback_and_one_snapshot() {
        let (_directory, registry, id, original) = fixture();
        let entry = &registry.entries[&id];
        let repeat = json!({"op":"repeat_objects","object_id":"r","group_id":"repeated",
            "placement":{"kind":"polyline","points":[{"x":0,"y":0},{"x":15,"y":0}],"count":2}});
        let replacement = json!({"op":"replace_svg_fragment","object_id":"r",
            "svg":"<rect xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\" fill=\"blue\"/>",
            "reference_policy":"allow_retained"});
        let mut args = json!({"doc_id":id,"edits":[repeat,replacement]});
        assert_eq!(
            batch::apply(&registry, &args).unwrap_err(),
            "high-risk operation requires explicit approval"
        );
        assert!(list_snapshots(&registry, entry).unwrap().is_empty());
        args["approval_token"] = json!("approved");
        args["edits"]
            .as_array_mut()
            .unwrap()
            .push(json!({"op":"set_fill","object_ids":["missing"],"color":"red"}));
        assert_eq!(
            batch::apply(&registry, &args).unwrap_err(),
            "object id not found in document"
        );
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap(),
            original
        );
        assert!(list_snapshots(&registry, entry).unwrap().is_empty());
        args["edits"].as_array_mut().unwrap().pop();
        let result = batch::apply(&registry, &args).unwrap();
        assert_eq!(result["risk_class"], "high");
        assert_eq!(result["edit_count"], 2);
        assert_eq!(list_snapshots(&registry, entry).unwrap().len(), 1);
        restore(&registry, &id, result["snapshot_id"].as_str().unwrap()).unwrap();
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap(),
            original
        );
    }

    #[test]
    fn approval_and_tampered_snapshot_fail_before_mutation() {
        let (_directory, registry, id, original) = fixture();
        let entry = &registry.entries[&id];
        assert_eq!(
            apply(
                &registry,
                &id,
                "delete_object",
                json!({}),
                "high",
                Some("  "),
                |_| Ok("ignored".into())
            )
            .unwrap_err(),
            "high-risk operation requires explicit approval"
        );
        assert_eq!(
            restore(&registry, &id, "../../fixture.svg").unwrap_err(),
            "snapshot not found"
        );
        let mut checkpoint = snapshot(&registry, entry, None, None).unwrap();
        checkpoint["file"] = json!("../../fixture.svg");
        registry
            .workspace
            .atomic_write(
                0,
                &entry.directory().join("snapshots/index.json"),
                &serde_json::to_vec(&json!({"snapshots":[checkpoint]})).unwrap(),
            )
            .unwrap();
        assert_eq!(
            restore(&registry, &id, checkpoint["snapshot_id"].as_str().unwrap()).unwrap_err(),
            "snapshot not found"
        );
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap(),
            original
        );
    }

    #[test]
    fn audit_failure_rolls_back_and_never_follows_a_replaced_directory() {
        let (_directory, registry, id, original) = fixture();
        let entry = &registry.entries[&id];
        let outside = tempfile::tempdir().unwrap();
        let operations = registry.workspace.roots[0]
            .join(entry.directory())
            .join("operations");
        let moved = operations.with_extension("saved");
        let result = apply(
            &registry,
            &id,
            "set_fill",
            json!({}),
            "medium",
            None,
            |document| {
                let mut node = crate::document::elements(document.get_root_element().unwrap())
                    .pop()
                    .unwrap();
                node.set_property("fill", "red").unwrap();
                std::fs::rename(&operations, &moved).unwrap();
                std::os::unix::fs::symlink(outside.path(), &operations).unwrap();
                Ok("simulated audit directory replacement".into())
            },
        );
        assert!(result.is_err());
        assert_eq!(
            registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap(),
            original
        );
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    #[test]
    fn final_audit_failure_after_edit_or_restore_recovers_from_durable_journal() {
        for restoring in [false, true] {
            let (_directory, registry, id, original) = fixture();
            let entry = &registry.entries[&id];
            let initial = if restoring {
                Some(
                    style::apply(
                        &registry,
                        "set_fill",
                        &json!({"doc_id":id,"object_ids":["r"],"color":"blue"}),
                    )
                    .unwrap(),
                )
            } else {
                None
            };
            let before = registry
                .workspace
                .read(0, &entry.working(), 52428800)
                .unwrap();
            let outside = tempfile::tempdir().unwrap();
            let operations = registry.workspace.roots[0]
                .join(entry.directory())
                .join("operations");
            let moved = operations.with_extension("saved");
            let hook_operations = operations.clone();
            let hook_moved = moved.clone();
            let hook_outside = outside.path().to_owned();
            AFTER_WORKING_TEST.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    std::fs::rename(&hook_operations, &hook_moved).unwrap();
                    std::os::unix::fs::symlink(&hook_outside, &hook_operations).unwrap();
                }))
            });
            let result = if let Some(initial) = initial {
                restore(&registry, &id, initial["snapshot_id"].as_str().unwrap())
            } else {
                style::apply(
                    &registry,
                    "set_fill",
                    &json!({"doc_id":id,"object_ids":["r"],"color":"red"}),
                )
            };
            assert!(result.unwrap_err().contains("rolled back"));
            assert_eq!(
                registry
                    .workspace
                    .read(0, &entry.working(), 52428800)
                    .unwrap(),
                before
            );
            assert_eq!(
                registry
                    .workspace
                    .read(0, &entry.directory().join("original.svg"), 52428800)
                    .unwrap(),
                original
            );
            assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
            assert!(crate::recovery::recover_document(&registry, entry).is_err());
            assert_eq!(
                registry
                    .workspace
                    .directory_entries(0, &entry.directory().join("recovery"))
                    .unwrap()
                    .len(),
                1
            );
            std::fs::remove_file(&operations).unwrap();
            std::fs::rename(moved, operations).unwrap();
            assert_eq!(
                crate::recovery::recover_document(&registry, entry).unwrap(),
                1
            );
            assert_eq!(
                registry
                    .workspace
                    .read(0, &entry.working(), 52428800)
                    .unwrap(),
                before
            );
            assert!(
                registry
                    .workspace
                    .directory_entries(0, &entry.directory().join("recovery"))
                    .unwrap()
                    .is_empty()
            );
        }
    }
}
