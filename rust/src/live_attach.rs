//! Attach existing managed session metadata. No mkdir, daemon, GUI or environment mutation.
use crate::{
    live_bus::{Bus, Context},
    live_probe::Inputs,
    workspace::Workspace,
};
use serde_json::Value;
use std::{os::unix::fs::MetadataExt, time::Duration};
const ERROR: &str = "managed Inkscape session unavailable; check launcher diagnostics";
pub struct Attached {
    pub inputs: Inputs,
    pub guarded: bool,
}
pub fn refresh(mut inputs: Inputs) -> Result<Attached, &'static str> {
    let mut attached = Attached {
        inputs: inputs.clone(),
        guarded: false,
    };
    let Some(path) = inputs.directory.clone().filter(|_| inputs.macos) else {
        return Ok(attached);
    };
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            inputs.stream = None;
            if std::env::var("DBUS_SESSION_BUS_ADDRESS")
                .is_ok_and(|s| s.starts_with(&format!("unix:path={}/bus.sock", path.display())))
            {
                inputs.session_bus = false;
            }
            attached.inputs = inputs;
            return Ok(attached);
        }
        Err(_) => return Err(ERROR),
    };
    if meta.file_type().is_symlink() {
        return Err(ERROR);
    }
    // Canonicalize the trusted OS /tmp alias used by the legacy launcher, then
    // pin every real ancestor and the root descriptor before reading metadata.
    let directory = path.canonicalize().map_err(|_| ERROR)?;
    let workspace = Workspace {
        roots: vec!["/".into()],
        max_input: 8192,
        max_output: 1024 * 1024,
    };
    let relative = directory.strip_prefix("/").map_err(|_| ERROR)?;
    let handle = workspace.directory_file(0, relative).map_err(|_| ERROR)?;
    let info = handle.metadata().map_err(|_| ERROR)?;
    if info.uid() != unsafe { libc::getuid() }
        || info.mode() & 0o7777 != 0o700
        || directory
            .join("bus.sock")
            .as_os_str()
            .as_encoded_bytes()
            .len()
            >= 104
    {
        return Err(ERROR);
    }
    inputs.directory = Some(directory.clone());
    inputs.stream = None;
    // D-Bus address metacharacters/percent decoding must not reinterpret an
    // owned filesystem path as another endpoint or a transport fallback.
    if !directory
        .to_str()
        .ok_or(ERROR)?
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || b"/._-".contains(&c))
    {
        return Err(ERROR);
    }
    let prefix = format!("unix:path={}/bus.sock", directory.to_str().ok_or(ERROR)?);
    if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_ok_and(|s| s.starts_with(&prefix)) {
        inputs.session_bus = false;
    }
    let manifest = workspace
        .read(0, &relative.join("session.json"), 8192)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    if let Some(data) = manifest.filter(Value::is_object)
        && let Some(address) = data["address"]
            .as_str()
            .filter(|s| private_address(s, &prefix))
        && let Some(binary) = inputs.binary.clone()
    {
        let mut bus = Bus::new(binary, Duration::from_secs(2), inputs.cap);
        bus.address(Some(address.into()));
        let guard = data["context_bridge"] == true;
        if bus.connect().is_ok() && (!guard || bus.context(Context::List).is_ok()) {
            inputs.session_bus = true;
            inputs.address = Some(address.into());
            inputs.stream = Some(directory.join("inkscape.stdout.log"));
            attached.guarded = guard;
        }
    }
    attached.inputs = inputs;
    Ok(attached)
}
fn private_address(address: &str, prefix: &str) -> bool {
    address == prefix
        || address
            .strip_prefix(prefix)
            .and_then(|tail| tail.strip_prefix(",guid="))
            .is_some_and(|guid| guid.len() == 32 && guid.bytes().all(|c| c.is_ascii_hexdigit()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn address_is_only_the_fixed_private_unix_socket_and_optional_guid() {
        let prefix = "unix:path=/private/tmp/test/bus.sock";
        assert!(private_address(prefix, prefix));
        assert!(private_address(
            &format!("{prefix},guid={}", "a".repeat(32)),
            prefix
        ));
        for tail in [
            "other",
            ",guid=short",
            ";tcp:host=example.test",
            ",guid=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;tcp:host=example.test",
            ",other=yes",
        ] {
            assert!(!private_address(&format!("{prefix}{tail}"), prefix));
        }
    }
    #[test]
    fn refresh_requires_owned_private_directory_and_nofollow_manifest() {
        use std::os::unix::fs::PermissionsExt;
        let (root, backend, log) = crate::live_dbus::tests::fixture("effect-success", 4096);
        let directory = root.path().canonicalize().unwrap();
        let inputs = Inputs {
            session_bus: false,
            binary: Some(backend.bus.binary.clone()),
            macos: true,
            stream: None,
            directory: Some(directory.clone()),
            timeout: Duration::from_secs(1),
            cap: 8192,
            address: None,
        };
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(refresh(inputs.clone()).is_err());
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
        let address = format!("unix:path={}/bus.sock", directory.display());
        let data = serde_json::json!({"address":address,"context_bridge":true});
        std::fs::write(
            directory.join("session.json"),
            serde_json::to_vec(&data).unwrap(),
        )
        .unwrap();
        let result = refresh(inputs.clone()).unwrap();
        assert!(result.guarded);
        assert!(result.inputs.session_bus);
        assert_eq!(result.inputs.address.as_deref(), data["address"].as_str());
        let unsafe_root = directory.join("bad;tcp");
        std::fs::create_dir(&unsafe_root).unwrap();
        std::fs::set_permissions(&unsafe_root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut unsafe_inputs = inputs.clone();
        unsafe_inputs.directory = Some(unsafe_root);
        assert!(refresh(unsafe_inputs).is_err());
        let before = std::fs::read(&log).unwrap();
        std::fs::remove_file(directory.join("session.json")).unwrap();
        std::os::unix::fs::symlink(directory.join("gdbus"), directory.join("session.json"))
            .unwrap();
        assert!(!refresh(inputs).unwrap().inputs.session_bus);
        assert_eq!(std::fs::read(&log).unwrap(), before);
        for line in String::from_utf8(before).unwrap().lines().skip(2) {
            let argv: Vec<String> = serde_json::from_str(line).unwrap();
            assert_eq!(argv[1], "--address");
            assert_eq!(argv[2], address);
            assert!(matches!(
                argv[8].as_str(),
                "org.freedesktop.DBus.GetNameOwner"
                    | "org.gtk.Actions.List"
                    | "org.inkscape.MCP.Context1.ListDocuments"
            ));
        }
    }
}
