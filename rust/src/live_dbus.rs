//! Native no-freeze GAction backend. Stateful export options are an explicit side effect.
use crate::{
    live_bus::{Action, Bus, Context, Identity, Parameter, Target},
    live_protocol::Command,
    live_socket::{Error, Viewport},
    live_transport::Transport,
    workspace::Workspace,
};
use libxml::tree::NodeType;
use regex::Regex;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, sync::LazyLock};
static WINDOW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/org/inkscape/Inkscape/window/([0-9]+)\b").unwrap());
static TRANSFORM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:translate\((-?[0-9]+(?:\.[0-9]+)?),(-?[0-9]+(?:\.[0-9]+)?)\)|rotate\((-?[0-9]+(?:\.[0-9]+)?)\)|scale\((-?[0-9]+(?:\.[0-9]+)?)\))$").unwrap()
});
pub struct Dbus {
    pub bus: Bus,
    pub max_input: usize,
    binding: Option<Identity>,
    managed_export: bool,
}
impl Dbus {
    pub fn new(bus: Bus, max_input: usize) -> Self {
        Self {
            bus,
            max_input,
            binding: None,
            managed_export: false,
        }
    }
    pub fn bind_context(&mut self, identity: Option<Identity>) {
        self.binding = identity;
    }
    pub fn managed_export(&mut self) {
        self.managed_export = true;
    }
    pub(crate) fn activate(
        &mut self,
        action: Action,
        parameter: Parameter<'_>,
        target: Target,
    ) -> Result<(), Error> {
        if let Some(identity) = self.binding.as_ref() {
            self.bus
                .context(Context::Activate(identity, action, parameter))
                .map(|_| ())
        } else {
            self.bus.activate(action, parameter, target)
        }
    }
    fn export(
        &mut self,
        png: bool,
        region: Option<[f64; 4]>,
        scale: Option<f64>,
    ) -> Result<Vec<u8>, Error> {
        // Validate all numeric options before changing the instance's export settings.
        let area = if let Some([x, y, width, height]) = region {
            if ![x, y, width, height, x + width, y + height]
                .iter()
                .all(|n| n.is_finite())
            {
                return Err(Error::Protocol(
                    "non-finite render region rejected before DBus activation",
                ));
            }
            Some(
                [x, y, x + width, y + height]
                    .map(crate::live_models::float_repr)
                    .join(":"),
            )
        } else {
            None
        };
        let dpi = scale.map(|n| n * 96.0);
        if let Some(n) = dpi {
            Parameter::Double(n).text()?;
        }
        let temp = tempfile::Builder::new()
            .prefix("inkscape-mcp-live-")
            .tempdir()
            .map_err(|_| Error::Connection("could not prepare DBus export"))?;
        let root = temp
            .path()
            .canonicalize()
            .map_err(|_| Error::Connection("could not prepare DBus export"))?;
        let filename = if png { "live.png" } else { "live.svg" };
        let path = root.join(filename);
        let path = path.to_str().ok_or(Error::Protocol(
            "temp directory path is not safe for a DBus export (check TMPDIR)",
        ))?;
        Parameter::String(path).text().map_err(|_| {
            Error::Protocol("temp directory path is not safe for a DBus export (check TMPDIR)")
        })?;
        if self.managed_export {
            self.activate(Action::ExportId, Parameter::String(""), Target::App)?;
            self.activate(Action::ExportIdOnly, Parameter::Bool(false), Target::App)?;
            self.activate(
                Action::ExportTextToPath,
                Parameter::Bool(false),
                Target::App,
            )?;
            self.activate(Action::ExportPlainSvg, Parameter::Bool(false), Target::App)?;
        }
        self.activate(Action::ExportFilename, Parameter::String(path), Target::App)?;
        self.activate(
            Action::ExportType,
            Parameter::String(if png { "png" } else { "svg" }),
            Target::App,
        )?;
        if !png && !self.managed_export {
            self.activate(Action::ExportPlainSvg, Parameter::Bool(true), Target::App)?;
        }
        if let Some(area) = area {
            self.activate(Action::ExportArea, Parameter::String(&area), Target::App)?;
        } else if png {
            self.activate(Action::ExportAreaPage, Parameter::Bool(true), Target::App)?;
        }
        if let Some(dpi) = dpi {
            self.activate(Action::ExportDpi, Parameter::Double(dpi), Target::App)?;
        }
        self.activate(Action::ExportDo, Parameter::Empty, Target::App)?;
        // The external app writes the file; read through the existing no-follow regular-file pipeline.
        let workspace = Workspace {
            roots: vec![root],
            max_input: self.max_input,
            max_output: self.max_input,
        };
        workspace
            .read(0, Path::new(filename), self.max_input)
            .map_err(|message| {
                if message == "input file exceeds the configured size limit" {
                    Error::Protocol("live document export exceeds the input size cap")
                } else {
                    Error::Connection("Inkscape did not produce a safe DBus export")
                }
            })
    }
    fn window(&mut self) -> Result<u32, Error> {
        let text = self.bus.introspect()?;
        WINDOW
            .captures(&text)
            .and_then(|c| c[1].parse().ok())
            .ok_or(Error::Unsupported(
                "no Inkscape window is available for viewport control",
            ))
    }
}
impl Transport for Dbus {
    fn name(&self) -> &str {
        "dbus"
    }
    fn supports(&self, command: Command) -> bool {
        matches!(
            command,
            Command::Ping
                | Command::ActiveDocument
                | Command::DocumentSvg
                | Command::RenderView
                | Command::SetViewport
                | Command::ApplySelection
        )
    }
    fn connect(&mut self) -> Result<(), Error> {
        self.bus.connect()
    }
    fn disconnect(&mut self) {
        self.bus.disconnect();
    }
    fn is_connected(&self) -> bool {
        self.bus.is_connected()
    }
    fn active_document(&mut self) -> Result<Value, Error> {
        let bytes = self.export(false, None, None)?;
        let document = crate::xml::parse(&bytes, self.max_input)
            .map_err(|_| Error::Protocol("live document could not be parsed safely"))?;
        let root = document
            .get_root_element()
            .ok_or(Error::Protocol("live document could not be parsed safely"))?;
        let name = root.get_property_ns(
            "docname",
            "http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd",
        );
        let mut pending = root.get_child_nodes();
        let mut count = 0usize;
        while let Some(node) = pending.pop() {
            if matches!(
                node.get_type(),
                Some(
                    NodeType::ElementNode
                        | NodeType::CommentNode
                        | NodeType::PiNode
                        | NodeType::EntityRefNode
                )
            ) {
                count += 1;
                if node.get_type() == Some(NodeType::ElementNode) {
                    pending.extend(node.get_child_nodes());
                }
            }
        }
        Ok(
            json!({"window_id":null,"document_id":null,"name":name,"path":null,"object_count":count}),
        )
    }
    fn document_svg(&mut self) -> Result<String, Error> {
        String::from_utf8(self.export(false, None, None)?)
            .map_err(|_| Error::Protocol("live document export was not valid UTF-8"))
    }
    fn selection(&mut self) -> Result<Value, Error> {
        Err(Error::Unsupported(
            "DBus cannot read selection ids; use the extension-socket transport",
        ))
    }
    fn inspect_selection(&mut self) -> Result<Value, Error> {
        Err(Error::Unsupported(
            "DBus cannot inspect the selection; use the extension-socket transport",
        ))
    }
    fn render_view(
        &mut self,
        region: Option<[f64; 4]>,
        scale: Option<f64>,
    ) -> Result<Vec<u8>, Error> {
        self.export(true, region, scale)
    }
    fn set_viewport(&mut self, viewport: &Viewport) -> Result<Value, Error> {
        let (action, parameter, mode, detail) = match viewport {
            Viewport::FitPage => (
                Action::ZoomPage,
                Parameter::Empty,
                "fit_page",
                "fit page (DBus, no-freeze)",
            ),
            Viewport::FitSelection => (
                Action::ZoomSelection,
                Parameter::Empty,
                "fit_selection",
                "fit selection (DBus, no-freeze)",
            ),
            Viewport::Zoom { zoom, .. } => (
                Action::ZoomAbsolute,
                Parameter::Double(*zoom),
                "zoom",
                "zoom (DBus, no-freeze)",
            ),
            Viewport::Pan { .. } => {
                return Err(Error::Unsupported(
                    "DBus cannot pan the viewport; use the extension-socket transport",
                ));
            }
        };
        parameter.text()?;
        let window = self.window()?;
        self.activate(action, parameter, Target::Window(window))?;
        Ok(json!({"mode":mode,"applied":true,"detail":detail}))
    }
    fn apply_selection(
        &mut self,
        style: &BTreeMap<String, String>,
        transform: Option<&str>,
    ) -> Result<Value, Error> {
        let mut actions = Vec::new();
        if style.len() > 64 || transform.is_some_and(|s| s.len() > 4096) {
            return Err(Error::Protocol("DBus selection edit exceeds size cap"));
        }
        for (property, value) in style {
            Parameter::String(property).text()?;
            Parameter::String(value).text()?;
            actions.push((
                Action::ObjectProperty,
                format!("{property}, {value}"),
                property.as_str(),
            ));
        }
        if let Some(transform) = transform {
            for part in transform.split(' ').filter(|s| !s.is_empty()) {
                if actions.len() >= 80 {
                    return Err(Error::Protocol("DBus selection edit exceeds size cap"));
                }
                let captures = TRANSFORM.captures(part).ok_or(Error::Protocol(
                    "unsupported transform for the DBus action path",
                ))?;
                let item = if let Some(dx) = captures.get(1) {
                    (
                        Action::Translate,
                        format!("{},{}", dx.as_str(), &captures[2]),
                        "translate",
                    )
                } else if let Some(degrees) = captures.get(3) {
                    (Action::Rotate, degrees.as_str().into(), "rotate")
                } else {
                    (Action::Scale, captures[4].into(), "scale")
                };
                actions.push(item);
            }
        }
        if actions.is_empty() {
            return Err(Error::Protocol(
                "supply at least one style or transform parameter",
            ));
        }
        // Refuse the whole request before any mutation; no partial activation on malformed tail.
        for (action, value, _) in &actions {
            if let Some(identity) = self.binding.as_ref() {
                self.bus.validate_context(Context::Activate(
                    identity,
                    *action,
                    Parameter::String(value),
                ))?;
            } else {
                self.bus
                    .validate_activation(*action, Parameter::String(value), Target::App)?;
            }
        }
        let mut applied = Vec::new();
        for (action, value, label) in actions {
            self.activate(action, Parameter::String(&value), Target::App)?;
            applied.push(label);
        }
        Ok(
            json!({"affected_ids":[],"count":0,"detail":format!("applied {} to the selection (DBus, no-freeze)",applied.join(", ")),"undo_friendly":true}),
        )
    }
}
impl Drop for Dbus {
    fn drop(&mut self) {
        self.bus.disconnect();
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use std::{os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
    pub(crate) fn fixture(mode: &str, cap: usize) -> (tempfile::TempDir, Dbus, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let log = root.path().join("calls.jsonl");
        let binary = root.path().join("gdbus");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("migration/contracts/dbus-backend-cases.json");
        let python = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(".venv/bin/python");
        let body = format!(
            r#"#!{}
import sys,json,base64
from pathlib import Path
root=Path({:?})
mode={:?}
argv=sys.argv[1:]
control=json.loads((root/'probe-control.json').read_text()) if (root/'probe-control.json').exists() else {{}}
with (root/'calls.jsonl').open('a') as f: f.write(json.dumps(argv)+'\n')
if 'org.freedesktop.DBus.GetNameOwner' in argv: print("(':1.23',)")
elif 'org.gtk.Actions.List' in argv:
    if not control.get('reachable',True): raise SystemExit(1)
    print('(list,)')
elif 'org.gtk.Actions.Describe' in argv:
    if not control.get('reachable',True): raise SystemExit(1)
    available=control.get('insert' if argv[-1]=='org.inkscape-mcp.insert.noprefs' else 'edit',mode!='effect-absent')
    print('((true,),)' if available else '((false,),)')
elif 'introspect' in argv: print('node /org/inkscape/Inkscape/window/17 {{ }}')
elif 'org.inkscape.MCP.Context1.GetContext' in argv:
    row=json.loads((root/'context.json').read_text()) if (root/'context.json').exists() else ['12345678-1234-1234-1234-123456789abc','abcdefab-1234-1234-1234-123456789abc','Drawing.svg']
    print(repr(tuple(row)))
elif 'org.inkscape.MCP.Context1.ListDocuments' in argv:
    print("([('12345678-1234-1234-1234-123456789abc','abcdefab-1234-1234-1234-123456789abc','Drawing.svg')],)")
elif 'org.inkscape.MCP.Context1.SelectDocument' in argv:
    (root/'context.json').write_text(json.dumps(argv[-2:]+['Chosen.svg']))
    print('()')
else:
    if 'org.inkscape.MCP.Context1.Activate' in argv:
        row=json.loads((root/'context.json').read_text()) if (root/'context.json').exists() else ['12345678-1234-1234-1234-123456789abc','abcdefab-1234-1234-1234-123456789abc','Drawing.svg']
        if argv[-4:-2]!=row[:2]:
            sys.stderr.write('org.inkscape.MCP.ContextChanged: private detail')
            raise SystemExit(1)
        action,parameter=argv[-2:]
    else: action,parameter=argv[-3:-1]
    if action in ['org.inkscape-mcp.edit.noprefs','org.inkscape-mcp.insert.noprefs']:
        request=json.loads((root/'insert-request.json').read_text())
        (root/'captured-request.json').write_text(json.dumps(request))
        response=dict(nonce=request['nonce'],ok=True,ids=request.get('selection',[]),fingerprint=request['expected_fingerprint'])
        if mode=='effect-mutated' or action=='org.inkscape-mcp.insert.noprefs':
            from lxml import etree
            fixture_path=Path({:?})
            sys.path.insert(0,str(fixture_path.parents[2]/"runtime"))
            from insert_payload import document_fingerprint,prepare_fragment
            data=json.loads(fixture_path.read_text())
            drawing=etree.fromstring(base64.b64decode(data['svg']))
            if document_fingerprint(drawing)!=request['expected_fingerprint']: raise SystemExit(1)
            if action=='org.inkscape-mcp.insert.noprefs':
                payload,ids=prepare_fragment(request['fragment'],request['nonce'])
                drawing.append(etree.fromstring(payload))
                response=dict(nonce=request['nonce'],ok=True,ids=ids)
            else:
                drawing.xpath('//*[@id="r"]')[0].set('fill',request['style']['fill'])
                response['fingerprint']=document_fingerprint(drawing)
            if mode not in ['effect-mismatch','effect-refused','effect-lost-refused']:
                (root/'drawing.svg').write_bytes(etree.tostring(drawing))
        if mode=='effect-refused' or mode=='effect-lost-refused': response=dict(nonce=request['nonce'],ok=False,error='invalid selection')
        elif mode=='effect-unknown-refusal': response=dict(nonce=request['nonce'],ok=False,error='private /host/path')
        elif mode=='effect-stale': response['nonce']='stale'
        elif mode=='effect-mismatch': response['fingerprint']='wrong'
        if mode!='effect-missing': (root/'insert-result.json').write_text(json.dumps(response))
        if mode=='effect-lost' or mode=='effect-lost-refused': raise SystemExit(1)
        if mode=='effect-switch': (root/'context.json').write_text(json.dumps(['abcdefab-1234-1234-1234-123456789abc','12345678-1234-1234-1234-123456789abc','Changed']))
    elif action=='select-list':
        with (root/'stdout.log').open('ab') as stream:
            if mode=='selection-empty': pass
            elif mode=='selection-overflow': stream.write(b'x'*(1024*1024+1))
            elif mode=='selection-invalid': stream.write(b'changed format\n')
            elif mode=='selection-utf8': stream.write(b'\xff')
            elif mode.startswith('effect-'): stream.write(b'r cloned: true ref: 1 href: 0 total href: 0\n')
            else: stream.write('r cloned: true ref: 1 href: 0 total href: 0\nПривіт cloned: false ref: 0 href: 0 total href: 0\nr cloned: true ref: 1 href: 0 total href: 0\n'.encode())
    elif action=='query-x':
        if mode!='selection-incomplete':
            with (root/'stdout.log').open('ab') as stream: stream.write(b'0\n')
    elif action=='export-filename': (root/'filename').write_text(parameter[3:-3])
    elif action=='export-do':
        out=Path((root/'filename').read_text())
        data=json.loads(Path({:?}).read_text())
        if mode=='missing': pass
        elif mode=='symlink':
            (root/'original').write_bytes(b'keep original')
            out.symlink_to(root/'original')
        elif mode=='oversize': out.write_bytes(b'x'*1024)
        elif mode=='bad-xml': out.write_bytes(b'<invalid')
        elif mode=='bad-utf8': out.write_bytes(b'\xff')
        else:
            payload=base64.b64decode(data['png' if out.suffix=='.png' else 'svg'])
            if (root/'drawing.svg').exists() and out.suffix=='.svg': payload=(root/'drawing.svg').read_bytes()
            if mode=='mapped' and out.suffix=='.svg': payload=payload.replace(b'<svg ',b'<svg id="svgroot" width="200" height="100" viewBox="10 20 50 25" preserveAspectRatio="none" ')
            out.write_bytes(payload)
    print('()')
"#,
            python.display(),
            root.path().to_string_lossy(),
            mode,
            fixture.to_string_lossy(),
            fixture.to_string_lossy()
        );
        std::fs::write(&binary, body).unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut backend = Dbus::new(Bus::new(binary, Duration::from_secs(1), 8192), cap);
        backend.connect().unwrap();
        (root, backend, log)
    }
    fn calls(log: &Path) -> Vec<Value> {
        std::fs::read_to_string(log)
            .unwrap()
            .lines()
            .skip(2)
            .map(|line| {
                let argv: Vec<String> = serde_json::from_str(line).unwrap();
                assert_eq!(argv[3], ":1.23");
                if argv[0] == "introspect" {
                    return json!({"introspect":true});
                }
                let action = &argv[8];
                let parameter = if action == "export-filename" {
                    "[<'<EXPORT>'>]"
                } else {
                    &argv[9]
                };
                json!({"action":action,"parameter":parameter,"target":argv[5]})
            })
            .collect()
    }
    #[test]
    fn python_backend_results_and_fixed_action_traces_match_with_explicit_preflight_differences() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/dbus-backend-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let (_root, mut backend, log) = self::fixture("normal", 1024 * 1024);
            let spec = &case["spec"];
            let result = match spec["method"].as_str().unwrap() {
                "active_document" => backend.active_document(),
                "document_svg" => backend.document_svg().map(|s| json!(s)),
                "render_view" => backend
                    .render_view(
                        spec["region"]
                            .as_array()
                            .map(|a| std::array::from_fn(|i| a[i].as_f64().unwrap())),
                        spec["scale"].as_f64(),
                    )
                    .map(|bytes| json!({"bytes":STANDARD.encode(bytes)})),
                "set_viewport" => {
                    let viewport = match spec["mode"].as_str().unwrap() {
                        "fit_page" => Viewport::FitPage,
                        "fit_selection" => Viewport::FitSelection,
                        "zoom" => Viewport::Zoom {
                            zoom: spec["zoom"].as_f64().unwrap(),
                            center: Some([3.0, 4.0]),
                        },
                        _ => Viewport::Pan { dx: 0.0, dy: 0.0 },
                    };
                    backend.set_viewport(&viewport)
                }
                "selection" => backend.selection(),
                "inspect_selection" => backend.inspect_selection(),
                _ => {
                    let style = spec["style"]
                        .as_object()
                        .map(|m| {
                            m.iter()
                                .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    backend.apply_selection(&style, spec["transform"].as_str())
                }
            };
            let actual = match result {
                Ok(value) => json!({"result":value}),
                Err(Error::Unsupported(message) | Error::Protocol(message)) => {
                    json!({"error":message})
                }
                Err(error) => json!({"error":error.public_message()}),
            };
            assert_eq!(actual, case["expected"], "{spec}");
            let trace = calls(&log);
            if spec["hardened"] == true {
                assert!(trace.is_empty(), "{spec}");
                assert!(!case["calls"].as_array().unwrap().is_empty());
            } else {
                assert_eq!(json!(trace), case["calls"], "{spec}");
            }
        }
    }
    #[test]
    fn exported_paths_are_regular_bounded_utf8_safe_xml_and_owned_cleanup_only() {
        for mode in ["missing", "symlink", "oversize", "bad-xml", "bad-utf8"] {
            let (root, mut backend, log) = fixture(mode, 512);
            let result = if mode == "bad-utf8" {
                backend.document_svg().map(|s| json!(s))
            } else {
                backend.active_document()
            };
            assert!(result.is_err(), "{mode}");
            let exported = std::fs::read_to_string(root.path().join("filename")).unwrap();
            assert!(!Path::new(&exported).parent().unwrap().exists());
            if mode == "symlink" {
                assert_eq!(
                    std::fs::read(root.path().join("original")).unwrap(),
                    b"keep original"
                );
            }
            assert_eq!(calls(&log).len(), 4);
        }
    }
    #[test]
    fn numeric_and_entire_mutation_preflight_refuses_without_any_app_dispatch() {
        let (_root, mut backend, log) = fixture("normal", 512);
        assert!(
            backend
                .render_view(Some([f64::MAX, 0.0, f64::MAX, 1.0]), None)
                .is_err()
        );
        assert!(backend.render_view(None, Some(f64::INFINITY)).is_err());
        assert!(
            backend
                .set_viewport(&Viewport::Zoom {
                    zoom: f64::NAN,
                    center: None
                })
                .is_err()
        );
        assert!(
            backend
                .apply_selection(
                    &BTreeMap::from([("fill".into(), "red".into())]),
                    Some("rotate(1) bad")
                )
                .is_err()
        );
        assert!(
            backend
                .apply_selection(
                    &BTreeMap::from([
                        ("fill".into(), "red".into()),
                        ("stroke".into(), "bad'".into())
                    ]),
                    None
                )
                .is_err()
        );
        assert!(calls(&log).is_empty());
        assert!(
            backend
                .apply_selection(
                    &BTreeMap::from([
                        ("fill".into(), "red".into()),
                        ("stroke".into(), "x".repeat(8192)),
                    ]),
                    None
                )
                .is_err()
        );
        assert!(calls(&log).is_empty());
        assert!(backend.is_connected());
    }
}
