//! Owned synthetic gdbus and lock-holder for Rust regression tests.
use inkscape_mcp_rust::{helper_svg, xml, serde_json, libc};
use serde_json::{Value, json};
use std::{fs::{self, OpenOptions}, io::{Write, stdout}, os::fd::AsRawFd, path::Path, time::Duration};
fn read(path: &Path) -> Value { serde_json::from_slice(&fs::read(path).unwrap()).unwrap() }
fn write(path: &Path, value: Value) { fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap(); }
fn append(path: &Path, bytes: &[u8]) { OpenOptions::new().create(true).append(true).open(path).unwrap().write_all(bytes).unwrap(); }
fn default_context() -> Value { json!(["12345678-1234-1234-1234-123456789abc","abcdefab-1234-1234-1234-123456789abc","Drawing.svg"]) }
fn context(root: &Path) -> Value { if root.join("context.json").exists() {read(&root.join("context.json"))} else {default_context()} }
fn tuple(row: &Value) -> String { format!("('{}','{}','{}')",row[0].as_str().unwrap(),row[1].as_str().unwrap(),row[2].as_str().unwrap()) }
fn fail() -> ! { std::process::exit(1) }
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("hold-lock") {
        let file = OpenOptions::new().create(true).append(true).open(&args[1]).unwrap();
        assert_eq!(unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) },0);
        println!("locked"); stdout().flush().unwrap();
        std::thread::sleep(Duration::from_secs(60)); return;
    }
    let executable = std::env::current_exe().unwrap();
    let root = executable.parent().unwrap();
    let config = read(&root.join("fixture-mode.json"));
    let mode = config["mode"].as_str().unwrap();
    let backend = config["backend"].as_str().unwrap();
    if backend == "engine" {
        use std::io::BufRead;
        print!("synthetic banner\n> "); stdout().flush().unwrap();
        for line in std::io::stdin().lock().lines() {
            let command=line.unwrap();
            match command.as_str() {
                "quit" => break,
                "hang" => std::thread::sleep(Duration::from_secs(10)),
                "crash" => fail(),
                "close-stderr" => {unsafe {libc::close(2)};},
                "split" => {print!("{command}\n>"); stdout().flush().unwrap(); std::thread::sleep(Duration::from_millis(10)); print!(" "); stdout().flush().unwrap(); continue;},
                "unknown" => {let _=std::io::stderr().write_all(b"InkscapeApplication::parse_actions: could not find action for: unknown\n"); let _=std::io::stderr().flush();},
                "flood" => print!("{}","x".repeat(70000)),
                _ => (),
            }
            print!("{command}\n> "); stdout().flush().unwrap();
        }
        return;
    }
    append(&root.join("calls.jsonl"), format!("{}\n",json!(args)).as_bytes());
    let has = |text: &str| args.iter().any(|a| a == text);
    if has("org.freedesktop.DBus.GetNameOwner") {
        if mode == "unowned" {fail()}
        println!("{}",if mode == "bad-owner" {"('org.inkscape.Inkscape',)"} else {"(':1.23',)"}); return;
    }
    let control = if root.join("probe-control.json").exists() {read(&root.join("probe-control.json"))} else {json!({})};
    // Test-controlled scheduling delay for exactly one read-only action. The call trace
    // is already recorded, so timeout tests can distinguish preflight from effect dispatch.
    if control["delay_action"].as_str().is_some_and(has) {
        let delay = control["delay_ms"].as_u64().unwrap();
        assert!(delay <= 2000);
        std::thread::sleep(Duration::from_millis(delay));
    }
    if has("org.gtk.Actions.List") {
        if control["reachable"] == false {fail()}
        println!("(list,)"); return;
    }
    if backend == "bus" {
        match mode {
            "context-data" => {
                let row = tuple(&default_context());
                if has("org.inkscape.MCP.Context1.GetContext") {println!("{row}")}
                else if has("org.inkscape.MCP.Context1.ListDocuments") {println!("([{row}],)")}
                else {fail()}
            }
            "context-changed" => {eprint!("org.inkscape.MCP.ContextChanged: private detail"); fail()}
            "timeout" => std::thread::sleep(Duration::from_secs(2)),
            "failed" => fail(), "capped" => println!("{}","x".repeat(9000)),
            _ => println!("()"),
        }
        return;
    }
    if has("org.gtk.Actions.Describe") {
        if control["reachable"] == false {fail()}
        let key = if args.last().unwrap() == "org.inkscape-mcp.insert.noprefs" {"insert"} else {"edit"};
        println!("{}",if control[key].as_bool().unwrap_or(mode != "effect-absent") {"((true,),)"} else {"((false,),)"}); return;
    }
    if has("introspect") {println!("node /org/inkscape/Inkscape/window/17 {{ }}"); return;}
    if has("org.inkscape.MCP.Context1.GetContext") {println!("{}",tuple(&context(root))); return;}
    if has("org.inkscape.MCP.Context1.ListDocuments") {println!("([{}],)",tuple(&default_context())); return;}
    if has("org.inkscape.MCP.Context1.SelectDocument") {
        write(&root.join("context.json"),json!([args[args.len()-2],args[args.len()-1],"Chosen.svg"])); println!("()"); return;
    }
    let (action, parameter) = if has("org.inkscape.MCP.Context1.Activate") {
        let row = context(root);
        if args[args.len()-4] != row[0].as_str().unwrap() || args[args.len()-3] != row[1].as_str().unwrap() {
            eprint!("org.inkscape.MCP.ContextChanged: private detail"); fail();
        }
        (&args[args.len()-2],&args[args.len()-1])
    } else { (&args[args.len()-3],&args[args.len()-2]) };
    match action.as_str() {
        "org.inkscape-mcp.edit.noprefs" | "org.inkscape-mcp.insert.noprefs" => {
            let request = read(&root.join("insert-request.json"));
            write(&root.join("captured-request.json"),request.clone());
            let mut response = json!({"nonce":request["nonce"],"ok":true,"ids":request.get("selection").cloned().unwrap_or(json!([])),"fingerprint":request["expected_fingerprint"]});
            if mode == "effect-mutated" || action == "org.inkscape-mcp.insert.noprefs" {
                let svg = fs::read_to_string(root.join("fixture.svg")).unwrap();
                if helper_svg::fingerprint::fingerprint(&svg,1024*1024).unwrap() != request["expected_fingerprint"] {fail()}
                let bytes = if action == "org.inkscape-mcp.insert.noprefs" {
                    let applied = helper_svg::apply::insert(&svg,request["fragment"].as_str().unwrap(),request["nonce"].as_str().unwrap(),1024*1024).unwrap();
                    response = json!({"nonce":request["nonce"],"ok":true,"ids":applied.ids}); applied.bytes
                } else if request.get("edits").is_some() {
                    let (applied,fp)=helper_svg::oneshot::prepare(&svg,serde_json::from_value(request.clone()).unwrap(),&["r".into()],1024*1024).unwrap();
                    response["fingerprint"]=json!(fp);response["ids"]=json!(applied.ids);applied.bytes
                } else {
                    let document = xml::parse(svg.as_bytes(),1024*1024).unwrap();
                    let mut stack = vec![document.get_root_element().unwrap()];
                    while let Some(mut node) = stack.pop() {
                        if node.get_property("id").as_deref() == Some("r") {node.set_property("fill",request["style"]["fill"].as_str().unwrap()).unwrap();}
                        stack.extend(node.get_child_elements());
                    }
                    let bytes = xml::serialize(&document);
                    response["fingerprint"] = json!(helper_svg::fingerprint::fingerprint(std::str::from_utf8(&bytes).unwrap(),1024*1024).unwrap()); bytes
                };
                if !["effect-mismatch","effect-refused","effect-lost-refused"].contains(&mode) {fs::write(root.join("drawing.svg"),bytes).unwrap();}
            }
            match mode {
                "effect-refused" | "effect-lost-refused" => response = json!({"nonce":request["nonce"],"ok":false,"error":"invalid selection"}),
                "effect-unknown-refusal" => response = json!({"nonce":request["nonce"],"ok":false,"error":"private /host/path"}),
                "effect-stale" => response["nonce"] = json!("stale"),
                "effect-mismatch" => response["fingerprint"] = json!("wrong"), _ => (),
            }
            if mode != "effect-missing" {write(&root.join("insert-result.json"),response);}
            if ["effect-lost","effect-lost-refused"].contains(&mode) {fail()}
            if mode == "effect-switch" {write(&root.join("context.json"),json!(["abcdefab-1234-1234-1234-123456789abc","12345678-1234-1234-1234-123456789abc","Changed"]));}
        }
        "select-list" => {
            let data = match mode {
                "selection-empty" => vec![], "selection-overflow" => vec![b'x';1024*1024+1],
                "selection-invalid" => b"changed format\n".to_vec(), "selection-utf8" => vec![255],
                mode if mode.starts_with("effect-") => b"r cloned: true ref: 1 href: 0 total href: 0\n".to_vec(),
                _ => "r cloned: true ref: 1 href: 0 total href: 0\nПривіт cloned: false ref: 0 href: 0 total href: 0\nr cloned: true ref: 1 href: 0 total href: 0\n".as_bytes().to_vec(),
            }; append(&root.join("stdout.log"),&data);
        }
        "query-x" => if mode != "selection-incomplete" {append(&root.join("stdout.log"),b"0\n")},
        "export-filename" => fs::write(root.join("filename"),&parameter[3..parameter.len()-3]).unwrap(),
        "export-do" => {
            let out = fs::read_to_string(root.join("filename")).unwrap();
            let out = Path::new(&out);
            match mode {
                "missing" => (),
                "symlink" => {fs::write(root.join("original"),b"keep original").unwrap(); std::os::unix::fs::symlink(root.join("original"),out).unwrap();}
                "oversize" => fs::write(out,vec![b'x';1024]).unwrap(),
                "bad-xml" => fs::write(out,b"<invalid").unwrap(), "bad-utf8" => fs::write(out,[255]).unwrap(),
                _ => {
                    let png = out.extension().is_some_and(|ext| ext == "png");
                    let source = if png {"fixture.png"} else if root.join("drawing.svg").exists() {"drawing.svg"} else {"fixture.svg"};
                    let mut data = fs::read(root.join(source)).unwrap();
                    if mode == "mapped" && !png {data = String::from_utf8(data).unwrap().replace("<svg ","<svg id=\"svgroot\" width=\"200\" height=\"100\" viewBox=\"10 20 50 25\" preserveAspectRatio=\"none\" ").into_bytes();}
                    fs::write(out,data).unwrap();
                }
            }
        }
        _ => (),
    }
    println!("()");
}
