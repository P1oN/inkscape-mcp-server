mod actions;
mod adopt;
mod argument_normalization;
mod argument_numbers;
mod argument_validation;
mod arguments;
mod artifact_stat;
mod batch;
mod canvas;
mod collection;
mod contract;
mod create;
mod css;
mod css_assets;
mod delete;
mod doctor;
mod document;
mod duplicate;
mod editability;
mod engine;
mod engine_input;
mod export_batch;
mod find;
mod fit;
mod fonts;
mod fragment;
mod frames;
mod geometry;
mod gradient;
mod grid;
mod grid_plan;
mod group;
mod identity;
mod inspect;
mod intents;
// Native public live integration and private fixed IPC kernels.
mod decimal;
mod live;
mod live_arm;
mod live_attach;
#[allow(dead_code)]
mod live_bus;
#[allow(dead_code)]
mod live_cache;
#[allow(dead_code)]
mod live_context;
#[allow(dead_code)]
mod live_dbus;
mod live_diff;
mod live_discovery;
#[allow(dead_code)]
mod live_effect;
mod live_events;
#[allow(dead_code)]
mod live_insert;
mod live_install;
mod live_launch;
mod live_loop;
#[allow(dead_code)]
mod live_managed;
#[allow(dead_code)]
mod live_models;
mod live_mutation;
#[allow(dead_code)]
mod live_probe;
#[allow(dead_code)]
mod live_protocol;
mod live_records;
mod live_render;
#[allow(dead_code)]
mod live_scene;
#[allow(dead_code)]
mod live_selection;
#[allow(dead_code)]
mod live_session;
#[allow(dead_code)]
mod live_socket;
mod live_sync;
#[allow(dead_code)]
mod live_transport;
mod live_view;
mod optimization_analysis;
mod optimize;
mod paths;
mod placement;
mod preview;
mod process;
mod profiles;
mod prompts;
mod quality;
mod recolor;
mod recovery;
mod render;
mod reparent;
mod repeat;
mod repeat_plan;
mod retention;
mod runtime;
mod save;
mod stdio_limit;
mod structure;
mod style;
mod telemetry;
mod text;
mod tile;
mod transaction;
mod transform;
mod transform_objects;
mod use_object;
mod validate;
mod workspace;
use inkscape_mcp_rust::xml;

use base64::{Engine, engine::general_purpose::STANDARD};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt, model::*, service::RequestContext};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// Preserve the frozen wire shape without cloning a potentially large JSON tree
/// through an intermediate Value and rmcp's compatibility deserializer.
fn tool_result(native: Result<Value, String>) -> CallToolResult {
    match native {
        Ok(value) => {
            let text = value.to_string();
            let wrapped = value.is_boolean();
            let structured_content = if wrapped {
                Some(json!({"result": value}))
            } else {
                Some(value)
            };
            let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
            result.result_type = None;
            result.structured_content = structured_content;
            result.meta = wrapped.then(|| {
                serde_json::from_value(json!({"fastmcp": {"wrap_result": true}}))
                    .expect("fixed boolean metadata")
            });
            result
        }
        Err(message) => {
            let mut result = CallToolResult::error(vec![ContentBlock::text(message)]);
            result.result_type = None;
            result
        }
    }
}

#[derive(Clone)]
struct Server {
    contract: Arc<Value>,
    operations: Arc<tokio::sync::Mutex<()>>,
    workers: Arc<tokio::sync::Semaphore>,
    registry: Arc<Mutex<document::Registry>>,
    capabilities: Arc<Mutex<Option<Value>>>,
    runtime_bus: Arc<Mutex<Option<live_probe::Inputs>>>,
    live: Arc<Mutex<live::Live>>,
}

