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
    if args.command == "engine-routes-acceptance" && mode == "shell" {
        require_shell_boolean(&root, &mut command)?;
    }
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let initialized = wire.initialize()?;
    let result = match args.command.as_str() {
        "diagnostic-acceptance" => diagnostics(&mut wire, &workspace, &out),
        "special-file-acceptance" => special(&mut wire, &root, &workspace),
        "defects-acceptance" => defects(&mut wire, &workspace, &initialized),
        "compare-acceptance" => compare(&mut wire, &root, &workspace, &out),
        "engine-routes-acceptance" => {
            routes(&mut wire, &root, &workspace, &out).and_then(|checks| {
                let evidence = out.join("union");
                fs::create_dir_all(&evidence)?;
                boolean_union(&mut wire, &workspace, &evidence)?;
                let difference_evidence = out.join("difference");
                fs::create_dir_all(&difference_evidence)?;
                boolean_difference(&mut wire, &workspace, &difference_evidence)?;
                Ok(checks + 2)
            })
        }
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
fn require_shell_boolean(root: &Path, command: &mut std::process::Command) -> Result<()> {
    // Keep the normal fallbacks for other routes in the suite. Boolean
    // fixtures must exercise successful warm actions even if one-shot works.
    let engine = tool("inkscape")
        .unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "/Applications/Inkscape.app/Contents/MacOS/inkscape".into()
            } else {
                "/usr/bin/inkscape".into()
            }
        })
        .canonicalize()?;
    let vendor = root.join("shell-probe");
    fs::create_dir(&vendor)?;
    let proxy = vendor.join("inkscape");
    fs::write(
        &proxy,
        r#"#!/bin/sh
for arg in "$@"; do
    case "$arg" in
        --actions=*path-union*|--actions=*path-difference*) echo 'Boolean operation fell back from shell to one-shot' >&2; exit 77 ;;
    esac
done
exec "$INKSCAPE_MCP_ACCEPTANCE_ENGINE" "$@"
"#,
    )?;
    fs::set_permissions(proxy, fs::Permissions::from_mode(0o700))?;
    let path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::env::join_paths(std::iter::once(vendor).chain(std::env::split_paths(&path)))?;
    command
        .env("PATH", paths)
        .env("INKSCAPE_MCP_ACCEPTANCE_ENGINE", engine);
    Ok(())
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

fn open_fixture(wire: &mut Wire, workspace: &Path, name: &str, svg: &str) -> Result<Value> {
    fs::write(workspace.join(name), svg)?;
    Ok(data(wire.call("open_document", json!({"path":name}))?)?["doc_id"].clone())
}

fn fixture_pixels(
    wire: &mut Wire,
    workspace: &Path,
    doc: &Value,
    evidence: &Path,
) -> Result<Vec<u8>> {
    let result = data(wire.call(
        "render_preview",
        json!({"doc_id":doc,"width_px":256,"inline":false}),
    )?)?;
    let bytes = read(
        &workspace.join(result["artifact_path"].as_str().ok_or("preview missing")?),
        16 * 1024 * 1024,
    )?;
    fs::write(evidence, &bytes)?;
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info()?;
    let mut pixels = vec![0; reader.output_buffer_size().ok_or("PNG size missing")?];
    let info = reader.next_frame(&mut pixels)?;
    ensure(
        (info.width, info.height, info.color_type, info.bit_depth)
            == (256, 192, png::ColorType::Rgba, png::BitDepth::Eight),
        "unexpected geometry fixture render format",
    )?;
    Ok(pixels)
}

fn parse_geometry_fixture(svg: &[u8]) -> Result<libxml::tree::Document> {
    Ok(libxml::parser::Parser::default().parse_string_with_options(
        svg,
        libxml::parser::ParserOptions {
            no_blanks: true,
            no_net: true,
            ..Default::default()
        },
    )?)
}

