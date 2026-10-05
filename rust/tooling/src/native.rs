//! Explicit, opt-in native acceptance. Failures retain the owned session for inspection.
use crate::{
    acceptance::capture_full,
    common::*,
    wire::{Wire, data},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};
const CAP: u64 = 64 * 1024 * 1024;
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}").into())
}
fn deadline(mut predicate: impl FnMut() -> bool, seconds: u64) -> Result<()> {
    let end = Instant::now() + Duration::from_secs(seconds);
    while !predicate() {
        ensure(Instant::now() < end, "native acceptance deadline exceeded")?;
        thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}
fn tree(svg: &str) -> Result<Value> {
    ensure(
        !svg.to_ascii_lowercase().contains("<!doctype")
            && !svg.to_ascii_lowercase().contains("<!entity"),
        "native capture contains a DTD/entity",
    )?;
    let doc = inkscape_mcp_rust::xml::parse(svg.as_bytes(), CAP as usize)?;
    fn node(n: libxml::tree::Node) -> Value {
        if !n.is_element_node() {
            return json!({"text":n.get_content(),"type":format!("{:?}",n.get_type())});
        }
        let mut attrs = std::collections::BTreeMap::new();
        for ((name, ns), value) in n.get_properties_ns() {
            attrs.insert(
                format!("{}:{name}", ns.map(|ns| ns.get_href()).unwrap_or_default()),
                value,
            );
        }
        let children: Vec<_> = n
            .get_child_nodes()
            .into_iter()
            .filter(|c| !(c.is_element_node() && c.get_name() == "namedview"))
            .map(node)
            .collect();
        json!({"name":n.get_name(),"ns":n.get_namespace().map(|ns|ns.get_href()),"attrs":attrs,"text":if n.is_element_node(){String::new()}else{n.get_content()},"type":format!("{:?}",n.get_type()),"children":children})
    }
    Ok(node(doc.get_root_element().ok_or("SVG missing root")?))
}
fn same(a: &Path, b: &Path) -> Result<bool> {
    Ok(tree(&String::from_utf8(read(a, CAP)?)?)? == tree(&String::from_utf8(read(b, CAP)?)?)?)
}
struct Session {
    out: PathBuf,
    package: PathBuf,
    root: PathBuf,
    evidence: Value,
    env: Vec<(String, String)>,
}
impl Session {
    fn load(out: &Path) -> Result<Self> {
        let evidence = crate::common::json(&out.join("session.json"))?;
        ensure(
            evidence["state"] == "connected-owned-blank",
            "owned session unavailable",
        )?;
        let package = PathBuf::from(text(&evidence, "package")?);
        let root = PathBuf::from(text(&evidence, "root")?);
        ensure(
            crate::common::json(&root.join("session/session.json"))? == evidence["manifest"],
            "owned manifest changed; refuse operation",
        )?;
        let env = evidence["env"]
            .as_object()
            .ok_or("invalid recorded environment")?
            .iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    v.as_str().ok_or("invalid environment row")?.to_string(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            out: out.into(),
            package,
            root,
            evidence,
            env,
        })
    }
    fn command(&self, path: &Path) -> Command {
        let mut c = Command::new(path);
        c.env_clear().envs(self.env.clone());
        c
    }
    fn wire(&self, label: &str) -> Result<Wire> {
        let mut wire = Wire::spawn(
            &mut self.command(&self.package.join("bin/inkscape-mcp")),
            &self.out.join(format!("{label}.stderr.log")),
        )?;
        wire.initialize()?;
        ensure(
            data(wire.call("live_connect", json!({"prefer":"no_freeze"}))?)?["connected"] == true,
            "native reconnect failed",
        )?;
        let docs = data(wire.call("live_list_documents", json!({}))?)?;
        let docs = docs["documents"]
            .as_array()
            .ok_or("document list missing")?;
        ensure(
            docs.len() == 1
                && ["window_id", "document_id", "path"]
                    .iter()
                    .all(|k| docs[0][k] == self.evidence["document"][k]),
            "owned context changed; refuse edit",
        )?;
        data(wire.call("live_select_document",json!({"window_id":self.evidence["document"]["window_id"],"document_id":self.evidence["document"]["document_id"]}))?)?;
        Ok(wire)
    }
    fn capture(&self, wire: &mut Wire, label: &str) -> Result<PathBuf> {
        let name = format!("native-{label}.svg");
        let path = self.root.join("workspace").join(&name);
        ensure(!path.exists(), "capture already exists; use a fresh label")?;
        let reply = wire.call("live_sync_to_workspace", json!({"dest_path":name}))?;
        write_json(&self.out.join(format!("{label}.sync.json")), &reply)?;
        data(reply)?;
        ensure(path.is_file(), "native capture absent")?;
        Ok(path)
    }
    fn dbus_command(&self, dest: &str, path: &str, method: &str, args: &[&str]) -> Command {
        let mut c = self.command(&self.package.join("libexec/inkscape-mcp/dbus/bin/gdbus"));
        c.args([
            "call",
            "--address",
            self.evidence["manifest"]["address"].as_str().unwrap_or(""),
            "--dest",
            dest,
            "--object-path",
            path,
            "--method",
            method,
        ])
        .args(args);
        c
    }
    fn dbus(&self, dest: &str, path: &str, method: &str, args: &[&str]) -> Result<String> {
        let (status, out, err) = capture_full(&mut self.dbus_command(dest, path, method, args))?;
        ensure(status == 0, format!("native D-Bus failed: {err}"))?;
        Ok(out)
    }
    fn owner(&self) -> Result<String> {
        let reply = self.dbus(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.GetNameOwner",
            &["org.inkscape.Inkscape"],
        )?;
        let re = regex::Regex::new(r"^\('(:[0-9]+\.[0-9]+)',\)\s*$")?;
        Ok(re
            .captures(&reply)
            .ok_or("unique existing bus owner missing")?[1]
            .into())
    }
    fn activate(&self, owner: &str, action: &str) -> Command {
        self.dbus_command(
            owner,
            "/org/inkscape/Inkscape/MCPContext",
            "org.inkscape.MCP.Context1.Activate",
            &[
                self.evidence["document"]["window_id"]
                    .as_str()
                    .unwrap_or(""),
                self.evidence["document"]["document_id"]
                    .as_str()
                    .unwrap_or(""),
                action,
                "[]",
            ],
        )
    }
    fn close(&mut self, wire: &mut Wire) -> Result<()> {
        // Re-check the recorded context via MCP immediately before any graceful quit.
        let docs = data(wire.call("live_list_documents", json!({}))?)?;
        ensure(
            docs["documents"].as_array().is_some_and(|d| {
                d.len() == 1
                    && d[0]["window_id"] == self.evidence["document"]["window_id"]
                    && d[0]["document_id"] == self.evidence["document"]["document_id"]
            }),
            "quit context changed",
        )?;
        ensure(
            crate::common::json(&self.root.join("session/session.json"))?
                == self.evidence["manifest"],
            "quit manifest changed",
        )?;
        let owner = self.owner()?;
        let pid = self.evidence["manifest"]["inkscape_pid"]
            .as_u64()
            .ok_or("GUI PID missing")?;
        let (status, out, err) = capture_full(&mut self.dbus_command(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.GetConnectionUnixProcessID",
            &[&owner],
        ))?;
        if status == 0 {
            let re = regex::Regex::new(r"^\(uint32 ([0-9]+),\)\s*$")?;
            ensure(
                re.captures(&out)
                    .is_some_and(|c| c[1].parse::<u64>().ok() == Some(pid)),
                "bus peer PID differs",
            )?;
        } else {
            ensure(
                cfg!(target_os = "macos") && err.contains("UnixProcessIdUnknown"),
                "bus ownership credentials failed",
            )?;
        }
        let ps =
            output(Command::new("/bin/ps").args(["-p", &pid.to_string(), "-o", "ppid=,command="]))?;
        let (parent, command) = ps
            .trim()
            .split_once(char::is_whitespace)
            .ok_or("owned process ancestry missing")?;
        ensure(
            parent.parse::<u64>().ok() == self.evidence["manifest"]["supervisor_pid"].as_u64()
                && command.trim()
                    == format!(
                        "{} --with-gui",
                        self.root
                            .join("session/context-bridge/Inkscape.app/Contents/MacOS/inkscape")
                            .display()
                    ),
            "GUI ancestry/private path changed",
        )?;
        let actions = self.dbus(
            &owner,
            "/org/inkscape/Inkscape",
            "org.gtk.Actions.List",
            &[],
        )?;
        ensure(actions.contains("'quit'"), "graceful quit unavailable")?;
        self.dbus(
            &owner,
            "/org/inkscape/Inkscape",
            "org.gtk.Actions.Activate",
            &["quit", "[]", "{}"],
        )?;
        deadline(|| !self.root.join("session/session.json").exists(), 15)?;
        let (status, _, _) = capture_full(&mut self.dbus_command(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.ListNames",
            &[],
        ))?;
        ensure(status != 0, "owned bus survived quit")?;
        let ids = format!("{pid},{}", self.evidence["manifest"]["supervisor_pid"]);
        deadline(
            || {
                capture_full(Command::new("/bin/ps").args(["-p", &ids, "-o", "pid="]))
                    .is_ok_and(|(_, s, _)| s.trim().is_empty())
            },
            5,
        )?;
        self.evidence["state"] = json!("gracefully-closed");
        write_json(&self.out.join("session.json"), &self.evidence)?;
        write_json(
            &self.out.join("shutdown-acceptance.json"),
            &json!({"passed":true,"unique_owned_PID":pid,"graceful_quit":true,"manifest_removed":true,"owned_bus_stopped":true}),
        )
    }
}
fn launch(package: &Path, out: &Path) -> Result<()> {
    ensure(
        cfg!(target_os = "macos"),
        "native managed context acceptance currently requires macOS",
    )?;
    ensure(
        !out.join("session.json").exists(),
        "inspect prior session before launching",
    )?;
    fs::create_dir_all(out)?;
    let root = tempfile::Builder::new()
        .prefix("imcp-native-")
        .tempdir_in(Path::new("/tmp").canonicalize()?)?
        .keep();
    for dir in ["home", "workspace"] {
        fs::create_dir(root.join(dir))?;
    }
    let session = root.join("session");
    let mut env = serde_json::Map::new();
    for (k, v) in [
        ("HOME", root.join("home").display().to_string()),
        ("TMPDIR", root.display().to_string()),
        ("PATH", String::new()),
        ("LANG", "en_US.UTF-8".into()),
        (
            "INKSCAPE_PROFILE_DIR",
            root.join("profile").display().to_string(),
        ),
        (
            "INKSCAPE_MCP_WORKSPACE_ROOTS",
            root.join("workspace").display().to_string(),
        ),
        ("INKSCAPE_MCP_LIVE_ENABLED", "1".into()),
        ("INKSCAPE_MCP_TOOL_PROFILE", "full".into()),
        ("INKSCAPE_MCP_MANAGED_DIR", session.display().to_string()),
        (
            "INKSCAPE_MCP_LIVE_RENDEZVOUS",
            root.join("absent-rendezvous").display().to_string(),
        ),
        ("INKSCAPE_MCP_PROCESS_TIMEOUT_S", "10".into()),
    ] {
        env.insert(k.into(), json!(v));
    }
    let engine = Path::new("/Applications/Inkscape.app/Contents/MacOS/inkscape");
    let original = hash(engine)?;
    let pairs: Vec<_> = env
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect();
    let (status, profile, _) = capture_full(
        Command::new(engine)
            .arg("--user-data-directory")
            .env_clear()
            .envs(pairs.clone()),
    )?;
    ensure(
        status == 0
            && Path::new(profile.trim()).is_absolute()
            && Path::new(profile.trim()).starts_with(&root),
        "profile escaped owned root; refuse launch",
    )?;
    let mut evidence = json!({"root":root,"package":package,"binary_sha256":hash(&package.join("bin/inkscape-mcp"))?,"profile":profile.trim(),"vendor_executable_sha256":original,"state":"preflight","env":env});
    write_json(&out.join("session.json"), &evidence)?;
    let mut wire = Wire::spawn(
        Command::new(package.join("bin/inkscape-mcp"))
            .env_clear()
            .envs(pairs.clone()),
        &out.join("launch.stderr.log"),
    )?;
    let result = (|| -> Result<()> {
        wire.initialize()?;
        ensure(!session.exists(), "startup created managed session")?;
        let reply = wire.call("live_launch", json!({}))?;
        write_json(&out.join("launch.json"), &reply)?;
        ensure(data(reply)?["result"] == true, "launch not ready")?;
        let manifest = crate::common::json(&session.join("session.json"))?;
        ensure(
            text(&manifest, "address")?.split(',').next()
                == Some(format!("unix:path={}", session.join("bus.sock").display()).as_str())
                && manifest["context_bridge"] == true,
            "private bus/context guard missing",
        )?;
        evidence["manifest"] = manifest.clone();
        evidence["state"] = json!("launched");
        write_json(&out.join("session.json"), &evidence)?;
        let connected = data(wire.call("live_connect", json!({"prefer":"no_freeze"}))?)?;
        ensure(
            connected["connected"] == true && connected["transport"] == "managed-dbus",
            "wrong native transport",
        )?;
        let docs = data(wire.call("live_list_documents", json!({}))?)?;
        let docs = docs["documents"].as_array().ok_or("documents missing")?;
        ensure(docs.len() == 1, "unexpected private windows")?;
        let doc = docs[0].clone();
        ensure(
            data(wire.call(
                "live_select_document",
                json!({"window_id":doc["window_id"],"document_id":doc["document_id"]}),
            )?)?["ready_to_edit"]
                == true,
            "owned blank not ready",
        )?;
        evidence["document"] = doc;
        evidence["state"] = json!("connected-owned-blank");
        write_json(&out.join("session.json"), &evidence)?;
        Ok(())
    })();
    wire.save_trace(&out.join("launch.trace.json"))?;
    wire.close();
    result?;
    let owned = Session::load(out)?;
    let mut reconnect = owned.wire("reconnect")?;
    let blank = owned.capture(&mut reconnect, "owned-blank")?;
    fs::copy(blank, out.join("owned-blank.svg"))?;
    reconnect.save_trace(&out.join("reconnect.trace.json"))?;
    reconnect.close();
    ensure(
        crate::common::json(&session.join("session.json"))? == evidence["manifest"]
            && hash(engine)? == original,
        "reconnect changed session or vendor",
    )?;
    write_json(
        &out.join("launch-acceptance.json"),
        &json!({"passed":true,"native_GUI":true,"private_runtime_supervisor":true,"private_bus":true,"context_guard":true,"document_binding":true,"reconnect_without_launch":true,"vendor_unchanged":true,"Undo_Redo_tested":false}),
    )?;
    println!("Owned native session retained: {}", root.display());
    Ok(())
}
fn inx(s: &Session, wire: &mut Wire, phase: &str, label: &str) -> Result<()> {
    let before = s.capture(wire, &format!("{label}-before"))?;
    let selected = data(wire.call("live_get_selection", json!({}))?)?["object_ids"].clone();
    if phase == "capture" {
        return write_json(
            &s.out.join(format!("{label}.json")),
            &json!({"captured":before,"selection":selected}),
        );
    }
    if phase.starts_with("stale-") {
        ensure(
            selected.as_array().is_some_and(|s| !s.is_empty()),
            "select a synthetic object first",
        )?;
        let svg = String::from_utf8(read(&before, CAP)?)?;
        let doc = inkscape_mcp_rust::xml::parse(svg.as_bytes(), CAP as usize)?;
        let ids: Vec<_> =
            inkscape_mcp_rust::helper_svg::elements(doc.get_root_element().ok_or("root missing")?)
                .iter()
                .filter_map(|n| n.get_property("id"))
                .collect();
        let mut request = json!({"nonce":format!("mcp_{}","f".repeat(32)),"operation":"style","selection":selected,"style":{"opacity":"0.2"},"expected_ids":ids,"expected_fingerprint":inkscape_mcp_rust::helper_svg::fingerprint::fingerprint(&svg,CAP as usize)?});
        let reason = match phase {
            "stale-content" => {
                request["expected_fingerprint"] = json!("0".repeat(64));
                "drawing content changed before insertion"
            }
            "stale-ids" => {
                request["expected_ids"] = json!([]);
                "document changed before insertion"
            }
            "stale-selection" => {
                request["selection"] = json!(["missing-selection"]);
                "selection changed before edit"
            }
            _ => return Err("unknown stale phase".into()),
        };
        let exchange = s.root.join("session");
        let path = exchange.join("insert-request.json");
        let result = exchange.join("insert-result.json");
        ensure(
            !path.exists() && !result.exists(),
            "existing exchange; inspect",
        )?;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(serde_json::to_string(&request)?.as_bytes())?;
        file.sync_all()?;
        let owner = s.owner()?;
        let (status, _, err) =
            capture_full(&mut s.activate(&owner, "org.inkscape-mcp.edit.noprefs"))?;
        ensure(status == 0, format!("native activation failed: {err}"))?;
        deadline(|| result.exists(), 10)?;
        let reply = crate::common::json(&result)?;
        write_json(&s.out.join(format!("{label}.json")), &reply)?;
        ensure(
            reply["nonce"] == request["nonce"] && reply["ok"] == false && reply["error"] == reason,
            "stale request not refused as expected",
        )?;
        fs::remove_file(path)?;
        fs::remove_file(result)?;
    } else {
        let (tool, mut args) = match phase {
            "insert" => (
                "live_insert_svg",
                json!({"svg_fragment":"<g transform=\"translate(10,5)\"><rect id=\"r\" x=\"10\" y=\"10\" width=\"40\" height=\"30\" fill=\"#1464dc\"/><circle id=\"c\" cx=\"60\" cy=\"30\" r=\"20\" fill=\"#dc6414\"/></g>"}),
            ),
            "insert-text" => (
                "live_insert_svg",
                json!({"svg_fragment":"<text id=\"t\" x=\"20\" y=\"40\" font-size=\"15\">Hello</text>"}),
            ),
            _ => {
                ensure(
                    selected.as_array().is_some_and(|s| !s.is_empty()),
                    "select synthetic object first",
                )?;
                match phase {
                    "style" => (
                        "live_apply_to_selection",
                        json!({"opacity":0.75,"dx":3,"dy":2}),
                    ),
                    "noop" => ("live_apply_to_selection", json!({"opacity":0.75})),
                    "text" => ("live_set_selected_text", json!({"text":"New & editable"})),
                    "duplicate" | "delete" | "group" | "ungroup" | "raise" | "lower" | "front"
                    | "back" => ("live_edit_selection", json!({"operation":phase})),
                    _ => return Err("unknown native INX phase".into()),
                }
            }
        };
        args["approval_token"] = json!("owned-native-acceptance");
        let reply = wire.call(tool, args)?;
        write_json(&s.out.join(format!("{label}.json")), &reply)?;
        let id = data(reply)?["operation_id"].clone();
        let operations = wire.request(
            "resources/read",
            Some(json!({"uri":"inkscape://live/operations"})),
        )?;
        write_json(&s.out.join(format!("{label}.operations.json")), &operations)?;
        let records: Value = serde_json::from_str(
            operations["result"]["contents"][0]["text"]
                .as_str()
                .ok_or("audit missing")?,
        )?;
        let matches: Vec<_> = records["operations"]
            .as_array()
            .ok_or("audit list missing")?
            .iter()
            .filter(|r| r["operation_id"] == id)
            .collect();
        ensure(
            matches.len() == 1 && matches[0]["status"] == "applied",
            "audit confirmation missing",
        )?;
    }
    let after = s.capture(wire, &format!("{label}-after"))?;
    if phase.starts_with("stale-") || phase == "noop" {
        ensure(same(&before, &after)?, "refusal/noop changed SVG")?;
    }
    Ok(())
}
fn bridge(
    s: &Session,
    wire: &mut Wire,
    label: &str,
    command: Option<(&str, Value)>,
) -> Result<PathBuf> {
    let rendezvous = PathBuf::from(
        s.env
            .iter()
            .find(|(k, _)| k == "INKSCAPE_MCP_LIVE_RENDEZVOUS")
            .ok_or("rendezvous absent")?
            .1
            .clone(),
    );
    ensure(!rendezvous.exists(), "previous socket active")?;
    let owner = s.owner()?;
    let mut c = s.activate(&owner, "org.inkscape-mcp.live.noprefs");
    let log = s.out.join(format!("{label}.activation.log"));
    c.stdout(fs::File::create(&log)?)
        .stderr(fs::OpenOptions::new().append(true).open(&log)?);
    struct Owned(std::process::Child);
    impl Drop for Owned {
        fn drop(&mut self) {
            if self.0.try_wait().ok().flatten().is_none() {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
    }
    let mut activation = Owned(c.spawn()?);
    deadline(|| rendezvous.exists(), 15)?;
    let info = crate::common::json(&rendezvous)?;
    let pid = info["pid"].as_u64().ok_or("helper PID missing")?;
    let ps = output(Command::new("/bin/ps").args(["-p", &pid.to_string(), "-o", "command="]))?;
    ensure(
        ps.contains(s.package.join("bin/inkscape-mcp-live").to_str().unwrap()),
        "wrong native helper process",
    )?;
    let port = u16::try_from(info["port"].as_u64().ok_or("helper port missing")?)?;
    let mut connection = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(40),
    )?;
    connection.set_read_timeout(Some(Duration::from_secs(40)))?;
    connection.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut reader = BufReader::new(connection.try_clone()?);
    let mut request = |cmd: &str, params: Value| -> Result<Value> {
        let message = json!({"v":5,"cmd":cmd,"token":info["token"],"params":params});
        connection.write_all(format!("{message}\n").as_bytes())?;
        let mut bytes = Vec::new();
        use std::io::Read;
        let size = (&mut reader).take(CAP + 1).read_until(b'\n', &mut bytes)?;
        ensure(
            size <= CAP as usize && bytes.last() == Some(&b'\n'),
            "socket reply frame exceeds cap",
        )?;
        let reply: Value = serde_json::from_slice(&bytes)?;
        ensure(
            reply["ok"] == true,
            format!("native socket failed: {reply}"),
        )?;
        Ok(reply["result"].clone())
    };
    request("hello", json!({}))?;
    let before = request("get_document_svg", json!({}))?["svg"].clone();
    let scene = request("get_scene", json!({}))?;
    let render = request("render_view", json!({"region":[0,0,100,100],"scale":1}))?;
    let png = STANDARD.decode(render["png_base64"].as_str().ok_or("render PNG missing")?)?;
    ensure(png.starts_with(b"\x89PNG\r\n\x1a\n"), "native PNG invalid")?;
    fs::write(s.out.join(format!("{label}.png")), png)?;
    if let Some((cmd, params)) = command {
        ensure(
            request(cmd, params)?["undo_friendly"] == true,
            "native edit acknowledgment missing",
        )?;
    }
    if label == "style" {
        request(
            "insert_svg",
            json!({"svg":"<circle id=\"socket-circle\" cx=\"70\" cy=\"40\" r=\"5\" fill=\"green\"/>"}),
        )?;
    }
    let after = request("get_document_svg", json!({}))?["svg"].clone();
    write_json(
        &s.out.join(format!("{label}.socket.json")),
        &json!({"scene":scene,"changed":before!=after,"helper_pid":pid,"helper_sha256":hash(&s.package.join("bin/inkscape-mcp-live"))?}),
    )?;
    drop(reader);
    drop(connection);
    deadline(|| !rendezvous.exists(), 15)?;
    let mut exit = None;
    deadline(
        || {
            exit = activation.0.try_wait().ok().flatten();
            exit.is_some()
        },
        15,
    )?;
    ensure(
        exit.is_some_and(|e| e.success()),
        "native activation failed; inspect activation log",
    )?;
    s.capture(wire, &format!("socket-{label}"))
}
fn socket(s: &mut Session, wire: &mut Wire, phase: &str) -> Result<()> {
    let workspace = s.root.join("workspace");
    let capture = |wire: &mut Wire, label: &str| s.capture(wire, &format!("socket-{label}"));
    match phase {
        "setup" => {
            capture(wire, "blank")?;
            let reply=data(wire.call("live_insert_svg",json!({"svg_fragment":"<rect id=\"r\" x=\"10\" y=\"10\" width=\"30\" height=\"20\" style=\"fill:red\"/><text id=\"t\" x=\"10\" y=\"60\">Hello</text>","approval_token":"owned-native-acceptance"}))?)?;
            write_json(&s.out.join("ids.json"), &reply["affected_ids"])?;
            capture(wire, "baseline")?;
            println!(
                "Select the synthetic rectangle in the recorded private app, then run --phase style."
            );
        }
        "style" | "package-style" => {
            let ids = crate::common::json(&s.out.join("ids.json"))?;
            let expected = ids[if phase == "package-style" { 0 } else { 1 }].clone();
            ensure(
                !expected.is_null()
                    && data(wire.call("live_get_selection", json!({}))?)?["object_ids"]
                        == json!([expected]),
                "select owned target first",
            )?;
            let baseline = capture(wire, "baseline-selected")?;
            let after = bridge(
                s,
                wire,
                "style",
                Some(("apply_to_selection", json!({"style":{"fill":"blue"}}))),
            )?;
            ensure(!same(&baseline, &after)?, "native style not applied")?;
            let noop = bridge(
                s,
                wire,
                "style-noop",
                Some(("apply_to_selection", json!({"style":{"fill":"blue"}}))),
            )?;
            ensure(same(&after, &noop)?, "native style noop changed drawing")?;
        }
        "text" => {
            let ids = crate::common::json(&s.out.join("ids.json"))?;
            ensure(
                !ids[2].is_null()
                    && data(wire.call("live_get_selection", json!({}))?)?["object_ids"]
                        == json!([ids[2]]),
                "select owned text first",
            )?;
            let text = bridge(
                s,
                wire,
                "text",
                Some(("set_selected_text", json!({"text":"Native & editable"}))),
            )?;
            ensure(
                fs::read_to_string(&text)?.contains("Native &amp; editable"),
                "native text not applied",
            )?;
            let noop = bridge(
                s,
                wire,
                "text-noop",
                Some(("set_selected_text", json!({"text":"Native & editable"}))),
            )?;
            ensure(same(&text, &noop)?, "native text noop changed drawing")?;
        }
        "verify-text-undo" | "verify-style-undo" => {
            let path = capture(wire, phase)?;
            let expected = if phase == "verify-text-undo" {
                "style"
            } else {
                "baseline"
            };
            ensure(
                same(
                    &path,
                    &workspace.join(format!("native-socket-{expected}.svg")),
                )?,
                "one native Undo failed after no-op",
            )?;
            write_json(
                &s.out.join(format!("{phase}.json")),
                &json!({"passed":true,"capture":path}),
            )?;
        }
        "finish" | "close-owned" => {
            let path = capture(wire, phase)?;
            ensure(
                same(&path, &workspace.join("native-socket-blank.svg"))?,
                "drawing not restored to blank",
            )?;
            if phase == "finish" {
                for name in ["verify-text-undo", "verify-style-undo"] {
                    ensure(
                        crate::common::json(&s.out.join(format!("{name}.json")))?["passed"] == true,
                        "native Undo evidence missing",
                    )?;
                }
                for (label, changed) in [
                    ("style", true),
                    ("style-noop", false),
                    ("text", true),
                    ("text-noop", false),
                ] {
                    ensure(
                        crate::common::json(&s.out.join(format!("{label}.socket.json")))?["changed"]
                            == changed,
                        "native changed/noop evidence missing",
                    )?;
                }
                ensure(
                    !s.package.join("libexec/inkscape-mcp/python").exists(),
                    "bundled Python found",
                )?;
                write_json(
                    &s.out.join("acceptance.json"),
                    &json!({"passed":true,"native_GUI":true,"actual_INX_socket":true,"accumulated_style_insertion_one_step_Undo":true,"style_text_one_step_Undo":true,"no_op_Undo":true,"Undo_Redo_tested":false,"helper_sha256":hash(&s.package.join("bin/inkscape-mcp-live"))?}),
                )?;
            }
            s.close(wire)?;
        }
        _ => return Err("unknown native socket phase".into()),
    }
    Ok(())
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--package", "--output", "--phase", "--label"])?;
    let out = args.required("--output")?;
    let phase = args.values.get("--phase").ok_or("required --phase")?;
    let allowed: &[&str] = match args.command.as_str() {
        "native-gui" => &["setup", "close-owned"],
        "native-socket" => &[
            "setup",
            "style",
            "package-style",
            "text",
            "verify-text-undo",
            "verify-style-undo",
            "finish",
            "close-owned",
        ],
        "native-inx" => &[
            "capture",
            "insert",
            "insert-text",
            "style",
            "noop",
            "text",
            "duplicate",
            "delete",
            "group",
            "ungroup",
            "raise",
            "lower",
            "front",
            "back",
            "stale-content",
            "stale-ids",
            "stale-selection",
        ],
        _ => return Err("unknown native suite".into()),
    };
    ensure(
        allowed.contains(&phase.as_str()),
        "unknown phase; refusing before launch",
    )?;
    let label = args
        .values
        .get("--label")
        .map(String::as_str)
        .unwrap_or(phase);
    ensure(
        !label.is_empty()
            && label.len() <= 80
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "invalid evidence label",
    )?;
    if phase == "setup" {
        let package = args.required("--package")?.canonicalize()?;
        crate::acceptance::verify_files(&package)?;
        launch(&package, &out)?;
        if args.command == "native-gui" {
            return Ok(());
        }
    }
    let mut s = Session::load(&out)?;
    if let Some(package) = args.values.get("--package") {
        ensure(
            Path::new(package).canonicalize()? == s.package.canonicalize()?,
            "package differs from owned launch",
        )?;
    }
    let mut wire = s.wire(label)?;
    let result = match args.command.as_str() {
        "native-inx" => inx(&s, &mut wire, phase, label),
        "native-socket" => socket(&mut s, &mut wire, phase),
        _ => {
            let capture = s.capture(&mut wire, label)?;
            let blank = s.out.join("owned-blank.svg");
            ensure(
                blank.exists() && same(&capture, &blank)?,
                "recorded blank missing or drawing changed; refuse quit",
            )?;
            s.close(&mut wire)
        }
    };
    wire.save_trace(&out.join(format!("{label}.trace.json")))?;
    wire.close();
    result?;
    println!("Native {phase}: passed on recorded owned drawing");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn svg_equality_preserves_artwork_and_ignores_view() {
        let a = "<svg xmlns='http://www.w3.org/2000/svg'><namedview id='v'/><rect x='1' fill='red'/></svg>";
        let b = "<svg xmlns='http://www.w3.org/2000/svg'><rect fill='red' x='1'/></svg>";
        assert_eq!(tree(a).unwrap(), tree(b).unwrap());
        assert_ne!(tree(b).unwrap(), tree(&b.replace("red", "blue")).unwrap());
        assert!(tree("<!DOCTYPE svg [<!ENTITY e SYSTEM 'file:///tmp/x'>]><svg>&e;</svg>").is_err());
    }
    #[test]
    fn ownership_record_refuses_missing_changed_and_closed_manifest() {
        let root = tempfile::tempdir().unwrap();
        let out = root.path().join("out");
        fs::create_dir(&out).unwrap();
        assert!(Session::load(&out).is_err());
        let managed = root.path().join("session");
        fs::create_dir(&managed).unwrap();
        let evidence = json!({"root":root.path(),"package":root.path(),"manifest":{"inkscape_pid":1},"state":"connected-owned-blank","env":{}});
        write_json(&out.join("session.json"), &evidence).unwrap();
        assert!(Session::load(&out).is_err());
        write_json(&managed.join("session.json"), &json!({"inkscape_pid":2})).unwrap();
        assert!(Session::load(&out).is_err());
        write_json(&managed.join("session.json"), &evidence["manifest"]).unwrap();
        assert!(Session::load(&out).is_ok());
        let mut closed = evidence;
        closed["state"] = json!("gracefully-closed");
        write_json(&out.join("session.json"), &closed).unwrap();
        assert!(Session::load(&out).is_err());
    }
}
