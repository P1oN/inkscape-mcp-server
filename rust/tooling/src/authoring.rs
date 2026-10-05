//! Real STDIO + Inkscape CLI acceptance on deliberately authored synthetic vectors.
use crate::{
    acceptance::inventory,
    common::*,
    wire::{Wire, data, isolated},
};
use serde_json::{Value, json};
use std::{fs, io::Cursor, path::Path};
const ARTWORK: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" width="320" height="160" viewBox="0 0 320 160">
<rect id="background" width="320" height="160" fill="white"/>
<g id="flower" inkscape:label="Flower"><path id="petals" fill="#e76798" d="M60.000 23.000 C67.330 23.000 67.675 44.028 71.000 45.947 C74.325 47.867 92.708 37.652 96.373 44.000 C100.038 50.348 82.000 61.160 82.000 65.000 C82.000 68.840 100.038 79.652 96.373 86.000 C92.708 92.348 74.325 82.133 71.000 84.053 C67.675 85.972 67.330 107.000 60.000 107.000 C52.670 107.000 52.325 85.972 49.000 84.053 C45.675 82.133 27.292 92.348 23.627 86.000 C19.962 79.652 38.000 68.840 38.000 65.000 C38.000 61.160 19.962 50.348 23.627 44.000 C27.292 37.652 45.675 47.867 49.000 45.947 C52.325 44.028 52.670 23.000 60.000 23.000 Z"/><circle id="center" cx="60" cy="65" r="15" fill="#ffd454"/></g>
<g id="folds" inkscape:label="Independent folds" fill="none" stroke="#3c5472" stroke-width="3"><path id="fold-a" d="M130 35 Q150 60 140 100"/><path id="fold-b" d="M155 35 Q175 60 165 100"/></g>
<g id="snowball" inkscape:label="Snowball"><circle id="round-outline" cx="235" cy="65" r="40" fill="#e8f4fc" stroke="#83a7c0" stroke-width="2"/></g>
<rect id="covered" x="285" y="20" width="20" height="20" fill="red"/><rect id="cover" x="280" y="15" width="30" height="30" fill="#3c5472"/>
<rect id="partial" x="280" y="80" width="25" height="25" fill="#ffd454"/><rect id="partial-cover" x="290" y="90" width="25" height="25" fill="#e76798"/>
<g id="reference-holder" display="none"><path id="reference-target" d="M0 0L4 4"/></g><use id="required-use" href="#reference-target" opacity="0"/>
<defs><clipPath id="clip"><circle id="clip-geometry" cx="60" cy="65" r="15"/></clipPath><mask id="mask"><rect id="mask-geometry" width="10" height="10" fill="white"/></mask></defs>
</svg>"##;
fn preview(
    wire: &mut Wire,
    workspace: &Path,
    out: &Path,
    doc: &Value,
    label: &str,
) -> Result<Vec<u8>> {
    let result = data(wire.call(
        "render_preview",
        json!({"doc_id":doc,"width_px":640,"inline":false}),
    )?)?;
    let bytes = read(
        &workspace.join(
            result["artifact_path"]
                .as_str()
                .ok_or("preview path missing")?,
        ),
        16 * 1024 * 1024,
    )?;
    fs::write(out.join(format!("{label}.png")), &bytes)?;
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut pixels = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or("PNG output size missing")?
    ];
    let info = reader.next_frame(&mut pixels)?;
    pixels.truncate(info.buffer_size());
    ensure(
        info.width == 640 && info.height == 320,
        "unexpected fixture dimensions",
    )?;
    Ok(pixels)
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output", "--engine-mode"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let mode = args
        .values
        .get("--engine-mode")
        .map(String::as_str)
        .unwrap_or("per_call");
    ensure(["shell", "per_call"].contains(&mode), "invalid engine mode")?;
    let root = tempfile::tempdir()?;
    let workspace = root.path().canonicalize()?;
    fs::write(workspace.join("artwork.svg"), ARTWORK)?;
    fs::write(out.join("artwork.svg"), ARTWORK)?;
    let mut command = isolated(&binary, &workspace);
    command
        .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
        .env("INKSCAPE_MCP_TOOL_PROFILE", "full")
        .env("INKSCAPE_MCP_ENGINE_MODE", mode);
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let mut evidence = json!({"binary_sha256":hash(&binary)?,"engine_mode":mode,"scope":"Owned synthetic vectors, real STDIO and CLI pixels; manual silhouette review is separate. No GUI, tracing or installed runtime changes."});
    let result = (|| -> Result<()> {
        let initialize = wire.initialize()?;
        let prompt=wire.request("prompts/get",Some(json!({"name":"compose_artwork","arguments":{"goal":"Editable flower, folds and snowball"}})))?;
        let guidance =
            include_str!("../../../migration/contracts/authoring-guidance.txt").trim_end();
        ensure(
            initialize["result"]["instructions"]
                .as_str()
                .is_some_and(|s| s.contains(guidance))
                && prompt["result"]["messages"][0]["content"]["text"]
                    .as_str()
                    .is_some_and(|s| s.contains(guidance)),
            "shared authoring guidance not delivered",
        )?;
        let doc =
            data(wire.call("open_document", json!({"path":"artwork.svg"}))?)?["doc_id"].clone();
        let fixture = inkscape_mcp_rust::xml::parse(ARTWORK.as_bytes(), 65536)?;
        let nodes = inkscape_mcp_rust::helper_svg::elements(
            fixture.get_root_element().ok_or("fixture root missing")?,
        );
        let target = |id: &str| {
            nodes
                .iter()
                .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
                .cloned()
                .ok_or_else(|| format!("fixture object missing: {id}"))
        };
        let flower = target("flower")?;
        ensure(
            flower
                .get_child_nodes()
                .iter()
                .filter(|n| n.is_element_node())
                .count()
                == 2
                && target("petals")?.get_name() == "path"
                && target("center")?.get_name() == "circle",
            "flower must have one outline and separate center",
        )?;
        ensure(
            target("round-outline")?.get_name() == "circle"
                && target("folds")?
                    .get_child_nodes()
                    .iter()
                    .filter(|n| n.is_element_node())
                    .count()
                    == 2
                && target("fold-a")? != target("fold-b")?,
            "round silhouette/separate folds structure changed",
        )?;
        ensure(
            !nodes.iter().any(|n| n.get_name() == "image"),
            "fixture contains raster artwork",
        )?;
        let before = inventory(&workspace)?;
        let report=data(wire.call("quality_report",json!({"doc_id":doc,"editability":{"object_roles":[{"object_id":"folds","role":"independent_strokes"}]}}))?)?;
        ensure(
            inventory(&workspace)? == before,
            "quality report changed bytes/history",
        )?;
        let findings = report["editability"]["authoring"]["findings"]
            .as_array()
            .ok_or("authoring report absent")?;
        ensure(
            !findings
                .iter()
                .any(|f| matches!(f["code"].as_str(), Some("stroke_fill" | "combined_strokes"))),
            "separate folds failed",
        )?;
        ensure(
            !findings
                .iter()
                .any(|f| f["object_id"] == "covered" || f["object_id"] == "partial"),
            "occlusion inferred from geometry",
        )?;
        ensure(
            findings.iter().any(|f| {
                f["object_id"] == "reference-target" && f["code"] == "hidden_required_geometry"
            }),
            "required source not protected",
        )?;
        let disabled = data(wire.call(
            "quality_report",
            json!({"doc_id":doc,"editability":{"enabled":false}}),
        )?)?;
        ensure(
            report["score"] == disabled["score"] && report["ok"] == disabled["ok"],
            "authoring advice changed validity/score",
        )?;
        ensure(
            inventory(&workspace)? == before,
            "disabled report changed workspace",
        )?;
        let invalid=wire.call("quality_report",json!({"doc_id":doc,"editability":{"object_roles":[{"object_id":"folds","role":"guessed"}]}}))?;
        ensure(
            invalid["result"]["isError"] == true && inventory(&workspace)? == before,
            "typed role rejection mutated workspace",
        )?;
        evidence["report"] = report;
        let rendered = preview(&mut wire, &workspace, &out, &doc, "before")?;
        let refusal_before = inventory(&workspace)?;
        let refused = wire.call(
            "delete_object",
            json!({"doc_id":doc,"object_ids":["covered"]}),
        )?;
        ensure(
            refused["result"]["isError"] == true && inventory(&workspace)? == refusal_before,
            "approval gate failed",
        )?;
        // Explicitly reviewed synthetic repair; no report selects or deletes objects.
        let deleted=data(wire.call("delete_object",json!({"doc_id":doc,"object_ids":["covered"],"approval_token":"owned synthetic acceptance repair"}))?)?;
        ensure(
            deleted["changed"] == true && !deleted["snapshot_id"].as_str().unwrap_or("").is_empty(),
            "approved repair lacks snapshot",
        )?;
        ensure(
            rendered == preview(&mut wire, &workspace, &out, &doc, "after")?,
            "approved repair changed rendered pixels",
        )?;
        let before_noop = inventory(&workspace)?;
        let noop=data(wire.call("delete_object",json!({"doc_id":doc,"object_ids":["covered"],"approval_token":"owned synthetic acceptance repair"}))?)?;
        ensure(
            noop["changed"] == false && inventory(&workspace)? == before_noop,
            "repeat repair is not a history-free no-op",
        )?;
        data(wire.call(
            "restore_snapshot",
            json!({"doc_id":doc,"snapshot_id":deleted["snapshot_id"]}),
        )?)?;
        ensure(
            rendered == preview(&mut wire, &workspace, &out, &doc, "restored")?,
            "snapshot restore changed pixels",
        )?;
        let snapshots_before = data(wire.call("list_snapshots", json!({"doc_id":doc}))?)?;
        let referenced=wire.call("delete_object",json!({"doc_id":doc,"object_ids":["reference-holder"],"approval_token":"owned synthetic refusal test"}))?;
        ensure(
            referenced["result"]["isError"] == true,
            "external reference to subtree not refused",
        )?;
        ensure(
            snapshots_before == data(wire.call("list_snapshots", json!({"doc_id":doc}))?)?,
            "reference refusal created snapshot",
        )?;
        ensure(
            rendered == preview(&mut wire, &workspace, &out, &doc, "refused-reference")?,
            "reference refusal changed pixels",
        )?;
        ensure(
            fs::read_to_string(workspace.join("artwork.svg"))? == ARTWORK
                && !workspace.join("absent-session").exists(),
            "original/GUI invariant failed",
        )?;
        // Retain the operation records for independent inspection of approval and status.
        for (relative, _) in inventory(&workspace)?.as_object().unwrap() {
            if relative.contains("/operations/") && relative.ends_with(".json") {
                fs::write(
                    out.join(Path::new(relative).file_name().unwrap()),
                    fs::read(workspace.join(relative))?,
                )?;
            }
        }
        evidence["checks"] = json!([
            "shared initialization/compose instructions",
            "read-only bytes/snapshots/records",
            "typed role refusal",
            "separate editable fold objects",
            "advice independent of validity/score",
            "occlusion deferred and partial overlap retained",
            "required referenced subtree preserved",
            "approval refusal before mutation",
            "approved deletion with snapshot/record",
            "identical before/after pixels",
            "history-free repeated no-op",
            "snapshot restore pixels",
            "referenced subtree deletion refusal",
            "original SVG and GUI preserved"
        ]);
        Ok(())
    })();
    evidence["probe_completed"] = json!(result.is_ok());
    if let Err(error) = &result {
        evidence["error"] = json!(error.to_string());
    }
    wire.save_trace(&out.join("server.trace.json"))?;
    write_json(&out.join("probe.json"), &evidence)?;
    result?;
    println!(
        "Authoring: explicit roles, read-only review and approved pixel-preserving repair passed ({mode})"
    );
    Ok(())
}

