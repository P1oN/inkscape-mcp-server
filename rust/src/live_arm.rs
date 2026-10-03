//! Explicit fixed helper arming. Startup/connection never calls this module.
use crate::{live_install, live_socket, process, workspace::Workspace};
use serde_json::{Value, json};
use std::{
    io::Write,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const BLANK: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\" viewBox=\"0 0 100 100\"></svg>\n";
fn installed(capabilities: &Value) -> bool {
    let workspace = Workspace {
        roots: vec![PathBuf::from("/")],
        max_input: 1024 * 1024,
        max_output: 1024 * 1024,
    };
    ["system_data_dir", "user_data_dir"]
        .iter()
        .filter_map(|key| capabilities[key].as_str())
        .filter(|s| !s.is_empty())
        .any(|directory| {
            let path = Path::new(directory).join("extensions/inkscape_mcp_live.py");
            let absolute = if path.is_absolute() {
                Some(path)
            } else {
                std::env::current_dir().ok().map(|cwd| cwd.join(path))
            };
            absolute
                .and_then(|path| {
                    path.strip_prefix("/")
                        .ok()
                        .and_then(|relative| workspace.file_kind(0, relative).ok())
                })
                .flatten()
                == Some(libc::S_IFREG)
        })
}
fn display_available() -> bool {
    cfg!(target_os = "macos")
        || cfg!(target_os = "windows")
        || ["DISPLAY", "WAYLAND_DISPLAY"]
            .iter()
            .any(|key| std::env::var_os(key).is_some_and(|v| !v.is_empty()))
}
fn launch(user: &str, cap: usize) -> Result<(), String> {
    // Mirror the source's second discovery, including its race with the tool-level check.
    if live_socket::discover(Some(user), cap).is_some() {
        return Ok(());
    }
    let binary = process::inkscape_binary()
        .ok_or("inkscape binary not found; cannot arm the live socket helper")?;
    if !display_available() {
        return Err("no GUI display available to launch a live Inkscape (the socket helper needs a headful instance); arm it from a desktop session, or run live_install_helper and invoke the helper from Inkscape's Extensions menu".into());
    }
    let mut document = tempfile::Builder::new()
        .prefix("inkscape-mcp-arm-")
        .suffix(".svg")
        .tempfile()
        .map_err(|_| "could not prepare the live helper launch document")?;
    document
        .write_all(BLANK)
        .and_then(|_| document.as_file().sync_all())
        .map_err(|_| "could not prepare the live helper launch document")?;
    // Keep the minted document alive for GUI use and OS temp retention, as the reference does.
    let (_, path) = document
        .keep()
        .map_err(|_| "could not prepare the live helper launch document")?;
    let mut command = Command::new(binary);
    command
        .args(["--with-gui", "--actions=org.inkscape_mcp.live.noprefs"])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: runs only async-signal-safe setsid in our newly forked child.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = command
        .spawn()
        .map_err(|_| "could not launch a live Inkscape to arm the socket helper")?;
    // Observe/reap only this owned child; timeout/disconnect must never terminate its drawing.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    let deadline = Instant::now() + process::timeout().max(Duration::from_secs(5));
    while Instant::now() < deadline {
        if live_socket::discover(Some(user), cap).is_some() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Err(
        "live Inkscape launched but the socket helper did not advertise a rendezvous in time"
            .into(),
    )
}
pub fn arm(capabilities: &Value, cap: usize) -> Result<Value, String> {
    let user = capabilities["user_data_dir"].as_str().filter(|s| !s.is_empty()).ok_or("could not determine the Inkscape extensions directory; call list_capabilities to see what this runtime supports")?;
    if !installed(capabilities) {
        live_install::install(capabilities)?;
    }
    let already = live_socket::discover(Some(user), cap).is_some();
    launch(user, cap)?;
    Ok(
        json!({"armed":true,"launched":!already,"helper_installed":true,"transport":"extension-socket","notes":[if already { "reused an already-armed live session" } else { "launched a headful Inkscape and armed the socket helper" }]}),
    )
}