impl Server {
    fn call_tool_blocking(
        &self,
        name: &str,
        arguments: &Value,
        known: bool,
    ) -> Result<CallToolResponse, ErrorData> {
        if process::cancelled() {
            return Ok(tool_result(Err("operation cancelled".into())).into());
        }
        if known
            && matches!(
                name,
                "render_preview"
                    | "export_document"
                    | "export_object"
                    | "capture_frame"
                    | "compare_region"
            )
        {
            let registry = self
                .registry
                .lock()
                .map_err(|_| telemetry::internal_error("registry lock failed", None))?
                .clone();
            let value = match render::call(&registry, name, arguments) {
                Ok(value) => value,
                Err(message) => json!({"content":[{"type":"text","text":message}],"isError":true}),
            };
            let result: CallToolResult = serde_json::from_value(value)
                .map_err(|_| telemetry::internal_error("render serialization failed", None))?;
            return Ok(result.into());
        }
        let native = if known {
            let mut guard = self
                .registry
                .lock()
                .map_err(|_| telemetry::internal_error("registry lock failed", None))?;
            let mut snapshot;
            let registry = if matches!(
                name,
                "open_document"
                    | "create_document"
                    | "reload_document"
                    | "compose_grid"
                    | "live_sync_to_workspace"
            ) {
                &mut *guard
            } else {
                snapshot = guard.clone();
                drop(guard);
                &mut snapshot
            };
            match name {
                "live_launch" => Some((|| {
                    let enabled = self
                        .live
                        .lock()
                        .map_err(|_| {
                            telemetry::failure(telemetry::Failure::Internal);
                            "live session lock failed"
                        })?
                        .session
                        .settings
                        .enabled;
                    live_install::gate(enabled)?;
                    let attached = live_launch::launch(registry.workspace.max_output)?;
                    self.live
                        .lock()
                        .map_err(|_| {
                            telemetry::failure(telemetry::Failure::Internal);
                            "live session lock failed"
                        })?
                        .adopt_launch(attached);
                    Ok(json!(true))
                })()),
                "live_install_helper" | "live_arm_socket" => Some((|| {
                    let enabled = self
                        .live
                        .lock()
                        .map_err(|_| {
                            telemetry::failure(telemetry::Failure::Internal);
                            "live session lock failed"
                        })?
                        .session
                        .settings
                        .enabled;
                    live_install::gate(enabled)?;
                    let capabilities = self.capabilities(registry, false)?;
                    if name == "live_install_helper" {
                        live_install::install(&capabilities)
                    } else {
                        live_arm::arm(&capabilities, registry.workspace.max_input)
                    }
                })()),
                "check_live_support" | "live_status" | "live_disconnect" => Some((|| {
                    let capabilities = self.capabilities(registry, false)?;
                    self.live
                        .lock()
                        .map_err(|_| {
                            telemetry::failure(telemetry::Failure::Internal);
                            "live session lock failed"
                        })?
                        .call(&registry.workspace, &capabilities, name, arguments)
                })(
                )),
                "live_get_active_document"
                | "live_get_selection"
                | "live_inspect_selection"
                | "live_list_documents"
                | "live_set_viewport"
                | "live_render_view"
                | "live_get_scene"
                | "live_export_selection"
                | "live_find_objects"
                | "live_preview_object"
                | "live_diff_view"
                | "live_session_step"
                | "live_apply_to_selection"
                | "live_set_selected_text"
                | "live_insert_svg"
                | "live_edit_selection" => Some((|| {
                    self.live
                        .lock()
                        .map_err(|_| {
                            telemetry::failure(telemetry::Failure::Internal);
                            "live session lock failed"
                        })?
                        .call(&registry.workspace, &Value::Null, name, arguments)
                })()),
                "live_select_document" => Some((|| {
                    let mut live = self.live.lock().map_err(|_| {
                        telemetry::failure(telemetry::Failure::Internal);
                        "live session lock failed"
                    })?;
                    live.session
                        .require_transport()
                        .map_err(|e| e.public_message().to_string())?;
                    let capabilities = self.capabilities(registry, false)?;
                    live.call(
                        &registry.workspace,
                        &capabilities,
                        "live_select_document",
                        arguments,
                    )
                })()),
                "live_connect" => Some((|| {
                    let mut live = self.live.lock().map_err(|_| {
                        telemetry::failure(telemetry::Failure::Internal);
                        "live session lock failed"
                    })?;
                    live.prepare_connect(&registry.workspace, arguments)?;
                    *self
                        .runtime_bus
                        .lock()
                        .map_err(|_| "runtime bus lock failed")? = live.runtime_inputs();
                    let capabilities = self.capabilities(registry, false)?;
                    live.call(
                        &registry.workspace,
                        &capabilities,
                        "live_connect",
                        arguments,
                    )
                })()),
                "get_workspace_info" => Some(Ok(registry.workspace.info())),
                "list_capabilities" | "diagnose_runtime" => {
                    Some(self.capabilities(registry, name == "diagnose_runtime"))
                }
                "svg_web_optimize" | "optimize_set" => {
                    Some(optimize::apply(registry, name, arguments))
                }
                "set_document_svg" | "insert_svg_fragment" => {
                    Some(adopt::apply(registry, name, arguments))
                }
                "replace_svg_fragment" => Some(fragment::apply(registry, arguments)),
                "place_document" => Some(placement::apply(registry, arguments)),
                "compose_grid" => Some(grid::apply(registry, arguments)),
                "how_do_i" => Some(intents::apply(registry, arguments)),
                "quality_report" | "quality_report_set" => {
                    Some(quality::apply(registry, name, arguments))
                }
                "stat_artifact" | "stat_artifacts" => {
                    Some(artifact_stat::apply(registry, name, arguments))
                }
                "live_sync_to_workspace" => Some((|| {
                    let mut live = self.live.lock().map_err(|_| {
                        telemetry::failure(telemetry::Failure::Internal);
                        "live session lock failed"
                    })?;
                    live_sync::sync(&mut live, registry, arguments)
                })()),
                "open_document" => Some(registry.open(arguments)),
                "create_document" => Some(registry.create(arguments)),
                "reload_document" => Some(registry.reload(arguments)),
                "prune_snapshots" => Some(retention::apply(registry, arguments)),
                "save_document_as" => Some(save::document(registry, arguments)),
                "list_frames" => Some(frames::list(registry, arguments)),
                "export_batch" => Some(export_batch::call(registry, arguments)),
                "export_set" => Some(collection::export_set(registry, arguments)),
                "export_web_profile" | "create_icon_set" | "export_print_profile" => {
                    Some(profiles::call(registry, name, arguments))
                }
                "validate_document" => Some((|| {
                    validate::document(
                        registry,
                        arguments["doc_id"]
                            .as_str()
                            .ok_or("doc_id must be a string")?,
                    )
                })()),
                "inspect_document" => Some((|| {
                    inspect::document(
                        registry,
                        arguments["doc_id"]
                            .as_str()
                            .ok_or("doc_id must be a string")?,
                    )
                })()),
                "create_rect" | "create_circle" | "create_ellipse" | "create_line"
                | "create_polygon" | "create_polyline" | "create_path" | "create_text" => {
                    Some(create::apply(registry, name, arguments))
                }
                "add_linear_gradient" | "add_radial_gradient" => {
                    Some(gradient::apply(registry, name, arguments))
                }
                "create_group" | "set_group_mode" => Some(group::apply(registry, name, arguments)),
                "create_use" => Some(use_object::apply(registry, arguments)),
                "group_objects" => Some(structure::apply(registry, arguments)),
                "reparent_object" => Some(reparent::apply(registry, arguments)),
                "duplicate_object" => Some(duplicate::apply(registry, arguments)),
                "tile" => Some(tile::apply(registry, arguments)),
                "repeat_objects" => Some(repeat::apply(registry, arguments)),
                "find_objects" => Some(find::apply(registry, arguments)),
                "delete_object" => Some(delete::apply(registry, arguments)),
                "rename_object" => Some(identity::apply(registry, arguments)),
                "fit_to_content" => Some(fit::apply(registry, arguments)),
                "resize_canvas" | "normalize_viewbox" => {
                    Some(canvas::apply(registry, name, arguments))
                }
                "move_object" | "scale_object" | "rotate_object" => {
                    Some(transform::apply(registry, name, arguments))
                }
                "replace_text" | "set_font" => Some(text::apply(registry, name, arguments)),
                "list_actions" | "discover_extensions" => {
                    Some(runtime::discovery(registry, name, arguments))
                }
                "validate_action_chain" | "run_action_chain" | "run_raw_action" => {
                    Some(actions::apply(registry, name, arguments))
                }
                "simplify_path" | "boolean_union" | "boolean_difference" | "combine_paths"
                | "break_apart" | "stroke_to_path" | "cleanup_paths" => {
                    Some(paths::apply(registry, name, arguments))
                }
                "transform_objects" => Some(transform_objects::apply(registry, arguments)),
                "apply_edits" => Some(batch::apply(registry, arguments)),
                "replace_color" | "apply_palette" => {
                    Some(recolor::apply(registry, name, arguments))
                }
                "set_fill" | "set_stroke" | "set_opacity" => {
                    Some(style::apply(registry, name, arguments))
                }
                "list_snapshots" => Some((|| {
                    let id = arguments["doc_id"]
                        .as_str()
                        .ok_or("doc_id must be a string")?;
                    let entry = registry.entries.get(id).ok_or("document id not found")?;
                    Ok(
                        json!({"doc_id":id,"snapshots":transaction::list_snapshots(registry,entry)?}),
                    )
                })()),
                "create_snapshot" => Some((|| {
                    let id = arguments["doc_id"]
                        .as_str()
                        .ok_or("doc_id must be a string")?;
                    let entry = registry.entries.get(id).ok_or("document id not found")?;
                    transaction::snapshot(
                        registry,
                        entry,
                        arguments::string(arguments, "label")?,
                        None,
                    )
                })()),
                "restore_snapshot" => Some((|| {
                    transaction::restore(
                        registry,
                        arguments["doc_id"]
                            .as_str()
                            .ok_or("doc_id must be a string")?,
                        arguments["snapshot_id"]
                            .as_str()
                            .ok_or("snapshot_id must be a string")?,
                    )
                })()),
                _ => None,
            }
        } else {
            None
        };
        if let Some(native) = native {
            let result = tool_result(native);
            return Ok(result.into());
        }
        let message = if known {
            format!("Rust migration pending: {} has not been ported", name)
        } else {
            format!("Unknown tool: '{}'", name)
        };
        let result: CallToolResult = serde_json::from_value(json!({
            "content": [{"type": "text", "text": message}], "isError": true
        }))
        .expect("tool error wire shape");
        Ok(result.into())
    }
    fn read_resource_blocking(
        &self,
        request: ReadResourceRequestParams,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let registry = self
            .registry
            .lock()
            .map_err(|_| telemetry::internal_error("registry lock failed", None))?
            .clone();
        if let Some((root_key, token)) = request
            .uri
            .strip_prefix("inkscape://artifact/")
            .and_then(|tail| tail.split_once('/'))
        {
            let bytes = registry
                .workspace
                .read_artifact(root_key, token)
                .map_err(|message| ErrorData::new(ErrorCode(0), message, None))?;
            let result: ReadResourceResult = serde_json::from_value(json!({"contents":[{
                "uri":request.uri,"mimeType":"application/octet-stream","blob":STANDARD.encode(bytes)
            }]})).map_err(|_| telemetry::internal_error("resource serialization failed", None))?;
            return Ok(result.into());
        }
        if request.uri == "inkscape://runtime/intents" {
            let result: ReadResourceResult = serde_json::from_value(json!({"contents":[{
                "uri":request.uri,"mimeType":"application/json","text":intents::resource_text()
            }]}))
            .map_err(|_| telemetry::internal_error("resource serialization failed", None))?;
            return Ok(result.into());
        }
        let value = if request.uri == "inkscape://live/operations" {
            live_records::list(&registry.workspace)
        } else if request.uri == "inkscape://live/events" {
            let mut live = self
                .live
                .lock()
                .map_err(|_| telemetry::internal_error("live session lock failed", None))?;
            match live_events::detect(&mut live.session) {
                Ok(value) => value,
                Err(error) => {
                    if let Some(message) = live_events::protocol_message(&error) {
                        return Err(ErrorData::new(
                            ErrorCode(0),
                            format!("Error reading resource 'inkscape://live/events': {message}"),
                            None,
                        ));
                    }
                    live_events::empty()
                }
            }
        } else if matches!(
            request.uri.as_str(),
            "inkscape://live/session" | "inkscape://live/selection" | "inkscape://live/view"
        ) {
            let capabilities = if request.uri == "inkscape://live/session" {
                self.capabilities(&registry, false)
                    .map_err(|message| ErrorData::new(ErrorCode(0), message, None))?
            } else {
                Value::Null
            };
            self.live
                .lock()
                .map_err(|_| telemetry::internal_error("live session lock failed", None))?
                .resource(
                    &registry.workspace,
                    &capabilities,
                    request.uri.rsplit('/').next().unwrap(),
                )
        } else if request.uri == "inkscape://runtime/capabilities" {
            self.capabilities(&registry, false)
                .map_err(|message| ErrorData::new(ErrorCode(0), message, None))?
        } else if request.uri == "inkscape://workspace" {
            registry.workspace.info()
        } else if request.uri == "inkscape://prompts" {
            prompts::index(&self.contract)
        } else if request.uri == "inkscape://documents" {
            json!({"documents":registry.entries.keys().map(|id| {
                let resources:serde_json::Map<String,Value>=["summary","tree","layers","objects","styles","fonts","assets"].into_iter().map(|leaf| (leaf.to_owned(),json!(format!("inkscape://document/{id}/{leaf}")))).collect();
                json!({"doc_id":id,"resources":resources})
            }).collect::<Vec<_>>()})
        } else if let Some((id, leaf)) = request
            .uri
            .strip_prefix("inkscape://document/")
            .and_then(|tail| tail.split_once('/'))
        {
            if !matches!(
                leaf,
                "summary" | "tree" | "layers" | "objects" | "styles" | "fonts" | "assets"
            ) {
                return Err(Self::unknown_resource(&request.uri));
            }
            (if leaf == "summary" {
                registry.summary(id)
            } else {
                inspect::resource(&registry, id, leaf)
            })
            .map_err(|message| ErrorData::new(ErrorCode(0), message, None))?
        } else {
            let known = self.contract["resources/list"]["resources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["uri"] == request.uri);
            return Err(if known {
                ErrorData::new(
                    ErrorCode(0),
                    "Rust migration pending: resource not ported",
                    None,
                )
            } else {
                Self::unknown_resource(&request.uri)
            });
        };
        let mime = if matches!(
            request.uri.as_str(),
            "inkscape://workspace"
                | "inkscape://documents"
                | "inkscape://prompts"
                | "inkscape://runtime/capabilities"
                | "inkscape://live/session"
                | "inkscape://live/selection"
                | "inkscape://live/view"
                | "inkscape://live/events"
                | "inkscape://live/operations"
        ) {
            "application/json"
        } else {
            "text/plain"
        };
        let result: ReadResourceResult =
            serde_json::from_value(json!({"contents":[{"uri":request.uri,"mimeType":mime,
                                                 "text":value.to_string()}]}))
            .map_err(|_| telemetry::internal_error("resource serialization failed", None))?;
        Ok(result.into())
    }
    fn capabilities(&self, registry: &document::Registry, refresh: bool) -> Result<Value, String> {
        let host = self
            .runtime_bus
            .lock()
            .map_err(|_| "runtime bus lock failed")?
            .clone();
        let mut cached = self
            .capabilities
            .lock()
            .map_err(|_| "capability cache lock failed")?;
        if refresh || cached.is_none() {
            let detected = runtime::detect(registry, host.as_ref());
            if process::cancelled() {
                return Err("operation cancelled".into());
            }
            *cached = Some(detected);
        }
        Ok(runtime::overlay(cached.as_ref().unwrap(), &self.contract))
    }
    fn unknown_resource(uri: &str) -> ErrorData {
        ErrorData::resource_not_found(
            format!(
                "Resource not found: Unknown resource: {}",
                style::python_repr(uri)
            ),
            None,
        )
    }
    fn decode<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, ErrorData> {
        serde_json::from_value(self.contract[key].clone())
            .map_err(|_| telemetry::internal_error("invalid frozen contract", None))
    }
}

impl ServerHandler for Server {
    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        prompts::render(&self.contract, request)
    }
    fn get_info(&self) -> ServerConfig {
        self.decode("initialize").expect("reference initialization")
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        self.decode("tools/list")
    }

