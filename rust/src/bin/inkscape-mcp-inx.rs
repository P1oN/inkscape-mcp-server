//! Fixed one-shot INX consumer. No interpreter or arbitrary command execution.
#[allow(dead_code)]
#[path = "../workspace.rs"]
mod workspace;
use inkscape_mcp_rust::helper_svg::oneshot;
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
const CAP: usize = 16 * 1024 * 1024;
fn ws() -> workspace::Workspace {
    workspace::Workspace {
        roots: vec!["/".into()],
        max_input: CAP,
        max_output: CAP,
    }
}
fn relative(p: &Path) -> Result<&Path, String> {
    p.strip_prefix("/")
        .map_err(|_| "absolute path required".into())
}
fn run() -> Result<(), String> {
    let root = PathBuf::from(
        std::env::var_os("INKSCAPE_MCP_MANAGED_DIR").ok_or("managed session required")?,
    );
    let mut selection = vec![];
    let mut input = None;
    for arg in std::env::args().skip(1) {
        if let Some(id) = arg.strip_prefix("--id=") {
            if id.is_empty() || id.len() > 4096 || selection.len() >= 10_000 {
                return Err("invalid selection".into());
            }
            selection.push(id.to_string());
        } else if arg.starts_with("--selected-nodes=") {
        } else if arg.starts_with('-') || input.replace(PathBuf::from(arg)).is_some() {
            return Err("invalid INX arguments".into());
        }
    }
    if selection.len() > 10_000 {
        return Err("invalid selection".into());
    }
    let input = workspace::normalize_macos_var_alias(&input.ok_or("SVG input required")?);
    let svg = ws().read(0, relative(&input)?, CAP)?;
    let svg = std::str::from_utf8(&svg).map_err(|_| "invalid SVG encoding")?;
    let request: Value = serde_json::from_slice(&ws().read(
        0,
        &relative(&root)?.join("insert-request.json"),
        2 * 1024 * 1024,
    )?)
    .map_err(|_| "invalid request")?;
    let nonce = request["nonce"].as_str().ok_or("invalid edit identity")?;
    if !inkscape_mcp_rust::helper_svg::valid_nonce(nonce) {
        return Err("invalid edit identity".into());
    }
    let candidate = serde_json::from_value::<oneshot::Request>(request.clone())
        .map_err(|_| "invalid document or selection")
        .and_then(|request| oneshot::prepare(svg, request, &selection, CAP));
    let (reply, output) = match candidate {
        Ok((applied, fp)) => (
            json!({"nonce":nonce,"ok":true,"ids":applied.ids,"fingerprint":fp}),
            applied.bytes,
        ),
        Err(error) => (json!({"nonce":nonce,"ok":false,"error":error}), vec![]),
    };
    ws().regular_info(0, &relative(&root)?.join("insert-result.json"), false)?;
    ws().atomic_write(
        0,
        &relative(&root)?.join("insert-result.json"),
        &serde_json::to_vec(&reply).map_err(|_| "invalid result")?,
    )?;
    std::io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|_| "output failed".into())
}
fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}
