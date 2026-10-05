use crate::{
    common::*,
    wire::{Wire, data},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    process::Command,
};
pub fn verify_files(package: &Path) -> Result<usize> {
    let inventory = crate::common::json(&package.join("FILES.json"))?;
    let rows = inventory.as_object().ok_or("invalid FILES inventory")?;
    for (relative, expected) in rows {
        let path = Path::new(relative);
        ensure(crate::archive::safe(path), "inventory escaped package")?;
        let path = package.join(path);
        ensure(
            path.is_file() && !path.is_symlink(),
            "packaged file missing/linked",
        )?;
        let bytes = read(&path, 256 * 1024 * 1024)?;
        ensure(
            json!(bytes.len()) == expected["bytes"] && sha(&bytes) == expected["sha256"],
            format!("packaged hash/size drift: {relative}"),
        )?;
    }
    let mut pending = vec![package.to_path_buf()];
    while let Some(path) = pending.pop() {
        if path.is_symlink() {
            ensure(
                path.canonicalize()?.starts_with(package.canonicalize()?),
                "package link escapes",
            )?;
        } else if path.is_dir() {
            for row in fs::read_dir(path)? {
                pending.push(row?.path());
            }
        }
    }
    Ok(rows.len())
}
pub fn copy_tree(source: &Path, dest: &Path) -> Result<()> {
    // Ready packages are regular assets. Refuse links/special files before copying,
    // rather than silently omitting them or following them into a different tree.
    let mut pending = vec![source.to_path_buf()];
    let mut entries = Vec::new();
    let mut total = 0u64;
    while let Some(path) = pending.pop() {
        ensure(
            entries.len() + pending.len() < 100_000,
            "package copy member cap",
        )?;
        let metadata = fs::symlink_metadata(&path)?;
        ensure(
            metadata.is_file() || metadata.is_dir(),
            "package copy refuses linked/special assets",
        )?;
        if metadata.is_dir() {
            for row in fs::read_dir(&path)? {
                pending.push(row?.path());
            }
        } else {
            total = total
                .checked_add(metadata.len())
                .ok_or("package copy size overflow")?;
            ensure(
                metadata.len() <= 256 * 1024 * 1024 && total <= 2 * 1024 * 1024 * 1024,
                "package copy size cap",
            )?;
        }
        entries.push((path, metadata.is_dir()));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    ensure(
        !dest.exists() && !dest.is_symlink(),
        "package copy destination must be new",
    )?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(dest)?;
    for (path, dir) in entries {
        if path == source {
            continue;
        }
        let target = dest.join(path.strip_prefix(source)?);
        if dir {
            fs::create_dir(&target)?;
        } else {
            copy(&path, &target)?;
        }
    }
    Ok(())
}
pub fn inventory(root: &Path) -> Result<Value> {
    let mut pending = vec![root.to_path_buf()];
    let mut rows = serde_json::Map::new();
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            for row in fs::read_dir(&path)? {
                pending.push(row?.path());
            }
        }
        rows.insert(path.strip_prefix(root)?.to_string_lossy().to_string(),json!({"mtime_ns":metadata.mtime() as i128*1_000_000_000+metadata.mtime_nsec() as i128,"bytes":if metadata.is_file() {json!(metadata.len())} else {Value::Null},"sha256":if metadata.is_file() {json!(hash(&path)?)} else {Value::Null}}));
    }
    Ok(Value::Object(rows))
}
pub fn environment(root: &Path, library: &Path) -> Result<Vec<(String, String)>> {
    let home = root.join("home");
    fs::create_dir_all(&home)?;
    Ok(vec![
        ("HOME".into(), home.display().to_string()),
        ("PATH".into(), String::new()),
        ("LANG".into(), "en_US.UTF-8".into()),
        ("TMPDIR".into(), root.display().to_string()),
        (
            "INKSCAPE_MCP_WORKSPACE_ROOTS".into(),
            root.join("workspace").display().to_string(),
        ),
        (
            "INKSCAPE_MCP_MANAGED_DIR".into(),
            root.join("not-launched").display().to_string(),
        ),
        (
            "INKSCAPE_MCP_LIVE_RENDEZVOUS".into(),
            root.join("absent-rendezvous").display().to_string(),
        ),
        (
            "GIO_MODULE_DIR".into(),
            library.join("dbus/lib/gio/modules").display().to_string(),
        ),
        ("INKSCAPE_MCP_LIVE_ENABLED".into(), "1".into()),
        ("INKSCAPE_MCP_RAW_ACTION_ENABLED".into(), "1".into()),
        ("INKSCAPE_MCP_TOOL_PROFILE".into(), "full".into()),
        ("INKSCAPE_MCP_TOOL_DESC".into(), "full".into()),
    ])
}
fn command_with_env(binary: &Path, env: &[(String, String)]) -> Command {
    let mut command = Command::new(binary);
    command.env_clear().envs(env.iter().map(|(k, v)| (k, v)));
    command
}
fn count(root: &Path, suffix: &str) -> Result<usize> {
    Ok(walk(root)?
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == suffix))
        .count())
}
fn error(reply: &Value) -> bool {
    reply["result"]["isError"] == true || reply.get("error").is_some()
}
pub fn png_pixel(bytes: &[u8], x: u32, y: u32, expected: [u8; 4]) -> Result<(u32, u32)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let size = reader.output_buffer_size().ok_or("PNG output cap")?;
    ensure(size <= 64 * 1024 * 1024, "PNG output cap")?;
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer)?;
    ensure(
        x < info.width && y < info.height,
        "PNG pixel outside dimensions",
    )?;
    let channels = info.color_type.samples();
    let pos = (y as usize * info.width as usize + x as usize) * channels;
    let rgba = match info.color_type {
        png::ColorType::Rgb => [buffer[pos], buffer[pos + 1], buffer[pos + 2], 255],
        png::ColorType::Rgba => buffer[pos..pos + 4].try_into()?,
        png::ColorType::Grayscale => [buffer[pos], buffer[pos], buffer[pos], 255],
        png::ColorType::GrayscaleAlpha => [buffer[pos], buffer[pos], buffer[pos], buffer[pos + 1]],
        _ => return Err("unsupported PNG colour type".into()),
    };
    ensure(
        rgba == expected,
        format!("real CLI pixel differs: {rgba:?}"),
    )?;
    Ok((info.width, info.height))
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--package", "--archive", "--output", "--engine-mode"])?;
    ensure(
        args.flag("--package") != args.flag("--archive"),
        "choose exactly one of --package/--archive",
    )?;
    let source = args
        .required(if args.flag("--archive") {
            "--archive"
        } else {
            "--package"
        })?
        .canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir_in(Path::new("/tmp").canonicalize()?)?;
    let root = root.path().canonicalize()?;
    let package = root.join("installed");
    if args.flag("--archive") {
        let extracted = root.join("extracted");
        fs::create_dir(&extracted)?;
        let top = crate::archive::package_root(&source)?;
        crate::archive::extract(&source, &extracted, &top)?;
        fs::rename(extracted.join(top), &package)?;
    } else {
        copy_tree(&source, &package)?;
    }
    let files = verify_files(&package)?;
    let library = package.join("libexec/inkscape-mcp");
    ensure(
        !library.join("python").exists()
            && walk(&package)?.iter().all(|p| {
                !p.extension()
                    .is_some_and(|e| ["py", "pyc", "pyo", "whl"].iter().any(|s| e == *s))
            }),
        "Python runtime/source/wheels still bundled",
    )?;
    let manifest = crate::common::json(&library.join("package.json"))?;
    ensure(
        manifest["runtime"] == "native-rust"
            && ["python", "runtime_source", "python_deps"]
                .iter()
                .all(|k| manifest.get(k).is_none()),
        "package declares Python",
    )?;
    let mode = args
        .values
        .get("--engine-mode")
        .map(String::as_str)
        .unwrap_or("per_call");
    ensure(["per_call", "shell"].contains(&mode), "invalid engine mode")?;
    let mut env = environment(&root, &library)?;
    env.push(("INKSCAPE_MCP_ENGINE_MODE".into(), mode.into()));
    native_cli(&package, &root, &env, &out)?;
    let workspace = root.join("workspace");
    fs::create_dir(&workspace)?;
    let original=b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"><rect id=\"r\" width=\"100\" height=\"100\" fill=\"#ff0000\"/></svg>";
    fs::write(workspace.join("fixture.svg"), original)?;
    let mut wire = Wire::spawn(
        &mut command_with_env(&package.join("bin/inkscape-mcp"), &env),
        &out.join("packaged-server.stderr.log"),
    )?;
    let result = (|| -> Result<()> {
        wire.initialize()?;
        let contract = crate::common::json(Path::new(
            "migration/contracts/live-true_raw-true_full_full.json",
        ))?;
        for method in [
            "tools/list",
            "prompts/list",
            "resources/list",
            "resources/templates/list",
        ] {
            ensure(
                wire.request(method, None)?["result"] == contract[method],
                "discovery contract drift",
            )?;
        }
        ensure(
            data(wire.call("list_capabilities", json!({}))?)?["inkscape_available"] == true,
            "minimal PATH did not find Inkscape",
        )?;
        let doc = data(wire.call("open_document", json!({"path":"fixture.svg"}))?)?["doc_id"]
            .as_str()
            .ok_or("doc id missing")?
            .to_string();
        let directory = workspace.join(".inkscape-mcp/documents").join(&doc);
        let working = directory.join("working/document.svg");
        let records = directory.join("operations");
        let before = fs::read(&working)?;
        let timestamp = std::ffi::CString::new(records.as_os_str().as_encoded_bytes())?;
        let times = [libc::timespec {
            tv_sec: 1,
            tv_nsec: 0,
        }; 2];
        ensure(
            unsafe { libc::utimensat(libc::AT_FDCWD, timestamp.as_ptr(), times.as_ptr(), 0) } == 0,
            "cannot set audit directory timestamp",
        )?;
        let noop_before = inventory(&workspace)?;
        let noop = data(wire.call(
            "set_fill",
            json!({"doc_id":doc,"object_ids":["r"],"color":"#ff0000"}),
        )?)?;
        let noop_after = inventory(&workspace)?;
        write_json(
            &out.join("noop-filesystem.json"),
            &json!({"before":noop_before,"after":noop_after}),
        )?;
        ensure(
            noop["changed"] == false && fs::read(&working)? == before && noop_before == noop_after,
            "no-op added audit/transient writes",
        )?;
        ensure(
            data(wire.call(
                "set_fill",
                json!({"doc_id":doc,"object_ids":["r"],"color":"#0000ff"}),
            )?)?["changed"]
                == true,
            "packaged edit failed",
        )?;
        let records_before = count(&records, "json")?;
        let batch=data(wire.call("apply_edits",json!({"doc_id":doc,"edits":[{"op":"set_stroke","object_ids":["r"],"color":"black","width":"1px"},{"op":"move_object","object_id":"r","dx":1,"dy":0}]}))?)?;
        ensure(
            batch["changed"] == true && count(&records, "json")? == records_before + 1,
            "batch audit count drift",
        )?;
        let bytes = fs::read(&working)?;
        ensure(error(&wire.call("apply_edits",json!({"doc_id":doc,"edits":[{"op":"move_object","object_id":"r","dx":50,"dy":0},{"op":"set_opacity","object_ids":["missing"],"opacity":0.2}]}))?)&&fs::read(&working)?==bytes,"invalid batch did not roll back")?;
        ensure(
            error(&wire.call("delete_object", json!({"doc_id":doc,"object_ids":["r"]}))?)
                && fs::read(&working)? == bytes,
            "approval guard failed",
        )?;
        let saved = data(wire.call(
            "save_document_as",
            json!({"doc_id":doc,"dest_path":"saved.svg"}),
        )?)?;
        let resource = wire.request(
            "resources/read",
            Some(json!({"uri":saved["artifact"]["uri"]})),
        )?;
        ensure(
            STANDARD.decode(
                resource["result"]["contents"][0]["blob"]
                    .as_str()
                    .ok_or("resource missing")?,
            )? == bytes,
            "readback drift",
        )?;
        fs::write(workspace.join("batch-members.svg"), original)?;
        let batch_doc =
            data(wire.call("open_document", json!({"path":"batch-members.svg"}))?)?["doc_id"]
                .as_str()
                .ok_or("doc missing")?
                .to_string();
        let batch_directory = workspace.join(".inkscape-mcp/documents").join(&batch_doc);
        let batch_working = batch_directory.join("working/document.svg");
        let repeated = json!({"op":"repeat_objects","object_id":"r","group_id":"repeated","placement":{"kind":"polyline","points":[{"x":0,"y":0},{"x":10,"y":10}],"count":2}});
        let fragment = json!({"op":"replace_svg_fragment","object_id":"r","svg":"<rect xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\" fill=\"blue\"/>","reference_policy":"allow_retained"});
        let mixed = data(wire.call(
            "apply_edits",
            json!({"doc_id":batch_doc,"edits":[repeated,fragment],"approval_token":"test"}),
        )?)?;
        ensure(
            mixed["changed"] == true
                && mixed["risk_class"] == "high"
                && count(&batch_directory.join("snapshots"), "svg")? == 1
                && count(&batch_directory.join("operations"), "json")? == 1,
            "mixed batch snapshot/audit drift",
        )?;
        let committed = fs::read(&batch_working)?;
        let mut repeated = repeated;
        repeated["group_id"] = json!("next");
        let mut fragment = fragment;
        fragment["object_id"] = json!("missing");
        ensure(
            error(&wire.call(
                "apply_edits",
                json!({"doc_id":batch_doc,"edits":[repeated,fragment],"approval_token":"test"}),
            )?) && fs::read(&batch_working)? == committed
                && count(&batch_directory.join("snapshots"), "svg")? == 1,
            "late batch failure did not roll back",
        )?;
        data(wire.call(
            "restore_snapshot",
            json!({"doc_id":batch_doc,"snapshot_id":mixed["snapshot_id"]}),
        )?)?;
        ensure(
            fs::read(&batch_working)? == original
                && fs::read(workspace.join("batch-members.svg"))? == original,
            "restore/original drift",
        )?;
        let preview = data(wire.call(
            "render_preview",
            json!({"doc_id":doc,"width_px":100,"inline":false}),
        )?)?;
        write_json(&out.join("packaged-preview.json"), &preview)?;
        ensure(
            png_pixel(
                &read(
                    &workspace.join(
                        preview["artifact_path"]
                            .as_str()
                            .ok_or("preview path missing")?,
                    ),
                    16 * 1024 * 1024,
                )?,
                50,
                50,
                [0, 0, 255, 255],
            )? == (100, 100),
            "preview dimensions drift",
        )?;
        let exported = data(wire.call(
            "export_document",
            json!({"doc_id":doc,"format":"png","width_px":100}),
        )?)?;
        let resource = wire.request(
            "resources/read",
            Some(json!({"uri":exported["artifact"]["uri"]})),
        )?;
        png_pixel(
            &STANDARD.decode(
                resource["result"]["contents"][0]["blob"]
                    .as_str()
                    .ok_or("export resource missing")?,
            )?,
            50,
            50,
            [0, 0, 255, 255],
        )?;
        ensure(
            fs::read(workspace.join("fixture.svg"))? == original
                && !root.join("not-launched").exists(),
            "original changed/GUI launched",
        )?;
        Ok(())
    })();
    wire.save_trace(&out.join("packaged-server.trace.json"))?;
    result?;
    write_json(
        &out.join("acceptance.json"),
        &json!({"passed":true,"relocated_files":files,"engine_mode":mode,"empty_PATH":true,"atomic_batch_and_rollback":true,"repeat_fragment_batch_and_restore":true,"approval_refusal":true,"no_op_without_transient_filesystem_writes":true,"real_CLI_blue_pixels":true,"source_original_preserved":true,"native_GUI":false}),
    )?;
    println!("Package headless acceptance passed ({mode}, {files} files)");
    Ok(())
}
pub fn doctor(args: &Args) -> Result<()> {
    args.check(&["--package", "--output"])?;
    let source = args.required("--package")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir_in(Path::new("/tmp").canonicalize()?)?;
    let root = root.path().canonicalize()?;
    let package = root.join("installed");
    copy_tree(&source, &package)?;
    let library = package.join("libexec/inkscape-mcp");
    let env = environment(&root, &library)?;
    let executable = package.join("bin/inkscape-mcp");
    let mut profiles = 0;
    let mut check = |label: &str,
                     expected: bool,
                     key: Option<&str>,
                     env: &[(String, String)]|
     -> Result<Value> {
        let mut before = inventory(&root)?;
        // Doctor may create and remove an owned diagnostic profile under TMPDIR.
        // Preserve all surviving paths/bytes; root mtime is not a doctor invariant.
        before.as_object_mut().unwrap().remove("");
        let (status, text) = capture(command_with_env(&executable, env).arg("--doctor"))?;
        let report: Value = serde_json::from_str(&text)?;
        ensure(
            report["ready"] == expected && status == if expected { 0 } else { 1 },
            "doctor ready/exit drift",
        )?;
        ensure(
            report["native_gui_verified"] == false
                && report["checks"].get("clang").is_none()
                && report["checks"].get("glib_headers").is_none(),
            "doctor claimed GUI/compiler requirement",
        )?;
        if let Some(key) = key {
            ensure(
                report["checks"][key] == false,
                "doctor missing asset not named",
            )?;
        }
        let mut after = inventory(&root)?;
        after.as_object_mut().unwrap().remove("");
        write_json(
            &out.join(format!("{label}.filesystem.json")),
            &json!({"before":before,"after":after}),
        )?;
        ensure(after == before, "doctor changed filesystem")?;
        write_json(&out.join(format!("{label}.json")), &report)?;
        profiles += 1;
        Ok(report)
    };
    let baseline = check("ready", true, None, &env)?;
    ensure(
        baseline["checks"].as_object().is_some_and(|a| {
            a.iter()
                .all(|(k, v)| v == true && !k.contains("python") && !k.contains("inkex"))
        }),
        "doctor still requires Python/inkex or prerequisites failed",
    )?;
    verify_files(&package)?;
    for (label, path, key) in [
        (
            "supervisor",
            package.join("bin/inkscape-mcp-supervisor"),
            "fixed_supervisor",
        ),
        (
            "bridge",
            library.join("context.so"),
            "prebuilt_context_architecture",
        ),
        (
            "helper",
            library.join("helpers/inkscape_mcp_insert.inx"),
            "fixed_helper_assets",
        ),
        (
            "socket",
            package.join("bin/inkscape-mcp-live"),
            "fixed_socket_helper",
        ),
        (
            "inx",
            package.join("bin/inkscape-mcp-inx"),
            "fixed_inx_helper",
        ),
        (
            "bus",
            library.join("dbus/bin/dbus-daemon"),
            "dbus_daemon_architecture",
        ),
        (
            "gdbus",
            library.join("dbus/bin/gdbus"),
            "gdbus_architecture",
        ),
        ("manifest", library.join("package.json"), "package_manifest"),
    ] {
        if label == "bridge" && !cfg!(target_os = "macos") {
            continue;
        }
        let held = root.join("held-asset");
        fs::rename(&path, &held)?;
        let result = check(&format!("missing-{label}"), false, Some(key), &env);
        fs::rename(held, &path)?;
        result?;
    }
    let (path, key) = if cfg!(target_os = "macos") {
        (library.join("context.so"), "prebuilt_context_architecture")
    } else {
        (package.join("bin/inkscape-mcp-inx"), "fixed_inx_helper")
    };
    let held = root.join("held-asset");
    fs::rename(&path, &held)?;
    std::os::unix::fs::symlink(&held, &path)?;
    check("linked-bridge", false, Some(key), &env)?;
    fs::remove_file(&path)?;
    fs::write(&path, [0; 32])?;
    check("wrong-bridge", false, Some(key), &env)?;
    fs::remove_file(&path)?;
    fs::rename(held, &path)?;
    let fake = root.join("bin");
    fs::create_dir(&fake)?;
    let engine = fake.join("inkscape");
    fs::write(
        &engine,
        b"#!/bin/sh\n[ \"$1\" = --version ] || exit 73\nprintf 'Inkscape 1.2.2\\n'\n",
    )?;
    fs::set_permissions(&engine, fs::Permissions::from_mode(0o700))?;
    let mut old_env = env.clone();
    old_env.retain(|(k, _)| k != "PATH");
    old_env.push(("PATH".into(), fake.display().to_string()));
    ensure(
        check("old-engine", false, Some("inkscape_minimum_1_4"), &old_env)?["inkscape_version"]
            == "Inkscape 1.2.2",
        "old version fixture failed",
    )?;
    verify_files(&package)?;
    ensure(!root.join("not-launched").exists(), "doctor launched GUI")?;
    write_json(
        &out.join("acceptance.json"),
        &json!({"passed":true,"profiles":profiles,"read_only":true,"GUI_launched":false}),
    )?;
    println!("Doctor: {profiles} profiles passed");
    Ok(())
}
pub fn capture(command: &mut Command) -> Result<(i32, String)> {
    let (status, stdout, _) = capture_full(command)?;
    Ok((status, stdout))
}
pub fn capture_full(command: &mut Command) -> Result<(i32, String, String)> {
    capture_deadline(command, 60)
}
pub fn capture_deadline(command: &mut Command, seconds: u64) -> Result<(i32, String, String)> {
    let root = tempfile::tempdir()?;
    let stdout = root.path().join("stdout");
    let stderr = root.path().join("stderr");
    command
        .stdout(fs::File::create(&stdout)?)
        .stderr(fs::File::create(&stderr)?);
    let mut child = command.spawn()?;
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > std::time::Duration::from_secs(seconds)
            || fs::metadata(&stdout)?.len() + fs::metadata(&stderr)?.len() > 32 * 1024 * 1024
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err("owned acceptance process timed out".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    Ok((
        status.code().unwrap_or(-1),
        String::from_utf8(read(&stdout, 16 * 1024 * 1024)?)?,
        String::from_utf8(read(&stderr, 16 * 1024 * 1024)?)?,
    ))
}
fn native_cli(package: &Path, root: &Path, env: &[(String, String)], out: &Path) -> Result<()> {
    let native = root.join("native-inx");
    fs::create_dir(&native)?;
    fs::set_permissions(&native, fs::Permissions::from_mode(0o700))?;
    let fixture=crate::common::json(Path::new("migration/contracts/effect-data-cases.json"))?["fingerprints"][0].clone();
    let source = native.join("input.svg");
    fs::write(
        &source,
        fixture["svg"].as_str().ok_or("fixture SVG missing")?,
    )?;
    let mut request = json!({"nonce":format!("mcp_{}","a".repeat(32)),"operation":"style","selection":["r"],"style":{"fill":"blue"},"expected_ids":["g","r"],"expected_fingerprint":fixture["expected"]});
    for changed in [true, false] {
        write_json(&native.join("insert-request.json"), &request)?;
        let before = fs::read(&source)?;
        let (status, stdout, stderr) = capture_full(
            command_with_env(&package.join("bin/inkscape-mcp-inx"), env)
                .arg("--id=r")
                .arg(&source)
                .env("INKSCAPE_MCP_MANAGED_DIR", &native),
        )?;
        let reply = crate::common::json(&native.join("insert-result.json"))?;
        ensure(
            status == 0
                && stderr.is_empty()
                && reply["ok"] == true
                && (!stdout.is_empty()) == changed
                && fs::read(&source)? == before,
            "native one-shot change/noop failed/modified source",
        )?;
        if changed {
            fs::write(&source, stdout)?;
            request["expected_fingerprint"] = reply["fingerprint"].clone();
        }
    }
    let bus_dir = root.join("bus");
    fs::create_dir(&bus_dir)?;
    fs::set_permissions(&bus_dir, fs::Permissions::from_mode(0o700))?;
    let socket = bus_dir.join("bus.sock");
    let address = format!("unix:path={}", socket.display());
    let library = package.join("libexec/inkscape-mcp");
    let log = fs::File::create(out.join("packaged-bus.log"))?;
    let child = command_with_env(&library.join("dbus/bin/dbus-daemon"), env)
        .arg(format!(
            "--config-file={}",
            library.join("dbus/session.conf").display()
        ))
        .arg(format!("--address={address}"))
        .args(["--nofork", "--print-address=1"])
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    struct OwnedBus(std::process::Child);
    impl Drop for OwnedBus {
        fn drop(&mut self) {
            if self.0.try_wait().ok().flatten().is_none() {
                unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) };
                let start = std::time::Instant::now();
                while self.0.try_wait().ok().flatten().is_none() {
                    if start.elapsed() > std::time::Duration::from_secs(5) {
                        let _ = self.0.kill();
                        let _ = self.0.wait();
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
    let mut bus = OwnedBus(child);
    let start = std::time::Instant::now();
    while !socket.exists() {
        ensure(
            bus.0.try_wait()?.is_none() && start.elapsed() < std::time::Duration::from_secs(5),
            "packaged private bus exited/readiness timeout",
        )?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let (status, id) = capture(
        command_with_env(&library.join("dbus/bin/gdbus"), env).args([
            "call",
            "--address",
            &address,
            "--dest",
            "org.freedesktop.DBus",
            "--object-path",
            "/org/freedesktop/DBus",
            "--method",
            "org.freedesktop.DBus.GetId",
        ]),
    )?;
    ensure(
        status == 0 && id.trim().len() > 30,
        "private bus exchange failed",
    )?;
    Ok(())
}
#[cfg(test)]
mod copy_tests {
    use super::*;
    #[test]
    fn nested_copy_preserves_empty_directories_and_refuses_links_before_any_publication() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("empty")).unwrap();
        fs::write(source.join("asset"), "bytes").unwrap();
        let out = root.path().join("deep/nested/copy");
        copy_tree(&source, &out).unwrap();
        assert!(out.join("empty").is_dir());
        assert_eq!(fs::read_to_string(out.join("asset")).unwrap(), "bytes");
        assert!(copy_tree(&source, &out).is_err());
        std::os::unix::fs::symlink(source.join("asset"), source.join("link")).unwrap();
        let bad = root.path().join("unpublished/copy");
        assert!(copy_tree(&source, &bad).is_err());
        assert!(!bad.parent().unwrap().exists());
    }
}
