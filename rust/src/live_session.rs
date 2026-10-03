//! Live lifecycle owner. Connection factories attach transports; they never launch a GUI.
use crate::{
    live_cache::Cache,
    live_socket::Error,
    live_transport::{self, Preference, Probe, Transport},
};
use serde_json::{Value, json};
pub struct Settings {
    pub enabled: bool,
    pub cache_entries: usize,
    pub cache_bytes: usize,
    pub coalesce_ms: f64,
}
pub struct Session {
    pub settings: Settings,
    transport: Option<Box<dyn Transport>>,
    pub connected_at: Option<String>,
    pub active_document: Value,
    pub last_token: Option<Value>,
    pub last_change: Option<Value>,
    pub cache: Option<Cache>,
}
impl Session {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            transport: None,
            connected_at: None,
            active_document: Value::Null,
            last_token: None,
            last_change: None,
            cache: None,
        }
    }
    pub fn teardown(&mut self, clear_history: impl FnOnce()) {
        if let Some(mut transport) = self.transport.take() {
            transport.disconnect();
        }
        self.connected_at = None;
        self.active_document = Value::Null;
        self.last_token = None;
        self.last_change = None;
        if let Some(cache) = self.cache.as_mut() {
            cache.clear();
        }
        self.cache = None;
        clear_history();
    }
    pub fn connect(
        &mut self,
        prefer: Preference,
        probes: &[Probe],
        factory: impl FnOnce(&Probe) -> Result<Box<dyn Transport>, Error>,
        clear_history: impl FnOnce(),
    ) -> Result<(), Error> {
        if !self.settings.enabled {
            return Err(Error::Disabled);
        }
        self.teardown(clear_history);
        let chosen = live_transport::best(probes, prefer).ok_or(Error::NotAvailable)?;
        let mut transport = factory(&chosen)?;
        let result = transport
            .connect()
            .and_then(|()| transport.active_document());
        match result {
            Ok(document) => self.active_document = document,
            Err(error) => {
                transport.disconnect();
                return Err(error);
            }
        }
        self.connected_at =
            Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true));
        self.cache = Some(Cache::new(
            self.settings.cache_entries,
            self.settings.cache_bytes,
            self.settings.coalesce_ms,
        ));
        self.transport = Some(transport);
        Ok(())
    }
    pub fn require_transport(&mut self) -> Result<&mut (dyn Transport + '_), Error> {
        if let Some(transport) = self.transport.as_mut()
            && transport.is_connected()
        {
            return Ok(transport.as_mut());
        }
        Err(Error::NotAvailable)
    }
    pub fn record_change(&mut self, token: Value, change: Value) {
        self.last_token = Some(token);
        self.last_change = Some(change);
    }
    pub fn operation<T>(
        &mut self,
        op: impl FnOnce(&mut dyn Transport) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let transport = self.require_transport()?;
        transport.begin_operation()?;
        struct Scope<'a>(&'a mut dyn Transport);
        impl Drop for Scope<'_> {
            fn drop(&mut self) {
                self.0.end_operation();
            }
        }
        let scope = Scope(transport);
        let result = op(scope.0);
        if matches!(result, Err(Error::Uncertain)) {
            crate::telemetry::failure(crate::telemetry::Failure::LiveUncertain);
        }
        result
    }
    pub fn status(&mut self, probes: &[Probe], helper_installed: bool) -> Value {
        let mut available: Vec<_> = live_transport::ranked(probes)
            .into_iter()
            .filter(|p| p.available)
            .map(|p| p.name)
            .collect();
        if helper_installed && !available.iter().any(|n| n == "extension-socket") {
            available.push("extension-socket".into());
        }
        let connected = self.transport.as_ref().is_some_and(|t| t.is_connected());
        let mut notes: Vec<&str> = Vec::new();
        if !self.settings.enabled {
            notes.push("live mode is disabled by configuration (set INKSCAPE_MCP_LIVE_ENABLED)");
        }
        if available.is_empty() {
            notes.push("no live transport available on this host");
        }
        let mut document = if connected {
            self.active_document.clone()
        } else {
            Value::Null
        };
        let name = self.transport.as_ref().map(|t| t.name().to_owned());
        if connected && name.as_deref() == Some("managed-dbus") {
            match self.transport.as_mut().unwrap().active_document() {
                Ok(current) => document = current,
                Err(_) => {
                    document = Value::Null;
                    notes.push("active document unavailable; activate a drawing and retry");
                }
            }
        } else if self.transport.is_some() && !connected {
            notes.push("connection unavailable; call live_connect to reconnect");
        }
        let selected = if connected {
            self.transport
                .as_ref()
                .and_then(|t| t.selected_document())
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        let guard = self.transport.as_ref().is_some_and(|t| t.guard_available());
        let ready = connected
            && guard
            && document["window_id"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
            && document["document_id"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
            && !selected.is_null()
            && document["window_id"] == selected["window_id"]
            && document["document_id"] == selected["document_id"];
        let mut recovery: Vec<&str> = Vec::new();
        let state = if !self.settings.enabled {
            "disabled"
        } else if self.transport.is_none() {
            recovery.push("Call live_connect to attach; it never closes the GUI.");
            "disconnected"
        } else if !connected {
            recovery.extend([
                "Reconnect with live_connect, then select the task drawing again.",
                "If the private bus failed, save work before restarting the GUI session.",
                "Inspect the drawing after an edit timeout: it may have applied.",
            ]);
            "connection_lost"
        } else if document.is_null() {
            recovery.push("Activate a drawing, dismiss any dialog, and retry live_status.");
            "document_unavailable"
        } else {
            if guard && !ready {
                recovery.push("Use live_list_documents and live_select_document before editing.");
            } else if name.as_deref() == Some("managed-dbus") && !guard {
                notes.push("legacy session: window identity and task document guard unavailable");
                recovery.push("Save work before restarting the managed GUI to load the bridge.");
            }
            "connected"
        };
        json!({"enabled":self.settings.enabled,"connected":connected,"transport":if connected{name}else{None},"available_transports":available,"active_document":document,"connected_at":if connected{self.connected_at.clone()}else{None},"selected_document":selected,"document_guard_available":guard,"ready_to_edit":ready,"connection_state":state,"recovery_actions":recovery,"notes":notes})
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if let Some(transport) = self.transport.as_mut() {
            transport.disconnect();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_protocol::Command;
    use std::sync::{Arc, Mutex};
    struct Fake {
        name: String,
        connected: bool,
        guard: bool,
        document: Value,
        selected: Option<Value>,
        fail_connect: bool,
        calls: Arc<Mutex<Vec<&'static str>>>,
    }
    impl Transport for Fake {
        fn name(&self) -> &str {
            &self.name
        }
        fn supports(&self, _: Command) -> bool {
            true
        }
        fn connect(&mut self) -> Result<(), Error> {
            self.calls.lock().unwrap().push("connect");
            if self.fail_connect {
                return Err(Error::Connection("test failure"));
            }
            self.connected = true;
            self.selected = None;
            Ok(())
        }
        fn disconnect(&mut self) {
            self.calls.lock().unwrap().push("disconnect");
            self.connected = false;
            self.selected = None;
        }
        fn is_connected(&self) -> bool {
            self.connected
        }
        fn active_document(&mut self) -> Result<Value, Error> {
            self.calls.lock().unwrap().push("document");
            if self.document.is_null() {
                Err(Error::Connection("document unavailable"))
            } else {
                Ok(self.document.clone())
            }
        }
        fn guard_available(&self) -> bool {
            self.guard
        }
        fn selected_document(&self) -> Option<Value> {
            self.selected.clone()
        }
        fn begin_operation(&mut self) -> Result<(), Error> {
            self.calls.lock().unwrap().push("begin");
            Ok(())
        }
        fn end_operation(&mut self) {
            self.calls.lock().unwrap().push("end");
        }
    }
    fn settings(enabled: bool) -> Settings {
        Settings {
            enabled,
            cache_entries: 2,
            cache_bytes: 1024 * 1024,
            coalesce_ms: 100.,
        }
    }
    fn probe() -> Probe {
        Probe {
            name: "managed-dbus".into(),
            available: true,
            rank: 30,
            commands: vec![
                Command::ActiveDocument,
                Command::Selection,
                Command::InspectSelection,
            ],
            no_freeze: true,
            detail: String::new(),
        }
    }
    fn fake(
        document: Value,
        fail_connect: bool,
        calls: Arc<Mutex<Vec<&'static str>>>,
    ) -> Box<dyn Transport> {
        Box::new(Fake {
            name: "managed-dbus".into(),
            connected: false,
            guard: true,
            document,
            selected: None,
            fail_connect,
            calls,
        })
    }
    #[test]
    fn compiled_reference_status_recovery_and_helper_reconciliation_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/session-state-cases.json"
        ))
        .unwrap();
        for case in fixture["status"].as_array().unwrap() {
            let input = &case["input"];
            let mut session = Session::new(settings(input["enabled"].as_bool().unwrap()));
            session.active_document = input["document"].clone();
            session.connected_at = Some("fixed-connect-time".into());
            if let Some(name) = input["name"].as_str() {
                session.transport = Some(Box::new(Fake {
                    name: name.into(),
                    connected: input["connected"].as_bool().unwrap(),
                    guard: input["guard"].as_bool().unwrap(),
                    document: input["document"].clone(),
                    selected: if input["selected"].is_null() {
                        None
                    } else {
                        Some(input["selected"].clone())
                    },
                    fail_connect: false,
                    calls: Arc::default(),
                }));
            }
            let probes = [Probe {
                name: "dbus".into(),
                available: input["available"].as_bool().unwrap(),
                rank: 10,
                commands: vec![],
                no_freeze: true,
                detail: String::new(),
            }];
            assert_eq!(
                session.status(&probes, input["installed"].as_bool().unwrap()),
                case["expected"],
                "{input}"
            );
        }
    }
    #[test]
    fn reconnect_and_failed_attach_clear_session_state_and_require_transport() {
        let mut session = Session::new(settings(true));
        let calls: Arc<Mutex<Vec<&'static str>>> = Arc::default();
        let clear = std::cell::Cell::new(0);
        let document = json!({"window_id":"w","document_id":"d"});
        session
            .connect(
                Preference::Read,
                &[probe()],
                |_| Ok(fake(document.clone(), false, calls.clone())),
                || clear.set(clear.get() + 1),
            )
            .unwrap();
        assert!(session.require_transport().is_ok());
        session.last_token = Some(json!({"old":true}));
        session.last_change = Some(json!({"old":true}));
        session.cache.as_mut().unwrap().put(
            crate::live_cache::Key {
                revision: "old".into(),
                viewport: "full".into(),
                scale: "native".into(),
            },
            json!({"frame":"old"}),
            10,
            0.,
        );
        assert!(
            session
                .connect(
                    Preference::NoFreeze,
                    &[probe()],
                    |_| Ok(fake(Value::Null, false, calls.clone())),
                    || clear.set(clear.get() + 1)
                )
                .is_err()
        );
        assert!(matches!(
            session.require_transport(),
            Err(Error::NotAvailable)
        ));
        assert!(
            session.connected_at.is_none()
                && session.active_document.is_null()
                && session.last_token.is_none()
                && session.last_change.is_none()
                && session.cache.is_none()
        );
        assert_eq!(clear.get(), 2);
        assert_eq!(
            *calls.lock().unwrap(),
            [
                "connect",
                "document",
                "disconnect",
                "connect",
                "document",
                "disconnect"
            ]
        );
        session.teardown(|| clear.set(clear.get() + 1));
        session.teardown(|| clear.set(clear.get() + 1));
        assert_eq!(clear.get(), 4);
        session.settings.enabled = false;
        assert!(matches!(
            session.connect(
                Preference::Read,
                &[probe()],
                |_| panic!("disabled factory invoked"),
                || panic!("disabled session cleared")
            ),
            Err(Error::Disabled)
        ));
        session.settings.enabled = true;
        assert!(matches!(
            session.connect(
                Preference::Read,
                &[],
                |_| panic!("missing backend instantiated"),
                || ()
            ),
            Err(Error::NotAvailable)
        ));
    }
    #[test]
    fn operation_scope_releases_context_on_error_and_panic() {
        let mut session = Session::new(settings(true));
        let calls: Arc<Mutex<Vec<&'static str>>> = Arc::default();
        session
            .connect(
                Preference::Read,
                &[probe()],
                |_| {
                    Ok(fake(
                        json!({"window_id":"w","document_id":"d"}),
                        false,
                        calls.clone(),
                    ))
                },
                || (),
            )
            .unwrap();
        assert_eq!(
            session.operation::<()>(|_| Err(Error::Uncertain)),
            Err(Error::Uncertain)
        );
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            session.operation::<()>(|_| panic!("synthetic operation failure"))
        }));
        assert!(outcome.is_err());
        assert_eq!(
            *calls.lock().unwrap(),
            ["connect", "document", "begin", "end", "begin", "end"]
        );
    }
}
