use inkscape_mcp_rust::update::{
    download::{self, Transport},
    install::*,
    instructions::*,
    manifests::*,
    storage::*,
    transaction::*,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    settings: Settings,
    before: Selector,
}
fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap().join("installation");
    mkdir(&root).unwrap();
    for name in ["runtime", "instructions", "staging", "backups", "history"] {
        mkdir(&root.join(name)).unwrap();
    }
    let binary = Path::new(env!("CARGO_BIN_EXE_inkscape-mcp-rust"));
    let info: Value = serde_json::from_slice(
        &Command::new(binary)
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let build = info["build_id"].as_str().unwrap();
    let runtime = root.join("runtime").join(build);
    mkdir(&runtime.join("bin")).unwrap();
    mkdir(&runtime.join("libexec/inkscape-mcp")).unwrap();
    fs::hard_link(binary, runtime.join("bin/inkscape-mcp")).unwrap();
    for name in [
        "inkscape-mcp-client",
        "inkscape-mcp-supervisor",
        "inkscape-mcp-inx",
        "inkscape-mcp-live",
    ] {
        fs::write(
            runtime.join("bin").join(name),
            "fixture helper; never executed",
        )
        .unwrap();
    }
    write_json(
        &runtime.join("libexec/inkscape-mcp/package.json"),
        &json!({"build_info":info}),
    )
    .unwrap();
    let files = inventory(&runtime, 1024 * 1024 * 1024).unwrap();
    write_json(&runtime.join("FILES.json"), &files).unwrap();
    let instructions = make_bundle(&root, default_files(), "v1");
    let runtime = RuntimeManifest {
        format: 1,
        distribution_tag: "v1".into(),
        build_id: build.into(),
        source_revision: info["revision"].as_str().unwrap().into(),
        os: "macos".into(),
        architecture: "aarch64".into(),
        minimum_os_major: 15,
        text_interface: 1,
        helper_protocol: HELPER_PROTOCOL,
        launcher_minimum: 1,
        asset: Asset {
            name: "runtime.tar.gz".into(),
            identity: FileIdentity::of(b"fixture"),
        },
    };
    let before = Selector {
        format: 1,
        current: Pair {
            runtime,
            instructions,
            tag: "v1".into(),
        },
        previous: None,
        channel: "prerelease".into(),
    };
    let workspace = temp.path().canonicalize().unwrap().join("drawings");
    mkdir(&workspace).unwrap();
    fs::write(workspace.join("original.svg"), "preserve").unwrap();
    let skill = temp
        .path()
        .canonicalize()
        .unwrap()
        .join("skills/inkscape-mcp");
    mkdir(&skill.join("agents")).unwrap();
    mkdir(&skill.join(".inkscape-mcp-upstream/agents")).unwrap();
    for name in ["SKILL.md", "agents/openai.yaml"] {
        let bytes = read(
            &instruction_path(&root, &before.current)
                .unwrap()
                .join("skills/inkscape-mcp")
                .join(name),
            256 * 1024,
        )
        .unwrap();
        write(&skill.join(name), &bytes).unwrap();
        write(&skill.join(".inkscape-mcp-upstream").join(name), &bytes).unwrap();
    }
    write(
        &skill.join(".inkscape-mcp-owner"),
        root.to_str().unwrap().as_bytes(),
    )
    .unwrap();
    let settings = Settings {
        legacy_skill_owner: root.clone(),
        format: 1,
        environment: BTreeMap::from([
            (
                "INKSCAPE_MCP_WORKSPACE_ROOTS".into(),
                workspace.display().to_string(),
            ),
            ("PATH".into(), "/usr/bin:/bin".into()),
            ("INKSCAPE_MCP_LIVE_ENABLED".into(), "false".into()),
            ("INKSCAPE_MCP_ENGINE_MODE".into(), "per_call".into()),
        ]),
        skills: vec![skill],
    };
    write_json(&root.join("active.json"), &before).unwrap();
    write_json(&root.join("settings.json"), &settings).unwrap();
    Fixture {
        _temp: temp,
        root,
        settings,
        before,
    }
}
fn make_bundle(
    root: &Path,
    files: BTreeMap<String, Vec<u8>>,
    version: &str,
) -> InstructionManifest {
    let m = manifest(version, &files);
    let target = root.join("instructions").join(&m.content_id);
    mkdir(&target).unwrap();
    for (name, bytes) in files {
        let p = target.join(name);
        mkdir(p.parent().unwrap()).unwrap();
        write(&p, &bytes).unwrap();
    }
    write_json(&target.join("manifest.json"), &m).unwrap();
    m
}
fn next(f: &Fixture) -> Selector {
    let mut files = default_files();
    files
        .get_mut("initialize.txt")
        .unwrap()
        .extend_from_slice(b"\nExternal update test.");
    files
        .get_mut("skills/inkscape-mcp/SKILL.md")
        .unwrap()
        .extend_from_slice(b"\nUpstream update.");
    let mut next = f.before.clone();
    next.current.instructions = make_bundle(&f.root, files, "v2");
    next.current.tag = "v2".into();
    next.previous = Some(f.before.current.clone());
    next
}
#[test]
fn real_stdio_text_update_preserves_custom_skill_and_rollback_refuses_later_edits() {
    let f = fixture();
    let skill = &f.settings.skills[0];
    let before = read(&skill.join("SKILL.md"), 256 * 1024).unwrap();
    let mut custom = b"Local customization.\n".to_vec();
    custom.extend_from_slice(&before);
    write(&skill.join("SKILL.md"), &custom).unwrap();
    let after = next(&f);
    assert!(
        activate(&f.root, f.before.clone(), after.clone(), &f.settings, None)
            .unwrap()
            .changed
    );
    assert_eq!(selector(&f.root).unwrap(), after);
    let updated = String::from_utf8(read(&skill.join("SKILL.md"), 256 * 1024).unwrap()).unwrap();
    assert!(updated.contains("Local customization."));
    assert!(updated.contains("Upstream update."));
    write(&skill.join("extra.txt"), b"later user edit").unwrap();
    assert!(rollback(&f.root, &f.settings).is_err());
    fs::remove_file(skill.join("extra.txt")).unwrap();
    assert!(rollback(&f.root, &f.settings).unwrap().changed);
    assert_eq!(selector(&f.root).unwrap().current, f.before.current);
    assert_eq!(read(&skill.join("SKILL.md"), 256 * 1024).unwrap(), custom);
    assert_eq!(
        fs::read_to_string(
            Path::new(&f.settings.environment["INKSCAPE_MCP_WORKSPACE_ROOTS"]).join("original.svg")
        )
        .unwrap(),
        "preserve"
    );
}
#[test]
fn skill_conflict_and_noop_preserve_active_state() {
    let f = fixture();
    let before = inventory(&f.settings.skills[0], 16 * 1024 * 1024).unwrap();
    assert!(
        !activate(
            &f.root,
            f.before.clone(),
            f.before.clone(),
            &f.settings,
            None
        )
        .unwrap()
        .changed
    );
    let mut files = default_files();
    files.insert(
        "skills/inkscape-mcp/SKILL.md".into(),
        b"Upstream rewrote everything\n".to_vec(),
    );
    let mut after = f.before.clone();
    after.current.instructions = make_bundle(&f.root, files, "v2");
    write(
        &f.settings.skills[0].join("SKILL.md"),
        b"User rewrote everything\n",
    )
    .unwrap();
    let edited = inventory(&f.settings.skills[0], 16 * 1024 * 1024).unwrap();
    assert_ne!(before, edited);
    assert!(
        activate(&f.root, f.before.clone(), after, &f.settings, None)
            .unwrap_err()
            .contains("conflict")
    );
    assert_eq!(selector(&f.root).unwrap(), f.before);
    assert_eq!(
        inventory(&f.settings.skills[0], 16 * 1024 * 1024).unwrap(),
        edited
    );
    assert!(!f.root.join("transaction.json").exists());
}
#[test]
fn crash_recovery_at_each_move_boundary_is_idempotent_and_locked() {
    for phase in 0..4 {
        let f = fixture();
        let lock = Lock::acquire(&f.root).unwrap();
        assert!(Lock::acquire(&f.root).is_err());
        drop(lock);
        let after = next(&f);
        let base = f.root.join("backups/crash-fixture");
        mkdir(&base).unwrap();
        let skill = &f.settings.skills[0];
        let before = inventory(skill, 16 * 1024 * 1024).unwrap();
        copy_tree(skill, &base.join("new-0"), 16 * 1024 * 1024).unwrap();
        write(&base.join("new-0/SKILL.md"), b"candidate").unwrap();
        let updated = inventory(&base.join("new-0"), 16 * 1024 * 1024).unwrap();
        let journal = Journal {
            format: 1,
            id: "crash-fixture".into(),
            before: f.before.clone(),
            after: after.clone(),
            skills: vec![SkillChange {
                destination: skill.clone(),
                before: before.clone(),
                after: updated,
            }],
        };
        write_json(&base.join("selector-before.json"), &f.before).unwrap();
        write_json(&f.root.join("transaction.json"), &journal).unwrap();
        if phase >= 1 {
            fs::rename(skill, base.join("old-0")).unwrap();
        }
        if phase >= 2 {
            fs::rename(base.join("new-0"), skill).unwrap();
        }
        if phase >= 3 {
            write_json(&f.root.join("active.json"), &after).unwrap();
        }
        assert!(recover(&f.root, &f.settings).unwrap(), "phase {phase}");
        assert!(!recover(&f.root, &f.settings).unwrap());
        assert_eq!(selector(&f.root).unwrap(), f.before);
        assert_eq!(inventory(skill, 16 * 1024 * 1024).unwrap(), before);
    }
}
struct Fake(BTreeMap<String, Vec<u8>>);
impl Transport for Fake {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>> {
        let bytes = self.0.get(url).ok_or("fixture network failure")?.clone();
        if bytes.len() as u64 > limit {
            return Err("fixture byte limit".into());
        }
        Ok(bytes)
    }
}
#[test]
fn fixed_repository_discovery_checks_channels_old_releases_and_asset_hashes() {
    let f = fixture();
    let mut fake = Fake(BTreeMap::new());
    let release = ReleaseManifest {
        format: 1,
        tag: "v2".into(),
        prerelease: true,
        runtime: f.before.current.runtime.clone(),
        instructions: f.before.current.instructions.clone(),
        instruction_asset: Asset {
            name: "instructions.tar.gz".into(),
            identity: FileIdentity::of(b"archive"),
        },
        launcher_asset: Asset {
            name: "launcher.tar.gz".into(),
            identity: FileIdentity::of(b"launcher"),
        },
    };
    fake.0.insert("https://api.github.com/repos/P1oN/inkscape-mcp-server/releases?per_page=100&page=1".into(),serde_json::to_vec(&json!([{"tag_name":"v2","draft":false,"prerelease":true,"assets":[{"name":download::MANIFEST_ASSET}]},{"tag_name":"v1","draft":false,"prerelease":false,"assets":[]}])).unwrap());
    fake.0.insert(
        download::asset_url("v2", download::MANIFEST_ASSET).unwrap(),
        serde_json::to_vec(&release).unwrap(),
    );
    assert_eq!(
        download::discover(&fake, "prerelease", None).unwrap(),
        release
    );
    assert!(download::discover(&fake, "stable", None).is_err());
    assert!(
        download::discover(&fake, "stable", Some("v1"))
            .unwrap_err()
            .contains("old release")
    );
    fake.0.insert(
        download::asset_url("v2", "instructions.tar.gz").unwrap(),
        b"corrupt".to_vec(),
    );
    assert!(download::asset(&fake, "v2", &release.instruction_asset).is_err());
    for url in [
        "http://github.com/asset",
        "https://evil.test/asset",
        "https://github.com.evil.test/",
        "https://user@github.com/",
        "https://github.com:444/",
    ] {
        assert!(download::allowed(url).is_err());
    }
    assert!(download::allowed("https://release-assets.githubusercontent.com/asset").is_ok());
}

