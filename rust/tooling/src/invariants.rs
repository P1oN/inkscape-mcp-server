use crate::{
    acceptance::{inventory, png_pixel},
    common::*,
    wire::{Wire, data, isolated},
};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
fn error(reply: &Value) -> bool {
    reply["result"]["isError"] == true
}
pub fn png(path: &Path, size: u32, pixel: [u8; 4]) -> Result<()> {
    let mut encoder = png::Encoder::new(fs::File::create(path)?, size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixel.repeat(size as usize * size as usize))?;
    Ok(())
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output", "--engine-mode"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let root = root.path().canonicalize()?;
    let workspace = root.join("workspace");
    fs::create_dir(&workspace)?;
    let mut command = isolated(&binary, &workspace);
    command
        .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
        .env("INKSCAPE_MCP_TOOL_PROFILE", "full")
        .env("INKSCAPE_MCP_TOOL_DESC", "full")
        .env("INKSCAPE_MCP_RAW_ACTION_ENABLED", "true");
    let mode = args
        .values
        .get("--engine-mode")
        .map(String::as_str)
        .unwrap_or("per_call");
    ensure(["shell", "per_call"].contains(&mode), "invalid engine mode")?;
    command.env("INKSCAPE_MCP_ENGINE_MODE", mode);
    if args.command == "defects-acceptance" {
        let vendor = root.join("vendor");
        fs::create_dir(&vendor)?;
        let engine = vendor.join("inkscape");
        fs::write(&engine, "#!/bin/bash\nexit 1\n")?;
        fs::set_permissions(engine, fs::Permissions::from_mode(0o700))?;
        command.env("PATH", vendor);
    }
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let initialized = wire.initialize()?;
    let result = match args.command.as_str() {
        "diagnostic-acceptance" => diagnostics(&mut wire, &workspace, &out),
        "special-file-acceptance" => special(&mut wire, &root, &workspace),
        "defects-acceptance" => defects(&mut wire, &workspace, &initialized),
        "compare-acceptance" => compare(&mut wire, &root, &workspace, &out),
        "engine-routes-acceptance" => routes(&mut wire, &root, &workspace, &out),
        _ => Err("unknown invariant suite".into()),
    };
    wire.save_trace(&out.join("server.trace.json"))?;
    let checks = result?;
    wire.close();
    if args.command == "defects-acceptance" {
        let failed = root.join("failed-workspace");
        fs::create_dir(&failed)?;
        fs::write(
            failed.join("fixture.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
        )?;
        fs::create_dir_all(failed.join(".inkscape-mcp/registry.json"))?;
        let mut command = isolated(&binary, &failed);
        command
            .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
            .env("PATH", root.join("vendor"));
        let mut wire = Wire::spawn(&mut command, &out.join("failed-server.stderr.log"))?;
        let result = (|| -> Result<()> {
            wire.initialize()?;
            ensure(
                error(&wire.call("open_document", json!({"path":"fixture.svg"}))?),
                "invalid registry destination accepted",
            )?;
            let docs = wire.request(
                "resources/read",
                Some(json!({"uri":"inkscape://documents"})),
            )?;
            let docs: Value = serde_json::from_str(
                docs["result"]["contents"][0]["text"]
                    .as_str()
                    .ok_or("documents missing")?,
            )?;
            ensure(
                docs["documents"] == json!([]) && !failed.join(".inkscape-mcp/documents").exists(),
                "failed open published document/candidate copies",
            )?;
            Ok(())
        })();
        wire.save_trace(&out.join("failed-server.trace.json"))?;
        result?;
    }
    ensure(
        !workspace.join("absent-session").exists(),
        "managed GUI created",
    )?;
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"suite":args.command,"checks":checks,"engine_mode":mode,"binary_sha256":hash(&binary)?,"native_GUI":false}),
    )?;
    println!("{} passed ({checks} checks)", args.command);
    Ok(())
}
fn diagnostics(wire: &mut Wire, workspace: &Path, out: &Path) -> Result<usize> {
    let mut unknown = json!({"width":16,"height":16});
    unknown["q".repeat(250_000)] = json!(["🌿".repeat(250_000)]);
    for (label, tool, args, kind) in [
        (
            "unknown-field",
            "create_document",
            unknown,
            "unexpected_keyword_argument",
        ),
        (
            "discriminator",
            "apply_edits",
            json!({"doc_id":"none","edits":[{"op":"tag".repeat(250_000)}]}),
            "union_tag_invalid",
        ),
        (
            "nested-wrong-scalar",
            "create_document",
            json!({"width":[{"large":"🌿\n".repeat(250_000)}],"height":16}),
            "float_type",
        ),
    ] {
        let before = inventory(workspace)?;
        let reply = wire.call(tool, args)?;
        let size = serde_json::to_vec(&reply)?.len();
        ensure(
            error(&reply)
                && reply["result"]["content"][0]["text"]
                    .as_str()
                    .is_some_and(|s| s.contains(kind)),
            "unexpected validation error",
        )?;
        write_json(
            &out.join(format!("{label}-size.json")),
            &json!({"response_bytes":size,"limit":4096}),
        )?;
        ensure(
            size < 4096 && before == inventory(workspace)?,
            "diagnostic cap/before-write refusal failed",
        )?;
    }
    Ok(3)
}
fn special(wire: &mut Wire, root: &Path, workspace: &Path) -> Result<usize> {
    fs::create_dir(workspace.join("directory"))?;
    let path = std::ffi::CString::new(workspace.join("pipe").as_os_str().as_encoded_bytes())?;
    ensure(
        unsafe { libc::mkfifo(path.as_ptr(), 0o600) } == 0,
        "FIFO creation failed",
    )?;
    let outside = root.join("outside");
    fs::write(&outside, b"preserved outside original")?;
    std::os::unix::fs::symlink(&outside, workspace.join("link"))?;
    std::os::unix::fs::symlink(root, workspace.join("parent"))?;
    let key = data(wire.call("get_workspace_info", json!({}))?)?["roots"][0]["root_id"]
        .as_str()
        .ok_or("root key missing")?
        .to_string();
    for path in ["pipe", "directory", "link", "parent/outside"] {
        for tool in ["open_document", "stat_artifact"] {
            ensure(
                error(&wire.call(tool, json!({"path":path}))?),
                "special/escaped input accepted",
            )?;
        }
        ensure(wire.request("resources/read",Some(json!({"uri":format!("inkscape://artifact/{key}/{}",URL_SAFE_NO_PAD.encode(path))})))?.get("error").is_some(),"special/escaped artifact accepted")?;
    }
    ensure(
        fs::read(&outside)? == b"preserved outside original"
            && !workspace.join(".inkscape-mcp").exists(),
        "outside changed/refusals wrote files",
    )?;
    let mut names: Vec<_> = fs::read_dir(workspace)?
        .map(|row| row.map(|row| row.file_name()))
        .collect::<std::io::Result<_>>()?;
    names.sort();
    ensure(
        names == ["directory", "link", "parent", "pipe"].map(std::ffi::OsString::from),
        "special fixture changed",
    )?;
    Ok(12)
}
fn defects(wire: &mut Wire, workspace: &Path, initialized: &Value) -> Result<usize> {
    ensure(
        initialized["result"]["instructions"]
            .as_str()
            .is_some_and(|s| s.contains("client-trust boundary")),
        "approval trust boundary missing",
    )?;
    let original=b"<svg xmlns=\"http://www.w3.org/2000/svg\"><g id=\"g\"><rect id=\"r\"/></g><use id=\"u\" href=\"#r\"/><animate id=\"a\" begin=\"r.click+1.5s\"/></svg>";
    fs::write(workspace.join("fixture.svg"), original)?;
    let doc = data(wire.call("open_document", json!({"path":"fixture.svg"}))?)?["doc_id"].clone();
    let working = workspace
        .join(".inkscape-mcp/documents")
        .join(doc.as_str().ok_or("doc missing")?)
        .join("working/document.svg");
    let before = fs::read(&working)?;
    for (tool, args) in [
        (
            "delete_object",
            json!({"doc_id":doc,"object_ids":["g"],"approval_token":"client-confirmed-test"}),
        ),
        (
            "apply_edits",
            json!({"doc_id":doc,"approval_token":"client-confirmed-test","edits":[{"op":"set_fill","object_ids":["r"],"color":"blue"},{"op":"delete_object","object_ids":["g"]}]}),
        ),
        (
            "delete_object",
            json!({"doc_id":doc,"object_ids":["g","u"]}),
        ),
        (
            "delete_object",
            json!({"doc_id":doc,"object_ids":["g","u"],"approval_token":"client-confirmed-test"}),
        ),
    ] {
        ensure(
            error(&wire.call(tool, args)?) && fs::read(&working)? == before,
            "reference/SMIL/approval refusal mutated working copy",
        )?;
    }
    let changed = data(wire.call(
        "delete_object",
        json!({"doc_id":doc,"object_ids":["g","u","a"],"approval_token":"client-confirmed-test"}),
    )?)?;
    ensure(
        changed["changed"] == true && !fs::read_to_string(&working)?.contains("href=\"#r\""),
        "joint deletion left dangling use",
    )?;
    data(wire.call(
        "restore_snapshot",
        json!({"doc_id":doc,"snapshot_id":changed["snapshot_id"]}),
    )?)?;
    ensure(
        fs::read(&working)? == before && fs::read(workspace.join("fixture.svg"))? == original,
        "restore/original preservation failed",
    )?;
    Ok(5)
}
fn compare(wire: &mut Wire, root: &Path, workspace: &Path, out: &Path) -> Result<usize> {
    let safe = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><rect id=\"r\" width=\"16\" height=\"16\" fill=\"blue\"/></svg>";
    fs::write(workspace.join("source.svg"), safe)?;
    png(&root.join("outside.png"), 16, [213, 47, 89, 255])?;
    let doc = data(wire.call("open_document", json!({"path":"source.svg"}))?)?["doc_id"].clone();
    let snapshot =
        data(wire.call("create_snapshot", json!({"doc_id":doc}))?)?["snapshot_id"].clone();
    let working = workspace
        .join(".inkscape-mcp/documents")
        .join(doc.as_str().ok_or("doc missing")?)
        .join("working/document.svg");
    let unsafe_svg = safe.replace(
        "<rect id=\"r\" width=\"16\" height=\"16\" fill=\"blue\"/>",
        &format!(
            "<image href=\"{}/outside.png\" width=\"16\" height=\"16\"/>",
            root.display()
        ),
    );
    fs::write(&working, unsafe_svg)?;
    let args = json!({"doc_id":doc,"snapshot_id":snapshot,"region":{"x":0,"y":0,"width":16,"height":16},"width_px":16,"inline":false});
    let before = inventory(workspace)?;
    ensure(
        error(&wire.call("compare_region", args.clone())?),
        "unsafe second source accepted",
    )?;
    let after = inventory(workspace)?;
    write_json(
        &out.join("refusal-filesystem.json"),
        &json!({"before":before,"after":after}),
    )?;
    ensure(
        before == after,
        "second source refusal published first image",
    )?;
    fs::write(&working, safe.replace("blue", "red"))?;
    let compared = data(wire.call("compare_region", args)?)?;
    for (key, pixel) in [("before", [0, 0, 255, 255]), ("after", [255, 0, 0, 255])] {
        let resource = wire.request("resources/read", Some(json!({"uri":compared[key]["uri"]})))?;
        let bytes = STANDARD.decode(
            resource["result"]["contents"][0]["blob"]
                .as_str()
                .ok_or("PNG resource missing")?,
        )?;
        fs::write(out.join(format!("{key}.png")), &bytes)?;
        png_pixel(&bytes, 8, 8, pixel)?;
    }
    ensure(
        fs::read_to_string(workspace.join("source.svg"))? == safe,
        "original changed",
    )?;
    Ok(3)
}
fn routes(wire: &mut Wire, root: &Path, workspace: &Path, out: &Path) -> Result<usize> {
    png(&workspace.join("tile.png"), 16, [213, 47, 89, 255])?;
    png(&root.join("outside.png"), 16, [213, 47, 89, 255])?;
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"32\" height=\"16\"><image id=\"img\" href=\"tile.png\" width=\"16\" height=\"16\"/><path id=\"p\" d=\"M20,0 L28,0 L28,16 L20,16 Z\" fill=\"blue\"/></svg>";
    fs::write(workspace.join("safe.svg"), svg)?;
    fs::write(
        workspace.join("unsafe.svg"),
        svg.replace("tile.png", root.join("outside.png").to_str().unwrap()),
    )?;
    let doc = data(wire.call("open_document", json!({"path":"safe.svg"}))?)?["doc_id"].clone();
    let working = workspace
        .join(".inkscape-mcp/documents")
        .join(doc.as_str().ok_or("doc missing")?)
        .join("working/document.svg");
    for (tool, mut args) in [
        ("find_objects", json!({"accurate_bbox":true})),
        ("fit_to_content", json!({})),
        (
            "cleanup_paths",
            json!({"object_ids":["p"],"dry_run":false,"approval_token":"owned fixture"}),
        ),
        ("capture_frame", json!({"width_px":32,"inline":false})),
    ] {
        args["doc_id"] = doc.clone();
        let result = data(wire.call(tool, args)?)?;
        ensure(!result.is_null(), format!("empty result: {tool}"))?;
        if tool == "find_objects" {
            ensure(
                result["objects"]
                    .as_array()
                    .and_then(|a| a.iter().find(|item| item["object_id"] == "p"))
                    .is_some_and(|p| !p["bbox"].is_null()),
                "accurate bbox didn't use engine",
            )?;
        }
    }
    let working_svg = fs::read_to_string(&working)?;
    ensure(
        working_svg.contains("tile.png") && !working_svg.contains("imcp-assets-"),
        "linked asset lost/private path leaked",
    )?;
    let preview = data(wire.call(
        "render_preview",
        json!({"doc_id":doc,"width_px":32,"inline":false}),
    )?)?;
    png_pixel(
        &read(
            &workspace.join(
                preview["artifact_path"]
                    .as_str()
                    .ok_or("preview path missing")?,
            ),
            16 * 1024 * 1024,
        )?,
        4,
        4,
        [213, 47, 89, 255],
    )?;
    let bad = data(wire.call("open_document", json!({"path":"unsafe.svg"}))?)?["doc_id"].clone();
    for (tool, mut args) in [
        ("find_objects", json!({"accurate_bbox":true})),
        ("capture_frame", json!({"width_px":32,"inline":false})),
        (
            "export_document",
            json!({"format":"png","out_dir":"new-output"}),
        ),
    ] {
        args["doc_id"] = bad.clone();
        let before = inventory(workspace)?;
        let reply = wire.call(tool, args)?;
        if tool == "find_objects" {
            let value = data(reply)?;
            ensure(
                value["objects"]
                    .as_array()
                    .and_then(|a| a.iter().find(|p| p["object_id"] == "p"))
                    .is_some_and(|p| p["bbox"].is_null()),
                "unsafe bbox didn't fall back to DOM",
            )?;
        } else {
            ensure(
                error(&reply),
                format!("outside dependency accepted: {tool}"),
            )?;
        }
        let after = inventory(workspace)?;
        write_json(
            &out.join(format!("{tool}-filesystem.json")),
            &json!({"before":before,"after":after}),
        )?;
        ensure(before == after, "refused route changed workspace")?;
    }
    ensure(
        fs::read_to_string(workspace.join("safe.svg"))? == svg,
        "original SVG changed",
    )?;
    Ok(8)
}
