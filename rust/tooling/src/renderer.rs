use crate::{
    acceptance::{inventory, png_pixel},
    common::*,
    invariants::png,
    wire::{Wire, data, isolated},
};
use serde_json::json;
use std::fs;
fn quote(text: &str) -> String {
    let mut result = String::new();
    for c in text.bytes() {
        if c.is_ascii_alphanumeric() || b"_.~-/".contains(&c) {
            result.push(c as char)
        } else {
            result.push_str(&format!("%{c:02X}"))
        }
    }
    result
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
    let root = root.path().canonicalize()?;
    let workspace = root.join("workspace");
    fs::create_dir(&workspace)?;
    let assets = workspace.join("assets");
    fs::create_dir(&assets)?;
    let outside = root.join("owned-outside.png");
    let inside = workspace.join("owned-inside.png");
    let marker = [213, 47, 89, 255];
    for path in [&outside, &inside] {
        png(path, 16, marker)?;
    }
    std::os::unix::fs::symlink(&outside, workspace.join("owned-link.png"))?;
    fs::write(
        assets.join("leaf.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><image href=\"../owned-inside.png\" width=\"16\" height=\"16\"/></svg>",
    )?;
    fs::write(
        assets.join("unsafe.svg"),
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><image href=\"{}\" width=\"16\" height=\"16\"/></svg>",
            outside.display()
        ),
    )?;
    let shape = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><rect id=\"marker\" width=\"16\" height=\"16\" fill=\"rgb(213,47,89)\"/></svg>";
    for path in [
        assets.join("shape.svg"),
        assets.join("shape#%?.svg"),
        root.join("outside.svg"),
    ] {
        fs::write(path, shape)?;
    }
    let paint = "<svg xmlns=\"http://www.w3.org/2000/svg\"><defs><linearGradient id=\"paint\"><stop stop-color=\"rgb(213,47,89)\"/></linearGradient></defs></svg>";
    for path in [
        assets.join("paint.svg"),
        assets.join("paint#%?.svg"),
        root.join("outside-paint.svg"),
    ] {
        fs::write(path, paint)?;
    }
    for path in [
        assets.join("style.css"),
        assets.join("style#%?.css"),
        root.join("outside.css"),
    ] {
        fs::write(path, ".mark { fill:#d52f59; }")?;
    }
    fs::create_dir(assets.join("sub"))?;
    fs::write(assets.join("sub/nested.css"), "@import \"../style.css\";")?;
    fs::write(
        assets.join("unsafe.css"),
        format!("@import \"{}/outside.css\";", root.display()),
    )?;
    let reserved = "owned#%? &é.png";
    png(&workspace.join(reserved), 16, marker)?;
    fs::write(
        assets.join("leaf#%?.svg"),
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><image href=\"../{}\" width=\"16\" height=\"16\"/></svg>",
            quote(reserved)
        ),
    )?;
    let outside_hash = hash(&outside)?;
    let inside_hash = hash(&inside)?;
    let mut command = isolated(&binary, &workspace);
    command
        .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
        .env("INKSCAPE_MCP_TOOL_PROFILE", "full")
        .env("INKSCAPE_MCP_ENGINE_MODE", mode);
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let mut observations = vec![];
    let result = (|| -> Result<()> {
        wire.initialize()?;
        let cases = [
            ("outside-absolute", outside.display().to_string(), true),
            (
                "outside-file-uri",
                format!("file://{}", quote(outside.to_str().unwrap())),
                true,
            ),
            (
                "outside-symlink",
                workspace.join("owned-link.png").display().to_string(),
                true,
            ),
            ("outside-nested-svg", "assets/unsafe.svg".into(), true),
            ("inside-absolute", inside.display().to_string(), false),
            ("inside-relative", "owned-inside.png".into(), false),
            ("inside-nested-svg", "assets/leaf.svg".into(), false),
            ("outside-feimage", outside.display().to_string(), true),
            (
                "outside-use",
                format!("{}/outside.svg#marker", root.display()),
                true,
            ),
            ("inside-feimage", "owned-inside.png".into(), false),
            ("inside-use", "assets/shape.svg#marker".into(), false),
            ("inside-css", "assets/paint.svg#paint".into(), false),
            (
                "outside-css",
                format!("{}/outside-paint.svg#paint", root.display()),
                true,
            ),
            ("inside-import", "assets/style.css".into(), false),
            (
                "inside-nested-import",
                "assets/sub/nested.css".into(),
                false,
            ),
            (
                "outside-import",
                format!("{}/outside.css", root.display()),
                true,
            ),
            ("outside-nested-import", "assets/unsafe.css".into(), true),
            ("inside-reserved-image", quote(reserved), false),
            (
                "inside-reserved-use",
                format!("{}#marker", quote("assets/shape#%?.svg")),
                false,
            ),
            (
                "inside-reserved-css",
                format!("{}#paint", quote("assets/paint#%?.svg")),
                false,
            ),
            (
                "inside-reserved-import",
                quote("assets/style#%?.css"),
                false,
            ),
            (
                "inside-reserved-nested-svg",
                quote("assets/leaf#%?.svg"),
                false,
            ),
        ];
        for (name, href, outside_reference) in cases {
            let body = if name.ends_with("-import") {
                format!(
                    "<style>@import &quot;{href}&quot;;</style><rect class=\"mark\" width=\"16\" height=\"16\" fill=\"blue\"/>"
                )
            } else if name.ends_with("-css") {
                format!("<rect width=\"16\" height=\"16\" style=\"fill:url({href})\"/>")
            } else if name.ends_with("-feimage") {
                format!(
                    "<defs><filter id=\"f\" x=\"0\" y=\"0\" width=\"1\" height=\"1\"><feImage xlink:href=\"{href}\"/></filter></defs><rect width=\"16\" height=\"16\" filter=\"url(#f)\"/>"
                )
            } else if name.ends_with("-use") {
                format!("<use xlink:href=\"{href}\"/>")
            } else {
                format!("<image xlink:href=\"{href}\" width=\"16\" height=\"16\"/>")
            };
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"16\" height=\"16\">{body}</svg>"
            );
            fs::write(workspace.join(format!("{name}.svg")), &svg)?;
            let doc =
                data(wire.call("open_document", json!({"path":format!("{name}.svg")}))?)?["doc_id"]
                    .clone();
            let before = inventory(&workspace)?;
            let reply = wire.call(
                "render_preview",
                json!({"doc_id":doc,"width_px":16,"inline":false}),
            )?;
            let refused = reply["result"]["isError"] == true;
            if outside_reference {
                ensure(
                    refused && before == inventory(&workspace)?,
                    format!("outside asset not refused before writes: {name}"),
                )?;
                observations.push(json!({"case":name,"outside_reference":true,"refused":true,"refusal_preserved_workspace":true}));
                continue;
            }
            let preview = data(reply)?;
            let path = workspace.join(
                preview["artifact_path"]
                    .as_str()
                    .ok_or("preview path missing")?,
            );
            let bytes = read(&path, 16 * 1024 * 1024)?;
            fs::write(out.join(format!("{name}.png")), &bytes)?;
            png_pixel(&bytes, 8, 8, marker)?;
            let mut args = json!({"doc_id":doc,"format":"svg","inline":false});
            if name.contains("reserved") {
                args["out_dir"] = json!("exports#%? &é");
            }
            let exported = data(wire.call("export_document", args)?)?;
            let path = workspace.join(
                exported["artifact_path"]
                    .as_str()
                    .ok_or("export path missing")?,
            );
            let bytes = read(&path, 16 * 1024 * 1024)?;
            let text = std::str::from_utf8(&bytes)?;
            ensure(
                !text.contains("imcp-assets-") && !text.contains("data:image"),
                "staged asset path leaked/artwork embedded",
            )?;
            fs::write(out.join(format!("{name}-export.svg")), &bytes)?;
            let reopened = data(
                wire.call("open_document", json!({"path":exported["artifact_path"]}))?,
            )?["doc_id"]
                .clone();
            let preview = data(wire.call(
                "render_preview",
                json!({"doc_id":reopened,"width_px":16,"inline":false}),
            )?)?;
            png_pixel(
                &read(
                    &workspace.join(preview["artifact_path"].as_str().ok_or("preview missing")?),
                    16 * 1024 * 1024,
                )?,
                8,
                8,
                marker,
            )?;
            ensure(
                fs::read_to_string(workspace.join(format!("{name}.svg")))? == svg,
                "original SVG changed",
            )?;
            observations.push(json!({"case":name,"outside_reference":false,"marker_rendered":true,"svg_export_links_restored":true,"svg_export_reopened_and_rendered":true}));
        }
        ensure(
            hash(&outside)? == outside_hash
                && hash(&inside)? == inside_hash
                && !workspace.join("absent-session").exists(),
            "original assets changed/GUI created",
        )?;
        Ok(())
    })();
    wire.save_trace(&out.join("server.trace.json"))?;
    write_json(
        &out.join("probe.json"),
        &json!({"probe_completed":result.is_ok(),"isolation_passed":result.is_ok(),"engine_mode":mode,"observations":observations,"binary_sha256":hash(&binary)?,"scope":"Owned image/use/feImage/CSS URLs/imports, nested assets and SVG export/reopen; no network/GUI/user assets; not xml:base/all routes"}),
    )?;
    result?;
    println!("Renderer: 22 asset cases, export/reopen and isolation passed");
    Ok(())
}
