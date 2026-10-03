//! Public headless render/export: private staging, bounded verification and secure adoption.
use crate::{
    arguments,
    document::{Entry, Registry},
    inspect, process,
    workspace::Workspace,
    xml,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use regex::Regex;
use serde_json::{Value, json};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^A-Za-z0-9_.-]+").unwrap());
static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_.:-]*$").unwrap());
static NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*([0-9]*\.?[0-9]+)").unwrap());

pub fn integer(args: &Value, key: &str) -> Result<Option<i64>, String> {
    if args.get(key).is_none_or(Value::is_null) {
        return Ok(None);
    }
    if let Some(i) = args[key].as_i64() {
        return Ok(Some(i));
    }
    if let Some(b) = args[key].as_bool() {
        return Ok(Some(i64::from(b)));
    }
    if let Some(s) = args[key].as_str()
        && let Ok(n) = s.trim().parse()
    {
        return Ok(Some(n));
    }
    Err(format!("{key} must be an integer"))
}

pub fn cap() -> u32 {
    std::env::var("INKSCAPE_MCP_MAX_EXPORT_PX")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .filter(|n| *n >= 1)
        .unwrap_or(8192)
}

pub fn dimensions(summary: &Value, width: Option<i64>) -> Result<(i64, i64), String> {
    let (w, h) = estimated_dimensions(summary, width);
    if w > i64::from(cap()) || h > i64::from(cap()) {
        return Err(format!(
            "export dimensions exceed cap: {w}x{h} > {}px per side",
            cap()
        ));
    }
    Ok((w, h))
}

pub(crate) fn estimated_dimensions(summary: &Value, width: Option<i64>) -> (i64, i64) {
    let num = |key: &str| {
        summary[key]
            .as_str()
            .and_then(|s| NUMBER.captures(s))
            .and_then(|c| c[1].parse::<f64>().ok())
            .filter(|n| *n > 0.0)
    };
    let intrinsic = summary["viewbox"]
        .as_array()
        .and_then(|v| Some((v[2].as_f64()?, v[3].as_f64()?)))
        .filter(|(w, h)| *w > 0.0 && *h > 0.0)
        .or_else(|| Some((num("width")?, num("height")?)));
    let (w, h) = match (width, intrinsic) {
        (Some(w), Some((a, b))) => (w, (w as f64 * b / a).round_ties_even().max(1.0) as i64),
        (Some(w), None) => (w, w),
        (None, Some((a, b))) => (
            a.round_ties_even().max(1.0) as i64,
            b.round_ties_even().max(1.0) as i64,
        ),
        _ => (1024, 1024),
    };
    (w, h)
}

pub(crate) fn safe_object_id(id: &str) -> bool {
    ID.is_match(id)
}

pub(crate) fn name(value: &str) -> String {
    NAME.replace_all(value, "-")
        .trim_matches(['-', '.'])
        .to_owned()
}
fn stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

pub(crate) fn input(
    workspace: &Workspace,
    entry: &Entry,
) -> Result<crate::engine_input::Prepared, String> {
    input_source(
        workspace,
        entry.root,
        &entry.working(),
        Path::new(&entry.source),
    )
}
fn input_source(
    workspace: &Workspace,
    root_index: usize,
    source: &Path,
    origin: &Path,
) -> Result<crate::engine_input::Prepared, String> {
    let bytes = workspace.read(root_index, source, workspace.max_input)?;
    crate::engine_input::prepare(workspace, root_index, origin, &bytes)
}

fn relative_path(target: &Path, parent: &Path) -> String {
    let target = target.components().collect::<Vec<_>>();
    let parent = parent.components().collect::<Vec<_>>();
    let common = target
        .iter()
        .zip(&parent)
        .take_while(|(a, b)| a == b)
        .count();
    let mut path = PathBuf::new();
    for _ in &parent[common..] {
        path.push("..");
    }
    for part in &target[common..] {
        path.push(part.as_os_str());
    }
    path.to_string_lossy().into_owned()
}

