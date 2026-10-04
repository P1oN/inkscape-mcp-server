use std::{
    hash::{Hash, Hasher},
    path::Path,
};
fn fingerprint(path: &Path, hash: &mut std::collections::hash_map::DefaultHasher) {
    println!("cargo:rerun-if-changed={}", path.display());
    if path.is_dir() {
        let mut files: Vec<_> = std::fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        files.sort();
        for file in files {
            if file.file_name().is_some_and(|n| n == "__pycache__")
                || file.extension().is_some_and(|e| e == "pyc")
            {
                continue;
            }
            file.file_name().hash(hash);
            fingerprint(&file, hash);
        }
    } else if let Ok(bytes) = std::fs::read(path) {
        bytes.hash(hash);
    }
}
fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let revision = if root.join(".git").exists() {
        println!("cargo:rerun-if-changed={}", root.join(".git").display());
        for item in ["HEAD", "refs", "packed-refs"] {
            if let Some(path) = std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["rev-parse", "--path-format=absolute", "--git-path", item])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
            {
                // Missing optional packed-refs would make Cargo rebuild every time.
                let path = Path::new(path.trim());
                if path.exists() {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
        }
        std::process::Command::new("git")
            .args(["-C", root.to_str().unwrap(), "rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
    } else {
        println!(
            "cargo:rerun-if-changed={}",
            root.join("SOURCE_REVISION").display()
        );
        std::fs::read_to_string(root.join("SOURCE_REVISION"))
            .unwrap_or_default()
            .lines()
            .nth(1)
            .unwrap_or("")
            .to_owned()
    };
    let revision = revision.trim();
    let revision = if revision.len() == 40 && revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        revision
    } else {
        "unknown"
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for item in [
        "rust/src",
        "rust/build.rs",
        "rust/Cargo.toml",
        "rust/Cargo.lock",
        "runtime",
        "migration/contracts",
        "scripts",
        "setup.sh",
        "run-mcp.sh",
        "skills",
        "uninstall.sh",
    ] {
        fingerprint(&root.join(item), &mut hash);
    }
    for key in ["TARGET", "PROFILE", "RUSTC"] {
        std::env::var(key).unwrap_or_default().hash(&mut hash);
    }
    revision.hash(&mut hash);
    println!("cargo:rustc-env=INKSCAPE_MCP_REVISION={revision}");
    println!(
        "cargo:rustc-env=INKSCAPE_MCP_BUILD_ID={:016x}",
        hash.finish()
    );
}
