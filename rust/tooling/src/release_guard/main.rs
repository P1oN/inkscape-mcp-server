use flate2::read::GzDecoder;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ASSETS: &[&str] = &[
    "Inkscape-MCP-Manager.dmg",
    "inkscape-mcp-macos-arm64.tar.gz",
    "inkscape-mcp-instructions.tar.gz",
    "inkscape-mcp-launcher.tar.gz",
    "inkscape-mcp-source-bootstrap.tar.gz",
    "inkscape-mcp-update.json",
    "CANDIDATE.json",
    "distribution-evidence.json",
    "FINAL-ACCEPTANCE.json",
    "RELEASE-METADATA.json",
    "Inkscape-MCP-Manager.dmg.sha256",
    "inkscape-mcp-macos-arm64.tar.gz.sha256",
    "inkscape-mcp-instructions.tar.gz.sha256",
    "inkscape-mcp-launcher.tar.gz.sha256",
    "inkscape-mcp-source-bootstrap.tar.gz.sha256",
];
fn require(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into().into())
    }
}
fn at<'a>(value: &'a Value, pointer: &str) -> Result<&'a Value> {
    value
        .pointer(pointer)
        .filter(|v| !v.is_null())
        .ok_or_else(|| format!("Missing field: {pointer}").into())
}
fn string<'a>(value: &'a Value, pointer: &str) -> Result<&'a str> {
    at(value, pointer)?
        .as_str()
        .ok_or_else(|| format!("Expected string: {pointer}").into())
}
fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn regular(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    require(metadata.is_file(), "Missing/linked final asset")?;
    Ok(metadata)
}
fn read(path: &Path, cap: u64) -> Result<Vec<u8>> {
    require(regular(path)?.len() <= cap, "Input exceeds byte budget")?;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(cap + 1)
        .read_to_end(&mut bytes)?;
    require(bytes.len() as u64 <= cap, "Input grew beyond byte budget")?;
    Ok(bytes)
}
fn load(root: &Path, name: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&read(
        &root.join(name),
        64 * 1024 * 1024,
    )?)?)
}
fn digest(path: &Path) -> Result<String> {
    regular(path)?;
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn team() -> Result<String> {
    let team = std::env::var("EXPECTED_SIGNING_TEAM")?;
    require(
        team.len() == 10
            && team
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
        "Invalid expected signing team",
    )?;
    Ok(team)
}
fn signature(info: &str, native: bool) -> Result<()> {
    require(
        info.lines()
            .any(|l| l.starts_with("Authority=Developer ID Application:")),
        "signature is not Developer ID Application",
    )?;
    let timestamps: Vec<_> = info
        .lines()
        .filter_map(|l| l.strip_prefix("Timestamp="))
        .collect();
    require(
        timestamps.len() == 1 && !timestamps[0].trim().is_empty() && timestamps[0] != "none",
        "signature is missing a secure timestamp",
    )?;
    require(
        !native || info.contains("(runtime)"),
        "native code is missing hardened runtime",
    )
}
fn startup(value: &Value) -> Result<()> {
    require(
        value["sessions"] == 128
            && value["completed"] == 128
            && value["parallel"] == 4
            && hex(string(value, "/binary_sha256")?, 64),
        "128-session/four-client launcher startup required",
    )
}
fn validate(root: &Path, expected_team: &str) -> Result<Value> {
    let c = load(root, "CANDIDATE.json")?;
    let e = load(root, "distribution-evidence.json")?;
    let a = load(root, "FINAL-ACCEPTANCE.json")?;
    let m = load(root, "inkscape-mcp-update.json")?;
    require(
        c["format"] == 1 && hex(string(&c, "/source_revision")?, 40),
        "Invalid candidate provenance",
    )?;
    require(
        e["format"] == 2
            && e["developer_id_verified"] == true
            && e["notarized"] == true
            && e["private_glib_hardened"] == true,
        "Unsigned/unhardened distribution",
    )?;
    require(
        string(&e, "/team_id")? == expected_team,
        "Wrong signing team",
    )?;
    require(
        at(&e, "/tag")? == at(&c, "/tag")? && at(&c, "/tag")? == at(&m, "/tag")?,
        "Tag differs from candidate",
    )?;
    require(
        at(&e, "/candidate_package_build")? == at(&c, "/package_build")?,
        "Final Manager candidate differs from source build",
    )?;
    require(
        at(&m, "/prerelease")? == at(&c, "/prerelease")?,
        "Channel mismatch",
    )?;
    require(
        at(&m, "/runtime/source_revision")? == at(&e, "/build_info/revision")?
            && at(&m, "/runtime/build_id")? == at(&e, "/build_info/build_id")?,
        "Runtime identity mismatch",
    )?;
    if at(&e, "/runtime_reused")? == true {
        require(
            at(&m, "/runtime")? == at(&c, "/runtime_reference")?
                && at(&c, "/runtime_reference")? == at(&e, "/runtime_reference")?,
            "Immutable runtime reference changed",
        )?;
    } else {
        require(e["runtime_reused"] == false, "Invalid runtime reuse flag")?;
        require(
            at(&m, "/runtime/source_revision")? == at(&c, "/source_revision")?,
            "Runtime source differs from candidate",
        )?;
    }
    require(
        m["runtime"]["launcher_minimum"] == 2 && m["runtime"]["format"] == 1,
        "Unsupported bundle update contract",
    )?;
    let code = at(&e, "/code")?
        .as_array()
        .ok_or("Invalid code inventory")?;
    let mut paths = BTreeSet::new();
    let prefix = "Inkscape MCP Runtime.app/Contents/";
    for item in code {
        let path = string(item, "/path")?;
        require(
            path.starts_with(prefix)
                && hex(string(item, "/final_sha256")?, 64)
                && paths.insert(path),
            "Invalid/duplicate code identity",
        )?;
    }
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
        require(
            paths.contains(format!("{prefix}MacOS/{name}").as_str()),
            "Incomplete code inventory",
        )?;
    }
    require(
        paths.contains(format!("{prefix}Frameworks/context.so").as_str())
            && paths
                .iter()
                .any(|p| p.starts_with(&format!("{prefix}Frameworks/")) && p.ends_with(".dylib")),
        "Incomplete context/library inventory",
    )?;
    for label in ["runtime", "manager", "dmg"] {
        let r = at(&e, &format!("/notarization/{label}"))?;
        require(
            r["status"]["status"] == "Accepted"
                && r["ticket_validated"] == true
                && at(r, "/id")? == at(r, "/status/id")?,
            format!("Missing accepted ticket: {label}"),
        )?;
        require(
            r["log"]["status"] == "Accepted"
                && at(r, "/log/jobId")? == at(r, "/id")?
                && at(r, "/log/sha256")? == at(r, "/input_sha256")?
                && hex(string(r, "/input_sha256")?, 64)
                && hex(string(r, "/log_sha256")?, 64),
            format!("Accepted notarization log differs from submission: {label}"),
        )?;
    }
    require(
        digest(&root.join(ASSETS[0]))? == string(&e, "/dmg_sha256")?
            && json!(regular(&root.join(ASSETS[0]))?.len()) == *at(&e, "/dmg_bytes")?,
        "Final DMG changed",
    )?;
    require(
        digest(&root.join(ASSETS[1]))? == string(&e, "/runtime_archive_sha256")?,
        "Final runtime changed",
    )?;
    for key in ["runtime", "instruction_asset", "launcher_asset"] {
        let asset = at(
            &m,
            if key == "runtime" {
                "/runtime/asset"
            } else if key == "instruction_asset" {
                "/instruction_asset"
            } else {
                "/launcher_asset"
            },
        )?;
        let name = string(asset, "/name")?;
        require(ASSETS.contains(&name), "Unexpected component asset")?;
        require(
            digest(&root.join(name))? == string(asset, "/sha256")?
                && json!(regular(&root.join(name))?.len()) == *at(asset, "/bytes")?,
            "Component final-byte mismatch",
        )?;
    }
    require(
        a["format"] == 1 && a["passed"] == true && a["runtime_exercised"] == true,
        "Final distinct-runtime acceptance missing/failed",
    )?;
    require(
        at(&a, "/source_revision")? == at(&c, "/source_revision")?
            && at(&a, "/team_id")? == at(&e, "/team_id")?,
        "Acceptance provenance mismatch",
    )?;
    for name in ASSETS.iter().filter(|n| **n != "FINAL-ACCEPTANCE.json") {
        require(
            at(&a, &format!("/assets/{name}"))? == &json!(digest(&root.join(name))?),
            format!("Acceptance bound to different final asset: {name}"),
        )?;
    }
    for key in [
        "signatures",
        "package",
        "installer",
        "updates",
        "launcher_startup",
    ] {
        require(a["checks"][key] == true, "Missing final native gate")?;
    }
    startup(at(&a, "/launcher_startup")?)?;
    for name in ASSETS.iter().filter(|n| n.ends_with(".sha256")) {
        let original = name.strip_suffix(".sha256").ok_or("Invalid sidecar")?;
        let contents = String::from_utf8(read(&root.join(name), 4096)?)?;
        require(
            contents.split_whitespace().collect::<Vec<_>>()
                == [digest(&root.join(original))?.as_str(), original],
            format!("Incorrect checksum sidecar: {name}"),
        )?;
    }
    let mut inventory = serde_json::Map::new();
    for name in ASSETS {
        inventory.insert(
            (*name).into(),
            json!({"sha256":digest(&root.join(name))?,"bytes":regular(&root.join(name))?.len()}),
        );
    }
    Ok(Value::Object(inventory))
}
const INSTALLER_GATES: &[&str] = &[
    "missing_client_exercised",
    "foreign_binding_refused",
    "skill_conflict_preserved",
    "legacy_custom_root_migration",
    "component_recovery",
    "custom_engine_reused",
    "missing_engine_picker_repair",
];
fn record(root: &Path, expected_team: &str) -> Result<()> {
    let c = load(root, "CANDIDATE.json")?;
    for name in [
        "package/acceptance.json",
        "installer/report.json",
        "updates/report.json",
        "launcher-startup/comparison.json",
    ] {
        require(
            load(&root.join("acceptance"), name)?["passed"] == true,
            "Native acceptance failed",
        )?;
    }
    require(
        load(&root.join("acceptance"), "updates/report.json")?["runtime_exercised"] == true,
        "Distinct-runtime activation required",
    )?;
    let installer = load(&root.join("acceptance"), "installer/report.json")?;
    for gate in INSTALLER_GATES {
        require(
            installer[*gate] == true,
            format!("Native installer gate missing: {gate}"),
        )?;
    }
    let s = load(&root.join("acceptance"), "launcher-startup/comparison.json")?;
    startup(&s)?;
    require(
        string(&s, "/binary_sha256")?
            == digest(
                &root.join("Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher"),
            )?,
        "Startup tested a different launcher",
    )?;
    let mut hashes = serde_json::Map::new();
    for name in ASSETS.iter().filter(|n| **n != "FINAL-ACCEPTANCE.json") {
        hashes.insert((*name).into(), json!(digest(&root.join(name))?));
    }
    let receipt = json!({"format":1,"passed":true,"runtime_exercised":true,"source_revision":at(&c,"/source_revision")?,"team_id":expected_team,"packaging_revision":at(&c,"/source_revision")?,"checks":{"signatures":true,"package":true,"installer":true,"updates":true,"launcher_startup":true},"launcher_startup":{"sessions":s["sessions"],"completed":s["completed"],"parallel":s["parallel"],"binary_sha256":s["binary_sha256"]},"assets":hashes});
    // Refusals above leave the old receipt untouched; replace atomically only after all gates.
    let mut file = tempfile::NamedTempFile::new_in(root)?;
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string_pretty(&receipt)?)?;
    file.persist(root.join("FINAL-ACCEPTANCE.json"))?;
    Ok(())
}
fn reference(root: &Path, expected_team: &str) -> Result<Value> {
    let o = load(root, "inkscape-mcp-update.json")?;
    let q = load(root, "requested-update.json")?;
    let c = load(root, "CANDIDATE.json")?;
    let e = load(root, "distribution-evidence.json")?;
    let a = load(root, "FINAL-ACCEPTANCE.json")?;
    let r = at(&o, "/runtime")?;
    require(
        r == at(&q, "/runtime")? && at(r, "/distribution_tag")? == at(&o, "/tag")?,
        "Original immutable runtime reference differs",
    )?;
    require(
        r["launcher_minimum"] == 2
            && e["developer_id_verified"] == true
            && e["private_glib_hardened"] == true
            && string(&e, "/team_id")? == expected_team,
        "Unsigned or differently signed historical runtime cannot be reused",
    )?;
    require(
        e["runtime_reused"] == false && at(&e, "/build_info")? == at(&c, "/package_build")?,
        "Original runtime build evidence differs",
    )?;
    require(
        at(r, "/build_id")? == at(&e, "/build_info/build_id")?
            && at(r, "/source_revision")? == at(&c, "/source_revision")?,
        "Reference source/build differs",
    )?;
    require(
        e["notarization"]["runtime"]["status"]["status"] == "Accepted"
            && e["notarization"]["runtime"]["ticket_validated"] == true,
        "Referenced runtime ticket missing",
    )?;
    require(
        a["passed"] == true
            && a["runtime_exercised"] == true
            && at(&a, "/team_id")? == at(&e, "/team_id")?,
        "Referenced native acceptance missing",
    )?;
    for name in [
        "CANDIDATE.json",
        "distribution-evidence.json",
        "inkscape-mcp-update.json",
        "inkscape-mcp-macos-arm64.tar.gz",
    ] {
        require(
            at(&a, &format!("/assets/{name}"))? == &json!(digest(&root.join(name))?),
            "Reference receipt/final bytes differ",
        )?;
    }
    require(
        string(r, "/asset/sha256")? == digest(&root.join(ASSETS[1]))?
            && at(r, "/asset/bytes")? == &json!(regular(&root.join(ASSETS[1]))?.len()),
        "Reference runtime bytes differ",
    )?;
    Ok(r.clone())
}
fn safe_path(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => (),
            _ => return Err("Unsafe CI archive member".into()),
        }
    }
    require(
        !normalized.as_os_str().is_empty(),
        "Unsafe CI archive member",
    )?;
    Ok(normalized)
}
fn extract(archive: &Path, destination: &Path) -> Result<()> {
    require(
        fs::symlink_metadata(destination).is_err_and(|e| e.kind() == io::ErrorKind::NotFound),
        "Extraction output must be new",
    )?;
    regular(archive)?;
    // Keep one open descriptor and rewind; validate every header before creating output.
    let mut file = fs::File::open(archive)?;
    let mut members = Vec::new();
    let mut seen = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut total = 0u64;
    {
        let mut bundle = tar::Archive::new(GzDecoder::new(&mut file));
        for member in bundle.entries()? {
            let mut member = member?;
            let path = safe_path(&member.path()?)?;
            let kind = member.header().entry_type();
            require(
                (kind.is_file() || kind.is_dir()) && seen.insert(path.clone()),
                "Unsafe CI archive member",
            )?;
            total = total
                .checked_add(member.size())
                .ok_or("CI archive size overflow")?;
            require(
                members.len() < 50_000 && total <= 4 * 1024u64.pow(3),
                "CI artifact exceeds extraction bounds",
            )?;
            if kind.is_file() {
                files.insert(path.clone());
            }
            let mode = member.header().mode()? & 0o755;
            io::copy(&mut member, &mut io::sink())?;
            members.push((path, kind.is_dir(), mode, member.size()));
        }
    }
    for (path, _, _, _) in &members {
        require(
            !path.ancestors().skip(1).any(|p| files.contains(p)),
            "CI archive has conflicting parents",
        )?;
    }
    use std::io::Seek;
    file.rewind()?;
    fs::create_dir(destination)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(destination, fs::Permissions::from_mode(0o700))?;
    }
    let mut bundle = tar::Archive::new(GzDecoder::new(file));
    let mut count = 0;
    for (index, member) in bundle.entries()?.enumerate() {
        let mut member = member?;
        let (path, directory, mode, size) = members
            .get(index)
            .ok_or("CI archive changed after validation")?;
        require(
            safe_path(&member.path()?)? == *path
                && member.header().entry_type().is_dir() == *directory
                && (member.header().mode()? & 0o755) == *mode
                && member.size() == *size
                && (member.header().entry_type().is_file() || *directory),
            "CI archive changed after validation",
        )?;
        let target = destination.join(path);
        if *directory {
            fs::create_dir_all(&target)?;
        } else {
            fs::create_dir_all(target.parent().ok_or("Missing archive parent")?)?;
            let mut output = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&target)?;
            io::copy(&mut member, &mut output)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&target, fs::Permissions::from_mode(*mode))?;
            }
        }
        count += 1;
    }
    require(
        count == members.len(),
        "CI archive changed after validation",
    )
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, mode] if command == "verify-signature-info" && ["native","container"].contains(&mode.as_str()) => {
            let mut bytes = Vec::new();
            io::stdin().take(65537).read_to_end(&mut bytes)?;
            require(bytes.len() <= 65536,"signature display exceeds byte budget")?;
            signature(std::str::from_utf8(&bytes)?, mode == "native")?;
        }
        [command, archive, destination] if command == "extract-ci-artifact" => extract(Path::new(archive),Path::new(destination))?,
        [command, root] => {
            let root = Path::new(root);
            match command.as_str() {
                "verify-signed-distribution" => println!("{}",validate(root,&team()?)?),
                "verify-runtime-reference" => println!("{}",reference(root,&team()?)?),
                "record-signed-acceptance" => record(root,&team()?)?,
                _ => return Err("Unknown release guard command".into()),
            }
        }
        _ => return Err("Usage: release-guard.sh verify-signed-distribution|verify-runtime-reference|record-signed-acceptance DIRECTORY; verify-signature-info native|container; extract-ci-artifact ARCHIVE NEW_DIRECTORY".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Release guard refused: {error}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests;
