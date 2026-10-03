//! Read-only readiness of fixed transports. Never starts a service or a GUI.
use crate::{
    live_bus::{Action, Bus},
    live_protocol::Command,
    live_socket::Rendezvous,
    live_transport::Probe,
    workspace::Workspace,
};
use std::{path::PathBuf, time::Duration};
#[derive(Clone)]
pub struct Inputs {
    pub session_bus: bool,
    pub binary: Option<PathBuf>,
    pub macos: bool,
    pub stream: Option<PathBuf>,
    pub directory: Option<PathBuf>,
    pub timeout: Duration,
    pub cap: usize,
    pub address: Option<String>,
}
impl Inputs {
    pub fn environment(timeout: Duration, cap: usize) -> Self {
        let path = |key: &str| {
            std::env::var_os(key)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        Self {
            session_bus: std::env::var("DBUS_SESSION_BUS_ADDRESS")
                .is_ok_and(|s| !s.trim().is_empty()),
            binary: crate::process::binary("gdbus"),
            macos: cfg!(target_os = "macos"),
            stream: path("INKSCAPE_MCP_MANAGED_STDOUT"),
            directory: path("INKSCAPE_MCP_MANAGED_DIR"),
            timeout,
            cap,
            address: None,
        }
    }
}
const DBUS: [Command; 6] = [
    Command::Ping,
    Command::ActiveDocument,
    Command::DocumentSvg,
    Command::RenderView,
    Command::SetViewport,
    Command::ApplySelection,
];
const MANAGED: [Command; 10] = [
    Command::Ping,
    Command::ActiveDocument,
    Command::DocumentSvg,
    Command::Selection,
    Command::InspectSelection,
    Command::RenderView,
    Command::ApplySelection,
    Command::Scene,
    Command::InsertSvg,
    Command::SetText,
];
fn base(inputs: &Inputs, managed: bool) -> (Probe, Option<Bus>) {
    let mut probe = Probe {
        name: if managed { "managed-dbus" } else { "dbus" }.into(),
        available: false,
        rank: if managed { 30 } else { 10 },
        commands: DBUS.to_vec(),
        no_freeze: true,
        detail: String::new(),
    };
    if !inputs.session_bus {
        probe.detail = "no session bus (DBUS_SESSION_BUS_ADDRESS unset) — Linux/BSD only".into();
        return (probe, None);
    }
    let Some(binary) = &inputs.binary else {
        probe.detail = "gdbus not available; cannot drive org.gtk.Actions".into();
        return (probe, None);
    };
    let mut bus = Bus::new(binary.clone(), inputs.timeout, inputs.cap);
    bus.address(inputs.address.clone());
    probe.available = bus.connect().is_ok();
    probe.detail = if probe.available {
        "org.inkscape.Inkscape reachable on session bus (no-freeze action path)"
    } else {
        "session bus present; no org.inkscape.Inkscape instance running"
    }
    .into();
    (probe, Some(bus))
}
pub fn dbus(inputs: &Inputs) -> Probe {
    base(inputs, false).0
}
fn regular_stream(path: &std::path::Path) -> bool {
    let Ok(relative) = path.strip_prefix("/") else {
        return false;
    };
    let workspace = Workspace {
        roots: vec!["/".into()],
        max_input: 1,
        max_output: 1,
    };
    workspace.regular_file(0, relative).is_ok()
}
pub fn managed(inputs: &Inputs) -> Probe {
    let (mut probe, bus) = base(inputs, true);
    probe.commands = MANAGED.to_vec();
    if !inputs.macos || !inputs.stream.as_ref().is_some_and(|p| regular_stream(p)) {
        probe.available = false;
        probe.detail =
            "explicitly open Inkscape with live_launch or inkscape-mcp-macos --launch".into();
    } else if probe.available {
        probe.detail =
            "managed Inkscape: current selection and undoable edits, no modal session".into();
    }
    for (command, action) in [
        (Command::InsertSvg, Action::Insert),
        (Command::SetText, Action::Edit),
    ] {
        if !(probe.available
            && inputs.directory.is_some()
            && bus.as_ref().is_some_and(|bus| bus.effect_available(action)))
        {
            probe.commands.retain(|c| *c != command);
        }
    }
    probe
}
/// Reference socket readiness reports an advertising rendezvous, not a live
/// authenticated connection. Connect performs the separate bounded handshake.
pub fn socket(rendezvous: Option<&Rendezvous>, helper_installed: bool) -> Probe {
    Probe {
        name: "extension-socket".into(),
        available: rendezvous.is_some(),
        rank: 20,
        commands: Command::ALL
            .into_iter()
            .filter(|c| *c != Command::Hello)
            .collect(),
        no_freeze: false,
        detail: if let Some(rv) = rendezvous {
            format!("live session reachable on 127.0.0.1:{}", rv.port)
        } else if helper_installed {
            "helper installed; no live session advertising a socket".into()
        } else {
            "no rendezvous; helper not detected as installed".into()
        },
    }
}
/// Fixed attach-only backend factory for Session::connect. Constructing a
/// transport validates configured stream files without spawning a process;
/// connect performs its bounded handshake separately.
pub enum Kind {
    Socket,
    Dbus,
    Managed,
}
pub fn transport(
    inputs: &Inputs,
    kind: Kind,
    rendezvous: Option<Rendezvous>,
    guarded: bool,
    max_input: usize,
    max_output: usize,
) -> Result<Box<dyn crate::live_transport::Transport>, crate::live_socket::Error> {
    use crate::{
        live_dbus::Dbus, live_managed::Managed, live_socket::Error, live_transport::Socket,
    };
    if matches!(kind, Kind::Socket) {
        return Ok(Box::new(Socket::new(
            rendezvous.ok_or(Error::NotAvailable)?,
            inputs.timeout,
            max_output,
        )));
    }
    if !inputs.session_bus {
        return Err(Error::Connection("no session bus available"));
    }
    let mut bus = Bus::new(
        inputs
            .binary
            .clone()
            .ok_or(Error::Connection("gdbus not available"))?,
        inputs.timeout,
        inputs.cap,
    );
    bus.address(inputs.address.clone());
    let dbus = Dbus::new(bus, max_input);
    if matches!(kind, Kind::Dbus) {
        return Ok(Box::new(dbus));
    }
    if !inputs.macos {
        return Err(Error::NotAvailable);
    }
    let stream = inputs
        .stream
        .clone()
        .filter(|p| regular_stream(p))
        .ok_or(Error::NotAvailable)?;
    let mut backend = Managed::new(dbus, stream, guarded);
    if let Some(directory) = &inputs.directory {
        backend.effect_directory(directory.clone());
    }
    Ok(Box::new(backend))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn attach_factory_is_lazy_and_session_connect_uses_fixed_backends() {
        let (root, backend, log) = crate::live_dbus::tests::fixture("effect-success", 4096);
        let directory = root.path().canonicalize().unwrap();
        let stream = directory.join("stdout.log");
        std::fs::write(&stream, b"").unwrap();
        let inputs = Inputs {
            session_bus: true,
            binary: Some(backend.bus.binary.clone()),
            macos: true,
            stream: Some(stream),
            directory: Some(directory),
            timeout: Duration::from_secs(1),
            cap: 8192,
            address: None,
        };
        for (kind, name) in [(Kind::Dbus, "dbus"), (Kind::Managed, "managed-dbus")] {
            let before = std::fs::read(&log).unwrap();
            let backend = transport(&inputs, kind, None, true, 4096, 4096).unwrap();
            assert_eq!(backend.name(), name);
            assert!(!backend.is_connected());
            assert_eq!(std::fs::read(&log).unwrap(), before);
            let mut session = crate::live_session::Session::new(crate::live_session::Settings {
                enabled: true,
                cache_entries: 2,
                cache_bytes: 1024,
                coalesce_ms: 10.,
            });
            let probe = Probe {
                name: name.into(),
                available: true,
                rank: 30,
                commands: vec![Command::ActiveDocument],
                no_freeze: true,
                detail: String::new(),
            };
            let mut clears = 0;
            session
                .connect(
                    crate::live_transport::Preference::NoFreeze,
                    &[probe],
                    |_| Ok(backend),
                    || clears += 1,
                )
                .unwrap();
            assert_eq!(clears, 1);
            assert!(session.active_document.is_object());
            assert!(session.require_transport().is_ok());
            session.teardown(|| clears += 1);
            assert_eq!(clears, 2);
            assert!(session.require_transport().is_err());
        }
        assert!(transport(&inputs, Kind::Socket, None, false, 4096, 4096).is_err());
    }
    #[test]
    fn compiled_python_host_probe_profiles_match_without_actions() {
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/transport-probe-cases.json"
        ))
        .unwrap();
        let (root, backend, log) = crate::live_dbus::tests::fixture("effect-success", 4096);
        let directory = root.path().canonicalize().unwrap();
        let stream = directory.join("stdout.log");
        std::fs::write(&stream, b"").unwrap();
        for case in cases["bus"].as_array().unwrap() {
            let spec = &case["input"];
            std::fs::write(directory.join("probe-control.json"),serde_json::to_vec(&json!({"reachable":spec["reachable"],"insert":spec["insert"],"edit":spec["edit"]})).unwrap()).unwrap();
            let inputs = Inputs {
                session_bus: spec["session"] == true,
                binary: if spec["binary"] == true {
                    Some(backend.bus.binary.clone())
                } else {
                    None
                },
                macos: spec["mac"] == true,
                stream: Some(if spec["stream"] == true {
                    stream.clone()
                } else {
                    directory.join("missing")
                }),
                directory: if spec["directory"] == true {
                    Some(directory.clone())
                } else {
                    None
                },
                timeout: Duration::from_secs(1),
                cap: 8192,
                address: None,
            };
            assert_eq!(dbus(&inputs).value(), case["dbus"], "{spec}");
            assert_eq!(managed(&inputs).value(), case["managed"], "{spec}");
        }
        for case in cases["socket"].as_array().unwrap() {
            let rv = Rendezvous {
                port: 33333,
                token: "private-token".into(),
                pid: None,
            };
            assert_eq!(
                socket(
                    if case["advertised"] == true {
                        Some(&rv)
                    } else {
                        None
                    },
                    case["installed"] == true
                )
                .value(),
                case["expected"]
            );
        }
        for line in std::fs::read_to_string(log).unwrap().lines() {
            let argv: Vec<String> = serde_json::from_str(line).unwrap();
            assert!(matches!(
                argv[7].as_str(),
                "org.freedesktop.DBus.GetNameOwner"
                    | "org.gtk.Actions.List"
                    | "org.gtk.Actions.Describe"
            ));
            assert_eq!(
                argv[3],
                if argv[7] == "org.freedesktop.DBus.GetNameOwner" {
                    "org.freedesktop.DBus"
                } else {
                    ":1.23"
                }
            );
        }
    }
    #[test]
    fn managed_stream_refuses_file_and_ancestor_links_and_no_session_has_no_calls() {
        let (root, backend, log) = crate::live_dbus::tests::fixture("effect-success", 4096);
        let directory = root.path().canonicalize().unwrap();
        let original = directory.join("stdout.log");
        std::fs::write(&original, b"keep").unwrap();
        let mut inputs = Inputs {
            session_bus: true,
            binary: Some(backend.bus.binary.clone()),
            macos: true,
            stream: Some(original.clone()),
            directory: Some(directory.clone()),
            timeout: Duration::from_secs(1),
            cap: 8192,
            address: None,
        };
        assert!(managed(&inputs).available);
        for (link, target, child) in [
            ("stream-link", original.clone(), ""),
            ("dir-link", directory.clone(), "stdout.log"),
        ] {
            let path = directory.join(link);
            std::os::unix::fs::symlink(target, &path).unwrap();
            inputs.stream = Some(if child.is_empty() {
                path
            } else {
                path.join(child)
            });
            let result = managed(&inputs);
            assert!(!result.available);
            assert!(!result.commands.contains(&Command::InsertSvg));
        }
        assert_eq!(std::fs::read(&original).unwrap(), b"keep");
        let before = std::fs::read(&log).unwrap();
        inputs.session_bus = false;
        assert!(!dbus(&inputs).available);
        assert!(!managed(&inputs).available);
        assert_eq!(std::fs::read(&log).unwrap(), before);
    }
}