    async fn list_resources(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        self.decode("resources/list")
    }

    async fn list_resource_templates(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        self.decode("resources/templates/list")
    }

    async fn list_prompts(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        self.decode("prompts/list")
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let tool = self.contract["tools/list"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"].as_str() == Some(request.name.as_ref()));
        let known = tool.is_some();
        let _timing = telemetry::tool_timing(if known {
            request.name.as_ref()
        } else {
            "unknown_tool"
        });
        let arguments = request
            .arguments
            .map(Value::Object)
            .unwrap_or_else(|| json!({}));
        if let Some(tool) = tool
            && let Err(message) = argument_validation::validate(tool, &arguments)
        {
            return Ok(tool_result(Err(message)).into());
        }
        let arguments = tool
            .map(|tool| argument_normalization::normalize(tool, &arguments))
            .unwrap_or(arguments);
        if known && request.name.as_ref() == "live_wait_for_change" {
            let value = match live_events::wait(&self.live, &self.workers, &arguments, &context)
                .await
            {
                Ok(value) => {
                    json!({"content":[{"type":"text","text":serde_json::to_string(&value).unwrap()}],"structuredContent":value,"isError":false})
                }
                Err(message) => json!({"content":[{"type":"text","text":message}],"isError":true}),
            };
            let result: CallToolResult = serde_json::from_value(value)
                .map_err(|_| telemetry::internal_error("live wait serialization failed", None))?;
            return Ok(result.into());
        }
        let operation = if (!request.name.starts_with("live_")
            && request.name != "get_workspace_info")
            || request.name == "live_sync_to_workspace"
        {
            Some(tokio::select! {
                _ = context.ct.cancelled() => return Ok(tool_result(Err("operation cancelled".into())).into()),
                guard = self.operations.clone().lock_owned() => guard,
            })
        } else {
            None
        };
        let permit = tokio::select! {
            _ = context.ct.cancelled() => return Ok(tool_result(Err("operation cancelled".into())).into()),
            permit = self.workers.clone().acquire_owned() => permit.map_err(|_| telemetry::internal_error("worker queue closed", None))?,
        };
        let server = self.clone();
        let dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let _cancel_on_drop = process::CancelOnDrop(dropped.clone());
        let ct = context.ct.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let _operation = operation;
            process::with_cancellation(
                move || ct.is_cancelled() || dropped.load(std::sync::atomic::Ordering::Acquire),
                || server.call_tool_blocking(request.name.as_ref(), &arguments, known),
            )
        })
        .await
        .map_err(|_| telemetry::internal_error("tool worker failed", None))?
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let operation = if matches!(
            request.uri.as_str(),
            "inkscape://workspace"
                | "inkscape://documents"
                | "inkscape://prompts"
                | "inkscape://runtime/intents"
        ) {
            None
        } else {
            Some(tokio::select! {
                _ = context.ct.cancelled() => return Err(ErrorData::internal_error("resource read cancelled", None)),
                guard = self.operations.clone().lock_owned() => guard,
            })
        };
        let permit = tokio::select! {
            _ = context.ct.cancelled() => return Err(ErrorData::internal_error("resource read cancelled", None)),
            permit = self.workers.clone().acquire_owned() => permit.map_err(|_| telemetry::internal_error("worker queue closed", None))?,
        };
        let server = self.clone();
        let dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let _cancel_on_drop = process::CancelOnDrop(dropped.clone());
        let ct = context.ct.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let _operation = operation;
            process::with_cancellation(
                move || ct.is_cancelled() || dropped.load(std::sync::atomic::Ordering::Acquire),
                || server.read_resource_blocking(request),
            )
        })
        .await
        .map_err(|_| telemetry::internal_error("resource worker failed", None))?
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--version")
    {
        println!("{}", identity::build_info());
        return Ok(());
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--doctor")
    {
        let report = doctor::report();
        println!("{}", serde_json::to_string_pretty(&report)?);
        if report["ready"] != true {
            std::process::exit(1);
        }
        return Ok(());
    }
    // Initialize before Tokio so worker threads inherit the Sentry client.
    let _telemetry = telemetry::init();
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_server());
    if result.is_err() {
        sentry::capture_message("MCP server terminated unexpectedly", sentry::Level::Error);
    }
    result
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let registry = document::Registry::new();
    let request_cap = stdio_limit::cap(registry.workspace.max_input);
    let maintenance = retention::sweep(&registry);
    if maintenance["failures"].as_u64().unwrap_or(0) > 0 {
        // STDERR only: one malformed/inaccessible persisted document must not
        // prevent MCP startup or contaminate its JSON-RPC output.
        eprintln!(
            "startup retention skipped {} failed passes",
            maintenance["failures"]
        );
    }
    let server = Server {
        contract: Arc::new(contract::contract()),
        operations: Arc::new(tokio::sync::Mutex::new(())),
        workers: Arc::new(tokio::sync::Semaphore::new(4)),
        registry: Arc::new(Mutex::new(registry)),
        capabilities: Arc::new(Mutex::new(None)),
        runtime_bus: Arc::new(Mutex::new(None)),
        live: Arc::new(Mutex::new(live::Live::new())),
    };
    let result = server
        .serve((
            stdio_limit::Lines::new(tokio::io::stdin(), request_cap),
            tokio::io::stdout(),
        ))
        .await?
        .waiting()
        .await;
    engine::shutdown();
    result?;
    Ok(())
}

