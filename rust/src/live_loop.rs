//! One fixed perceive/act/observe iteration, composing existing governed live kernels.
use crate::{
    arguments, live::Live, live_mutation, live_protocol::Command, live_render, live_socket::Error,
    live_view::Frame, workspace::Workspace,
};
use serde_json::{Value, json};
fn perceive(live: &mut Live, workspace: &Workspace) -> Result<(Value, Value), String> {
    let mut cache = live.session.cache.take();
    let result = (|| {
        let transport = live.session.require_transport()?;
        let frame = live_render::render(
            transport,
            cache.as_mut(),
            workspace,
            Frame {
                region: None,
                scale: None,
            },
        )?;
        let scene = transport.scene()?;
        Ok((scene, frame))
    })();
    live.session.cache = cache;
    result.map_err(|e: Error| e.public_message().to_string())
}
fn act_error(message: String) -> String {
    if message.starts_with("Error calling tool '") {
        "live operation failed".into()
    } else {
        message
    }
}
pub fn step(live: &mut Live, workspace: &Workspace, args: &Value) -> Result<Value, String> {
    let action = arguments::string(args, "action")?;
    if let Some(action) = action
        && !matches!(action, "apply" | "insert_svg" | "set_text")
    {
        return Err(format!(
            "1 validation error for call[live_session_step]\naction\n  Input should be 'apply', 'insert_svg' or 'set_text' [type=enum, input_value={}, input_type=str]\n    For further information visit https://errors.pydantic.dev/2.13/v/enum",
            crate::style::python_repr(action)
        ));
    }
    let (before_scene, before_frame) = perceive(live, workspace)?;
    let Some(action) = action else {
        return Ok(
            json!({"acted":false,"action":null,"before_scene":before_scene,"before_frame":before_frame,"operation_id":null,"edit":null,"diff":null,"after_scene":null,"after_frame":null}),
        );
    };
    let approval = arguments::string(args, "approval_token")?;
    let edit = match action {
        "apply" => {
            let plan = live_mutation::style_plan(args).map_err(|e| {
                if e == "supply at least one style or transform parameter" {
                    "apply requires at least one style or transform parameter".into()
                } else {
                    e
                }
            })?;
            live_mutation::run(
                live,
                workspace,
                "live_session_step:apply",
                json!({"style":plan.properties,"transform":plan.transform}),
                Command::ApplySelection,
                approval,
                |t| t.apply_selection(&plan.properties, plan.transform.as_deref()),
            )
        }
        "insert_svg" => {
            let fragment = arguments::string(args, "svg_fragment")?
                .ok_or("insert_svg requires an svg_fragment")?;
            live_mutation::validate_fragment(fragment)?;
            live_mutation::run(
                live,
                workspace,
                "live_session_step:insert_svg",
                json!({"fragment_bytes":fragment.len()}),
                Command::InsertSvg,
                approval,
                |t| t.insert_svg(fragment),
            )
        }
        "set_text" => {
            let text = arguments::string(args, "text")?.ok_or("set_text requires a text value")?;
            let length = live_mutation::validate_text(text)?;
            live_mutation::run(
                live,
                workspace,
                "live_session_step:set_text",
                json!({"text_len":length}),
                Command::SetText,
                approval,
                |t| t.set_text(text),
            )
        }
        _ => unreachable!(),
    }
    .map_err(act_error)?;
    let after = perceive(live, workspace).ok();
    let scene = after
        .as_ref()
        .map(|(scene, _)| scene)
        .unwrap_or(&before_scene);
    let diff =
        crate::live_diff::diff_scene(workspace, edit["operation_id"].as_str().unwrap(), scene).ok();
    let (after_scene, after_frame) = after.unwrap_or((Value::Null, Value::Null));
    Ok(
        json!({"acted":true,"action":action,"before_scene":before_scene,"before_frame":before_frame,"operation_id":edit["operation_id"],"edit":edit,"diff":diff,"after_scene":after_scene,"after_frame":after_frame}),
    )
}
