//! Version-2 macOS layout checkpoint, assembled from an already verified candidate.
//! Ad-hoc signatures establish relocation integrity only, never Gatekeeper acceptance.
use crate::{archive, common::*};
use inkscape_mcp_rust::runtime_layout::{self, RUNTIME_APP};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
fn macho(path: &Path) -> Result<bool> {
    let mut magic = [0; 4];
    let mut file = fs::File::open(path)?;
    if file.read(&mut magic)? != 4 {
        return Ok(false);
    }
    Ok(matches!(
        magic,
        [0xcf, 0xfa, 0xed, 0xfe]
            | [0xfe, 0xed, 0xfa, 0xcf]
            | [0xca, 0xfe, 0xba, 0xbe]
            | [0xbe, 0xba, 0xfe, 0xca]
    ))
}
fn destination(name: &Path, is_code: bool) -> Result<PathBuf> {
    let text = name.to_str().ok_or("invalid candidate filename")?;
    if text.starts_with("bin/inkscape-mcp.dSYM/") {
        return Ok(Path::new("debug").join(name));
    }
    let contents = Path::new(RUNTIME_APP).join("Contents");
    if is_code {
        if let Some(name) = text.strip_prefix("bin/") {
            ensure(
                [
                    "inkscape-mcp",
                    "inkscape-mcp-launcher",
                    "inkscape-mcp-client",
                    "inkscape-mcp-supervisor",
                    "inkscape-mcp-inx",
                    "inkscape-mcp-live",
                ]
                .contains(&name),
                "unexpected candidate native executable",
            )?;
            return Ok(contents.join("MacOS").join(name));
        }
        if let Some(name) = text.strip_prefix("libexec/inkscape-mcp/dbus/bin/") {
            ensure(
                ["gdbus", "dbus-daemon"].contains(&name),
                "unexpected private bus executable",
            )?;
            return Ok(contents.join("MacOS").join(name));
        }
        if text == "libexec/inkscape-mcp/context.so" {
            return Ok(contents.join("Frameworks/context.so"));
        }
        if let Some(name) = text.strip_prefix("libexec/inkscape-mcp/dbus/lib/") {
            ensure(
                !name.contains('/') && name.ends_with(".dylib"),
                "unexpected private native library",
            )?;
            return Ok(contents.join("Frameworks").join(name));
        }
        return Err(format!("uninventoried candidate Mach-O: {text}").into());
    }
    Ok(contents
        .join("Resources")
        .join(text.strip_prefix("libexec/inkscape-mcp/").unwrap_or(text)))
}
struct Signing {
    identity: String,
    team: Option<String>,
    profile: Option<String>,
    keychain: Option<PathBuf>,
}
impl Signing {
    fn parse(args: &Args) -> Result<Self> {
        let identity = args.values.get("--identity").cloned();
        let team = args.values.get("--team-id").cloned();
        let profile = args.values.get("--notary-profile").cloned();
        let keychain = args.values.get("--keychain").map(PathBuf::from);
        if let Some(path) = &keychain {
            ensure(
                path.is_absolute() && path.is_file() && !path.is_symlink(),
                "keychain must be an existing absolute regular path",
            )?;
        }
        if identity.is_none() && team.is_none() && profile.is_none() {
            ensure(keychain.is_none(), "keychain requires Developer ID signing")?;
            return Ok(Self {
                identity: "-".into(),
                team: None,
                profile: None,
                keychain: None,
            });
        }
        ensure(
            identity.as_ref().is_some_and(|v| {
                v.starts_with("Developer ID Application:")
                    && v.len() <= 256
                    && !v.contains(['\n', '\r', '\0'])
            }) && team.as_ref().is_some_and(|v| {
                v.len() == 10
                    && v.bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            }) && profile
                .as_ref()
                .is_some_and(|v| inkscape_mcp_rust::update::manifests::identifier(v)),
            "Developer ID preparation requires an Application identity, ten-character Team ID, and bounded notarization keychain profile",
        )?;
        Ok(Self {
            identity: identity.unwrap(),
            team,
            profile,
            keychain,
        })
    }
    fn keychain_args(&self) -> Vec<std::ffi::OsString> {
        self.keychain
            .as_ref()
            .map(|path| vec!["--keychain".into(), path.as_os_str().to_owned()])
            .unwrap_or_default()
    }
    fn verify(&self, path: &Path) -> Result<()> {
        let mut command = Command::new("/usr/bin/codesign");
        command.args(["--verify", "--strict"]);
        if let Some(team) = &self.team {
            command.args([
                "-R",
                &format!("=anchor apple generic and certificate leaf[subject.OU] = \"{team}\""),
            ]);
        }
        output(command.arg(path))?;
        if self.team.is_some() {
            let (_, info) = output_streams(
                Command::new("/usr/bin/codesign")
                    .args(["--display", "--verbose=4"])
                    .arg(path),
            )?;
            verify_developer_signature_info(&info, macho(path).unwrap_or(false))?;
        }
        Ok(())
    }
}
fn verify_developer_signature_info(info: &str, native: bool) -> Result<()> {
    ensure(
        info.lines()
            .any(|line| line.starts_with("Authority=Developer ID Application:")),
        "signature is not Developer ID Application",
    )?;
    let timestamps: Vec<_> = info
        .lines()
        .filter_map(|line| line.strip_prefix("Timestamp="))
        .collect();
    ensure(
        timestamps.len() == 1 && !timestamps[0].trim().is_empty() && timestamps[0] != "none",
        "signature is missing a secure timestamp",
    )?;
    ensure(
        !native || info.contains("(runtime)"),
        "native code is missing hardened runtime",
    )?;
    Ok(())
}
fn sign(path: &Path, signing: &Signing) -> Result<()> {
    let mut command = Command::new("/usr/bin/codesign");
    command
        .args(["--force", "--sign", &signing.identity])
        .args(signing.keychain_args());
    command.arg(if signing.team.is_some() {
        "--timestamp"
    } else {
        "--timestamp=none"
    });
    // Ad-hoc code has no Team ID, so hardened private GLib tools cannot load
    // their libraries. This checkpoint proves relocation without adding library
    // validation entitlements. Developer ID qualification must harden them later.
    if signing.team.is_some()
        || !path
            .file_name()
            .is_some_and(|n| n == "dbus-daemon" || n == "gdbus")
    {
        command.args(["--options", "runtime"]);
    }
    output(command.arg(path))?;
    signing.verify(path)?;
    Ok(())
}
fn code_hash(path: &Path, signing: &Signing) -> Result<String> {
    signing.verify(path)?;
    if path.extension().is_some_and(|e| e == "app") {
        output(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict"])
                .arg(path),
        )?;
    }
    let (_, info) = output_streams(
        Command::new("/usr/bin/codesign")
            .args(["--display", "--verbose=4"])
            .arg(path),
    )?;
    verify_developer_signature_info(
        &info,
        path.extension().is_some_and(|e| e == "app") || macho(path).unwrap_or(false),
    )?;
    let hashes: Vec<_> = info
        .lines()
        .filter_map(|line| line.strip_prefix("CDHash="))
        .collect();
    ensure(
        hashes.len() == 1
            && hashes[0].len() == 40
            && hashes[0]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "missing/ambiguous signed code identity",
    )?;
    Ok(hashes[0].to_owned())
}
fn accepted_log(log: &Value, id: &str, identity: &str, cdhash: &str, label: &str) -> Result<()> {
    let ticket_path = match label {
        "runtime" => "notary-runtime.zip/Inkscape MCP Runtime.app/Contents/MacOS/inkscape-mcp",
        "manager" => {
            "notary-manager.zip/Inkscape MCP Manager.app/Contents/MacOS/inkscape-mcp-manager"
        }
        "dmg" => "Inkscape-MCP-Manager.dmg",
        _ => return Err("unknown notarization container".into()),
    };
    ensure(
        log["status"] == "Accepted"
            && log["jobId"] == id
            && log["sha256"] == identity
            && log["ticketContents"].as_array().is_some_and(|rows| {
                rows.iter().any(|row| {
                    row["path"] == ticket_path
                        && row["cdhash"] == cdhash
                        && if label == "dmg" {
                            row.get("arch").is_none()
                        } else {
                            row["arch"] == "arm64"
                        }
                })
            }),
        "accepted notarization log does not match submitted bytes/identity/signed payload",
    )
}
fn notarize(
    path: &Path,
    out: &Path,
    label: &str,
    signing: &Signing,
    resume: bool,
) -> Result<Value> {
    let Some(profile) = &signing.profile else {
        return Ok(Value::Null);
    };
    let input = if path.extension().is_some_and(|e| e == "app") {
        let zip = out.join(format!("notary-{label}.zip"));
        if !resume {
            output(
                Command::new("/usr/bin/ditto")
                    .args(["-c", "-k", "--keepParent"])
                    .arg(path)
                    .arg(&zip),
            )?;
        }
        zip
    } else {
        path.to_owned()
    };
    let record = out.join(format!("notary-{label}.json"));
    let cdhash = code_hash(path, signing)?;
    let (id, identity) = if resume {
        inkscape_mcp_rust::update::instructions::guarded(&record)?;
        let saved: Value = serde_json::from_slice(&read(&record, 1024 * 1024)?)?;
        let id = saved["id"]
            .as_str()
            .filter(|s| {
                s.len() == 36
                    && s.bytes().enumerate().all(|(i, b)| {
                        if [8, 13, 18, 23].contains(&i) {
                            b == b'-'
                        } else {
                            b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
                        }
                    })
            })
            .ok_or("invalid retained notarization ID")?
            .to_owned();
        let identity = saved["input_sha256"]
            .as_str()
            .filter(|s| inkscape_mcp_rust::update::manifests::hash(s))
            .ok_or("invalid retained notarization input identity")?
            .to_owned();
        if input.extension().is_some_and(|e| e == "zip") {
            inkscape_mcp_rust::update::instructions::guarded(&input)?;
            ensure(
                hash(&input)? == identity,
                "retained notarization archive changed",
            )?;
        }
        if let Some(expected) = saved["payload_cdhash"].as_str() {
            ensure(
                expected == cdhash,
                "signed payload changed since submission",
            )?;
        }
        (id, identity)
    } else {
        ensure(
            !record.exists() && !record.is_symlink(),
            "notarization record already exists; use explicit recovery",
        )?;
        let identity = hash(&input)?;
        let submission: Value = serde_json::from_str(&output(
            Command::new("/usr/bin/xcrun")
                .args(["notarytool", "submit"])
                .args(signing.keychain_args())
                .arg(&input)
                .args([
                    "--keychain-profile",
                    profile,
                    "--no-wait",
                    "--output-format",
                    "json",
                ]),
        )?)?;
        let id = submission["id"]
            .as_str()
            .filter(|s| inkscape_mcp_rust::update::manifests::identifier(s))
            .ok_or("notary submission identity missing")?
            .to_owned();
        inkscape_mcp_rust::update::storage::write_json(
            &record,
            &json!({"id":id,"input_sha256":identity,"payload_cdhash":cdhash,"submission":submission}),
        )?;
        (id, identity)
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(600);
    let status = loop {
        let status: Value = serde_json::from_str(&output(
            Command::new("/usr/bin/xcrun")
                .args(["notarytool", "info"])
                .args(signing.keychain_args())
                .args([
                    &id,
                    "--keychain-profile",
                    profile,
                    "--output-format",
                    "json",
                ]),
        )?)?;
        ensure(
            status["id"] == id,
            "notarization status belongs to another submission",
        )?;
        inkscape_mcp_rust::update::storage::write_json(
            &record,
            &json!({"id":id,"input_sha256":identity,"payload_cdhash":cdhash,"status":status}),
        )?;
        if status["status"] == "Accepted" {
            break status;
        }
        if status["status"] != "In Progress" || std::time::Instant::now() >= deadline {
            let _ = output(
                Command::new("/usr/bin/xcrun")
                    .args(["notarytool", "log", &id, "--keychain-profile", profile])
                    .args(signing.keychain_args())
                    .arg(out.join(format!("notary-{label}-log.json"))),
            );
            return Err(format!(
                "notarization {label} failed or timed out; submission {id} retained at {}",
                record.display()
            )
            .into());
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    };
    let log_path = out.join(format!("notary-{label}-log.json"));
    let fetched = tempfile::tempdir()?;
    let fetched_log = fetched.path().join("log.json");
    output(
        Command::new("/usr/bin/xcrun")
            .args(["notarytool", "log", &id, "--keychain-profile", profile])
            .args(signing.keychain_args())
            .arg(&fetched_log),
    )?;
    let log = json(&fetched_log)?;
    accepted_log(&log, &id, &identity, &code_hash(path, signing)?, label)?;
    if log_path.exists() || log_path.is_symlink() {
        inkscape_mcp_rust::update::instructions::guarded(&log_path)?;
    }
    inkscape_mcp_rust::update::storage::write(&log_path, &read(&fetched_log, 64 * 1024 * 1024)?)?;
    if path.extension().is_some_and(|e| e == "app") {
        let ticket = path.join("Contents/CodeResources");
        if ticket.exists() || ticket.is_symlink() {
            inkscape_mcp_rust::update::instructions::guarded(&ticket)?;
        }
    }
    output(
        Command::new("/usr/bin/xcrun")
            .args(["stapler", "staple"])
            .arg(path),
    )?;
    output(
        Command::new("/usr/bin/xcrun")
            .args(["stapler", "validate"])
            .arg(path),
    )?;
    if path.extension().is_some_and(|e| e == "app") {
        signing.verify(path)?;
        output(
            Command::new("/usr/sbin/spctl")
                .args(["--assess", "--type", "execute"])
                .arg(path),
        )?;
    }
    Ok(
        json!({"id":id,"input_sha256":identity,"payload_cdhash":cdhash,"status":status,"log":log,"log_sha256":hash(&log_path)?,"ticket_validated":true,"submission_reused":resume}),
    )
}
/// Resume only a retained fixed container. Never signs, rebuilds or submits code.
pub fn resume(args: &Args) -> Result<()> {
    args.check(&[
        "--output",
        "--label",
        "--identity",
        "--team-id",
        "--notary-profile",
        "--keychain",
    ])?;
    let signing = Signing::parse(args)?;
    ensure(
        signing.team.is_some(),
        "recovery requires Developer ID qualification",
    )?;
    let requested = args.required("--output")?;
    inkscape_mcp_rust::update::instructions::guarded(&requested)?;
    let out = requested.canonicalize()?;
    inkscape_mcp_rust::update::instructions::guarded(&out)?;
    let _lock = inkscape_mcp_rust::update::storage::Lock::acquire(&out)?;
    let label = args.values.get("--label").ok_or("--label is required")?;
    let path = match label.as_str() {
        "runtime" => out.join("inkscape-mcp-macos-arm64").join(RUNTIME_APP),
        "manager" => out.join("Inkscape MCP Manager.app"),
        "dmg" => out.join("Inkscape-MCP-Manager.dmg"),
        _ => return Err("--label must be runtime, manager or dmg".into()),
    };
    inkscape_mcp_rust::update::instructions::guarded(&path)?;
    let report = out.join(format!("notary-{label}-resume.json"));
    if report.exists() || report.is_symlink() {
        inkscape_mcp_rust::update::instructions::guarded(&report)?;
    }
    let evidence = notarize(&path, &out, label, &signing, true)?;
    inkscape_mcp_rust::update::storage::write_json(&report, &evidence)?;
    println!("{}", serde_json::to_string(&evidence)?);
    Ok(())
}
fn plist(executable: &str, identifier: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>{executable}</string><key>CFBundleIdentifier</key><string>{identifier}</string><key>CFBundleName</key><string>Inkscape MCP Runtime</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleVersion</key><string>2</string><key>LSMinimumSystemVersion</key><string>15.0</string></dict></plist>"#
    )
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&[
        "--package",
        "--output",
        "--identity",
        "--team-id",
        "--notary-profile",
        "--keychain",
        "--instructions",
        "--runtime-reference",
        "--channel",
        "--tag",
    ])?;
    let signing = Signing::parse(args)?;
    let channel = args
        .values
        .get("--channel")
        .map(String::as_str)
        .unwrap_or("prerelease");
    ensure(
        ["stable", "prerelease"].contains(&channel),
        "invalid distribution channel",
    )?;
    let tag = args
        .values
        .get("--tag")
        .map(String::as_str)
        .unwrap_or("local-checkpoint");
    ensure(
        inkscape_mcp_rust::update::manifests::identifier(tag),
        "invalid distribution tag",
    )?;
    ensure(
        cfg!(target_os = "macos") && cfg!(target_arch = "aarch64"),
        "distribution checkpoint requires native Apple Silicon macOS",
    )?;
    let source = args.required("--package")?.canonicalize()?;
    let expected: BTreeMap<String, inkscape_mcp_rust::update::manifests::FileIdentity> =
        inkscape_mcp_rust::update::storage::json(&source.join("FILES.json"))?;
    let mut inventory = inkscape_mcp_rust::update::storage::inventory(&source, 1024 * 1024 * 1024)?;
    inventory.remove("FILES.json");
    ensure(inventory == expected, "candidate inventory differs")?;
    let out = args.required("--output")?;
    fs::create_dir(&out)?;
    let out = out.canonicalize()?;
    let runtime = out.join("inkscape-mcp-macos-arm64");
    let mut code = BTreeMap::new();
    let reference = args.values.get("--runtime-reference").map(PathBuf::from);
    let candidate_metadata = json(&runtime_layout::library(&source).join("package.json"))?;
    let mut reference_evidence = Value::Null;
    let (metadata, runtime_notary) = if let Some(reference) = &reference {
        ensure(
            signing.team.is_some(),
            "immutable signed runtime reuse requires Developer ID preparation",
        )?;
        let release: inkscape_mcp_rust::update::manifests::ReleaseManifest =
            inkscape_mcp_rust::update::storage::json(&reference.join("inkscape-mcp-update.json"))?;
        release.compatible("macos", "aarch64", 15)?;
        ensure(
            release.runtime.launcher_minimum == 2,
            "referenced runtime has no signed bundle contract",
        )?;
        let archive = reference.join("inkscape-mcp-macos-arm64.tar.gz");
        ensure(
            hash(&archive)? == release.runtime.asset.identity.sha256
                && fs::metadata(&archive)?.len() == release.runtime.asset.identity.bytes,
            "referenced runtime bytes differ",
        )?;
        let extracted = tempfile::tempdir()?;
        archive::extract(
            &archive,
            &extracted.path().canonicalize()?,
            Path::new("inkscape-mcp-macos-arm64"),
        )?;
        let original = extracted
            .path()
            .canonicalize()?
            .join("inkscape-mcp-macos-arm64");
        inkscape_mcp_rust::update::install::verify_runtime(&original, &release.runtime)?;
        reference_evidence = json(&reference.join("distribution-evidence.json"))?;
        ensure(
            reference_evidence["developer_id_verified"] == true
                && reference_evidence["team_id"] == json!(signing.team)
                && reference_evidence["notarization"]["runtime"]["status"]["status"] == "Accepted"
                && reference_evidence["notarization"]["runtime"]["ticket_validated"] == true,
            "referenced runtime lacks matching signing/notarization evidence",
        )?;
        let metadata = json(&runtime_layout::library(&original).join("package.json"))?;
        ensure(
            metadata["build_info"] == reference_evidence["build_info"],
            "reference build evidence differs",
        )?;
        for row in reference_evidence["code"]
            .as_array()
            .ok_or("reference code inventory missing")?
        {
            let path = PathBuf::from(row["path"].as_str().ok_or("reference code path missing")?);
            ensure(
                inkscape_mcp_rust::update::storage::managed_relative(
                    path.to_str().ok_or("invalid code path")?,
                ) && hash(&original.join(&path))?
                    == row["final_sha256"]
                        .as_str()
                        .ok_or("reference code hash missing")?,
                "reference code identity differs",
            )?;
            signing.verify(&original.join(&path))?;
            code.insert(
                path,
                row["source"]
                    .as_str()
                    .ok_or("reference origin missing")?
                    .to_owned(),
            );
        }
        output(
            Command::new("/usr/bin/xcrun")
                .args(["stapler", "validate"])
                .arg(original.join(RUNTIME_APP)),
        )?;
        inkscape_mcp_rust::update::storage::copy_tree(&original, &runtime, 1024 * 1024 * 1024)?;
        (
            metadata,
            reference_evidence["notarization"]["runtime"].clone(),
        )
    } else {
        fs::create_dir(&runtime)?;
        let mut outputs = BTreeMap::new();
        for name in expected.keys() {
            if name.starts_with("management/") {
                continue;
            }
            let path = source.join(name);
            let is_code = macho(&path)?;
            let target = destination(Path::new(name), is_code)?;
            ensure(
                outputs.insert(target.clone(), name.clone()).is_none(),
                "colliding distribution layout files",
            )?;
            copy(&path, &runtime.join(&target))?;
            if is_code && !name.starts_with("bin/inkscape-mcp.dSYM/") {
                code.insert(target, name.clone());
            }
        }
        ensure(code.len() >= 9, "incomplete native code inventory")?;
        let by_name: BTreeMap<_, _> = code
            .keys()
            .map(|p| (p.file_name().unwrap().to_owned(), p.clone()))
            .collect();
        ensure(by_name.len() == code.len(), "colliding native code names")?;
        // Rewrite every non-system dependency to another inventoried final code item.
        for target in code.keys() {
            let path = runtime.join(target);
            if path.extension().is_some_and(|e| e == "dylib" || e == "so") {
                output(
                    Command::new("/usr/bin/install_name_tool")
                        .args([
                            "-id",
                            &format!(
                                "@loader_path/{}",
                                path.file_name().unwrap().to_string_lossy()
                            ),
                        ])
                        .arg(&path),
                )?;
            }
            let dependencies = output(Command::new("/usr/bin/otool").arg("-L").arg(&path))?;
            for line in dependencies.lines().skip(1) {
                let dependency = line.trim().split(" (compatibility").next().unwrap();
                if dependency.starts_with("/usr/lib/") || dependency.starts_with("/System/Library/")
                {
                    continue;
                }
                let other = by_name
                    .get(
                        Path::new(dependency)
                            .file_name()
                            .ok_or("dependency name missing")?,
                    )
                    .ok_or("external/unknown native dependency after relocation")?;
                if other == target {
                    continue;
                }
                let adjusted = format!(
                    "@loader_path/{}",
                    relative(&runtime.join(other), path.parent().unwrap())?.display()
                );
                output(
                    Command::new("/usr/bin/install_name_tool")
                        .args(["-change", dependency, &adjusted])
                        .arg(&path),
                )?;
            }
        }
        for target in code
            .keys()
            .filter(|p| p.extension().is_some_and(|e| e == "so" || e == "dylib"))
        {
            sign(&runtime.join(target), &signing)?;
        }
        for target in code
            .keys()
            .filter(|p| !p.extension().is_some_and(|e| e == "so" || e == "dylib"))
        {
            sign(&runtime.join(target), &signing)?;
        }
        let library = runtime_layout::library(&runtime);
        let mut metadata = json(&library.join("package.json"))?;
        metadata["layout"] = json!("macos-runtime-app-v2");
        metadata["release_signed"] = json!(signing.team.is_some());
        metadata["notarized"] = json!(false);
        metadata["update_contract"]["launcher_minimum"] = json!(2);
        metadata.as_object_mut().unwrap().remove("management_app");
        metadata["context_bridge_build"]["sha256"] =
            json!(hash(&runtime_layout::asset(&library, "context.so"))?);
        metadata["code_inventory"] = json!(code.keys().collect::<Vec<_>>());
        for row in metadata["dbus_inputs"]
            .as_array_mut()
            .ok_or("DBus inventory missing")?
        {
            let old = format!(
                "libexec/inkscape-mcp/{}",
                row["output"].as_str().ok_or("missing DBus output")?
            );
            let target = code
                .iter()
                .find(|(_, name)| **name == old)
                .map(|(p, _)| p)
                .ok_or("DBus code inventory mismatch")?;
            row["output"] = json!(target);
            row["output_sha256"] = json!(hash(&runtime.join(target))?);
        }
        write_json(&library.join("package.json"), &metadata)?;
        fs::write(
            runtime.join(RUNTIME_APP).join("Contents/Info.plist"),
            plist("inkscape-mcp", "org.inkscape-mcp.runtime"),
        )?;
        sign(&runtime.join(RUNTIME_APP), &signing)?;
        output(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict"])
                .arg(runtime.join(RUNTIME_APP)),
        )?;
        let runtime_notary =
            notarize(&runtime.join(RUNTIME_APP), &out, "runtime", &signing, false)?;
        let files = inkscape_mcp_rust::update::storage::inventory(&runtime, 1024 * 1024 * 1024)?;
        write_json(&runtime.join("FILES.json"), &serde_json::to_value(files)?)?;
        (metadata, runtime_notary)
    };
    let library = runtime_layout::library(&runtime);
    let app = out.join("Inkscape MCP Manager.app");
    inkscape_mcp_rust::update::storage::copy_tree(
        &source.join("management/Inkscape MCP Manager.app"),
        &app,
        128 * 1024 * 1024,
    )?;
    if let Some(instructions) = args.values.get("--instructions") {
        let instructions = Path::new(instructions).canonicalize()?;
        inkscape_mcp_rust::update::instructions::Instructions::load(&instructions)?;
        let target = app.join("Contents/Resources/instructions");
        fs::remove_dir_all(&target)?;
        inkscape_mcp_rust::update::storage::copy_tree(&instructions, &target, 32 * 1024 * 1024)?;
    }
    let helper = app.join("Contents/Helpers/inkscape-mcp-launcher");
    if let Some(reference) = &reference {
        copy(&source.join("bin/inkscape-mcp-launcher"), &helper)?;
        sign(&helper, &signing)?;
        copy(
            &reference.join("inkscape-mcp-macos-arm64.tar.gz"),
            &app.join("Contents/Resources/offline-runtime.tar.gz"),
        )?;
    } else {
        copy(
            &runtime_layout::binary(&library, "inkscape-mcp-launcher"),
            &helper,
        )?;
        archive::pack(
            &runtime,
            &app.join("Contents/Resources/offline-runtime.tar.gz"),
        )?;
    }
    let info = fs::read_to_string(app.join("Contents/Info.plist"))?;
    fs::write(app.join("Contents/Info.plist"), info.replace("</dict></plist>",&format!("<key>InkscapeMCPChannel</key><string>{channel}</string><key>InkscapeMCPDistribution</key><string>{tag}</string></dict></plist>")))?;
    sign(&app.join("Contents/MacOS/inkscape-mcp-manager"), &signing)?;
    sign(&app, &signing)?;
    output(
        Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(&app),
    )?;
    let manager_notary = notarize(&app, &out, "manager", &signing, false)?;
    // Relocate a real archive to a fresh path and independently verify each signature.
    let archive_path = out.join("inkscape-mcp-macos-arm64.tar.gz");
    if let Some(reference) = &reference {
        copy(
            &reference.join("inkscape-mcp-macos-arm64.tar.gz"),
            &archive_path,
        )?;
    } else {
        archive::pack(&runtime, &archive_path)?;
    }
    let relocated = tempfile::tempdir()?;
    let relocated_root = relocated.path().canonicalize()?;
    archive::extract(
        &archive_path,
        &relocated_root,
        Path::new("inkscape-mcp-macos-arm64"),
    )?;
    let extracted = relocated_root.join("inkscape-mcp-macos-arm64");
    output(
        Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(extracted.join(RUNTIME_APP)),
    )?;
    for target in code.keys() {
        output(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--strict"])
                .arg(extracted.join(target)),
        )?;
    }
    let dmg_root = tempfile::tempdir()?;
    inkscape_mcp_rust::update::storage::copy_tree(
        &app,
        &dmg_root
            .path()
            .canonicalize()?
            .join("Inkscape MCP Manager.app"),
        1024 * 1024 * 1024,
    )?;
    if signing.team.is_some() {
        let copied = dmg_root
            .path()
            .canonicalize()?
            .join("Inkscape MCP Manager.app");
        output(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict"])
                .arg(&copied),
        )?;
        output(
            Command::new("/usr/bin/xcrun")
                .args(["stapler", "validate"])
                .arg(&copied),
        )?;
        output(
            Command::new("/usr/bin/xcrun")
                .args(["stapler", "validate"])
                .arg(extracted.join(RUNTIME_APP)),
        )?;
        output(
            Command::new("/usr/sbin/spctl")
                .args(["--assess", "--type", "execute"])
                .arg(extracted.join(RUNTIME_APP)),
        )?;
    }
    let dmg = out.join("Inkscape-MCP-Manager.dmg");
    output(
        Command::new("/usr/bin/hdiutil")
            .args([
                "create",
                "-format",
                "UDZO",
                "-fs",
                "HFS+",
                "-volname",
                "Inkscape MCP",
            ])
            .arg("-srcfolder")
            .arg(dmg_root.path())
            .arg(&dmg),
    )?;
    if signing.team.is_some() {
        output(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", &signing.identity, "--timestamp"])
                .args(signing.keychain_args())
                .arg(&dmg),
        )?;
        signing.verify(&dmg)?;
    }
    let dmg_notary = notarize(&dmg, &out, "dmg", &signing, false)?;
    let final_code: Vec<Value> = if reference.is_some() {
        serde_json::from_value(reference_evidence["code"].clone())?
    } else {
        code.iter().map(|(target,origin)|Ok(json!({"source":origin,"path":target,"source_sha256":hash(&source.join(origin))?,"final_sha256":hash(&runtime.join(target))?}))).collect::<Result<_>>()?
    };
    let evidence = json!({"format":2,"candidate_inventory_sha256":hash(&source.join("FILES.json"))?,"build_info":metadata["build_info"],"candidate_package_build":candidate_metadata["build_info"],"runtime_reused":reference.is_some(),"runtime_reference":if let Some(reference) = &reference {json(&reference.join("inkscape-mcp-update.json"))?["runtime"].clone()}else{Value::Null},"layout":"macos-runtime-app-v2","signing":if signing.team.is_some(){"Developer ID Application; secure timestamp; hardened runtime"}else{"ad-hoc relocation checkpoint; Rust and Manager hardened; private GLib tools require Developer ID for hardening"},"private_glib_hardened":signing.team.is_some(),"developer_id_verified":signing.team.is_some(),"team_id":signing.team,"notarized":signing.profile.is_some(),"notarization":{"runtime":runtime_notary,"manager":manager_notary,"dmg":dmg_notary},"dmg_sha256":hash(&dmg)?,"dmg_bytes":fs::metadata(&dmg)?.len(),"channel":channel,"tag":tag,"gatekeeper_verified":false,"relocation_signatures_verified":true,"code":final_code,"runtime_archive_sha256":hash(&archive_path)?,"manager_application":app});
    write_json(&out.join("distribution-evidence.json"), &evidence)?;
    println!("{}", serde_json::to_string(&evidence)?);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepted_log_binds_submission_input_and_fixed_signed_container() {
        let id = "31c8eeca-65c4-45d2-a4c1-ec66f2e3b688";
        let identity = "a".repeat(64);
        let cdhash = "b".repeat(40);
        let log = json!({"status":"Accepted","jobId":id,"sha256":identity,"ticketContents":[{"path":"notary-runtime.zip/Inkscape MCP Runtime.app/Contents/MacOS/inkscape-mcp","arch":"arm64","cdhash":cdhash}]});
        assert!(accepted_log(&log, id, &identity, &cdhash, "runtime").is_ok());
        for (field, wrong) in [
            ("status", "Invalid"),
            ("jobId", "other"),
            ("sha256", "different"),
        ] {
            let mut invalid = log.clone();
            invalid[field] = json!(wrong);
            assert!(accepted_log(&invalid, id, &identity, &cdhash, "runtime").is_err());
        }
        for field in ["path", "arch", "cdhash"] {
            let mut invalid = log.clone();
            invalid["ticketContents"][0][field] = json!("different");
            assert!(accepted_log(&invalid, id, &identity, &cdhash, "runtime").is_err());
        }
        for label in ["manager", "dmg", "unknown"] {
            assert!(accepted_log(&log, id, &identity, &cdhash, label).is_err());
        }
        let mut manager = log.clone();
        manager["ticketContents"][0]["path"] = json!(
            "notary-manager.zip/Inkscape MCP Manager.app/Contents/MacOS/inkscape-mcp-manager"
        );
        assert!(accepted_log(&manager, id, &identity, &cdhash, "manager").is_ok());
        let mut dmg = log.clone();
        dmg["ticketContents"][0]["path"] = json!("Inkscape-MCP-Manager.dmg");
        dmg["ticketContents"][0]
            .as_object_mut()
            .unwrap()
            .remove("arch");
        assert!(accepted_log(&dmg, id, &identity, &cdhash, "dmg").is_ok());
    }
    #[test]
    fn developer_signature_requires_authority_secure_timestamp_and_native_hardening() {
        let valid = "Authority=Developer ID Application: Fixture (ABCDE12345)\nTimestamp=Oct 8, 2026 at 12:00:00 PM\nCodeDirectory flags=0x10000(runtime)\n";
        assert!(verify_developer_signature_info(valid, true).is_ok());
        assert!(verify_developer_signature_info(valid, false).is_ok());
        for invalid in [
            valid.replace("Timestamp=", "Signed Time="),
            valid.replace("Timestamp=Oct 8, 2026 at 12:00:00 PM", "Timestamp=none"),
            valid.replace("Developer ID Application:", "Apple Development:"),
        ] {
            assert!(verify_developer_signature_info(&invalid, false).is_err());
        }
        assert!(verify_developer_signature_info(&valid.replace("(runtime)", ""), true).is_err());
    }
    #[test]
    fn native_inventory_uses_supported_locations_and_refuses_extra_code() {
        assert!(destination(Path::new("surprise/tool"), true).is_err());
        assert!(
            destination(
                Path::new("libexec/inkscape-mcp/dbus/lib/modules/plugin.dylib"),
                true
            )
            .is_err()
        );
        assert!(
            destination(Path::new("bin/inkscape-mcp"), true)
                .unwrap()
                .ends_with("Contents/MacOS/inkscape-mcp")
        );
        assert!(
            destination(Path::new("libexec/inkscape-mcp/context.so"), true)
                .unwrap()
                .ends_with("Contents/Frameworks/context.so")
        );
        assert!(
            destination(Path::new("libexec/inkscape-mcp/helpers/fixed.inx"), false)
                .unwrap()
                .ends_with("Contents/Resources/helpers/fixed.inx")
        );
    }
}
