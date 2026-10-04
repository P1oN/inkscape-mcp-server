//! Real separate Rust supervisor process with native synthetic children and no Python.
#![cfg(target_os = "macos")]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};
#[test]
fn standalone_supervisor_without_python_owns_manifest_gui_and_bus_lifecycle() {
    let temp = tempfile::tempdir_in(Path::new("/tmp").canonicalize().unwrap()).unwrap();
    let root = temp.path();
    let fixture = root.join("fixture");
    let rustc = std::env::var_os("RUSTC")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("rustc"));
    assert!(
        Command::new(rustc)
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/supervisor-process.rs"))
            .args(["--edition=2024", "-o"])
            .arg(&fixture)
            .status()
            .unwrap()
            .success()
    );
    let package = root.join("relocated-package");
    let lib = package.join("libexec/inkscape-mcp");
    fs::create_dir_all(package.join("bin")).unwrap();
    fs::create_dir_all(lib.join("dbus/bin")).unwrap();
    fs::create_dir_all(lib.join("helpers")).unwrap();
    let supervisor = package.join("bin/inkscape-mcp-supervisor");
    fs::copy(env!("CARGO_BIN_EXE_inkscape-mcp-supervisor"), &supervisor).unwrap();
    fs::copy(&fixture, lib.join("dbus/bin/dbus-daemon")).unwrap();
    fs::write(lib.join("dbus/session.conf"), "fixture").unwrap();
    fs::write(lib.join("context.so"), "fixture bridge").unwrap();
    fs::copy(&fixture, package.join("bin/inkscape-mcp-inx")).unwrap();
    for name in ["inkscape_mcp_insert.inx", "inkscape_mcp_edit.inx"] {
        fs::write(lib.join("helpers").join(name), "fixed helper").unwrap();
    }
    let contents = root.join("Vendor.app/Contents");
    fs::create_dir_all(contents.join("MacOS")).unwrap();
    fs::create_dir_all(contents.join("Resources/lib")).unwrap();
    fs::create_dir_all(contents.join("Resources/share/inkscape/extensions/inkex")).unwrap();
    fs::write(contents.join("Resources/lib/libgtk-3.0.dylib"), "fixture").unwrap();
    fs::write(contents.join("Info.plist"),"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>CFBundleIdentifier</key><string>vendor</string></dict></plist>").unwrap();
    let vendor = contents.join("MacOS/inkscape");
    fs::copy(&fixture, &vendor).unwrap();
    let original = fs::read(&vendor).unwrap();
    let session = root.join("session");
    fs::create_dir(&session).unwrap();
    fs::set_permissions(&session, fs::Permissions::from_mode(0o700)).unwrap();
    let mut child = Command::new(&supervisor)
        .args([&session, &vendor])
        .env_clear()
        .env("HOME", root)
        .env("PATH", "")
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !session.join("session.json").exists() {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("supervisor failed before readiness: {status}");
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("fixture startup deadline");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(session.join("session.json")).unwrap()).unwrap();
    assert_eq!(manifest["supervisor_pid"], child.id());
    assert_eq!(manifest["context_bridge"], true);
    assert!(
        !lib.join("python").exists(),
        "fixture must not have a Python runtime"
    );
    fs::write(session.join("finish-gui"), "").unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("supervisor exit deadline");
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(!session.join("session.json").exists());
    let bus: i32 = fs::read_to_string(session.join("bus.pid"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(bus, 0) }, -1);
    assert_eq!(fs::read(vendor).unwrap(), original);
}
