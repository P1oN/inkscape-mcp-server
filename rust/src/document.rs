//! Registry and summary inspection over descriptor-anchored working copies.

use crate::{workspace::Workspace, xml};
use libxml::tree::{Document, Node};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const INKSCAPE_NS: &str = "http://www.inkscape.org/namespaces/inkscape";
const SODIPODI_NS: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd";
const PERSIST_HINT: &str = "Edits apply to a working copy — your original file on disk is NEVER changed. To persist the result, call save_document_as (writes a NEW .svg) or export_document (PNG/PDF/SVG). render_preview only displays it.";

#[cfg(test)]
thread_local! {
    static BEFORE_REGISTRY_TEST: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub root: usize,
    pub source: String,
    pub opened_at: String,
}

impl Entry {
    pub fn directory(&self) -> PathBuf {
        Path::new(".inkscape-mcp/documents").join(&self.id)
    }

    pub fn working(&self) -> PathBuf {
        self.directory().join("working/document.svg")
    }
}

#[derive(Clone)]
pub struct Registry {
    pub workspace: Workspace,
    pub entries: indexmap::IndexMap<String, Entry>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            workspace: Workspace::from_env(),
            entries: indexmap::IndexMap::new(),
        }
    }

    pub fn open(&mut self, arguments: &Value) -> Result<Value, String> {
        let path = arguments["path"].as_str().ok_or("path must be a string")?;
        let (root, relative) = self
            .workspace
            .resolve_input(path, crate::arguments::string(arguments, "root_id")?)?;
        let bytes = self
            .workspace
            .read(root, &relative, self.workspace.max_input)?;
        self.seed(
            root,
            relative.to_string_lossy().into_owned(),
            bytes,
            "open_document",
        )
    }

    pub fn create(&mut self, arguments: &Value) -> Result<Value, String> {
        let width = crate::arguments::number(&arguments["width"])
            .ok_or("width must be a finite number greater than 0")?;
        let height = crate::arguments::number(&arguments["height"])
            .ok_or("height must be a finite number greater than 0")?;
        if !width.is_finite() || width <= 0.0 {
            return Err("width must be a finite number greater than 0".into());
        }
        if !height.is_finite() || height <= 0.0 {
            return Err("height must be a finite number greater than 0".into());
        }
        let width = crate::style::format_num(width)?;
        let height = crate::style::format_num(height)?;
        let viewbox = if let Some(value) = crate::arguments::string(arguments, "viewBox")? {
            let numbers: Result<Vec<f64>, _> = value
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .map(str::parse)
                .collect();
            let numbers = numbers
                .ok()
                .filter(|v| v.len() == 4 && v.iter().all(|n| n.is_finite()))
                .ok_or("viewBox must be four numbers: 'minx miny width height'")?;
            if numbers[2] <= 0.0 || numbers[3] <= 0.0 {
                return Err("viewBox width/height must be positive".into());
            }
            numbers
                .into_iter()
                .map(crate::style::format_num)
                .collect::<Result<Vec<_>, _>>()?
                .join(" ")
        } else {
            format!("0 0 {width} {height}")
        };
        let background = if let Some(value) = crate::arguments::string(arguments, "background")? {
            format!(
                "<rect id=\"background\" x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" fill=\"{}\"/>",
                crate::style::color(value)?
            )
        } else {
            String::new()
        };
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"{viewbox}\">{background}</svg>"
        );
        let document = xml::parse(svg.as_bytes(), self.workspace.max_input)?;
        if self.workspace.roots.is_empty() {
            return Err("no workspace root configured".into());
        }
        self.seed(
            0,
            "document.svg".into(),
            xml::serialize(&document),
            "create_document",
        )
    }

    fn seed(
        &mut self,
        root: usize,
        source: String,
        bytes: Vec<u8>,
        tool: &str,
    ) -> Result<Value, String> {
        // Validate before creating directories, copies or the registry entry.
        // A refused open must not leave an unusable managed document behind.
        let document = xml::parse(&bytes, self.workspace.max_input)
            .map_err(|message| format!("Error calling tool '{tool}': {message}"))?;
        let mut result_summary = summary("", &document)
            .map_err(|message| format!("Error calling tool '{tool}': {message}"))?;
        let id = self.seed_entry(root, source, bytes)?;
        result_summary["doc_id"] = json!(id);
        Ok(json!({"doc_id":id,"summary":result_summary,"persist_hint":PERSIST_HINT}))
    }

    /// Register immutable baseline/working bytes without requiring an inspection.
    /// Live sync mirrors the reference registry, including raw snapshots inspected later.
    pub(crate) fn seed_entry(
        &mut self,
        root: usize,
        source: String,
        bytes: Vec<u8>,
    ) -> Result<String, String> {
        let id = format!("d_{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let entry = Entry {
            id: id.clone(),
            root,
            source,
            opened_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false),
        };
        // Reject an unreadable/nonregular registry before creating document files.
        let index = Path::new(".inkscape-mcp/registry.json");
        let previous_index = match self.workspace.file_kind(root, Path::new(".inkscape-mcp"))? {
            None => None,
            Some(_) => self
                .workspace
                .read_optional(root, index, self.workspace.max_output)?,
        };
        self.workspace
            .write_new(root, &entry.directory().join("original.svg"), &bytes)?;
        self.workspace.write_new(root, &entry.working(), &bytes)?;
        for directory in [
            "snapshots",
            "operations",
            "artifacts/exports",
            "artifacts/preview",
        ] {
            self.workspace
                .ensure_directory(root, &entry.directory().join(directory))?;
        }
        let mut entries: Vec<_> = self.entries.values().filter(|e| e.root == root).map(|e| json!({
            "doc_id":e.id,"source_path":e.source,"workspace_dir_name":e.id,"opened_at":e.opened_at
        })).collect();
        entries.push(json!({"doc_id":entry.id,"source_path":entry.source,"workspace_dir_name":entry.id,"opened_at":entry.opened_at}));
        #[cfg(test)]
        BEFORE_REGISTRY_TEST.with(|hook| {
            if let Some(action) = hook.borrow_mut().take() {
                action();
            }
        });
        if let Err(error) = self.workspace.atomic_write(
            root,
            index,
            &serde_json::to_vec_pretty(&json!({"documents":entries})).unwrap(),
        ) {
            // A post-rename fsync failure can still have published the candidate.
            let rollback = match previous_index {
                Some(bytes) => self.workspace.atomic_write(root, index, &bytes),
                None => self
                    .workspace
                    .read_optional(root, index, self.workspace.max_output)
                    .and_then(|current| {
                        if current.is_some() {
                            self.workspace.remove_file(root, index)
                        } else {
                            Ok(())
                        }
                    }),
            };
            if rollback.is_err() {
                return Err("document registration failed; registry recovery required, candidate bytes retained".into());
            }
            // Candidate files remain available for diagnosis; never expose a failed
            // registration as an active in-memory document.
            return Err(error);
        }
        self.entries.insert(id.clone(), entry);
        Ok(id)
    }

    pub fn summary(&self, id: &str) -> Result<Value, String> {
        let entry = self.entries.get(id).ok_or("document id not found")?;
        let bytes = self
            .workspace
            .read(entry.root, &entry.working(), self.workspace.max_input)?;
        let document = xml::parse(&bytes, self.workspace.max_input)?;
        summary(id, &document)
    }

    pub fn reload(&self, arguments: &Value) -> Result<Value, String> {
        let id = arguments["doc_id"]
            .as_str()
            .ok_or("doc_id must be a string")?;
        let entry = self.entries.get(id).ok_or("document id not found")?;
        // Reference ordering: even a refused reload keeps its pre-reload checkpoint.
        let checkpoint = crate::transaction::snapshot(self, entry, Some("pre-reload"), None)?;
        let original = entry.directory().join("original.svg");
        let previous_original =
            self.workspace
                .read(entry.root, &original, self.workspace.max_input)?;
        let previous_working =
            self.workspace
                .read(entry.root, &entry.working(), self.workspace.max_input)?;
        let source = self.workspace.roots[entry.root].join(&entry.source);
        // Preserve the legacy seed sentinel and missing/nonregular-source fallback.
        let external = entry.source != "document.svg" && source.is_file();
        let bytes = if external {
            let (root, relative) = self
                .workspace
                .resolve_input(&source.to_string_lossy(), None)?;
            self.workspace
                .read(root, &relative, self.workspace.max_input)?
        } else {
            previous_original.clone()
        };
        // Refuse malformed replacement before overwriting either managed copy.
        let parsed = xml::parse(&bytes, self.workspace.max_input)?;
        let result_summary = summary(id, &parsed)?;
        let write = (|| {
            if external {
                self.workspace.atomic_write(entry.root, &original, &bytes)?;
            }
            self.workspace
                .atomic_write(entry.root, &entry.working(), &bytes)
        })();
        if let Err(error) = write {
            // fsync errors can occur after rename; restore both prior byte streams.
            let restore = |path: &Path, previous: &[u8]| {
                if self
                    .workspace
                    .read(entry.root, path, self.workspace.max_input)
                    .is_ok_and(|current| current == previous)
                {
                    Ok(())
                } else {
                    self.workspace.atomic_write(entry.root, path, previous)
                }
            };
            let baseline = restore(&original, &previous_original);
            let working = restore(&entry.working(), &previous_working);
            if baseline.is_err() || working.is_err() {
                return Err(format!(
                    "{error}; reload rollback failed; pre-reload snapshot retained"
                ));
            }
            return Err(error);
        }
        Ok(
            json!({"doc_id":id,"pre_reload_snapshot_id":checkpoint["snapshot_id"],"summary":result_summary}),
        )
    }
}

