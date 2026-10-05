use crate::{
    common::*,
    wire::{Wire, data, isolated},
};
use serde_json::json;
use std::{
    fs,
    io::{BufRead, Write},
    time::{Duration, Instant},
};
pub fn engine_fixture() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|s| s == "--version") {
        println!("Inkscape 1.4");
        return;
    }
    if args.iter().any(|s| s == "--action-list") {
        return;
    }
    let slow = || {
        fs::write(
            std::env::var_os("IMCP_CANCEL_READY").unwrap(),
            std::process::id().to_string(),
        )
        .unwrap();
        std::thread::sleep(Duration::from_secs(60));
    };
    if args.iter().any(|s| s == "--shell") {
        print!("> ");
        std::io::stdout().flush().unwrap();
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();
            if line.contains("export-do") {
                slow();
            }
            print!("> ");
            std::io::stdout().flush().unwrap();
        }
    } else {
        slow();
    }
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let mut evidence = vec![];
    for mode in ["per_call", "shell"] {
        let root = tempfile::tempdir()?;
        let root = root.path().canonicalize()?;
        let vendor = root.join("vendor");
        fs::create_dir(&vendor)?;
        copy(&std::env::current_exe()?, &vendor.join("inkscape"))?;
        let ready = root.join("ready");
        let mut command = isolated(&binary, &root);
        command
            .env("PATH", &vendor)
            .env("INKSCAPE_MCP_TOOLING_FIXTURE", "cancel-engine")
            .env("IMCP_CANCEL_READY", &ready)
            .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
            .env("INKSCAPE_MCP_ENGINE_MODE", mode)
            .env("INKSCAPE_MCP_PROCESS_TIMEOUT_S", "60");
        let mut wire = Wire::spawn(&mut command, &out.join(format!("{mode}.log")))?;
        let result = (|| -> Result<()> {
            wire.initialize()?;
            let doc =
                data(wire.call("create_document", json!({"width":32,"height":32}))?)?["doc_id"]
                    .as_str()
                    .ok_or("doc missing")?
                    .to_string();
            let working = root
                .join(".inkscape-mcp/documents")
                .join(&doc)
                .join("working/document.svg");
            let before = fs::read(&working)?;
            wire.send(&json!({"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"render_preview","arguments":{"doc_id":doc}}}))?;
            let start = Instant::now();
            while !ready.exists() {
                ensure(
                    start.elapsed() < Duration::from_secs(5),
                    "headless engine did not start",
                )?;
                std::thread::sleep(Duration::from_millis(10));
            }
            let pid: i32 = fs::read_to_string(&ready)?.parse()?;
            ensure(pid > 0, "invalid owned engine PID")?;
            let start = Instant::now();
            wire.send(&json!({"jsonrpc":"2.0","id":101,"method":"tools/list","params":{}}))?;
            wire.send(&json!({"jsonrpc":"2.0","id":102,"method":"resources/read","params":{"uri":"inkscape://workspace"}}))?;
            let mut replies = std::collections::BTreeMap::new();
            while !(replies.contains_key(&101) && replies.contains_key(&102)) {
                let reply = wire.receive(Duration::from_secs(2).saturating_sub(start.elapsed()))?;
                replies.insert(reply["id"].as_u64().ok_or("reply ID missing")?, reply);
            }
            ensure(
                start.elapsed() < Duration::from_secs(2),
                "discovery/workspace blocked by render",
            )?;
            wire.send(&json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":100,"reason":"acceptance"}}))?;
            wire.send(&json!({"jsonrpc":"2.0","id":103,"method":"tools/call","params":{"name":"create_document","arguments":{"width":16,"height":16}}}))?;
            let start = Instant::now();
            while !replies.contains_key(&103) {
                let reply = wire.receive(Duration::from_secs(3).saturating_sub(start.elapsed()))?;
                replies.insert(reply["id"].as_u64().ok_or("reply ID missing")?, reply);
            }
            ensure(
                replies[&103]["result"]["isError"] != true && replies[&103].get("result").is_some(),
                "gate not released after cancellation",
            )?;
            ensure(
                unsafe { libc::kill(pid, 0) } == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH),
                "cancelled owned engine survived",
            )?;
            ensure(
                fs::read(&working)? == before
                    && !walk(&root.join(".inkscape-mcp"))?
                        .iter()
                        .any(|p| p.extension().is_some_and(|e| e == "png")),
                "cancelled render changed copy/published PNG",
            )?;
            write_json(
                &out.join(format!("{mode}.replies.json")),
                &json!(replies.into_values().collect::<Vec<_>>()),
            )?;
            Ok(())
        })();
        wire.save_trace(&out.join(format!("{mode}.trace.json")))?;
        result?;
        evidence.push(json!({"engine":mode,"responsive":true,"cancelled_child_reaped":true,"followup_edit":true}));
    }
    write_json(&out.join("comparison.json"), &json!(evidence))?;
    println!("Responsive discovery and cancellation passed for per-call/shell native fixtures");
    Ok(())
}
