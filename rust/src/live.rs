//! Public live lifecycle integration; probes/status/disconnect never launch GUI.
use crate::{
    live_probe,
    live_session::{Session, Settings},
    live_transport::{self, Preference, Probe},
    workspace::Workspace,
};
use serde_json::{Value, json};
use std::path::Path;
fn session() -> Session {
    let count = |key: &str, default: usize| {
        std::env::var(key)
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(default)
    };
    let budget = std::env::var("INKSCAPE_MCP_LIVE_COALESCE_BUDGET_MS")
        .ok()
        .and_then(|s| crate::decimal::float(&s))
        .filter(|n| n.is_finite() && *n >= 0.)
        .unwrap_or(200.);
    Session::new(Settings {
        enabled: crate::contract::flag("INKSCAPE_MCP_LIVE_ENABLED", true),
        cache_entries: count("INKSCAPE_MCP_LIVE_CACHE_MAX_ENTRIES", 64),
        cache_bytes: count("INKSCAPE_MCP_LIVE_CACHE_MAX_BYTES", 256 * 1024 * 1024),
        coalesce_ms: budget,
    })
}
pub struct Facts {
    pub probes: Vec<Probe>,
    pub helper_installed: bool,
    pub runtime_helper_installed: bool,
}
fn facts(
    workspace: &Workspace,
    capabilities: &Value,
    host: Option<&crate::live_attach::Attached>,
) -> Facts {
    let input = host.map(|host| host.inputs.clone()).unwrap_or_else(|| {
        live_probe::Inputs::environment(crate::process::timeout(), workspace.max_output)
    });
    let rendezvous =
        crate::live_socket::discover(capabilities["user_data_dir"].as_str(), workspace.max_input);
    let cached = capabilities["live_extension_socket_available"]
        .as_bool()
        .unwrap_or(false);
    let installed = ["system_data_dir", "user_data_dir"].iter().any(|key| {
        capabilities[*key].as_str().is_some_and(|directory| {
            Path::new(directory)
                .join("extensions/inkscape_mcp_live_run.sh")
                .is_file()
        })
    });
    Facts {
        probes: live_transport::ranked(&[
            live_probe::socket(rendezvous.as_ref(), installed),
            live_probe::dbus(&input),
            live_probe::managed(&input),
        ]),
        helper_installed: installed,
        runtime_helper_installed: cached,
    }
}
pub fn support(session: &Session, facts: &Facts) -> Value {
    let mut notes = Vec::new();
    if !session.settings.enabled {
        notes.push("live mode is disabled by configuration (INKSCAPE_MCP_LIVE_ENABLED opt-out)");
    }
    let available = facts.probes.iter().any(|p| p.available);
    if !available {
        notes.push("no running Inkscape detected via any transport");
    }
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        std::env::consts::OS
    };
    json!({"platform":platform,"live_enabled":session.settings.enabled,"any_available":available,"best_transport":live_transport::best(&facts.probes,Preference::Read).map(|p|p.name),"helper_installed":facts.helper_installed,"transports":facts.probes.iter().map(Probe::value).collect::<Vec<_>>(),"notes":notes})
}
/// Bounded best-effort record clearing on the configured first root. Unlike the
/// reference unlink glob, links are retained and never loaded as history.
pub fn clear_history(workspace: &Workspace) -> usize {
    if workspace.roots.is_empty() {
        return 0;
    }
    let directory = Path::new(".inkscape-mcp/live/operations");
    let Ok(entries) = workspace.directory_entries(0, directory) else {
        return 0;
    };
    entries
        .into_iter()
        .filter(|(name, mode)| {
            name.starts_with("op_")
                && name.ends_with(".json")
                && mode & libc::S_IFMT == libc::S_IFREG
        })
        .filter(|(name, _)| {
            workspace
                .regular_info(0, &directory.join(name), true)
                .is_ok_and(|info| info.is_some())
        })
        .count()
}
pub struct Live {
    pub session: Session,
    host: Option<crate::live_attach::Attached>,
}
impl Live {
    pub fn adopt_launch(&mut self, attached: crate::live_attach::Attached) {
        self.host = Some(attached);
    }
    pub fn prepare_connect(&mut self, workspace: &Workspace, args: &Value) -> Result<(), String> {
        let prefer = crate::arguments::string(args, "prefer")?.unwrap_or("read");
        Preference::parse(prefer).map_err(str::to_string)?;
        if !self.session.settings.enabled {
            return Err(crate::live_socket::Error::Disabled.public_message().into());
        }
        let mut inputs =
            live_probe::Inputs::environment(crate::process::timeout(), workspace.max_output);
        if inputs.directory.is_none() {
            inputs.directory = self
                .host
                .as_ref()
                .and_then(|host| host.inputs.directory.clone());
        }
        if inputs.macos && inputs.directory.is_some() {
            self.host = Some(crate::live_attach::refresh(inputs).map_err(str::to_string)?);
        }
        Ok(())
    }
    pub fn runtime_inputs(&self) -> Option<live_probe::Inputs> {
        self.host.as_ref().map(|host| host.inputs.clone())
    }
    pub fn new() -> Self {
        Self {
            session: session(),
            host: None,
        }
    }
    pub fn status(&mut self, workspace: &Workspace, capabilities: &Value) -> Value {
        let facts = facts(workspace, capabilities, self.host.as_ref());
        self.session
            .status(&facts.probes, facts.runtime_helper_installed)
    }
    pub fn call(
        &mut self,
        workspace: &Workspace,
        capabilities: &Value,
        name: &str,
        args: &Value,
    ) -> Result<Value, String> {
        use crate::{live_probe::Kind, live_socket::Error};
        if name == "live_connect" {
            let prefer = crate::arguments::string(args, "prefer")?.unwrap_or("read");
            let prefer = Preference::parse(prefer).map_err(str::to_string)?;
            if !self.session.settings.enabled {
                return Err(Error::Disabled.public_message().into());
            }
            let facts = facts(workspace, capabilities, self.host.as_ref());
            let inputs = self
                .host
                .as_ref()
                .map(|host| host.inputs.clone())
                .unwrap_or_else(|| {
                    live_probe::Inputs::environment(crate::process::timeout(), workspace.max_output)
                });
            let guarded = self
                .host
                .as_ref()
                .map(|host| host.guarded)
                .unwrap_or_else(|| {
                    std::env::var("INKSCAPE_MCP_CONTEXT_BRIDGE").is_ok_and(|v| v == "1")
                });
            let rendezvous = crate::live_socket::discover(
                capabilities["user_data_dir"].as_str(),
                workspace.max_input,
            );
            self.session
                .connect(
                    prefer,
                    &facts.probes,
                    |probe| {
                        let kind = match probe.name.as_str() {
                            "extension-socket" => Kind::Socket,
                            "dbus" => Kind::Dbus,
                            "managed-dbus" => Kind::Managed,
                            _ => return Err(Error::NotAvailable),
                        };
                        live_probe::transport(
                            &inputs,
                            kind,
                            rendezvous,
                            guarded,
                            workspace.max_input,
                            workspace.max_output,
                        )
                    },
                    || {
                        clear_history(workspace);
                    },
                )
                .map_err(|e| e.public_message().to_string())?;
            return Ok(self.status(workspace, capabilities));
        }
        if matches!(
            name,
            "check_live_support" | "live_status" | "live_disconnect"
        ) {
            let facts = facts(workspace, capabilities, self.host.as_ref());
            if name == "check_live_support" {
                return Ok(support(&self.session, &facts));
            }
            if name == "live_disconnect" {
                self.session.teardown(|| {
                    clear_history(workspace);
                });
            }
            return Ok(self
                .session
                .status(&facts.probes, facts.runtime_helper_installed));
        }
        match name {
            "live_apply_to_selection" => {
                return crate::live_mutation::apply_style(self, workspace, args);
            }
            "live_set_selected_text" => {
                return crate::live_mutation::apply_text(self, workspace, args);
            }
            "live_insert_svg" => return crate::live_mutation::apply_insert(self, workspace, args),
            "live_edit_selection" => {
                return crate::live_mutation::apply_structural(self, workspace, args);
            }
            _ => (),
        }
        if name == "live_session_step" {
            return crate::live_loop::step(self, workspace, args);
        }
        if name == "live_diff_view" {
            return crate::live_diff::diff(self, workspace, args);
        }
        if name == "live_preview_object" {
            return crate::live_discovery::preview(self, workspace, args);
        }
        if name == "live_find_objects" {
            return crate::live_discovery::find(self, workspace, args);
        }
        if name == "live_export_selection" {
            return self
                .session
                .require_transport()
                .and_then(|transport| crate::live_render::export_selection(transport, workspace))
                .map_err(|error| {
                    crate::live_events::protocol_message(&error)
                        .map(|message| {
                            format!("Error calling tool 'live_export_selection': {message}")
                        })
                        .unwrap_or_else(|| error.public_message().to_string())
                });
        }
        if matches!(name, "live_render_view" | "live_get_scene") {
            let frame = crate::live_view::frame(args)?;
            let mut cache = self.session.cache.take();
            let result = if name == "live_get_scene" {
                self.session.operation(|transport| {
                    let render =
                        crate::live_render::render(transport, cache.as_mut(), workspace, frame)?;
                    let scene = transport.scene()?;
                    Ok(json!({"render":render,"scene":scene}))
                })
            } else {
                self.session.require_transport().and_then(|transport| {
                    crate::live_render::render(transport, cache.as_mut(), workspace, frame)
                })
            };
            self.session.cache = cache;
            return result.map_err(|error| {
                crate::live_events::protocol_message(&error)
                    .map(|message| format!("Error calling tool '{name}': {message}"))
                    .unwrap_or_else(|| error.public_message().to_string())
            });
        }
        if name == "live_set_viewport" {
            let viewport = crate::live_view::viewport(args)?;
            let result = (|| {
                let transport = self.session.require_transport()?;
                if !transport.supports(crate::live_protocol::Command::SetViewport) {
                    return Err(Error::Unsupported("viewport unavailable"));
                }
                transport.set_viewport(&viewport)
            })();
            return result.map_err(|error| {
                crate::live_events::protocol_message(&error)
                    .map(|message| format!("Error calling tool 'live_set_viewport': {message}"))
                    .unwrap_or_else(|| error.public_message().to_string())
            });
        }
        let result = (|| {
            let transport = self.session.require_transport()?;
            match name {
                "live_get_active_document" => transport.active_document(),
                "live_get_selection" => transport.selection(),
                "live_inspect_selection" => transport.inspect_selection(),
                "live_list_documents" if transport.name() == "managed-dbus" => transport
                    .list_documents()
                    .map(|documents| json!({"documents":documents})),
                "live_select_document" if transport.name() == "managed-dbus" => {
                    let window = args["window_id"]
                        .as_str()
                        .ok_or(Error::Context("window_id must be a string"))?;
                    let document = args["document_id"]
                        .as_str()
                        .ok_or(Error::Context("document_id must be a string"))?;
                    transport.select_document(window, document)?;
                    Ok(Value::Null)
                }
                _ => Err(Error::Unsupported("managed document selection unavailable")),
            }
        })()
        .map_err(|e| e.public_message().to_string())?;
        if name == "live_select_document" {
            Ok(self.status(workspace, capabilities))
        } else {
            Ok(result)
        }
    }
    pub fn resource(&mut self, workspace: &Workspace, capabilities: &Value, leaf: &str) -> Value {
        if leaf == "session" {
            return self.status(workspace, capabilities);
        }
        if leaf == "selection" {
            return self
                .session
                .require_transport()
                .and_then(|t| t.selection())
                .unwrap_or_else(|_| json!({"object_ids":[],"count":0}));
        }
        self.session
            .require_transport()
            .and_then(|t| t.scene())
            .unwrap_or_else(|_| crate::live_models::scene(&json!({}), &Value::Null))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clearing_records_is_idempotent_and_preserves_links_and_non_records() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let directory = Path::new(".inkscape-mcp/live/operations");
        workspace.ensure_directory(0, directory).unwrap();
        for name in ["op_01234567.json", "op_legacy.json", "keep.json"] {
            workspace
                .atomic_write(0, &directory.join(name), b"keep")
                .unwrap();
        }
        let original = root.path().join("original");
        std::fs::write(&original, b"original").unwrap();
        std::os::unix::fs::symlink(&original, root.path().join(directory).join("op_link.json"))
            .unwrap();
        assert_eq!(clear_history(&workspace), 2);
        assert_eq!(clear_history(&workspace), 0);
        assert_eq!(std::fs::read(&original).unwrap(), b"original");
        assert!(
            root.path()
                .join(directory)
                .join("op_link.json")
                .is_symlink()
        );
        assert!(root.path().join(directory).join("keep.json").exists());
    }
}