pub fn elements(root: Node) -> Vec<Node> {
    inkscape_mcp_rust::helper_svg::elements(root)
}

fn summary(id: &str, document: &Document) -> Result<Value, String> {
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let nodes = elements(root.clone());
    let width = root.get_property_no_ns("width");
    let units = root
        .get_property_ns("document-units", INKSCAPE_NS)
        .or_else(|| root.get_property_ns("document-units", SODIPODI_NS))
        .or_else(|| {
            width.as_ref().and_then(|value| {
                let trimmed = value.trim();
                let suffix: String = trimmed
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_lowercase() || *c == '%')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                (!suffix.is_empty()).then_some(suffix)
            })
        });
    let viewbox = root.get_property_no_ns("viewBox").and_then(|value| {
        let numbers: Result<Vec<f64>, _> = value
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .map(str::parse)
            .collect();
        numbers.ok().filter(|v| v.len() == 4)
    });
    let page_count = nodes
        .iter()
        .filter(|n| {
            n.get_name() == "page"
                && n.get_namespace()
                    .is_some_and(|ns| ns.get_href() == INKSCAPE_NS)
        })
        .count()
        .max(1);
    let num_objects = nodes
        .iter()
        .filter(|n| {
            !matches!(
                n.get_name().as_str(),
                "svg" | "defs" | "metadata" | "style" | "title" | "desc" | "namedview"
            )
        })
        .count();
    let num_layers = nodes
        .iter()
        .filter(|n| {
            n.get_name() == "g"
                && n.get_property_ns("groupmode", INKSCAPE_NS).as_deref() == Some("layer")
        })
        .count();
    Ok(
        json!({"doc_id":id,"width":width,"height":root.get_property_no_ns("height"),"units":units,
        "viewbox":viewbox,"page_count":page_count,"num_objects":num_objects,
        "num_layers":num_layers,"root_tag":root.get_name()}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_registry_destination_does_not_publish_a_document_or_copy() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let mut registry = Registry {
            workspace: Workspace {
                roots: vec![root.clone()],
                max_input: 4096,
                max_output: 4096,
            },
            entries: indexmap::IndexMap::new(),
        };
        std::fs::create_dir_all(root.join(".inkscape-mcp/registry.json")).unwrap();
        std::fs::write(root.join("fixture.svg"), "<svg><rect id=\"r\"/></svg>").unwrap();
        assert!(registry.open(&json!({"path":"fixture.svg"})).is_err());
        assert!(registry.entries.is_empty());
        assert!(!root.join(".inkscape-mcp/documents").exists());
        assert!(root.join(".inkscape-mcp/registry.json").is_dir());
        std::fs::remove_dir(root.join(".inkscape-mcp/registry.json")).unwrap();
        assert!(registry.open(&json!({"path":"fixture.svg"})).is_ok());
        assert_eq!(registry.entries.len(), 1);
    }

    #[test]
    fn late_registry_write_failure_does_not_publish_candidate_in_memory() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let mut registry = Registry {
            workspace: Workspace {
                roots: vec![root.clone()],
                max_input: 4096,
                max_output: 4096,
            },
            entries: indexmap::IndexMap::new(),
        };
        let original = b"<svg><rect id=\"r\"/></svg>";
        std::fs::write(root.join("fixture.svg"), original).unwrap();
        let destination = root.join(".inkscape-mcp/registry.json");
        BEFORE_REGISTRY_TEST.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || std::fs::create_dir(destination).unwrap()))
        });
        assert!(registry.open(&json!({"path":"fixture.svg"})).is_err());
        assert!(registry.entries.is_empty());
        assert_eq!(std::fs::read(root.join("fixture.svg")).unwrap(), original);
        // Ambiguous persistence retains candidate bytes for diagnosis, never
        // presents them as a successfully opened document or deletes originals.
        assert_eq!(
            std::fs::read_dir(root.join(".inkscape-mcp/documents"))
                .unwrap()
                .count(),
            1
        );
    }

    #[test]
    fn refused_open_validates_before_any_managed_write_and_valid_open_still_works() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let mut registry = Registry {
            workspace: Workspace {
                roots: vec![root.clone()],
                max_input: 4096,
                max_output: 4096,
            },
            entries: indexmap::IndexMap::new(),
        };
        for bytes in [b"<svg><g></svg>".as_slice(), b"<x:svg/>"] {
            let original = root.join("fixture.svg");
            std::fs::write(&original, bytes).unwrap();
            let modified = std::fs::metadata(&root).unwrap().modified().unwrap();
            let error = registry.open(&json!({"path":"fixture.svg"})).unwrap_err();
            assert_eq!(
                error,
                "Error calling tool 'open_document': document could not be parsed safely"
            );
            assert!(registry.entries.is_empty());
            assert!(!root.join(".inkscape-mcp").exists());
            assert_eq!(std::fs::read(&original).unwrap(), bytes);
            assert_eq!(
                std::fs::metadata(&root).unwrap().modified().unwrap(),
                modified
            );
        }
        let bytes = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r"/></svg>"#;
        std::fs::write(root.join("fixture.svg"), bytes).unwrap();
        let opened = registry.open(&json!({"path":"fixture.svg"})).unwrap();
        let id = opened["doc_id"].as_str().unwrap();
        assert_eq!(opened["summary"]["doc_id"], id);
        assert_eq!(opened["summary"]["num_objects"], 1);
        assert_eq!(registry.summary(id).unwrap(), opened["summary"]);
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(std::fs::read(root.join("fixture.svg")).unwrap(), bytes);
    }
}
