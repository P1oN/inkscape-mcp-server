//! Save working bytes to a new, root-qualified artifact; protect all managed files.
use crate::{document::Registry, transaction, validate};
use serde_json::{Value, json};

pub fn document(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let overwrite = crate::arguments::boolean(args, "overwrite", false)?;
    let approval = crate::arguments::string(args, "approval_token")?;
    let pre = validate::document(registry, id)?;
    let entry = &registry.entries[id];
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let vector_only = crate::arguments::boolean(args, "vector_only", false)?;
    if vector_only {
        crate::vector_content::require(&bytes, registry.workspace.max_input)?;
    }
    let raw = args["dest_path"]
        .as_str()
        .ok_or("dest_path must be a string")?;
    let (root, relative) = registry
        .workspace
        .resolve_output(raw, crate::arguments::string(args, "root_id")?)?;
    let destination = registry.workspace.roots[root].join(&relative);
    let resolved = destination
        .canonicalize()
        .unwrap_or_else(|_| destination.clone());
    for managed in registry.entries.values() {
        for path in [
            managed.directory().join("original.svg"),
            managed.working(),
            managed.source.clone().into(),
        ] {
            let path = registry.workspace.roots[managed.root].join(path);
            if path.canonicalize().unwrap_or(path) == resolved {
                return Err("cannot overwrite a managed document file".into());
            }
        }
    }
    let kind = registry.workspace.file_kind(root, &relative)?;
    if kind == Some(libc::S_IFLNK) {
        return Err("cannot overwrite a symbolic link".into());
    }
    let overwritten = kind.is_some();
    if overwritten && (!overwrite || approval.is_none_or(str::is_empty)) {
        return Err(
            "destination already exists; overwriting requires overwrite=True and an approval_token"
                .into(),
        );
    }
    let risk = if overwritten { "high" } else { "medium" };
    transaction::policy(risk, approval)?;
    let mut record = transaction::record(
        id,
        "save_document_as",
        risk,
        json!({"dest_path":raw,"overwrite":overwrite,"vector_only":vector_only}),
        true,
    );
    transaction::persist(registry, entry, &record)?;
    let old = if overwritten {
        Some(
            registry
                .workspace
                .read(root, &relative, registry.workspace.max_output)
                .map_err(|_| "saved file could not be written")?,
        )
    } else {
        None
    };
    let write = if overwritten {
        registry.workspace.atomic_write(root, &relative, &bytes)
    } else {
        registry.workspace.write_new(root, &relative, &bytes)
    };
    if write.is_err() {
        // Atomic replacement can fail after rename (directory fsync). Restore
        // the approved destination bytes before reporting an unsuccessful save.
        let rollback = old
            .as_ref()
            .map(|old| registry.workspace.atomic_write(root, &relative, old));
        record["status"] = json!("discarded");
        record["updated_at"] = json!(transaction::now());
        let _ = transaction::persist(registry, entry, &record);
        if rollback.is_some_and(|r| r.is_err()) {
            return Err(
                "save persistence failed; destination recovery is required before retrying".into(),
            );
        }
        return Err("saved file could not be written".into());
    }
    let finish = (|| {
        if registry
            .workspace
            .read(root, &relative, registry.workspace.max_input)?
            != bytes
        {
            return Err("saved file could not be verified after write".into());
        }
        let post = validate::document(registry, id)?;
        record["status"] = json!("applied");
        record["artifacts"] = json!([relative.to_string_lossy()]);
        record["updated_at"] = json!(transaction::now());
        transaction::persist(registry, entry, &record)?;
        Ok(
            json!({"doc_id":id,"artifact":registry.workspace.artifact_link(root,&relative),"saved_path":relative.to_string_lossy(),"operation_id":record["operation_id"],"overwritten":overwritten,"pre_validation":pre,"post_validation":post}),
        )
    })();
    if finish.is_err() {
        let rollback = if let Some(old) = old {
            registry.workspace.atomic_write(root, &relative, &old)
        } else {
            registry.workspace.remove_file(root, &relative)
        };
        record["status"] = json!("discarded");
        record["updated_at"] = json!(transaction::now());
        let _ = transaction::persist(registry, entry, &record);
        if rollback.is_err() {
            return Err(
                "save persistence failed; destination recovery is required before retrying".into(),
            );
        }
    }
    finish
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{style, workspace::Workspace};
    use std::path::Path;

    #[test]
    fn vector_guard_refuses_before_destination_and_record_writes() {
        for body in [
            "<image href='data:image/png;base64,YQ=='/>",
            "<image href='linked.png'/>",
            "<defs><image id='r' href='data:image/png;base64,YQ=='/></defs><use href='#r'/>",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let source =
                format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'>{body}</svg>");
            std::fs::write(temp.path().join("source.svg"), &source).unwrap();
            let mut registry = Registry {
                workspace: Workspace {
                    roots: vec![temp.path().canonicalize().unwrap()],
                    max_input: 100000,
                    max_output: 100000,
                },
                entries: indexmap::IndexMap::new(),
            };
            let id = registry.open(&json!({"path":"source.svg"})).unwrap()["doc_id"]
                .as_str()
                .unwrap()
                .to_owned();
            std::fs::write(temp.path().join("existing.svg"), b"preserve").unwrap();
            for args in [
                json!({"doc_id":id,"dest_path":"new/final.svg","vector_only":true}),
                json!({"doc_id":id,"dest_path":"existing.svg","vector_only":true,"overwrite":true,"approval_token":"approved"}),
            ] {
                assert!(document(&registry, &args).is_err());
            }
            assert!(!temp.path().join("new").exists());
            assert_eq!(
                std::fs::read(temp.path().join("existing.svg")).unwrap(),
                b"preserve"
            );
            let entry = &registry.entries[&id];
            assert_eq!(
                std::fs::read(temp.path().join(entry.working())).unwrap(),
                source.as_bytes()
            );
            assert_eq!(
                std::fs::read_dir(temp.path().join(entry.directory()).join("operations"))
                    .unwrap()
                    .count(),
                0
            );
            document(&registry, &json!({"doc_id":id,"dest_path":"allowed.svg"})).unwrap();
        }
    }
    #[test]
    fn save_preserves_original_and_refuses_links_and_escapes() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("workspace");
        std::fs::create_dir(&root).unwrap();
        let outside = temporary.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        let workspace = Workspace {
            roots: vec![root.canonicalize().unwrap()],
            max_input: 100000,
            max_output: 100000,
        };
        let source = br#"<svg viewBox="0 0 20 20"><rect id="r" fill="red"/></svg>"#;
        workspace
            .write_new(0, Path::new("source.svg"), source)
            .unwrap();
        let mut registry = Registry {
            workspace,
            entries: indexmap::IndexMap::new(),
        };
        let id = registry.open(&json!({"path":"source.svg"})).unwrap()["doc_id"]
            .as_str()
            .unwrap()
            .to_owned();
        style::apply(
            &registry,
            "set_fill",
            &json!({"doc_id":id,"object_ids":["r"],"color":"blue"}),
        )
        .unwrap();
        let save = document(&registry, &json!({"doc_id":id,"dest_path":"out/final.svg"})).unwrap();
        assert_eq!(save["overwritten"], false);
        assert_eq!(save["pre_validation"]["ok"], true);
        let saved = std::fs::read(root.join("out/final.svg")).unwrap();
        assert_eq!(
            saved,
            registry
                .workspace
                .read(0, &registry.entries[&id].working(), 100000)
                .unwrap()
        );
        for dest in ["source.svg", "../outside/new/final.svg"] {
            assert!(
                document(
                    &registry,
                    &json!({"doc_id":id,"dest_path":dest,"overwrite":true,"approval_token":"yes"})
                )
                .is_err()
            );
        }
        std::os::unix::fs::symlink(
            outside.join("never-created.svg"),
            root.join("outside-link.svg"),
        )
        .unwrap();
        assert!(document(&registry,&json!({"doc_id":id,"dest_path":"outside-link.svg","overwrite":true,"approval_token":"yes"})).is_err());
        assert!(!outside.join("never-created.svg").exists());
        assert!(!outside.join("new").exists());
        assert!(
            document(
                &registry,
                &json!({"doc_id":id,"dest_path":"invalid/new.svg","overwrite":"wrong"})
            )
            .is_err()
        );
        assert!(!root.join("invalid").exists());
        // Atomic destination replacement preserves a managed source even if a
        // previously unrelated destination is a hard link to its inode.
        std::fs::hard_link(root.join("source.svg"), root.join("hard-link.svg")).unwrap();
        document(&registry,&json!({"doc_id":id,"dest_path":"hard-link.svg","overwrite":true,"approval_token":"yes"})).unwrap();
        assert_eq!(std::fs::read(root.join("source.svg")).unwrap(), source);
        assert_eq!(std::fs::read(root.join("hard-link.svg")).unwrap(), saved);
        assert_eq!(
            registry
                .workspace
                .read(
                    0,
                    &registry.entries[&id].directory().join("original.svg"),
                    100000
                )
                .unwrap(),
            source
        );
        let (_, path) = registry
            .workspace
            .resolve_output("raced/out.svg", None)
            .unwrap();
        std::fs::rename(root.join("raced"), root.join("raced-old")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("raced")).unwrap();
        assert!(
            registry
                .workspace
                .write_new(0, &path, b"must not escape")
                .is_err()
        );
        assert!(!outside.join("out.svg").exists());
    }
}
