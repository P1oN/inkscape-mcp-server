use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_inkscape-mcp-tools")
}
fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-c")
        .arg("core.hooksPath=/dev/null")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn repository(root: &Path) {
    fs::create_dir(root).unwrap();
    git(root, &["-c", "init.templateDir=", "init", "--quiet"]);
    git(root, &["config", "user.name", "Rust tooling fixture"]);
    git(root, &["config", "user.email", "fixture@example.invalid"]);
    fs::create_dir(root.join("rust")).unwrap();
    fs::write(root.join("rust/source.txt"), b"committed\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "--quiet", "-m", "fixture"]);
}
#[test]
fn source_exports_distinguish_commits_from_unpublished_files() {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    repository(&repo);
    let revision = git(&repo, &["rev-parse", "HEAD"]);
    fs::write(repo.join("rust/source.txt"), b"uncommitted\n").unwrap();
    fs::create_dir_all(repo.join("rust/tooling/src")).unwrap();
    fs::write(repo.join("rust/tooling/src/new.rs"), b"new\n").unwrap();
    for working_tree in [false, true] {
        let output = root.path().join(format!("{working_tree}.tar.gz"));
        let mut command = Command::new(binary());
        command
            .current_dir(&repo)
            .args(["source-archive", "--output"])
            .arg(&output);
        if working_tree {
            command.arg("--working-tree");
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut stream = tar::Archive::new(flate2::read::GzDecoder::new(
            fs::File::open(&output).unwrap(),
        ));
        let mut files = std::collections::BTreeMap::new();
        for entry in stream.entries().unwrap() {
            let mut entry = entry.unwrap();
            let name = entry.path().unwrap().to_string_lossy().into_owned();
            let mut bytes = vec![];
            std::io::Read::read_to_end(&mut entry, &mut bytes).unwrap();
            files.insert(name, bytes);
        }
        let prefix = "inkscape-mcp-source-bootstrap/";
        assert_eq!(
            files[&format!("{prefix}rust/source.txt")],
            if working_tree {
                b"uncommitted\n".as_slice()
            } else {
                b"committed\n"
            }
        );
        assert_eq!(
            files.contains_key(&format!("{prefix}SOURCE_REVISION")),
            !working_tree
        );
        if working_tree {
            assert_eq!(
                files[&format!("{prefix}SOURCE_STATE")],
                b"uncommitted-working-tree\n"
            );
            assert_eq!(files[&format!("{prefix}rust/tooling/src/new.rs")], b"new\n");
        } else {
            assert_eq!(
                files[&format!("{prefix}SOURCE_REVISION")],
                format!("inkscape-mcp-source-v1\n{revision}\n").as_bytes()
            );
        }
        let before = fs::read(&output).unwrap();
        assert!(!command.output().unwrap().status.success());
        assert_eq!(before, fs::read(&output).unwrap());
    }
}
#[test]
fn git_build_identity_tracks_both_normal_and_linked_checkout_paths() {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    repository(&repo);
    git(&repo, &["pack-refs", "--all"]);
    for linked in [false, true] {
        let checkout = if linked {
            let path = root.path().join("linked");
            git(
                &repo,
                &["worktree", "add", "--detach", path.to_str().unwrap()],
            );
            path
        } else {
            repo.clone()
        };
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../build.rs");
        let executable = root.path().join(format!("build-{linked}"));
        let result = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .arg(source)
            .env("CARGO_MANIFEST_DIR", checkout.join("rust"))
            .args(["--edition=2024", "-o"])
            .arg(&executable)
            .status()
            .unwrap();
        assert!(result.success());
        let invoke = || {
            String::from_utf8(
                Command::new(&executable)
                    .env("CARGO_MANIFEST_DIR", checkout.join("rust"))
                    .output()
                    .unwrap()
                    .stdout,
            )
            .unwrap()
        };
        let output = invoke();
        for item in ["HEAD", "refs", "packed-refs"] {
            let path = git(
                &checkout,
                &["rev-parse", "--path-format=absolute", "--git-path", item],
            );
            assert!(Path::new(&path).exists());
            assert!(output.contains(&format!("cargo:rerun-if-changed={path}\n")));
        }
        assert!(output.contains(&format!(
            "cargo:rustc-env=INKSCAPE_MCP_REVISION={}",
            git(&checkout, &["rev-parse", "HEAD"])
        )));
        fs::write(
            checkout.join("rust/source.txt"),
            format!("next commit-{linked}\n"),
        )
        .unwrap();
        git(&checkout, &["add", "."]);
        git(&checkout, &["commit", "--quiet", "-m", "next fixture"]);
        let next = invoke();
        assert_ne!(output, next);
        assert!(next.contains(&format!(
            "cargo:rustc-env=INKSCAPE_MCP_REVISION={}",
            git(&checkout, &["rev-parse", "HEAD"])
        )));
    }
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn failed_bootstrap_download_removes_only_owned_temporary_tools() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let checkout = root.path().join("checkout");
    let scripts = checkout.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    let fake = root.path().join("failed-download");
    fs::write(&fake, b"#!/bin/bash\nexit 22\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    let source = include_str!("../../../scripts/bootstrap-local-package.sh")
        .replace("/usr/bin/curl", fake.to_str().unwrap());
    fs::write(scripts.join("bootstrap-local-package.sh"), source).unwrap();
    let private = checkout.join(".inkscape-mcp-local");
    fs::create_dir(&private).unwrap();
    let sentinel = private.join("existing-user-file");
    fs::write(&sentinel, b"preserve").unwrap();
    let output = Command::new("/bin/bash")
        .arg(scripts.join("bootstrap-local-package.sh"))
        .arg("--fresh-tools")
        .output()
        .unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    assert_eq!(fs::read(sentinel).unwrap(), b"preserve");
    let files: Vec<PathBuf> = fs::read_dir(private)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1);
}
