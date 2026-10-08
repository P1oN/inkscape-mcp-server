use crate::{archive, common::*, notices};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn dependencies(path: &Path) -> Result<Vec<String>> {
    Ok(output(Command::new("/usr/bin/otool").arg("-L").arg(path))?
        .lines()
        .skip(1)
        .map(|s| {
            s.trim()
                .split(" (compatibility")
                .next()
                .unwrap()
                .to_string()
        })
        .collect())
}
fn system(dep: &str) -> bool {
    dep.starts_with("/usr/lib/") || dep.starts_with("/System/Library/")
}
fn native_input(path: &str) -> Result<PathBuf> {
    native_input_at(
        path,
        std::env::var_os("INKSCAPE_MCP_BUILD_NATIVE_ROOT")
            .as_deref()
            .map(Path::new),
    )
}
fn native_input_at(path: &str, private_root: Option<&Path>) -> Result<PathBuf> {
    let path = path
        .replace("@@HOMEBREW_CELLAR@@", "/opt/homebrew/Cellar")
        .replace("@@HOMEBREW_PREFIX@@", "/opt/homebrew");
    let Some(root) = private_root else {
        return Ok(Path::new(&path).canonicalize()?);
    };
    let root = root.canonicalize()?;
    let candidate = if let Some(relative) = path.strip_prefix("/opt/homebrew/Cellar/") {
        root.join(relative)
    } else if let Some(relative) = path.strip_prefix("/opt/homebrew/opt/") {
        let relative = Path::new(relative);
        let name = relative.components().next().ok_or("empty formula path")?;
        let versions: Vec<_> = fs::read_dir(root.join(name))?
            .filter_map(|r| r.ok())
            .map(|r| r.path())
            .filter(|p| p.is_dir())
            .collect();
        ensure(versions.len() == 1, "ambiguous native formula version")?;
        versions[0].join(relative.strip_prefix(name)?)
    } else {
        return Ok(Path::new(&path).canonicalize()?);
    };
    let candidate = candidate.canonicalize()?;
    ensure(
        candidate.starts_with(root),
        "native input escaped private tree",
    )?;
    Ok(candidate)
}
fn resolve(source: &Path, dep: &str) -> Result<PathBuf> {
    resolve_with(source, dep, || {
        output(Command::new("/usr/bin/otool").arg("-l").arg(source))
    })
}
fn resolve_with(
    source: &Path,
    dep: &str,
    commands: impl FnOnce() -> Result<String>,
) -> Result<PathBuf> {
    if dep.starts_with('/') || dep.starts_with("@@HOMEBREW_") {
        return native_input(dep);
    }
    if let Some(relative) = dep.strip_prefix("@loader_path/") {
        return Ok(source.parent().unwrap().join(relative).canonicalize()?);
    }
    if let Some(relative) = dep.strip_prefix("@rpath/") {
        let commands = commands()?;
        let regex = regex::Regex::new(r"cmd LC_RPATH\s+cmdsize \d+\s+path (.*?) \(offset \d+\)")?;
        let mut candidates = BTreeSet::new();
        for row in regex.captures_iter(&commands) {
            let rpath = &row[1];
            let base = if let Some(relative) = rpath.strip_prefix("@loader_path/") {
                source.parent().unwrap().join(relative)
            } else if rpath.starts_with('/') || rpath.starts_with("@@HOMEBREW_") {
                native_input(rpath)?
            } else {
                continue;
            };
            let candidate = base.join(relative);
            if candidate.is_file() {
                candidates.insert(candidate.canonicalize()?);
            }
        }
        if candidates.len() == 1 {
            return Ok(candidates.into_iter().next().unwrap());
        }
    }
    Err(format!("unresolved/ambiguous development D-Bus dependency: {dep}").into())
}
fn bundle_dbus(library: &Path, manifest: &mut Value) -> Result<()> {
    let root = library.join("dbus");
    let mut mapping = BTreeMap::new();
    let mut queue = VecDeque::new();
    for name in ["dbus-daemon", "gdbus"] {
        let source = tool(name)?.canonicalize()?;
        let dest = root.join("bin").join(name);
        copy(&source, &dest)?;
        mapping.insert(source.clone(), dest);
        queue.push_back(source);
    }
    let mut graph = BTreeMap::new();
    while let Some(source) = queue.pop_front() {
        let deps = dependencies(&source)?;
        let identity = if source.extension().is_some_and(|e| e == "dylib") {
            output(Command::new("/usr/bin/otool").arg("-D").arg(&source))?
        } else {
            String::new()
        };
        let mut edges = vec![];
        for dep in deps {
            if system(&dep) || identity.lines().skip(1).any(|line| line == dep) {
                continue;
            }
            let target = resolve(&source, &dep)?;
            edges.push((dep, target.clone()));
            if target == source || mapping.contains_key(&target) {
                continue;
            }
            ensure(
                mapping.len() < 64
                    && target.is_file()
                    && fs::metadata(&target)?.len() <= 64 * 1024 * 1024,
                "invalid D-Bus dependency graph",
            )?;
            let dest = root
                .join("lib")
                .join(target.file_name().ok_or("dependency filename missing")?);
            ensure(
                !mapping.values().any(|d| d == &dest),
                "colliding D-Bus library names",
            )?;
            copy(&target, &dest)?;
            mapping.insert(target.clone(), dest);
            queue.push_back(target);
        }
        graph.insert(source, edges);
    }
    for (source, dest) in &mapping {
        if dest.extension().is_some_and(|e| e == "dylib") {
            output(
                Command::new("/usr/bin/install_name_tool")
                    .args([
                        "-id",
                        &format!(
                            "@loader_path/{}",
                            dest.file_name().unwrap().to_string_lossy()
                        ),
                    ])
                    .arg(dest),
            )?;
        }
        for (dep, target) in &graph[source] {
            let relative = relative(&mapping[target], dest.parent().unwrap())?;
            output(
                Command::new("/usr/bin/install_name_tool")
                    .args([
                        "-change",
                        dep,
                        &format!("@loader_path/{}", relative.display()),
                    ])
                    .arg(dest),
            )?;
        }
        output(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-"])
                .arg(dest),
        )?;
        let deps = dependencies(dest)?;
        ensure(
            deps.iter()
                .all(|dep| system(dep) || dep.starts_with("@loader_path/")),
            "D-Bus relocation left external dependency",
        )?;
        manifest["dbus_inputs"].as_array_mut().unwrap().push(json!({"source":source,"source_sha256":hash(source)?,"output":dest.strip_prefix(library)?,"dependencies":deps}));
    }
    fs::create_dir_all(root.join("lib/gio/modules"))?;
    copy(
        Path::new("rust/package/session.conf"),
        &root.join("session.conf"),
    )?;
    for source in mapping.keys() {
        let formula = source.ancestors().skip(1).find(|p| {
            p.join("INSTALL_RECEIPT.json").is_file()
                || (p.join("sbom.spdx.json").is_file()
                    && p.join(".brew")
                        .join(format!(
                            "{}.rb",
                            p.parent().unwrap().file_name().unwrap().to_string_lossy()
                        ))
                        .is_file())
        });
        if let Some(formula) = formula {
            let name = formula.parent().unwrap().file_name().unwrap();
            let licenses = library.join("licenses/dbus").join(name);
            for name in [
                "COPYING",
                "COPYING.LIB",
                "LICENSE",
                "LICENCE.md",
                "AUTHORS",
                "sbom.spdx.json",
                "LGPL-2.1-or-later.txt",
                "GPL-2.0-or-later.txt",
                "AFL-2.1.txt",
            ] {
                if formula.join(name).is_file() {
                    copy(&formula.join(name), &licenses.join(name))?;
                }
            }
            for name in [
                "INSTALL_RECEIPT.json".into(),
                format!(".brew/{}.rb", name.to_string_lossy()),
            ] {
                if formula.join(&name).is_file() {
                    copy(
                        &formula.join(&name),
                        &licenses.join("build-metadata").join(&name),
                    )?;
                }
            }
        }
    }
    Ok(())
}
const ELF_SYSTEM: [&str; 10] = [
    "libc.so.6",
    "libm.so.6",
    "libdl.so.2",
    "libpthread.so.0",
    "librt.so.1",
    "libresolv.so.2",
    "libutil.so.1",
    "ld-linux-x86-64.so.2",
    "ld-linux-aarch64.so.1",
    "linux-vdso.so.1",
];
fn bundle_elf(output_dir: &Path, library: &Path, manifest: &mut Value) -> Result<()> {
    bundle_elf_with(output_dir, library, manifest, &mut output, &tool)
}
fn bundle_elf_with(
    output_dir: &Path,
    library: &Path,
    manifest: &mut Value,
    execute: &mut impl FnMut(&mut Command) -> Result<String>,
    lookup: &impl Fn(&str) -> Result<PathBuf>,
) -> Result<()> {
    let native = library.join("native-lib");
    fs::create_dir(&native)?;
    for name in ["dbus-daemon", "gdbus"] {
        copy(&lookup(name)?, &library.join("dbus/bin").join(name))?;
    }
    fs::write(
        library.join("dbus/session.conf"),
        include_bytes!("../../package/session.conf"),
    )?;
    fs::create_dir_all(library.join("dbus/lib/gio/modules"))?;
    let mut queue = VecDeque::new();
    for path in walk(output_dir)? {
        use std::io::Read;
        let mut magic = [0; 4];
        if fs::File::open(&path)?.read(&mut magic)? == 4 && magic == *b"\x7fELF" {
            queue.push_back(path);
        }
    }
    let mut inputs: BTreeMap<String, Value> = BTreeMap::new();
    let mut processed = BTreeSet::new();
    let regex = regex::Regex::new(r"^\s*(\S+)\s+=>\s+(\S+)\s+\(")?;
    while let Some(dest) = queue.pop_front() {
        if !processed.insert(dest.clone()) {
            continue;
        }
        ensure(processed.len() <= 512, "ELF graph exceeds 512 files")?;
        let needed = execute(Command::new("patchelf").arg("--print-needed").arg(&dest))?;
        let text = execute(Command::new("ldd").arg(&dest))?;
        let resolved: BTreeMap<_, _> = text
            .lines()
            .filter_map(|line| regex.captures(line))
            .map(|c| (c[1].to_string(), PathBuf::from(&c[2])))
            .collect();
        for name in needed.lines() {
            if ELF_SYSTEM.contains(&name) {
                continue;
            }
            ensure(
                !name.contains('/') && resolved.contains_key(name),
                format!("unresolved ELF dependency: {name}"),
            )?;
            let source = resolved[name].canonicalize()?;
            if source.starts_with(output_dir) {
                continue;
            }
            ensure(
                source.is_file() && fs::metadata(&source)?.len() <= 64 * 1024 * 1024,
                "invalid ELF dependency",
            )?;
            let checksum = hash(&source)?;
            if let Some(row) = inputs.get(name) {
                ensure(
                    row["source_sha256"] == checksum,
                    format!("colliding ELF dependency: {name}"),
                )?;
            } else {
                ensure(inputs.len() < 64, "ELF external graph cap exceeded")?;
                let target = native.join(name);
                copy(&source, &target)?;
                queue.push_back(target.clone());
                inputs.insert(name.into(),json!({"source":source,"source_sha256":checksum,"output":target.strip_prefix(library)?}));
                let owners = execute(Command::new("dpkg-query").arg("-S").arg(&source))?;
                for row in owners.lines() {
                    let package = row.split(": ").next().unwrap().split(':').next().unwrap();
                    let notice = Path::new("/usr/share/doc").join(package).join("copyright");
                    if notice.is_file() {
                        copy(
                            &notice,
                            &library
                                .join("licenses/linux")
                                .join(package)
                                .join("copyright"),
                        )?;
                    }
                }
            }
        }
        let old = execute(Command::new("patchelf").arg("--print-rpath").arg(&dest))?;
        let mut paths = vec![format!(
            "$ORIGIN/{}",
            relative(&native, dest.parent().unwrap())?.display()
        )];
        for path in old.trim().split(':') {
            if path == "$ORIGIN" || path.starts_with("$ORIGIN/") {
                let relative = path
                    .strip_prefix("$ORIGIN")
                    .unwrap()
                    .trim_start_matches('/');
                if dest
                    .parent()
                    .unwrap()
                    .join(relative)
                    .canonicalize()
                    .is_ok_and(|p| p.starts_with(output_dir))
                    && !paths.iter().any(|p| p == path)
                {
                    paths.push(path.into());
                }
            }
        }
        execute(
            Command::new("patchelf")
                .args(["--set-rpath", &paths.join(":")])
                .arg(&dest),
        )?;
    }
    for dest in processed {
        let text = execute(Command::new("ldd").arg(dest))?;
        ensure(
            !text.contains("not found"),
            "relocated ELF dependency missing",
        )?;
        for row in text.lines().filter_map(|line| regex.captures(line)) {
            if !ELF_SYSTEM.contains(&&row[1]) {
                ensure(
                    Path::new(&row[2]).canonicalize()?.starts_with(output_dir),
                    "relocated ELF dependency escaped package",
                )?;
            }
        }
    }
    manifest["elf_inputs"] = json!(inputs.into_values().collect::<Vec<_>>());
    manifest["system_elf_dependencies"] = json!(ELF_SYSTEM);
    manifest["glibc_build_version"] =
        json!(execute(Command::new("getconf").arg("GNU_LIBC_VERSION"))?.trim());
    Ok(())
}
fn source_revision() -> Result<Value> {
    source_revision_at(Path::new("."))
}
fn source_revision_at(root: &Path) -> Result<Value> {
    if root.join(".git").exists() {
        return Ok(json!(
            output(
                Command::new(tool("git")?)
                    .current_dir(root)
                    .args(["rev-parse", "HEAD"])
            )?
            .trim()
        ));
    }
    let path = root.join("SOURCE_REVISION");
    if !path.exists() && !path.is_symlink() {
        return Ok(Value::Null);
    }
    ensure(
        !path.is_symlink() && path.is_file(),
        "invalid source revision metadata",
    )?;
    let text = String::from_utf8(read(&path, 128)?)?;
    let lines: Vec<_> = text.lines().collect();
    ensure(
        lines.len() == 2
            && lines[0] == "inkscape-mcp-source-v1"
            && lines[1].len() == 40
            && lines[1]
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "invalid source revision metadata",
    )?;
    Ok(json!(lines[1]))
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--output", "--archive", "--binary"])?;
    let out = args.required("--output")?;
    let binary = args
        .path("--binary", "rust/target/release/inkscape-mcp-rust")
        .canonicalize()?;
    let archive = args.values.get("--archive").map(PathBuf::from);
    let out = std::path::absolute(out)?;
    if let Some(archive) = &archive {
        ensure(
            !archive.exists()
                && !archive.is_symlink()
                && !std::path::absolute(archive)?.starts_with(&out),
            "archive must be new and outside package",
        )?;
    }
    build(&out, &binary)?;
    if let Some(archive) = archive {
        archive::pack(&out, &archive)?;
    }
    Ok(())
}
fn native_target(os: &str, arch: &str) -> Result<(String, String)> {
    ensure(
        ["macos", "linux"].contains(&os) && ["aarch64", "x86_64"].contains(&arch),
        "native builder requires macOS/Linux arm64/x86_64",
    )?;
    let target = format!("{os}-{}", if arch == "aarch64" { "arm64" } else { arch });
    let triple = format!(
        "{arch}-{}",
        if os == "macos" {
            "apple-darwin"
        } else {
            "unknown-linux-gnu"
        }
    );
    Ok((target, triple))
}
fn build(out: &Path, binary: &Path) -> Result<()> {
    let os = std::env::consts::OS;
    let (target, triple) = native_target(os, std::env::consts::ARCH)?;
    if os == "linux" {
        ensure(
            command("getconf", &["GNU_LIBC_VERSION"])?.starts_with("glibc "),
            "Linux packages require GNU/glibc",
        )?;
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(out)?;
    let out = out.canonicalize()?;
    let library = out.join("libexec/inkscape-mcp");
    fs::create_dir_all(&library)?;
    let mut manifest = json!({"target":target,"runtime":"native-rust","dbus_inputs":[],"release_signed":false,"notarized":false,"source_head":source_revision()?,"build_info":serde_json::from_str::<Value>(&output(Command::new(binary).arg("--version"))?)?});
    manifest["update_contract"] = json!({"text_interface":inkscape_mcp_rust::update::manifests::TEXT_INTERFACE,"helper_protocol":inkscape_mcp_rust::update::manifests::HELPER_PROTOCOL,"launcher_minimum":1});
    for name in [
        "inkscape_mcp_insert.inx",
        "inkscape_mcp_edit.inx",
        "inkscape_mcp_live.inx",
    ] {
        copy(
            &Path::new("runtime/helper_extension").join(name),
            &library.join("helpers").join(name),
        )?;
    }
    copy(binary, &out.join("bin/inkscape-mcp"))?;
    for (name, key) in [
        ("inkscape-mcp-launcher", "permanent_launcher"),
        ("inkscape-mcp-client", "client_manager"),
        ("inkscape-mcp-supervisor", "managed_supervisor"),
        ("inkscape-mcp-inx", "one_shot_helper"),
        ("inkscape-mcp-live", "socket_helper"),
    ] {
        let source = binary.parent().unwrap().join(name);
        ensure(
            source.is_file(),
            format!("native binary missing: {name}; build all six binaries"),
        )?;
        copy(&source, &out.join("bin").join(name))?;
        manifest[key] = json!(format!("bin/{name}"));
    }
    if os == "macos" {
        let symbols = PathBuf::from(format!("{}.dSYM", binary.display())).canonicalize()?;
        ensure(symbols.is_dir(), "Rust release dSYM missing")?;
        for file in walk(&symbols)? {
            copy(
                &file,
                &out.join("bin/inkscape-mcp.dSYM")
                    .join(file.strip_prefix(&symbols)?),
            )?;
        }
        manifest["server_debug_symbols"] = json!("bin/inkscape-mcp.dSYM");
    }
    for name in [
        "setup.sh",
        "run-mcp.sh",
        "uninstall.sh",
        "LICENSE",
        "scripts/install-skill.sh",
        "scripts/mcp-client.sh",
        "skills/inkscape-mcp/SKILL.md",
        "skills/inkscape-mcp/agents/openai.yaml",
    ] {
        copy(Path::new(name), &out.join(name))?;
    }
    if os == "macos" {
        let prefixes = [
            std::env::var_os("INKSCAPE_MCP_BUILD_GLIB_PREFIX")
                .map(PathBuf::from)
                .unwrap_or("/nonexistent".into()),
            "/opt/homebrew".into(),
            "/usr/local".into(),
        ];
        let headers = prefixes
            .iter()
            .find(|p| p.join("include/glib-2.0/gio/gio.h").is_file())
            .ok_or("development GLib headers required")?;
        let source = Path::new("runtime/native/context.m");
        let dest = library.join("context.so");
        let argv = vec![
            "/usr/bin/clang".into(),
            "-Wall".into(),
            "-Wextra".into(),
            "-Werror".into(),
            "-dynamiclib".into(),
            "-framework".into(),
            "Cocoa".into(),
            "-undefined".into(),
            "dynamic_lookup".into(),
            "-Wl,-install_name,@rpath/inkscape-mcp-context.so".into(),
            format!("-I{}", headers.join("include/glib-2.0").display()),
            format!("-I{}", headers.join("lib/glib-2.0/include").display()),
            source.display().to_string(),
            "-o".into(),
            dest.display().to_string(),
        ];
        output(Command::new(&argv[0]).args(&argv[1..]))?;
        manifest["context_bridge_build"] = json!({"source_sha256":hash(source)?,"compiler_command":argv,"install_name":"@rpath/inkscape-mcp-context.so","sha256":hash(&dest)?});
        bundle_dbus(&library, &mut manifest)?;
    } else {
        bundle_elf(&out, &library, &mut manifest)?;
    }
    if let Some(root) = std::env::var_os("INKSCAPE_MCP_BUILD_NATIVE_ROOT") {
        copy(
            &Path::new(&root).join("inputs.json"),
            &library.join("licenses/bootstrap/native-inputs.json"),
        )?;
    }
    if os == "macos" {
        let app = out.join("management/Inkscape MCP Manager.app/Contents");
        fs::create_dir_all(app.join("MacOS"))?;
        output(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-", "--timestamp=none"])
                .arg(out.join("bin/inkscape-mcp-launcher")),
        )?;
        copy(
            &out.join("bin/inkscape-mcp-launcher"),
            &app.join("Helpers/inkscape-mcp-launcher"),
        )?;
        let instruction_files = inkscape_mcp_rust::update::instructions::default_files();
        let instruction_manifest = inkscape_mcp_rust::update::instructions::manifest(
            manifest["build_info"]["build_id"]
                .as_str()
                .ok_or("missing build ID")?,
            &instruction_files,
        );
        let instruction_root = app.join("Resources/instructions");
        for (name, bytes) in instruction_files {
            let path = instruction_root.join(name);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, bytes)?;
        }
        write_json(
            &instruction_root.join("manifest.json"),
            &serde_json::to_value(instruction_manifest)?,
        )?;
        output(
            Command::new("/usr/bin/clang")
                .args([
                    "-fobjc-arc",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-Wno-unused-parameter",
                    "runtime/manager/main.m",
                    "-framework",
                    "Cocoa",
                    "-o",
                ])
                .arg(app.join("MacOS/inkscape-mcp-manager")),
        )?;
        fs::write(
            app.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>inkscape-mcp-manager</string><key>CFBundleIdentifier</key><string>org.inkscape-mcp.manager</string><key>CFBundleName</key><string>Inkscape MCP Manager</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>1.0</string><key>LSMinimumSystemVersion</key><string>15.0</string><key>NSHighResolutionCapable</key><true/></dict></plist>"#,
        )?;
        let info = fs::read_to_string(app.join("Info.plist"))?;
        let build = hash(&app.join("MacOS/inkscape-mcp-manager"))?;
        fs::write(
            app.join("Info.plist"),
            info.replace(
                "</dict></plist>",
                &format!(
                    "<key>InkscapeMCPBuildID</key><string>{}</string></dict></plist>",
                    &build[..16]
                ),
            ),
        )?;
        output(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-", "--timestamp=none"])
                .arg(app.parent().unwrap()),
        )?;
        manifest["management_app"] = json!("management/Inkscape MCP Manager.app");
    }
    manifest["license_inventory"] = notices::collect(&out, &target, &triple)?;
    write_json(&library.join("package.json"), &manifest)?;
    let mut files = serde_json::Map::new();
    let mut bytes = 0;
    for path in walk(&out)? {
        let size = fs::metadata(&path)?.len();
        bytes += size;
        files.insert(
            path.strip_prefix(&out)?.to_string_lossy().into_owned(),
            json!({"sha256":hash(&path)?,"bytes":size}),
        );
    }
    let count = files.len();
    write_json(&out.join("FILES.json"), &Value::Object(files))?;
    println!("{}", json!({"package":out,"files":count,"bytes":bytes}));
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elf_closure_deduplicates_refuses_missing_and_collisions_preserving_sources() {
        for scenario in ["valid", "missing", "collision"] {
            let root = tempfile::tempdir().unwrap();
            let root = root.path().canonicalize().unwrap();
            let output_dir = root.join("package");
            let library = output_dir.join("libexec/inkscape-mcp");
            fs::create_dir_all(&library).unwrap();
            fs::create_dir(output_dir.join("bin")).unwrap();
            fs::write(output_dir.join("bin/inkscape-mcp"), b"\x7fELFserver").unwrap();
            let sources = root.join("source");
            fs::create_dir(&sources).unwrap();
            for name in ["dbus-daemon", "gdbus", "libfoo.so.1", "collision.so"] {
                fs::write(sources.join(name), format!("\x7fELF{name}")).unwrap();
            }
            let before: Vec<_> = walk(&sources)
                .unwrap()
                .iter()
                .map(|p| hash(p).unwrap())
                .collect();
            let mut rpaths = BTreeMap::new();
            let mut execute = |command: &mut Command| -> Result<String> {
                let program = command.get_program().to_string_lossy();
                let args: Vec<_> = command
                    .get_args()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect();
                match program.as_ref() {
                    "getconf" => Ok("glibc 2.39\n".into()),
                    "dpkg-query" => Ok("migration-fixture: source\n".into()),
                    "patchelf" => {
                        let dest = PathBuf::from(args.last().unwrap());
                        match args[0].as_str() {
                            "--print-needed" => Ok(if dest.parent().unwrap().file_name().unwrap()
                                == "native-lib"
                            {
                                "libc.so.6\n"
                            } else {
                                "libfoo.so.1\n"
                            }
                            .into()),
                            "--print-rpath" => Ok(
                                "$ORIGIN:$ORIGIN/../../../../../../../../outside:/build/external\n"
                                    .into(),
                            ),
                            "--set-rpath" => {
                                rpaths.insert(dest, args[1].clone());
                                Ok(String::new())
                            }
                            _ => Err("unexpected fixture patchelf command".into()),
                        }
                    }
                    "ldd" => {
                        let dest = PathBuf::from(&args[0]);
                        if dest.parent().unwrap().file_name().unwrap() == "native-lib" {
                            return Ok("libc.so.6 => /lib/libc.so.6 (0x1)\n".into());
                        }
                        if scenario == "missing" {
                            return Ok("libfoo.so.1 => not found\n".into());
                        }
                        let dep = if scenario == "collision" && dest.file_name().unwrap() == "gdbus"
                        {
                            sources.join("collision.so")
                        } else if rpaths.contains_key(&dest) {
                            library.join("native-lib/libfoo.so.1")
                        } else {
                            sources.join("libfoo.so.1")
                        };
                        Ok(format!(
                            "libfoo.so.1 => {} (0x2)\nlibc.so.6 => /lib/libc.so.6 (0x1)\n",
                            dep.display()
                        ))
                    }
                    _ => Err(format!("unexpected fixture command: {program}").into()),
                }
            };
            let mut manifest = json!({});
            let result = bundle_elf_with(
                &output_dir,
                &library,
                &mut manifest,
                &mut execute,
                &|name| Ok(sources.join(name)),
            );
            assert_eq!(
                before,
                walk(&sources)
                    .unwrap()
                    .iter()
                    .map(|p| hash(p).unwrap())
                    .collect::<Vec<_>>()
            );
            if scenario == "valid" {
                assert!(result.is_ok(), "{result:?}");
                assert_eq!(manifest["elf_inputs"].as_array().unwrap().len(), 1);
                assert_eq!(rpaths.len(), 4);
                for (path, rpath) in rpaths {
                    assert!(!rpath.contains("outside") && !rpath.contains("/build"));
                    for part in rpath.split(':') {
                        assert!(
                            path.parent()
                                .unwrap()
                                .join(
                                    part.strip_prefix("$ORIGIN")
                                        .unwrap()
                                        .trim_start_matches('/')
                                )
                                .canonicalize()
                                .unwrap()
                                .starts_with(&output_dir)
                        );
                    }
                }
            } else {
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains(if scenario == "missing" {
                        "unresolved ELF dependency"
                    } else {
                        "colliding ELF dependency"
                    }),
                    "{error}"
                );
            }
        }
    }
}
#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn targets_source_identity_and_bottle_paths_refuse_unsupported_ambiguous_or_escaped_inputs() {
        for (os, arch, target) in [
            ("macos", "aarch64", "macos-arm64"),
            ("macos", "x86_64", "macos-x86_64"),
            ("linux", "aarch64", "linux-arm64"),
            ("linux", "x86_64", "linux-x86_64"),
        ] {
            assert_eq!(native_target(os, arch).unwrap().0, target);
        }
        assert!(native_target("windows", "x86_64").is_err());
        assert!(native_target("linux", "riscv64").is_err());
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("SOURCE_REVISION");
        assert_eq!(source_revision_at(root.path()).unwrap(), Value::Null);
        fs::write(
            &marker,
            format!("inkscape-mcp-source-v1\n{}\n", "a".repeat(40)),
        )
        .unwrap();
        assert_eq!(source_revision_at(root.path()).unwrap(), "a".repeat(40));
        fs::write(
            &marker,
            "inkscape-mcp-source-v1\n$(touch SHOULD_NOT_EXIST)\n",
        )
        .unwrap();
        assert!(source_revision_at(root.path()).is_err());
        assert!(!root.path().join("SHOULD_NOT_EXIST").exists());
        fs::remove_file(&marker).unwrap();
        std::os::unix::fs::symlink(root.path().join("absent"), &marker).unwrap();
        assert!(
            source_revision_at(root.path()).is_err(),
            "dangling revision link accepted"
        );
        let inputs = root.path().join("native");
        let lib = inputs.join("glib/1.0/lib/libglib.dylib");
        fs::create_dir_all(lib.parent().unwrap()).unwrap();
        fs::write(&lib, "fixture").unwrap();
        for name in [
            "/opt/homebrew/Cellar/glib/1.0/lib/libglib.dylib",
            "@@HOMEBREW_CELLAR@@/glib/1.0/lib/libglib.dylib",
            "@@HOMEBREW_PREFIX@@/opt/glib/lib/libglib.dylib",
        ] {
            assert_eq!(
                native_input_at(name, Some(&inputs)).unwrap(),
                lib.canonicalize().unwrap()
            );
        }
        assert!(native_input_at("/opt/homebrew/Cellar/../../escape", Some(&inputs)).is_err());
        fs::create_dir_all(inputs.join("glib/2.0")).unwrap();
        assert!(
            native_input_at("/opt/homebrew/opt/glib/lib/libglib.dylib", Some(&inputs)).is_err()
        );
    }
    #[test]
    fn macho_loader_and_rpaths_require_unique_existing_library() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let source = root.join("bin/gdbus");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "fixture").unwrap();
        let lib = root.join("lib/libdbus.dylib");
        fs::create_dir(lib.parent().unwrap()).unwrap();
        fs::write(&lib, "fixture").unwrap();
        let alternate = root.join("other/libdbus.dylib");
        fs::create_dir(alternate.parent().unwrap()).unwrap();
        fs::write(&alternate, "other").unwrap();
        for rpath in [
            lib.parent().unwrap().display().to_string(),
            "@loader_path/../lib".into(),
        ] {
            let commands = format!("cmd LC_RPATH\ncmdsize 48\npath {rpath} (offset 12)\n");
            assert_eq!(
                resolve_with(&source, "@rpath/libdbus.dylib", || Ok(commands)).unwrap(),
                lib
            );
        }
        assert_eq!(
            resolve_with(&source, "@loader_path/../lib/libdbus.dylib", || panic!(
                "loader needs no otool"
            ))
            .unwrap(),
            lib
        );
        for commands in [
            String::new(),
            format!(
                "cmd LC_RPATH\ncmdsize 48\npath {} (offset 12)\ncmd LC_RPATH\ncmdsize 48\npath {} (offset 12)\n",
                lib.parent().unwrap().display(),
                alternate.parent().unwrap().display()
            ),
        ] {
            assert!(resolve_with(&source, "@rpath/libdbus.dylib", || Ok(commands)).is_err());
        }
    }
}
