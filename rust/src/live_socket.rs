//! Private extension socket client; session/tool integration is a separate layer.
use crate::{
    live_protocol::{self, Command, ResponseError},
    workspace::Workspace,
};
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const MAX_RENDEZVOUS: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendezvous {
    pub port: u16,
    pub token: String,
    pub pid: Option<i64>,
}
fn integer(value: &Value) -> Option<i64> {
    if let Some(b) = value.as_bool() {
        return Some(i64::from(b));
    }
    if let Some(n) = value.as_i64() {
        return Some(n);
    }
    if let Some(s) = value.as_str() {
        static INTEGER: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"^[+-]?[0-9]+(?:_[0-9]+)*(?:\.0+)?$").unwrap()
        });
        let s = s.trim();
        if !INTEGER.is_match(s) {
            return None;
        }
        return s.split('.').next()?.replace('_', "").parse::<i64>().ok();
    }
    let n = value.as_f64()?;
    if !n.is_finite() || n.fract() != 0.0 || n < i64::MIN as f64 || n >= i64::MAX as f64 {
        return None;
    }
    Some(n as i64)
}
impl Rendezvous {
    pub fn parse(value: &Value) -> Option<Self> {
        let port = integer(&value["port"])
            .and_then(|n| u16::try_from(n).ok())
            .filter(|p| *p > 0)?;
        let token = value["token"]
            .as_str()
            .filter(|t| !t.is_empty() && t.len() <= 4096)?
            .to_owned();
        let version = value
            .get("protocol_version")
            .map(integer)
            .unwrap_or(Some(live_protocol::VERSION as i64))?;
        if version != live_protocol::VERSION as i64 {
            return None;
        }
        let pid = if value.get("pid").is_none_or(Value::is_null) {
            None
        } else {
            Some(integer(&value["pid"])?)
        };
        Some(Self { port, token, pid })
    }
}
/// No-follow descent from the filesystem root. An explicit override cannot escape through
/// a file or ancestor symlink; OS temp's standard alias is canonicalized by its caller.
pub fn read_candidate(path: &Path, cap: usize) -> Option<Rendezvous> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let relative = absolute.strip_prefix(Path::new("/")).ok()?;
    let workspace = Workspace {
        roots: vec![PathBuf::from("/")],
        max_input: cap.min(MAX_RENDEZVOUS),
        max_output: cap,
    };
    let bytes = workspace.read(0, relative, cap.min(MAX_RENDEZVOUS)).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    Rendezvous::parse(&value)
}
pub fn discover(user_data: Option<&str>, cap: usize) -> Option<Rendezvous> {
    let mut candidates = Vec::new();
    if let Ok(value) = std::env::var("INKSCAPE_MCP_LIVE_RENDEZVOUS")
        && !value.trim().is_empty()
    {
        candidates.push(PathBuf::from(value.trim()));
    }
    if let Some(user) = user_data.filter(|s| !s.is_empty()) {
        candidates.push(Path::new(user).join("inkscape-mcp-live.json"));
    }
    if let Ok(temp) = std::env::temp_dir().canonicalize() {
        candidates.push(temp.join("inkscape-mcp-live.json"));
    }
    candidates.iter().find_map(|p| read_candidate(p, cap))
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EditRefused(String),
    Context(&'static str),
    Disabled,
    NotAvailable,
    Connection(&'static str),
    Protocol(&'static str),
    Rejected,
    Unsupported(&'static str),
    Uncertain,
}
impl Error {
    pub fn public_message(&self) -> &str {
        match self {
            Self::EditRefused(message) => message,
            Self::Context(message) => message,
            Self::Disabled => {
                "live mode is disabled (set INKSCAPE_MCP_LIVE_ENABLED=1 to enable); call check_live_support to inspect live readiness"
            }
            Self::NotAvailable => {
                "no live session available; call live_connect to connect (or check_live_support to probe what this host supports)"
            }
            Self::Uncertain => {
                "edit completion is uncertain; inspect the task drawing before retrying"
            }
            Self::Connection(_) => "live session communication failed",
            Self::Protocol(_) | Self::Rejected => "live operation failed",
            Self::Unsupported(_) => {
                "the active live transport does not support this operation; call check_live_support to see which transport supports it"
            }
        }
    }
}
/// Transport arguments are built by the governed live tool validators before dispatch.
/// This enum permits only the four semantic viewport modes; no raw action member exists.
pub enum Viewport {
    Zoom { zoom: f64, center: Option<[f64; 2]> },
    Pan { dx: f64, dy: f64 },
    FitSelection,
    FitPage,
}
impl Viewport {
    pub(crate) fn params(&self) -> Value {
        match self {
            Self::Zoom { zoom, center } => {
                let mut params = json!({"mode":"zoom","zoom":zoom});
                if let Some(center) = center {
                    params["center"] = json!(center);
                }
                params
            }
            Self::Pan { dx, dy } => json!({"mode":"pan","dx":dx,"dy":dy}),
            Self::FitSelection => json!({"mode":"fit_selection"}),
            Self::FitPage => json!({"mode":"fit_page"}),
        }
    }
}
/// Decode transport binary data, not PNG visual validity (the render pipeline checks that).
/// Python validate=True accepts nonzero unused padding bits; preserve that wire behavior.
pub fn decode_png(result: &Value, unsupported: &'static str) -> Result<Vec<u8>, Error> {
    use base64::{
        Engine, alphabet,
        engine::{GeneralPurpose, GeneralPurposeConfig},
    };
    let encoded = result["png_base64"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::Unsupported(unsupported))?;
    let engine = GeneralPurpose::new(
        &alphabet::STANDARD,
        GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
    );
    engine
        .decode(encoded)
        .map_err(|_| Error::Connection("live render payload was malformed"))
}
pub struct Client {
    stream: Option<TcpStream>,
    token: String,
    pub capabilities: Vec<String>,
    timeout: Duration,
    cap: usize,
}
impl Client {
    pub fn connect(rv: &Rendezvous, timeout: Duration, cap: usize) -> Result<Self, Error> {
        let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, rv.port));
        let stream = TcpStream::connect_timeout(&address, timeout)
            .map_err(|_| Error::Connection("could not connect to the live session"))?;
        let mut client = Self {
            stream: Some(stream),
            token: rv.token.clone(),
            capabilities: Vec::new(),
            timeout,
            cap: cap.min(live_protocol::MAX_MESSAGE),
        };
        let result = client
            .request(Command::Hello, &json!({"client":"inkscape-mcp"}))
            .map_err(|_| Error::Connection("live handshake failed"))?;
        if result["protocol_version"].as_u64() != Some(live_protocol::VERSION)
            && result["protocol_version"].as_f64() != Some(live_protocol::VERSION as f64)
        {
            client.disconnect();
            return Err(Error::Connection("live helper protocol version mismatch"));
        }
        client.capabilities = crate::live_models::ids(&result["capabilities"]);
        Ok(client)
    }
    pub fn active_document(&mut self) -> Result<Value, Error> {
        self.request(Command::ActiveDocument, &json!({}))
            .map(|r| crate::live_models::document(&r))
    }
    pub fn selection(&mut self) -> Result<Value, Error> {
        self.request(Command::Selection, &json!({}))
            .map(|r| crate::live_models::selection(&r))
    }
    pub fn inspect_selection(&mut self) -> Result<Value, Error> {
        self.request(Command::InspectSelection, &json!({}))
            .map(|r| crate::live_models::inspection(&r))
    }
    pub fn document_svg(&mut self) -> Result<String, Error> {
        let result = self.request(Command::DocumentSvg, &json!({}))?;
        result["svg"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .ok_or(Error::Connection("live document could not be read"))
    }
    pub fn scene(&mut self) -> Result<Value, Error> {
        let result = self.request(Command::Scene, &json!({}))?;
        let document = self.active_document()?;
        Ok(crate::live_models::scene(&result, &document))
    }
    pub fn state_token(&mut self) -> Result<(Value, Vec<String>), Error> {
        self.request(Command::StateToken, &json!({}))
            .map(|r| crate::live_models::token(&r))
    }
    pub fn render_view(
        &mut self,
        region: Option<[f64; 4]>,
        scale: Option<f64>,
    ) -> Result<Vec<u8>, Error> {
        if region.is_some_and(|r| r.iter().any(|n| !n.is_finite()))
            || scale.is_some_and(|n| !n.is_finite())
        {
            return Err(Error::Protocol("nonfinite render argument"));
        }
        let mut params = json!({});
        if let Some(region) = region {
            params["region"] = json!(region);
        }
        if let Some(scale) = scale {
            params["scale"] = json!(scale);
        }
        let result = self.request(Command::RenderView, &params)?;
        decode_png(&result, "live render is not available")
    }
    pub fn set_viewport(&mut self, viewport: &Viewport) -> Result<Value, Error> {
        let params = viewport.params();
        // serde_json converts nonfinite floats to null; refuse before framing instead.
        let finite = match viewport {
            Viewport::Zoom { zoom, center } => {
                zoom.is_finite() && center.is_none_or(|c| c.iter().all(|n| n.is_finite()))
            }
            Viewport::Pan { dx, dy } => dx.is_finite() && dy.is_finite(),
            _ => true,
        };
        if !finite {
            return Err(Error::Protocol("nonfinite viewport argument"));
        }
        let result = self.request(Command::SetViewport, &params)?;
        Ok(crate::live_models::viewport_result(
            &result,
            params["mode"].as_str().unwrap(),
        ))
    }
    pub fn apply_selection(
        &mut self,
        style: &std::collections::BTreeMap<String, String>,
        transform: Option<&str>,
    ) -> Result<Value, Error> {
        self.request(
            Command::ApplySelection,
            &json!({"style":style,"transform":transform}),
        )
        .map(|r| crate::live_models::mutation(&r))
    }
    pub fn insert_svg(&mut self, fragment: &str) -> Result<Value, Error> {
        self.request(Command::InsertSvg, &json!({"svg":fragment}))
            .map(|r| crate::live_models::mutation(&r))
    }
    pub fn set_text(&mut self, text: &str) -> Result<Value, Error> {
        self.request(Command::SetText, &json!({"text":text}))
            .map(|r| crate::live_models::mutation(&r))
    }
    pub fn export_selection(&mut self) -> Result<Vec<u8>, Error> {
        let result = self.request(Command::ExportSelection, &json!({}))?;
        decode_png(&result, "live selection export is not available")
    }
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
    pub fn disconnect(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
        self.token.clear();
        self.capabilities.clear();
    }
    /// Internal fixed enum only. No retry: a transmitted mutation with a lost/untrusted
    /// response is uncertain, and the old stream is quarantined before any reconnect.
    pub fn request(&mut self, command: Command, params: &Value) -> Result<Value, Error> {
        let bytes =
            live_protocol::request(command, &self.token, params).map_err(Error::Protocol)?;
        let stream = self
            .stream
            .as_mut()
            .ok_or(Error::Connection("not connected to a live session"))?;
        let deadline = Instant::now()
            .checked_add(self.timeout)
            .ok_or(Error::Connection("invalid live timeout"))?;
        let result = live_protocol::exchange(stream, &bytes, self.cap, deadline)
            .map_err(Error::Connection)
            .and_then(|frame| {
                live_protocol::response(frame).map_err(|e| match e {
                    ResponseError::Rejected => Error::Rejected,
                    ResponseError::Protocol(message) => Error::Protocol(message),
                })
            });
        // A complete negative reply to a read leaves framing synchronized. Keep
        // that channel usable, as the reference does. Mutations and malformed/
        // lost replies still quarantine the channel without an automatic retry.
        if result.is_err() && (command.mutates() || !matches!(result, Err(Error::Rejected))) {
            self.disconnect();
            if command.mutates() && !matches!(result, Err(Error::Rejected)) {
                return Err(Error::Uncertain);
            }
        }
        result
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
        thread,
    };
    fn peer<F>(
        mut reply: F,
        count: usize,
    ) -> (Rendezvous, thread::JoinHandle<()>, Arc<Mutex<Vec<Value>>>)
    where
        F: FnMut(usize, &Value) -> Vec<u8> + Send + 'static,
    {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let handle = thread::spawn(move || {
            let (mut stream, address) = listener.accept().unwrap();
            assert!(address.ip().is_loopback());
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            for i in 0..count {
                let mut bytes = String::new();
                reader.read_line(&mut bytes).unwrap();
                let request: Value = serde_json::from_str(&bytes).unwrap();
                assert_eq!(request["v"], 5);
                assert_eq!(request["token"], "test-secret");
                log.lock().unwrap().push(request.clone());
                let bytes = reply(i, &request);
                let _ = stream.write_all(&bytes);
            }
        });
        (
            Rendezvous {
                port,
                token: "test-secret".into(),
                pid: None,
            },
            handle,
            seen,
        )
    }
    fn success(value: Value) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(&json!({"v":5,"ok":true,"result":value})).unwrap();
        bytes.push(b'\n');
        bytes
    }
    fn hello() -> Vec<u8> {
        success(json!({"protocol_version":5,"capabilities":["get_scene","set_viewport"]}))
    }
    #[test]
    fn session_socket_adapter_attaches_reads_reconnects_and_clears_tokens() {
        use crate::{
            live_session::{Session, Settings},
            live_transport::{Preference, Probe, Socket},
        };
        let mut session = Session::new(Settings {
            enabled: true,
            cache_entries: 2,
            cache_bytes: 1024 * 1024,
            coalesce_ms: 100.,
        });
        let cleared = std::cell::Cell::new(0);
        for iteration in 0..2 {
            let (rv, handle, seen) = peer(
                |i, request| {
                    if i == 0 {
                        hello()
                    } else if request["cmd"] == "get_active_document" {
                        success(json!({"name":"task","object_count":1}))
                    } else {
                        success(json!({"object_ids":["r"]}))
                    }
                },
                3,
            );
            let probes = [Probe {
                name: "extension-socket".into(),
                available: true,
                rank: 20,
                commands: vec![
                    Command::ActiveDocument,
                    Command::Selection,
                    Command::InspectSelection,
                ],
                no_freeze: false,
                detail: String::new(),
            }];
            session
                .connect(
                    Preference::Read,
                    &probes,
                    |_| Ok(Box::new(Socket::new(rv, Duration::from_secs(1), 4096))),
                    || cleared.set(cleared.get() + 1),
                )
                .unwrap();
            assert!(session.last_token.is_none() && session.last_change.is_none());
            assert_eq!(
                session.status(&probes, false)["active_document"]["name"],
                "task"
            );
            assert_eq!(
                session.require_transport().unwrap().selection().unwrap()["object_ids"],
                json!(["r"])
            );
            session.record_change(json!({"iteration":iteration}), json!({"changed":true}));
            handle.join().unwrap();
            assert_eq!(seen.lock().unwrap().len(), 3);
        }
        session.teardown(|| cleared.set(cleared.get() + 1));
        assert_eq!(cleared.get(), 3);
        assert!(
            session.last_token.is_none()
                && session.last_change.is_none()
                && session.cache.is_none()
        );
    }
    #[test]
    fn compiled_reference_binary_payload_cases_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/socket-binary-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let actual = match decode_png(&case["input"], "unavailable") {
                Ok(bytes) => {
                    json!({"hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()})
                }
                Err(Error::Unsupported(message)) => {
                    json!({"error":"LiveCapabilityUnsupported","message":message})
                }
                Err(Error::Connection(message)) => {
                    json!({"error":"LiveConnectionError","message":message})
                }
                _ => panic!("unexpected binary error"),
            };
            assert_eq!(actual, case["expected"], "{:?}", case["input"]);
        }
    }
    #[test]
    fn typed_view_render_and_edit_wrappers_send_fixed_semantics() {
        let (rv, handle, seen) = peer(
            |i, request| {
                if i == 0 {
                    return hello();
                }
                success(match request["cmd"].as_str().unwrap() {
                    "render_view" | "export_selection" => json!({"png_base64":"UE5HREFUQQ=="}),
                    "set_viewport" => json!({"mode":request["params"]["mode"],"applied":true}),
                    "apply_to_selection" | "insert_svg" | "set_selected_text" => {
                        json!({"affected_ids":["r"],"undo_friendly":true,"detail":"test"})
                    }
                    _ => panic!("unexpected command"),
                })
            },
            11,
        );
        let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
        assert_eq!(client.render_view(None, None).unwrap(), b"PNGDATA");
        assert_eq!(
            client
                .render_view(Some([-1., 2., 100., 50.]), Some(1.5))
                .unwrap(),
            b"PNGDATA"
        );
        for viewport in [
            Viewport::Zoom {
                zoom: 2.,
                center: Some([1., 2.]),
            },
            Viewport::Pan { dx: -1., dy: 2. },
            Viewport::FitSelection,
            Viewport::FitPage,
        ] {
            assert_eq!(client.set_viewport(&viewport).unwrap()["applied"], true);
        }
        let style = std::collections::BTreeMap::from([("fill".to_owned(), "#ff0000".to_owned())]);
        assert_eq!(
            client
                .apply_selection(&style, Some("translate(1,2)"))
                .unwrap()["undo_friendly"],
            true
        );
        assert_eq!(
            client.insert_svg("<rect/>").unwrap()["affected_ids"],
            json!(["r"])
        );
        assert_eq!(client.set_text("Привіт").unwrap()["count"], 1);
        assert_eq!(client.export_selection().unwrap(), b"PNGDATA");
        handle.join().unwrap();
        let seen = seen.lock().unwrap();
        assert_eq!(seen[1]["params"], json!({}));
        assert_eq!(
            seen[2]["params"],
            json!({"region":[-1.,2.,100.,50.],"scale":1.5})
        );
        assert_eq!(
            seen[3]["params"],
            json!({"mode":"zoom","zoom":2.,"center":[1.,2.]})
        );
        assert_eq!(
            seen[7]["params"],
            json!({"style":{"fill":"#ff0000"},"transform":"translate(1,2)"})
        );
        assert_eq!(seen[8]["params"], json!({"svg":"<rect/>"}));
        assert_eq!(seen[9]["params"], json!({"text":"Привіт"}));
    }
    #[test]
    fn nonfinite_view_arguments_refuse_before_dispatch_and_edit_wrapper_keeps_uncertain() {
        let (rv, handle, seen) = peer(|_, _| hello(), 1);
        let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
        assert!(client.render_view(None, Some(f64::NAN)).is_err());
        assert!(
            client
                .render_view(Some([0., 0., f64::INFINITY, 1.]), None)
                .is_err()
        );
        assert!(
            client
                .set_viewport(&Viewport::Pan {
                    dx: f64::INFINITY,
                    dy: 1.
                })
                .is_err()
        );
        assert!(
            client
                .set_viewport(&Viewport::Zoom {
                    zoom: 1.,
                    center: Some([f64::NAN, 0.])
                })
                .is_err()
        );
        handle.join().unwrap();
        assert_eq!(seen.lock().unwrap().len(), 1);
        let (rv, handle, seen) = peer(|i, _| if i == 0 { hello() } else { b"partial".to_vec() }, 2);
        let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
        assert_eq!(client.insert_svg("<rect/>").unwrap_err(), Error::Uncertain);
        assert!(!client.is_connected());
        handle.join().unwrap();
        assert_eq!(seen.lock().unwrap().len(), 2);
    }
    #[test]
    fn semantic_reads_use_fixed_commands_and_authoritative_document() {
        let (rv, handle, seen) = peer(
            |i, request| {
                if i == 0 {
                    return hello();
                }
                success(match request["cmd"].as_str().unwrap() {
                    "get_scene" => {
                        json!({"active_document":{"path":"spoof"},"selection":[{"id":"r"}],"visible_objects":[{"id":"r","tag":"rect"}]})
                    }
                    "get_active_document" => {
                        json!({"name":"authoritative","path":"task.svg","object_count":2})
                    }
                    "get_selection" => json!({"object_ids":["r",true]}),
                    "inspect_selection" => json!({"objects":[{"id":"r","tag":"rect"}]}),
                    "get_document_svg" => json!({"svg":"<svg/>"}),
                    "get_state_token" => json!({"revision":"one","selection":["r"]}),
                    _ => panic!("unexpected semantic command"),
                })
            },
            7,
        );
        let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
        let scene = client.scene().unwrap();
        assert_eq!(scene["active_document"]["name"], "authoritative");
        assert_eq!(scene["active_document"]["path"], "task.svg");
        assert_eq!(
            client.selection().unwrap()["object_ids"],
            json!(["r", "True"])
        );
        assert_eq!(client.inspect_selection().unwrap()["count"], 1);
        assert_eq!(client.document_svg().unwrap(), "<svg/>");
        assert_eq!(client.state_token().unwrap().1, ["r"]);
        handle.join().unwrap();
        assert_eq!(seen.lock().unwrap()[2]["cmd"], "get_active_document");
    }
    #[test]
    fn reference_rendezvous_coercion_and_no_follow_sources() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/socket-protocol-cases.json"
        ))
        .unwrap();
        for case in fixture["rendezvous"].as_array().unwrap() {
            let actual = Rendezvous::parse(&case["value"])
                .map(|rv| json!({"port":rv.port,"token":rv.token,"pid":rv.pid}))
                .unwrap_or(Value::Null);
            assert_eq!(actual, case["expected"], "{:?}", case["value"]);
        }
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("protected.json");
        let content = br#"{"port":4321,"token":"test-secret"}"#;
        std::fs::write(&path, content).unwrap();
        assert_eq!(read_candidate(&path, 4096).unwrap().port, 4321);
        assert!(read_candidate(&path, 10).is_none());
        let link = root.join("link.json");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(read_candidate(&link, 4096).is_none());
        let directory = root.join("alias");
        std::os::unix::fs::symlink(&root, &directory).unwrap();
        assert!(read_candidate(&directory.join("protected.json"), 4096).is_none());
        assert_eq!(std::fs::read(&path).unwrap(), content);
        assert!(Rendezvous::parse(&json!({"port":4321,"token":"x".repeat(4097)})).is_none());
    }
    #[test]
    fn real_loopback_all_fixed_commands_handshake_token_and_disconnect() {
        let (rv, handle, seen) = peer(
            |i, request| {
                if i == 0 {
                    assert_eq!(request["cmd"], "hello");
                    assert_eq!(request["params"], json!({"client":"inkscape-mcp"}));
                    hello()
                } else {
                    success(json!({"echo":request["cmd"],"params":request["params"]}))
                }
            },
            14,
        );
        let mut client = Client::connect(&rv, Duration::from_secs(2), 4096).unwrap();
        assert!(client.is_connected());
        assert_eq!(client.capabilities, ["get_scene", "set_viewport"]);
        for command in Command::ALL.into_iter().filter(|c| *c != Command::Hello) {
            let params = json!({"unicode":"Привіт","number":1.5});
            let response = client.request(command, &params).unwrap();
            assert_eq!(response, json!({"echo":command.name(),"params":params}));
        }
        client.disconnect();
        client.disconnect();
        assert!(!client.is_connected());
        assert!(client.capabilities.is_empty());
        assert_eq!(
            client.request(Command::Ping, &json!({})),
            Err(Error::Connection("not connected to a live session"))
        );
        handle.join().unwrap();
        assert_eq!(seen.lock().unwrap().len(), 14);
    }
    #[test]
    fn handshake_version_token_rejection_and_malformed_frames_never_attach() {
        for reply in [
            success(json!({"protocol_version":4})),
            success(json!({"protocol_version":"5"})),
            br#"{"v":5,"ok":false,"error":"secret host path"}
"#
            .to_vec(),
            b"[]\n".to_vec(),
            b"bad json\n".to_vec(),
        ] {
            let (rv, handle, _) = peer(move |_, _| reply.clone(), 1);
            match Client::connect(&rv, Duration::from_secs(1), 4096) {
                Err(Error::Connection(_)) => (),
                _ => panic!("invalid handshake attached"),
            }
            handle.join().unwrap();
        }
    }
    #[test]
    fn complete_read_refusal_keeps_channel_framing_without_a_retry() {
        let (rv, handle, seen) = peer(
            |i, _| match i {
                0 => hello(),
                1 => b"{\"v\":5,\"ok\":false,\"error\":\"private details\"}\n".to_vec(),
                _ => success(json!({"revision":"after refusal"})),
            },
            3,
        );
        let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
        assert_eq!(client.state_token(), Err(Error::Rejected));
        assert!(client.is_connected());
        assert_eq!(
            client.state_token().unwrap(),
            crate::live_models::token(&json!({"revision":"after refusal"}))
        );
        handle.join().unwrap();
        assert_eq!(seen.lock().unwrap().len(), 3);
    }
    #[test]
    fn lost_untrusted_or_capped_mutation_reply_is_uncertain_without_retry() {
        for reply in [
            b"partial".to_vec(),
            b"bad json\n".to_vec(),
            br#"{"v":4,"ok":true,"result":{}}
"#
            .to_vec(),
            b"x".repeat(4097),
            [success(json!({})), success(json!({"stale":true}))].concat(),
        ] {
            let (rv, handle, seen) =
                peer(move |i, _| if i == 0 { hello() } else { reply.clone() }, 2);
            let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
            let error = client
                .request(Command::InsertSvg, &json!({"svg":"<rect/>"}))
                .unwrap_err();
            assert_eq!(error, Error::Uncertain);
            assert_eq!(
                error.public_message(),
                "edit completion is uncertain; inspect the task drawing before retrying"
            );
            assert!(!client.is_connected());
            assert!(client.request(Command::Ping, &json!({})).is_err());
            handle.join().unwrap();
            assert_eq!(seen.lock().unwrap().len(), 2);
        }
        let (rv, handle, _) = peer(
            |i, _| {
                if i == 0 {
                    hello()
                } else {
                    br#"{"v":5,"ok":false,"error":"host path"}
"#
                    .to_vec()
                }
            },
            2,
        );
        let mut client = Client::connect(&rv, Duration::from_secs(1), 4096).unwrap();
        assert_eq!(
            client.request(Command::SetText, &json!({"text":"test"})),
            Err(Error::Rejected)
        );
        assert!(!client.is_connected());
        handle.join().unwrap();
    }
    #[test]
    fn slow_trickle_has_whole_request_deadline_and_view_failure_is_not_mutation() {
        for command in [Command::ApplySelection, Command::SetViewport] {
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            let rv = Rendezvous {
                port: listener.local_addr().unwrap().port(),
                token: "test-secret".into(),
                pid: None,
            };
            let handle = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                stream.write_all(&hello()).unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                for _ in 0..8 {
                    if stream.write_all(b" ").is_err() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            });
            let mut client = Client::connect(&rv, Duration::from_millis(80), 4096).unwrap();
            let start = Instant::now();
            let error = client.request(command, &json!({})).unwrap_err();
            assert!(start.elapsed() < Duration::from_millis(400));
            assert_eq!(matches!(error, Error::Uncertain), command.mutates());
            assert!(!client.is_connected());
            handle.join().unwrap();
        }
    }
}
