use super::*;
use std::process::Command;
const TEAM: &str = "ABCDE12345";
struct Fixture {
    temp: tempfile::TempDir,
    c: Value,
    e: Value,
    m: Value,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        for name in ASSETS {
            fs::write(root.join(name), b"synthetic fixture asset").unwrap();
        }
        let c = json!({"format":1,"tag":"v9.0.0","source_revision":"a".repeat(40),"package_build":{"build_id":"fixture","revision":"a".repeat(40)},"prerelease":true});
        let mut code = Vec::new();
        for name in [
            "inkscape-mcp",
            "inkscape-mcp-launcher",
            "inkscape-mcp-client",
            "inkscape-mcp-supervisor",
            "inkscape-mcp-inx",
            "inkscape-mcp-live",
            "dbus-daemon",
            "gdbus",
        ] {
            code.push(json!({"path":format!("Inkscape MCP Runtime.app/Contents/MacOS/{name}"),"final_sha256":"b".repeat(64)}));
        }
        for name in ["context.so", "libgio.dylib"] {
            code.push(json!({"path":format!("Inkscape MCP Runtime.app/Contents/Frameworks/{name}"),"final_sha256":"b".repeat(64)}));
        }
        let ticket = |label: &str| json!({"id":format!("fixture-{label}"),"input_sha256":"c".repeat(64),"log_sha256":"d".repeat(64),"log":{"jobId":format!("fixture-{label}"),"status":"Accepted","sha256":"c".repeat(64)},"status":{"id":format!("fixture-{label}"),"status":"Accepted"},"ticket_validated":true});
        let e = json!({"format":2,"developer_id_verified":true,"notarized":true,"private_glib_hardened":true,"runtime_reused":false,"tag":"v9.0.0","team_id":TEAM,"build_info":c["package_build"],"candidate_package_build":c["package_build"],"code":code,"notarization":{"runtime":ticket("runtime"),"manager":ticket("manager"),"dmg":ticket("dmg")},"dmg_sha256":digest(&root.join(ASSETS[0])).unwrap(),"dmg_bytes":regular(&root.join(ASSETS[0])).unwrap().len(),"runtime_archive_sha256":digest(&root.join(ASSETS[1])).unwrap()});
        let asset = |name: &str| json!({"name":name,"sha256":digest(&root.join(name)).unwrap(),"bytes":regular(&root.join(name)).unwrap().len()});
        let m = json!({"tag":"v9.0.0","prerelease":true,"runtime":{"format":1,"distribution_tag":"v9.0.0","source_revision":"a".repeat(40),"build_id":"fixture","launcher_minimum":2,"asset":asset(ASSETS[1])},"instruction_asset":asset(ASSETS[2]),"launcher_asset":asset(ASSETS[3])});
        let fixture = Self { temp, c, e, m };
        fixture.reset();
        fixture
    }
    fn root(&self) -> &Path {
        self.temp.path()
    }
    fn write(&self, name: &str, value: &Value) {
        fs::write(self.root().join(name), serde_json::to_vec(value).unwrap()).unwrap();
    }
    fn reset(&self) {
        self.write("CANDIDATE.json", &self.c);
        self.write("distribution-evidence.json", &self.e);
        self.write("inkscape-mcp-update.json", &self.m);
        for name in ASSETS.iter().filter(|n| n.ends_with(".sha256")) {
            let original = name.strip_suffix(".sha256").unwrap();
            fs::write(
                self.root().join(name),
                format!(
                    "{}  {original}\n",
                    digest(&self.root().join(original)).unwrap()
                ),
            )
            .unwrap();
        }
        self.receipt();
    }
    fn receipt(&self) {
        let mut assets = serde_json::Map::new();
        for name in ASSETS.iter().filter(|n| **n != "FINAL-ACCEPTANCE.json") {
            assets.insert(
                (*name).into(),
                json!(digest(&self.root().join(name)).unwrap()),
            );
        }
        let c = load(self.root(), "CANDIDATE.json").unwrap();
        self.write("FINAL-ACCEPTANCE.json",&json!({"format":1,"passed":true,"runtime_exercised":true,"source_revision":c["source_revision"],"team_id":TEAM,"checks":{"signatures":true,"package":true,"installer":true,"updates":true,"launcher_startup":true},"launcher_startup":{"sessions":128,"completed":128,"parallel":4,"binary_sha256":"b".repeat(64)},"assets":assets}));
    }
}
#[test]
fn signature_refusals() {
    let valid = "Authority=Developer ID Application: Fixture\nTimestamp=Oct 8, 2026 at noon\nCodeDirectory flags=0x10000(runtime)\n";
    assert!(signature(valid, true).is_ok());
    assert!(signature(&valid.replace("(runtime)", ""), false).is_ok());
    for info in [
        valid.replace("Developer ID Application:", "Apple Development:"),
        valid.replace("Timestamp=Oct 8, 2026 at noon\n", ""),
        valid.replace("Oct 8, 2026 at noon", "none"),
        valid.replace("(runtime)", ""),
        format!("{valid}Timestamp=duplicate\n"),
    ] {
        assert!(signature(&info, true).is_err());
    }
}
#[test]
fn final_distribution_refusals() {
    let f = Fixture::new();
    assert!(validate(f.root(), TEAM).is_ok());
    let cases = [
        (
            "CANDIDATE.json",
            "/package_build",
            json!({"build_id":"different"}),
        ),
        ("CANDIDATE.json", "/source_revision", json!("invalid")),
        (
            "distribution-evidence.json",
            "/team_id",
            json!("ZZZZZ12345"),
        ),
        (
            "distribution-evidence.json",
            "/developer_id_verified",
            json!(false),
        ),
        (
            "distribution-evidence.json",
            "/notarization/runtime/status/status",
            json!("Invalid"),
        ),
        (
            "distribution-evidence.json",
            "/notarization/manager/status/status",
            json!("In Progress"),
        ),
        (
            "distribution-evidence.json",
            "/notarization/runtime/log/jobId",
            json!("other"),
        ),
        (
            "distribution-evidence.json",
            "/notarization/manager/log/sha256",
            json!("e".repeat(64)),
        ),
        (
            "distribution-evidence.json",
            "/notarization/dmg/log/status",
            json!("Invalid"),
        ),
        (
            "distribution-evidence.json",
            "/notarization/dmg/ticket_validated",
            json!(false),
        ),
        ("distribution-evidence.json", "/code", json!([])),
        ("FINAL-ACCEPTANCE.json", "/runtime_exercised", json!(false)),
        ("FINAL-ACCEPTANCE.json", "/checks/installer", json!(false)),
        (
            "FINAL-ACCEPTANCE.json",
            "/checks/launcher_startup",
            Value::Null,
        ),
        (
            "FINAL-ACCEPTANCE.json",
            "/launcher_startup/parallel",
            json!(1),
        ),
    ];
    for (name, pointer, value) in cases {
        f.reset();
        let mut v = load(f.root(), name).unwrap();
        *v.pointer_mut(pointer).unwrap() = value;
        f.write(name, &v);
        assert!(
            validate(f.root(), TEAM).is_err(),
            "accepted {name}{pointer}"
        );
    }
    for suffix in ["context.so", ".dylib"] {
        f.reset();
        let mut v = f.e.clone();
        v["code"]
            .as_array_mut()
            .unwrap()
            .retain(|row| !row["path"].as_str().unwrap().ends_with(suffix));
        f.write("distribution-evidence.json", &v);
        assert!(validate(f.root(), TEAM).is_err());
    }
    f.reset();
    let mut v = f.e.clone();
    let duplicate = v["code"][0].clone();
    v["code"].as_array_mut().unwrap().push(duplicate);
    f.write("distribution-evidence.json", &v);
    assert!(validate(f.root(), TEAM).is_err());
    f.reset();
    fs::write(f.root().join(ASSETS[0]), b"changed").unwrap();
    assert!(validate(f.root(), TEAM).is_err());
    fs::write(f.root().join(ASSETS[0]), b"synthetic fixture asset").unwrap();
    f.reset();
    fs::write(
        f.root().join(ASSETS[10]),
        format!("{}  {}\n", "0".repeat(64), ASSETS[0]),
    )
    .unwrap();
    f.receipt();
    assert!(validate(f.root(), TEAM).is_err());
    f.reset();
    fs::remove_file(f.root().join("CANDIDATE.json")).unwrap();
    assert!(validate(f.root(), TEAM).is_err());
}
#[test]
fn immutable_reference_and_distinct_manager() {
    let f = Fixture::new();
    f.write("requested-update.json", &f.m);
    assert!(reference(f.root(), TEAM).is_ok());
    let mut e = f.e.clone();
    e["developer_id_verified"] = json!(false);
    f.write("distribution-evidence.json", &e);
    assert!(reference(f.root(), TEAM).is_err());
    f.reset();
    let mut e = f.e.clone();
    let mut c = f.c.clone();
    c["source_revision"] = json!("b".repeat(40));
    c["package_build"] = json!({"build_id":"new-manager","revision":"b".repeat(40)});
    c["runtime_reference"] = f.m["runtime"].clone();
    e["candidate_package_build"] = c["package_build"].clone();
    e["runtime_reused"] = json!(true);
    e["runtime_reference"] = f.m["runtime"].clone();
    f.write("CANDIDATE.json", &c);
    f.write("distribution-evidence.json", &e);
    f.receipt();
    assert!(validate(f.root(), TEAM).is_ok());
}
#[test]
fn recording_refuses_incomplete_gates_without_replacing_receipt() {
    let f = Fixture::new();
    let helper = f
        .root()
        .join("Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher");
    fs::create_dir_all(helper.parent().unwrap()).unwrap();
    fs::write(&helper, b"synthetic final launcher").unwrap();
    let mut installer = json!({"passed":true});
    for gate in INSTALLER_GATES {
        installer[*gate] = json!(true);
    }
    let startup = json!({"passed":true,"sessions":128,"completed":128,"parallel":4,"binary_sha256":digest(&helper).unwrap()});
    for (name, v) in [
        ("package/acceptance.json", json!({"passed":true})),
        ("installer/report.json", installer.clone()),
        (
            "updates/report.json",
            json!({"passed":true,"runtime_exercised":true}),
        ),
        ("launcher-startup/comparison.json", startup.clone()),
    ] {
        let path = f.root().join("acceptance").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(&v).unwrap()).unwrap();
    }
    record(f.root(), TEAM).unwrap();
    validate(f.root(), TEAM).unwrap();
    let old = fs::read(f.root().join("FINAL-ACCEPTANCE.json")).unwrap();
    for (key, v) in [
        ("passed", json!(false)),
        ("completed", json!(127)),
        ("parallel", json!(1)),
        ("binary_sha256", json!("0".repeat(64))),
    ] {
        let mut s = startup.clone();
        s[key] = v;
        f.write("acceptance/launcher-startup/comparison.json", &s);
        assert!(record(f.root(), TEAM).is_err());
        assert_eq!(
            fs::read(f.root().join("FINAL-ACCEPTANCE.json")).unwrap(),
            old
        );
    }
    f.write("acceptance/launcher-startup/comparison.json", &startup);
    for gate in INSTALLER_GATES {
        let mut i = installer.clone();
        i[*gate] = json!(false);
        f.write("acceptance/installer/report.json", &i);
        assert!(
            record(f.root(), TEAM)
                .unwrap_err()
                .to_string()
                .contains(gate)
        );
        assert_eq!(
            fs::read(f.root().join("FINAL-ACCEPTANCE.json")).unwrap(),
            old
        );
    }
}
#[test]
fn archive_refusals_before_output() {
    for (label, names, kind) in [
        ("valid", vec!["a/file"], b'0'),
        ("traversal", vec!["../file"], b'0'),
        ("duplicate", vec!["a/file", "a/./file"], b'0'),
        ("parent", vec!["a", "a/file"], b'0'),
        ("symlink", vec!["a"], b'2'),
        ("hardlink", vec!["a"], b'1'),
        ("fifo", vec!["a"], b'6'),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let archive = temp.path().join("fixture.tar.gz");
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            fs::File::create(&archive).unwrap(),
            flate2::Compression::default(),
        ));
        for name in names {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::new(kind));
            header.set_size(if kind == b'0' { 1 } else { 0 });
            header.set_mode(0o644);
            header.as_mut_bytes()[..100].fill(0);
            header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
            header.set_cksum();
            if kind == b'0' {
                tar.append(&header, &b"x"[..]).unwrap();
            } else {
                tar.append(&header, io::empty()).unwrap();
            }
        }
        tar.into_inner().unwrap().finish().unwrap();
        let destination = temp.path().join("out");
        assert_eq!(
            extract(&archive, &destination).is_ok(),
            label == "valid",
            "{label}"
        );
        assert_eq!(destination.exists(), label == "valid");
        if label == "valid" {
            assert!(extract(&archive, &destination).is_err());
        }
    }
}
#[cfg(unix)]
#[test]
fn linked_final_asset_refused() {
    let f = Fixture::new();
    let p = f.root().join(ASSETS[0]);
    fs::rename(&p, f.root().join("real")).unwrap();
    std::os::unix::fs::symlink("real", p).unwrap();
    assert!(validate(f.root(), TEAM).is_err());
}
#[cfg(unix)]
#[test]
fn publisher_draft_upload_and_recovery_boundaries() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let root = f.root();
    fs::create_dir(root.join("scripts")).unwrap();
    fs::create_dir(root.join("mock")).unwrap();
    let inventory = serde_json::to_string(&validate(root, TEAM).unwrap()).unwrap();
    fs::write(
        root.join("scripts/release-guard.sh"),
        "#!/bin/bash\nset -eu\nprintf '%s\\n' \"$SIGNED_FIXTURE_INVENTORY\"\n",
    )
    .unwrap();
    fs::set_permissions(
        root.join("scripts/release-guard.sh"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    fs::write(root.join("mock/gh"), include_str!("gh-fixture.sh")).unwrap();
    fs::set_permissions(root.join("mock/gh"), fs::Permissions::from_mode(0o755)).unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    for mode in [
        "draft",
        "upload-failure",
        "remote-duplicate",
        "missing-created-draft",
        "duplicate-created-draft",
        "second-page",
        "success",
        "recover-success",
        "recover-complete",
        "recover-corrupt",
        "recover-foreign",
        "recover-duplicate",
        "recover-published",
        "recover-source",
    ] {
        for name in ["created", "uploaded", "calls"] {
            let _ = fs::remove_file(root.join(name));
        }
        let mut command = Command::new("/bin/bash");
        command
            .arg(repo.join("scripts/publish-signed-distribution.sh"))
            .arg(root);
        if mode.starts_with("recover-") {
            command.args(["--recover-draft", "42"]);
        }
        let output = command
            .current_dir(root)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    root.join("mock").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("GITHUB_REPOSITORY", "fixture/repository")
            .env("RUNNER_TEMP", root)
            .env("SIGNED_FIXTURE_ROOT", root)
            .env("SIGNED_FIXTURE_INVENTORY", &inventory)
            .env("SIGNED_FIXTURE_MODE", mode)
            .output()
            .unwrap();
        let calls = fs::read_to_string(root.join("calls")).unwrap();
        let success = [
            "success",
            "second-page",
            "recover-success",
            "recover-complete",
        ]
        .contains(&mode);
        assert_eq!(
            output.status.success(),
            success,
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(calls.lines().any(|l| l == "publish"), success, "{mode}");
        if mode == "draft" || (mode.starts_with("recover-") && !success) {
            assert!(
                !calls
                    .lines()
                    .any(|l| ["create", "upload", "publish"].contains(&l)),
                "{mode}"
            );
        }
        if mode == "recover-complete" {
            assert!(!calls.lines().any(|l| l == "upload"));
        }
        if mode == "recover-success" {
            assert!(!calls.lines().any(|l| l == "create"));
            let uploaded = fs::read_to_string(root.join("uploaded-names")).unwrap();
            let actual: BTreeSet<_> = uploaded.lines().collect();
            let expected: BTreeSet<_> = ASSETS
                .iter()
                .copied()
                .filter(|n| ![ASSETS[0], ASSETS[1]].contains(n))
                .collect();
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn archive_resource_limits_and_modes() {
    use std::io::Write;
    for (count, size) in [(50_001, 0), (1, 4 * 1024u64.pow(3) + 1)] {
        let temp = tempfile::tempdir().unwrap();
        let archive = temp.path().join("large.tar.gz");
        let mut gzip = flate2::write::GzEncoder::new(
            fs::File::create(&archive).unwrap(),
            flate2::Compression::default(),
        );
        for index in 0..count {
            let mut header = tar::Header::new_gnu();
            header.set_path(format!("member-{index}")).unwrap();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_mode(0o755);
            header.set_size(size);
            header.set_cksum();
            gzip.write_all(header.as_bytes()).unwrap();
        }
        gzip.write_all(&[0; 1024]).unwrap();
        gzip.finish().unwrap();
        let target = temp.path().join("out");
        assert!(
            extract(&archive, &target)
                .unwrap_err()
                .to_string()
                .contains("extraction bounds")
        );
        assert!(!target.exists());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let archive = temp.path().join("executable.tar.gz");
        let mut bundle = tar::Builder::new(flate2::write::GzEncoder::new(
            fs::File::create(&archive).unwrap(),
            flate2::Compression::default(),
        ));
        let mut header = tar::Header::new_gnu();
        header.set_path("bin/helper").unwrap();
        header.set_mode(0o4755);
        header.set_size(1);
        header.set_cksum();
        bundle.append(&header, &b"x"[..]).unwrap();
        bundle.into_inner().unwrap().finish().unwrap();
        let target = temp.path().join("out");
        extract(&archive, &target).unwrap();
        assert_eq!(
            fs::metadata(target.join("bin/helper"))
                .unwrap()
                .permissions()
                .mode()
                & 0o7777,
            0o755
        );
    }
}
