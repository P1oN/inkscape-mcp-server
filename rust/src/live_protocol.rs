//! Fixed v5 live wire. No code, shell, raw Actions or routable endpoints.
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
pub const VERSION: u64 = 5;
pub const MAX_MESSAGE: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Hello,
    Ping,
    ActiveDocument,
    Selection,
    InspectSelection,
    DocumentSvg,
    RenderView,
    ApplySelection,
    InsertSvg,
    SetText,
    ExportSelection,
    SetViewport,
    Scene,
    StateToken,
}
impl Command {
    pub const ALL: [Self; 14] = [
        Self::Hello,
        Self::Ping,
        Self::ActiveDocument,
        Self::Selection,
        Self::InspectSelection,
        Self::DocumentSvg,
        Self::RenderView,
        Self::ApplySelection,
        Self::InsertSvg,
        Self::SetText,
        Self::ExportSelection,
        Self::SetViewport,
        Self::Scene,
        Self::StateToken,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Hello => "hello",
            Self::Ping => "ping",
            Self::ActiveDocument => "get_active_document",
            Self::Selection => "get_selection",
            Self::InspectSelection => "inspect_selection",
            Self::DocumentSvg => "get_document_svg",
            Self::RenderView => "render_view",
            Self::ApplySelection => "apply_to_selection",
            Self::InsertSvg => "insert_svg",
            Self::SetText => "set_selected_text",
            Self::ExportSelection => "export_selection",
            Self::SetViewport => "set_viewport",
            Self::Scene => "get_scene",
            Self::StateToken => "get_state_token",
        }
    }
    pub fn mutates(self) -> bool {
        matches!(self, Self::ApplySelection | Self::InsertSvg | Self::SetText)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ResponseError {
    Protocol(&'static str),
    Rejected,
}
#[derive(serde::Serialize)]
struct Request<'a> {
    v: u64,
    cmd: &'a str,
    token: &'a str,
    params: &'a Value,
}
struct FrameBuffer {
    bytes: Vec<u8>,
    cap: usize,
    exceeded: bool,
}
impl Write for FrameBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.cap.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(std::io::Error::other("live frame cap"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(
    command: Command,
    token: &str,
    params: &Value,
    cap: usize,
) -> Result<Vec<u8>, &'static str> {
    if !params.is_object() {
        return Err("request params must be an object");
    }
    if cap == 0 {
        return Err("outbound message exceeds size cap");
    }
    let mut writer = FrameBuffer {
        bytes: Vec::new(),
        cap: cap - 1,
        exceeded: false,
    };
    let result = serde_json::to_writer(
        &mut writer,
        &Request {
            v: VERSION,
            cmd: command.name(),
            token,
            params,
        },
    );
    if writer.exceeded {
        return Err("outbound message exceeds size cap");
    }
    result.map_err(|_| "request serialization failed")?;
    writer.bytes.push(b'\n');
    Ok(writer.bytes)
}
pub fn request(command: Command, token: &str, params: &Value) -> Result<Vec<u8>, &'static str> {
    encode(command, token, params, MAX_MESSAGE)
}
pub fn response(frame: Value) -> Result<Value, ResponseError> {
    if frame["v"].as_u64() != Some(VERSION) && frame["v"].as_f64() != Some(VERSION as f64) {
        return Err(ResponseError::Protocol("protocol version mismatch"));
    }
    match frame["ok"].as_bool() {
        Some(true) if frame["result"].is_object() => Ok(frame["result"].clone()),
        Some(true) => Err(ResponseError::Protocol("malformed success response")),
        Some(false) => Err(ResponseError::Rejected),
        None => Err(ResponseError::Protocol("response missing ok discriminator")),
    }
}
fn remaining(deadline: Instant) -> Result<Duration, &'static str> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or("live request timed out")
}
pub fn exchange(
    stream: &mut TcpStream,
    bytes: &[u8],
    cap: usize,
    deadline: Instant,
) -> Result<Value, &'static str> {
    let cap = cap.min(MAX_MESSAGE);
    let mut written = 0;
    while written < bytes.len() {
        stream
            .set_write_timeout(Some(remaining(deadline)?))
            .map_err(|_| "socket write failed")?;
        let n = stream
            .write(&bytes[written..])
            .map_err(|_| "socket write failed")?;
        if n == 0 {
            return Err("socket write failed");
        }
        written += n;
    }
    let mut frame = Vec::new();
    let mut chunk = [0u8; 65536];
    loop {
        stream
            .set_read_timeout(Some(remaining(deadline)?))
            .map_err(|_| "socket read failed")?;
        let bound = cap
            .saturating_sub(frame.len())
            .saturating_add(1)
            .min(chunk.len());
        let n = stream
            .read(&mut chunk[..bound])
            .map_err(|_| "socket read failed")?;
        if n == 0 {
            return Err("connection closed before a complete message");
        }
        if let Some(end) = chunk[..n].iter().position(|b| *b == b'\n') {
            if frame.len() + end + 1 > cap {
                return Err("inbound message exceeds size cap");
            }
            if end + 1 != n {
                return Err("unsolicited trailing live frame");
            }
            frame.extend_from_slice(&chunk[..end]);
            break;
        }
        frame.extend_from_slice(&chunk[..n]);
        if frame.len() > cap {
            return Err("inbound message exceeds size cap");
        }
    }
    let frame: Value =
        serde_json::from_slice(&frame).map_err(|_| "message is not valid UTF-8 JSON")?;
    if !frame.is_object() {
        return Err("message is not a JSON object");
    }
    Ok(frame)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn outbound_size_includes_utf8_escaping_and_newline_without_unbounded_buffer() {
        let params = json!({"text":"Привіт\n\"\\"});
        let bytes = request(Command::SetText, "token", &params).unwrap();
        assert_eq!(
            encode(Command::SetText, "token", &params, bytes.len()).unwrap(),
            bytes
        );
        assert_eq!(
            encode(Command::SetText, "token", &params, bytes.len() - 1),
            Err("outbound message exceeds size cap")
        );
        assert!(encode(Command::Ping, "token", &json!({}), 0).is_err());
        let mut writer = FrameBuffer {
            bytes: vec![],
            cap: 4,
            exceeded: false,
        };
        assert!(writer.write_all(&[1; 5]).is_err());
        assert!(writer.bytes.is_empty());
    }
    #[test]
    fn compiled_reference_request_and_response_cases_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/socket-protocol-cases.json"
        ))
        .unwrap();
        assert_eq!(fixture["version"], VERSION);
        assert_eq!(fixture["max_message"], MAX_MESSAGE);
        let names: Vec<_> = Command::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(json!(names), fixture["commands"]);
        for case in fixture["requests"].as_array().unwrap() {
            let command = *Command::ALL
                .iter()
                .find(|c| c.name() == case["command"].as_str().unwrap())
                .unwrap();
            assert_eq!(
                String::from_utf8(
                    request(command, case["token"].as_str().unwrap(), &case["params"]).unwrap()
                )
                .unwrap(),
                case["encoded"].as_str().unwrap()
            );
        }
        for case in fixture["responses"].as_array().unwrap() {
            match response(case["frame"].clone()) {
                Ok(value) => assert_eq!(value, case["result"]),
                Err(ResponseError::Protocol(message)) => {
                    assert_eq!(message, case["error"].as_str().unwrap())
                }
                Err(ResponseError::Rejected) => assert_eq!(
                    "live helper rejected the request",
                    case["error"].as_str().unwrap()
                ),
            }
        }
        assert!(request(Command::Ping, "token", &json!([])).is_err());
        assert!(!Command::SetViewport.mutates());
        assert_eq!(Command::ALL.iter().filter(|c| c.mutates()).count(), 3);
    }
}