fn rebase_svg(bytes: &[u8], stage: &Path, destination: &Path) -> Result<Vec<u8>, String> {
    let document = xml::parse(bytes, bytes.len())?;
    for mut node in
        crate::document::elements(document.get_root_element().ok_or("render/export failed")?)
    {
        for namespace in [None, Some("http://www.w3.org/1999/xlink")] {
            let href = match namespace {
                None => node.get_property_no_ns("href"),
                Some(ns) => node.get_property_ns("href", ns),
            };
            let Some(href) = href else { continue };
            if href.is_empty() || href.starts_with('#') || href.contains(':') {
                continue;
            }
            let (resource, fragment) = href
                .split_once('#')
                .map(|(path, fragment)| (path, format!("#{fragment}")))
                .unwrap_or((&href, String::new()));
            let path = stage.join(crate::engine_input::uri_path(resource)?);
            let mut normalized = PathBuf::new();
            for part in path.components() {
                if part == std::path::Component::ParentDir {
                    normalized.pop();
                } else if part != std::path::Component::CurDir {
                    normalized.push(part);
                }
            }
            let relative = relative_path(&normalized, destination);
            let restored = format!(
                "{}{fragment}",
                crate::engine_input::path_uri(Path::new(&relative))?
            );
            if let Some(namespace) = namespace {
                crate::identity::set_namespaced(
                    &mut node, &document, namespace, "href", &restored,
                )?;
            } else {
                node.set_property("href", &restored)
                    .map_err(|_| "render/export failed")?;
            }
        }
    }
    Ok(xml::serialize(&document))
}

fn raster(bytes: &[u8]) -> Option<(u32, u32, usize, bool)> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits {
        bytes: 512 * 1024 * 1024,
    });
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    if reader.info().width > cap() || reader.info().height > cap() {
        return None;
    }
    let size = reader.output_buffer_size()?;
    if size > 512 * 1024 * 1024 {
        return None;
    }
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer).ok()?;
    let data = &buffer[..info.buffer_size()];
    let channels = info.color_type.samples();
    let total = info.width as usize * info.height as usize;
    let alpha = matches!(
        info.color_type,
        png::ColorType::Rgba | png::ColorType::GrayscaleAlpha
    );
    let opaque = if alpha {
        data.chunks_exact(channels)
            .filter(|p| p[channels - 1] != 0)
            .count()
    } else {
        total
    };
    let blank = if alpha {
        opaque == 0
    } else {
        data.chunks_exact(channels).all(|p| p == &data[..channels])
    };
    Some((info.width, info.height, opaque, blank))
}

/// Return the full MCP tool result so inline PNG text and structured metadata remain distinct.
pub fn call(registry: &Registry, tool: &str, args: &Value) -> Result<Value, String> {
    if tool == "compare_region" {
        return compare_region(registry, args);
    }
    call_source(registry, tool, args, None, false, None)
}

pub(crate) fn print_profile(registry: &Registry, args: &Value) -> Result<Value, String> {
    call_source(registry, "export_document", args, None, true, None)
}

struct PendingArtifact {
    root: usize,
    path: PathBuf,
    bytes: Vec<u8>,
}
fn publish_pair(workspace: &Workspace, pending: &[PendingArtifact]) -> Result<(), String> {
    for (index, artifact) in pending.iter().enumerate() {
        if let Err(error) = workspace.atomic_create(artifact.root, &artifact.path, &artifact.bytes)
        {
            let mut recovery_required = false;
            for published in &pending[..index] {
                if workspace
                    .read(published.root, &published.path, workspace.max_output)
                    .is_ok_and(|bytes| bytes == published.bytes)
                {
                    if workspace
                        .remove_file(published.root, &published.path)
                        .is_err()
                    {
                        recovery_required = true;
                    }
                } else {
                    recovery_required = true;
                }
            }
            // Never remove the failing destination blindly: it may have existed
            // before the attempt. An inspection failure also requires recovery.
            recovery_required |=
                !matches!(workspace.file_kind(artifact.root, &artifact.path), Ok(None));
            return Err(if recovery_required {
                format!(
                    "comparison publication failed; inspect these artifacts before retrying: {} ({error})",
                    serde_json::Value::Array(
                        pending
                            .iter()
                            .map(|item| workspace.artifact_link(item.root, &item.path))
                            .collect()
                    )
                )
            } else {
                error
            });
        }
    }
    Ok(())
}

