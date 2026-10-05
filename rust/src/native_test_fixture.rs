//! Test-only native subprocess fixture. Never linked into the production server.
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};
pub fn binary() -> &'static Path {
    static FIXTURE: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &FIXTURE
        .get_or_init(|| {
            let root = tempfile::tempdir().unwrap();
            let source = root.path().join("fixture.rs");
            std::fs::write(&source, include_str!("../tests/fixtures/live-process.rs")).unwrap();
            let deps = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();
            let mut command =
                Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
            command
                .arg(&source)
                .arg("--edition=2024")
                .arg("-L")
                .arg(format!("dependency={}", deps.display()));
            // Use Cargo's already built dependency artifacts; this performs no downloads.
            for name in ["serde_json", "inkscape_mcp_rust", "libc"] {
                let prefix = format!("lib{name}-");
                let path = std::fs::read_dir(&deps)
                    .unwrap()
                    .map(|row| row.unwrap().path())
                    .filter(|path| {
                        path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with(&prefix)
                            && path.extension().is_some_and(|ext| ext == "rlib")
                    })
                    .max_by_key(|path| path.metadata().unwrap().modified().unwrap())
                    .unwrap();
                command
                    .arg("--extern")
                    .arg(format!("{name}={}", path.display()));
            }
            let binary = root.path().join("fixture");
            let result = command.arg("-o").arg(&binary).output().unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            (root, binary)
        })
        .1
}
pub fn install(root: &Path, mode: &str, backend: &str) -> PathBuf {
    let binary = root.join("gdbus");
    std::fs::copy(self::binary(), &binary).unwrap();
    std::fs::write(
        root.join("fixture-mode.json"),
        serde_json::to_vec(&serde_json::json!({"mode":mode,"backend":backend})).unwrap(),
    )
    .unwrap();
    binary
}
