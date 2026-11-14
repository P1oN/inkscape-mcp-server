//! Opt-in native replacement qualification, using only owned synthetic apps/profiles.
use super::*;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Stdio},
    time::{Duration, Instant},
};
fn run(command: &mut Command) -> Vec<u8> {
    let out = tempfile::tempfile().unwrap();
    let err = tempfile::tempfile().unwrap();
    let mut out_reader = out.try_clone().unwrap();
    let mut err_reader = err.try_clone().unwrap();
    let mut child = command.stdout(out).stderr(err).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("native fixture command timed out: {command:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    use std::io::{Read, Seek};
    out_reader.rewind().unwrap();
    err_reader.rewind().unwrap();
    let mut stdout = vec![];
    let mut stderr = String::new();
    out_reader.read_to_end(&mut stdout).unwrap();
    err_reader.read_to_string(&mut stderr).unwrap();
    assert!(status.success(), "{command:?}: {stderr}");
    stdout
}
fn sign(path: &Path, identity: &str, identifier: &str) {
    run(Command::new("/usr/bin/codesign")
        .env(
            "HOME",
            std::env::var_os("INKSCAPE_MCP_SIGNING_HOME").unwrap(),
        )
        .env(
            "CFFIXED_USER_HOME",
            std::env::var_os("INKSCAPE_MCP_SIGNING_HOME").unwrap(),
        )
        .args([
            "--force",
            "--sign",
            identity,
            "--timestamp=none",
            "--options",
            "runtime",
            "--identifier",
            identifier,
        ])
        .arg(path));
}
fn app(root: &Path, helper: &Path, identity: &str, build: &str) -> PathBuf {
    let app = root.join(APP);
    mkdir(&app.join("Contents/MacOS")).unwrap();
    mkdir(&app.join("Contents/Helpers")).unwrap();
    run(Command::new("/usr/bin/clang")
        .args([
            "-fobjc-arc",
            "-Wall",
            "-Wextra",
            "-Werror",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../runtime/manager/self_update_fixture.m"
            ),
            "-framework",
            "Cocoa",
            "-o",
        ])
        .arg(app.join(release::EXECUTABLE)));
    fs::copy(helper, app.join(release::HELPER)).unwrap();
    write(&app.join("Contents/Info.plist"),format!(r#"<plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.inkscape-mcp.manager</string><key>CFBundleExecutable</key><string>inkscape-mcp-manager</string><key>CFBundlePackageType</key><string>APPL</string><key>InkscapeMCPDistribution</key><string>{}</string><key>InkscapeMCPBuildID</key><string>{build}</string><key>InkscapeMCPRevision</key><string>{}</string></dict></plist>"#,if build=="native-old"{"v1.0.0"}else if build=="native-only"{"v3.0.0"}else{"v2.0.0"},"a".repeat(40)).as_bytes()).unwrap();
    sign(
        &app.join(release::EXECUTABLE),
        identity,
        "inkscape-mcp-manager",
    );
    sign(&app, identity, release::BUNDLE);
    app
}
struct Network(Vec<u8>);
impl super::super::download::Transport for Network {
    fn get(&self, _: &str, _: u64) -> Result<Vec<u8>> {
        Ok(self.0.clone())
    }
}
fn archive(app: &Path, helper: &Path) -> Vec<u8> {
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    tar.append_dir_all(Path::new("inkscape-mcp-launcher").join(APP), app)
        .unwrap();
    tar.append_path_with_name(helper, "inkscape-mcp-launcher/bin/inkscape-mcp-launcher")
        .unwrap();
    tar.into_inner().unwrap().finish().unwrap()
}
fn runtime(root: &Path, server: &Path, launcher: &Path) {
    use super::super::{install::*, instructions};
    let info: Value = serde_json::from_slice(&run(Command::new(server).arg("--version"))).unwrap();
    let build = info["build_id"].as_str().unwrap();
    let path = root.join("runtime").join(build);
    mkdir(&path.join("bin")).unwrap();
    mkdir(&path.join("libexec/inkscape-mcp")).unwrap();
    fs::copy(server, path.join("bin/inkscape-mcp")).unwrap();
    for name in [
        "inkscape-mcp-client",
        "inkscape-mcp-supervisor",
        "inkscape-mcp-inx",
        "inkscape-mcp-live",
    ] {
        write(&path.join("bin").join(name), b"unused fixture helper").unwrap();
    }
    write_json(
        &path.join("libexec/inkscape-mcp/package.json"),
        &json!({"build_info":info}),
    )
    .unwrap();
    write_json(
        &path.join("FILES.json"),
        &inventory(&path, 512 * 1024 * 1024).unwrap(),
    )
    .unwrap();
    let text = instructions::manifest("native-test", &instructions::default_files());
    let text_root = root.join("instructions").join(&text.content_id);
    mkdir(&text_root).unwrap();
    for (name, bytes) in instructions::default_files() {
        let p = text_root.join(name);
        mkdir(p.parent().unwrap()).unwrap();
        write(&p, &bytes).unwrap();
    }
    write_json(&text_root.join("manifest.json"), &text).unwrap();
    let selected = Selector {
        format: 1,
        channel: "stable".into(),
        previous: None,
        current: Pair {
            tag: "v1.0.0".into(),
            instructions: text,
            runtime: RuntimeManifest {
                format: 1,
                distribution_tag: "v1.0.0".into(),
                build_id: build.into(),
                source_revision: info["revision"].as_str().unwrap().into(),
                os: "macos".into(),
                architecture: "aarch64".into(),
                minimum_os_major: 15,
                text_interface: 1,
                helper_protocol: HELPER_PROTOCOL,
                launcher_minimum: 1,
                asset: Asset {
                    name: "unused.tar.gz".into(),
                    identity: FileIdentity::of(b"unused"),
                },
            },
        },
    };
    let workspace = root.join("workspace");
    mkdir(&workspace).unwrap();
    write(
        &workspace.join("original.svg"),
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    )
    .unwrap();
    let settings = Settings {
        format: 1,
        legacy_skill_owner: root.into(),
        skills: vec![],
        environment: BTreeMap::from([
            (
                "INKSCAPE_MCP_WORKSPACE_ROOTS".into(),
                workspace.display().to_string(),
            ),
            ("INKSCAPE_MCP_LIVE_ENABLED".into(), "false".into()),
            ("INKSCAPE_MCP_ENGINE_MODE".into(), "per_call".into()),
            (
                "PATH".into(),
                "/Applications/Inkscape.app/Contents/MacOS:/usr/bin:/bin".into(),
            ),
        ]),
    };
    write_json(&root.join("settings.json"), &settings).unwrap();
    write_json(&root.join("active.json"), &selected).unwrap();
    mkdir(&root.join("bin")).unwrap();
    for n in ["inkscape-mcp", "inkscape-mcp-launcher"] {
        fs::copy(launcher, root.join("bin").join(n)).unwrap();
    }
    write_json(
        &root.join("bootstrap.json"),
        &release::bootstrap_info(launcher).unwrap()["bootstrap"],
    )
    .unwrap();
}
fn bounded_line(reader: &mut BufReader<ChildStdout>, timeout: Duration) -> String {
    use std::os::fd::AsRawFd;
    let fd = reader.get_ref().as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    assert!(flags >= 0);
    assert_eq!(
        unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) },
        0
    );
    let deadline = Instant::now() + timeout;
    let mut line = String::new();
    loop {
        assert!(
            Instant::now() < deadline,
            "owned native/MCP response timed out"
        );
        match reader.read_line(&mut line) {
            Ok(0) => panic!("owned native/MCP stream closed before a complete response"),
            Ok(_) => {
                assert!(
                    line.len() <= 2 * 1024 * 1024,
                    "native response exceeds limit"
                );
                if line.ends_with('\n') {
                    return line;
                }
            }
            Err(error) => assert!(
                matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ),
                "{error}"
            ),
        }
        assert!(
            line.len() <= 2 * 1024 * 1024,
            "native response exceeds limit"
        );
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "owned native/MCP response timed out");
        let mut poll = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let result = unsafe {
            libc::poll(
                &mut poll,
                1,
                remaining.as_millis().min(i32::MAX as u128) as i32,
            )
        };
        assert!(
            result >= 0
                || std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted,
            "native response polling failed"
        );
    }
}
struct OwnedProcess(Child);
impl std::ops::Deref for OwnedProcess {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}
impl std::ops::DerefMut for OwnedProcess {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
struct Session {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    id: u32,
}
impl Session {
    fn new(root: &Path) -> Self {
        let mut child = Command::new(root.join("bin/inkscape-mcp-launcher"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut s = Self {
            input: child.stdin.take().unwrap(),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            id: 0,
        };
        s.call("initialize",json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"owned-self-update-test","version":"1"}}));
        writeln!(
            s.input,
            "{}",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .unwrap();
        s
    }
    fn call(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        writeln!(
            self.input,
            "{}",
            json!({"jsonrpc":"2.0","id":self.id,"method":method,"params":params})
        )
        .unwrap();
        self.input.flush().unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let line = bounded_line(
                &mut self.output,
                deadline.saturating_duration_since(Instant::now()),
            );

            let v: Value = serde_json::from_str(&line).unwrap();
            if v["id"] == self.id {
                assert!(v.get("error").is_none(), "{v}");
                return v["result"].clone();
            }
        }
    }
    fn verify(&mut self) {
        let tools = self.call("tools/list", json!({}));
        let contract: Value = serde_json::from_str(include_str!(
            "../../../migration/contracts/live-false_raw-false_full_full.json"
        ))
        .unwrap();
        assert_eq!(tools, contract["tools/list"]);
        let result = self.call(
            "tools/call",
            json!({"name":"get_workspace_info","arguments":{}}),
        );
        assert_ne!(result["isError"], true);
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn activate_prepared(root: &Path, application: &Path, prepared: &Value, success: bool) {
    let marker = root.join(format!("close-old-{}", uuid::Uuid::new_v4()));
    let mut parent = OwnedProcess(
        Command::new(application.join(release::EXECUTABLE))
            .arg("--hold")
            .arg(&marker)
            .spawn()
            .unwrap(),
    );
    let mut updater = OwnedProcess(
        Command::new(prepared["activation_helper"].as_str().unwrap())
            .args([
                "manager-online-activate",
                "--preparation",
                prepared["review_id"].as_str().unwrap(),
                "--parent-pid",
                &parent.id().to_string(),
                "--install-dir",
                root.to_str().unwrap(),
                "--json",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let mut output = BufReader::new(updater.stdout.take().unwrap());
    let ready = bounded_line(&mut output, Duration::from_secs(25));
    assert_eq!(
        serde_json::from_str::<Value>(&ready).unwrap()["manager_activation_ready"],
        true
    );
    write(&marker, b"exit only owned confirming app").unwrap();
    assert!(parent.wait().unwrap().success());
    let deadline = Instant::now() + Duration::from_secs(45);
    while updater.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            updater.kill().unwrap();
            updater.wait().unwrap();
            panic!("owned updater did not finish");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(updater.wait().unwrap().success(), success);
}
#[test]
#[ignore = "requires explicitly authorized owned native apps and a Developer ID signing identity"]
fn native_signed_manager_replacement_restart_and_recovery() {
    if std::env::var_os("INKSCAPE_MCP_SELF_UPDATE_CHILD").is_none() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let home = base.join("home");
        mkdir(&home).unwrap();
        let mut child=OwnedProcess(Command::new(std::env::current_exe().unwrap()).args(["--ignored","--exact","update::manager_app::native_tests::native_signed_manager_replacement_restart_and_recovery","--nocapture"]).env("INKSCAPE_MCP_SELF_UPDATE_CHILD","1").env("INKSCAPE_MCP_SIGNING_HOME",std::env::var_os("HOME").unwrap()).env("HOME",&home).env("CODEX_HOME",home.join("custom-codex")).env("CFFIXED_USER_HOME",&home).spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(300);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                eprintln!(
                    "Timed-out owned native fixture retained: {}",
                    temp.keep().display()
                );
                panic!("native fixture exceeded total deadline");
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        if !status.success() {
            eprintln!(
                "Failed owned native fixture retained: {}",
                temp.keep().display()
            );
        }
        assert!(status.success());
        return;
    }
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    let root = home.join("custom installation");
    mkdir(&root).unwrap();
    let apps = home.join("Applications");
    mkdir(&apps).unwrap();
    let identity = std::env::var("INKSCAPE_MCP_ACCEPTANCE_SIGNING_IDENTITY")
        .expect("explicit signing identity required");
    let input = PathBuf::from(std::env::var_os("INKSCAPE_MCP_ACCEPTANCE_LAUNCHER").unwrap());
    let helper = root.join("signed-launcher");
    fs::copy(input, &helper).unwrap();
    sign(&helper, &identity, "inkscape-mcp-launcher");
    let old_helper = root.join("old-launcher");
    fs::copy(&helper, &old_helper).unwrap();
    sign(
        &old_helper,
        &identity,
        "inkscape-mcp-launcher.previous-fixture",
    );
    let old = app(&apps, &old_helper, &identity, "native-old");
    let candidate_dir = root.join("candidate");
    mkdir(&candidate_dir).unwrap();
    let candidate = app(&candidate_dir, &helper, &identity, "native-new");
    let mut manifest = release::fixture_manifest();
    manifest.manager_build = "native-new".into();
    manifest.manager_revision = "a".repeat(40);
    let compiled = release::bootstrap_info(&helper).unwrap();
    manifest.bootstrap = serde_json::from_value(compiled["bootstrap"].clone()).unwrap();
    manifest.launcher_version = compiled["launcher_version"].as_u64().unwrap() as u32;
    manifest.files = inventory(&candidate, 128 * 1024 * 1024).unwrap();
    let network = Network(archive(&candidate, &helper));
    manifest.asset.identity = FileIdentity::of(&network.0);
    release::verify(&candidate, &manifest).unwrap();
    // Complete custom profile, including secrets and conflicting skills, is byte-bound.
    let codex = PathBuf::from(std::env::var_os("CODEX_HOME").unwrap());
    mkdir(&codex.join("skills/inkscape-mcp")).unwrap();
    write(&codex.join("config.toml"), b"model='preserved'\n").unwrap();
    write(&codex.join("auth.json"), b"synthetic-secret-must-survive").unwrap();
    write(
        &codex.join("skills/inkscape-mcp/SKILL.md"),
        b"custom conflicting skill",
    )
    .unwrap();
    let profile_before = inventory(&codex, 1024 * 1024).unwrap();
    let server = PathBuf::from(std::env::var_os("INKSCAPE_MCP_ACCEPTANCE_SERVER").unwrap());
    runtime(&root, &server, &old_helper);
    let retained = inventory(&root.join("runtime"), 512 * 1024 * 1024).unwrap();
    let settings = fs::read(root.join("settings.json")).unwrap();
    let selection = fs::read(root.join("active.json")).unwrap();
    let mut existing = Session::new(&root);
    existing.verify();
    let prepared = prepare_online_verified(
        &root,
        &old,
        manifest.clone(),
        "prerelease",
        &network,
        &helper,
    )
    .unwrap();
    let plan = load(&root, "manager-prepared.json").unwrap();
    assert_eq!(plan.online.as_ref().unwrap().bootstrap.len(), 3);
    // Real publication and rollback through each durable state, followed by actual
    // permanent-launcher discovery against the current contract and workspace call.
    for boundary in [
        "app-backup",
        "app-publish",
        "bootstrap-0",
        "bootstrap-1",
        "bootstrap-2",
    ] {
        write_json(&root.join("manager-journal.json"), &plan).unwrap();
        assert!(
            publish_files(&root, &plan, |at| if at == boundary {
                Err("interrupted".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        recover_online(&root).unwrap();
        assert!(!recover_online(&root).unwrap());
        Session::new(&root).verify();
        existing.verify();
    }
    // A second Manager refuses before readiness and both existing instances live.
    let first_marker = root.join("close-first");
    let second_marker = root.join("close-second");
    let mut first = OwnedProcess(
        Command::new(old.join(release::EXECUTABLE))
            .arg("--hold")
            .arg(&first_marker)
            .spawn()
            .unwrap(),
    );
    let mut second = OwnedProcess(
        Command::new(old.join(release::EXECUTABLE))
            .arg("--hold")
            .arg(&second_marker)
            .spawn()
            .unwrap(),
    );
    let mut refusal_child = OwnedProcess(
        Command::new(prepared["activation_helper"].as_str().unwrap())
            .args([
                "manager-online-activate",
                "--preparation",
                prepared["review_id"].as_str().unwrap(),
                "--parent-pid",
                &first.id().to_string(),
                "--install-dir",
                root.to_str().unwrap(),
                "--json",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let refusal_line = bounded_line(
        &mut BufReader::new(refusal_child.stdout.take().unwrap()),
        Duration::from_secs(25),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while refusal_child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let refusal_status = refusal_child.wait().unwrap();
    assert!(!refusal_status.success());
    let refusal: Value = serde_json::from_str(&refusal_line).unwrap();
    assert!(
        refusal["error"]
            .as_str()
            .unwrap()
            .contains("using the destination")
    );
    assert!(first.try_wait().unwrap().is_none() && second.try_wait().unwrap().is_none());
    assert!(!root.join("manager-journal.json").exists());
    write(&first_marker, b"owned exit").unwrap();
    write(&second_marker, b"owned exit").unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    activate_prepared(&root, &old, &prepared, true);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !root.join("native-startup.json").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(25));
    }
    let startup: Value = json(&root.join("native-startup.json")).unwrap();
    assert_eq!(startup["status"], 0);
    assert_eq!(startup["HOME"], home.to_str().unwrap());
    assert_eq!(startup["CODEX_HOME"], codex.to_str().unwrap());
    assert_eq!(startup["installation"], root.to_str().unwrap());
    assert_eq!(startup["channel"], "prerelease");
    assert!(!root.join("manager-journal.json").exists());
    assert!(root.join("manager-last.json").exists());
    Session::new(&root).verify();
    existing.verify();
    let only_dir = root.join("manager-only-candidate");
    mkdir(&only_dir).unwrap();
    let only = app(&only_dir, &helper, &identity, "native-only");
    let mut only_manifest = manifest.clone();
    only_manifest.tag = "v3.0.0".into();
    only_manifest.manager_build = "native-only".into();
    only_manifest.files = inventory(&only, 128 * 1024 * 1024).unwrap();
    let only_network = Network(archive(&only, &helper));
    only_manifest.asset.identity = FileIdentity::of(&only_network.0);
    for label in ["fresh", "prepared-runtime", "damaged"] {
        let empty = home.join(label);
        mkdir(&empty).unwrap();
        if label == "prepared-runtime" {
            mkdir(&empty.join("runtime")).unwrap();
        }
        if label == "damaged" {
            write(&empty.join("active.json"), b"damaged selector").unwrap();
        }
        let first = prepare_online_verified(
            &empty,
            &old,
            only_manifest.clone(),
            "prerelease",
            &only_network,
            &helper,
        )
        .unwrap();
        assert!(first["bootstrap_changes"].as_array().unwrap().is_empty());
        assert!(!empty.join("settings.json").exists());
        assert!(!empty.join("bin").exists());
    }
    let prepared_only = prepare_online_verified(
        &root,
        &old,
        only_manifest,
        "prerelease",
        &only_network,
        &helper,
    )
    .unwrap();
    assert!(
        prepared_only["bootstrap_changes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let before_failed_restart = inventory(&old, 128 * 1024 * 1024).unwrap();
    write(&root.join("native-no-ack"), b"owned failure fixture").unwrap();
    activate_prepared(&root, &old, &prepared_only, false);
    assert_eq!(
        inventory(&old, 128 * 1024 * 1024).unwrap(),
        before_failed_restart
    );
    assert!(!root.join("manager-journal.json").exists());
    Session::new(&root).verify();
    existing.verify();
    fs::remove_file(root.join("native-no-ack")).unwrap();
    activate_prepared(&root, &old, &prepared_only, true);
    Session::new(&root).verify();
    existing.verify();
    assert_eq!(inventory(&codex, 1024 * 1024).unwrap(), profile_before);
    assert_eq!(fs::read(root.join("settings.json")).unwrap(), settings);
    assert_eq!(fs::read(root.join("active.json")).unwrap(), selection);
    assert_eq!(
        inventory(&root.join("runtime"), 512 * 1024 * 1024).unwrap(),
        retained
    );
    eprintln!(
        "Owned Developer ID synthetic app: verified staging, five publication/recovery boundaries, idempotence, Manager-only, fresh/prepared/damaged roots, failed restart rollback/retry, actual restart/ack, custom profile, retained session and current MCP contract passed. No notarization or final release qualification claimed."
    );
}
