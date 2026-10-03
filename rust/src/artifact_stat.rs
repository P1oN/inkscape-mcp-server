//! Read-only, descriptor-anchored streaming artifact hashes.
use crate::{document::Registry, workspace::Workspace};
use serde_json::{Value, json};
fn one(workspace: &Workspace, path: &str) -> Result<Value, String> {
    if path.contains('\0') {
        return Err("path rejected: invalid characters".into());
    }
    if workspace.roots.is_empty() {
        return Err("path rejected: no workspace root configured".into());
    }
    let (root, relative) = workspace.resolve_input(path, None).map_err(|e| {
        if e.starts_with("path rejected: outside workspace") {
            "path rejected: outside workspace".to_owned()
        } else if e == "path rejected: could not resolve path" {
            "path rejected: file not found".to_owned()
        } else {
            e
        }
    })?;
    let (size, hash) = workspace.hash_file(root, &relative)?;
    Ok(json!({"path":relative.to_string_lossy(),"bytes":size,"sha256":hash}))
}
pub fn apply(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    let result = if tool == "stat_artifact" {
        one(
            &registry.workspace,
            args["path"].as_str().ok_or("path must be a string")?,
        )?
    } else {
        let paths = args["paths"]
            .as_array()
            .ok_or("paths must be a list of strings")?;
        if paths.is_empty() {
            return Err("stat_artifacts requires at least one path".into());
        }
        if paths.len() > 1024 {
            return Err("stat_artifacts exceeds 1024 paths".into());
        }
        let mut values = Vec::with_capacity(paths.len());
        let mut total = 0u64;
        for path in paths {
            let value = one(
                &registry.workspace,
                path.as_str().ok_or("paths must be a list of strings")?,
            )?;
            total = total
                .checked_add(value["bytes"].as_u64().unwrap())
                .ok_or("artifact total exceeds the supported size")?;
            values.push(value);
        }
        json!({"artifacts":values,"total_bytes":total,"count":paths.len()})
    };
    if result.to_string().len() > registry.workspace.max_output {
        return Err("artifact metadata exceeds the configured output size limit".into());
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn streaming_digest_caps_and_link_guards() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace {
            roots: vec![dir.path().canonicalize().unwrap()],
            max_input: 3,
            max_output: 1024,
        };
        std::fs::write(dir.path().join("file"), b"abc").unwrap();
        assert_eq!(
            ws.hash_file(0, Path::new("file")).unwrap(),
            (
                3,
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
            )
        );
        std::os::unix::fs::symlink("file", dir.path().join("link")).unwrap();
        assert!(ws.hash_file(0, Path::new("link")).is_err());
        assert_eq!(one(&ws, "link").unwrap()["path"], "file");
        std::fs::write(dir.path().join("file"), b"abcd").unwrap();
        assert_eq!(
            one(&ws, "file").unwrap_err(),
            "artifact exceeds the configured size limit"
        );
        assert_eq!(
            one(&ws, "/outside/missing").unwrap_err(),
            "path rejected: outside workspace"
        );
    }
}
