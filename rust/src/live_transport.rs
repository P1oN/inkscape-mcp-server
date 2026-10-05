//! Fixed semantic interface shared by socket, D-Bus and managed document backends.
use crate::{
    live_protocol::Command,
    live_socket::{Client, Error, Rendezvous, Viewport},
};
use serde_json::Value;
use std::{collections::BTreeMap, time::Duration};
#[derive(Clone)]
pub struct Probe {
    pub name: String,
    pub available: bool,
    pub rank: i32,
    pub commands: Vec<Command>,
    pub no_freeze: bool,
    pub detail: String,
}
impl Probe {
    pub fn value(&self) -> Value {
        let mut commands: Vec<_> = self.commands.iter().map(|c| c.name()).collect();
        commands.sort();
        serde_json::json!({"name":self.name,"available":self.available,"rank":self.rank,"supported_commands":commands,"no_freeze":self.no_freeze,"detail":self.detail})
    }
}
#[derive(Clone, Copy)]
pub enum Preference {
    Read,
    NoFreeze,
}
impl Preference {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "read" => Ok(Self::Read),
            "no_freeze" => Ok(Self::NoFreeze),
            _ => Err("prefer must be 'read' or 'no_freeze'"),
        }
    }
}
pub fn ranked(probes: &[Probe]) -> Vec<Probe> {
    let mut probes = probes.to_vec();
    probes.sort_by_key(|p| std::cmp::Reverse((p.available, p.rank)));
    probes
}
pub fn best(probes: &[Probe], prefer: Preference) -> Option<Probe> {
    ranked(probes).into_iter().find(|p| {
        p.available
            && match prefer {
                Preference::Read => [
                    Command::ActiveDocument,
                    Command::Selection,
                    Command::InspectSelection,
                ]
                .iter()
                .all(|c| p.commands.contains(c)),
                Preference::NoFreeze => {
                    p.no_freeze && p.commands.contains(&Command::ActiveDocument)
                }
            }
    })
}
pub trait Transport: Send {
    fn name(&self) -> &str;
    fn supports(&self, command: Command) -> bool;
    fn connect(&mut self) -> Result<(), Error>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn active_document(&mut self) -> Result<Value, Error>;
    fn selection(&mut self) -> Result<Value, Error> {
        Err(Error::Unsupported("selection unavailable"))
    }
    fn inspect_selection(&mut self) -> Result<Value, Error> {
        Err(Error::Unsupported("inspection unavailable"))
    }
    fn document_svg(&mut self) -> Result<String, Error> {
        Err(Error::Unsupported("document SVG unavailable"))
    }
    fn render_view(
        &mut self,
        _region: Option<[f64; 4]>,
        _scale: Option<f64>,
    ) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported("render unavailable"))
    }
    fn set_viewport(&mut self, _viewport: &Viewport) -> Result<Value, Error> {
        Err(Error::Unsupported("viewport unavailable"))
    }
    fn scene(&mut self) -> Result<Value, Error> {
        Err(Error::Unsupported("scene unavailable"))
    }
    fn state_token(&mut self) -> Result<(Value, Vec<String>), Error> {
        Err(Error::Unsupported("state token unavailable"))
    }
    fn apply_selection(
        &mut self,
        _style: &BTreeMap<String, String>,
        _transform: Option<&str>,
    ) -> Result<Value, Error> {
        Err(Error::Unsupported("selection edit unavailable"))
    }
    fn insert_svg(&mut self, _fragment: &str) -> Result<Value, Error> {
        Err(Error::Unsupported("SVG insertion unavailable"))
    }
    fn set_text(&mut self, _text: &str) -> Result<Value, Error> {
        Err(Error::Unsupported("text edit unavailable"))
    }
    fn order_selection(
        &mut self,
        _operation: crate::live_managed::SelectionEdit,
    ) -> Result<Value, Error> {
        Err(Error::Unsupported(
            "structural edits require managed macOS Inkscape",
        ))
    }
    fn package_available(&mut self) -> Result<bool, Error> {
        Ok(false)
    }
    fn change_package(&mut self, _params: &Value) -> Result<Value, Error> {
        Err(Error::Unsupported(
            "reviewed packages require the guarded managed native helper",
        ))
    }
    fn export_selection(&mut self) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported("selection export unavailable"))
    }
    fn guard_available(&self) -> bool {
        false
    }
    fn selected_document(&self) -> Option<Value> {
        None
    }
    fn list_documents(&mut self) -> Result<Value, Error> {
        Err(Error::Unsupported("managed document list unavailable"))
    }
    fn select_document(&mut self, _window: &str, _document: &str) -> Result<Value, Error> {
        Err(Error::Unsupported("managed document choice unavailable"))
    }
    /// Managed backends pin/validate document context across a public operation.
    fn begin_operation(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_operation(&mut self) {}
}
pub struct Socket {
    pub rendezvous: Rendezvous,
    pub timeout: Duration,
    pub max_output: usize,
    client: Option<Client>,
}
impl Socket {
    pub fn new(rendezvous: Rendezvous, timeout: Duration, max_output: usize) -> Self {
        Self {
            rendezvous,
            timeout,
            max_output,
            client: None,
        }
    }
    fn client(&mut self) -> Result<&mut Client, Error> {
        self.client
            .as_mut()
            .filter(|c| c.is_connected())
            .ok_or(Error::NotAvailable)
    }
}
impl Transport for Socket {
    fn name(&self) -> &str {
        "extension-socket"
    }
    fn supports(&self, c: Command) -> bool {
        c != Command::Hello
    }
    fn connect(&mut self) -> Result<(), Error> {
        self.disconnect();
        self.client = Some(Client::connect(
            &self.rendezvous,
            self.timeout,
            self.max_output,
        )?);
        Ok(())
    }
    fn disconnect(&mut self) {
        if let Some(mut client) = self.client.take() {
            client.disconnect();
        }
    }
    fn is_connected(&self) -> bool {
        self.client.as_ref().is_some_and(Client::is_connected)
    }
    fn active_document(&mut self) -> Result<Value, Error> {
        self.client()?.active_document()
    }
    fn selection(&mut self) -> Result<Value, Error> {
        self.client()?.selection()
    }
    fn inspect_selection(&mut self) -> Result<Value, Error> {
        self.client()?.inspect_selection()
    }
    fn document_svg(&mut self) -> Result<String, Error> {
        self.client()?.document_svg()
    }
    fn render_view(&mut self, r: Option<[f64; 4]>, s: Option<f64>) -> Result<Vec<u8>, Error> {
        self.client()?.render_view(r, s)
    }
    fn set_viewport(&mut self, v: &Viewport) -> Result<Value, Error> {
        self.client()?.set_viewport(v)
    }
    fn scene(&mut self) -> Result<Value, Error> {
        self.client()?.scene()
    }
    fn state_token(&mut self) -> Result<(Value, Vec<String>), Error> {
        self.client()?.state_token()
    }
    fn apply_selection(
        &mut self,
        s: &BTreeMap<String, String>,
        t: Option<&str>,
    ) -> Result<Value, Error> {
        self.client()?.apply_selection(s, t)
    }
    fn insert_svg(&mut self, f: &str) -> Result<Value, Error> {
        self.client()?.insert_svg(f)
    }
    fn set_text(&mut self, t: &str) -> Result<Value, Error> {
        self.client()?.set_text(t)
    }
    fn export_selection(&mut self) -> Result<Vec<u8>, Error> {
        self.client()?.export_selection()
    }
}
impl Drop for Socket {
    fn drop(&mut self) {
        self.disconnect();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranking_requires_capability_and_no_freeze_not_platform_assumption() {
        let p =
            |name: &str, rank: i32, no_freeze: bool, commands: Vec<Command>, available: bool| {
                Probe {
                    name: name.into(),
                    rank,
                    no_freeze,
                    commands,
                    available,
                    detail: String::new(),
                }
            };
        let probes = vec![
            p(
                "socket",
                20,
                false,
                vec![
                    Command::ActiveDocument,
                    Command::Selection,
                    Command::InspectSelection,
                ],
                true,
            ),
            p("dbus", 10, true, vec![Command::ActiveDocument], true),
            p(
                "managed",
                30,
                true,
                vec![Command::ActiveDocument, Command::Selection],
                true,
            ),
            p("missing", 100, true, Command::ALL.to_vec(), false),
        ];
        assert_eq!(best(&probes, Preference::Read).unwrap().name, "socket");
        assert_eq!(best(&probes, Preference::NoFreeze).unwrap().name, "managed");
        assert_eq!(ranked(&probes).last().unwrap().name, "missing");
        assert!(best(&probes[..1], Preference::NoFreeze).is_none());
        assert!(Preference::parse("unknown").is_err());
    }
}
