use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

struct Wire {
    child: Child,
    input: ChildStdin,
    output: Receiver<String>,
}
impl Drop for Wire {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Wire {
    fn start(root: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_inkscape-mcp-rust"))
            .env_clear()
            .env("HOME", root)
            .env("PATH", "/usr/bin:/bin")
            .env("INKSCAPE_MCP_WORKSPACE_ROOTS", root)
            .env("INKSCAPE_MCP_LIVE_ENABLED", "false")
            .env("SENTRY_DSN", "")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, output) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if sender.send(line.unwrap()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            output,
        }
    }
    fn send(&mut self, value: Value) {
        writeln!(self.input, "{value}").unwrap();
        self.input.flush().unwrap();
    }
    fn request(&mut self, id: usize, method: &str, mut params: Value, modern: bool) -> Value {
        if modern {
            params["_meta"] = json!({
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientInfo": {"name":"wire-regression","version":"1"},
                "io.modelcontextprotocol/clientCapabilities": {}
            });
        }
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let line = self
            .output
            .recv_timeout(Duration::from_secs(10))
            .expect("bounded MCP response");
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], id);
        response
    }
}

fn exercise(root: &Path, modern: bool) -> Vec<Value> {
    let mut wire = Wire::start(root);
    if modern {
        let discovery = wire.request(0, "server/discover", json!({}), true);
        assert_eq!(discovery["result"]["resultType"], "complete", "{discovery}");
        assert!(
            discovery["result"]["supportedVersions"]
                .as_array()
                .unwrap()
                .contains(&json!("2026-07-28"))
        );
    } else {
        let init = wire.request(
            0,
            "initialize",
            json!({
                "protocolVersion":"2025-03-26", "capabilities":{},
                "clientInfo":{"name":"wire-regression","version":"1"}
            }),
            false,
        );
        assert_eq!(init["result"]["protocolVersion"], "2025-03-26");
        wire.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    }
    let mut results = Vec::new();
    for (index, (method, params)) in [
        ("tools/list", json!({})),
        ("resources/list", json!({})),
        ("resources/templates/list", json!({})),
        ("prompts/list", json!({})),
        (
            "prompts/get",
            json!({"name":"compose_artwork","arguments":{"goal":"synthetic protocol acceptance"}}),
        ),
        ("resources/read", json!({"uri":"inkscape://workspace"})),
        (
            "resources/read",
            json!({"uri":"inkscape://runtime/intents"}),
        ),
        (
            "tools/call",
            json!({"name":"render_preview","arguments":{"doc_id":"missing-fixture-document"}}),
        ),
        (
            "tools/call",
            json!({"name":"get_workspace_info","arguments":{}}),
        ),
        (
            "tools/call",
            json!({"name":"missing_fixture_tool","arguments":{}}),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut response = wire.request(index + 1, method, params, modern);
        assert!(response.get("error").is_none(), "{method}: {response}");
        if modern {
            assert_eq!(response["result"]["resultType"], "complete", "{method}");
            response["result"]
                .as_object_mut()
                .unwrap()
                .remove("resultType");
            if method.ends_with("/list") || method == "resources/read" {
                assert_eq!(response["result"]["ttlMs"], 0, "{method}");
                assert_eq!(response["result"]["cacheScope"], "private", "{method}");
                response["result"].as_object_mut().unwrap().remove("ttlMs");
                response["result"]
                    .as_object_mut()
                    .unwrap()
                    .remove("cacheScope");
            }
        } else {
            assert!(response["result"].get("resultType").is_none(), "{method}");
        }
        results.push(response);
    }
    assert_eq!(results.last().unwrap()["result"]["isError"], true);
    results
}

#[test]
fn stateless_complete_results_preserve_legacy_payloads() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(exercise(root.path(), true), exercise(root.path(), false));
}
