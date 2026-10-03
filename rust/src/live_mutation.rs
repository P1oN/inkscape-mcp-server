//! Fixed live mutations share approval, scoped context, proposed/applied/discarded records and feedback.
use crate::{
    arguments, live::Live, live_protocol::Command, live_records, live_render, live_socket::Error,
    live_view::Frame, style, workspace::Workspace,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub struct Style {
    pub properties: BTreeMap<String, String>,
    pub transform: Option<String>,
}

pub fn style_plan(args: &Value) -> Result<Style, String> {
    let mut properties = BTreeMap::new();
    for key in ["fill", "stroke"] {
        if let Some(raw) = arguments::string(args, key)? {
            properties.insert(key.into(), style::color(raw)?);
        }
    }
    if let Some(raw) = arguments::string(args, "stroke_width")? {
        properties.insert("stroke-width".into(), style::length(raw)?);
    }
    if let Some(value) = arguments::optional_number(args, "opacity")? {
        if !value.is_finite() || !(0. ..=1.).contains(&value) {
            return Err("opacity must be between 0 and 1".into());
        }
        let raw = format!("{value:.6}");
        let opacity = raw.trim_end_matches('0').trim_end_matches('.');
        properties.insert(
            "opacity".into(),
            if opacity.is_empty() {
                "0".into()
            } else {
                opacity.into()
            },
        );
    }
    let dx = arguments::optional_number(args, "dx")?;
    let dy = arguments::optional_number(args, "dy")?;
    if dx.is_some() != dy.is_some() {
        return Err("translate requires both dx and dy".into());
    }
    let mut parts = Vec::new();
    if let (Some(x), Some(y)) = (dx, dy) {
        parts.push(format!(
            "translate({},{})",
            style::format_num(x)?,
            style::format_num(y)?
        ));
    }
    if let Some(rotate) = arguments::optional_number(args, "rotate")? {
        parts.push(format!("rotate({})", style::format_num(rotate)?));
    }
    if let Some(scale) = arguments::optional_number(args, "scale")? {
        if !scale.is_finite() || scale <= 0. {
            return Err("scale factor must be positive".into());
        }
        parts.push(format!("scale({})", style::format_num(scale)?));
    }
    let transform = if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    };
    if properties.is_empty() && transform.is_none() {
        return Err("supply at least one style or transform parameter".into());
    }
    Ok(Style {
        properties,
        transform,
    })
}
pub fn apply_style(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let plan = style_plan(args)?;
    let approval = arguments::string(args, "approval_token")?;
    let params = json!({"style":plan.properties,"transform":plan.transform});
    run(
        live,
        workspace,
        "live_apply_to_selection",
        params,
        Command::ApplySelection,
        approval,
        |transport| transport.apply_selection(&plan.properties, plan.transform.as_deref()),
    )
}
pub fn validate_text(text: &str) -> Result<usize, String> {
    let length = text.chars().count();
    if length > 100000 {
        return Err(format!("text too long: {length} > 100000 characters"));
    }
    if text.chars().any(
        |c| matches!(c,'\u{0}'..='\u{8}'|'\u{b}'..='\u{c}'|'\u{e}'..='\u{1f}'|'\u{7f}'..='\u{9f}'),
    ) {
        return Err("text contains forbidden control characters".into());
    }
    Ok(length)
}
pub fn apply_text(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let text = arguments::string(args, "text")?.ok_or("text is required")?;
    let length = validate_text(text)?;
    let approval = arguments::string(args, "approval_token")?;
    run(
        live,
        workspace,
        "live_set_selected_text",
        json!({"text_len":length}),
        Command::SetText,
        approval,
        |transport| transport.set_text(text),
    )
}
pub fn validate_fragment(fragment: &str) -> Result<(), String> {
    if fragment
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
        .is_empty()
    {
        return Err("svg fragment is empty".into());
    }
    if fragment.len() > 1024 * 1024 {
        return Err("svg fragment is too large".into());
    }
    let wrapped = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\">{fragment}</svg>"
    );
    crate::xml::parse(wrapped.as_bytes(), 1024 * 1024 + 128)
        .map_err(|_| "svg fragment could not be parsed safely")?;
    Ok(())
}
pub fn apply_insert(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let fragment = arguments::string(args, "svg_fragment")?.ok_or("svg_fragment is required")?;
    validate_fragment(fragment)?;
    let approval = arguments::string(args, "approval_token")?;
    run(
        live,
        workspace,
        "live_insert_svg",
        json!({"fragment_bytes":fragment.len()}),
        Command::InsertSvg,
        approval,
        |transport| transport.insert_svg(fragment),
    )
}
pub fn apply_structural(
    live: &mut Live,
    workspace: &Workspace,
    args: &Value,
) -> Result<Value, String> {
    let operation = arguments::string(args, "operation")?.ok_or("operation is required")?;
    let edit = crate::live_managed::SelectionEdit::parse(operation)
        .ok_or("invalid selection edit operation")?;
    let approval = arguments::string(args, "approval_token")?;
    run(
        live,
        workspace,
        "live_edit_selection",
        json!({"operation":operation}),
        Command::ApplySelection,
        approval,
        |transport| transport.order_selection(edit),
    )
}
pub fn run(
    live: &mut Live,
    workspace: &Workspace,
    tool: &str,
    params: Value,
    command: Command,
    approval: Option<&str>,
    op: impl FnOnce(&mut dyn crate::live_transport::Transport) -> Result<Value, Error>,
) -> Result<Value, String> {
    let supported = live
        .session
        .require_transport()
        .map_err(|e| e.public_message().to_string())?
        .supports(command);
    if !supported {
        return Err(Error::Unsupported("edit unavailable")
            .public_message()
            .into());
    }
    let document = live.session.active_document.clone();
    let mut cache = live.session.cache.take();
    let result=live.session.operation(|transport|{
        let document=if transport.name()=="managed-dbus"{transport.active_document().unwrap_or(Value::Null)}else{document};
        let selection=match transport.selection(){
            Ok(selection)=>crate::live_models::ids(&selection["object_ids"]),
            Err(error @ (Error::Protocol(_)|Error::Rejected)) if transport.name()=="extension-socket"=>return Err(error),
            Err(_)=>Vec::new(),
        };
        let mut record=live_records::new(workspace,tool,params,transport.name(),&document,selection,approval).map_err(Error::EditRefused)?;
        let frame=Frame{region:None,scale:None};
        let before=live_render::render(transport,cache.as_mut(),workspace,frame).ok().and_then(|r|r["artifact_path"].as_str().map(str::to_owned));
        let mutation=match op(transport){
            Ok(result)=>result,
            Err(error)=>{
                if !(transport.name()=="extension-socket"&&matches!(error,Error::Protocol(_)|Error::Rejected)) {
                    // Audit persistence must never hide the transport outcome after dispatch.
                    let _ = live_records::update(workspace,&mut record,json!({"status":"discarded","completion_uncertain":matches!(error,Error::Uncertain)}));
                }
                return Err(error);
            }
        };
        let after=live_render::render(transport,cache.as_mut(),workspace,frame).ok().and_then(|r|r["artifact_path"].as_str().map(str::to_owned));
        let mut previews=serde_json::Map::new();
        if let Some(before)=before.as_ref(){previews.insert("before".into(),json!(before));}
        if let Some(after)=after.as_ref(){previews.insert("after".into(),json!(after));}
        // The GUI effect already completed; a lost audit is not a clean refusal.
        live_records::update(workspace,&mut record,json!({"status":"applied","previews":previews,"affected_ids":mutation["affected_ids"],"undo_friendly":mutation["undo_friendly"]})).map_err(|_| Error::Uncertain)?;
        let detail=mutation["detail"].as_str().filter(|s|!s.is_empty()).unwrap_or(tool);
        Ok(json!({"operation_id":record["operation_id"],"transport":transport.name(),"summary":detail,"affected_ids":mutation["affected_ids"],"count":mutation["count"],"undo_friendly":mutation["undo_friendly"],"preview_before":before,"preview_after":after}))
    });
    live.session.cache = cache;
    result.map_err(|error| {
        crate::live_events::protocol_message(&error)
            .map(|message| format!("Error calling tool '{tool}': {message}"))
            .unwrap_or_else(|| error.public_message().into())
    })
}

