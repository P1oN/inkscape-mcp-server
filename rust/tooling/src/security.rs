use crate::{
    acceptance::inventory,
    common::*,
    wire::{Wire, data, isolated},
};
use serde_json::{Value, json};
use std::{
    fs,
    time::{Duration, Instant},
};
fn refused(reply: &Value) -> bool {
    reply["result"]["isError"] == true
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let workspace = root.path().join("workspace");
    fs::create_dir(&workspace)?;
    let cases = [
        ("malformed", "<svg><g></svg>".into()),
        ("namespace", "<x:svg/>".into()),
        (
            "depth",
            format!("<svg>{}{}</svg>", "<g>".repeat(300), "</g>".repeat(300)),
        ),
    ];
    for (name, content) in &cases {
        fs::write(workspace.join(format!("{name}.svg")), content)?;
    }
    let mut command = isolated(&binary, &workspace);
    command
        .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
        .env("INKSCAPE_MCP_TOOL_PROFILE", "full");
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let result = (|| -> Result<usize> {
        wire.initialize()?;
        let mut checks = 0;
        for (name, _) in &cases {
            let before = inventory(&workspace)?;
            ensure(
                refused(&wire.call("open_document", json!({"path":format!("{name}.svg")}))?),
                "unsafe XML accepted",
            )?;
            let after = inventory(&workspace)?;
            write_json(
                &out.join(format!("{name}-filesystem.json")),
                &json!({"before":before,"after":after}),
            )?;
            ensure(before == after, "refused XML open changed workspace")?;
            checks += 1;
        }
        for (label, tool, arguments) in [
            (
                "unknown-field",
                "create_document",
                json!({"width":10,"height":10,"shell":"x"}),
            ),
            (
                "invalid-number",
                "create_document",
                json!({"width":"NaN","height":10}),
            ),
            ("missing-path", "open_document", json!({})),
            (
                "container-cap",
                "set_fill",
                json!({"doc_id":"none","object_ids":vec![Value::Null;10001],"color":"red"}),
            ),
        ] {
            let before = inventory(&workspace)?;
            ensure(
                refused(&wire.call(tool, arguments)?),
                format!("invalid args accepted: {label}"),
            )?;
            ensure(
                before == inventory(&workspace)?,
                "invalid arguments changed workspace",
            )?;
            checks += 1;
        }
        let doc = data(wire.call("create_document", json!({"width":10,"height":10}))?)?["doc_id"]
            .as_str()
            .ok_or("doc missing")?
            .to_string();
        let resources = wire.request("resources/list", Some(json!({})))?["result"]["resources"]
            .as_array()
            .ok_or("resources missing")?
            .clone();
        ensure(resources.len() == 10, "static resource count drift")?;
        for item in resources {
            let reply = wire.request("resources/read", Some(json!({"uri":item["uri"]})))?;
            ensure(reply.get("error").is_none(), "static resource read failed")?;
            let contents = reply["result"]["contents"]
                .as_array()
                .ok_or("resource contents missing")?;
            ensure(
                contents.len() == 1
                    && contents[0]["uri"] == item["uri"]
                    && contents[0]["mimeType"] == "application/json",
                "resource envelope URI/MIME drift",
            )?;
            let _: Value = serde_json::from_str(
                contents[0]["text"]
                    .as_str()
                    .ok_or("resource text missing")?,
            )?;
            checks += 1;
        }
        for leaf in [
            "summary", "tree", "layers", "objects", "styles", "fonts", "assets",
        ] {
            let uri = format!("inkscape://document/{doc}/{leaf}");
            let reply = wire.request("resources/read", Some(json!({"uri":uri})))?;
            ensure(
                reply.get("error").is_none() && reply["result"]["contents"][0]["uri"] == uri,
                "document resource URI drift",
            )?;
            let value: Value = serde_json::from_str(
                reply["result"]["contents"][0]["text"]
                    .as_str()
                    .ok_or("resource text missing")?,
            )?;
            ensure(value["doc_id"] == doc, "document resource ID drift")?;
            checks += 1;
        }
        let prompts = wire.request("prompts/list", Some(json!({})))?["result"]["prompts"]
            .as_array()
            .ok_or("prompts missing")?
            .clone();
        ensure(prompts.len() == 7, "prompt count drift")?;
        for item in prompts {
            let mut arguments = serde_json::Map::new();
            if let Some(args) = item["arguments"].as_array() {
                for arg in args {
                    arguments.insert(
                        arg["name"]
                            .as_str()
                            .ok_or("prompt arg name missing")?
                            .into(),
                        json!("security audit goal"),
                    );
                }
            }
            let reply = wire.request(
                "prompts/get",
                Some(json!({"name":item["name"],"arguments":arguments})),
            )?;
            ensure(
                reply.get("error").is_none()
                    && reply["result"]["messages"].as_array().is_some_and(|a| {
                        !a.is_empty()
                            && a.iter().all(|m| {
                                m["content"]["text"]
                                    .as_str()
                                    .is_some_and(|s| !s.contains("__MIGRATION_GOAL_SLOT_"))
                            })
                    }),
                "prompt render failed/slot leaked",
            )?;
            checks += 1;
        }
        for uri in [
            format!("inkscape://document/{doc}/../../outside"),
            "inkscape://document/missing/tree".into(),
            "inkscape://unknown".into(),
        ] {
            ensure(
                wire.request("resources/read", Some(json!({"uri":uri})))?
                    .get("error")
                    .is_some(),
                "invalid resource URI accepted",
            )?;
            checks += 1;
        }
        ensure(
            !workspace.join("absent-session").exists(),
            "startup created GUI session",
        )?;
        for (name, content) in &cases {
            ensure(
                fs::read_to_string(workspace.join(format!("{name}.svg")))? == *content,
                "original changed",
            )?;
        }
        checks += 1;
        Ok(checks)
    })();
    wire.save_trace(&out.join("server.trace.json"))?;
    let checks = result?;
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"binary_sha256":hash(&binary)?,"checks":checks,"native_GUI":false,"scope":"Selected XML/argument refusal before writes; resources/prompts; not exhaustive security/race/native acceptance"}),
    )?;
    println!("{checks} Rust STDIO security checks passed");
    Ok(())
}
pub fn frames(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let mut command = isolated(&binary, root.path());
    command
        .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
        .env("INKSCAPE_MCP_MAX_REQUEST_BYTES", "4096");
    let mut wire = Wire::spawn(&mut command, &out.join("server.stderr.log"))?;
    let result = (|| -> Result<()> {
        wire.initialize()?;
        for _ in 0..3 {
            ensure(
                wire.request("ping", None)?.get("result").is_some(),
                "ordinary frame rejected",
            )?;
        }
        let before = inventory(root.path())?;
        wire.write_raw(
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":99,\"method\":\"tools/call\",\"params\":{{\"x\":\"{}",
                "a".repeat(5000)
            )
            .as_bytes(),
        )?;
        let start = Instant::now();
        while wire.child.try_wait()?.is_none() {
            ensure(
                start.elapsed() < Duration::from_secs(3),
                "unfinished oversized input waits without cap",
            )?;
            std::thread::sleep(Duration::from_millis(10));
        }
        ensure(
            before == inventory(root.path())?,
            "frame refusal changed workspace",
        )?;
        write_json(
            &out.join("comparison.json"),
            &json!({"passed":true,"unfinished_frame_refused_without_EOF":true,"ordinary_frames_accepted":3,"workspace_unchanged":true,"binary_sha256":hash(&binary)?}),
        )
    })();
    wire.save_trace(&out.join("server.trace.json"))?;
    result
}