#[cfg(test)]
mod wire_result_tests {
    use super::*;

    #[test]
    fn cancelled_detection_does_not_publish_or_replace_capability_cache() {
        let registry = document::Registry::new();
        for previous in [None, Some(json!({"previous_success": true}))] {
            let server = Server {
                contract: Arc::new(contract::contract()),
                operations: Arc::new(tokio::sync::Mutex::new(())),
                workers: Arc::new(tokio::sync::Semaphore::new(4)),
                registry: Arc::new(Mutex::new(registry.clone())),
                capabilities: Arc::new(Mutex::new(previous.clone())),
                runtime_bus: Arc::new(Mutex::new(None)),
                live: Arc::new(Mutex::new(live::Live::new())),
            };
            let result = process::with_cancellation(Box::new(|| true), || {
                server.capabilities(&registry, true)
            });
            assert_eq!(result.unwrap_err(), "operation cancelled");
            assert_eq!(*server.capabilities.lock().unwrap(), previous);
            if previous.is_some() {
                assert_eq!(
                    server.capabilities(&registry, false).unwrap()["previous_success"],
                    true
                );
            }
        }
    }

    #[test]
    fn direct_result_preserves_all_frozen_scalar_and_error_shapes() {
        for input in [
            Ok(json!({"tree": [{"id": "r", "text": "a\nb", "bbox": null}]})),
            Ok(json!(["x", 2, null])),
            Ok(Value::Null),
            Ok(json!(true)),
            Ok(json!(false)),
            Ok(json!("text")),
            Ok(json!(1.5)),
            Err("refused: \"missing\"\nunchanged".to_owned()),
        ] {
            let legacy: CallToolResult = serde_json::from_value(match input.clone() {
                Ok(value) if value.is_boolean() => json!({
                    "content": [{"type": "text", "text": value.to_string()}],
                    "structuredContent": {"result": value}, "isError": false,
                    "_meta": {"fastmcp": {"wrap_result": true}}
                }),
                Ok(value) => json!({
                    "content": [{"type": "text", "text": value.to_string()}],
                    "structuredContent": value, "isError": false
                }),
                Err(message) => json!({
                    "content": [{"type": "text", "text": message}], "isError": true
                }),
            })
            .unwrap();
            assert_eq!(
                serde_json::to_value(tool_result(input)).unwrap(),
                serde_json::to_value(legacy).unwrap()
            );
        }
    }
}
