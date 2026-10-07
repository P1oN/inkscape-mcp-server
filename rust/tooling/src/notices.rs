use crate::{archive, common::*};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn copy_notice(output_dir: &Path, relative: &Path, content: &[u8]) -> Result<Value> {
    ensure(archive::safe(relative), "unsafe notice output path")?;
    let path = output_dir
        .join("libexec/inkscape-mcp/licenses")
        .join(relative);
    ensure(!path.exists(), "notice output collision")?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, content)?;
    Ok(
        json!({"path":path.strip_prefix(output_dir)?.to_string_lossy(),"bytes":content.len(),"sha256":sha(content)}),
    )
}
fn native_sources(output_dir: &Path, gaps: &mut Vec<String>) -> Result<Vec<Value>> {
    native_sources_from(
        output_dir,
        gaps,
        Path::new("migration/vendor-notices/native"),
    )
}
fn native_sources_from(
    output_dir: &Path,
    gaps: &mut Vec<String>,
    vendor: &Path,
) -> Result<Vec<Value>> {
    let destination = output_dir.join("libexec/inkscape-mcp/licenses/dbus");
    let mut rows = vec![];
    if !destination.exists() {
        return Ok(rows);
    }
    for path in walk(&destination)?
        .into_iter()
        .filter(|p| p.file_name().unwrap() == "sbom.spdx.json")
    {
        let name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        let sbom = crate::common::json(&path)?;
        let Some(source) = sbom["packages"].as_array().and_then(|a| {
            a.iter()
                .find(|p| p["SPDXID"] == format!("SPDXRef-Archive-{name}-src"))
        }) else {
            gaps.push(format!("missing native source SBOM: {name}"));
            continue;
        };
        let version = source["versionInfo"]
            .as_str()
            .ok_or("native version missing")?;
        let relative = Path::new(&*name).join(version);
        ensure(archive::safe(&relative), "unsafe native source path")?;
        let root = vendor.join(relative);
        if !root.join("provenance.json").is_file() {
            gaps.push(format!(
                "missing exact native source notice supplement: {name} {version}"
            ));
            continue;
        }
        let provenance = crate::common::json(&root.join("provenance.json"))?;
        let urls = provenance
            .get("equivalent_source_urls")
            .cloned()
            .unwrap_or(json!([provenance["url"]]));
        let checksums: Vec<_> = source["checksums"]
            .as_array()
            .ok_or("native checksums missing")?
            .iter()
            .filter(|v| v["algorithm"] == "SHA256")
            .map(|v| v["checksumValue"].clone())
            .collect();
        ensure(
            provenance["name"] == *name
                && provenance["version"] == version
                && checksums == vec![provenance["sha256"].clone()]
                && urls.as_array().is_some_and(|a| {
                    !a.is_empty()
                        && a[0] == provenance["url"]
                        && a.contains(&source["downloadLocation"])
                }),
            format!("native notice provenance differs: {name}"),
        )?;
        let mut notices = vec![];
        for row in provenance["notices"]
            .as_array()
            .ok_or("native notices missing")?
        {
            let relative = Path::new(row["path"].as_str().ok_or("notice path missing")?);
            ensure(archive::safe(relative), "unsafe notice path")?;
            let path = root.join(relative);
            ensure(!path.is_symlink(), "linked native notice")?;
            let content = read(&path, 4 * 1024 * 1024)?;
            ensure(
                json!(content.len()) == row["bytes"] && sha(&content) == row["sha256"],
                "native notice hash differs",
            )?;
            notices.push(copy_notice(
                output_dir,
                &Path::new("native-source")
                    .join(&*name)
                    .join(version)
                    .join(relative),
                &content,
            )?);
        }
        ensure(!notices.is_empty(), "empty native source notice supplement")?;
        notices.push(copy_notice(
            output_dir,
            &Path::new("native-source")
                .join(&*name)
                .join(version)
                .join("provenance.json"),
            &read(&root.join("provenance.json"), 4 * 1024 * 1024)?,
        )?);
        rows.push(json!({"source":provenance,"notices":notices}));
    }
    Ok(rows)
}
fn glib(output_dir: &Path, target: &str, gaps: &mut Vec<String>) -> Result<Value> {
    let recipe =
        output_dir.join("libexec/inkscape-mcp/licenses/dbus/glib/build-metadata/.brew/glib.rb");
    if !target.starts_with("macos") || !recipe.is_file() {
        gaps.push(format!(
            "Exact native GLib build metadata unavailable for {target}"
        ));
        return Ok(Value::Null);
    }
    let root = Path::new("migration/vendor-notices/native-build/glib/2.90.0");
    let provenance = crate::common::json(&root.join("provenance.json"))?;
    if hash(&recipe)? != provenance["formula_sha256"] {
        gaps.push("Native GLib recipe differs from reviewed bottle".into());
        return Ok(Value::Null);
    }
    let mut notices = vec![];
    for row in provenance["files"]
        .as_array()
        .ok_or("GLib notice files missing")?
    {
        let relative = Path::new(row["path"].as_str().ok_or("GLib notice path missing")?);
        ensure(archive::safe(relative), "unsafe GLib notice path")?;
        let content = read(&root.join(relative), 4 * 1024 * 1024)?;
        ensure(
            json!(content.len()) == row["bytes"] && sha(&content) == row["sha256"],
            "native build source metadata differs",
        )?;
        notices.push(copy_notice(
            output_dir,
            &Path::new("native-build/glib/2.90.0").join(relative),
            &content,
        )?);
    }
    notices.push(copy_notice(
        output_dir,
        Path::new("native-build/glib/2.90.0/provenance.json"),
        &read(&root.join("provenance.json"), 4 * 1024 * 1024)?,
    )?);
    Ok(json!({"source":provenance,"notices":notices}))
}
pub fn collect(output_dir: &Path, target: &str, triple: &str) -> Result<Value> {
    let cargo = tool("cargo")?;
    let tree_args = [
        "tree",
        "--locked",
        "--offline",
        "--manifest-path",
        "rust/Cargo.toml",
        "--target",
        triple,
        "--prefix",
        "none",
        "--edges",
        "normal,build",
        "--format",
        "{p}",
    ];
    let graph = output(Command::new(&cargo).args(tree_args))?;
    let regex = regex::Regex::new(r"^([\w-]+) v([\w.+-]+)")?;
    let pairs: BTreeSet<_> = graph
        .lines()
        .filter_map(|line| regex.captures(line))
        .filter(|c| &c[1] != "inkscape-mcp-rust")
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect();
    ensure(
        !pairs.is_empty() && pairs.len() <= 512,
        "unexpected native dependency graph size",
    )?;
    let lock: toml::Value = toml::from_str(std::str::from_utf8(&read(
        Path::new("rust/Cargo.lock"),
        4 * 1024 * 1024,
    )?)?)?;
    let locked: BTreeMap<_, _> = lock["package"]
        .as_array()
        .ok_or("lock packages missing")?
        .iter()
        .map(|p| {
            (
                (p["name"].as_str().unwrap(), p["version"].as_str().unwrap()),
                p,
            )
        })
        .collect();
    let cargo_root = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or(PathBuf::from(std::env::var_os("HOME").ok_or("HOME missing")?).join(".cargo"));
    let caches: Vec<_> = fs::read_dir(cargo_root.join("registry/cache"))?
        .map(|row| row.map(|row| row.path()))
        .collect::<std::io::Result<_>>()?;
    let mut rows = vec![];
    let mut gaps = vec![];
    for (name, version) in pairs {
        let package = locked
            .get(&(name.as_str(), version.as_str()))
            .ok_or("crate missing from lock")?;
        ensure(
            package["source"].as_str()
                == Some("registry+https://github.com/rust-lang/crates.io-index"),
            "unsupported crate notice source",
        )?;
        let archives: Vec<_> = caches
            .iter()
            .map(|root| root.join(format!("{name}-{version}.crate")))
            .filter(|p| p.is_file())
            .collect();
        ensure(
            archives.len() == 1,
            format!("missing/ambiguous locked crate archive: {name}"),
        )?;
        let (metadata, vcs, mut notices) = archive::crate_notices(
            &archives[0],
            package["checksum"]
                .as_str()
                .ok_or("lock checksum missing")?,
            &name,
            &version,
        )?;
        let mut upstream = Value::Null;
        if notices.is_empty() && (name.as_str(), version.as_str()) == ("rmcp", "3.5.0") {
            let root = Path::new("migration/vendor-notices/rmcp-3.5.0");
            upstream = crate::common::json(&root.join("provenance.json"))?;
            let content = read(&root.join("LICENSE"), 4 * 1024 * 1024)?;
            ensure(
                vcs["git"]["sha1"] == upstream["commit"] && sha(&content) == upstream["sha256"],
                "rmcp notice provenance differs",
            )?;
            notices.insert("LICENSE".into(), content);
        }
        if notices.is_empty() {
            gaps.push(format!("missing crate license text: {name} {version}"));
        }
        let mut copied = vec![];
        for (path, content) in notices {
            copied.push(copy_notice(
                output_dir,
                &Path::new("rust")
                    .join(format!("{name}-{version}"))
                    .join(path),
                &content,
            )?);
        }
        rows.push(json!({"name":name,"version":version,"source":package["source"].as_str(),"archive_sha256":package["checksum"].as_str(),"license_expression":metadata.get("license").and_then(toml::Value::as_str),"repository":metadata.get("repository").and_then(toml::Value::as_str),"upstream_notice":upstream,"notices":copied}));
    }
    let sysroot = command(tool("rustc")?, &["--print", "sysroot"])?;
    let docs = Path::new(sysroot.trim()).join("share/doc/rust");
    let mut standard = vec![docs.join("COPYRIGHT-library.html")];
    for row in fs::read_dir(docs.join("licenses"))? {
        let path = row?.path();
        if path.extension().is_some_and(|e| e == "txt") {
            standard.push(path);
        }
    }
    ensure(
        standard.len() > 1,
        "Rust standard-library notice text unavailable",
    )?;
    standard.sort();
    let mut std_rows = vec![];
    for path in standard {
        std_rows.push(copy_notice(
            output_dir,
            &Path::new("rust-standard-library").join(path.file_name().unwrap()),
            &read(&path, 4 * 1024 * 1024)?,
        )?);
    }
    let native_sources = native_sources(output_dir, &mut gaps)?;
    let glib_build = glib(output_dir, target, &mut gaps)?;
    let destination = output_dir.join("libexec/inkscape-mcp/licenses");
    let native_paths: Vec<_> = walk(&destination)?
        .into_iter()
        .filter(|p| {
            !p.starts_with(destination.join("rust"))
                && !p.starts_with(destination.join("rust-standard-library"))
        })
        .map(|p| {
            p.strip_prefix(output_dir)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    gaps.extend([
        "Native bus/GLib/gettext source obligations and referenced license texts need audit."
            .into(),
        "Foreign target and exact linked-vs-build dependency attribution are not established."
            .into(),
    ]);
    let mut argv = vec![cargo.to_string_lossy().into_owned()];
    argv.extend(tree_args.iter().map(|s| s.to_string()));
    let inventory = json!({"target":target,"cargo_tree_command":argv,"cargo_tree":graph,"cargo_lock_sha256":hash(Path::new("rust/Cargo.lock"))?,"rust_crates":rows,"rust_standard_library_notices":std_rows,"native_notice_paths":native_paths,"native_source_supplements":native_sources,"glib_build_source_supplement":glib_build,"redistribution_audit_complete":false,"remaining_gaps":gaps});
    write_json(&output_dir.join("LICENSE-INVENTORY.json"), &inventory)?;
    let mut text = String::from(
        "# Third-party notices\n\nThe server's MIT license is in LICENSE. Third-party terms remain separate.\nNative-target Cargo normal/build dependency notices are included conservatively;\nthis inventory does not claim every listed crate is linked into the binary.\nRust standard-library notices are in libexec/inkscape-mcp/licenses/rust-standard-library.\nNative bus notices and SBOMs are under libexec/inkscape-mcp/licenses.\nLICENSE-INVENTORY.json records source hashes and remaining audit gaps.\nNo license alternatives are silently reselected.\n\n| Crate | Version | Declared license |\n|---|---|---|\n",
    );
    for row in &rows {
        text.push_str(&format!(
            "| {} | {} | {} |\n",
            row["name"].as_str().unwrap(),
            row["version"].as_str().unwrap(),
            row["license_expression"].as_str().unwrap_or("None")
        ));
    }
    fs::write(output_dir.join("THIRD_PARTY_NOTICES.md"), text)?;
    Ok(
        json!({"inventory":"LICENSE-INVENTORY.json","rust_crates":rows.len(),"rust_notice_files":rows.iter().map(|r|r["notices"].as_array().unwrap().len()).sum::<usize>(),"redistribution_audit_complete":false}),
    )
}
pub fn acceptance(args: &Args) -> Result<()> {
    args.check(&["--package", "--output"])?;
    let package = args.required("--package")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let inventory = crate::common::json(&package.join("LICENSE-INVENTORY.json"))?;
    let files = crate::common::json(&package.join("FILES.json"))?;
    let verify = |row: &Value| -> Result<()> {
        let relative = Path::new(row["path"].as_str().ok_or("notice path missing")?);
        ensure(archive::safe(relative), "notice escaped package")?;
        let target = package.join(relative);
        ensure(
            target.is_file() && !target.is_symlink(),
            "notice missing/linked",
        )?;
        let bytes = read(&target, 4 * 1024 * 1024)?;
        ensure(
            json!(bytes.len()) == row["bytes"]
                && sha(&bytes) == row["sha256"]
                && files[row["path"].as_str().unwrap()]["sha256"] == row["sha256"],
            "notice bytes/hash/FILES binding differs",
        )
    };
    let crates = inventory["rust_crates"]
        .as_array()
        .ok_or("crates missing")?;
    ensure(!crates.is_empty(), "no real crate notices")?;
    for row in crates {
        let notices = row["notices"].as_array().ok_or("crate notices missing")?;
        ensure(!notices.is_empty(), "missing real crate notices")?;
        for row in notices {
            verify(row)?;
        }
    }
    for row in inventory["rust_standard_library_notices"]
        .as_array()
        .ok_or("Rust standard notices missing")?
    {
        verify(row)?;
    }
    let supplements = inventory["native_source_supplements"]
        .as_array()
        .ok_or("native supplements missing")?;
    for row in supplements.iter().chain(
        inventory
            .get("glib_build_source_supplement")
            .filter(|v| !v.is_null()),
    ) {
        for notice in row["notices"].as_array().ok_or("native notices missing")? {
            verify(notice)?;
        }
    }
    ensure(
        [
            "cpython_source_supplement",
            "helper_wheel_provenance",
            "python_wheel_notice_paths",
        ]
        .iter()
        .all(|k| inventory.get(k).is_none())
            && files
                .as_object()
                .ok_or("FILES missing")?
                .keys()
                .all(|k| !k.contains("cpython-source")),
        "Python runtime notices still shipped",
    )?;
    let mut checks = 4;
    if !supplements.is_empty() {
        let paths: BTreeSet<_> = supplements
            .iter()
            .flat_map(|r| r["notices"].as_array().unwrap())
            .map(|v| v["path"].as_str().unwrap())
            .collect();
        for (name, required) in [
            ("glib", vec!["LICENSES/LGPL-2.1-or-later.txt"]),
            (
                "dbus",
                vec!["LICENSES/AFL-2.1.txt", "LICENSES/GPL-2.0-or-later.txt"],
            ),
            ("gettext", vec!["gettext-runtime/intl/COPYING.LIB"]),
            ("pcre2", vec!["COPYING"]),
        ] {
            let version = supplements
                .iter()
                .find(|r| r["source"]["name"] == name)
                .and_then(|r| r["source"]["version"].as_str())
                .ok_or("runtime source supplement missing")?;
            for path in required {
                ensure(
                    paths.contains(
                        format!(
                            "libexec/inkscape-mcp/licenses/native-source/{name}/{version}/{path}"
                        )
                        .as_str(),
                    ),
                    "runtime license text missing",
                )?;
            }
        }
        checks += 1;
    }
    if inventory["target"]
        .as_str()
        .is_some_and(|s| s.starts_with("macos"))
    {
        let root = tempfile::tempdir()?;
        let vendor = root.path().join("vendor");
        crate::acceptance::copy_tree(Path::new("migration/vendor-notices/native"), &vendor)?;
        let fixtures = root.path().join("valid");
        crate::acceptance::copy_tree(
            &package.join("libexec/inkscape-mcp/licenses/dbus"),
            &fixtures.join("libexec/inkscape-mcp/licenses/dbus"),
        )?;
        let mut gaps = vec![];
        let rows = native_sources_from(&fixtures, &mut gaps, &vendor)?;
        ensure(
            rows.iter()
                .map(|r| r["source"]["name"].as_str().unwrap())
                .collect::<BTreeSet<_>>()
                == BTreeSet::from(["glib", "dbus", "gettext", "pcre2"])
                && gaps.is_empty(),
            "exact native source supplements missing",
        )?;
        checks += 1;
        let glib = rows.iter().find(|r| r["source"]["name"] == "glib").unwrap();
        let path = vendor
            .join("glib")
            .join(glib["source"]["version"].as_str().unwrap())
            .join("provenance.json");
        let saved = read(&path, 4 * 1024 * 1024)?;
        for name in [
            "archive_checksum",
            "upstream_url",
            "notice_checksum",
            "traversal",
        ] {
            let mut provenance: Value = serde_json::from_slice(&saved)?;
            match name {
                "archive_checksum" => provenance["sha256"] = json!("0".repeat(64)),
                "upstream_url" => provenance["url"] = json!("https://invalid.example/source"),
                "notice_checksum" => provenance["notices"][0]["sha256"] = json!("0".repeat(64)),
                _ => provenance["notices"][0]["path"] = json!("../../outside"),
            };
            write_json(&path, &provenance)?;
            let fixture = root.path().join(name);
            crate::acceptance::copy_tree(
                &package.join("libexec/inkscape-mcp/licenses/dbus"),
                &fixture.join("libexec/inkscape-mcp/licenses/dbus"),
            )?;
            let mut gaps = vec![];
            let result = native_sources_from(&fixture, &mut gaps, &vendor);
            fs::write(&path, &saved)?;
            ensure(
                result.is_err(),
                format!("native notice accepted corrupt {name}"),
            )?;
            checks += 1;
        }
    }
    ensure(
        inventory["redistribution_audit_complete"] == false
            && inventory["remaining_gaps"]
                .as_array()
                .is_some_and(|v| !v.is_empty()),
        "audit gaps not explicit",
    )?;
    checks += 1;
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"checks":checks,"rust_crates":crates.len(),"scope":"Real crate/native notice hashes, FILES bindings and provenance refusals; archive guard regressions run separately in Cargo; not legal clearance"}),
    )?;
    println!(
        "Notices: {checks} package checks, {} crates passed",
        crates.len()
    );
    Ok(())
}
