#![cfg(target_os = "macos")]

use std::{
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

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
