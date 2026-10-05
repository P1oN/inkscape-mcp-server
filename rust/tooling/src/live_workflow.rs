//! Owned snapshot helper + real STDIO/CLI evidence; never launches an Inkscape GUI.
use crate::{
    acceptance::png_pixel,
    common::*,
    wire::{Wire, data, isolated},
};
use serde_json::json;
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
const SVG: &str = include_str!("../../tests/fixtures/live-resources.svg");
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn retain(workspace: &Path, out: &Path, path: &str, name: &str) -> Result<Vec<u8>> {
    let bytes = read(&workspace.join(path), 16 * 1024 * 1024)?;
    fs::write(out.join(name), &bytes)?;
    Ok(bytes)
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--helper", "--output"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let helper = args.required("--helper")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let workspace = root.path().canonicalize()?;
    let input = workspace.join("resources.svg");
    fs::write(&input, SVG)?;
    fs::write(out.join("resources.svg"), SVG)?;
    let rendezvous = workspace.join("rendezvous.json");
    let mut child = Owned(
        Command::new(&helper)
            .arg("--id=caption")
            .arg(&input)
            .env("INKSCAPE_MCP_LIVE_RENDEZVOUS", &rendezvous)
            .stdout(Stdio::piped())
            .stderr(fs::File::create(out.join("helper.stderr.log"))?)
            .spawn()?,
    );
    let start = Instant::now();
    while !rendezvous.exists() {
        ensure(
            child.0.try_wait()?.is_none(),
            "helper exited before readiness",
        )?;
        ensure(
            start.elapsed() < Duration::from_secs(10),
            "helper readiness timeout",
        )?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut command = isolated(&binary, &workspace);
    command
        .env("INKSCAPE_MCP_LIVE_ENABLED", "true")
        .env("INKSCAPE_MCP_TOOL_PROFILE", "full")
        .env("INKSCAPE_MCP_LIVE_RENDEZVOUS", &rendezvous);
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let result = (|| -> Result<()> {
        wire.initialize()?;
        data(wire.call("live_connect", json!({}))?)?;
        let report = data(wire.call(
            "live_inspect_objects",
            json!({"object_ids":["masked-paint","panel","leaf-clone","petals"]}),
        )?)?;
        ensure(
            report["objects"][0]["computed_style"]["ancestor_effects"][0]["properties"]["mask"]["value"]
                == "url(#half-mask)",
            "ancestor mask lost",
        )?;
        for (i, id) in [
            (0, "round-clip"),
            (0, "half-mask"),
            (1, "linked-dots"),
            (1, "dots"),
            (2, "leaf-source"),
            (3, "linked-gradient"),
            (3, "petal-gradient"),
        ] {
            ensure(
                report["objects"][i]["computed_style"]["relationships"]
                    .as_array()
                    .is_some_and(|rs| {
                        rs.iter()
                            .any(|r| r["target_id"] == id && r["resolved"] == true)
                    }),
                format!("resource relationship missing: {id}"),
            )?;
        }
        write_json(&out.join("styles.json"), &report)?;
        let found = data(wire.call("live_find_objects", json!({"limit":100}))?)?;
        for id in ["masked-paint", "panel", "leaf-clone", "petals"] {
            let obj = found["objects"]
                .as_array()
                .and_then(|a| a.iter().find(|v| v["id"] == id))
                .ok_or("identity missing from discovery")?;
            ensure(
                obj["bbox"]["width"].as_f64().is_some_and(|n| n > 0.),
                "engine bounds missing",
            )?;
            let preview = data(wire.call(
                "live_preview_object",
                json!({"object_id":id,"expected_fingerprint":found["fingerprint"],"width":160}),
            )?)?;
            let bytes = retain(
                &workspace,
                &out,
                preview["artifact_path"].as_str().ok_or("preview missing")?,
                &format!("{id}.png"),
            )?;
            if id == "masked-paint" {
                png_pixel(&bytes, 30, 80, [217, 93, 114, 255])?;
                png_pixel(&bytes, 140, 80, [255, 255, 255, 0])?;
            }
        }
        write_json(&out.join("discovery.json"), &found)?;
        let edits =
            json!([{"op":"style","fill":"#d95d72"},{"op":"text","text":"Reviewed resource study"}]);
        let plan = data(wire.call("live_change_package", json!({"edits":edits}))?)?;
        ensure(
            plan["dry_run"] == true
                && plan["changed"] == true
                && plan["expected_selection"] == json!(["caption"]),
            "package plan failed",
        )?;
        for phase in ["before", "after"] {
            retain(
                &workspace,
                &out,
                plan[format!("preview_{phase}")]
                    .as_str()
                    .ok_or("package preview missing")?,
                &format!("package-{phase}.png"),
            )?;
        }
        for phase in ["before", "after"] {
            let uri = plan["preview_artifacts"][phase]["uri"]
                .as_str()
                .ok_or("portable package preview missing")?;
            let resource = wire.request("resources/read", Some(json!({"uri":uri})))?;
            ensure(
                resource.get("error").is_none()
                    && resource["result"]["contents"][0]["blob"].is_string(),
                "package preview resource unreadable",
            )?;
        }
        write_json(&out.join("package-plan.json"), &plan)?;
        let make_apply = || json!({"edits":edits,"dry_run":false,"expected_document":plan["expected_document"],"expected_selection":plan["expected_selection"],"expected_fingerprint":plan["expected_fingerprint"],"expected_package_digest":plan["expected_package_digest"],"approval_token":"synthetic-acceptance"});
        let mut stale = make_apply();
        stale["expected_fingerprint"] = json!("stale");
        ensure(
            wire.call("live_change_package", stale)?["result"]["isError"] == true,
            "stale review accepted",
        )?;
        let mut changed = make_apply();
        changed["edits"] = json!([{"op":"style","fill":"blue"}]);
        ensure(
            wire.call("live_change_package", changed)?["result"]["isError"] == true,
            "changed package accepted",
        )?;
        let refused = wire.call("live_change_package", make_apply())?;
        ensure(
            refused["result"]["isError"] == true,
            "socket applied unsupported package",
        )?;
        let mut unapproved = make_apply();
        unapproved["approval_token"] = json!("");
        ensure(
            wire.call("live_change_package", unapproved)?["result"]["isError"] == true,
            "missing approval accepted",
        )?;
        let after = data(wire.call("live_find_objects", json!({"limit":100}))?)?;
        ensure(
            after["fingerprint"] == found["fingerprint"],
            "read-only review/refusal changed content",
        )?;
        data(wire.call("live_disconnect", json!({}))?)?;
        ensure(fs::read(&input)? == SVG.as_bytes(), "original changed")?;
        Ok(())
    })();
    wire.save_trace(&out.join("trace.json"))?;
    wire.close();
    write_json(
        &out.join("result.json"),
        &json!({"passed":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),"binary_sha256":hash(&binary)?,"helper_sha256":hash(&helper)?,"fixture_sha256":sha(SVG.as_bytes()),"scope":"Real STDIO, native snapshot helper and CLI resources/previews. No GUI or artist pilot; native package publication/Undo/Redo are separate."}),
    )?;
    result
}