#[test]
fn publication_write_failure_restores_selector_and_skills() {
    let f = fixture();
    let after = next(&f);
    let skills = inventory(&f.settings.skills[0], 16 * 1024 * 1024).unwrap();
    // Fail the durable history write after skill moves and selector publication.
    fs::remove_dir(f.root.join("history")).unwrap();
    fs::write(f.root.join("history"), "owned write-failure fixture").unwrap();
    assert!(activate(&f.root, f.before.clone(), after, &f.settings, None).is_err());
    assert_eq!(selector(&f.root).unwrap(), f.before);
    assert_eq!(
        inventory(&f.settings.skills[0], 16 * 1024 * 1024).unwrap(),
        skills
    );
    assert!(!f.root.join("transaction.json").exists());
    probe(&f.root, &f.before.current, &f.settings).unwrap();
}

#[test]
fn startup_waits_for_brief_lock_and_contention_has_a_deadline() {
    use std::time::{Duration, Instant};
    let f = fixture();
    let lock = Lock::acquire(&f.root).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        drop(lock);
    });
    let start = Instant::now();
    inkscape_mcp_rust::client_management::probe(
        Command::new(env!("CARGO_BIN_EXE_inkscape-mcp-launcher"))
            .args(["--install-dir", f.root.to_str().unwrap()]),
        // Startup also verifies the full debug runtime inventory; use the package
        // acceptance budget independently of the much shorter lock deadline below.
        Duration::from_secs(30),
    )
    .unwrap();
    release.join().unwrap();
    assert!(start.elapsed() >= Duration::from_millis(100));
    let _lock = Lock::acquire(&f.root).unwrap();
    let start = Instant::now();
    assert!(Lock::acquire_wait(&f.root, Duration::from_millis(80)).is_err());
    assert!(start.elapsed() >= Duration::from_millis(80));
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(Lock::acquire(&f.root).is_err());
}

