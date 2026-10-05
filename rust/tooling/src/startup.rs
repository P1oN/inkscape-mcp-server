use crate::{
    common::*,
    wire::{Wire, data, isolated},
};
use serde_json::json;
use std::{
    fs,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output", "--sessions", "--parallel"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let sessions: usize = args
        .values
        .get("--sessions")
        .map(String::as_str)
        .unwrap_or("128")
        .parse()?;
    let parallel: usize = args
        .values
        .get("--parallel")
        .map(String::as_str)
        .unwrap_or("4")
        .parse()?;
    ensure(
        (1..=512).contains(&sessions) && (1..=8).contains(&parallel),
        "invalid startup stress bounds",
    )?;
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let results = Mutex::new(vec![]);
    let errors = Mutex::new(vec![]);
    let started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..parallel {
            scope.spawn(|| {
                while !failed.load(Ordering::Acquire) {
                    let index=next.fetch_add(1,Ordering::Relaxed); if index>=sessions {break}
                    let result=(|| -> Result<serde_json::Value> {
                        let root=tempfile::tempdir()?; let mut command=isolated(&binary,root.path());
                        command.env("INKSCAPE_MCP_LIVE_ENABLED","false").env("INKSCAPE_MCP_TOOL_PROFILE","core").env("INKSCAPE_MCP_TOOL_DESC","short");
                        let mut wire=Wire::spawn(&mut command,&out.join(format!("session-{index}.stderr.log")))?; wire.timeout=Duration::from_secs(5);
                        let result=(|| -> Result<()> {
                            wire.initialize()?;
                            ensure(data(wire.call("create_document",json!({"width":32,"height":32}))?)?["doc_id"].as_str().is_some_and(|s|!s.is_empty()),"first tool request did not create document")?;
                            for method in ["tools/list","resources/list","resources/templates/list","prompts/list"] {ensure(wire.request(method,None)?.get("result").is_some(),"discovery response missing")?;} Ok(())
                        })();
                        wire.save_trace(&out.join(format!("session-{index}.trace.json")))?;
                        result?;
                        Ok(json!({"session":index,"roundtrips_ns":wire.trace.iter().map(|item|item["roundtrip_ns"].clone()).collect::<Vec<_>>()}))
                    })();
                    match result {
                        Ok(result)=>results.lock().unwrap().push(result),
                        Err(error)=>{failed.store(true,Ordering::Release); errors.lock().unwrap().push(error.to_string());break;},
                    }
                }
            });
        }
    });
    let mut results = results.into_inner()?;
    results.sort_by_key(|v| v["session"].as_u64().unwrap());
    let errors = errors.into_inner()?;
    let passed = results.len() == sessions && errors.is_empty();
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":passed,"sessions":sessions,"parallel":parallel,"completed":results.len(),"elapsed_seconds":started.elapsed().as_secs_f64(),"binary_sha256":hash(&binary)?,"results":results,"errors":errors,"native_GUI":false}),
    )?;
    ensure(
        passed,
        "startup stress failed; see comparison and pending traces",
    )?;
    println!("Startup: {sessions} fresh sessions, {parallel} workers passed");
    Ok(())
}
