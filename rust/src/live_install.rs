//! Installation of the two fixed shipped helper assets, never arbitrary extension input.
use crate::{live_socket::Error, workspace::Workspace};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
const ASSETS: [(&str, &[u8]); 1] = [(
    "inkscape_mcp_live.inx",
    include_bytes!("../../runtime/helper_extension/inkscape_mcp_live.inx"),
)];
fn wrapper() -> Result<Vec<u8>, String> {
    let binary = std::env::current_exe()
        .map_err(|_| "helper unavailable")?
        .with_file_name("inkscape-mcp-live");
    wrapper_for(&binary)
}
fn wrapper_for(binary: &Path) -> Result<Vec<u8>, String> {
    if !binary.is_absolute() || !std::fs::symlink_metadata(binary).is_ok_and(|m| m.is_file()) {
        return Err("native socket helper missing".into());
    }
    let quoted = binary
        .to_str()
        .ok_or("invalid helper path")?
        .replace('\'', "'\"'\"'");
    Ok(format!("#!/bin/sh\nexec '{quoted}' \"$@\"\n").into_bytes())
}
pub fn gate(enabled: bool) -> Result<(), String> {
    if enabled {
        Ok(())
    } else {
        Err(Error::Disabled.public_message().into())
    }
}
pub fn install(capabilities: &Value) -> Result<Value, String> {
    let user=capabilities["user_data_dir"].as_str().filter(|s|!s.is_empty()).ok_or("could not determine the Inkscape extensions directory; call list_capabilities to see what this runtime supports")?;
    let path = Path::new(user).join("extensions");
    install_at(&path, &wrapper()?).map_err(|_| "could not install the live helper extension")?;
    let display = std::env::var_os("HOME")
        .and_then(|home| {
            path.strip_prefix(home)
                .ok()
                .map(|p| Path::new("~").join(p).to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "extensions".into());
    Ok(
        json!({"installed_files":["inkscape_mcp_live.inx","inkscape_mcp_live_run.sh"],"extensions_dir":display}),
    )
}
fn install_at(path: &Path, wrapper: &[u8]) -> Result<(), String> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|_| "installation unavailable")?
            .join(path)
    };
    let relative = path
        .strip_prefix("/")
        .map_err(|_| "installation path unavailable")?;
    let workspace = Workspace {
        roots: vec![PathBuf::from("/")],
        max_input: 1024 * 1024,
        max_output: 1024 * 1024,
    };
    workspace.ensure_directory(0, relative)?;
    // Refuse links/nonregular destinations before either fixed asset is replaced.
    for name in ["inkscape_mcp_live.inx", "inkscape_mcp_live_run.sh"] {
        if workspace
            .file_kind(0, &relative.join(name))?
            .is_some_and(|kind| kind != libc::S_IFREG)
        {
            return Err("installation destination unavailable".into());
        }
    }
    for (name, bytes) in ASSETS {
        workspace.atomic_write(0, &relative.join(name), bytes)?;
    }
    workspace.atomic_write_executable(0, &relative.join("inkscape_mcp_live_run.sh"), wrapper)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_install_refuses_links_and_atomic_upgrade_preserves_hardlinked_original() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        // macOS TMPDIR may traverse /var -> /private/var; keep that alias out
        // of the fixture so production no-follow descent remains exercised.
        let root = temp.path().canonicalize().unwrap();
        let dir = root.join("data/extensions");
        install_at(&dir, b"#!/bin/sh\n").unwrap();
        for (name, bytes) in ASSETS {
            assert_eq!(std::fs::read(dir.join(name)).unwrap(), bytes);
        }
        install_at(&dir, b"#!/bin/sh\n").unwrap();
        std::fs::write(outside.path().join("original"), b"original").unwrap();
        std::fs::remove_file(dir.join(ASSETS[0].0)).unwrap();
        std::fs::hard_link(outside.path().join("original"), dir.join(ASSETS[0].0)).unwrap();
        install_at(&dir, b"#!/bin/sh\n").unwrap();
        assert_eq!(
            std::fs::read(outside.path().join("original")).unwrap(),
            b"original"
        );
        use std::os::unix::fs::PermissionsExt;
        let original_mode = std::fs::metadata(outside.path().join("original"))
            .unwrap()
            .permissions()
            .mode();
        std::fs::remove_file(dir.join("inkscape_mcp_live_run.sh")).unwrap();
        std::fs::hard_link(
            outside.path().join("original"),
            dir.join("inkscape_mcp_live_run.sh"),
        )
        .unwrap();
        install_at(&dir, b"#!/bin/sh\n").unwrap();
        assert_eq!(
            std::fs::metadata(outside.path().join("original"))
                .unwrap()
                .permissions()
                .mode(),
            original_mode
        );
        assert_eq!(
            std::fs::read(outside.path().join("original")).unwrap(),
            b"original"
        );
        assert_eq!(
            std::fs::metadata(dir.join("inkscape_mcp_live_run.sh"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        std::fs::remove_file(dir.join("inkscape_mcp_live_run.sh")).unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("original"),
            dir.join("inkscape_mcp_live_run.sh"),
        )
        .unwrap();
        std::fs::write(dir.join(ASSETS[0].0), b"previous helper").unwrap();
        assert!(install_at(&dir, b"#!/bin/sh\n").is_err());
        assert_eq!(
            std::fs::read(dir.join(ASSETS[0].0)).unwrap(),
            b"previous helper"
        );
        assert_eq!(
            std::fs::read(outside.path().join("original")).unwrap(),
            b"original"
        );
        std::os::unix::fs::symlink(outside.path(), root.join("linked-parent")).unwrap();
        assert!(install_at(&root.join("linked-parent/extensions"), b"#!/bin/sh\n").is_err());
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 1);
        assert!(gate(false).is_err());
    }
}
