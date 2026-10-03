//! Live snapshot capture as a new headless document with shared snapshot/audit machinery.
use crate::{document::Registry, live::Live, transaction};
use serde_json::{Value, json};
pub fn sync(live: &mut Live, registry: &mut Registry, args: &Value) -> Result<Value, String> {
    let destination = args["dest_path"]
        .as_str()
        .ok_or("dest_path must be a string")?;
    let (root, relative) = registry
        .workspace
        .resolve_output(destination, None)
        .map_err(|message| {
            if message.starts_with("path rejected: outside workspace;") {
                "path rejected: outside workspace".to_owned()
            } else {
                message
            }
        })?;
    if registry.workspace.file_kind(root, &relative)?.is_some() {
        return Err("destination already exists; live sync never overwrites".into());
    }
    let transport = live
        .session
        .require_transport()
        .map_err(|e| e.public_message().to_string())?;
    let name = transport.name().to_owned();
    let svg = transport.document_svg().map_err(|error| {
        // Source ProtocolError is not a LiveError and escapes this wrapper.
        crate::live_events::protocol_message(&error)
            .map(|message| format!("Error calling tool 'live_sync_to_workspace': {message}"))
            .unwrap_or_else(|| "could not read the live document".into())
    })?;
    if svg.len() > registry.workspace.max_input {
        return Err("live document exceeds the configured size limit".into());
    }
    registry
        .workspace
        .atomic_create(root, &relative, svg.as_bytes())
        .map_err(|e| -> String {
            if e == "destination already exists" {
                "destination already exists; live sync never overwrites".into()
            } else {
                "failed to write the synced document".into()
            }
        })?;
    let saved = relative.to_string_lossy().into_owned();
    let id = registry
        .seed_entry(root, saved.clone(), svg.as_bytes().to_vec())
        .map_err(|_| "synced document could not be registered")?;
    let entry = &registry.entries[&id];
    transaction::policy("medium", None)?;
    let mut record = transaction::record(
        &id,
        "live_sync_to_workspace",
        "medium",
        json!({"dest_path":destination,"transport":name}),
        true,
    );
    transaction::persist(registry, entry, &record)?;
    let snapshot = transaction::snapshot(
        registry,
        entry,
        Some("live sync"),
        record["operation_id"].as_str(),
    )?;
    record["snapshot_id"] = snapshot["snapshot_id"].clone();
    record["artifacts"] = json!([saved]);
    record["status"] = json!("applied");
    record["updated_at"] = json!(transaction::now());
    transaction::persist(registry, entry, &record)?;
    Ok(
        json!({"doc_id":id,"saved_path":saved,"operation_id":record["operation_id"],"snapshot_id":snapshot["snapshot_id"],"size_bytes":svg.len()}),
    )
}
