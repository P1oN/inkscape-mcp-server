//! Fixed bounded gdbus IPC pinned to a unique running owner. No service activation.
use crate::{live_socket::Error, process, runtime};
use regex::Regex;
use std::{path::PathBuf, sync::LazyLock, time::Duration};
const APP: &str = "/org/inkscape/Inkscape";
const CONTEXT: &str = "/org/inkscape/Inkscape/MCPContext";
#[derive(Clone, Copy)]
pub enum Action {
    ExportFilename,
    ExportType,
    ExportPlainSvg,
    ExportArea,
    ExportAreaPage,
    ExportDpi,
    ExportDo,
    ExportId,
    ExportIdOnly,
    ExportTextToPath,
    SelectList,
    QueryX,
    ZoomPage,
    ZoomSelection,
    ZoomAbsolute,
    ObjectProperty,
    Translate,
    Rotate,
    Scale,
    Insert,
    Edit,
}
impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Self::ExportFilename => "export-filename",
            Self::ExportType => "export-type",
            Self::ExportPlainSvg => "export-plain-svg",
            Self::ExportArea => "export-area",
            Self::ExportAreaPage => "export-area-page",
            Self::ExportDpi => "export-dpi",
            Self::ExportDo => "export-do",
            Self::ExportId => "export-id",
            Self::ExportIdOnly => "export-id-only",
            Self::ExportTextToPath => "export-text-to-path",
            Self::SelectList => "select-list",
            Self::QueryX => "query-x",
            Self::ZoomPage => "canvas-zoom-page",
            Self::ZoomSelection => "canvas-zoom-selection",
            Self::ZoomAbsolute => "canvas-zoom-absolute",
            Self::ObjectProperty => "object-set-property",
            Self::Translate => "transform-translate",
            Self::Rotate => "transform-rotate",
            Self::Scale => "transform-scale",
            Self::Insert => "org.inkscape-mcp.insert.noprefs",
            Self::Edit => "org.inkscape-mcp.edit.noprefs",
        }
    }
    pub fn mutates(self) -> bool {
        matches!(
            self,
            Self::ObjectProperty
                | Self::Translate
                | Self::Rotate
                | Self::Scale
                | Self::Insert
                | Self::Edit
        )
    }
}
pub enum Parameter<'a> {
    String(&'a str),
    Bool(bool),
    Double(f64),
    Empty,
}
impl Parameter<'_> {
    pub fn text(&self) -> Result<String, Error> {
        match self {
            Self::String(s) => {
                if s.chars()
                    .any(|c| c == '\'' || c == '\\' || c <= '\u{1f}' || c == '\u{7f}')
                {
                    return Err(Error::Protocol(
                        "unsafe value rejected before DBus activation",
                    ));
                }
                Ok(format!("[<'{s}'>]"))
            }
            Self::Bool(b) => Ok(format!("[<{}>]", if *b { "true" } else { "false" })),
            Self::Double(n) => {
                if n.is_finite() {
                    Ok(format!("[<{}>]", crate::live_models::float_repr(*n)))
                } else {
                    Err(Error::Protocol(
                        "non-finite value rejected before DBus activation",
                    ))
                }
            }
            Self::Empty => Ok("@av []".into()),
        }
    }
}
pub enum Target {
    App,
    Window(u32),
}
impl Target {
    fn path(&self) -> String {
        match self {
            Self::App => APP.into(),
            Self::Window(id) => format!("{APP}/window/{id}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    window: String,
    document: String,
}
impl Identity {
    pub fn new(window: &str, document: &str) -> Result<Self, Error> {
        static UUID: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$").unwrap());
        if !UUID.is_match(window) || !UUID.is_match(document) {
            return Err(Error::Context(
                "use window_id and document_id returned by live_list_documents",
            ));
        }
        Ok(Self {
            window: window.into(),
            document: document.into(),
        })
    }
}
pub enum Context<'a> {
    Get,
    List,
    Select(&'a Identity),
    Activate(&'a Identity, Action, Parameter<'a>),
}
pub struct Bus {
    pub binary: PathBuf,
    pub timeout: Duration,
    pub cap: usize,
    owner: Option<String>,
    address: Option<String>,
}
impl Bus {
    pub fn new(binary: PathBuf, timeout: Duration, cap: usize) -> Self {
        Self {
            binary,
            timeout,
            cap: cap.min(64 * 1024 * 1024),
            owner: None,
            address: None,
        }
    }
    pub fn is_connected(&self) -> bool {
        self.owner.is_some()
    }
    pub fn disconnect(&mut self) {
        self.owner = None;
    }
    pub(crate) fn address(&mut self, address: Option<String>) {
        self.address = address;
    }
    fn addressed(&self, mut args: Vec<String>) -> Vec<String> {
        if let Some(address) = &self.address {
            args.splice(1..2, ["--address".into(), address.clone()]);
        }
        args
    }
    /// Read-only probes retain the captured unique owner. They never resolve
    /// the well-known application name or reconnect to a different instance.
    fn readonly(
        &self,
        method: &str,
        params: Vec<String>,
        timeout: Duration,
    ) -> Result<String, Error> {
        let args = self.argv(APP, method, params)?;
        let probe = Self {
            binary: self.binary.clone(),
            timeout,
            cap: self.cap,
            owner: self.owner.clone(),
            address: self.address.clone(),
        };
        probe.run(args, false, false)
    }
    pub fn reachable(&self) -> bool {
        self.readonly(
            "org.gtk.Actions.List",
            vec![],
            self.timeout.min(Duration::from_secs(2)),
        )
        .is_ok()
    }
    pub fn effect_available(&self, action: Action) -> bool {
        self.readonly(
            "org.gtk.Actions.Describe",
            vec![action.name().into()],
            self.timeout,
        )
        .is_ok_and(|reply| reply.starts_with("((true,"))
    }
    fn run(&self, args: Vec<String>, mutation: bool, context: bool) -> Result<String, Error> {
        if args.iter().map(String::len).sum::<usize>() > self.cap {
            return Err(Error::Protocol("DBus request exceeds size cap"));
        }
        let cap = if context {
            self.cap.min(1024 * 1024)
        } else {
            self.cap
        };
        let outcome =
            process::run_bounded(&self.binary, &args, self.timeout, cap).map_err(|_| {
                if mutation {
                    Error::Uncertain
                } else {
                    Error::Connection("could not reach Inkscape on the session bus")
                }
            })?;
        if outcome.timed_out {
            return Err(if mutation {
                Error::Uncertain
            } else {
                Error::Connection("DBus request timed out")
            });
        }
        if !outcome.success {
            if context
                && String::from_utf8_lossy(&outcome.stderr)
                    .contains("org.inkscape.MCP.ContextChanged")
            {
                return Err(Error::Context(
                    "document context unavailable or changed; list and select the drawing again",
                ));
            }
            return Err(if mutation {
                Error::Uncertain
            } else {
                Error::Connection("Inkscape rejected a DBus request")
            });
        }
        if outcome.stdout.len() >= cap {
            return Err(if mutation {
                Error::Uncertain
            } else {
                Error::Protocol("DBus response exceeds size cap")
            });
        }
        String::from_utf8(outcome.stdout)
            .map(|s| s.trim().to_owned())
            .map_err(|_| {
                if mutation {
                    Error::Uncertain
                } else {
                    Error::Protocol("DBus response is not UTF-8")
                }
            })
    }
    fn argv(&self, path: &str, method: &str, params: Vec<String>) -> Result<Vec<String>, Error> {
        let owner = self.owner.as_ref().ok_or(Error::NotAvailable)?;
        let mut args = vec![
            "call".into(),
            "--session".into(),
            "--dest".into(),
            owner.clone(),
            "--object-path".into(),
            path.into(),
            "--method".into(),
            method.into(),
        ];
        args.extend(params);
        Ok(self.addressed(args))
    }
    fn dispatch(
        &mut self,
        args: Vec<String>,
        mutation: bool,
        context: bool,
    ) -> Result<String, Error> {
        let result = self.run(args, mutation, context);
        if matches!(
            result,
            Err(Error::Connection(_) | Error::Uncertain | Error::Protocol(_))
        ) {
            self.disconnect();
        }
        result
    }
    pub fn connect(&mut self) -> Result<(), Error> {
        self.disconnect();
        let args = [
            "call",
            "--session",
            "--dest",
            "org.freedesktop.DBus",
            "--object-path",
            "/org/freedesktop/DBus",
            "--method",
            "org.freedesktop.DBus.GetNameOwner",
            "org.inkscape.Inkscape",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let reply = self.run(self.addressed(args), false, false)?;
        self.owner = Some(runtime::owner_name(&reply).ok_or(Error::Connection(
            "Inkscape did not answer on the session bus",
        ))?);
        self.list().map(|_| ())
    }
    pub fn list(&mut self) -> Result<String, Error> {
        let args = self.argv(APP, "org.gtk.Actions.List", vec![])?;
        self.dispatch(args, false, false)
    }
    pub fn describe(&mut self, action: Action) -> Result<String, Error> {
        let args = self.argv(APP, "org.gtk.Actions.Describe", vec![action.name().into()])?;
        self.dispatch(args, false, false)
    }
    pub fn activate(
        &mut self,
        action: Action,
        parameter: Parameter<'_>,
        target: Target,
    ) -> Result<(), Error> {
        let args = self.activation_argv(action, parameter, target)?;
        self.dispatch(args, action.mutates(), false).map(|_| ())
    }
    fn activation_argv(
        &self,
        action: Action,
        parameter: Parameter<'_>,
        target: Target,
    ) -> Result<Vec<String>, Error> {
        self.argv(
            &target.path(),
            "org.gtk.Actions.Activate",
            vec![action.name().into(), parameter.text()?, "{}".into()],
        )
    }
    /// Validate a whole fixed action plan before sending its first mutation.
    pub fn validate_activation(
        &self,
        action: Action,
        parameter: Parameter<'_>,
        target: Target,
    ) -> Result<(), Error> {
        let args = self.activation_argv(action, parameter, target)?;
        if args.iter().map(String::len).sum::<usize>() > self.cap {
            Err(Error::Protocol("DBus request exceeds size cap"))
        } else {
            Ok(())
        }
    }
    pub fn introspect(&mut self) -> Result<String, Error> {
        let owner = self.owner.as_ref().ok_or(Error::NotAvailable)?;
        let args = vec![
            "introspect".into(),
            "--session".into(),
            "--dest".into(),
            owner.clone(),
            "--object-path".into(),
            APP.into(),
            "--recurse".into(),
        ];
        self.dispatch(self.addressed(args), false, false)
    }
    pub fn context(&mut self, request: Context<'_>) -> Result<String, Error> {
        let (args, mutation) = self.context_argv(request)?;
        self.dispatch(args, mutation, true)
    }
    pub fn validate_context(&self, request: Context<'_>) -> Result<(), Error> {
        let (args, _) = self.context_argv(request)?;
        if args.iter().map(String::len).sum::<usize>() > self.cap {
            Err(Error::Protocol("DBus request exceeds size cap"))
        } else {
            Ok(())
        }
    }
    fn context_argv(&self, request: Context<'_>) -> Result<(Vec<String>, bool), Error> {
        let (method, params, mutation) = match request {
            Context::Get => ("GetContext", vec![], false),
            Context::List => ("ListDocuments", vec![], false),
            Context::Select(id) => (
                "SelectDocument",
                vec![id.window.clone(), id.document.clone()],
                false,
            ),
            Context::Activate(id, action, param) => (
                "Activate",
                vec![
                    id.window.clone(),
                    id.document.clone(),
                    action.name().into(),
                    param.text()?,
                ],
                action.mutates(),
            ),
        };
        let args = self.argv(
            CONTEXT,
            &format!("org.inkscape.MCP.Context1.{method}"),
            params,
        )?;
        Ok((args, mutation))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::os::unix::fs::PermissionsExt;
    fn fixture(mode: &str) -> (tempfile::TempDir, Bus, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let binary = root.path().join("gdbus");
        let log = root.path().join("calls.jsonl");
        let python = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(".venv/bin/python");
        let body = format!(
            r#"#!{}
import sys,json,time
from pathlib import Path
with Path({:?}).open('a') as f: f.write(json.dumps(sys.argv[1:])+'\n')
mode={:?}
if 'org.freedesktop.DBus.GetNameOwner' in sys.argv:
    if mode=='unowned': raise SystemExit(1)
    if mode=='bad-owner': print("('org.inkscape.Inkscape',)")
    else: print("(':1.23',)")
elif 'org.gtk.Actions.List' in sys.argv: print('(list,)')
elif mode=='context-data':
    row="('12345678-1234-1234-1234-123456789abc','abcdefab-1234-1234-1234-123456789abc','Drawing.svg')"
    if 'org.inkscape.MCP.Context1.GetContext' in sys.argv: print(row)
    elif 'org.inkscape.MCP.Context1.ListDocuments' in sys.argv: print('(['+row+'],)')
    else: raise SystemExit(1)
elif mode=='context-changed':
    sys.stderr.write('org.inkscape.MCP.ContextChanged: private detail')
    raise SystemExit(1)
elif mode=='timeout': time.sleep(2)
elif mode=='failed': raise SystemExit(1)
elif mode=='capped': print('x'*9000)
else: print('()')
"#,
            python.display(),
            log.to_string_lossy(),
            mode
        );
        std::fs::write(&binary, body).unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let bus = Bus::new(binary, Duration::from_secs(1), 8192);
        (root, bus, log)
    }
    fn calls(log: &PathBuf) -> Vec<Value> {
        std::fs::read_to_string(log)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
    #[test]
    fn compiled_reference_variant_strings_bools_doubles_and_empty_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/bus-variant-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let param = match case["kind"].as_str().unwrap() {
                "string" => Parameter::String(case["value"].as_str().unwrap()),
                "bool" => Parameter::Bool(case["value"].as_bool().unwrap()),
                "double" => Parameter::Double(
                    case["value"]
                        .as_f64()
                        .unwrap_or_else(|| case["value"].as_str().unwrap().parse().unwrap()),
                ),
                "empty" => Parameter::Empty,
                _ => panic!("bad fixture"),
            };
            let actual = match param.text() {
                Ok(text) => json!({"text":text}),
                Err(Error::Protocol(message)) => json!({"error":message}),
                _ => panic!("wrong variant error"),
            };
            assert_eq!(actual, case["expected"]);
        }
    }
    #[test]
    fn owner_is_pinned_for_all_actions_context_and_introspection_never_activated() {
        let (_root, mut bus, log) = fixture("normal");
        bus.connect().unwrap();
        bus.describe(Action::Insert).unwrap();
        bus.activate(Action::ZoomPage, Parameter::Empty, Target::Window(17))
            .unwrap();
        let id = Identity::new(
            "12345678-1234-1234-1234-123456789abc",
            "abcdefab-1234-1234-1234-123456789abc",
        )
        .unwrap();
        bus.context(Context::Get).unwrap();
        bus.context(Context::List).unwrap();
        bus.context(Context::Select(&id)).unwrap();
        bus.context(Context::Activate(
            &id,
            Action::ObjectProperty,
            Parameter::String("fill, #ff0000"),
        ))
        .unwrap();
        bus.introspect().unwrap();
        let trace = calls(&log);
        assert_eq!(trace.len(), 9);
        assert_eq!(trace[0][3], "org.freedesktop.DBus");
        for call in &trace[1..] {
            assert_eq!(call[3], ":1.23");
            assert!(
                !call
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v == "org.inkscape.Inkscape")
            );
        }
        assert_eq!(trace[3][5], "/org/inkscape/Inkscape/window/17");
        assert_eq!(trace[7][8], id.window);
        assert_eq!(trace[7][9], id.document);
        assert_eq!(trace[7][10], "object-set-property");
        assert_eq!(trace[7][11], "[<'fill, #ff0000'>]");
        bus.disconnect();
        assert!(matches!(bus.list(), Err(Error::NotAvailable)));
        assert_eq!(calls(&log).len(), 9);
    }
    #[test]
    fn context_replies_bind_validated_identity_through_the_pinned_owner() {
        let (_root, mut bus, log) = fixture("context-data");
        bus.connect().unwrap();
        let active = crate::live_context::read(&mut bus).unwrap();
        let documents = crate::live_context::list(&mut bus).unwrap();
        assert_eq!(documents, vec![active.clone()]);
        assert_eq!(active["name"], "Drawing.svg");
        let trace = calls(&log);
        assert_eq!(trace.len(), 4);
        for call in &trace[1..] {
            assert_eq!(call[3], ":1.23");
        }
        assert_eq!(trace[2][7], "org.inkscape.MCP.Context1.GetContext");
        assert_eq!(trace[3][7], "org.inkscape.MCP.Context1.ListDocuments");
    }
    #[test]
    fn missing_or_bad_owner_never_sends_a_call_to_the_application() {
        for mode in ["unowned", "bad-owner"] {
            let (_root, mut bus, log) = fixture(mode);
            assert!(bus.connect().is_err());
            assert!(!bus.is_connected());
            assert_eq!(calls(&log).len(), 1);
        }
        assert!(Identity::new("bad;token", "bad").is_err());
    }
    #[test]
    fn mutations_are_uncertain_on_timeout_failure_or_truncated_stdout_without_retry() {
        for mode in ["timeout", "failed", "capped"] {
            let (_root, mut bus, log) = fixture(mode);
            bus.connect().unwrap();
            assert_eq!(
                bus.activate(
                    Action::ObjectProperty,
                    Parameter::String("fill, #ff0000"),
                    Target::App
                ),
                Err(Error::Uncertain)
            );
            assert!(!bus.is_connected());
            assert!(matches!(bus.list(), Err(Error::NotAvailable)));
            assert_eq!(calls(&log).len(), 3);
        }
        let (_root, mut bus, log) = fixture("context-changed");
        bus.connect().unwrap();
        let id = Identity::new(
            "12345678-1234-1234-1234-123456789abc",
            "abcdefab-1234-1234-1234-123456789abc",
        )
        .unwrap();
        let error = bus
            .context(Context::Activate(
                &id,
                Action::ObjectProperty,
                Parameter::String("fill, #ff0000"),
            ))
            .unwrap_err();
        assert!(matches!(error, Error::Context(_)));
        assert!(bus.is_connected());
        assert!(!error.public_message().contains("private detail"));
        assert_eq!(calls(&log).len(), 3);
    }
}
