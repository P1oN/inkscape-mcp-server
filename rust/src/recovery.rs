//! Private write-ahead recovery for a working SVG plus its Operation Record.
//! Only typed, hash-bound managed paths are admitted; unrelated heads are preserved.
use crate::{
    document::{Entry, Registry},
    transaction,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u8,
    doc_id: String,
    operation_id: String,
    snapshot_id: String,
    before_sha256: String,
    after_sha256: String,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|suffix| {
        suffix.len() == 8
            && suffix
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    })
}
fn path(entry: &Entry, operation: &str) -> Result<PathBuf, String> {
    if !valid_id(operation, "op_") {
        return Err("invalid recovery operation id".into());
    }
    Ok(entry
        .directory()
        .join("recovery")
        .join(format!("{operation}.json")))
}
pub fn prepare(
    registry: &Registry,
    entry: &Entry,
    record: &Value,
    before: &[u8],
    after: &[u8],
) -> Result<(), String> {
    let operation = record["operation_id"]
        .as_str()
        .ok_or("invalid recovery record")?;
    let snapshot = record["snapshot_id"]
        .as_str()
        .ok_or("recovery snapshot missing")?;
    let source = transaction::snapshot_source(registry, entry, snapshot)?;
    let bytes = registry
        .workspace
        .read(entry.root, &source, registry.workspace.max_input)?;
    if bytes != before {
        return Err("recovery snapshot does not match working bytes".into());
    }
    let journal = Journal {
        version: 1,
        doc_id: entry.id.clone(),
        operation_id: operation.to_owned(),
        snapshot_id: snapshot.to_owned(),
        before_sha256: hash(before),
        after_sha256: hash(after),
    };
    registry.workspace.write_new(
        entry.root,
        &path(entry, operation)?,
        &serde_json::to_vec_pretty(&journal)
            .map_err(|_| "recovery journal serialization failed")?,
    )
}
pub fn finish(registry: &Registry, entry: &Entry, operation: &str) -> Result<(), String> {
    registry
        .workspace
        .regular_info(entry.root, &path(entry, operation)?, true)
        .map(|_| ())
}
fn recover_one(registry: &Registry, entry: &Entry, operation: &str) -> Result<(), String> {
    let bytes = registry.workspace.read(
        entry.root,
        &path(entry, operation)?,
        registry.workspace.max_output.min(4096),
    )?;
    let journal: Journal =
        serde_json::from_slice(&bytes).map_err(|_| "invalid recovery journal")?;
    let valid_hash = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    };
    if journal.version != 1
        || journal.doc_id != entry.id
        || journal.operation_id != operation
        || !valid_id(&journal.snapshot_id, "snap_")
        || !valid_hash(&journal.before_sha256)
        || !valid_hash(&journal.after_sha256)
    {
        return Err("invalid recovery journal identity".into());
    }
    let record_path = entry
        .directory()
        .join("operations")
        .join(format!("{operation}.json"));
    let bytes = registry.workspace.read(
        entry.root,
        &record_path,
        registry.workspace.max_output.min(4 * 1024 * 1024),
    )?;
    let mut record: Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid recovery record")?;
    if record["operation_id"] != operation
        || record["doc_id"] != entry.id
        || record["snapshot_id"] != journal.snapshot_id
        || !record["params"].is_object()
        || !record["tool"].is_string()
        || !record["risk_class"]
            .as_str()
            .is_some_and(|risk| ["low", "medium", "high"].contains(&risk))
    {
        return Err("recovery audit identity mismatch".into());
    }
    // A durable applied record commits the edit. Later legitimate edits may have
    // changed the head; never roll them back because cleanup of this journal failed.
    if record["status"] == "applied" {
        return finish(registry, entry, operation);
    }
    if record["status"] != "proposed" && record["status"] != "discarded" {
        return Err("recovery audit status is ambiguous".into());
    }
    let source = transaction::snapshot_source(registry, entry, &journal.snapshot_id)?;
    let before = registry
        .workspace
        .read(entry.root, &source, registry.workspace.max_input)?;
    if hash(&before) != journal.before_sha256 {
        return Err("recovery snapshot hash mismatch".into());
    }
    let current =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let current_hash = hash(&current);
    if current_hash == journal.after_sha256 {
        registry
            .workspace
            .atomic_write(entry.root, &entry.working(), &before)?;
    } else if current_hash != journal.before_sha256 {
        return Err("recovery head changed; preserve document for manual inspection".into());
    }
    record["status"] = json!("discarded");
    record["updated_at"] = json!(transaction::now());
    transaction::persist(registry, entry, &record)?;
    finish(registry, entry, operation)
}
/// Fail closed for an ambiguous/unsafe journal: callers must skip retention for
/// this document so recovery bytes and explaining records cannot be pruned.
pub fn recover_document(registry: &Registry, entry: &Entry) -> Result<usize, String> {
    let directory = entry.directory().join("recovery");
    match registry.workspace.file_kind(entry.root, &directory)? {
        None => return Ok(0),
        Some(kind) if kind == libc::S_IFDIR => (),
        _ => return Err("recovery directory is unsafe".into()),
    }
    let mut entries = registry
        .workspace
        .directory_entries(entry.root, &directory)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    if entries
        .iter()
        .filter(|(name, _)| name.ends_with(".json"))
        .count()
        > 1
    {
        // Normal serialized transactions recover before starting another edit.
        // Multiple pending journals indicate concurrent/external state: do not
        // guess an order from random operation IDs and roll back the wrong head.
        return Err("multiple recovery journals; preserve document for manual inspection".into());
    }
    let mut recovered = 0;
    for (name, kind) in entries {
        if !name.ends_with(".json") {
            continue;
        }
        if kind != libc::S_IFREG {
            return Err("recovery journal is unsafe".into());
        }
        let operation = name
            .strip_suffix(".json")
            .ok_or("invalid recovery filename")?;
        recover_one(registry, entry, operation)?;
        recovered += 1;
    }
    Ok(recovered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{retention, workspace::Workspace};
    const BEFORE: &[u8] =
        br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r" fill="blue"/></svg>"#;
    const AFTER: &[u8] =
        br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r" fill="red"/></svg>"#;
    const LATER: &[u8] =
        br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r" fill="green"/></svg>"#;
    fn registry(root: &std::path::Path) -> Registry {
        Registry {
            workspace: Workspace {
                roots: vec![root.canonicalize().unwrap()],
                max_input: 52428800,
                max_output: 104857600,
            },
            entries: indexmap::IndexMap::new(),
        }
    }
    #[test]
    #[ignore = "invoked in an exact owned child by crash_boundaries_recover_or_preserve"]
    fn child_crash_boundary() {
        let Some(root) = std::env::var_os("IMCP_RECOVERY_TEST_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let stage = std::env::var("IMCP_RECOVERY_TEST_STAGE").unwrap();
        let mut registry = registry(&root);
        registry
            .workspace
            .write_new(0, std::path::Path::new("source.svg"), BEFORE)
            .unwrap();
        let id = registry.open(&json!({"path":"source.svg"})).unwrap()["doc_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let entry = &registry.entries[&id];
        let mut record = transaction::record(
            &id,
            "set_fill",
            "medium",
            json!({"object_ids":["r"], "color":"red"}),
            true,
        );
        let operation = record["operation_id"].as_str().unwrap().to_owned();
        transaction::persist(&registry, entry, &record).unwrap();
        let snapshot =
            transaction::snapshot(&registry, entry, Some("pre-crash"), Some(&operation)).unwrap();
        record["snapshot_id"] = snapshot["snapshot_id"].clone();
        transaction::persist(&registry, entry, &record).unwrap();
        prepare(&registry, entry, &record, BEFORE, AFTER).unwrap();
        if stage != "prepared" {
            registry
                .workspace
                .atomic_write(0, &entry.working(), AFTER)
                .unwrap();
        }
        if stage == "applied" || stage == "applied-later" {
            record["status"] = json!("applied");
            transaction::persist(&registry, entry, &record).unwrap();
        }
        if stage == "discarded" {
            record["status"] = json!("discarded");
            transaction::persist(&registry, entry, &record).unwrap();
        }
        if stage == "unrelated" || stage == "applied-later" {
            registry
                .workspace
                .atomic_write(0, &entry.working(), LATER)
                .unwrap();
        }
        if stage == "snapshot-corrupt" {
            let source = transaction::snapshot_source(
                &registry,
                entry,
                record["snapshot_id"].as_str().unwrap(),
            )
            .unwrap();
            registry
                .workspace
                .atomic_write(0, &source, b"tampered snapshot")
                .unwrap();
        }
        if stage == "audit-corrupt" {
            registry
                .workspace
                .atomic_write(
                    0,
                    &entry
                        .directory()
                        .join("operations")
                        .join(format!("{operation}.json")),
                    b"{broken audit",
                )
                .unwrap();
        }
        if stage == "journal-corrupt" {
            registry
                .workspace
                .atomic_write(0, &path(entry, &operation).unwrap(), b"{broken journal")
                .unwrap();
        }
        if stage == "journal-link" {
            let journal = root.join(path(entry, &operation).unwrap());
            let bytes = std::fs::read(&journal).unwrap();
            std::fs::remove_file(&journal).unwrap();
            let outside = PathBuf::from(std::env::var_os("IMCP_RECOVERY_TEST_OUTSIDE").unwrap());
            std::fs::write(&outside, bytes).unwrap();
            std::os::unix::fs::symlink(outside, journal).unwrap();
        }
        // Exit bypasses Rust destructors after real fsync/publication boundaries.
        // This child is a test executable, never the user's MCP/GUI process.
        std::process::exit(17);
    }
    #[test]
    fn crash_boundaries_recover_or_preserve() {
        for stage in [
            "prepared",
            "head-written",
            "discarded",
            "applied",
            "applied-later",
            "unrelated",
            "snapshot-corrupt",
            "audit-corrupt",
            "journal-corrupt",
            "journal-link",
        ] {
            let root = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let sentinel = outside.path().join("sentinel");
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "recovery::tests::child_crash_boundary",
                    "--ignored",
                    "--nocapture",
                ])
                .env("IMCP_RECOVERY_TEST_ROOT", root.path())
                .env("IMCP_RECOVERY_TEST_STAGE", stage)
                .env("IMCP_RECOVERY_TEST_OUTSIDE", &sentinel)
                .output()
                .unwrap();
            assert_eq!(
                child.status.code(),
                Some(17),
                "{stage}: {}",
                String::from_utf8_lossy(&child.stderr)
            );
            let registry = registry(root.path());
            let outside_bytes = std::fs::read(&sentinel).ok();
            let directories = registry
                .workspace
                .directory_entries(0, std::path::Path::new(".inkscape-mcp/documents"))
                .unwrap();
            assert_eq!(directories.len(), 1);
            let entry = Entry {
                id: directories[0].0.clone(),
                root: 0,
                source: String::new(),
                opened_at: String::new(),
            };
            let result = retention::sweep(&registry);
            let ambiguous = [
                "unrelated",
                "snapshot-corrupt",
                "audit-corrupt",
                "journal-corrupt",
                "journal-link",
            ]
            .contains(&stage);
            assert_eq!(
                result["failures"],
                usize::from(ambiguous),
                "{stage}: {result}"
            );
            let expected = if stage == "applied" {
                AFTER
            } else if stage == "unrelated" || stage == "applied-later" {
                LATER
            } else if ambiguous {
                AFTER
            } else {
                BEFORE
            };
            assert_eq!(
                registry.workspace.read(0, &entry.working(), 1000).unwrap(),
                expected,
                "{stage}"
            );
            assert_eq!(
                registry
                    .workspace
                    .read(0, &entry.directory().join("original.svg"), 1000)
                    .unwrap(),
                BEFORE,
                "{stage}"
            );
            assert_eq!(
                registry
                    .workspace
                    .read(0, std::path::Path::new("source.svg"), 1000)
                    .unwrap(),
                BEFORE,
                "{stage}"
            );
            let journals = registry
                .workspace
                .directory_entries(0, &entry.directory().join("recovery"))
                .unwrap();
            assert_eq!(journals.len(), usize::from(ambiguous), "{stage}");
            if ambiguous {
                assert_eq!(
                    result["documents"],
                    json!([]),
                    "ambiguous recovery must block pruning: {stage}"
                );
                assert_eq!(
                    transaction::list_snapshots(&registry, &entry)
                        .unwrap()
                        .len(),
                    1,
                    "{stage}"
                );
            } else {
                let records = registry
                    .workspace
                    .directory_entries(0, &entry.directory().join("operations"))
                    .unwrap();
                let record: Value = serde_json::from_slice(
                    &registry
                        .workspace
                        .read(
                            0,
                            &entry.directory().join("operations").join(&records[0].0),
                            10000,
                        )
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(
                    record["status"],
                    if stage.starts_with("applied") {
                        "applied"
                    } else {
                        "discarded"
                    },
                    "{stage}"
                );
            }
            if stage == "journal-link" {
                assert_eq!(std::fs::read(&sentinel).ok(), outside_bytes);
            }
            let repeated = retention::sweep(&registry);
            assert_eq!(repeated["failures"], result["failures"], "{stage}");
            assert_eq!(
                registry.workspace.read(0, &entry.working(), 1000).unwrap(),
                expected,
                "{stage}"
            );
        }
    }
}