#[cfg(test)]
mod audit_failure_tests {
    use super::*;
    use crate::live_transport::{Preference, Probe, Transport};
    struct Fake;
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
    }
    #[test]
    fn audit_failure_after_dispatch_preserves_uncertainty_and_never_retries() {
        for succeeded in [true, false] {
            let root = tempfile::tempdir().unwrap();
            let external = tempfile::tempdir().unwrap();
            std::fs::write(external.path().join("original"), b"preserve me").unwrap();
            let workspace = Workspace {
                roots: vec![root.path().canonicalize().unwrap()],
                max_input: 4096,
                max_output: 4096,
            };
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
                    |_| Ok(Box::new(Fake)),
                    || (),
                )
                .unwrap();
            let dispatched = std::cell::Cell::new(0);
            let operations = workspace.roots[0].join(".inkscape-mcp/live/operations");
            let retained = workspace.roots[0].join("retained-operations");
            let result = run(
                &mut live,
                &workspace,
                "live_apply_to_selection",
                json!({}),
                Command::ApplySelection,
                Some("approved"),
                |_| {
                    dispatched.set(dispatched.get() + 1);
                    std::fs::rename(&operations, &retained).unwrap();
                    std::os::unix::fs::symlink(external.path(), &operations).unwrap();
                    if succeeded {
                        Ok(json!({"affected_ids":["r"],"count":1,"undo_friendly":true}))
                    } else {
                        Err(Error::Uncertain)
                    }
                },
            );
            assert_eq!(result, Err(Error::Uncertain.public_message().into()));
            assert_eq!(dispatched.get(), 1);
            assert_eq!(std::fs::read_dir(&retained).unwrap().count(), 1);
            assert_eq!(std::fs::read_dir(external.path()).unwrap().count(), 1);
            assert_eq!(
                std::fs::read(external.path().join("original")).unwrap(),
                b"preserve me"
            );
        }
    }
}
