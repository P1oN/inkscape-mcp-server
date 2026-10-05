use crate::common::*;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    os::fd::AsRawFd,
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
/// Sequential real-STDIO probe. Deadline failures preserve completed and pending messages.
pub struct Wire {
    pub child: Child,
    input: Option<ChildStdin>,
    receiver: mpsc::Receiver<std::result::Result<Value, String>>,
    reader: Option<thread::JoinHandle<()>>,
    stop_reader: Arc<AtomicBool>,
    pub trace: Vec<Value>,
    pub pending: Option<Value>,
    sequence: u64,
    pub timeout: Duration,
}
impl Wire {
    pub fn spawn(command: &mut Command, log: &Path) -> Result<Self> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(fs::File::create(log)?))
            .spawn()?;
        let input = child.stdin.take();
        let mut stdout = child.stdout.take().ok_or("stdout unavailable")?;
        let (sender, receiver) = mpsc::sync_channel(128);
        let stop_reader = Arc::new(AtomicBool::new(false));
        let stop = stop_reader.clone();
        let reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            loop {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                let mut fd = libc::pollfd {
                    fd: stdout.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                // A bounded poll lets cleanup stop the reader even if a peer's descendant
                // retains stdout or sends an unfinished frame. Never terminate descendants.
                let ready = unsafe { libc::poll(&mut fd, 1, 100) };
                if ready == 0 {
                    continue;
                }
                if ready < 0 {
                    let _ = sender.try_send(Err("reader_error: poll failed".into()));
                    break;
                }
                let mut chunk = [0u8; 4096];
                match stdout.read(&mut chunk) {
                    Ok(0) => {
                        let _ = sender.try_send(Err("eof".into()));
                        break;
                    }
                    Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                    Err(e) => {
                        let _ = sender.try_send(Err(format!("reader_error: {e}")));
                        break;
                    }
                }
                while let Some(end) = bytes.iter().position(|b| *b == b'\n') {
                    if end >= 64 * 1024 * 1024 {
                        let _ = sender.try_send(Err("stdout frame cap exceeded".into()));
                        return;
                    }
                    let message = serde_json::from_slice(&bytes[..=end])
                        .map_err(|e| format!("reader_error: {e}"));
                    bytes.drain(..=end);
                    let failed = message.is_err();
                    let mut pending = message;
                    loop {
                        match sender.try_send(pending) {
                            Ok(()) => break,
                            Err(mpsc::TrySendError::Full(message)) => {
                                if stop.load(Ordering::Acquire) {
                                    return;
                                }
                                pending = message;
                                thread::sleep(Duration::from_millis(5));
                            }
                            Err(mpsc::TrySendError::Disconnected(_)) => return,
                        }
                    }
                    if failed {
                        return;
                    }
                }
                if bytes.len() > 64 * 1024 * 1024 {
                    let _ = sender.try_send(Err("stdout frame cap exceeded".into()));
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            receiver,
            reader: Some(reader),
            stop_reader,
            trace: vec![],
            pending: None,
            sequence: 0,
            timeout: Duration::from_secs(90),
        })
    }
    pub fn receive(&mut self, timeout: Duration) -> Result<Value> {
        self.receiver
            .recv_timeout(timeout)
            .map_err(|error| format!("MCP reply wait: {error}"))?
            .map_err(Into::into)
    }
    pub fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
        let input = self.input.as_mut().ok_or("stdin closed")?;
        input.write_all(bytes)?;
        input.flush()?;
        Ok(())
    }
    pub fn send(&mut self, message: &Value) -> Result<()> {
        let input = self.input.as_mut().ok_or("stdin closed")?;
        writeln!(input, "{}", serde_json::to_string(message)?)?;
        input.flush()?;
        Ok(())
    }
    pub fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        self.sequence += 1;
        let mut message = json!({"jsonrpc":"2.0","id":self.sequence,"method":method});
        if let Some(params) = params {
            message["params"] = params;
        }
        self.pending = Some(message.clone());
        let start = Instant::now();
        self.send(&message)?;
        loop {
            let received = self
                .receiver
                .recv_timeout(self.timeout.saturating_sub(start.elapsed()));
            let reply=match received {
                Ok(Ok(reply))=>reply,
                Ok(Err(error))=>return Err(format!("MCP {error}: method={method}, id={}, pid={}",self.sequence,self.child.id()).into()),
                Err(_)=>return Err(format!("MCP response timed out after {:?}: method={method}, id={}, pid={}, exit_code={:?}",self.timeout,self.sequence,self.child.id(),self.child.try_wait()?).into()),
            };
            if reply["id"] == self.sequence {
                self.trace.push(json!({"request":message,"response":reply,"roundtrip_ns":start.elapsed().as_nanos() as u64}));
                self.pending = None;
                return Ok(reply);
            }
            ensure(start.elapsed() < self.timeout, "MCP deadline exceeded")?;
        }
    }
    pub fn initialize(&mut self) -> Result<Value> {
        let reply=self.request("initialize",Some(json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"migration-probe","version":"1"}})))?;
        self.send(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
        Ok(reply)
    }
    pub fn call(&mut self, name: &str, args: Value) -> Result<Value> {
        self.request("tools/call", Some(json!({"name":name,"arguments":args})))
    }
    pub fn save_trace(&mut self, path: &Path) -> Result<()> {
        write_json(
            path,
            &json!({"completed":self.trace,"pending":self.pending,"exit_code":self.child.try_wait()?.and_then(|s|s.code())}),
        )
    }
    pub fn close(&mut self) {
        self.input.take();
        let start = Instant::now();
        while !self.child.try_wait().ok().flatten().is_some() {
            if start.elapsed() > Duration::from_secs(10) {
                let _ = self.child.kill();
                let _ = self.child.wait();
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Drain a bounded reader queue before joining; no blocked sender can survive.
        self.stop_reader.store(true, Ordering::Release);
        while self
            .reader
            .as_ref()
            .is_some_and(|reader| !reader.is_finished())
        {
            while self.receiver.try_recv().is_ok() {}
            thread::sleep(Duration::from_millis(1));
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
impl Drop for Wire {
    fn drop(&mut self) {
        self.close();
    }
}
pub fn data(reply: Value) -> Result<Value> {
    ensure(
        reply.get("error").is_none() && reply["result"]["isError"] != true,
        format!("tool error: {reply}"),
    )?;
    Ok(reply["result"]["structuredContent"].clone())
}
const METHODS: [&str; 4] = [
    "tools/list",
    "resources/list",
    "resources/templates/list",
    "prompts/list",
];
fn surface(wire: &mut Wire) -> Result<Value> {
    let mut contract = json!({"initialize":wire.initialize()?["result"]});
    for method in METHODS {
        let reply = wire.request(method, None)?;
        ensure(reply.get("error").is_none(), "discovery failed")?;
        contract[method] = reply["result"].clone();
    }
    Ok(contract)
}
pub fn isolated(binary: &Path, root: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env("INKSCAPE_MCP_WORKSPACE_ROOTS", root)
        .env("INKSCAPE_MCP_MANAGED_DIR", root.join("absent-session"))
        .env(
            "INKSCAPE_MCP_LIVE_RENDEZVOUS",
            root.join("absent-rendezvous"),
        )
        .env("SENTRY_DSN", "")
        .env("SENTRY_TRACES_SAMPLE_RATE", "0");
    command
}
pub fn discovery(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output", "--matrix", "--contracts"])?;
    let binary = args
        .path("--binary", "rust/target/release/inkscape-mcp-rust")
        .canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let mut count = 0;
    for live in ["true", "false"] {
        for raw in ["true", "false"] {
            for profile in ["full", "core"] {
                for desc in ["full", "short"] {
                    if !args.flag("--matrix")
                        && (live, raw, profile, desc) != ("true", "true", "full", "full")
                    {
                        continue;
                    }
                    let key = format!("live-{live}_raw-{raw}_{profile}_{desc}");
                    let root = tempfile::tempdir()?;
                    let mut command = isolated(&binary, root.path());
                    command
                        .env("INKSCAPE_MCP_LIVE_ENABLED", live)
                        .env("INKSCAPE_MCP_RAW_ACTION_ENABLED", raw)
                        .env("INKSCAPE_MCP_TOOL_PROFILE", profile)
                        .env("INKSCAPE_MCP_TOOL_DESC", desc);
                    let mut wire =
                        Wire::spawn(&mut command, &out.join(format!("{key}.stderr.log")))?;
                    let result = surface(&mut wire);
                    wire.save_trace(&out.join(format!("{key}.trace.json")))?;
                    let contract = result?;
                    write_json(&out.join(format!("{key}.json")), &contract)?;
                    let reference = crate::common::json(
                        &args
                            .path("--contracts", "migration/contracts")
                            .join(format!("{key}.json")),
                    )?;
                    ensure(
                        contract == reference,
                        format!("frozen discovery differs: {key}"),
                    )?;
                    ensure(
                        !root.path().join("absent-session").exists()
                            && !root.path().join("absent-rendezvous").exists(),
                        "discovery mutated native session",
                    )?;
                    println!("{key}: exact contract");
                    count += 1;
                }
            }
        }
    }
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"configurations":count}),
    )
}
pub fn manifests(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output"])?;
    let binary = args
        .path("--binary", "rust/target/release/inkscape-mcp-rust")
        .canonicalize()?;
    let out = args.path("--output", ".");
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let mut command = isolated(&binary, root.path());
    for (key, value) in [
        ("LIVE_ENABLED", "true"),
        ("RAW_ACTION_ENABLED", "true"),
        ("TOOL_PROFILE", "full"),
        ("TOOL_DESC", "full"),
    ] {
        command.env(format!("INKSCAPE_MCP_{key}"), value);
    }
    let mut wire = Wire::spawn(&mut command, &root.path().join("stderr.log"))?;
    let mut contract = surface(&mut wire)?;
    // Full manifest historically includes the entire initialize reply.
    contract["initialize"] = wire.trace[0]["response"].clone();
    let contract = json!({"initialize":contract["initialize"],"tools/list":contract["tools/list"],"prompts/list":contract["prompts/list"],"resources/list":contract["resources/list"],"resources/templates/list":contract["resources/templates/list"]});
    ensure(
        !root.path().join("absent-session").exists(),
        "startup mutated session",
    )?;
    let header = "# inkscape-mcp: Rust STDIO server\n\nRun ./setup.sh then ./run-mcp.sh. See README.md and docs/agent-usage-guide.md.\nNative Rust runtime and build tools; ready packages contain no Python runtime.\n";
    let mut index = format!("{header}\n## Tools\n");
    let mut tools = contract["tools/list"]["tools"]
        .as_array()
        .ok_or("tools missing")?
        .clone();
    tools.sort_by_key(|v| v["name"].as_str().unwrap().to_string());
    for tool in tools {
        index.push_str(&format!(
            "- {}: {}\n",
            tool["name"].as_str().unwrap(),
            tool["description"]
                .as_str()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
        ));
    }
    for (title, method, key, uri) in [
        ("Prompts", "prompts/list", "prompts", "uri"),
        ("Resources", "resources/list", "resources", "uri"),
        (
            "Resource templates",
            "resources/templates/list",
            "resourceTemplates",
            "uriTemplate",
        ),
    ] {
        index.push_str(&format!("\n## {title}\n"));
        for item in contract[method][key].as_array().ok_or("surface missing")? {
            let row = format!(
                "- {}: {}",
                item["name"].as_str().unwrap(),
                item[uri].as_str().unwrap_or("")
            );
            index.push_str(row.trim_end());
            index.push('\n');
        }
    }
    fs::write(out.join("llms.txt"), index)?;
    fs::write(
        out.join("llms-full.txt"),
        format!(
            "{header}\n## Actual MCP initialization and surface\n\n{}\n",
            serde_json::to_string_pretty(&contract)?
        ),
    )?;
    println!("Generated live MCP manifests");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_discovery_retains_initialize_and_pending_list_and_inherited_stdout_cannot_hang_cleanup()
     {
        let root = tempfile::tempdir().unwrap();
        let mut wire=Wire::spawn(Command::new("/bin/bash").args(["-c", "read line; printf '%s\\n' '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}'; read line; read line; read line"]),&root.path().join("stderr")).unwrap();
        wire.timeout = Duration::from_millis(100);
        wire.initialize().unwrap();
        assert!(wire.request("tools/list", None).is_err());
        wire.save_trace(&root.path().join("trace.json")).unwrap();
        let trace = crate::common::json(&root.path().join("trace.json")).unwrap();
        assert_eq!(trace["completed"][0]["request"]["method"], "initialize");
        assert_eq!(trace["pending"]["method"], "tools/list");
        wire.close();
        let mut wire = Wire::spawn(
            Command::new("/bin/bash").args(["-c", "/bin/sleep 2 & printf '{'; exit 0"]),
            &root.path().join("inherited"),
        )
        .unwrap();
        wire.timeout = Duration::from_millis(100);
        assert!(wire.request("tools/list", None).is_err());
        let started = Instant::now();
        wire.close();
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(wire.reader.is_none());
    }
    #[test]
    fn failures_keep_pending_and_completed_and_close_owned_reader() {
        let root = tempfile::tempdir().unwrap();
        for mode in ["reply", "silent", "invalid", "eof"] {
            let script = match mode {
                "reply" => {
                    "read line; printf '%s\\n' '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}'; read line"
                }
                "silent" => "read line; read line",
                "invalid" => "read line; printf '\\377\\n'; read line",
                _ => "read line; exit 0",
            };
            let mut wire = Wire::spawn(
                Command::new("/bin/bash").args(["-c", script]),
                &root.path().join(format!("{mode}.log")),
            )
            .unwrap();
            wire.timeout = Duration::from_millis(100);
            let result = wire.request("tools/list", None);
            assert_eq!(result.is_ok(), mode == "reply");
            if mode == "reply" {
                assert!(wire.pending.is_none());
                assert_eq!(wire.trace.len(), 1);
            } else {
                assert_eq!(wire.pending.as_ref().unwrap()["method"], "tools/list");
            }
            wire.save_trace(&root.path().join(format!("{mode}.json")))
                .unwrap();
            wire.close();
            assert!(wire.child.try_wait().unwrap().is_some());
            assert!(wire.reader.is_none());
        }
    }
}