fn call_source(
    registry: &Registry,
    tool: &str,
    args: &Value,
    historical: Option<&Path>,
    print_profile: bool,
    pending: Option<&mut Vec<PendingArtifact>>,
) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let capture = tool == "capture_frame";
    let preview = tool == "render_preview" || capture;
    let object = if capture {
        None
    } else {
        arguments::string(args, "object_id")?
    };
    let inline_requested = arguments::boolean(args, "inline", true)?;
    let threshold = integer(args, "max_output_bytes")?
        .filter(|n| *n > 0)
        .unwrap_or(5 * 1024 * 1024) as usize;
    if let Some(oid) = object
        && !ID.is_match(oid)
    {
        return Err(if preview {
            "object id is not a safe svg id"
        } else {
            "object id is not valid"
        }
        .into());
    }
    let region_value = args
        .get("region")
        .filter(|v| tool == "render_preview" && !v.is_null());
    if region_value.is_some() && object.is_some() {
        return Err("choose object_id or region, not both".into());
    }
    let fmt = if preview {
        "png".into()
    } else {
        args["format"]
            .as_str()
            .unwrap_or(if tool == "export_object" { "png" } else { "" })
            .trim()
            .to_lowercase()
    };
    if !matches!(fmt.as_str(), "png" | "pdf" | "svg") {
        return Err("unsupported export format (expected one of: png, pdf, svg); call list_capabilities to see the supported export formats".into());
    }
    if tool == "export_object" {
        let oid = object.ok_or("object_id must be a string")?;
        if !ID.is_match(oid) {
            return Err("object id is not valid".into());
        }
        if !inspect::resource(registry, id, "objects")?["objects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["id"] == oid)
        {
            return Err("object id not found in document".into());
        }
    }
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let working = entry.working();
    let source = historical.unwrap_or(&working);
    let mut width = integer(args, "width_px")?;
    if capture && width.is_some_and(|w| w < 1) {
        return Err("render/export failed".into());
    }
    let region = if let Some(value) = region_value {
        width = Some(width.unwrap_or(512));
        let bytes = registry
            .workspace
            .read(entry.root, source, registry.workspace.max_input)?;
        let document = xml::parse(&bytes, registry.workspace.max_input)?;
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        Some(crate::geometry::Region::build(
            &root,
            value,
            width.unwrap(),
            arguments::string(args, "background")?.unwrap_or("transparent"),
        )?)
    } else {
        None
    };
    let dims = if let Some(region) = &region {
        Some((region.width, region.height))
    } else if fmt == "png" {
        Some(
            dimensions(&registry.summary(id)?, width)
                .map_err(|_| "export exceeds the configured size or dimension limit")?,
        )
    } else {
        None
    };
    let object_preview_prefix = (preview && object.is_some()).then(|| {
        format!(
            "preview-{}-{}",
            stamp(),
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        )
    });
    let prefix = if object_preview_prefix.is_some() {
        object_preview_prefix.as_deref()
    } else if preview {
        arguments::string(args, "name")?
    } else {
        arguments::string(args, "name_prefix")?
    };
    let descriptor = format!(
        "{}{}",
        object.map(|o| format!("obj-{o}-")).unwrap_or_default(),
        if fmt == "png" {
            width.map(|w| format!("{w}px")).unwrap_or("auto".into())
        } else {
            "auto".into()
        }
    );
    let filename = if region.is_some() {
        format!(
            "detail-{}-{}.png",
            stamp(),
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        )
    } else if preview && object.is_none() {
        let prefix = prefix
            .map(name)
            .filter(|s| !s.is_empty())
            .map(|p| format!("{p}-"))
            .unwrap_or_default();
        format!(
            "preview-{prefix}{descriptor}-{}-{}.png",
            stamp(),
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        )
    } else {
        let basename = name(
            Path::new(&entry.source)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document"),
        );
        let basename = if basename.is_empty() {
            "document"
        } else {
            &basename
        };
        let prefix = prefix
            .map(name)
            .filter(|s| !s.is_empty())
            .map(|p| format!("{p}-"))
            .unwrap_or_default();
        format!(
            "{}-{prefix}{basename}-{descriptor}-{}.{fmt}",
            stamp(),
            uuid::Uuid::new_v4().simple()
        )
    };
    let prepared = input_source(
        &registry.workspace,
        entry.root,
        source,
        Path::new(&entry.source),
    )?;
    let frame = if capture {
        Some(crate::frames::plan(registry, entry, args)?)
    } else {
        None
    };
    let (root, relative) = if let Some(frame) = &frame {
        (entry.root, frame.relative.clone())
    } else if let Some(out) = arguments::string(args, "out_dir")? {
        let directory = if Path::new(out).is_absolute() {
            PathBuf::from(out)
        } else {
            registry.workspace.roots[entry.root].join(out)
        };
        registry.workspace.resolve_output(
            directory
                .join(&filename)
                .to_str()
                .ok_or("path rejected: invalid characters")?,
            None,
        )?
    } else {
        (
            entry.root,
            entry
                .directory()
                .join(if preview && object.is_none() {
                    "artifacts/preview"
                } else {
                    "artifacts/exports"
                })
                .join(&filename),
        )
    };
    let binary = process::inkscape_binary().ok_or(
        "render/export failed: the Inkscape engine is unavailable on this runtime; call list_capabilities to see what this runtime supports",
    )?;
    let temporary = tempfile::Builder::new()
        .prefix("imcp-export-")
        .tempdir()
        .map_err(|_| "render staging unavailable")?;
    let stage = Workspace {
        roots: vec![
            temporary
                .path()
                .canonicalize()
                .map_err(|_| "render staging unavailable")?,
        ],
        max_input: registry.workspace.max_input,
        max_output: registry.workspace.max_output,
    };
    stage.write_new(0, Path::new("input.svg"), &prepared.bytes)?;
    let output = stage.roots[0].join(format!("output.{fmt}"));
    let mut argv = vec![
        stage.roots[0]
            .join("input.svg")
            .to_string_lossy()
            .into_owned(),
    ];
    if let Some(object) = object {
        argv.extend([format!("--export-id={object}"), "--export-id-only".into()]);
    }
    match fmt.as_str() {
        "png" => argv.push("--export-type=png".into()),
        "svg" => argv.extend(["--export-type=svg".into(), "--export-plain-svg".into()]),
        _ => (),
    }
    argv.push(format!("--export-filename={}", output.display()));
    if fmt == "png"
        && let Some(w) = width
    {
        if w <= 0 {
            return Err("width_px must be positive".into());
        }
        argv.push(format!("--export-width={w}"));
    }
    if let Some(region) = &region {
        argv.extend([
            format!(
                "--export-area={}",
                region
                    .bounds
                    .iter()
                    .map(|v| format!("{v:.16e}"))
                    .collect::<Vec<_>>()
                    .join(":")
            ),
            format!("--export-background={}", region.background),
            format!(
                "--export-background-opacity={}",
                if region.background == "transparent" {
                    "0"
                } else {
                    "1"
                }
            ),
        ]);
    } else if object.is_none() && fmt != "svg" {
        argv.push("--export-area-page".into());
    }
    if print_profile {
        argv.extend([
            "--export-pdf-version=1.4".into(),
            "--export-text-to-path".into(),
        ]);
    }
    let warm = object.is_none()
        && region.is_none()
        && !print_profile
        && matches!(fmt.as_str(), "png" | "svg")
        && crate::engine::export(
            &registry.workspace.roots[entry.root].join(entry.working()),
            &binary,
            &stage.roots[0].join("input.svg"),
            &output,
            &fmt,
            if fmt == "png" { width } else { None },
        )
        .is_ok()
        && stage
            .regular_info(0, Path::new(&format!("output.{fmt}")), false)?
            .is_some();
    if !warm {
        // A failed private shell can leave a partial staged artifact. Unlink only
        // its owned output entry before the fixed argument-list fallback.
        let _ = stage.remove_file(0, Path::new(&format!("output.{fmt}")));
        let outcome = process::run(&binary, &argv, process::timeout()).map_err(|_|"render/export failed: the Inkscape engine is unavailable on this runtime; call list_capabilities to see what this runtime supports")?;
        if outcome.timed_out {
            return Err("render/export failed".into());
        }
        if !outcome.success {
            return Err("render/export failed".into());
        }
    }
    let bytes = stage
        .read(
            0,
            Path::new(&format!("output.{fmt}")),
            registry.workspace.max_output,
        )
        .map_err(|error| {
            if error == "input file exceeds the configured size limit" {
                "export exceeds the configured size or dimension limit"
            } else {
                "render/export failed"
            }
        })?;
    // Enforce actual PNG dimensions even if the estimate used a different physical unit or
    // object bounds. A failed content verification must never bypass the output pixel cap.
    if fmt == "png" {
        let reader = png::Decoder::new(Cursor::new(&bytes))
            .read_info()
            .map_err(|_| "render/export failed")?;
        if reader.info().width > cap() || reader.info().height > cap() {
            return Err("export exceeds the configured size or dimension limit".into());
        }
    }
    let bytes = if fmt == "svg" {
        let document = xml::parse(&bytes, registry.workspace.max_output)?;
        prepared.restore(&document, true, Some(&stage.roots[0]))?;
        rebase_svg(
            &xml::serialize(&document),
            &stage.roots[0],
            registry.workspace.roots[root]
                .join(&relative)
                .parent()
                .unwrap(),
        )?
    } else {
        bytes
    };
    if bytes.len() > registry.workspace.max_output {
        return Err("export exceeds the configured size or dimension limit".into());
    }
    let mut value = json!({"doc_id":id,"artifact_path":relative.to_string_lossy(),"workspace_relative_path":relative.to_string_lossy(),"artifact":registry.workspace.artifact_link(root,&relative),"format":fmt,"width_px":dims.map(|d|d.0),"height_px":dims.map(|d|d.1),"stale":false,"opaque_px":null,"all_blank":null});
    if fmt == "png"
        && let Some((w, h, opaque, blank)) = raster(&bytes)
    {
        value["width_px"] = json!(w);
        value["height_px"] = json!(h);
        value["opaque_px"] = json!(opaque);
        value["all_blank"] = json!(blank);
    }
    if let Some(frame) = &frame {
        value.as_object_mut().unwrap().remove("opaque_px");
        value.as_object_mut().unwrap().remove("all_blank");
        value["series"] = json!(frame.series);
        value["frame_index"] = json!(frame.index);
    }
    if !preview {
        let pdf = if fmt == "pdf" && bytes.starts_with(b"%PDF") {
            let data = &bytes[..bytes.len().min(64 * 1024 * 1024)];
            let contains = |marker: &[u8]| data.windows(marker.len()).any(|w| w == marker);
            Some((
                ![b"/Subtype/Image".as_slice(), b"/Subtype /Image"]
                    .iter()
                    .any(|m| contains(m)),
                ![b"/Type/Font".as_slice(), b"/Type /Font", b"/FontFile"]
                    .iter()
                    .any(|m| contains(m)),
            ))
        } else {
            None
        };
        value["is_vector"] = json!(pdf.map(|p| p.0));
        value["fonts_outlined"] = json!(pdf.map(|p| p.1));
    }
    if let Some(pending) = pending {
        pending.push(PendingArtifact {
            root,
            path: relative.clone(),
            bytes: bytes.clone(),
        });
    } else {
        registry.workspace.write_new(root, &relative, &bytes)?;
    }
    if let (Some(working), Some(artifact)) = (
        registry.workspace.modified(entry.root, &entry.working()),
        registry.workspace.modified(root, &relative),
    ) {
        value["stale"] = json!(working > artifact);
    }
    let inline = inline_requested && fmt == "png" && bytes.len() <= threshold;
    let mut content = Vec::new();
    if inline {
        let mut display = value.clone();
        display.as_object_mut().unwrap().remove("artifact_path");
        display
            .as_object_mut()
            .unwrap()
            .remove("workspace_relative_path");
        display["note"] = json!(
            "Raster attached inline as an image — view it directly. The artifact is stored server-side at a workspace-relative path (see structured output); it is NOT on your local filesystem, so do not Read it."
        );
        content.push(json!({"type":"text","text":display.to_string()}));
        content.push(json!({"type":"image","mimeType":"image/png","data":STANDARD.encode(&bytes)}));
    } else {
        content.push(json!({"type":"text","text":value.to_string()}));
    }
    Ok(json!({"content":content,"structuredContent":value,"isError":false}))
}

