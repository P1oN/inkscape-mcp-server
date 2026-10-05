//! Review a bounded package on an immutable snapshot; apply via one guarded native effect.
use crate::{arguments, live::Live, workspace::Workspace};
use inkscape_mcp_rust::helper_svg::{package, package::Change};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
fn changes(args: &Value) -> Result<Vec<Change>, String> {
    let edits = args["edits"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 16)
        .ok_or("edits must contain 1..16 style/text changes")?;
    edits
        .iter()
        .map(|edit| {
            let fields = edit.as_object().ok_or("each edit must be an object")?;
            match edit["op"].as_str() {
                Some("style") => {
                    if fields.keys().any(|k| {
                        ![
                            "op",
                            "fill",
                            "stroke",
                            "stroke_width",
                            "opacity",
                            "dx",
                            "dy",
                            "scale",
                            "rotate",
                        ]
                        .contains(&k.as_str())
                    }) {
                        return Err("unsupported style edit field".into());
                    }
                    let plan = crate::live_mutation::style_plan(edit)?;
                    Ok(Change::Style {
                        style: plan.properties,
                        transform: plan.transform,
                    })
                }
                Some("text") => {
                    if fields.keys().any(|k| !["op", "text"].contains(&k.as_str())) {
                        return Err("unsupported text edit field".into());
                    }
                    let text = arguments::string(edit, "text")?.ok_or("text is required")?;
                    crate::live_mutation::validate_text(text)?;
                    Ok(Change::Text { text: text.into() })
                }
                _ => Err("package supports only style and text edits".into()),
            }
        })
        .collect()
}
/// No GUI rendering: candidates must pass the same self-contained snapshot gate as discovery.
fn preview(svg: &str, workspace: &Workspace) -> Result<String, String> {
    let doc = crate::live_discovery::snapshot(svg, workspace)?;
    let root = doc.get_root_element().ok_or("invalid preview SVG")?;
    let (sx, sy, _, _) = crate::geometry::root_mapping(&root)?;
    let page = match (
        root.get_property_no_ns("width"),
        root.get_property_no_ns("height"),
    ) {
        (Some(w), Some(h)) => (crate::geometry::length(&w)?, crate::geometry::length(&h)?),
        (None, None) => {
            let scene = crate::live_scene::scene(svg, &[], workspace.max_input)
                .map_err(|e| e.public_message().to_string())?;
            (
                scene["canvas"]["width"]
                    .as_f64()
                    .ok_or("page dimensions required")?,
                scene["canvas"]["height"]
                    .as_f64()
                    .ok_or("page dimensions required")?,
            )
        }
        _ => return Err("page dimensions required".into()),
    };
    let summary = json!({"viewbox":[0.,0.,page.0,page.1]});
    let (width, height) =
        crate::render::dimensions(&summary, Some(800.min(i64::from(crate::render::cap()))))?;
    if !sx.is_finite() || !sy.is_finite() || height < 1 {
        return Err("invalid preview mapping".into());
    }
    let tmp = tempfile::tempdir().map_err(|_| "preview staging unavailable")?;
    let stage = Workspace {
        roots: vec![
            tmp.path()
                .canonicalize()
                .map_err(|_| "preview staging unavailable")?,
        ],
        max_input: workspace.max_input,
        max_output: workspace.max_output,
    };
    stage.write_new(0, Path::new("input.svg"), svg.as_bytes())?;
    let binary =
        crate::process::inkscape_binary().ok_or("Inkscape CLI required for package preview")?;
    let outcome = crate::process::run_bounded(
        &binary,
        &[
            stage.roots[0]
                .join("input.svg")
                .to_string_lossy()
                .into_owned(),
            "--export-type=png".into(),
            "--export-area-page".into(),
            format!("--export-width={width}"),
            format!("--export-height={height}"),
            format!(
                "--export-filename={}",
                stage.roots[0].join("output.png").display()
            ),
        ],
        crate::process::timeout(),
        workspace.max_output,
    )
    .map_err(|_| "package preview failed")?;
    if !outcome.success || outcome.timed_out {
        return Err("package preview failed or timed out".into());
    }
    let bytes = stage.read(0, Path::new("output.png"), workspace.max_output)?;
    let reader = png::Decoder::new(std::io::Cursor::new(&bytes))
        .read_info()
        .map_err(|_| "package renderer did not produce PNG")?;
    if reader.info().width > crate::render::cap() || reader.info().height > crate::render::cap() {
        return Err("package preview exceeds dimension cap".into());
    }
    let path = format!(
        ".inkscape-mcp/live/artifacts/live-view-package-{}.png",
        uuid::Uuid::new_v4().simple()
    );
    workspace.ensure_directory(0, Path::new(".inkscape-mcp/live/artifacts"))?;
    workspace.write_new(0, Path::new(&path), &bytes)?;
    Ok(path)
}
pub fn call(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let edits = changes(args)?;
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&edits).map_err(|_| "invalid edits")?)
    );
    let dry = arguments::boolean(args, "dry_run", true)?;
    let nonce = format!("mcp_{}", uuid::Uuid::new_v4().simple());
    let (active, selection, svg) = live
        .session
        .operation(|t| Ok((t.active_document()?, t.selection()?, t.document_svg()?)))
        .map_err(|e| e.public_message().to_string())?;
    let ids = crate::live_models::ids(&selection["object_ids"]);
    let applied =
        package::prepare(&svg, &ids, &edits, &nonce, workspace.max_input).map_err(str::to_owned)?;
    let fp = crate::live_effect::fingerprint(&svg, workspace.max_input)
        .map_err(|e| e.public_message().to_string())?;
    let candidate = if applied.bytes.is_empty() {
        svg.as_str()
    } else {
        std::str::from_utf8(&applied.bytes).map_err(|_| "invalid candidate")?
    };
    if dry {
        let before = preview(&svg, workspace)?;
        let after = if applied.bytes.is_empty() {
            before.clone()
        } else {
            preview(candidate, workspace)?
        };
        return Ok(
            json!({"dry_run":true,"changed":!applied.bytes.is_empty(),"expected_document":crate::live_records::sanitize(&active,workspace),"expected_selection":ids,"expected_fingerprint":fp,"expected_package_digest":digest,"affected_ids":applied.ids,"preview_before":before,"preview_after":after,"preview_artifacts":{"before":workspace.artifact_link(0,Path::new(&before)),"after":workspace.artifact_link(0,Path::new(&after))},"native_undo_verified":false,"notes":["Review these previews, then resubmit the same edits with returned guards, dry_run=false and client-confirmed approval.","Only guarded managed native helpers can apply. Native GUI Undo/Redo acceptance is pending; never retry an uncertain edit before inspection."]}),
        );
    }
    let approval = arguments::string(args, "approval_token")?;
    crate::transaction::policy("high", approval)?;
    if args["expected_package_digest"] != digest {
        return Err("edits changed since review; prepare a new package".into());
    }
    if args["expected_fingerprint"] != fp
        || args["expected_selection"] != selection["object_ids"]
        || args["expected_document"] != crate::live_records::sanitize(&active, workspace)
    {
        return Err(
            "drawing, document or selection changed since review; prepare a new package".into(),
        );
    }
    let available = live
        .session
        .operation(|t| t.package_available())
        .map_err(|e| e.public_message().to_string())?;
    if !available {
        return Err("reviewed packages require the guarded managed native helper".into());
    }
    // A net no-op is validated and guarded but never dispatched/recorded.
    if applied.bytes.is_empty() {
        return Ok(
            json!({"dry_run":false,"changed":false,"affected_ids":applied.ids,"native_undo_verified":false}),
        );
    }
    let params = json!({"operation":null,"edits":edits,"expected_fingerprint":fp,"expected_selection":ids,"expected_document":active});
    let mut result = crate::live_mutation::run(
        live,
        workspace,
        "live_change_package",
        json!({"edits":args["edits"],"expected_fingerprint":fp}),
        crate::live_protocol::Command::ApplySelection,
        approval,
        |t| t.change_package(&params),
    )?;
    result["dry_run"] = json!(false);
    let mut artifacts = serde_json::Map::new();
    for phase in ["before", "after"] {
        if let Some(path) = result[format!("preview_{phase}")].as_str() {
            artifacts.insert(phase.into(), workspace.artifact_link(0, Path::new(path)));
        }
    }
    result["preview_artifacts"] = json!(artifacts);
    result["changed"] = json!(true);
    result["native_undo_verified"] = json!(false);
    result["recovery"] = json!(
        "Inspect the drawing and operation previews; use native Undo after a confirmed change. On uncertainty inspect before any retry."
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_protocol::Command;
    use crate::live_socket::Error;
    use crate::live_transport::{Preference, Probe, Transport};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    const SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80"><rect id="r" width="10" height="10" style="fill:red"/></svg>"#;
    struct Fake {
        svg: Arc<Mutex<String>>,
        dispatch: Arc<AtomicUsize>,
    }
    impl Transport for Fake {
        fn name(&self) -> &str {
            "test"
        }
        fn supports(&self, _: Command) -> bool {
            true
        }
        fn connect(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn disconnect(&mut self) {}
        fn is_connected(&self) -> bool {
            true
        }
        fn active_document(&mut self) -> Result<Value, Error> {
            Ok(json!({"window_id":"w","document_id":"d"}))
        }
        fn selection(&mut self) -> Result<Value, Error> {
            Ok(json!({"object_ids":["r"],"count":1}))
        }
        fn document_svg(&mut self) -> Result<String, Error> {
            Ok(self.svg.lock().unwrap().clone())
        }
        fn package_available(&mut self) -> Result<bool, Error> {
            Ok(true)
        }
        fn change_package(&mut self, params: &Value) -> Result<Value, Error> {
            self.dispatch.fetch_add(1, Ordering::SeqCst);
            let edits: Vec<Change> = serde_json::from_value(params["edits"].clone()).unwrap();
            let mut svg = self.svg.lock().unwrap();
            let applied = package::prepare(
                &svg,
                &["r".into()],
                &edits,
                "mcp_12345678901234567890123456789012",
                65536,
            )
            .unwrap();
            *svg = String::from_utf8(applied.bytes).unwrap();
            Ok(json!({"affected_ids":applied.ids,"count":1,"undo_friendly":false}))
        }
    }
    #[test]
    fn package_api_approval_staleness_failure_noops_and_one_record() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 65536,
            max_output: 65536,
        };
        let svg = Arc::new(Mutex::new(SVG.into()));
        let dispatch = Arc::new(AtomicUsize::new(0));
        let mut live = Live::new();
        live.session.settings.enabled = true;
        live.session
            .connect(
                Preference::NoFreeze,
                &[Probe {
                    name: "test".into(),
                    available: true,
                    rank: 1,
                    commands: vec![Command::ActiveDocument],
                    no_freeze: true,
                    detail: String::new(),
                }],
                |_| {
                    Ok(Box::new(Fake {
                        svg: svg.clone(),
                        dispatch: dispatch.clone(),
                    }))
                },
                || (),
            )
            .unwrap();
        let args = |edits: Value| {
            let changes = changes(&json!({"edits":edits})).unwrap();
            json!({"edits":edits,"dry_run":false,"expected_document":{"window_id":"w","document_id":"d"},"expected_selection":["r"],"expected_fingerprint":crate::live_effect::fingerprint(SVG,65536).unwrap(),"expected_package_digest":format!("{:x}",Sha256::digest(serde_json::to_vec(&changes).unwrap())),"approval_token":"test-approval"})
        };
        let approved = args(json!([{"op":"style","fill":"blue"}]));
        let mut missing = approved.clone();
        missing["approval_token"] = json!("");
        assert!(call(&mut live, &workspace, &missing).is_err());
        for key in [
            "expected_fingerprint",
            "expected_selection",
            "expected_document",
            "expected_package_digest",
        ] {
            let mut stale = approved.clone();
            stale[key] = Value::Null;
            assert!(call(&mut live, &workspace, &stale).is_err());
        }
        let invalid =
            args(json!([{"op":"style","fill":"blue"},{"op":"text","text":"refuse rectangle"}]));
        assert!(call(&mut live, &workspace, &invalid).is_err());
        let noop = args(json!([{"op":"style","fill":"red"}]));
        assert_eq!(
            call(&mut live, &workspace, &noop).unwrap()["changed"],
            false
        );
        assert_eq!(dispatch.load(Ordering::SeqCst), 0);
        assert_eq!(*svg.lock().unwrap(), SVG);
        assert!(!root.path().join(".inkscape-mcp/live/operations").exists());
        let applied = call(&mut live, &workspace, &approved).unwrap();
        assert_eq!(applied["changed"], true);
        assert_eq!(dispatch.load(Ordering::SeqCst), 1);
        let record =
            crate::live_records::get(&workspace, applied["operation_id"].as_str().unwrap())
                .unwrap();
        assert_eq!(record["status"], "applied");
        assert_eq!(
            std::fs::read_dir(root.path().join(".inkscape-mcp/live/operations"))
                .unwrap()
                .count(),
            1
        );
    }
}