#[test]
fn removed_first_skill_is_skipped_and_remaining_skill_rolls_back() {
    let mut f = fixture();
    let first = f.settings.skills[0].clone();
    let second = first.parent().unwrap().join("second-skill");
    copy_tree(&first, &second, 16 * 1024 * 1024).unwrap();
    f.settings.skills.push(second.clone());
    let original = inventory(&second, 16 * 1024 * 1024).unwrap();
    fs::remove_dir_all(&first).unwrap();
    f.settings.validate().unwrap();
    let activation = activate(&f.root, f.before.clone(), next(&f), &f.settings, None).unwrap();
    assert!(activation.changed && activation.skills_changed);
    assert_eq!(activation.skipped_skills, vec![first.clone()]);
    assert_ne!(inventory(&second, 16 * 1024 * 1024).unwrap(), original);
    assert!(!first.exists());
    let restored = rollback(&f.root, &f.settings).unwrap();
    assert_eq!(restored.skipped_skills, vec![first.clone()]);
    assert_eq!(inventory(&second, 16 * 1024 * 1024).unwrap(), original);
    assert!(!first.exists());
    // A post-publication failure must use the same dense indices during recovery.
    let before = selector(&f.root).unwrap();
    fs::remove_dir_all(f.root.join("history")).unwrap();
    fs::write(f.root.join("history"), "owned write-failure fixture").unwrap();
    assert!(activate(&f.root, before.clone(), next(&f), &f.settings, None).is_err());
    assert_eq!(selector(&f.root).unwrap(), before);
    assert_eq!(inventory(&second, 16 * 1024 * 1024).unwrap(), original);
    assert!(!f.root.join("transaction.json").exists());
}

