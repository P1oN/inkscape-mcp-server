#![cfg(target_os = "macos")]

use std::{
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[test]
fn manager_ui_preserves_context_and_gates_actions_without_launching_helpers() {
    let scratch = tempfile::tempdir().unwrap();
    let binary = scratch.path().join("manager-ui-test");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let compiled = Command::new("/usr/bin/clang")
        .args([
            "-fobjc-arc",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wno-unused-parameter",
        ])
        .arg(root.join("runtime/manager/ui_test.m"))
        .args(["-framework", "Cocoa", "-o"])
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut child = Command::new(&binary)
        .env_clear()
        .env("HOME", scratch.path())
        .env("CFFIXED_USER_HOME", scratch.path())
        .env("PATH", "/usr/bin:/bin")
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "Manager UI fixture: {status}");
            break;
        }
        if start.elapsed() > Duration::from_secs(15) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("Manager UI fixture exceeded deadline");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn manager_output_and_process_deadlines_survive_stalled_helpers() {
    let scratch = tempfile::tempdir().unwrap();
    let binary = scratch.path().join("manager-task-test");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let compiled = Command::new("/usr/bin/clang")
        .args(["-fobjc-arc", "-Wall", "-Wextra", "-Werror"])
        .arg(root.join("runtime/manager/task_io_test.m"))
        .args(["-framework", "Foundation", "-o"])
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    for mode in [
        "normal",
        "ignore-term",
        "held-pipe",
        "oversize",
        "blocked-input",
    ] {
        let mut child = Command::new(&binary).args(["test", mode]).spawn().unwrap();
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "{mode}: {status}");
                break;
            }
            if start.elapsed() > Duration::from_secs(5) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("Manager task fixture exceeded deadline: {mode}");
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}
