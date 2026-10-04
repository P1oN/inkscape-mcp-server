//! Read-only package/setup diagnosis. Never launches a bus, GUI or extension.
use crate::{live_launch, process, runtime, workspace::Workspace};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
fn workspace() -> Workspace {
    Workspace {
        roots: vec!["/".into()],
        max_input: 1024 * 1024,
        max_output: 1024 * 1024,
    }
}
fn regular(path: &Path) -> bool {
    path.strip_prefix("/")
        .ok()
        .is_some_and(|relative| workspace().regular_file(0, relative).is_ok())
}
fn compatible_header(header: &[u8], os: &str, arch: &str) -> bool {
    let cpu = match arch {
        "aarch64" => 0x0100000c,
        "x86_64" => 0x01000007,
        _ => return false,
    };
    if header.len() < 32 {
        return false;
    }
    if os == "linux" {
        return header[..6] == [0x7f, b'E', b'L', b'F', 2, 1]
            && u16::from_le_bytes(header[18..20].try_into().unwrap())
                == if arch == "aarch64" { 183 } else { 62 };
    }
    if os != "macos" {
        return false;
    }
    if header[..4] == [0xcf, 0xfa, 0xed, 0xfe] {
        return u32::from_le_bytes(header[4..8].try_into().unwrap()) == cpu;
    }
    let (little, stride) = match header[..4] {
        [0xca, 0xfe, 0xba, 0xbe] => (false, 20),
        [0xbe, 0xba, 0xfe, 0xca] => (true, 20),
        [0xca, 0xfe, 0xba, 0xbf] => (false, 32),
        [0xbf, 0xba, 0xfe, 0xca] => (true, 32),
        _ => return false,
    };
    let read = |bytes: &[u8]| {
        if little {
            u32::from_le_bytes(bytes.try_into().unwrap())
        } else {
            u32::from_be_bytes(bytes.try_into().unwrap())
        }
    };
    let count = read(&header[4..8]) as usize;
    count > 0
        && count <= 64
        && header.len() >= 8 + count * stride
        && (0..count).any(|index| read(&header[8 + index * stride..12 + index * stride]) == cpu)
}
fn compatible_binary(path: &Path) -> bool {
    let Some(relative) = path.strip_prefix("/").ok() else {
        return false;
    };
    let Ok(mut file) = workspace().regular_file(0, relative) else {
        return false;
    };
    let mut header = [0u8; 4096];
    let Ok(count) = file.read(&mut header) else {
        return false;
    };
    compatible_header(
        &header[..count],
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}
fn probe(binary: &PathBuf, args: &[String]) -> Option<String> {
    process::run_bounded(binary, args, Duration::from_secs(5), 65536)
        .ok()
        .filter(|result| result.success && !result.timed_out && result.stdout.len() < 65536)
        .and_then(|result| String::from_utf8(result.stdout).ok())
        .map(|text| text.trim().to_owned())
}
fn engine_info(binary: &PathBuf, argument: &str) -> Option<String> {
    // The official macOS launcher creates preferences even for --version.
    // Keep its bootstrap writes inside an owned temporary diagnostic profile.
    let temp = tempfile::Builder::new()
        .prefix("inkscape-mcp-doctor-")
        .tempdir()
        .ok()?;
    let profile = temp.path().join("profile");
    let config = temp.path().join("config");
    let cache = temp.path().join("cache");
    let bus = format!("unix:path={}/no-bus.sock", temp.path().display());
    let output = process::run_bounded_with_env(
        binary,
        &[argument.into()],
        Duration::from_secs(5),
        65536,
        &[
            ("HOME", temp.path().as_os_str()),
            ("INKSCAPE_PROFILE_DIR", profile.as_os_str()),
            ("XDG_CONFIG_HOME", config.as_os_str()),
            ("XDG_CACHE_HOME", cache.as_os_str()),
            ("DBUS_SESSION_BUS_ADDRESS", std::ffi::OsStr::new(&bus)),
        ],
    )
    .ok()?;
    if !output.success || output.timed_out || output.stdout.len() >= 65536 {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|text| text.trim().to_owned())
}
pub fn report() -> Value {
    let mut checks = BTreeMap::new();
    let mut steps = Vec::new();
    let supported_arch = matches!(std::env::consts::ARCH, "aarch64" | "x86_64");
    let platform = supported_arch && cfg!(any(target_os = "macos", target_os = "linux"));
    let target = format!(
        "{}-{}",
        if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        },
        if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x86_64"
        }
    );
    checks.insert("current_package_platform", platform);
    let library = live_launch::library().ok();
    let manifest = library
        .as_ref()
        .and_then(|library| {
            let path = library.join("package.json");
            workspace()
                .read(0, path.strip_prefix("/").ok()?, 65536)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        })
        .filter(Value::is_object);
    let manifest_ok = manifest
        .as_ref()
        .is_some_and(|manifest| platform && manifest["target"] == target);
    checks.insert("package_manifest", manifest_ok);
    let engine = process::inkscape_binary();
    let version = engine
        .as_ref()
        .and_then(|binary| engine_info(binary, "--version"));
    let engine_ok = version
        .as_deref()
        .and_then(runtime::version)
        .is_some_and(|version| version >= [1, 4, 0]);
    checks.insert("inkscape_minimum_1_4", engine_ok);
    let mut vendor = engine
        .as_ref()
        .and_then(|binary| binary.canonicalize().ok())
        .and_then(|binary| {
            binary
                .parent()
                .and_then(Path::parent)
                .map(|contents| contents.join("Resources/share/inkscape/extensions"))
        });
    if cfg!(target_os = "linux") {
        vendor = engine
            .as_ref()
            .and_then(|binary| engine_info(binary, "--system-data-directory"))
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .map(|path| path.join("extensions"));
    }
    let gtk = vendor
        .as_ref()
        .and_then(|path| path.parent().and_then(Path::parent).and_then(Path::parent))
        .map(|resources| resources.join("lib/libgtk-3.0.dylib"));
    if cfg!(target_os = "macos") {
        checks.insert(
            "official_vendor_gtk3",
            gtk.as_ref().is_some_and(|path| compatible_binary(path)),
        );
    }
    if let Some(library) = &library {
        if cfg!(target_os = "macos") {
            checks.insert(
                "prebuilt_context_architecture",
                compatible_binary(&library.join("context.so")),
            );
        }
        checks.insert(
            "fixed_supervisor",
            library
                .parent()
                .and_then(Path::parent)
                .is_some_and(|package| {
                    compatible_binary(&package.join("bin/inkscape-mcp-supervisor"))
                }),
        );
        checks.insert(
            "fixed_inx_helper",
            library
                .parent()
                .and_then(Path::parent)
                .is_some_and(|package| compatible_binary(&package.join("bin/inkscape-mcp-inx"))),
        );
        checks.insert(
            "fixed_socket_helper",
            library
                .parent()
                .and_then(Path::parent)
                .is_some_and(|package| compatible_binary(&package.join("bin/inkscape-mcp-live"))),
        );
        checks.insert(
            "private_bus_config",
            regular(&library.join("dbus/session.conf")),
        );
        checks.insert(
            "dbus_daemon_architecture",
            compatible_binary(&library.join("dbus/bin/dbus-daemon")),
        );
        checks.insert(
            "gdbus_architecture",
            compatible_binary(&library.join("dbus/bin/gdbus")),
        );
        checks.insert(
            "fixed_helper_assets",
            [
                "inkscape_mcp_insert.inx",
                "inkscape_mcp_edit.inx",
                "inkscape_mcp_live.inx",
            ]
            .iter()
            .all(|name| regular(&library.join("helpers").join(name))),
        );
        checks.insert(
            "private_bus_cli",
            checks["dbus_daemon_architecture"]
                && probe(&library.join("dbus/bin/dbus-daemon"), &["--version".into()]).is_some(),
        );
        checks.insert(
            "private_gdbus_cli",
            checks["gdbus_architecture"]
                && probe(&library.join("dbus/bin/gdbus"), &["help".into()]).is_some(),
        );
    } else {
        checks.insert("package_library", false);
    }
    if cfg!(target_os = "macos") {
        checks.insert("system_codesign", regular(Path::new("/usr/bin/codesign")));
    }
    if !platform {
        steps.push("Use a native package built for this host's OS and architecture.");
    }
    if !manifest_ok {
        steps.push("Extract the complete package and run its bin/inkscape-mcp; do not copy just the executable.");
    }
    if !engine_ok {
        steps.push(if cfg!(target_os = "macos") {
            "Install compatible Inkscape 1.4 or newer, normally at /Applications/Inkscape.app."
        } else { "Install compatible Inkscape 1.4 or newer in /usr/bin or /usr/local/bin, or include its directory in PATH." });
    }
    if checks.iter().any(|(key, ok)| {
        !ok && !matches!(
            *key,
            "current_package_platform" | "package_manifest" | "inkscape_minimum_1_4"
        )
    }) {
        steps.push("A fixed runtime/helper/bridge/bus prerequisite is missing or incompatible; re-extract the package and inspect the failed checks.");
    }
    let ready = checks.values().all(|ok| *ok);
    if ready {
        steps.push(if cfg!(target_os = "macos") {
            "Package prerequisites pass. Start MCP normally; use live_launch only when explicitly requesting a managed GUI."
        } else {
            "Package prerequisites pass. Start MCP normally; managed GUI launch is macOS-only. Linux D-Bus/socket connections require an explicitly opened Inkscape session."
        });
    }
    json!({"state":if ready {if cfg!(target_os = "macos") {"ready_to_launch"} else {"ready_for_mcp"}} else {"setup_incomplete"},"ready":ready,"platform":std::env::consts::OS,"architecture":std::env::consts::ARCH,"checks":checks,"inkscape_version":version,"next_steps":steps,"managed_gui_supported":cfg!(target_os = "macos"),"native_gui_verified":false,"notes":["Read-only prerequisite checks do not prove native GUI, Undo/Redo or signing/notarization acceptance.","No GUI, private bus or extension was launched; no setup was installed or repaired."]})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_headers_refuse_wrong_os_architecture_truncation_and_unbounded_fat_tables() {
        for (arch, machine) in [("aarch64", 183u16), ("x86_64", 62)] {
            let mut elf = [0u8; 64];
            elf[..6].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1]);
            elf[18..20].copy_from_slice(&machine.to_le_bytes());
            assert!(compatible_header(&elf, "linux", arch));
            assert!(!compatible_header(&elf, "macos", arch));
            assert!(!compatible_header(
                &elf,
                "linux",
                if arch == "aarch64" {
                    "x86_64"
                } else {
                    "aarch64"
                }
            ));
            elf[5] = 2;
            assert!(!compatible_header(&elf, "linux", arch));
        }
        let mut macho = [0u8; 32];
        macho[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
        macho[4..8].copy_from_slice(&0x0100000cu32.to_le_bytes());
        assert!(compatible_header(&macho, "macos", "aarch64"));
        assert!(!compatible_header(&macho, "macos", "x86_64"));
        assert!(!compatible_header(&macho[..31], "macos", "aarch64"));
        for (magic, little, stride) in [
            ([0xca, 0xfe, 0xba, 0xbe], false, 20),
            ([0xbe, 0xba, 0xfe, 0xca], true, 20),
            ([0xca, 0xfe, 0xba, 0xbf], false, 32),
            ([0xbf, 0xba, 0xfe, 0xca], true, 32),
        ] {
            let mut fat = vec![0u8; 8 + stride * 2];
            fat[..4].copy_from_slice(&magic);
            let pack = |n: u32| {
                if little {
                    n.to_le_bytes()
                } else {
                    n.to_be_bytes()
                }
            };
            fat[4..8].copy_from_slice(&pack(2));
            fat[8..12].copy_from_slice(&pack(0x01000007));
            fat[8 + stride..12 + stride].copy_from_slice(&pack(0x0100000c));
            assert!(compatible_header(&fat, "macos", "aarch64"));
            assert!(compatible_header(&fat, "macos", "x86_64"));
            assert!(!compatible_header(
                &fat[..fat.len() - 1],
                "macos",
                "aarch64"
            ));
            fat[4..8].copy_from_slice(&pack(65));
            assert!(!compatible_header(&fat, "macos", "aarch64"));
        }
    }
}