/// Update generated wire snapshots after editing the single policy source.
pub fn sync_guidance(args: &Args) -> Result<()> {
    args.check(&[])?;
    let root = Path::new("migration/contracts");
    let guidance = fs::read_to_string(root.join("authoring-guidance.txt"))?;
    let marker = "\n\nEditable vector authoring quality:\n";
    let expand = |text: &str| -> Result<String> {
        let (prefix, _) = text
            .split_once(marker)
            .ok_or("authoring snapshot marker missing")?;
        Ok(format!("{prefix}{marker}{}", guidance.trim_end()))
    };
    for live in ["false", "true"] {
        for raw in ["false", "true"] {
            for profile in ["core", "full"] {
                for desc in ["short", "full"] {
                    let path = root.join(format!("live-{live}_raw-{raw}_{profile}_{desc}.json"));
                    let mut v = crate::common::json(&path)?;
                    v["initialize"]["instructions"] = json!(expand(
                        v["initialize"]["instructions"]
                            .as_str()
                            .ok_or("instructions missing")?
                    )?);
                    write_json(&path, &v)?;
                }
            }
        }
    }
    let path = root.join("prompt-messages.json");
    let mut v = crate::common::json(&path)?;
    let text = &mut v["compose_artwork"]["messages"][0]["content"]["text"];
    *text = json!(expand(text.as_str().ok_or("compose template missing")?)?);
    write_json(&path, &v)?;
    println!("Authoring guidance synchronized in 16 discovery snapshots and compose template");
    Ok(())
}
