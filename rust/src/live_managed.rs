//! Managed document binding and cross-client operation scope. Helper transactions are separate.
use crate::{
    live_bus::{Action, Context, Identity, Parameter, Target},
    live_context,
    live_dbus::Dbus,
    live_protocol::Command,
    live_socket::Error,
    live_transport::Transport,
    workspace::Workspace,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs::File,
    os::fd::AsRawFd,
    os::unix::fs::FileExt,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
    thread::ThreadId,
    time::{Duration, Instant},
};
struct Operation {
    stream: PathBuf,
    current: Option<Value>,
    lock: Arc<File>,
    stdout: Arc<File>,
    depth: usize,
}
static OPERATIONS: LazyLock<Mutex<HashMap<ThreadId, Operation>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
fn identity(document: &Value) -> Result<Identity, Error> {
    Identity::new(
        document["window_id"].as_str().unwrap_or(""),
        document["document_id"].as_str().unwrap_or(""),
    )
}
pub struct Managed {
    pub dbus: Dbus,
    stream: PathBuf,
    guarded: bool,
    selected: Option<Value>,
    current: Option<Value>,
    lock: Option<Arc<File>>,
    stdout: Option<Arc<File>>,
    owner: Option<ThreadId>,
    depth: usize,
    root_id: Option<String>,
    effect_directory: Option<PathBuf>,
}
impl Managed {
    pub fn new(mut dbus: Dbus, stream: PathBuf, guarded: bool) -> Self {
        dbus.managed_export();
        Self {
            dbus,
            stream,
            guarded,
            selected: None,
            current: None,
            lock: None,
            stdout: None,
            owner: None,
            depth: 0,
            root_id: None,
            effect_directory: None,
        }
    }
    pub fn require_selected(&self) -> Result<(), Error> {
        if !self.guarded {
            return Ok(());
        }
        let selected = self.selected.as_ref().ok_or(Error::Context(
            "choose the task drawing with live_select_document before editing",
        ))?;
        if self.current.as_ref().and_then(|v| identity(v).ok()) != Some(identity(selected)?) {
            return Err(Error::Context(
                "task drawing changed; call live_select_document before editing",
            ));
        }
        Ok(())
    }
    pub fn effect_directory(&mut self, directory: PathBuf) {
        self.effect_directory = Some(directory);
    }
    fn edit_available(&mut self) -> Result<bool, Error> {
        Ok(self.effect_directory.is_some() && self.dbus.bus.effect_available(Action::Edit))
    }
    fn edit_selection(&mut self, operation: &str, params: Value) -> Result<Value, Error> {
        if !self.edit_available()? {
            return Err(Error::Unsupported(
                "save and restart managed Inkscape to load everyday edits",
            ));
        }
        let directory = self
            .effect_directory
            .clone()
            .ok_or(Error::Connection("managed edit directory is unavailable"))?;
        let nonce = format!("mcp_{}", uuid::Uuid::new_v4().simple());
        self.operation(|this| {
            this.require_selected()?;
            let selected=this.selection()?;
            if selected["count"]==0 { return Err(Error::Protocol("select an object in the managed Inkscape window first")); }
            let svg=this.document_svg()?;
            let document=crate::xml::parse(svg.as_bytes(),this.dbus.max_input).map_err(|_|Error::Protocol("active document export is invalid"))?;
            let root=document.get_root_element().ok_or(Error::Protocol("active document export is invalid"))?;
            let ids:Vec<_>=crate::document::elements(root).iter().filter_map(|node|node.get_property_no_ns("id").filter(|id|!id.is_empty())).collect();
            let fingerprint=crate::live_effect::fingerprint(&svg,this.dbus.max_input)?;
            let mut payload=json!({"nonce":nonce,"operation":operation,"selection":selected["object_ids"],"expected_ids":ids,"expected_fingerprint":fingerprint});
            for (key,value) in params.as_object().unwrap() { payload[key]=value.clone(); }
            let exchange=crate::live_effect::Exchange::prepare(&directory,&payload,this.dbus.max_input)?;
            let activation=this.dbus.activate(Action::Edit,Parameter::Empty,Target::App);
            // Reconcile a trustworthy refusal even when activation lost its reply; never retry.
            if let Err(error)=activation {
                if matches!(error,Error::Connection(_)|Error::Uncertain) {
                    if let Ok(Some(crate::live_effect::Reply::Refused{reason}))=exchange.read() {
                        exchange.cleanup()?;
                        return Err(Error::EditRefused(crate::live_effect::refusal(&reason)));
                    }
                    return Err(Error::Uncertain);
                }
                return Err(error);
            }
            let deadline=Instant::now()+this.dbus.bus.timeout;
            loop {
                if let Some(reply)=exchange.read()? {
                    match reply {
                        crate::live_effect::Reply::Refused { reason }=> {
                            exchange.cleanup()?;
                            return Err(Error::EditRefused(crate::live_effect::refusal(&reason)));
                        },
                        crate::live_effect::Reply::Applied {fingerprint,ids}=> {
                            let current=this.document_svg().map_err(|_|Error::Uncertain)?;
                            let confirmed=crate::live_effect::fingerprint(&current,this.dbus.max_input).map_err(|_|Error::Uncertain)?;
                            if confirmed==fingerprint {
                                exchange.cleanup()?;
                                return Ok(json!({"affected_ids":ids,"count":ids.len(),"detail":format!("{operation}: one Undo step; unchanged calls add no step"),"undo_friendly":true}));
                            }
                        },
                    }
                }
                if Instant::now()>=deadline { return Err(Error::Uncertain); }
                std::thread::sleep(Duration::from_millis(50));
            }
        })
    }
    pub fn order_selection(&mut self, operation: SelectionEdit) -> Result<Value, Error> {
        self.edit_selection(operation.name(), json!({}))
    }
    pub fn operation<T>(
        &mut self,
        op: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.begin_operation()?;
        struct Scope<'a>(&'a mut Managed);
        impl Drop for Scope<'_> {
            fn drop(&mut self) {
                self.0.end_operation();
            }
        }
        let scope = Scope(self);
        op(scope.0)
    }
}
impl Transport for Managed {
    fn order_selection(&mut self, operation: SelectionEdit) -> Result<Value, Error> {
        Managed::order_selection(self, operation)
    }
    fn name(&self) -> &str {
        "managed-dbus"
    }
    fn supports(&self, command: Command) -> bool {
        // Read support is fixed; helper commands require a fresh readonly Describe.
        command == Command::ApplySelection
            || (command == Command::SetText
                && self.effect_directory.is_some()
                && self.dbus.bus.effect_available(Action::Edit))
            || (command == Command::InsertSvg
                && self.effect_directory.is_some()
                && self.dbus.bus.effect_available(Action::Insert))
            || matches!(
                command,
                Command::Ping
                    | Command::ActiveDocument
                    | Command::DocumentSvg
                    | Command::RenderView
                    | Command::Selection
                    | Command::InspectSelection
                    | Command::Scene
            )
    }
    fn connect(&mut self) -> Result<(), Error> {
        self.disconnect();
        self.dbus.connect()
    }
    fn insert_svg(&mut self, fragment: &str) -> Result<Value, Error> {
        let nonce = format!("mcp_{}", uuid::Uuid::new_v4().simple());
        let ids = crate::live_insert::plan(fragment, &nonce)
            .map_err(|_| Error::Context("unsupported or unsafe SVG insertion fragment"))?;
        let directory = self.effect_directory.clone().ok_or(Error::Unsupported(
            "restart managed Inkscape to load the insertion helper",
        ))?;
        if !self.dbus.bus.effect_available(Action::Insert) {
            return Err(Error::Unsupported(
                "restart managed Inkscape to load the insertion helper",
            ));
        }
        self.operation(|this| {
            this.require_selected()?;
            let svg = this.document_svg()?;
            let doc = crate::xml::parse(svg.as_bytes(), this.dbus.max_input).map_err(|_| Error::Protocol("active document export is invalid"))?;
            let root = doc.get_root_element().ok_or(Error::Protocol("active document export is invalid"))?;
            let expected: Vec<_> = crate::document::elements(root).iter().filter_map(|n| n.get_property_no_ns("id").filter(|id| !id.is_empty())).collect();
            if ids.iter().any(|id| expected.contains(id)) { return Err(Error::Context("Inkscape refused insertion; document context changed")); }
            let payload = json!({"nonce":nonce,"fragment":fragment,"expected_ids":expected,"expected_fingerprint":crate::live_effect::fingerprint(&svg,this.dbus.max_input)?});
            let exchange = crate::live_effect::Exchange::prepare(&directory,&payload,this.dbus.max_input.min(2 * 1024 * 1024))?;
            if let Err(error) = this.dbus.activate(Action::Insert,Parameter::Empty,Target::App) {
                if matches!(error,Error::Connection(_)|Error::Uncertain) {
                    if exchange.read_insert() == Ok(Some(false)) {
                        exchange.cleanup()?;
                        return Err(Error::Context("Inkscape refused insertion; document context changed"));
                    }
                    return Err(Error::Uncertain);
                }
                return Err(error);
            }
            let deadline = Instant::now() + this.dbus.bus.timeout;
            loop {
                if let Some(ok) = exchange.read_insert()? {
                    if !ok {
                        exchange.cleanup()?;
                        return Err(Error::Context("Inkscape refused insertion; document context changed"));
                    }
                    let current = this.document_svg().map_err(|_| Error::Uncertain)?;
                    if crate::live_insert::confirmed_fragment(&current,&nonce,&ids,fragment,this.dbus.max_input)? {
                        exchange.cleanup()?;
                        return Ok(json!({"affected_ids":ids,"count":ids.len(),"detail":"inserted one SVG group; one native Undo step","undo_friendly":true}));
                    }
                }
                if Instant::now() >= deadline { return Err(Error::Uncertain); }
                std::thread::sleep(Duration::from_millis(50));
            }
        })
    }
    fn disconnect(&mut self) {
        while self.depth > 0 {
            self.end_operation();
        }
        self.depth = 0;
        self.current = None;
        self.selected = None;
        self.lock = None;
        self.stdout = None;
        self.dbus.bind_context(None);
        self.dbus.disconnect();
    }
    fn is_connected(&self) -> bool {
        self.dbus.bus.is_connected() && self.dbus.bus.reachable()
    }
    fn guard_available(&self) -> bool {
        self.guarded
    }
    fn selected_document(&self) -> Option<Value> {
        self.selected.clone()
    }
    fn list_documents(&mut self) -> Result<Value, Error> {
        if !self.guarded {
            return Err(Error::Unsupported(
                "restart a managed session with the context bridge after saving your work",
            ));
        }
        live_context::list(&mut self.dbus.bus).map(|rows| json!(rows))
    }
    fn select_document(&mut self, window: &str, document: &str) -> Result<Value, Error> {
        let selected = Identity::new(window, document)?;
        if !self.guarded {
            return Err(Error::Unsupported("document context bridge is unavailable"));
        }
        self.operation(|this| {
            this.dbus.bus.context(Context::Select(&selected))?;
            let deadline = Instant::now() + this.dbus.bus.timeout.min(Duration::from_secs(2));
            loop {
                let current = live_context::read(&mut this.dbus.bus)?;
                if identity(&current)? == selected {
                    this.selected = Some(current.clone());
                    return Ok(current);
                }
                if Instant::now() >= deadline {
                    return Err(Error::Context(
                        "activate the chosen drawing and call live_select_document again",
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        })
    }
    fn begin_operation(&mut self) -> Result<(), Error> {
        let thread = std::thread::current().id();
        if self.owner.is_some_and(|owner| owner != thread) {
            return Err(Error::Protocol(
                "cannot move an active managed operation between threads",
            ));
        }
        {
            let mut operations = OPERATIONS.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(operation) = operations.get_mut(&thread) {
                if operation.stream != self.stream {
                    return Err(Error::Protocol(
                        "cannot nest operations from different managed sessions",
                    ));
                }
                if operation.depth >= 16 {
                    return Err(Error::Protocol("managed operation nesting exceeds limit"));
                }
                let binding = if self.guarded {
                    Some(identity(operation.current.as_ref().ok_or(
                        Error::Protocol("context action requires a managed operation"),
                    )?)?)
                } else {
                    None
                };
                self.dbus.bind_context(binding);
                self.current = operation.current.clone();
                self.lock = Some(operation.lock.clone());
                self.stdout = Some(operation.stdout.clone());
                self.owner = Some(thread);
                self.depth += 1;
                operation.depth += 1;
                return Ok(());
            }
        }
        if !self.stream.is_absolute() {
            return Err(Error::Connection(
                "managed Inkscape stdout is not configured",
            ));
        }
        let workspace = Workspace {
            roots: vec![PathBuf::from("/")],
            max_input: 1024 * 1024,
            max_output: 1024 * 1024,
        };
        let relative = self
            .stream
            .strip_prefix("/")
            .map_err(|_| Error::Connection("managed Inkscape stdout is not configured"))?;
        // Require the stream itself to be a no-follow regular file before creating its lock.
        let stdout = workspace
            .regular_file(0, relative)
            .map_err(|_| Error::Connection("managed Inkscape stdout is unavailable"))?;
        let lock = workspace
            .lock_file(0, &relative.with_extension("lock"))
            .map_err(|_| Error::Connection("managed Inkscape lock is unavailable"))?;
        let deadline = Instant::now() + self.dbus.bus.timeout;
        loop {
            // SAFETY: uniquely owned live regular-file descriptor; advisory flock releases on close.
            if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                break;
            }
            let error = std::io::Error::last_os_error();
            if !matches!(error.raw_os_error(), Some(libc::EWOULDBLOCK | libc::EINTR)) {
                return Err(Error::Connection("managed Inkscape lock is unavailable"));
            }
            if Instant::now() >= deadline {
                return Err(Error::Connection("managed Inkscape is busy"));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let current = if self.guarded {
            Some(live_context::read(&mut self.dbus.bus)?)
        } else {
            None
        };
        self.dbus
            .bind_context(current.as_ref().map(identity).transpose()?);
        let lock = Arc::new(lock);
        let stdout = Arc::new(stdout);
        OPERATIONS.lock().unwrap_or_else(|e| e.into_inner()).insert(
            thread,
            Operation {
                stream: self.stream.clone(),
                current: current.clone(),
                lock: lock.clone(),
                stdout: stdout.clone(),
                depth: 1,
            },
        );
        self.current = current;
        self.lock = Some(lock);
        self.stdout = Some(stdout);
        self.owner = Some(thread);
        self.depth = 1;
        Ok(())
    }
    fn end_operation(&mut self) {
        if self.depth == 0 {
            return;
        }
        self.depth -= 1;
        if let Some(owner) = self.owner {
            let mut operations = OPERATIONS.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(operation) = operations.get_mut(&owner) {
                operation.depth -= 1;
                if operation.depth == 0 {
                    operations.remove(&owner);
                }
            }
        }
        if self.depth == 0 {
            self.dbus.bind_context(None);
            self.current = None;
            self.lock = None;
            self.stdout = None;
            self.owner = None;
        }
    }
    fn document_svg(&mut self) -> Result<String, Error> {
        self.operation(|this| {
            let svg = this.dbus.document_svg()?;
            let document = crate::xml::parse(svg.as_bytes(), this.dbus.max_input)
                .map_err(|_| Error::Protocol("active document export is invalid"))?;
            this.root_id = document
                .get_root_element()
                .and_then(|root| root.get_property_no_ns("id"));
            Ok(svg)
        })
    }
    fn apply_selection(
        &mut self,
        style: &std::collections::BTreeMap<String, String>,
        transform: Option<&str>,
    ) -> Result<Value, Error> {
        if self.edit_available()? {
            return self.edit_selection("style", json!({"style":style,"transform":transform}));
        }
        if style.len() != 1 || !style.contains_key("fill") || transform.is_some() {
            return Err(Error::Unsupported(
                "save and restart managed Inkscape to load everyday edits",
            ));
        }
        self.operation(|this| {
            this.require_selected()?;
            let selection = this.selection()?;
            if selection["count"] == 0 {
                return Err(Error::Protocol(
                    "select an object in the managed Inkscape window first",
                ));
            }
            let mut result = this.dbus.apply_selection(style, None)?;
            result["affected_ids"] = selection["object_ids"].clone();
            result["count"] = selection["count"].clone();
            result["detail"] =
                json!("changed fill on the current selection; use Inkscape Undo to revert");
            Ok(result)
        })
    }
    fn set_text(&mut self, text: &str) -> Result<Value, Error> {
        self.edit_selection("text", json!({"text":text}))
    }
    fn selection(&mut self) -> Result<Value, Error> {
        self.operation(|this| {
            let stream = this
                .stdout
                .as_ref()
                .ok_or(Error::Connection("managed Inkscape stdout is unavailable"))?
                .clone();
            let mut offset = stream
                .metadata()
                .map_err(|_| Error::Connection("managed Inkscape stdout is unavailable"))?
                .len();
            this.dbus
                .activate(Action::SelectList, Parameter::Empty, Target::App)?;
            this.dbus
                .activate(Action::QueryX, Parameter::Empty, Target::App)?;
            let deadline = Instant::now() + this.dbus.bus.timeout.min(Duration::from_secs(5));
            let mut data = Vec::new();
            let mut chunk = [0u8; 65536];
            while Instant::now() < deadline {
                let read = stream
                    .read_at(&mut chunk, offset)
                    .map_err(|_| Error::Connection("managed Inkscape stdout is unavailable"))?;
                if data.len() + read > 1024 * 1024 {
                    return Err(Error::Protocol(
                        "managed selection response exceeds the size cap",
                    ));
                }
                data.extend_from_slice(&chunk[..read]);
                offset = offset.checked_add(read as u64).ok_or(Error::Protocol(
                    "managed selection response exceeds the size cap",
                ))?;
                if let Ok(text) = std::str::from_utf8(&data)
                    && let Some(mut ids) = crate::live_selection::parse(text)?
                {
                    ids.retain(|id| Some(id) != this.root_id.as_ref());
                    return Ok(json!({"count":ids.len(),"object_ids":ids}));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(Error::Connection(
                "managed Inkscape did not complete its selection reply",
            ))
        })
    }
    fn render_view(
        &mut self,
        region: Option<[f64; 4]>,
        scale: Option<f64>,
    ) -> Result<Vec<u8>, Error> {
        self.operation(|this| {
            let region = if let Some([x, y, width, height]) = region {
                let svg = this.document_svg()?;
                let document =
                    crate::xml::parse(svg.as_bytes(), this.dbus.max_input).map_err(|_| {
                        Error::Protocol("render region document could not be parsed safely")
                    })?;
                let root = document.get_root_element().ok_or(Error::Protocol(
                    "render region document could not be parsed safely",
                ))?;
                let (sx, sy, tx, ty) = crate::geometry::root_mapping(&root)
                    .map_err(|_| Error::Protocol("render region mapping unavailable"))?;
                Some([x * sx + tx, y * sy + ty, width * sx, height * sy])
            } else {
                None
            };
            this.dbus.render_view(region, scale)
        })
    }
    fn inspect_selection(&mut self) -> Result<Value, Error> {
        self.operation(|this| {
            let selection = this.selection()?;
            let ids = selection["object_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            let svg = this.document_svg()?;
            crate::live_scene::inspection(&svg, &ids, this.dbus.max_input)
        })
    }
    fn scene(&mut self) -> Result<Value, Error> {
        self.operation(|this| {
            let selection = this.selection()?;
            let ids = selection["object_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            let svg = this.document_svg()?;
            let mut scene = crate::live_scene::scene(&svg, &ids, this.dbus.max_input)?;
            if let Some(current) = this.current.as_ref() {
                scene["active_document"]["window_id"] = current["window_id"].clone();
                scene["active_document"]["document_id"] = current["document_id"].clone();
            }
            Ok(scene)
        })
    }
    fn active_document(&mut self) -> Result<Value, Error> {
        self.operation(|this| {
            let svg=this.document_svg()?;
            let document=crate::xml::parse(svg.as_bytes(),this.dbus.max_input).map_err(|_|Error::Protocol("active document export is invalid"))?;
            let root=document.get_root_element().ok_or(Error::Protocol("active document export is invalid"))?;
            let mut pending=root.get_child_elements();
            let mut count=0usize;
            while let Some(node)=pending.pop() { count+=1;pending.extend(node.get_child_elements()); }
            Ok(json!({"window_id":this.current.as_ref().map(|d|&d["window_id"]),"document_id":this.current.as_ref().map(|d|&d["document_id"]),"name":root.get_property_ns("docname","http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"),"path":null,"object_count":count}))
        })
    }
}
impl Drop for Managed {
    fn drop(&mut self) {
        self.disconnect();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_dbus::tests::fixture;
    use std::{
        io::{BufRead, BufReader},
        process::{Command as Process, Stdio},
    };
    fn managed(guarded: bool) -> (tempfile::TempDir, Managed, PathBuf) {
        let (root, dbus, log) = fixture("normal", 1024 * 1024);
        let stream = root
            .path()
            .join("stdout.log")
            .canonicalize()
            .unwrap_or_else(|_| root.path().canonicalize().unwrap().join("stdout.log"));
        std::fs::write(&stream, b"existing stdout\n").unwrap();
        let backend = Managed::new(dbus, stream, guarded);
        (root, backend, log)
    }
    fn trace(log: &std::path::Path) -> Vec<Vec<String>> {
        std::fs::read_to_string(log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    #[test]
    fn frozen_selected_task_guards_cover_all_fifty_cases() {
        let (root, mut backend, _) = managed(true);
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/task-guard-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            backend.guarded = case["guarded"].as_bool().unwrap();
            backend.selected = (!case["selected"].is_null()).then(|| case["selected"].clone());
            backend.current = (!case["current"].is_null()).then(|| case["current"].clone());
            let actual = match backend.require_selected() {
                Ok(()) => json!({"result":null}),
                Err(error) => json!({"error":error.public_message()}),
            };
            assert_eq!(actual, case["expected"]);
        }
        assert_eq!(
            std::fs::read(root.path().join("stdout.log")).unwrap(),
            b"existing stdout\n"
        );
    }
    #[test]
    fn guarded_scope_selects_document_pins_all_exports_and_refuses_context_change() {
        let (root, mut backend, log) = managed(true);
        let docs = backend.list_documents().unwrap();
        let window = docs[0]["window_id"].as_str().unwrap();
        let doc = docs[0]["document_id"].as_str().unwrap();
        let chosen = backend.select_document(window, doc).unwrap();
        assert_eq!(backend.selected_document(), Some(chosen.clone()));
        let value = backend
            .operation(|this| {
                this.require_selected()?;
                let result = this.active_document()?;
                assert_eq!(this.depth, 1);
                Ok(result)
            })
            .unwrap();
        assert_eq!(value["object_count"], 2);
        assert_eq!(value["name"], Value::Null); // actual managed namespace differs from legacy DBus.
        assert_eq!(value["window_id"], window);
        assert_eq!(value["document_id"], doc);
        let calls = trace(&log);
        let actions: Vec<_> = calls
            .iter()
            .filter(|argv| {
                argv.get(7)
                    .is_some_and(|s| s == "org.inkscape.MCP.Context1.Activate")
            })
            .collect();
        assert_eq!(actions.len(), 7);
        for argv in actions {
            assert_eq!(argv[3], ":1.23");
            assert_eq!(argv[8], window);
            assert_eq!(argv[9], doc);
        }
        assert!(
            !calls
                .iter()
                .any(|argv| argv.get(7).is_some_and(|s| s == "org.gtk.Actions.Activate"))
        );
        assert_eq!(backend.depth, 0);
        assert!(backend.current.is_none());
        assert!(backend.lock.is_none());
        backend.begin_operation().unwrap();
        std::fs::write(
            root.path().join("context.json"),
            serde_json::to_vec(&json!([doc, window, "Changed"])).unwrap(),
        )
        .unwrap();
        let error = backend.dbus.document_svg().unwrap_err();
        assert!(matches!(error, Error::Context(_)));
        assert!(backend.is_connected());
        backend.end_operation();
        assert_eq!(
            backend.operation(|this| this.require_selected()),
            Err(Error::Context(
                "task drawing changed; call live_select_document before editing"
            ))
        );
        backend.disconnect();
        assert!(backend.selected_document().is_none());
    }
    #[test]
    fn error_and_unwind_release_lock_nested_scope_and_legacy_mode_never_capture_context() {
        let (_root, mut backend, log) = managed(false);
        assert_eq!(
            backend.operation(|this| {
                this.begin_operation()?;
                this.end_operation();
                Err::<(), _>(Error::Rejected)
            }),
            Err(Error::Rejected)
        );
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = backend.operation::<()>(|_| panic!("injected scope failure"));
        }));
        assert!(panic.is_err());
        assert_eq!(backend.depth, 0);
        assert!(backend.lock.is_none());
        let svg = backend.document_svg().unwrap();
        assert!(svg.contains("rect"));
        assert!(
            !trace(&log)
                .iter()
                .any(|argv| argv.iter().any(|v| v.contains("Context1")))
        );
        backend.begin_operation().unwrap();
        for _ in 1..16 {
            backend.begin_operation().unwrap();
        }
        assert!(backend.begin_operation().is_err());
        for _ in 0..16 {
            backend.end_operation();
        }
        assert!(backend.lock.is_none());
    }
    #[test]
    fn managed_liveness_and_helper_support_refresh_only_the_pinned_owner() {
        let (root, dbus, log) = fixture_for_effect("effect-success");
        let directory = root.path().canonicalize().unwrap();
        let mut backend = Managed::new(dbus, directory.join("stdout.log"), false);
        backend.effect_directory(directory.clone());
        assert!(backend.is_connected());
        assert!(backend.supports(Command::InsertSvg));
        assert!(backend.supports(Command::SetText));
        let before = trace(&log).len();
        std::fs::write(
            directory.join("probe-control.json"),
            br#"{"reachable":false}"#,
        )
        .unwrap();
        assert!(!backend.is_connected());
        assert!(!backend.supports(Command::InsertSvg));
        assert!(!backend.supports(Command::SetText));
        // Restoring reachability never resolves/replaces the original owner.
        std::fs::write(
            directory.join("probe-control.json"),
            br#"{"reachable":true,"insert":false,"edit":true}"#,
        )
        .unwrap();
        assert!(backend.is_connected());
        assert!(!backend.supports(Command::InsertSvg));
        assert!(backend.supports(Command::SetText));
        for argv in &trace(&log)[before..] {
            assert_eq!(argv[3], ":1.23");
            assert!(matches!(
                argv[7].as_str(),
                "org.gtk.Actions.List" | "org.gtk.Actions.Describe"
            ));
        }
        backend.disconnect();
        let before = trace(&log).len();
        assert!(!backend.is_connected());
        assert!(!backend.supports(Command::InsertSvg));
        assert_eq!(trace(&log).len(), before);
    }
    #[test]
    fn frozen_insertion_transactions_match_request_and_result() {
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/insertion-transaction-cases.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let mode = if case["refused"] == true {
                "effect-refused"
            } else {
                "effect-success"
            };
            let (root, dbus, log) = fixture_for_effect(mode);
            let mut backend = Managed::new(
                dbus,
                root.path().canonicalize().unwrap().join("stdout.log"),
                true,
            );
            backend.effect_directory(root.path().canonicalize().unwrap());
            let docs = backend.list_documents().unwrap();
            backend
                .select_document(
                    docs[0]["window_id"].as_str().unwrap(),
                    docs[0]["document_id"].as_str().unwrap(),
                )
                .unwrap();
            let outcome = backend.insert_svg(case["fragment"].as_str().unwrap());
            let mut request: Value = serde_json::from_slice(
                &std::fs::read(root.path().join("captured-request.json")).unwrap(),
            )
            .unwrap();
            let nonce = request["nonce"].as_str().unwrap().to_owned();
            assert_eq!(nonce.len(), 36);
            assert!(
                nonce[4..]
                    .bytes()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            );
            let mut actual = match outcome {
                Ok(value) => json!({"result":value}),
                Err(error) => json!({"error":error.public_message()}),
            };
            if actual["result"].is_object() {
                assert_eq!(
                    actual["result"]["affected_ids"],
                    json!(
                        crate::live_insert::plan(case["fragment"].as_str().unwrap(), &nonce)
                            .unwrap()
                    )
                );
                for id in actual["result"]["affected_ids"].as_array_mut().unwrap() {
                    *id = json!(id.as_str().unwrap().replacen(
                        &nonce,
                        case["request"]["nonce"].as_str().unwrap(),
                        1
                    ));
                }
            }
            request["nonce"] = case["request"]["nonce"].clone();
            assert_eq!(request, case["request"]);
            assert_eq!(actual, case["expected"]);
            let calls = trace(&log);
            assert_eq!(
                calls
                    .iter()
                    .filter(|a| a
                        .get(7)
                        .is_some_and(|m| m == "org.inkscape.MCP.Context1.Activate")
                        && a.get(10)
                            .is_some_and(|m| m == "org.inkscape-mcp.insert.noprefs"))
                    .count(),
                1
            );
            assert!(!root.path().join("insert-request.json").exists());
            assert!(!root.path().join("insert-result.json").exists());
            assert_eq!(backend.depth, 0);
        }
    }
    #[test]
    fn insertion_faults_are_uncertain_without_retry_and_preflight_never_dispatches() {
        for mode in [
            "effect-lost",
            "effect-stale",
            "effect-missing",
            "effect-mismatch",
            "effect-switch",
            "effect-lost-refused",
        ] {
            let (root, dbus, log) = fixture_for_effect(mode);
            let mut backend = Managed::new(
                dbus,
                root.path().canonicalize().unwrap().join("stdout.log"),
                true,
            );
            backend.effect_directory(root.path().canonicalize().unwrap());
            let docs = backend.list_documents().unwrap();
            backend
                .select_document(
                    docs[0]["window_id"].as_str().unwrap(),
                    docs[0]["document_id"].as_str().unwrap(),
                )
                .unwrap();
            let result = backend.insert_svg(r#"<rect id="new"/>"#);
            if mode == "effect-lost-refused" {
                assert_eq!(
                    result,
                    Err(Error::Context(
                        "Inkscape refused insertion; document context changed"
                    ))
                );
            } else {
                assert_eq!(result, Err(Error::Uncertain), "{mode}");
            }
            assert_eq!(
                trace(&log)
                    .iter()
                    .filter(|a| a
                        .get(7)
                        .is_some_and(|m| m == "org.inkscape.MCP.Context1.Activate")
                        && a.get(10)
                            .is_some_and(|m| m == "org.inkscape-mcp.insert.noprefs"))
                    .count(),
                1
            );
            assert!(!root.path().join("insert-request.json").exists());
            assert!(!root.path().join("insert-result.json").exists());
            assert!(backend.lock.is_none());
        }
        let (root, dbus, log) = fixture_for_effect("effect-success");
        let mut backend = Managed::new(
            dbus,
            root.path().canonicalize().unwrap().join("stdout.log"),
            true,
        );
        backend.effect_directory(root.path().canonicalize().unwrap());
        let before = trace(&log);
        assert_eq!(
            backend.insert_svg("<script/>"),
            Err(Error::Context(
                "unsupported or unsafe SVG insertion fragment"
            ))
        );
        assert_eq!(trace(&log), before);
        assert_eq!(
            backend.insert_svg("<rect/>"),
            Err(Error::Context(
                "choose the task drawing with live_select_document before editing"
            ))
        );
        assert!(!root.path().join("insert-request.json").exists());
    }
    #[test]
    fn frozen_edit_transactions_match_requests_results_and_refusals() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/edit-transaction-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let spec = &case["spec"];
            let mode = if spec["refusal"] == "invalid selection" {
                "effect-refused"
            } else if spec["refusal"].is_string() {
                "effect-unknown-refusal"
            } else {
                "effect-success"
            };
            let (root, dbus, log) = fixture_for_effect(mode);
            let mut backend = Managed::new(
                dbus,
                root.path().canonicalize().unwrap().join("stdout.log"),
                true,
            );
            backend.effect_directory(root.path().canonicalize().unwrap());
            let docs = backend.list_documents().unwrap();
            backend
                .select_document(
                    docs[0]["window_id"].as_str().unwrap(),
                    docs[0]["document_id"].as_str().unwrap(),
                )
                .unwrap();
            let result =
                backend.edit_selection(spec["operation"].as_str().unwrap(), spec["params"].clone());
            let actual = match result {
                Ok(value) => json!({"result":value}),
                Err(error) => json!({"error":error.public_message()}),
            };
            assert_eq!(actual, case["expected"], "{spec}");
            let mut request: Value = serde_json::from_slice(
                &std::fs::read(root.path().join("captured-request.json")).unwrap(),
            )
            .unwrap();
            let nonce = request["nonce"]
                .as_str()
                .unwrap()
                .strip_prefix("mcp_")
                .unwrap();
            assert_eq!(nonce.len(), 32);
            assert!(
                nonce
                    .bytes()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            );
            request["nonce"] = case["request"]["nonce"].clone();
            assert_eq!(request, case["request"]);
            let calls = trace(&log);
            let activations: Vec<_> = calls
                .iter()
                .filter(|a| {
                    a.get(10)
                        .is_some_and(|s| s == "org.inkscape-mcp.edit.noprefs")
                })
                .collect();
            assert_eq!(activations.len(), 1);
            assert_eq!(activations[0][7], "org.inkscape.MCP.Context1.Activate");
            for name in [
                "insert-request.json",
                "insert-result.json",
                "insert-request.tmp",
            ] {
                assert!(!root.path().join(name).exists());
            }
            assert_eq!(backend.depth, 0);
            assert!(backend.lock.is_none());
        }
    }
    fn fixture_for_effect(mode: &str) -> (tempfile::TempDir, Dbus, PathBuf) {
        let (root, dbus, log) = fixture(mode, 1024 * 1024);
        std::fs::write(root.path().join("stdout.log"), b"").unwrap();
        (root, dbus, log)
    }
    #[test]
    fn changed_effect_result_is_confirmed_against_independent_helper_fingerprint() {
        let (root, dbus, log) = fixture_for_effect("effect-mutated");
        let mut backend = Managed::new(
            dbus,
            root.path().canonicalize().unwrap().join("stdout.log"),
            false,
        );
        backend.effect_directory(root.path().canonicalize().unwrap());
        let value = backend
            .apply_selection(
                &std::collections::BTreeMap::from([("fill".into(), "blue".into())]),
                None,
            )
            .unwrap();
        assert_eq!(value["affected_ids"], json!(["r"]));
        assert_eq!(value["count"], 1);
        assert!(
            std::fs::read_to_string(root.path().join("drawing.svg"))
                .unwrap()
                .contains("fill=\"blue\"")
        );
        let calls = trace(&log);
        assert_eq!(
            calls
                .iter()
                .filter(
                    |a| a.get(7).is_some_and(|m| m == "org.gtk.Actions.Activate")
                        && a.get(8)
                            .is_some_and(|s| s == "org.inkscape-mcp.edit.noprefs")
                )
                .count(),
            1
        );
        assert_eq!(
            calls
                .iter()
                .filter(|a| a.get(8).is_some_and(|s| s == "export-do"))
                .count(),
            2
        );
    }
    #[test]
    fn effect_loss_stale_missing_mismatch_and_context_switch_stay_uncertain_without_retry() {
        for mode in [
            "effect-lost",
            "effect-stale",
            "effect-missing",
            "effect-mismatch",
            "effect-switch",
            "effect-lost-refused",
        ] {
            let (root, dbus, log) = fixture_for_effect(mode);
            let mut backend = Managed::new(
                dbus,
                root.path().canonicalize().unwrap().join("stdout.log"),
                mode == "effect-switch",
            );
            backend.effect_directory(root.path().canonicalize().unwrap());
            if mode == "effect-stale" {
                // A read-only preflight may exceed the former 300 ms shortcut under
                // CI scheduling pressure, before any effect has been dispatched.
                std::fs::write(
                    root.path().join("probe-control.json"),
                    serde_json::to_vec(&json!({"delay_action":"query-x","delay_ms":600})).unwrap(),
                )
                .unwrap();
            }
            if mode == "effect-switch" {
                let docs = backend.list_documents().unwrap();
                backend
                    .select_document(
                        docs[0]["window_id"].as_str().unwrap(),
                        docs[0]["document_id"].as_str().unwrap(),
                    )
                    .unwrap();
            }
            // This matrix tests replies after dispatch, not subprocess startup speed.
            // Keep preflight bounded but allow CI scheduling; shortening this shared
            // timeout can fail a read-only call before the effect exists. Missing and
            // mismatched replies still exercise the real bounded completion deadline.
            backend.dbus.bus.timeout = Duration::from_secs(5);
            let result = backend.set_text("literal");
            if mode == "effect-lost-refused" {
                assert_eq!(
                    result,
                    Err(Error::EditRefused(
                        "Inkscape refused edit: invalid selection".into()
                    ))
                );
            } else {
                assert_eq!(result, Err(Error::Uncertain), "{mode}");
            }
            let calls = trace(&log);
            assert_eq!(
                calls
                    .iter()
                    .filter(|a| a.iter().any(|s| s == "query-x"))
                    .count(),
                1,
                "{mode}: selection preflight completed without retry"
            );
            assert_eq!(
                calls
                    .iter()
                    .filter(|a| a.get(7).is_some_and(|m| m.ends_with(".Activate"))
                        && a.iter().any(|s| s == "org.inkscape-mcp.edit.noprefs"))
                    .count(),
                1
            );
            assert_eq!(backend.depth, 0);
            assert!(backend.lock.is_none());
            assert!(!root.path().join("insert-request.json").exists());
            assert!(!root.path().join("insert-result.json").exists());
        }
    }
    #[test]
    fn unselected_and_absent_helper_refuse_before_request_or_mutation_and_legacy_fill_remains() {
        let (root, dbus, log) = fixture_for_effect("effect-success");
        let mut backend = Managed::new(
            dbus,
            root.path().canonicalize().unwrap().join("stdout.log"),
            true,
        );
        backend.effect_directory(root.path().canonicalize().unwrap());
        assert!(matches!(
            backend.set_text("literal"),
            Err(Error::Context(_))
        ));
        assert!(!root.path().join("insert-request.json").exists());
        assert!(
            !trace(&log)
                .iter()
                .any(|a| a.get(7).is_some_and(|m| m.ends_with(".Activate"))
                    && a.iter().any(|v| v == "org.inkscape-mcp.edit.noprefs"))
        );
        let (legacy_root, dbus, legacy_log) = fixture_for_effect("effect-absent");
        let mut legacy = Managed::new(
            dbus,
            legacy_root
                .path()
                .canonicalize()
                .unwrap()
                .join("stdout.log"),
            false,
        );
        legacy.effect_directory(legacy_root.path().canonicalize().unwrap());
        assert!(matches!(
            legacy.set_text("literal"),
            Err(Error::Unsupported(_))
        ));
        let value = legacy
            .apply_selection(
                &std::collections::BTreeMap::from([("fill".into(), "red".into())]),
                None,
            )
            .unwrap();
        assert_eq!(value["affected_ids"], json!(["r"]));
        assert_eq!(value["count"], 1);
        assert_eq!(
            value["detail"],
            "changed fill on the current selection; use Inkscape Undo to revert"
        );
        assert!(
            !trace(&legacy_log)
                .iter()
                .any(|a| a.get(7).is_some_and(|m| m.ends_with(".Activate"))
                    && a.iter().any(|v| v == "org.inkscape-mcp.edit.noprefs"))
        );
    }
    #[test]
    fn scene_and_inspection_share_one_context_for_selection_and_document_export() {
        let (_root, mut backend, log) = managed(true);
        let scene = backend.scene().unwrap();
        assert_eq!(scene["selection_count"], 2);
        assert_eq!(scene["selection"][0]["id"], "r");
        assert_eq!(scene["selection"][1]["id"], "Привіт");
        assert_eq!(
            scene["active_document"]["document_id"],
            "abcdefab-1234-1234-1234-123456789abc"
        );
        assert_eq!(scene["active_document"]["object_count"], 2);
        assert_eq!(scene["visible_objects"][0]["id"], "g");
        assert_eq!(scene["visible_objects"][1]["id"], "r");
        assert_eq!(scene["viewport"]["zoom"], Value::Null);
        let inspection = backend.inspect_selection().unwrap();
        assert_eq!(inspection["count"], 1);
        assert_eq!(inspection["objects"][0]["id"], "r");
        let calls = trace(&log);
        assert_eq!(
            calls
                .iter()
                .filter(|a| a
                    .get(7)
                    .is_some_and(|m| m == "org.inkscape.MCP.Context1.GetContext"))
                .count(),
            2
        );
        let actions: Vec<_> = calls
            .iter()
            .filter(|a| {
                a.get(7)
                    .is_some_and(|m| m == "org.inkscape.MCP.Context1.Activate")
            })
            .collect();
        assert_eq!(actions.len(), 18);
        for start in [0, 9] {
            assert_eq!(actions[start][10], "select-list");
            assert_eq!(actions[start + 1][10], "query-x");
        }
        assert!(backend.lock.is_none());
        assert_eq!(backend.depth, 0);
    }
    #[test]
    fn scoped_selection_fence_deduplicates_unicode_and_filters_the_exported_root_id() {
        for (mode, expected) in [
            ("normal", json!(["r", "Привіт"])),
            ("selection-empty", json!([])),
        ] {
            let (root, dbus, log) = fixture(mode, 1024 * 1024);
            let stream = root.path().canonicalize().unwrap().join("stdout.log");
            std::fs::write(&stream, b"old unrelated output\n").unwrap();
            let mut backend = Managed::new(dbus, stream, true);
            let value = backend.selection().unwrap();
            assert_eq!(value["object_ids"], expected);
            assert_eq!(value["count"], expected.as_array().unwrap().len());
            let calls = trace(&log);
            assert_eq!(calls.len(), 5);
            assert_eq!(calls[3][10], "select-list");
            assert_eq!(calls[4][10], "query-x");
            backend.root_id = Some("r".into());
            let again = backend.selection().unwrap();
            if mode == "normal" {
                assert_eq!(again["object_ids"], json!(["Привіт"]));
            }
            assert_eq!(backend.depth, 0);
        }
    }
    #[test]
    fn incomplete_invalid_and_oversize_stdout_refuse_without_an_action_retry() {
        for mode in [
            "selection-incomplete",
            "selection-invalid",
            "selection-overflow",
            "selection-utf8",
        ] {
            let (root, dbus, log) = fixture(mode, 1024 * 1024);
            let stream = root.path().canonicalize().unwrap().join("stdout.log");
            std::fs::write(&stream, b"").unwrap();
            let mut backend = Managed::new(dbus, stream, false);
            backend.dbus.bus.timeout = Duration::from_millis(250);
            assert!(backend.selection().is_err(), "{mode}");
            assert_eq!(trace(&log).len(), 4);
            assert!(backend.lock.is_none());
            assert_eq!(backend.depth, 0);
        }
    }
    #[test]
    fn managed_export_resets_sticky_options_retains_metadata_and_maps_user_unit_regions() {
        let (root, dbus, log) = fixture("mapped", 1024 * 1024);
        let stream = root.path().canonicalize().unwrap().join("stdout.log");
        std::fs::write(&stream, b"").unwrap();
        let mut backend = Managed::new(dbus, stream, true);
        let bytes = backend
            .render_view(Some([10.0, 20.0, 5.0, 6.0]), None)
            .unwrap();
        assert_eq!(bytes, b"synthetic transport bytes, not pixel acceptance");
        assert_eq!(backend.root_id.as_deref(), Some("svgroot"));
        let calls = trace(&log);
        let actions: Vec<_> = calls
            .iter()
            .filter(|argv| {
                argv.get(7)
                    .is_some_and(|s| s == "org.inkscape.MCP.Context1.Activate")
            })
            .collect();
        assert_eq!(actions.len(), 15);
        for offset in [0, 7] {
            assert_eq!(actions[offset][10], "export-id");
            assert_eq!(actions[offset][11], "[<''>]");
            assert_eq!(actions[offset + 1][10], "export-id-only");
            assert_eq!(actions[offset + 1][11], "[<false>]");
            assert_eq!(actions[offset + 2][10], "export-text-to-path");
            assert_eq!(actions[offset + 3][10], "export-plain-svg");
            assert_eq!(actions[offset + 3][11], "[<false>]");
        }
        assert_eq!(actions[13][10], "export-area");
        assert_eq!(actions[13][11], "[<'0.0:0.0:20.0:24.0'>]");
        assert_eq!(
            calls
                .iter()
                .filter(|argv| argv
                    .get(7)
                    .is_some_and(|s| s == "org.inkscape.MCP.Context1.GetContext"))
                .count(),
            1
        );
    }
    #[test]
    fn nested_clients_share_context_and_lock_only_for_the_same_managed_session() {
        let (_root, mut first, _) = managed(true);
        let (_second_root, mut second, second_log) = managed(true);
        let (_other_root, mut other, other_log) = managed(true);
        second.stream = first.stream.clone();
        first.begin_operation().unwrap();
        second.begin_operation().unwrap();
        assert_eq!(first.current, second.current);
        assert_eq!(trace(&second_log).len(), 2);
        assert_eq!(
            other.begin_operation(),
            Err(Error::Protocol(
                "cannot nest operations from different managed sessions"
            ))
        );
        assert_eq!(trace(&other_log).len(), 2);
        second.end_operation();
        assert!(first.lock.is_some());
        first.end_operation();
        assert!(
            !OPERATIONS
                .lock()
                .unwrap()
                .contains_key(&std::thread::current().id())
        );
        first.guarded = false;
        first.begin_operation().unwrap();
        assert_eq!(
            second.begin_operation(),
            Err(Error::Protocol(
                "context action requires a managed operation"
            ))
        );
        first.end_operation();
    }
    #[test]
    fn external_client_lock_times_out_before_context_io_and_links_are_refused() {
        let (root, mut backend, log) = managed(true);
        let lock = backend.stream.with_extension("lock");
        let mut child = Process::new(crate::native_test_fixture::binary())
            .arg("hold-lock")
            .arg(&lock)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut ready = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut ready)
            .unwrap();
        assert_eq!(ready, "locked\n");
        backend.dbus.bus.timeout = Duration::from_millis(60);
        let result = backend.begin_operation();
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(result, Err(Error::Connection("managed Inkscape is busy")));
        assert_eq!(trace(&log).len(), 2);
        assert!(backend.lock.is_none());
        std::fs::remove_file(&lock).unwrap();
        std::os::unix::fs::symlink(&backend.stream, &lock).unwrap();
        assert!(backend.begin_operation().is_err());
        assert_eq!(trace(&log).len(), 2);
        std::fs::remove_file(&lock).unwrap();
        std::fs::remove_file(&backend.stream).unwrap();
        std::os::unix::fs::symlink(root.path().join("gdbus"), &backend.stream).unwrap();
        assert!(backend.begin_operation().is_err());
        assert!(!lock.exists());
    }
}
#[derive(Clone, Copy)]
pub enum SelectionEdit {
    Duplicate,
    Delete,
    Group,
    Ungroup,
    Raise,
    Lower,
    Front,
    Back,
}
impl SelectionEdit {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "duplicate" => Self::Duplicate,
            "delete" => Self::Delete,
            "group" => Self::Group,
            "ungroup" => Self::Ungroup,
            "raise" => Self::Raise,
            "lower" => Self::Lower,
            "front" => Self::Front,
            "back" => Self::Back,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Delete => "delete",
            Self::Group => "group",
            Self::Ungroup => "ungroup",
            Self::Raise => "raise",
            Self::Lower => "lower",
            Self::Front => "front",
            Self::Back => "back",
        }
    }
}
