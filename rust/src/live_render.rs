//! Read-only live frame persistence and the existing session-scoped revision cache.
use crate::{
    live_cache::{Cache, Key},
    live_socket::Error,
    live_transport::Transport,
    live_view::Frame,
    workspace::Workspace,
};
use serde_json::{Value, json};
use std::{path::Path, sync::LazyLock, time::Instant};
static START: LazyLock<Instant> = LazyLock::new(Instant::now);
fn rounded(value: f64, digits: usize) -> String {
    crate::live_models::float_repr(format!("{value:.digits$}").parse().unwrap())
}
fn key(transport: &mut dyn Transport, frame: Frame) -> Result<Option<Key>, Error> {
    let revision = match transport.state_token() {
        Ok((token, _)) => token["revision"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned),
        Err(error @ (Error::Protocol(_) | Error::Rejected)) => return Err(error),
        Err(_) => None,
    };
    Ok(revision.map(|revision| Key {
        revision,
        viewport: frame
            .region
            .map(|r| {
                r.into_iter()
                    .map(|n| rounded(n, 4))
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .unwrap_or("full".into()),
        scale: frame
            .scale
            .map(|n| rounded(n, 6))
            .unwrap_or("native".into()),
    }))
}
pub fn render(
    transport: &mut dyn Transport,
    mut cache: Option<&mut Cache>,
    workspace: &Workspace,
    frame: Frame,
) -> Result<Value, Error> {
    let key = if cache.is_some() {
        key(transport, frame)?
    } else {
        None
    };
    if let (Some(cache), Some(key)) = (cache.as_mut(), key.as_ref()) {
        let now = START.elapsed().as_secs_f64();
        if let Some(value) = cache.within_budget(key, now).or_else(|| cache.get(key)) {
            return Ok(value);
        }
    }
    let png = transport.render_view(frame.region, frame.scale)?;
    if workspace.roots.is_empty() {
        return Err(Error::NotAvailable);
    }
    if png.len() > workspace.max_output {
        return Err(Error::Context("live render exceeds output size limit"));
    }
    for directory in [
        ".inkscape-mcp/live/artifacts",
        ".inkscape-mcp/live/operations",
    ] {
        workspace
            .ensure_directory(0, Path::new(directory))
            .map_err(|_| Error::Context("could not store live render"))?;
    }
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S_%6fZ").to_string();
    let mut chosen = None;
    for _ in 0..4 {
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let name = format!(
            ".inkscape-mcp/live/artifacts/live-view-{stamp}-{}.png",
            &nonce[..8]
        );
        if workspace
            .file_kind(0, Path::new(&name))
            .map_err(|_| Error::Context("could not store live render"))?
            .is_none()
        {
            chosen = Some(name);
            break;
        }
    }
    let path = chosen.ok_or(Error::Context("could not store live render"))?;
    workspace
        .atomic_write(0, Path::new(&path), &png)
        .map_err(|_| Error::Context("could not store live render"))?;
    let value = json!({"artifact_path":path,"format":"png","size_bytes":png.len(),"region":frame.region.is_some(),"scale":frame.scale});
    if let (Some(cache), Some(key)) = (cache, key) {
        cache.put(key, value.clone(), png.len(), START.elapsed().as_secs_f64());
    }
    Ok(value)
}
/// Read-only selection feedback uses the source's second-resolution artifact name.
/// Repeated exports in the same second atomically replace that artifact, as in Python.
pub fn export_selection(
    transport: &mut dyn Transport,
    workspace: &Workspace,
) -> Result<Value, Error> {
    if !transport.supports(crate::live_protocol::Command::ExportSelection) {
        return Err(Error::Unsupported("selection export unavailable"));
    }
    let selection = match transport.selection() {
        Ok(selection) => crate::live_models::ids(&selection["object_ids"]),
        Err(error @ (Error::Protocol(_) | Error::Rejected))
            if transport.name() == "extension-socket" =>
        {
            return Err(error);
        }
        Err(_) => Vec::new(),
    };
    let png = transport.export_selection()?;
    if workspace.roots.is_empty() {
        return Err(Error::Protocol("no workspace for selection export"));
    }
    if png.len() > workspace.max_output {
        return Err(Error::Context(
            "live selection export exceeds output size limit",
        ));
    }
    for directory in [
        ".inkscape-mcp/live/artifacts",
        ".inkscape-mcp/live/operations",
    ] {
        workspace
            .ensure_directory(0, Path::new(directory))
            .map_err(|_| Error::Context("could not store live selection export"))?;
    }
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let path = format!(".inkscape-mcp/live/artifacts/live-selection-{stamp}.png");
    workspace
        .atomic_write(0, Path::new(&path), &png)
        .map_err(|_| Error::Context("could not store live selection export"))?;
    Ok(json!({"artifact_path":path,"format":"png","size_bytes":png.len(),"object_ids":selection}))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_protocol::Command;
    struct Fake {
        bytes: Vec<u8>,
    }
    impl Transport for Fake {
        fn name(&self) -> &str {
            "test"
        }
        fn supports(&self, _: Command) -> bool {
            true
        }
        fn connect(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn disconnect(&mut self) {}
        fn is_connected(&self) -> bool {
            true
        }
        fn active_document(&mut self) -> Result<Value, Error> {
            Ok(json!({}))
        }
        fn render_view(&mut self, _: Option<[f64; 4]>, _: Option<f64>) -> Result<Vec<u8>, Error> {
            Ok(self.bytes.clone())
        }
        fn export_selection(&mut self) -> Result<Vec<u8>, Error> {
            Ok(self.bytes.clone())
        }
    }
    #[test]
    fn selection_write_refuses_caps_and_links_before_publication() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let mut fake = Fake {
            bytes: vec![b'x'; 4097],
        };
        assert!(export_selection(&mut fake, &workspace).is_err());
        assert!(!root.path().join(".inkscape-mcp").exists());
        fake.bytes = b"bounded synthetic PNG".to_vec();
        workspace
            .ensure_directory(0, Path::new(".inkscape-mcp/live"))
            .unwrap();
        std::os::unix::fs::symlink(
            external.path(),
            root.path().join(".inkscape-mcp/live/artifacts"),
        )
        .unwrap();
        assert!(export_selection(&mut fake, &workspace).is_err());
        assert_eq!(std::fs::read_dir(external.path()).unwrap().count(), 0);
        std::fs::remove_file(root.path().join(".inkscape-mcp/live/artifacts")).unwrap();
        let value = export_selection(&mut fake, &workspace).unwrap();
        let path = Path::new(value["artifact_path"].as_str().unwrap());
        assert_eq!(workspace.read(0, path, 4096).unwrap(), fake.bytes);
        fake.bytes = vec![b'x'; 4097];
        assert!(export_selection(&mut fake, &workspace).is_err());
        assert_eq!(
            workspace.read(0, path, 4096).unwrap(),
            b"bounded synthetic PNG"
        );
        assert_eq!(
            std::fs::read_dir(root.path().join(".inkscape-mcp/live/operations"))
                .unwrap()
                .count(),
            0
        );
    }
    #[test]
    fn frame_write_refuses_caps_and_symlinks_and_leaves_no_partial_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let frame = Frame {
            region: None,
            scale: None,
        };
        let mut fake = Fake {
            bytes: vec![b'x'; 4097],
        };
        assert!(render(&mut fake, None, &workspace, frame).is_err());
        assert!(!root.path().join(".inkscape-mcp").exists());
        fake.bytes = b"synthetic bounded PNG".to_vec();
        workspace
            .ensure_directory(0, Path::new(".inkscape-mcp/live"))
            .unwrap();
        std::os::unix::fs::symlink(
            external.path(),
            root.path().join(".inkscape-mcp/live/artifacts"),
        )
        .unwrap();
        assert!(render(&mut fake, None, &workspace, frame).is_err());
        assert_eq!(std::fs::read_dir(external.path()).unwrap().count(), 0);
        std::fs::remove_file(root.path().join(".inkscape-mcp/live/artifacts")).unwrap();
        let value = render(&mut fake, None, &workspace, frame).unwrap();
        let bytes = workspace
            .read(0, Path::new(value["artifact_path"].as_str().unwrap()), 4096)
            .unwrap();
        assert_eq!(bytes, fake.bytes);
        assert_eq!(
            std::fs::read_dir(root.path().join(".inkscape-mcp/live/artifacts"))
                .unwrap()
                .count(),
            1
        );
    }
}