fn boolean_union(wire: &mut Wire, workspace: &Path, out: &Path) -> Result<()> {
    const SOURCE: &str = include_str!("../tests/fixtures/boolean-union.svg");
    const WING: &str = r##"<rect id="wing" x="50" y="16" width="28" height="22" fill="#d85f4b"/>"##;
    const ISLAND: &str =
        r##"<rect id="island" x="45" y="56" width="9" height="5" fill="#5274ab"/>"##;
    const BODY_D: &str = "M12 10H58V53H12Z M25 22H35V34H25Z";
    const UNION_D: &str = "M12 10H58V16H78V38H58V53H12Z M25 22H35V34H25Z M45 56H54V61H45Z";

    let open = |wire: &mut Wire, name: &str, svg: &str| open_fixture(wire, workspace, name, svg);
    let doc = open(wire, "union.svg", SOURCE)?;
    let doc_dir = workspace
        .join(".inkscape-mcp/documents")
        .join(doc.as_str().ok_or("union document ID missing")?);
    let working = doc_dir.join("working/document.svg");
    let original_working = fs::read(&working)?;
    let before = inventory(&doc_dir)?;
    let operands = json!(["island", "wing", "body"]);
    let planned = data(wire.call("boolean_union", json!({"doc_id":doc,"object_ids":operands}))?)?;
    let unapproved = wire.call(
        "boolean_union",
        json!({"doc_id":doc,"object_ids":operands,"dry_run":false}),
    )?;
    ensure(
        planned["result_id"] == "body"
            && planned["changed"] == false
            && planned["snapshot_id"].is_null()
            && error(&unapproved)
            && inventory(&doc_dir)? == before,
        "union plan or missing-approval refusal changed document state",
    )?;
    let apply = |wire: &mut Wire, doc: &Value| {
        wire.call(
            "boolean_union",
            json!({"doc_id":doc,"object_ids":operands,"dry_run":false,"approval_token":"owned fixture"}),
        )
    };
    let applied = data(apply(wire, &doc)?)?;
    let operation = applied["operation_id"]
        .as_str()
        .ok_or("union operation missing")?;
    let record: Value = serde_json::from_slice(&fs::read(
        doc_dir.join("operations").join(format!("{operation}.json")),
    )?)?;
    ensure(
        applied["changed"] == true
            && applied["result_id"] == "body"
            && applied["affected_ids"] == operands
            && applied["snapshot_id"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
            && record["status"] == "applied"
            && record["snapshot_id"] == applied["snapshot_id"],
        "union did not record a reversible edit for the selected shapes",
    )?;
    let validated = data(wire.call("validate_document", json!({"doc_id":doc}))?)?;
    ensure(validated["ok"] == true, "union result did not validate")?;
    data(wire.call(
        "save_document_as",
        json!({"doc_id":doc,"dest_path":"union-final.svg","vector_only":true}),
    )?)?;
    let saved = fs::read(workspace.join("union-final.svg"))?;
    fs::write(out.join("union-final.svg"), &saved)?;

    let reference_svg = SOURCE
        .replace(WING, "")
        .replace(ISLAND, "")
        .replace(BODY_D, UNION_D);
    fs::write(out.join("union-reference.svg"), &reference_svg)?;
    let saved_svg = parse_geometry_fixture(&saved)?;
    let expected = parse_geometry_fixture(reference_svg.as_bytes())?;
    let body = |svg: &libxml::tree::Document| -> Result<libxml::tree::Node> {
        let mut context =
            libxml::xpath::Context::new(svg).map_err(|_| "cannot create fixture XPath")?;
        let mut nodes = context
            .findnodes("//*[@id='body']", None)
            .map_err(|_| "cannot find union body")?;
        ensure(nodes.len() == 1, "union must retain one body ID")?;
        Ok(nodes.pop().ok_or("missing union body")?)
    };
    let actual_body = body(&saved_svg)?;
    let d = actual_body
        .get_property_no_ns("d")
        .ok_or("union path data missing")?;
    ensure(
        actual_body.get_name() == "path" && !d.trim().is_empty(),
        "union result lacks an editable path",
    )?;
    body(&expected)?
        .set_property("d", &d)
        .map_err(|_| "cannot set expected path data")?;
    let normalized_root = |svg: &libxml::tree::Document| -> Result<String> {
        Ok(svg.node_to_string(&svg.get_root_element().ok_or("union root missing")?))
    };
    ensure(
        normalized_root(&saved_svg)? == normalized_root(&expected)?,
        "union changed paint, fill rule, references, paint order, or unrelated SVG structure",
    )?;

    let reopened =
        data(wire.call("open_document", json!({"path":"union-final.svg"}))?)?["doc_id"].clone();
    let report = data(wire.call(
        "quality_report",
        json!({"doc_id":reopened,"editability":{"object_roles":[{"object_id":"body","role":"closed_shape"}]}}),
    )?)?;
    let geometry = &report["editability"]["geometry"];
    ensure(
        geometry["truncated"] == false
            && geometry["findings"].as_array().is_some_and(|findings| {
                findings.iter().all(|finding| {
                    finding["object_id"] != "body"
                        || !matches!(
                            finding["code"].as_str(),
                            Some("open_shape" | "path_geometry_unknown")
                        )
                })
            }),
        "saved union did not pass the editable closed path checks",
    )?;
    let reference = open(wire, "union-reference.svg", &reference_svg)?;
    let actual = fixture_pixels(wire, workspace, &reopened, &out.join("union-final.png"))?;
    let expected = fixture_pixels(
        wire,
        workspace,
        &reference,
        &out.join("union-reference.png"),
    )?;
    ensure(
        actual == expected,
        "saved union render differs from the independent reference",
    )?;

    let restored = data(wire.call(
        "restore_snapshot",
        json!({"doc_id":doc,"snapshot_id":applied["snapshot_id"]}),
    )?)?;
    ensure(
        fs::read(&working)? == original_working
            && restored["restored_sha256"] == sha(&original_working)
            && fs::read(workspace.join("union.svg"))? == SOURCE.as_bytes()
            && fs::read(workspace.join("union-final.svg"))? == saved,
        "union rollback, saved copy, or original preservation failed",
    )?;

    for (name, svg, reason) in [
        (
            "union-linked.svg",
            SOURCE.replace("href=\"#mark\" x=\"105\"", "href=\"#island\" x=\"105\""),
            "reference IDs in the removed subtree",
        ),
        (
            "union-cross-parent.svg",
            SOURCE.replace(ISLAND, &format!("<g>{ISLAND}</g>")),
            "sibling shapes with directly editable geometry",
        ),
        (
            "union-connector.svg",
            SOURCE.replace(
                "id=\"body\"",
                "id=\"body\" inkscape:connector-type=\"polyline\"",
            ),
            "sibling shapes with directly editable geometry",
        ),
        (
            "union-css-geometry.svg",
            SOURCE.replace(
                "fill-rule:evenodd",
                "fill-rule:evenodd;/* local frame */transform:translate(4px,0)",
            ),
            "sibling shapes with directly editable geometry",
        ),
    ] {
        let doc = open(wire, name, &svg)?;
        let dir = workspace
            .join(".inkscape-mcp/documents")
            .join(doc.as_str().ok_or("union document ID missing")?);
        let working = dir.join("working/document.svg");
        let before = fs::read(&working)?;
        let snapshots = inventory(&dir.join("snapshots"))?;
        let refused = apply(wire, &doc)?;
        ensure(
            error(&refused)
                && refused["result"]["content"][0]["text"]
                    .as_str()
                    .is_some_and(|text| text.contains(reason))
                && fs::read(working)? == before
                && inventory(&dir.join("snapshots"))? == snapshots
                && fs::read(workspace.join(name))? == svg.as_bytes(),
            format!("unsafe union must refuse without changing the drawing or snapshots: {name}"),
        )?;
    }
    Ok(())
}

fn boolean_difference(wire: &mut Wire, workspace: &Path, out: &Path) -> Result<()> {
    const SOURCE: &str = include_str!("../tests/fixtures/boolean-difference.svg");
    const BODY: &str = r##"<rect id="body" x="12" y="10" width="60" height="43" fill="#18666a"/>"##;
    const CUTTER: &str = r#"<circle id="cutter" cx="38" cy="30" r="11"/>"#;
    let input = SOURCE.replace(BODY, &format!("{BODY}\n  {CUTTER}"));
    let reference = SOURCE.replace(
        BODY,
        r##"<path id="body" fill="#18666a" fill-rule="evenodd" d="M12 10H72V53H12Z M27 30A11 11 0 1 0 49 30A11 11 0 1 0 27 30Z"/>"##,
    );
    let open = |wire: &mut Wire, name: &str, svg: &str| open_fixture(wire, workspace, name, svg);
    let preview = |wire: &mut Wire, doc: &Value, name: &str| {
        fixture_pixels(wire, workspace, doc, &out.join(name))
    };
    let original = open(wire, "boolean-uncut.svg", SOURCE)?;
    let before = preview(wire, &original, "boolean-before.png")?;
    let doc = open(wire, "boolean.svg", &input)?;
    let working = workspace
        .join(".inkscape-mcp/documents")
        .join(doc.as_str().ok_or("boolean document ID missing")?)
        .join("working/document.svg");
    let initial = fs::read(&working)?;
    for id in ["body", "cutter"] {
        let found = data(wire.call("find_objects", json!({"doc_id":doc,"id_prefix":id}))?)?;
        ensure(
            found["objects"][0]["object_id"] == id && found["count"] == 1,
            format!("fixture object lookup failed: {id}"),
        )?;
    }
    let plan = data(wire.call(
        "boolean_difference",
        json!({"doc_id":doc,"object_ids":["cutter","body"]}),
    )?)?;
    let unapproved = wire.call(
        "boolean_difference",
        json!({"doc_id":doc,"object_ids":["cutter","body"],"dry_run":false}),
    )?;
    ensure(
        plan["dry_run"] == true
            && plan["changed"] == false
            && plan["result_id"] == "body"
            && plan["snapshot_id"].is_null()
            && plan.get("changed_region").is_none()
            && error(&unapproved)
            && unapproved["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|text| text.contains("requires an explicit approval_token"))
            && fs::read(&working)? == initial,
        "boolean dry run or approval gate failed",
    )?;
    let applied = data(wire.call(
        "boolean_difference",
        json!({"doc_id":doc,"object_ids":["cutter","body"],"dry_run":false,"approval_token":"owned fixture"}),
    )?)?;
    ensure(
        applied["changed"] == true
            && applied["result_id"] == "body"
            && applied["snapshot_id"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
        "boolean edit was not recorded",
    )?;
    let region = &applied["changed_region"];
    ensure(
        region["width"].as_f64().is_some_and(|w| w > 0. && w < 128.)
            && region["height"].as_f64().is_some_and(|h| h > 0. && h < 96.),
        "boolean result did not report a focused region",
    )?;
    let comparison = data(wire.call(
        "compare_region",
        json!({"doc_id":applied["doc_id"],"snapshot_id":applied["snapshot_id"],"region":region,"width_px":128,"inline":false}),
    )?)?;
    let region_preview = |side: &str| {
        let path = comparison[side]["workspace_relative_path"]
            .as_str()
            .ok_or("region preview missing")?;
        read(&workspace.join(path), 16 * 1024 * 1024)
    };
    ensure(
        comparison["region"] == *region && region_preview("before")? != region_preview("after")?,
        "returned region did not show a before/after change",
    )?;
    let validated = data(wire.call("validate_document", json!({"doc_id":doc}))?)?;
    ensure(validated["ok"] == true, "boolean result did not validate")?;
    data(wire.call(
        "save_document_as",
        json!({"doc_id":doc,"dest_path":"boolean-final.svg","vector_only":true}),
    )?)?;
    let saved = fs::read(workspace.join("boolean-final.svg"))?;
    fs::write(out.join("boolean-final.svg"), &saved)?;
    let restored = data(wire.call(
        "restore_snapshot",
        json!({"doc_id":doc,"snapshot_id":applied["snapshot_id"]}),
    )?)?;
    ensure(
        fs::read(&working)? == initial
            && restored["restored_sha256"] == sha(&initial)
            && fs::read(workspace.join("boolean.svg"))? == input.as_bytes()
            && fs::read(workspace.join("boolean-final.svg"))? == saved,
        "boolean rollback, saved copy, or original preservation failed",
    )?;

    let saved_doc = parse_geometry_fixture(&saved)?;
    let root = saved_doc.get_root_element().ok_or("boolean root missing")?;
    let inner = root
        .get_child_elements()
        .into_iter()
        .find(|n| n.get_property_no_ns("id").as_deref() == Some("outer"))
        .and_then(|n| n.get_first_element_child())
        .ok_or("boolean group missing")?;
    let children = inner.get_child_elements();
    let body = children
        .get(1)
        .ok_or("boolean body missing from its stack position")?;
    let d = body
        .get_property_no_ns("d")
        .ok_or("boolean path data missing")?;
    let expected = parse_geometry_fixture(
        SOURCE
            .replace(BODY, r##"<path id="body" fill="#18666a"/>"##)
            .as_bytes(),
    )?;
    let mut context =
        libxml::xpath::Context::new(&expected).map_err(|_| "cannot create fixture XPath")?;
    let mut expected_body = context
        .findnodes("//*[@id='body']", None)
        .map_err(|_| "cannot find expected body")?
        .pop()
        .ok_or("expected body missing")?;
    expected_body
        .set_property("d", &d)
        .map_err(|_| "cannot set expected path data")?;
    ensure(
        saved_doc.node_to_string(&root)
            == expected
                .node_to_string(&expected.get_root_element().ok_or("expected root missing")?),
        "boolean paint, ID, or surrounding SVG structure changed",
    )?;

    let saved_id =
        data(wire.call("open_document", json!({"path":"boolean-final.svg"}))?)?["doc_id"].clone();
    let actual = preview(wire, &saved_id, "boolean-final.png")?;
    let reference = open(wire, "boolean-reference.svg", &reference)?;
    let expected = preview(wire, &reference, "boolean-reference.png")?;
    let (before, _) = before.as_chunks::<4>();
    let (expected, _) = expected.as_chunks::<4>();
    let (actual, _) = actual.as_chunks::<4>();
    let changed: Vec<_> = before
        .iter()
        .zip(expected)
        .enumerate()
        .filter_map(|(i, (a, b))| (a != b).then_some(i))
        .collect();
    ensure(!changed.is_empty(), "boolean reference has no cut")?;
    let mut cut = vec![false; 256 * 192];
    for i in changed {
        let (x, y) = (i % 256, i / 256);
        for row in y.saturating_sub(2)..=(y + 2).min(191) {
            for col in x.saturating_sub(2)..=(x + 2).min(255) {
                cut[row * 256 + col] = true;
            }
        }
    }
    ensure(
        actual
            .iter()
            .zip(expected)
            .zip(cut)
            .all(|((a, b), inside)| {
                a.iter()
                    .zip(b)
                    .all(|(a, b)| a.abs_diff(*b) <= if inside { 4 } else { 0 })
            }),
        "boolean render differs from reference",
    )?;

    let linked = input.replace("href=\"#mark\" x=\"105\"", "href=\"#cutter\" x=\"105\"");
    let linked = open(wire, "boolean-linked.svg", &linked)?;
    let working = workspace
        .join(".inkscape-mcp/documents")
        .join(
            linked
                .as_str()
                .ok_or("boolean linked document ID missing")?,
        )
        .join("working/document.svg");
    let before = fs::read(&working)?;
    let refused = wire.call(
        "boolean_difference",
        json!({"doc_id":linked,"object_ids":["body","cutter"],"dry_run":false,"approval_token":"owned fixture"}),
    )?;
    ensure(
        error(&refused)
            && refused
                .to_string()
                .contains("reference IDs in the removed subtree")
            && fs::read(working)? == before,
        "boolean difference must preserve a cutter referenced by surviving artwork",
    )?;
    {
        // The cutter extends beyond the body, and the viewBox is scaled and letterboxed.
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="10 20 100 50"><g id="group"><rect id="body" x="20" y="25" width="40" height="30" style="fill:#18666a;stroke:none"/><circle id="cutter" cx="56" cy="38" r="12"/></g></svg>"##;
        let doc = open(wire, "boolean-extended-cutter.svg", svg)?;
        let apply = |wire: &mut Wire, doc: &Value| {
            data(wire.call(
                "boolean_difference",
                json!({"doc_id":doc,"object_ids":["body","cutter"],"dry_run":false,"approval_token":"owned fixture"}),
            )?)
        };
        let applied = apply(wire, &doc)?;
        let region = &applied["changed_region"];
        let x = region["x"].as_f64().ok_or("missing extended-cutter x")?;
        let y = region["y"].as_f64().ok_or("missing extended-cutter y")?;
        let right = x + region["width"].as_f64().ok_or("missing region width")?;
        let bottom = y + region["height"].as_f64().ok_or("missing region height")?;
        ensure(
            (19.0..=20.0).contains(&x)
                && (24.0..=25.0).contains(&y)
                && (68.0..=70.0).contains(&right)
                && (55.0..=57.0).contains(&bottom),
            format!("region missed the cutter or used the wrong coordinate system: {region}"),
        )?;
        let svg = svg.replace("</svg>", r##"<use href="#group" x="65"/></svg>"##);
        let doc = open(wire, "boolean-clone.svg", &svg)?;
        let applied = apply(wire, &doc)?;
        ensure(
            applied["changed"] == true && applied.get("changed_region").is_none(),
            "region should be omitted when a clone of a changed ancestor can repaint elsewhere",
        )?;
    }
    Ok(())
}