fn compare_region(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let snapshot = args["snapshot_id"]
        .as_str()
        .ok_or("snapshot_id must be a string")?;
    let inline = arguments::boolean(args, "inline", true)?;
    let width = integer(args, "width_px")?.unwrap_or(512);
    let background = arguments::string(args, "background")?.unwrap_or("transparent");
    let region = args.get("region").ok_or("region is required")?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let historical =
        crate::transaction::snapshot_source(registry, entry, snapshot).map_err(|e| {
            if e == "snapshot not found" {
                "snapshot id not found".into()
            } else {
                e
            }
        })?;
    let before_bytes =
        registry
            .workspace
            .read(entry.root, &historical, registry.workspace.max_input)?;
    let after_bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let before_doc = xml::parse(&before_bytes, registry.workspace.max_input)?;
    let after_doc = xml::parse(&after_bytes, registry.workspace.max_input)?;
    let before_root = before_doc
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let after_root = after_doc
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    if crate::geometry::root_mapping(&before_root)? != crate::geometry::root_mapping(&after_root)? {
        return Err("canvas mapping differs; fixed-scale comparison refused".into());
    }
    // Prepare both sources before invoking the engine or adopting any artifacts.
    crate::geometry::Region::build(&before_root, region, width, background)?;
    crate::geometry::Region::build(&after_root, region, width, background)?;
    let normalized_region = ["x", "y", "width", "height"]
        .into_iter()
        .map(|key| {
            (
                key.to_owned(),
                json!(arguments::number(&region[key]).unwrap()),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let render_args = json!({"doc_id":id,"region":region,"width_px":width,"background":background,"inline":inline});
    let mut pending = Vec::with_capacity(2);
    let before = call_source(
        registry,
        "render_preview",
        &render_args,
        Some(&historical),
        false,
        Some(&mut pending),
    )?;
    let after = call_source(
        registry,
        "render_preview",
        &render_args,
        None,
        false,
        Some(&mut pending),
    )?;
    publish_pair(&registry.workspace, &pending)?;
    let value = json!({"doc_id":id,"snapshot_id":snapshot,"region":normalized_region,"width_px":width,"background":background,"before":before["structuredContent"]["artifact"],"after":after["structuredContent"]["artifact"],"note":"Same region, scale and background; no artistic quality score."});
    let mut content = vec![json!({"type":"text","text":value.to_string()})];
    if inline {
        for (label, result) in [("before", before), ("after", after)] {
            for block in result["content"].as_array().unwrap() {
                if block["type"] == "image" {
                    content.push(json!({"type":"text","text":label}));
                    content.push(block.clone());
                }
            }
        }
    }
    Ok(json!({"content":content,"structuredContent":value,"isError":false}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_asset_uris_survive_restore_rebase_and_reopen() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("workspace#%? &é");
        std::fs::create_dir(&root).unwrap();
        let workspace = Workspace {
            roots: vec![root.canonicalize().unwrap()],
            max_input: 65536,
            max_output: 65536,
        };
        let root = &workspace.roots[0];
        for name in ["asset#one.png", "asset%two.png", "asset?three.png"] {
            std::fs::write(root.join(name), b"\x89PNG\r\n\x1a\nfixture").unwrap();
        }
        std::fs::write(
            root.join("paint#%?.svg"),
            "<svg><defs><linearGradient id='g'/></defs><rect id='r'/></svg>",
        )
        .unwrap();
        std::fs::write(root.join("sheet#%?.css"), ".x{fill:red}").unwrap();
        let source = br##"<svg xmlns='http://www.w3.org/2000/svg' xmlns:link='http://www.w3.org/1999/xlink'>
            <image href='asset%23one.png'/><image link:href='asset%25two.png'/><image href='asset%3Fthree.png'/>
            <use href='paint%23%25%3F.svg#r'/><rect style="fill:url('paint%23%25%3F.svg#g')"/>
            <style>@import 'sheet%23%25%3F.css';</style><!--href="untouched.png"-->
        </svg>"##;
        let prepared =
            crate::engine_input::prepare(&workspace, 0, Path::new("source.svg"), source).unwrap();
        let staged = xml::parse(&prepared.bytes, workspace.max_input).unwrap();
        prepared.restore(&staged, true, None).unwrap();
        let destination = root.join("exports%#? &é");
        std::fs::create_dir(&destination).unwrap();
        let exported = rebase_svg(
            &xml::serialize(&staged),
            Path::new("/owned/stage#%?"),
            &destination,
        )
        .unwrap();
        let result = xml::parse(&exported, workspace.max_input).unwrap();
        let children = result.get_root_element().unwrap().get_child_elements();
        assert_eq!(
            children[0].get_property_no_ns("href").unwrap(),
            "../asset%23one.png"
        );
        assert_eq!(
            children[1]
                .get_property_ns("href", "http://www.w3.org/1999/xlink")
                .unwrap(),
            "../asset%25two.png"
        );
        assert_eq!(
            children[2].get_property_no_ns("href").unwrap(),
            "../asset%3Fthree.png"
        );
        assert_eq!(
            children[3].get_property_no_ns("href").unwrap(),
            "../paint%23%25%3F.svg#r"
        );
        let css = children[4].get_property_no_ns("style").unwrap();
        assert!(css.contains("paint%23%25%3F.svg#g"));
        assert!(children[5].get_content().contains("sheet%23%25%3F.css"));
        assert!(
            String::from_utf8(exported.clone())
                .unwrap()
                .contains("<!--href=\"untouched.png\"-->")
        );
        // The actual importer must resolve every restored/rebased link, including
        // CSS URLs/imports and fragments, under a destination containing URI syntax.
        crate::engine_input::prepare(
            &workspace,
            0,
            Path::new("exports%#? &é/result.svg"),
            &exported,
        )
        .unwrap();
    }

    #[test]
    fn comparison_publication_collision_rolls_back_new_file_preserves_existing() {
        let temporary = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![temporary.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let pending = [
            PendingArtifact {
                root: 0,
                path: "before.png".into(),
                bytes: b"before".to_vec(),
            },
            PendingArtifact {
                root: 0,
                path: "after.png".into(),
                bytes: b"after".to_vec(),
            },
        ];
        workspace
            .write_new(0, &pending[1].path, b"existing")
            .unwrap();
        let error = publish_pair(&workspace, &pending).unwrap_err();
        assert!(error.contains("inspect these artifacts before retrying"));
        assert!(error.contains("after.png"));
        assert!(!temporary.path().join("before.png").exists());
        assert_eq!(
            workspace.read(0, &pending[1].path, 4096).unwrap(),
            b"existing"
        );
        workspace.remove_file(0, &pending[1].path).unwrap();
        publish_pair(&workspace, &pending).unwrap();
        for artifact in &pending {
            assert_eq!(
                workspace.read(0, &artifact.path, 4096).unwrap(),
                artifact.bytes
            );
        }
    }

    #[test]
    fn comparison_publication_unusable_parent_rolls_back_first_file() {
        let temporary = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![temporary.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        workspace
            .write_new(0, Path::new("blocked"), b"owned existing file")
            .unwrap();
        let pending = [
            PendingArtifact {
                root: 0,
                path: "before.png".into(),
                bytes: b"before".to_vec(),
            },
            PendingArtifact {
                root: 0,
                path: "blocked/after.png".into(),
                bytes: b"after".to_vec(),
            },
        ];
        assert!(publish_pair(&workspace, &pending).is_err());
        assert!(!temporary.path().join("before.png").exists());
        assert_eq!(
            workspace.read(0, Path::new("blocked"), 4096).unwrap(),
            b"owned existing file"
        );
    }

    #[test]
    fn comparison_refuses_mapping_changes_and_tampered_snapshot_paths_before_export() {
        let temporary = tempfile::tempdir().unwrap();
        let mut registry = Registry {
            workspace: Workspace {
                roots: vec![temporary.path().canonicalize().unwrap()],
                max_input: 4096,
                max_output: 65536,
            },
            entries: indexmap::IndexMap::new(),
        };
        let original = b"<svg width='100' height='100' viewBox='0 0 100 100'/>";
        registry
            .workspace
            .write_new(0, Path::new("fixture.svg"), original)
            .unwrap();
        let opened = registry.open(&json!({"path":"fixture.svg"})).unwrap();
        let id = opened["doc_id"].as_str().unwrap();
        let entry = &registry.entries[id];
        let snapshot = crate::transaction::snapshot(&registry, entry, None, None).unwrap();
        let changed = b"<svg width='200' height='100' viewBox='0 0 100 100'/>";
        registry
            .workspace
            .atomic_write(0, &entry.working(), changed)
            .unwrap();
        let args = json!({"doc_id":id,"snapshot_id":snapshot["snapshot_id"],"region":{"x":0,"y":0,"width":10,"height":10},"width_px":100});
        assert_eq!(
            call(&registry, "compare_region", &args).unwrap_err(),
            "canvas mapping differs; fixed-scale comparison refused"
        );
        assert_eq!(
            registry.workspace.read(0, &entry.working(), 4096).unwrap(),
            changed
        );
        let mut tampered = snapshot.clone();
        tampered["file"] = json!("../working/document.svg");
        registry
            .workspace
            .atomic_write(
                0,
                &entry.directory().join("snapshots/index.json"),
                &serde_json::to_vec(&json!({"snapshots":[tampered]})).unwrap(),
            )
            .unwrap();
        assert_eq!(
            call(&registry, "compare_region", &args).unwrap_err(),
            "snapshot id not found"
        );
        assert_eq!(
            registry
                .workspace
                .read(0, Path::new("fixture.svg"), 4096)
                .unwrap(),
            original
        );
        assert!(
            registry
                .workspace
                .directory_entries(0, &entry.directory().join("artifacts/preview"))
                .unwrap_or_default()
                .is_empty()
        );
    }

    #[test]
    fn dimensions_use_viewbox_ties_even_and_reject_excess_height() {
        let summary = json!({"viewbox":[0,0,4,3],"width":"400mm","height":"300mm"});
        assert_eq!(dimensions(&summary, Some(2)).unwrap(), (2, 2));
        assert_eq!(dimensions(&summary, None).unwrap(), (4, 3));
        let tall = json!({"viewbox":[0,0,1,100000]});
        assert!(dimensions(&tall, Some(2)).is_err());
        assert_eq!(dimensions(&json!({}), None).unwrap(), (1024, 1024));
    }

    #[test]
    fn png_truth_counts_alpha_and_rejects_corruption() {
        let encode = |pixels: &[u8]| {
            let mut bytes = Vec::new();
            let mut encoder = png::Encoder::new(&mut bytes, 2, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(pixels)
                .unwrap();
            bytes
        };
        assert_eq!(
            raster(&encode(&[255, 0, 0, 0, 0, 255, 0, 128])),
            Some((2, 1, 1, false))
        );
        assert_eq!(
            raster(&encode(&[255, 0, 0, 0, 0, 255, 0, 0])),
            Some((2, 1, 0, true))
        );
        assert!(raster(b"\x89PNG\r\n\x1a\ntruncated").is_none());
    }

    #[test]
    fn svg_relocation_keeps_fragment_data_and_namespace_references() {
        let bytes = br##"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><image href="../working/asset.png"/><use xlink:href="#r"/><image href="data:image/png;base64,AA=="/></svg>"##;
        let moved = rebase_svg(
            bytes,
            Path::new("/workspace/stage"),
            Path::new("/workspace/artifacts/exports"),
        )
        .unwrap();
        let expected = br##"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><image href="../../working/asset.png"/><use xlink:href="#r"/><image href="data:image/png;base64,AA=="/></svg>"##;
        assert_eq!(moved, xml::serialize(&xml::parse(expected, 4096).unwrap()));
        assert!(rebase_svg(&[255], Path::new("/a"), Path::new("/b")).is_err());
    }
    #[test]
    fn unsafe_assets_refuse_capture_and_custom_export_before_output_directories() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("owned.png"), b"owned").unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let svg = format!(
            "<svg width='16' height='16'><image href='{}' width='16' height='16'/></svg>",
            outside.path().join("owned.png").display()
        );
        workspace
            .write_new(0, Path::new("source.svg"), svg.as_bytes())
            .unwrap();
        let mut registry = Registry {
            workspace,
            entries: indexmap::IndexMap::new(),
        };
        let id = registry.open(&json!({"path":"source.svg"})).unwrap()["doc_id"]
            .as_str()
            .unwrap()
            .to_owned();
        for (tool, args) in [
            ("capture_frame", json!({"doc_id":id})),
            (
                "export_document",
                json!({"doc_id":id,"format":"png","out_dir":"new-output"}),
            ),
        ] {
            assert!(
                call(&registry, tool, &args)
                    .err()
                    .unwrap()
                    .contains("outside workspace")
            );
            assert!(!root.path().join("new-output").exists());
            assert!(
                !root
                    .path()
                    .join(registry.entries[&id].directory())
                    .join("artifacts/frames")
                    .exists()
            );
        }
    }
}
