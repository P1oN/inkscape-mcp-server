//! Focused frame comparison. Artifact-only, no live document mutation or retry.
use crate::{live::Live, live_records, workspace::Workspace};
use serde_json::{Value, json};
use std::{
    io::Cursor,
    path::{Component, Path},
};
const DECODE_BUDGET: usize = 512 * 1024 * 1024;
struct Image {
    w: u32,
    h: u32,
    rgba: Vec<u8>,
}
fn frame(workspace: &Workspace, path: &str) -> Result<Image, String> {
    let p = Path::new(path);
    if !p.starts_with(".inkscape-mcp/live/artifacts")
        || p.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("a before/after frame for this operation is unavailable".into());
    }
    let (size, _) = workspace
        .regular_info(0, p, false)
        .map_err(|_| "a before/after frame for this operation is unavailable")?
        .ok_or("a before/after frame for this operation is unavailable")?;
    if size as usize > workspace.max_input {
        return Err("a before/after frame is too large to diff".into());
    }
    let bytes = workspace
        .read(0, p, workspace.max_input)
        .map_err(|_| "a before/after frame for this operation is unavailable")?;
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits {
        bytes: DECODE_BUDGET / 3,
    });
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder
        .read_info()
        .map_err(|_| "a before/after frame could not be decoded safely")?;
    let (w, h) = (reader.info().width, reader.info().height);
    let pixels = (w as u64) * (h as u64);
    let cap = u64::from(crate::render::cap());
    if pixels > 2 * cap * cap || pixels > DECODE_BUDGET as u64 / 12 {
        return Err("a before/after frame exceeds the pixel limit".into());
    }
    let size = reader
        .output_buffer_size()
        .ok_or("a before/after frame exceeds the pixel limit")?;
    let mut data = vec![0; size];
    let output = reader
        .next_frame(&mut data)
        .map_err(|_| "a before/after frame could not be decoded safely")?;
    let mut rgba = Vec::with_capacity(pixels as usize * 4);
    let sixteen = output.bit_depth == png::BitDepth::Sixteen;
    let stride = output.color_type.samples() * if sixteen { 2 } else { 1 };
    for raw in data[..output.buffer_size()].chunks_exact(stride) {
        let mut p = [0u8; 4];
        if sixteen {
            for (i, bytes) in raw.as_chunks::<2>().0.iter().enumerate() {
                p[i] = if output.color_type == png::ColorType::Grayscale {
                    u16::from_be_bytes([bytes[0], bytes[1]]).min(255) as u8
                } else {
                    bytes[0]
                };
            }
        } else {
            p[..raw.len()].copy_from_slice(raw);
        }
        let color = match output.color_type {
            png::ColorType::Rgba => [p[0], p[1], p[2], p[3]],
            png::ColorType::Rgb => [p[0], p[1], p[2], 255],
            png::ColorType::Grayscale => [p[0], p[0], p[0], 255],
            png::ColorType::GrayscaleAlpha => [p[0], p[0], p[0], p[1]],
            _ => return Err("a before/after frame could not be decoded safely".into()),
        };
        rgba.extend(color);
    }
    if rgba.len() != pixels as usize * 4 {
        return Err("a before/after frame could not be decoded safely".into());
    }
    Ok(Image { w, h, rgba })
}
fn changed(before: &Image, after: &Image) -> Option<[u32; 4]> {
    let mut bounds = [after.w, after.h, 0, 0];
    let mut found = false;
    for (i, (a, b)) in before
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(after.rgba.as_chunks::<4>().0.iter())
        .enumerate()
    {
        if a[..3] != b[..3] {
            let x = i as u32 % after.w;
            let y = i as u32 / after.w;
            bounds[0] = bounds[0].min(x);
            bounds[1] = bounds[1].min(y);
            bounds[2] = bounds[2].max(x + 1);
            bounds[3] = bounds[3].max(y + 1);
            found = true;
        }
    }
    found.then_some(bounds)
}
fn rectangle(image: &mut Image, r: [i64; 4], color: [u8; 4]) {
    let mut point = |x: i64, y: i64| {
        if x >= 0 && y >= 0 && x < i64::from(image.w) && y < i64::from(image.h) {
            let i = (y as usize * image.w as usize + x as usize) * 4;
            image.rgba[i..i + 4].copy_from_slice(&color);
        }
    };
    // Match source inclusive horizontal endpoints and exclusive vertical endpoint.
    // Very narrow outlines retain Pillow's overlapping/reversed vertical spans.
    for inset in 0..2 {
        for y in [r[1] + inset, r[3] - inset] {
            for x in r[0]..=r[2] {
                point(x, y);
            }
        }
        let from = r[1] + 2;
        let to = r[3] - 1;
        let step = if to < from { -1 } else { 1 };
        for x in [r[2] - inset, r[0] + inset] {
            for i in 0..(to - from).abs() {
                point(x, from + i * step);
            }
        }
    }
}
fn selected(scene: &Value, w: u32, h: u32) -> Vec<(String, [i64; 4])> {
    let Some(cw) = scene["canvas"]["width"]
        .as_f64()
        .filter(|n| n.is_finite() && *n > 0.)
    else {
        return vec![];
    };
    let Some(ch) = scene["canvas"]["height"]
        .as_f64()
        .filter(|n| n.is_finite() && *n > 0.)
    else {
        return vec![];
    };
    let mut result = Vec::new();
    for item in scene["selection"].as_array().into_iter().flatten() {
        let Some(id) = item["id"].as_str() else {
            continue;
        };
        let b = &item["bbox"];
        let Some(v) = ["x", "y", "width", "height"]
            .map(|k| b[k].as_f64())
            .into_iter()
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let p = [
            v[0] * f64::from(w) / cw,
            v[1] * f64::from(h) / ch,
            (v[0] + v[2]) * f64::from(w) / cw,
            (v[1] + v[3]) * f64::from(h) / ch,
        ];
        if p.iter().any(|n| !n.is_finite()) {
            continue;
        }
        let p = p.map(|n| n.round_ties_even() as i64);
        let x0 = p[0].min(p[2]).max(0);
        let x1 = p[0].max(p[2]).min(i64::from(w));
        let y0 = p[1].min(p[3]).max(0);
        let y1 = p[1].max(p[3]).min(i64::from(h));
        if x1 > x0 && y1 > y0 {
            result.push((id.into(), [x0, y0, x1, y1]));
        }
    }
    result
}
pub fn diff(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let scene = live
        .session
        .require_transport()
        .and_then(|t| t.scene())
        .unwrap_or(Value::Null);
    let operation = args["operation_id"]
        .as_str()
        .ok_or("operation_id must be a string")?;
    diff_scene(workspace, operation, &scene)
}
pub fn diff_scene(workspace: &Workspace, operation: &str, scene: &Value) -> Result<Value, String> {
    if !crate::live_records::id(operation) {
        return Err("no live operation with that id".into());
    }
    if workspace.roots.is_empty() {
        return Err("no workspace root configured to store the live diff".into());
    }
    let mut record = live_records::get(workspace, operation)?;
    let before = record["previews"]["before"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("this operation has no before/after frames to diff")?;
    let after = record["previews"]["after"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("this operation has no before/after frames to diff")?;
    let before = frame(workspace, before)?;
    let mut after = frame(workspace, after)?;
    if before.w != after.w || before.h != after.h {
        return Err("before/after frames have different dimensions and cannot be diffed".into());
    }
    let changed = changed(&before, &after);
    let selection = selected(scene, after.w, after.h);
    for (_, r) in &selection {
        rectangle(&mut after, *r, [0, 200, 255, 255]);
    }
    if let Some(r) = changed {
        rectangle(&mut after, r.map(i64::from), [255, 0, 0, 255]);
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, after.w, after.h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .map_err(|_| "live diff could not be encoded")?
            .write_image_data(&after.rgba)
            .map_err(|_| "live diff could not be encoded")?;
    }
    if bytes.len() > workspace.max_output {
        return Err(format!(
            "Error calling tool 'live_diff_view': output file exceeds max size: {} > {} bytes",
            bytes.len(),
            workspace.max_output
        ));
    }
    for directory in [
        ".inkscape-mcp/live/artifacts",
        ".inkscape-mcp/live/operations",
    ] {
        workspace
            .ensure_directory(0, Path::new(directory))
            .map_err(|_| "live diff could not be stored safely")?;
    }
    let path = format!(".inkscape-mcp/live/artifacts/live-diff-{operation}.png");
    workspace
        .atomic_write(0, Path::new(&path), &bytes)
        .map_err(|_| "live diff could not be stored safely")?;
    let mut artifacts = record["diff_artifacts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    artifacts.push(json!(path));
    let _ = live_records::update(workspace, &mut record, json!({"diff_artifacts":artifacts}));
    Ok(
        json!({"operation_id":operation,"artifact_path":path,"changed_bbox":changed.map(|r|json!({"x":r[0] as f64,"y":r[1] as f64,"width":(r[2]-r[0])as f64,"height":(r[3]-r[1])as f64})),"width":after.w,"height":after.h,"highlighted_ids":selection.iter().map(|(id,_)|id).collect::<Vec<_>>()}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    #[test]
    fn frame_limits_and_no_follow_guard_precede_pixel_allocation() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[1, 2, 3, 4])
                .unwrap();
        }
        workspace
            .atomic_write(
                0,
                Path::new(".inkscape-mcp/live/artifacts/frame.png"),
                &bytes,
            )
            .unwrap();
        std::fs::write(outside.path().join("external.png"), &bytes).unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("external.png"),
            root.path().join(".inkscape-mcp/live/artifacts/link.png"),
        )
        .unwrap();
        assert!(frame(&workspace, ".inkscape-mcp/live/artifacts/link.png").is_err());
        assert_eq!(
            std::fs::read(outside.path().join("external.png")).unwrap(),
            bytes
        );
        let small = Workspace {
            max_input: 4,
            roots: workspace.roots.clone(),
            max_output: 4096,
        };
        assert_eq!(
            frame(&small, ".inkscape-mcp/live/artifacts/frame.png")
                .err()
                .unwrap(),
            "a before/after frame is too large to diff"
        );
        bytes[16..20].copy_from_slice(&100_000u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&100_000u32.to_be_bytes());
        let mut crc = 0xffffffffu32;
        for b in &bytes[12..29] {
            crc ^= u32::from(*b);
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xedb88320
                } else {
                    crc >> 1
                };
            }
        }
        bytes[29..33].copy_from_slice(&(!crc).to_be_bytes());
        workspace
            .atomic_write(
                0,
                Path::new(".inkscape-mcp/live/artifacts/bomb.png"),
                &bytes,
            )
            .unwrap();
        assert_eq!(
            frame(&workspace, ".inkscape-mcp/live/artifacts/bomb.png")
                .err()
                .unwrap(),
            "a before/after frame exceeds the pixel limit"
        );
    }
    #[test]
    fn source_diff_and_annotation_cases_match_exact_rgba_pixels() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-diff-cases.json"
        ))
        .unwrap();
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 1024 * 1024,
            max_output: 1024 * 1024,
        };
        for (i, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            let before = ".inkscape-mcp/live/artifacts/before.png";
            let after = ".inkscape-mcp/live/artifacts/after.png";
            workspace
                .atomic_write(
                    0,
                    Path::new(before),
                    &STANDARD.decode(c["before"].as_str().unwrap()).unwrap(),
                )
                .unwrap();
            workspace
                .atomic_write(
                    0,
                    Path::new(after),
                    &STANDARD.decode(c["after"].as_str().unwrap()).unwrap(),
                )
                .unwrap();
            let before = frame(&workspace, before).unwrap();
            let mut after = frame(&workspace, after).unwrap();
            let bounds = changed(&before, &after);
            let bbox=bounds.map(|r|json!({"x":r[0]as f64,"y":r[1]as f64,"width":(r[2]-r[0])as f64,"height":(r[3]-r[1])as f64}));
            assert_eq!(json!(bbox), c["changed_bbox"], "bbox {i}");
            let rects = selected(&c["scene"], after.w, after.h);
            assert_eq!(
                json!(rects.iter().map(|(id, _)| id).collect::<Vec<_>>()),
                c["highlighted_ids"],
                "ids {i}"
            );
            for (_, r) in rects {
                rectangle(&mut after, r, [0, 200, 255, 255]);
            }
            if let Some(r) = bounds {
                rectangle(&mut after, r.map(i64::from), [255, 0, 0, 255]);
            }
            assert_eq!(STANDARD.encode(&after.rgba), c["rgba"], "pixels {i}");
        }
    }
}