#[test]
fn rollback_preserves_removed_and_recreated_skills() {
    let f = fixture();
    let skill = &f.settings.skills[0];
    activate(&f.root, f.before.clone(), next(&f), &f.settings, None).unwrap();
    fs::remove_dir_all(skill).unwrap();
    let restored = rollback(&f.root, &f.settings).unwrap();
    assert!(restored.changed && !restored.skills_changed);
    assert_eq!(restored.skipped_skills, vec![skill.clone()]);
    assert!(!skill.exists());
    // Roll forward while missing, then recreate a user skill before rollback.
    rollback(&f.root, &f.settings).unwrap();
    mkdir(skill).unwrap();
    write(&skill.join("user.txt"), b"recreated by user").unwrap();
    let user = inventory(skill, 1024).unwrap();
    let restored = rollback(&f.root, &f.settings).unwrap();
    assert!(!restored.skills_changed);
    assert_eq!(restored.skipped_skills, vec![skill.clone()]);
    assert_eq!(inventory(skill, 1024).unwrap(), user);
}

#[test]
fn disconnected_client_is_removed_from_legacy_records() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let repo = root.join("source");
    mkdir(&repo.join(".inkscape-mcp-local")).unwrap();
    let home = root.join("home");
    mkdir(&home.join(".codex")).unwrap();
    let bin = root.join("bin");
    mkdir(&bin).unwrap();
    for client in ["codex", "claude"] {
        write(&bin.join(client), b"#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(bin.join(client), fs::Permissions::from_mode(0o700)).unwrap();
    }
    let record = repo.join(".inkscape-mcp-local/clients.json");
    write_json(&record, &vec!["codex", "claude"]).unwrap();
    for client in ["codex", "claude"] {
        let result = Command::new(env!("CARGO_BIN_EXE_inkscape-mcp-client"))
            .args([
                "--repo",
                repo.to_str().unwrap(),
                "--client",
                client,
                "disconnect",
            ])
            .env_clear()
            .env("HOME", &home)
            .env("CODEX_HOME", home.join(".codex"))
            .env("PATH", &bin)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let clients: Vec<String> = json(&record).unwrap();
        assert!(!clients.iter().any(|name| name == client));
    }
    assert!(!home.join(".claude.json").exists());
    assert!(!home.join(".codex/config.toml").exists());
}
