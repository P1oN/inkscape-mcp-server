//! Fixed loopback v5 snapshot bridge. Never executes client-supplied actions or paths.
#[allow(dead_code)]
#[path = "../live_protocol.rs"]
mod live_protocol;
#[allow(dead_code)]
#[path = "../process.rs"]
mod process;
#[allow(dead_code)]
#[path = "../workspace.rs"]
mod workspace;
// Reuse the bounded owned-process runner without initializing MCP telemetry.
mod telemetry {
    #[allow(clippy::enum_variant_names)]
    pub enum Failure {
        ProcessStart,
        ProcessTimeout,
        ProcessCrash,
    }
    pub fn failure(_: Failure) {}
}
mod live_launch {
    pub fn library() -> Result<std::path::PathBuf, String> {
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .and_then(std::path::Path::parent)
                    .map(|p| p.join("libexec/inkscape-mcp"))
            })
            .ok_or("package unavailable".into())
    }
}
use base64::{Engine, engine::general_purpose::STANDARD};
use inkscape_mcp_rust::{
    helper_svg::{
        elements,
        socket::{CAP, Snapshot},
    },
    xml,
};
use libxml::tree::Node;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, String>;
const IDLE: Duration = Duration::from_secs(120);
fn ws() -> workspace::Workspace {
    workspace::Workspace {
        roots: vec!["/".into()],
        max_input: CAP,
        max_output: live_protocol::MAX_MESSAGE,
    }
}
fn relative(path: &Path) -> Result<&Path> {
    path.strip_prefix("/")
        .map_err(|_| "absolute path required".into())
}
fn validate_assets(svg: &str) -> Result<()> {
    if svg.contains("<!DOCTYPE") || svg.contains("<?") && !svg.trim_start().starts_with("<?xml ") {
        return Err("DTD/processing instructions require preparation".into());
    }
    let doc = xml::parse(svg.as_bytes(), CAP)?;
    for n in elements(doc.get_root_element().ok_or("invalid SVG")?) {
        if matches!(n.get_name().as_str(), "script" | "foreignObject") {
            return Err("active SVG content requires preparation".into());
        }
        for ((name, namespace), value) in n.get_properties_ns() {
            if name == "base" || name == "absref" || name.to_lowercase().starts_with("on") {
                return Err("external or active SVG content requires preparation".into());
            }
            if name == "href"
                && !value.starts_with('#')
                && !(n.get_name() == "image"
                    && (value.starts_with("data:image/png;base64,")
                        || value.starts_with("data:image/jpeg;base64,")))
            {
                return Err("external assets require preparation".into());
            }
            if namespace.is_none() && !matches!(name.as_str(), "id" | "d" | "points" | "href") {
                if value.contains('\\') || value.contains("/*") {
                    return Err("escaped CSS requires preparation".into());
                }
                static URL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
                    regex::Regex::new(r"(?is)url\s*\((.*?)\)").unwrap()
                });
                for c in URL.captures_iter(&value) {
                    if !c[1]
                        .trim_matches([' ', '\t', '\r', '\n', '\"', '\''])
                        .starts_with('#')
                    {
                        return Err("external paint requires preparation".into());
                    }
                }
                if URL
                    .replace_all(&value, "")
                    .to_ascii_lowercase()
                    .contains("url")
                {
                    return Err("unsupported paint URL".into());
                }
            }
        }
        if n.get_name() == "style" {
            let css = n.get_content();
            if css.contains(['\\', '@']) || css.contains("/*") {
                return Err("stylesheet requires snapshot preparation".into());
            }
            static URL: std::sync::LazyLock<regex::Regex> =
                std::sync::LazyLock::new(|| regex::Regex::new(r"(?is)url\s*\((.*?)\)").unwrap());
            static SIZING: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
                regex::Regex::new(r"(?i)(?:^|[;{])\s*(?:width|height)\s*:").unwrap()
            });
            if SIZING.is_match(&css) {
                return Err("CSS sizing requires preparation".into());
            }
            for c in URL.captures_iter(&css) {
                if !c[1]
                    .trim_matches([' ', '\t', '\r', '\n', '\"', '\''])
                    .starts_with('#')
                {
                    return Err("external CSS paint requires preparation".into());
                }
            }
            if URL
                .replace_all(&css, "")
                .to_ascii_lowercase()
                .contains("url")
            {
                return Err("unsupported CSS URL".into());
            }
        }
    }
    Ok(())
}
struct Raster {
    binary: PathBuf,
}
impl Raster {
    fn stage(&self, snapshot: &Snapshot) -> Result<(tempfile::TempDir, PathBuf)> {
        validate_assets(&snapshot.svg)?;
        let temp = tempfile::Builder::new()
            .prefix("imcp-socket-")
            .tempdir()
            .map_err(|_| "render unavailable")?;
        let path = temp
            .path()
            .canonicalize()
            .map_err(|_| "render unavailable")?
            .join("doc.svg");
        ws().write_new(0, relative(&path)?, snapshot.svg.as_bytes())?;
        Ok((temp, path))
    }
    fn query(&self, snapshot: &Snapshot) -> Result<HashMap<String, [f64; 4]>> {
        let (_temp, src) = self.stage(snapshot)?;
        let out = process::run_bounded(
            &self.binary,
            &[src.to_string_lossy().into_owned(), "--query-all".into()],
            Duration::from_secs(30),
            2 * 1024 * 1024,
        )?;
        if !out.success || out.stdout.len() >= 2 * 1024 * 1024 {
            return Err("geometry query failed".into());
        }
        let mut boxes = HashMap::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let p = line.rsplitn(5, ',').collect::<Vec<_>>();
            if p.len() != 5 {
                continue;
            }
            if let Ok(v) = p[..4]
                .iter()
                .rev()
                .map(|s| s.parse::<f64>())
                .collect::<std::result::Result<Vec<_>, _>>()
                && v.iter().all(|n| n.is_finite())
                && v[2] >= 0.
                && v[3] >= 0.
            {
                boxes.insert(p[4].into(), [v[0], v[1], v[2], v[3]]);
            }
        }
        Ok(boxes)
    }
    fn render(&self, snapshot: &Snapshot, params: &Value, selected: bool) -> Result<Value> {
        let (_temp, src) = self.stage(snapshot)?;
        let output = src.with_file_name("view.png");
        let mut args = vec![
            src.to_string_lossy().into_owned(),
            "--export-type=png".into(),
            format!("--export-filename={}", output.display()),
        ];
        let region = if params["region"].is_null() {
            None
        } else {
            let v = params["region"]
                .as_array()
                .filter(|v| v.len() == 4)
                .ok_or("invalid region")?;
            let v = v
                .iter()
                .map(|n| {
                    n.as_f64()
                        .filter(|n| n.is_finite() && n.abs() <= 1e7)
                        .ok_or("invalid region")
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            if v[2] <= 0. || v[3] <= 0. {
                return Err("invalid region".into());
            }
            Some(v)
        };
        let scale = if params["scale"].is_null() {
            None
        } else {
            Some(
                params["scale"]
                    .as_f64()
                    .filter(|n| n.is_finite() && (1e-3..=64.).contains(n))
                    .ok_or("invalid scale")?,
            )
        };
        if selected && !snapshot.selection.is_empty() {
            if snapshot
                .selection
                .iter()
                .any(|s| s.contains([',', ';', '\n', '\r']))
            {
                return Err("unsupported export ID".into());
            }
            args.extend([
                format!("--export-id={}", snapshot.selection.join(",")),
                "--export-id-only".into(),
            ]);
        } else if let Some(v) = &region {
            args.push(format!(
                "--export-area={:.6}:{:.6}:{:.6}:{:.6}",
                v[0],
                v[1],
                v[0] + v[2],
                v[1] + v[3]
            ));
        } else {
            args.push("--export-area-page".into());
        }
        if let Some(sc) = scale {
            if let Some(v) = &region {
                let width = (v[2] * sc).round_ties_even().max(1.);
                if width > 16384. || v[3] * sc > 16384. {
                    return Err("render dimensions exceed cap".into());
                }
                args.push(format!("--export-width={width:.0}"));
            } else {
                args.push(format!("--export-dpi={:.6}", (96. * sc).max(1.)));
            }
        }
        // Bound dimensions even for unscaled pages/selections before invoking the rasterizer.
        self.dimensions(
            snapshot,
            region.as_deref(),
            if region.is_some() {
                scale.unwrap_or(1.)
            } else {
                (96. * scale.unwrap_or(1.)).max(1.) / 96.
            },
            selected,
        )?;
        let result = process::run_bounded(&self.binary, &args, Duration::from_secs(60), 65536)?;
        if !result.success {
            return Err("render failed".into());
        }
        let bytes = ws().read(0, relative(&output)?, 32 * 1024 * 1024)?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err("invalid PNG".into());
        }
        Ok(json!({"png_base64":STANDARD.encode(bytes)}))
    }
    fn dimensions(
        &self,
        snapshot: &Snapshot,
        region: Option<&[f64]>,
        scale: f64,
        selected: bool,
    ) -> Result<()> {
        let (w, h) = if let Some(v) = region {
            (v[2], v[3])
        } else if selected && !snapshot.selection.is_empty() {
            let boxes = self.query(snapshot)?;
            let mut union = [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ];
            for id in &snapshot.selection {
                let b = boxes.get(id).ok_or("selection geometry unavailable")?;
                union[0] = union[0].min(b[0]);
                union[1] = union[1].min(b[1]);
                union[2] = union[2].max(b[0] + b[2]);
                union[3] = union[3].max(b[1] + b[3]);
            }
            (union[2] - union[0], union[3] - union[1])
        } else {
            let doc = xml::parse(snapshot.svg.as_bytes(), CAP)?;
            let root = doc.get_root_element().ok_or("invalid SVG")?;
            (
                pixel_length(&root, "width")?,
                pixel_length(&root, "height")?,
            )
        };
        if !w.is_finite()
            || !h.is_finite()
            || w <= 0.
            || h <= 0.
            || (w * scale).ceil() > 16384.
            || (h * scale).ceil() > 16384.
            || w * h * scale * scale > 64. * 1024. * 1024.
        {
            return Err("render dimensions exceed cap".into());
        }
        Ok(())
    }
    fn scene(&self, snapshot: &Snapshot) -> Result<Value> {
        let doc = xml::parse(snapshot.svg.as_bytes(), CAP)?;
        let root = doc.get_root_element().ok_or("invalid SVG")?;
        let by_id: HashMap<_, _> = elements(root.clone())
            .into_iter()
            .filter_map(|n| n.get_property_no_ns("id").map(|id| (id, n)))
            .collect();
        static CSS_GEOMETRY: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(
                r"(?i)(?:^|[;{])\s*(?:transform|d|x|y|width|height|r|cx|cy|rx|ry)\s*:",
            )
            .unwrap()
        });
        let stylesheet_geometry = elements(root.clone())
            .iter()
            .any(|n| n.get_name() == "style" && CSS_GEOMETRY.is_match(&n.get_content()));
        let mut content = String::new();
        let mut included = HashMap::new();
        let mut work = 0;
        for (index, id) in snapshot.selection.iter().enumerate() {
            let Some(n) = by_id.get(id) else {
                continue;
            };
            let nodes = elements(n.clone());
            // One bounded query for all eligible isolated roots, never one process per ID.
            if stylesheet_geometry
                || work + nodes.len() > 10_000
                || nodes.iter().any(|n| {
                    !matches!(
                        n.get_name().as_str(),
                        "g" | "rect"
                            | "circle"
                            | "ellipse"
                            | "line"
                            | "polygon"
                            | "polyline"
                            | "path"
                    ) || n.get_properties().iter().any(|(key, value)| {
                        matches!(
                            key.as_str(),
                            "x" | "y" | "width" | "height" | "cx" | "cy" | "r" | "rx" | "ry"
                        ) && value.contains('%')
                    })
                })
            {
                continue;
            }
            if nodes.iter().chain(std::iter::once(&root)).any(|n| {
                n.get_property_no_ns("style").is_some_and(|s| {
                    s.split(';').any(|p| {
                        p.split_once(':').is_some_and(|(k, _)| {
                            matches!(
                                k.trim(),
                                "transform"
                                    | "d"
                                    | "x"
                                    | "y"
                                    | "width"
                                    | "height"
                                    | "r"
                                    | "cx"
                                    | "cy"
                                    | "rx"
                                    | "ry"
                            )
                        })
                    })
                })
            }) {
                continue;
            }
            let temp = libxml::tree::Document::dup_node_into_new_doc(n)
                .map_err(|_| "geometry unavailable")?;
            let mut copy = temp.get_root_element().ok_or("geometry unavailable")?;
            for mut child in elements(copy.clone()) {
                for prop in [
                    "id",
                    "style",
                    "stroke",
                    "stroke-width",
                    "filter",
                    "clip-path",
                    "mask",
                    "display",
                    "visibility",
                    "opacity",
                    "marker",
                    "marker-start",
                    "marker-mid",
                    "marker-end",
                ] {
                    child
                        .remove_property_no_ns(prop)
                        .map_err(|_| "geometry unavailable")?;
                }
                child
                    .set_property("fill", "black")
                    .map_err(|_| "geometry unavailable")?;
            }
            let key = format!("query_{index}");
            copy.set_property("id", &key)
                .map_err(|_| "geometry unavailable")?;
            let bytes = temp.node_to_string(&copy);
            if content.len() + bytes.len() > CAP - 1024 {
                continue;
            }
            content.push_str(&bytes);
            included.insert(id.clone(), key);
            work += nodes.len();
        }
        let boxes = if included.is_empty() {
            HashMap::new()
        } else {
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">{content}</svg>"
            );
            Snapshot::new(svg, vec![], None)
                .ok()
                .and_then(|isolated| self.query(&isolated).ok())
                .unwrap_or_default()
        };
        let selection = snapshot
            .selection
            .iter()
            .map(|id| json!({"id":id,"bbox":included.get(id).and_then(|key|boxes.get(key))}))
            .collect::<Vec<_>>();
        Ok(
            json!({"selection":selection,"viewport":{"zoom":null,"center":null,"visible_region":null},"canvas":snapshot.canvas()?,"visible_objects":snapshot.objects()?}),
        )
    }
}
fn pixel_length(root: &Node, key: &str) -> Result<f64> {
    if root.get_property_no_ns("style").is_some_and(|s| {
        s.split(';').any(|p| {
            p.split_once(':')
                .is_some_and(|(k, _)| matches!(k.trim(), "width" | "height"))
        })
    }) {
        return Err("CSS canvas sizing requires preparation".into());
    }
    let raw = root
        .get_property_no_ns(key)
        .ok_or("absolute canvas dimensions required")?;
    static LENGTH: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"^\s*([+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)\s*(px|mm|cm|in|pt|pc|q)?\s*$",
        )
        .unwrap()
    });
    let c = LENGTH
        .captures(&raw)
        .ok_or("absolute canvas dimensions required")?;
    let factor = match c.get(2).map(|m| m.as_str()).unwrap_or("") {
        "mm" => 96. / 25.4,
        "cm" => 96. / 2.54,
        "in" => 96.,
        "pt" => 96. / 72.,
        "pc" => 16.,
        "q" => 96. / 101.6,
        _ => 1.,
    };
    c[1].parse::<f64>()
        .map(|n| n * factor)
        .map_err(|_| "invalid canvas".into())
}
fn receive(reader: &mut BufReader<TcpStream>) -> Result<Option<Value>> {
    receive_capped(reader, live_protocol::MAX_MESSAGE)
}
fn receive_capped(reader: &mut BufReader<TcpStream>, cap: usize) -> Result<Option<Value>> {
    let deadline = Instant::now() + IDLE;
    let mut bytes = vec![];
    loop {
        reader
            .get_ref()
            .set_read_timeout(Some(
                deadline
                    .saturating_duration_since(Instant::now())
                    .max(Duration::from_millis(1)),
            ))
            .map_err(|_| "socket unavailable")?;
        if Instant::now() >= deadline {
            return Err("socket idle timeout".into());
        }
        let chunk = reader.fill_buf().map_err(|_| "socket unavailable")?;
        if chunk.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err("incomplete frame".into())
            };
        }
        let end = chunk.iter().position(|b| *b == b'\n');
        let count = end.map(|i| i + 1).unwrap_or(chunk.len());
        if bytes.len() + count > cap {
            return Err("frame exceeds cap".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
        reader.consume(count);
        if end.is_some() {
            break;
        }
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "invalid frame")?;
    if !value.is_object() {
        return Err("invalid frame".into());
    }
    Ok(Some(value))
}
fn send(stream: &mut TcpStream, value: &Value) -> Result<()> {
    // The document and PNG limits bound serialization; enforce the wire cap including newline.
    let mut bytes = serde_json::to_vec(value).map_err(|_| "invalid response")?;
    if bytes.len() + 1 > live_protocol::MAX_MESSAGE {
        return Err("response exceeds cap".into());
    }
    bytes.push(b'\n');
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|_| "socket unavailable")?;
    stream
        .write_all(&bytes)
        .map_err(|_| "socket unavailable".into())
}
fn serve(stream: TcpStream, token: &str, snapshot: &mut Snapshot, raster: &Raster) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|_| "socket unavailable")?);
    let mut stream = stream;
    let first = receive(&mut reader)?.ok_or("missing hello")?;
    if first["cmd"] != "hello" || first["token"] != token || first["v"] != live_protocol::VERSION {
        send(
            &mut stream,
            &json!({"v":5,"ok":false,"error":"unauthorized"}),
        )?;
        return Ok(());
    }
    send(
        &mut stream,
        &json!({"v":5,"ok":true,"result":{"protocol_version":5,"capabilities":live_protocol::Command::ALL.iter().skip(1).map(|c|c.name()).collect::<Vec<_>>(),"inkscape_version":null}}),
    )?;
    while let Some(msg) = receive(&mut reader)? {
        if msg["token"] != token || msg["v"] != live_protocol::VERSION {
            send(
                &mut stream,
                &json!({"v":5,"ok":false,"error":"unauthorized"}),
            )?;
            break;
        }
        let cmd = msg["cmd"].as_str().unwrap_or("");
        let params = msg
            .get("params")
            .filter(|p| p.is_object())
            .cloned()
            .unwrap_or(json!({}));
        let result = match cmd {
            "render_view" => raster.render(snapshot, &params, false),
            "export_selection" => raster.render(snapshot, &params, true),
            "get_scene" => raster.scene(snapshot),
            "apply_to_selection" | "set_selected_text" | "insert_svg" => {
                snapshot.mutate(cmd, &params).map_err(str::to_string)
            }
            _ => snapshot.read(cmd, &params).map_err(str::to_string),
        };
        let reply = match result {
            Ok(value) => json!({"v":5,"ok":true,"result":value}),
            Err(_) => json!({"v":5,"ok":false,"error":"command failed"}),
        };
        send(&mut stream, &reply)?;
    }
    Ok(())
}
fn run() -> Result<()> {
    let mut ids = vec![];
    let mut input = None;
    for arg in std::env::args().skip(1) {
        if let Some(id) = arg.strip_prefix("--id=") {
            if id.len() > 4096 || ids.len() >= 10_000 {
                return Err("invalid selection".into());
            }
            ids.push(id.to_string());
        } else if arg.starts_with("--selected-nodes=") {
        } else if arg.starts_with('-') || input.replace(PathBuf::from(arg)).is_some() {
            return Err("invalid arguments".into());
        }
    }
    let input = workspace::normalize_macos_var_alias(&input.ok_or("SVG input required")?);
    let svg = String::from_utf8(ws().read(0, relative(&input)?, CAP)?)
        .map_err(|_| "invalid SVG encoding")?;
    let doc = xml::parse(svg.as_bytes(), CAP)?;
    let root = doc.get_root_element().ok_or("invalid SVG")?;
    if ids.len() == 1 && root.get_property_no_ns("id").as_ref() == ids.first() {
        ids.clear();
    }
    let path = root.get_property_ns(
        "docname",
        "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
    );
    let mut snapshot = Snapshot::new(svg, ids, path)?;
    let raster = Raster {
        binary: process::inkscape_binary().ok_or("Inkscape required")?,
    };
    let rendezvous = std::env::var_os("INKSCAPE_MCP_LIVE_RENDEZVOUS")
        .map(PathBuf::from)
        .unwrap_or(
            std::env::temp_dir()
                .canonicalize()
                .map_err(|_| "temp unavailable")?
                .join("inkscape-mcp-live.json"),
        );
    let relative = relative(&rendezvous)?;
    let lock = ws().lock_file(0, &relative.with_extension("lock"))?;
    use std::os::unix::fs::MetadataExt;
    if lock.metadata().map_err(|_| "lock unavailable")?.nlink() != 1 {
        return Err("linked lock refused".into());
    }
    // SAFETY: lock only our pinned regular-file descriptor, never an Inkscape process.
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err("bridge already active".into());
    }
    ws().regular_info(0, relative, false)?;
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).map_err(|_| "socket unavailable")?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "socket unavailable")?;
    let token = uuid::Uuid::new_v4().simple().to_string();
    let payload=serde_json::to_vec(&json!({"port":listener.local_addr().map_err(|_|"socket unavailable")?.port(),"token":token,"protocol_version":5,"pid":std::process::id()})).map_err(|_|"invalid rendezvous")?;
    ws().atomic_write(0, relative, &payload)?;
    let deadline = Instant::now() + IDLE;
    let result = (|| {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .map_err(|_| "socket unavailable")?;
                    return serve(stream, &token, &mut snapshot, &raster);
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(_) => return Err("socket unavailable".into()),
            }
        }
    })();
    // Preserve a newer advertiser; cleanup never follows a replaced link.
    if ws().read(0, relative, 4096).ok().as_deref() == Some(payload.as_slice()) {
        ws().regular_info(0, relative, true)?;
    }
    // A dropped connection can follow acknowledged edits; publish the accumulated candidate.
    if snapshot.changed() {
        std::io::stdout()
            .lock()
            .write_all(snapshot.svg.as_bytes())
            .map_err(|_| "output failed")?;
    }
    // Socket termination closes the modal session; acknowledged candidates still apply.
    let _ = result;
    Ok(())
}
fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_preserve_coalesced_messages_and_reject_bad_json() {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(server);
        client
            .write_all(b"{\"cmd\":\"ping\"}\n{\"cmd\":\"hello\"}\n[]\n")
            .unwrap();
        assert_eq!(receive(&mut reader).unwrap().unwrap()["cmd"], "ping");
        assert_eq!(receive(&mut reader).unwrap().unwrap()["cmd"], "hello");
        assert!(receive(&mut reader).is_err());
        client.write_all(b"{\"cmd\":\"long\"}\n").unwrap();
        assert!(receive_capped(&mut reader, 8).is_err());
        // Rejecting an oversized frame terminates the session; use a fresh peer for EOF.
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(server);
        client.write_all(b"partial").unwrap();
        drop(client);
        assert!(receive(&mut reader).is_err());
    }
    #[test]
    fn render_asset_and_dimension_refusals_precede_any_process() {
        for body in [
            "<image href='file:///secret'/>",
            "<rect fill='url(http://example.com/a)'/>",
            "<script/>",
            "<style>@import 'file:///secret';</style>",
            "<g xml:base='file:///secret'/>",
        ] {
            assert!(validate_assets(&format!("<svg>{body}</svg>")).is_err());
        }
        assert!(validate_assets("<!DOCTYPE svg SYSTEM 'file:///secret'><svg/>").is_err());
        assert!(validate_assets("<svg><rect fill='url(#local)'/></svg>").is_ok());
        assert!(validate_assets("<svg xmlns:ink='http://www.inkscape.org/namespaces/inkscape'><g ink:label='curly\\name'/></svg>").is_ok());
        assert!(
            validate_assets("<svg><style>rect {fill:red; stroke:url(#local)}</style></svg>")
                .is_ok()
        );
        let raster = Raster {
            binary: PathBuf::from("/nonexistent"),
        };
        let snapshot = Snapshot::new(
            "<svg xmlns='http://www.w3.org/2000/svg' width='1e100' height='100'/>".into(),
            vec![],
            None,
        )
        .unwrap();
        assert!(raster.dimensions(&snapshot, None, 1., false).is_err());
        assert!(
            raster
                .dimensions(&snapshot, Some(&[0., 0., 10000., 10000.]), 1., false)
                .is_err()
        );
    }
}
