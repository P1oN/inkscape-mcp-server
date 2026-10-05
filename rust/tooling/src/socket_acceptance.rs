use crate::{
    acceptance::{environment, png_pixel},
    common::*,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Child, Command},
    time::{Duration, Instant},
};
const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="0 0 100 80">
<style>path {fill:#00ff00}</style><g id="g" transform="translate(30,20)"><rect id="r" x="2" y="3" width="20" height="10" style="fill:red;stroke:black;stroke-width:4" transform="translate(5,7)"/></g><text id="t" x="5" y="65">Hi</text><path id="p" d="M0,0 C0,20 20,20 20,0Z"/></svg>"##;
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
struct Socket {
    reader: BufReader<TcpStream>,
    token: String,
    trace: Vec<Value>,
}
impl Socket {
    fn receive(&mut self) -> Result<Value> {
        let mut bytes = vec![];
        (&mut self.reader)
            .take(64 * 1024 * 1024 + 1)
            .read_until(b'\n', &mut bytes)?;
        ensure(bytes.len() <= 64 * 1024 * 1024, "socket frame cap")?;
        Ok(serde_json::from_slice(&bytes)?)
    }
    fn request(&mut self, cmd: &str, params: Value) -> Result<Value> {
        writeln!(
            self.reader.get_mut(),
            "{}",
            json!({"v":5,"cmd":cmd,"token":self.token,"params":params})
        )?;
        let reply = self.receive()?;
        self.trace.push(json!({"command":cmd,"ok":reply["ok"]}));
        Ok(reply)
    }
}
fn exercise(
    binary: &Path,
    root: &Path,
    change: bool,
    unauthorized: bool,
    drawing: &str,
    bbox: Value,
) -> Result<Value> {
    let source = root.join("input.svg");
    fs::write(&source, drawing)?;
    let rendezvous = root.join("rendezvous.json");
    let stdout = root.join("helper.stdout");
    let stderr = root.join("helper.stderr");
    let env = environment(
        root,
        &binary
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("libexec/inkscape-mcp"),
    )?;
    let mut child = Owned(
        Command::new(binary)
            .arg("--id=r")
            .arg(&source)
            .env_clear()
            .envs(env.iter().map(|(k, v)| (k, v)))
            .env("INKSCAPE_MCP_LIVE_RENDEZVOUS", &rendezvous)
            .stdout(fs::File::create(&stdout)?)
            .stderr(fs::File::create(&stderr)?)
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
            "helper readiness timed out",
        )?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let info = crate::common::json(&rendezvous)?;
    ensure(
        info["pid"] == child.0.id()
            && info["protocol_version"] == 5
            && fs::metadata(&rendezvous)?.permissions().mode() & 0o777 == 0o600,
        "helper identity/permissions",
    )?;
    let conn = TcpStream::connect((
        "127.0.0.1",
        info["port"].as_u64().ok_or("port missing")? as u16,
    ))?;
    conn.set_read_timeout(Some(Duration::from_secs(40)))?;
    conn.set_write_timeout(Some(Duration::from_secs(40)))?;
    let mut socket = Socket {
        reader: BufReader::new(conn),
        token: if unauthorized {
            "wrong"
        } else {
            info["token"].as_str().ok_or("token missing")?
        }
        .into(),
        trace: vec![],
    };
    let hello = socket.request("hello", json!({}))?;
    if unauthorized {
        ensure(hello["ok"] == false, "invalid token accepted")?;
    } else {
        ensure(
            hello["ok"] == true
                && hello["result"]["capabilities"]
                    .as_array()
                    .is_some_and(|a| a.len() == 13),
            "wrong capabilities",
        )?;
        ensure(
            socket.request("get_selection", json!({}))?["result"]["object_ids"] == json!(["r"]),
            "wrong selection",
        )?;
        ensure(
            socket.request("inspect_selection", json!({}))?["result"]["objects"][0]["has_style"]
                == true,
            "style perception",
        )?;
        ensure(
            socket.request("get_active_document", json!({}))?["result"]["object_count"] == 4,
            "object count",
        )?;
        let before = socket.request("get_document_svg", json!({}))?["result"]["svg"].clone();
        let revision = socket.request("get_state_token", json!({}))?["result"]["revision"].clone();
        let scene = socket.request("get_scene", json!({}))?["result"].clone();
        ensure(
            scene["selection"][0]["bbox"] == bbox,
            "local bbox includes ancestor/stroke or lost transform",
        )?;
        ensure(scene["viewport"]["zoom"].is_null(), "invented viewport")?;
        for params in [json!({}), json!({"region":[0,0,25,20],"scale":2})] {
            let reply = socket.request("render_view", params.clone())?;
            let bytes = STANDARD.decode(
                reply["result"]["png_base64"]
                    .as_str()
                    .ok_or(format!("render failed: {reply}"))?,
            )?;
            ensure(
                bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.len() >= 24,
                "render PNG",
            )?;
            if params.get("region").is_some() {
                ensure(
                    u32::from_be_bytes(bytes[16..20].try_into()?) == 50,
                    "region scale",
                )?;
            } else if !bbox.is_null() {
                png_pixel(&bytes, 10, 5, [0, 255, 0, 255])?;
            }
        }
        ensure(
            socket.request("export_selection", json!({}))?["ok"] == true,
            "selection export",
        )?;
        ensure(
            socket.request("set_viewport", json!({"mode":"fit_page"}))?["result"]["applied"]
                == false,
            "viewport mutated",
        )?;
        ensure(
            socket.request("arbitrary", json!({"code":"x"}))?["ok"] == false,
            "unknown command accepted",
        )?;
        ensure(
            socket.request("render_view", json!({"region":[0,0,1e8,20]}))?["ok"] == false,
            "render bounds bypass",
        )?;
        ensure(
            socket.request("apply_to_selection", json!({"style":{"fill":"red"}}))?["ok"] == true,
            "style noop",
        )?;
        ensure(
            socket.request("get_document_svg", json!({}))?["result"]["svg"] == before
                && socket.request("get_state_token", json!({}))?["result"]["revision"] == revision,
            "no-op serialization/revision",
        )?;
        ensure(
            socket.request("apply_to_selection", json!({"transform":"translate(NaN)"}))?["ok"]
                == false,
            "unsafe transform",
        )?;
        if change {
            ensure(
                socket.request("apply_to_selection", json!({"style":{"fill":"blue"}}))?["ok"]
                    == true,
                "style edit",
            )?;
            ensure(
                socket.request("get_state_token", json!({}))?["result"]["revision"] != revision,
                "change token",
            )?;
            ensure(
                socket.request(
                    "insert_svg",
                    json!({"svg":"<circle id='c' cx='40' cy='40' r='5'/>"}),
                )?["ok"]
                    == true,
                "insertion",
            )?;
            ensure(
                socket.request(
                    "insert_svg",
                    json!({"svg":"<image href='file:///secret'/>"}),
                )?["ok"]
                    == false,
                "unsafe insert",
            )?;
        }
        let msg = format!(
            "{}\n",
            json!({"v":5,"cmd":"ping","token":info["token"],"params":{}})
        );
        socket
            .reader
            .get_mut()
            .write_all(msg.repeat(2).as_bytes())?;
        ensure(
            socket.receive()?["ok"] == true && socket.receive()?["ok"] == true,
            "coalesced frames lost",
        )?;
    }
    let trace = json!(socket.trace);
    drop(socket);
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        ensure(
            start.elapsed() < Duration::from_secs(10),
            "helper disconnect timed out",
        )?;
        std::thread::sleep(Duration::from_millis(10));
    };
    let bytes = read(&stdout, 64 * 1024 * 1024)?;
    ensure(
        status.success() && fs::metadata(&stderr)?.len() == 0,
        "helper failed on disconnect",
    )?;
    ensure(
        !bytes.is_empty() == change,
        "unexpected SVG publication/no-op",
    )?;
    if change {
        let svg = std::str::from_utf8(&bytes)?;
        ensure(
            svg.contains("blue") && svg.contains("mcp_"),
            "acknowledged edits lost",
        )?;
    }
    ensure(
        fs::read_to_string(&source)? == drawing && !rendezvous.exists(),
        "input changed or rendezvous leaked",
    )?;
    Ok(trace)
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--binary", "--output"])?;
    let binary = args.required("--binary")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let root = root.path().canonicalize()?;
    let mut binary = binary;
    if !binary
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("libexec/inkscape-mcp/package.json")
        .is_file()
    {
        let package = root.join("relocated");
        fs::create_dir_all(package.join("libexec/inkscape-mcp"))?;
        copy(&binary, &package.join("bin/inkscape-mcp-live"))?;
        fs::write(package.join("libexec/inkscape-mcp/package.json"), "{}")?;
        binary = package.join("bin/inkscape-mcp-live");
    }
    let mut traces = vec![];
    for (change, unauthorized, drawing, bbox) in [
        (false, true, SVG.to_string(), json!([7.0, 10.0, 20.0, 10.0])),
        (
            false,
            false,
            SVG.to_string(),
            json!([7.0, 10.0, 20.0, 10.0]),
        ),
        (true, false, SVG.to_string(), json!([7.0, 10.0, 20.0, 10.0])),
        (
            false,
            false,
            SVG.replace("path {fill:#00ff00}", "#r {transform:translate(1px)}"),
            Value::Null,
        ),
    ] {
        traces.push(exercise(
            &binary,
            &root,
            change,
            unauthorized,
            &drawing,
            bbox,
        )?);
    }
    let original = root.join("original");
    fs::write(&original, "original")?;
    let link = root.join("linked.json");
    std::os::unix::fs::symlink(&original, &link)?;
    let env = environment(
        &root,
        &binary
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("libexec/inkscape-mcp"),
    )?;
    let (status, stdout) = crate::acceptance::capture(
        Command::new(&binary)
            .arg(root.join("input.svg"))
            .env_clear()
            .envs(env.iter().map(|(k, v)| (k, v)))
            .env("INKSCAPE_MCP_LIVE_RENDEZVOUS", link),
    )?;
    ensure(
        status != 0 && stdout.is_empty() && fs::read_to_string(original)? == "original",
        "linked rendezvous followed",
    )?;
    write_json(
        &out.join("acceptance.json"),
        &json!({"passed":true,"binary_sha256":hash(&binary)?,"empty_PATH":true,"Python_invoked_by_runtime":false,"native_GUI":false,"traces":traces}),
    )?;
    println!("Native socket acceptance passed");
    Ok(())
}
