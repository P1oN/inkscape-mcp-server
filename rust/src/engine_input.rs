//! Owned, bounded asset staging for headless engine inputs. Never pass source asset paths
//! to Inkscape. Unsupported dependency syntax fails before engine execution.
use crate::{document, workspace::Workspace, xml};
use base64::{Engine, engine::general_purpose::STANDARD};
use libxml::tree::{Document, NodeType};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

const UNSUPPORTED: &str = "engine input refused: unsupported external SVG/CSS dependency";
const LIMIT: &str = "engine input dependencies exceed the configured size limit";
const XLINK: &str = "http://www.w3.org/1999/xlink";
type Fallback = (Option<String>, String);
struct Original {
    href: String,
    export: String,
    fallback: Vec<Fallback>,
}

pub struct Prepared {
    pub bytes: Vec<u8>,
    // Retained until engine processing ends; staged paths cannot refer to a dropped directory.
    _assets: tempfile::TempDir,
    _nested: Vec<Prepared>,
    originals: HashMap<String, Original>,
    css_links: HashMap<String, Original>,
}
fn original_link<'a>(
    links: &'a HashMap<String, Original>,
    href: &str,
    output_base: Option<&Path>,
) -> Option<&'a Original> {
    if let Some(link) = links.get(href) {
        return Some(link);
    }
    let (resource, fragment) = href
        .split_once('#')
        .map(|(a, b)| (a, format!("#{b}")))
        .unwrap_or((href, String::new()));
    let path = uri_path(resource).ok()?;
    let absolute = if path.is_absolute() {
        path
    } else {
        output_base?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        if component == std::path::Component::ParentDir {
            normalized.pop();
        } else if component != std::path::Component::CurDir {
            normalized.push(component.as_os_str());
        }
    }
    links.get(&format!("{}{fragment}", path_uri(&normalized).ok()?))
}

impl Prepared {
    fn private_reference(&self, bytes: &[u8]) -> bool {
        let path = self._assets.path().to_string_lossy();
        bytes.windows(path.len()).any(|w| w == path.as_bytes())
            || path_uri(self._assets.path())
                .is_ok_and(|uri| bytes.windows(uri.len()).any(|w| w == uri.as_bytes()))
            || self
                ._nested
                .iter()
                .any(|child| child.private_reference(bytes))
    }
    pub fn restore(
        &self,
        result: &Document,
        absolute: bool,
        output_base: Option<&Path>,
    ) -> Result<(), String> {
        let root = result
            .get_root_element()
            .ok_or("engine produced unsafe output")?;
        for mut node in document::elements(root) {
            let restore_css = |value: &str| {
                crate::css_assets::rewrite(value, |url| {
                    Ok(
                        original_link(&self.css_links, url, output_base).map(|link| {
                            if absolute {
                                link.export.clone()
                            } else {
                                link.href.clone()
                            }
                        }),
                    )
                })
            };
            for ((name, ns), value) in node.get_properties_ns() {
                if css_property(&name) {
                    let restored = restore_css(&value)?;
                    if restored != value {
                        if let Some(ns) = ns {
                            node.set_property_ns(&name, &restored, &ns)
                                .map_err(|_| "engine CSS restore failed")?;
                        } else {
                            node.set_property(&name, &restored)
                                .map_err(|_| "engine CSS restore failed")?;
                        }
                    }
                }
            }
            if node.get_name() == "style" {
                let old = node.get_content();
                let restored = restore_css(&old)?;
                if restored != old {
                    node.set_content(&restored)
                        .map_err(|_| "engine CSS restore failed")?;
                }
            }
            for ns in [None, Some(XLINK)] {
                let href = match ns {
                    None => node.get_property_no_ns("href"),
                    Some(ns) => node.get_property_ns("href", ns),
                };
                if let Some(link) = href
                    .as_ref()
                    .and_then(|s| original_link(&self.originals, s, output_base))
                {
                    let (original, export, fallback) = (&link.href, &link.export, &link.fallback);
                    if !absolute {
                        for (namespace, value) in fallback {
                            if let Some(namespace) = namespace {
                                let found = node
                                    .get_namespaces(result)
                                    .into_iter()
                                    .find(|n| n.get_href() == *namespace);
                                let ns = if let Some(ns) = found {
                                    ns
                                } else {
                                    let mut suffix = 0;
                                    loop {
                                        let prefix = format!("imcp_restore_{suffix}");
                                        if node
                                            .get_namespaces(result)
                                            .iter()
                                            .all(|n| n.get_prefix() != prefix)
                                        {
                                            break libxml::tree::Namespace::new(
                                                &prefix, namespace, &mut node,
                                            )
                                            .map_err(|_| "engine metadata restore failed")?;
                                        }
                                        suffix += 1;
                                    }
                                };
                                node.set_property_ns("absref", value, &ns)
                                    .map_err(|_| "engine metadata restore failed")?;
                            } else {
                                node.set_property("absref", value)
                                    .map_err(|_| "engine metadata restore failed")?;
                            }
                        }
                    }
                    let original = if absolute { export } else { original };
                    if let Some(ns) = ns {
                        let namespace = node
                            .get_namespaces(result)
                            .into_iter()
                            .find(|n| n.get_href() == ns)
                            .ok_or("engine asset namespace unavailable")?;
                        node.set_property_ns("href", original, &namespace)
                            .map_err(|_| "engine asset restore failed")?;
                    } else {
                        node.set_property("href", original)
                            .map_err(|_| "engine asset restore failed")?;
                    }
                }
            }
        }
        if self.private_reference(&xml::serialize(result)) {
            return Err("engine asset restore failed: private staging reference remains".into());
        }
        Ok(())
    }
}

fn raster_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("jpg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some("webp")
    } else {
        None
    }
}

/// Encode a filesystem path as a URI path, leaving genuine fragments to the caller.
/// In particular, literal %, # and ? must never become URI syntax on restoration.
pub(crate) fn path_uri(path: &Path) -> Result<String, String> {
    let path = path.to_str().ok_or(UNSUPPORTED)?;
    let mut uri = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            uri.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(uri, "%{byte:02X}").map_err(|_| UNSUPPORTED)?;
        }
    }
    Ok(uri)
}

pub(crate) fn uri_path(href: &str) -> Result<PathBuf, String> {
    let raw = if let Some(file) = href.strip_prefix("file://") {
        if !file.starts_with('/') {
            return Err(UNSUPPORTED.into());
        }
        file
    } else {
        if href.contains(':') {
            return Err(UNSUPPORTED.into());
        }
        href
    };
    if raw.contains(['?', '#']) {
        return Err(UNSUPPORTED.into());
    }
    let mut decoded = Vec::new();
    let bytes = raw.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let pair = bytes.get(index + 1..index + 3).ok_or(UNSUPPORTED)?;
            let hex = std::str::from_utf8(pair).map_err(|_| UNSUPPORTED)?;
            decoded.push(u8::from_str_radix(hex, 16).map_err(|_| UNSUPPORTED)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let path = String::from_utf8(decoded).map_err(|_| UNSUPPORTED)?;
    if path.contains('\0') {
        return Err(UNSUPPORTED.into());
    }
    Ok(PathBuf::from(path))
}

fn css_property(name: &str) -> bool {
    matches!(
        name,
        "style"
            | "fill"
            | "stroke"
            | "filter"
            | "clip-path"
            | "mask"
            | "cursor"
            | "marker"
            | "marker-start"
            | "marker-mid"
            | "marker-end"
            | "color-profile"
    )
}

fn stage_css_url(
    workspace: &Workspace,
    root_index: usize,
    source: &Path,
    url: &str,
    consumed: &mut usize,
    count: &mut usize,
    depth: usize,
) -> Result<(String, String, Prepared), String> {
    let proxy = xml::parse(b"<svg><image/></svg>", workspace.max_input)?;
    let mut image = proxy
        .get_root_element()
        .unwrap()
        .get_child_elements()
        .remove(0);
    image
        .set_property("href", url)
        .map_err(|_| "engine CSS staging unavailable")?;
    let bytes = xml::serialize(&proxy);
    *consumed = consumed
        .checked_add(bytes.len())
        .filter(|n| *n <= workspace.max_input)
        .ok_or(LIMIT)?;
    let child = prepare_inner(
        workspace, root_index, source, &bytes, consumed, count, depth,
    )?;
    let staged_doc = xml::parse(&child.bytes, workspace.max_input)?;
    let staged = staged_doc
        .get_root_element()
        .unwrap()
        .get_child_elements()
        .remove(0)
        .get_property_no_ns("href")
        .ok_or("engine CSS staging unavailable")?;
    let export = child
        .originals
        .get(&staged)
        .ok_or("engine CSS staging unavailable")?
        .export
        .clone();
    Ok((staged, export, child))
}

fn stage_css_sheet(
    workspace: &Workspace,
    root_index: usize,
    source: &Path,
    url: &str,
    consumed: &mut usize,
    count: &mut usize,
    depth: usize,
) -> Result<(String, String, Prepared), String> {
    if depth > 8 {
        return Err("engine input dependency depth exceeds limit".into());
    }
    *count += 1;
    if *count > 128 {
        return Err("engine input dependency count exceeds limit".into());
    }
    let path = uri_path(url)?;
    let base = workspace.roots[root_index].join(source.parent().ok_or(UNSUPPORTED)?);
    let absolute = if path.is_absolute() {
        path
    } else {
        base.join(path)
    };
    let (index, relative) = workspace.resolve_input(absolute.to_str().ok_or(UNSUPPORTED)?, None)?;
    let bytes = workspace.read(
        index,
        &relative,
        workspace.max_input.saturating_sub(*consumed),
    )?;
    *consumed = consumed
        .checked_add(bytes.len())
        .filter(|n| *n <= workspace.max_input)
        .ok_or(LIMIT)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "engine stylesheet must be UTF-8")?;
    let mut nested = Vec::new();
    let mut css_links = HashMap::new();
    let staged_text = crate::css_assets::rewrite_with_imports(text, |url, import| {
        if !import && url.starts_with('#') && url.len() > 1 {
            return Ok(None);
        }
        let (staged, export, child) = if import {
            stage_css_sheet(workspace, index, &relative, url, consumed, count, depth + 1)?
        } else {
            stage_css_url(workspace, index, &relative, url, consumed, count, depth + 1)?
        };
        css_links.insert(
            staged.clone(),
            Original {
                href: url.to_owned(),
                export,
                fallback: Vec::new(),
            },
        );
        nested.push(child);
        Ok(Some(staged))
    })?;
    *consumed = consumed
        .checked_add(staged_text.len().saturating_sub(bytes.len()))
        .filter(|n| *n <= workspace.max_input)
        .ok_or(LIMIT)?;
    let assets = tempfile::Builder::new()
        .prefix("imcp-assets-")
        .tempdir()
        .map_err(|_| "engine CSS staging unavailable")?;
    let stage = Workspace {
        roots: vec![
            assets
                .path()
                .canonicalize()
                .map_err(|_| "engine CSS staging unavailable")?,
        ],
        max_input: workspace.max_input,
        max_output: workspace.max_input,
    };
    stage.write_new(0, Path::new("sheet.css"), staged_text.as_bytes())?;
    let staged = path_uri(&stage.roots[0].join("sheet.css"))?;
    let export = path_uri(&workspace.roots[index].join(&relative))?;
    Ok((
        staged,
        export,
        Prepared {
            bytes: staged_text.into_bytes(),
            _assets: assets,
            _nested: nested,
            originals: HashMap::new(),
            css_links,
        },
    ))
}

pub fn prepare(
    workspace: &Workspace,
    root_index: usize,
    source: &Path,
    bytes: &[u8],
) -> Result<Prepared, String> {
    let mut consumed = bytes.len();
    prepare_inner(
        workspace,
        root_index,
        source,
        bytes,
        &mut consumed,
        &mut 0,
        0,
    )
}

fn prepare_inner(
    workspace: &Workspace,
    root_index: usize,
    source: &Path,
    bytes: &[u8],
    consumed: &mut usize,
    count: &mut usize,
    depth: usize,
) -> Result<Prepared, String> {
    if depth > 8 {
        return Err("engine input dependency depth exceeds limit".into());
    }
    let doc = xml::parse(bytes, workspace.max_input)?;
    let root = doc
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let mut top = root.clone();
    while let Some(previous) = top.get_prev_sibling() {
        top = previous;
    }
    loop {
        if matches!(top.get_type(), Some(NodeType::DTDNode | NodeType::PiNode)) {
            return Err(UNSUPPORTED.into());
        }
        if let Some(next) = top.get_next_sibling() {
            top = next;
        } else {
            break;
        }
    }
    let mut nodes = vec![root.clone()];
    while let Some(node) = nodes.pop() {
        if matches!(
            node.get_type(),
            Some(NodeType::EntityRefNode | NodeType::PiNode)
        ) {
            return Err(UNSUPPORTED.into());
        }
        nodes.extend(node.get_child_nodes());
    }
    let assets = tempfile::Builder::new()
        .prefix("imcp-assets-")
        .tempdir()
        .map_err(|_| "engine asset staging unavailable")?;
    let stage = Workspace {
        roots: vec![
            assets
                .path()
                .canonicalize()
                .map_err(|_| "engine asset staging unavailable")?,
        ],
        max_input: workspace.max_input,
        max_output: workspace.max_input,
    };
    let base =
        workspace.roots[root_index].join(source.parent().ok_or("engine source unavailable")?);
    let mut originals = HashMap::new();
    let mut nested = Vec::new();
    let mut css_links = HashMap::new();
    for mut node in document::elements(root) {
        if matches!(node.get_name().as_str(), "foreignObject" | "script") {
            return Err(UNSUPPORTED.into());
        }
        let mut fallback = Vec::new();
        for ((name, ns), value) in node.get_properties_ns() {
            if name == "absref" {
                fallback.push((ns.as_ref().map(|n| n.get_href()), value));
                if node.get_name() == "image"
                    && node
                        .get_property_no_ns("href")
                        .or_else(|| node.get_property_ns("href", XLINK))
                        .is_none_or(|href| href.is_empty())
                {
                    return Err(UNSUPPORTED.into());
                }
                if let Some(ns) = ns {
                    node.remove_property_ns("absref", &ns.get_href())
                        .map_err(|_| "engine asset staging unavailable")?;
                } else {
                    node.remove_property_no_ns("absref")
                        .map_err(|_| "engine asset staging unavailable")?;
                }
            }
        }
        if node
            .get_property_ns("base", "http://www.w3.org/XML/1998/namespace")
            .is_some()
        {
            return Err(UNSUPPORTED.into());
        }
        let mut stage_css = |value: &str| {
            crate::css_assets::rewrite_with_imports(value, |url, import| {
                if !import && url.starts_with('#') && url.len() > 1 {
                    return Ok(None);
                }
                let (staged, export, child) = if import {
                    stage_css_sheet(
                        workspace,
                        root_index,
                        source,
                        url,
                        consumed,
                        count,
                        depth + 1,
                    )?
                } else {
                    stage_css_url(
                        workspace,
                        root_index,
                        source,
                        url,
                        consumed,
                        count,
                        depth + 1,
                    )?
                };
                css_links.insert(
                    staged.clone(),
                    Original {
                        href: url.to_owned(),
                        export,
                        fallback: Vec::new(),
                    },
                );
                nested.push(child);
                Ok(Some(staged))
            })
        };
        for ((name, ns), value) in node.get_properties_ns() {
            if css_property(&name) {
                let staged = stage_css(&value)?;
                if staged != value {
                    if let Some(ns) = ns {
                        node.set_property_ns(&name, &staged, &ns)
                            .map_err(|_| "engine CSS staging unavailable")?;
                    } else {
                        node.set_property(&name, &staged)
                            .map_err(|_| "engine CSS staging unavailable")?;
                    }
                }
            }
        }
        if node.get_name() == "style" {
            let old = node.get_content();
            let staged = stage_css(&old)?;
            if staged != old {
                node.set_content(&staged)
                    .map_err(|_| "engine CSS staging unavailable")?;
            }
        }
        for ns in [None, Some(XLINK)] {
            let href = match ns {
                None => node.get_property_no_ns("href"),
                Some(ns) => node.get_property_ns("href", ns),
            };
            let Some(href) = href else { continue };
            if href.is_empty() || href.starts_with('#') {
                continue;
            }
            if !matches!(node.get_name().as_str(), "image" | "feImage" | "use") {
                return Err(UNSUPPORTED.into());
            }
            *count += 1;
            if *count > 128 {
                return Err("engine input dependency count exceeds limit".into());
            }
            let (resource, fragment) = if !href.starts_with("data:") {
                if let Some((resource, fragment)) = href.split_once('#') {
                    if !crate::render::safe_object_id(fragment) {
                        return Err(UNSUPPORTED.into());
                    }
                    (resource, format!("#{fragment}"))
                } else {
                    (href.as_str(), String::new())
                }
            } else {
                (href.as_str(), String::new())
            };
            let mut export = href.clone();
            let mut asset_root = root_index;
            let mut asset_source = source.to_path_buf();
            let asset = if let Some(data) = resource.strip_prefix("data:") {
                let (header, payload) = data.split_once(',').ok_or(UNSUPPORTED)?;
                if !matches!(
                    header,
                    "image/png;base64"
                        | "image/jpeg;base64"
                        | "image/gif;base64"
                        | "image/webp;base64"
                        | "image/svg+xml;base64"
                ) {
                    return Err(UNSUPPORTED.into());
                }
                STANDARD
                    .decode(payload)
                    .map_err(|_| UNSUPPORTED.to_owned())?
            } else {
                let path = uri_path(resource)?;
                let absolute = if path.is_absolute() {
                    path
                } else {
                    base.join(path)
                };
                export = format!("{}{fragment}", path_uri(&absolute)?);
                let (index, relative) =
                    workspace.resolve_input(absolute.to_str().ok_or(UNSUPPORTED)?, None)?;
                asset_root = index;
                asset_source = relative.clone();
                workspace.read(
                    index,
                    &relative,
                    workspace.max_input.saturating_sub(*consumed),
                )?
            };
            *consumed = consumed
                .checked_add(asset.len())
                .filter(|size| *size <= workspace.max_input)
                .ok_or(LIMIT)?;
            let (extension, asset) = if let Some(extension) = raster_extension(&asset) {
                if node.get_name() == "use" || !fragment.is_empty() {
                    return Err(UNSUPPORTED.into());
                }
                (extension, asset)
            } else {
                let child_doc = xml::parse(&asset, workspace.max_input).map_err(|_| UNSUPPORTED)?;
                let child_root = child_doc.get_root_element().ok_or(UNSUPPORTED)?;
                if child_root.get_name() != "svg"
                    || child_root
                        .get_namespace()
                        .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
                {
                    return Err(UNSUPPORTED.into());
                }
                let child = prepare_inner(
                    workspace,
                    asset_root,
                    &asset_source,
                    &asset,
                    consumed,
                    count,
                    depth + 1,
                )?;
                let bytes = child.bytes.clone();
                nested.push(child);
                ("svg", bytes)
            };
            let name = format!("asset-{}.{}", originals.len(), extension);
            stage.write_new(0, Path::new(&name), &asset)?;
            let staged = format!("{}{fragment}", path_uri(&stage.roots[0].join(name))?);
            originals.insert(
                staged.clone(),
                Original {
                    href,
                    export,
                    fallback: fallback.clone(),
                },
            );
            if let Some(ns) = ns {
                let namespace = node
                    .get_namespaces(&doc)
                    .into_iter()
                    .find(|n| n.get_href() == ns)
                    .ok_or("engine asset namespace unavailable")?;
                node.set_property_ns("href", &staged, &namespace)
                    .map_err(|_| "engine asset staging unavailable")?;
            } else {
                node.set_property("href", &staged)
                    .map_err(|_| "engine asset staging unavailable")?;
            }
        }
    }
    let staged_bytes = xml::serialize(&doc);
    *consumed = consumed
        .checked_add(staged_bytes.len().saturating_sub(bytes.len()))
        .filter(|size| *size <= workspace.max_input)
        .ok_or(LIMIT)?;
    let bytes = staged_bytes;
    if bytes.len() > workspace.max_input {
        return Err(LIMIT.into());
    }
    Ok(Prepared {
        bytes,
        _assets: assets,
        _nested: nested,
        originals,
        css_links,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn filesystem_uri_encoding_is_reversible_and_does_not_admit_query_syntax() {
        let path = Path::new("/assets/a#%? &é:one.png");
        let encoded = path_uri(path).unwrap();
        assert_eq!(encoded, "/assets/a%23%25%3F%20%26%C3%A9%3Aone.png");
        assert_eq!(uri_path(&encoded).unwrap(), path);
        assert_eq!(uri_path(&format!("file://{encoded}")).unwrap(), path);
        for invalid in [
            "asset.png?query",
            "asset.png#fragment",
            "asset%",
            "asset%GG",
            "asset%00",
        ] {
            assert!(uri_path(invalid).is_err());
        }
    }

    #[test]
    fn assets_are_copied_from_pinned_workspace_reads_and_original_links_restore() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let png = b"\x89PNG\r\n\x1a\nowned-test";
        std::fs::write(root.path().join("asset.png"), png).unwrap();
        std::fs::write(outside.path().join("outside.png"), png).unwrap();
        symlink(
            outside.path().join("outside.png"),
            root.path().join("link.png"),
        )
        .unwrap();
        let svg = |href: &str| {
            format!("<svg xmlns='http://www.w3.org/2000/svg'><image href='{href}'/></svg>")
        };
        for href in [
            outside
                .path()
                .join("outside.png")
                .to_string_lossy()
                .into_owned(),
            format!("file://{}", outside.path().join("outside.png").display()),
            root.path().join("link.png").to_string_lossy().into_owned(),
        ] {
            assert!(prepare(&workspace, 0, Path::new("input.svg"), svg(&href).as_bytes()).is_err());
        }
        let prepared = prepare(
            &workspace,
            0,
            Path::new("input.svg"),
            svg("asset.png' xmlns:s='urn:metadata' s:absref='/owned/original.png").as_bytes(),
        )
        .unwrap();
        let doc = xml::parse(&prepared.bytes, 4096).unwrap();
        let image = doc
            .get_root_element()
            .unwrap()
            .get_child_elements()
            .remove(0);
        let staged = image.get_property_no_ns("href").unwrap();
        assert_eq!(std::fs::read(&staged).unwrap(), png);
        std::fs::remove_file(root.path().join("asset.png")).unwrap();
        symlink(
            outside.path().join("outside.png"),
            root.path().join("asset.png"),
        )
        .unwrap();
        assert_eq!(std::fs::read(&staged).unwrap(), png); // child sees the copy, not the swapped path
        let plain_output = xml::parse(
            format!("<svg><image href='{staged}'/></svg>").as_bytes(),
            4096,
        )
        .unwrap();
        prepared.restore(&plain_output, false, None).unwrap();
        let plain_image = plain_output
            .get_root_element()
            .unwrap()
            .get_child_elements()
            .remove(0);
        assert_eq!(
            plain_image
                .get_property_ns("absref", "urn:metadata")
                .unwrap(),
            "/owned/original.png"
        );
        prepared.restore(&doc, false, None).unwrap();
        assert_eq!(image.get_property_no_ns("href").unwrap(), "asset.png");
        assert_eq!(
            image.get_property_ns("absref", "urn:metadata").unwrap(),
            "/owned/original.png"
        );
        assert!(
            !xml::serialize(&doc)
                .windows(12)
                .any(|w| w == b"imcp-assets-")
        );
    }

    #[test]
    fn unsupported_indirect_dependencies_fail_and_local_fragment_styles_survive() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        for body in [
            "<style>@import 'file:///outside.css';</style>",
            "<rect style='fill:url(file:///outside.svg#x)'/>",
            "<rect style='fill:u\\72l(file:///outside.svg#x)'/>",
            "<g xml:base='/outside/'/>",
            "<image href='data:image/svg+xml;base64,PHN2Zz48aW1hZ2UgaHJlZj0iL291dHNpZGUucG5nIi8+PC9zdmc+' />",
            "<use href='/outside.svg#x'/>",
            "<image xmlns:s='http://sodipodi.sourceforge.net/DTD/sodipodi-0.0.dtd' s:absref='/outside.png'/>",
        ] {
            let svg = format!("<svg>{body}</svg>");
            assert!(
                prepare(&workspace, 0, Path::new("input.svg"), svg.as_bytes()).is_err(),
                "{body}"
            );
        }
        for svg in [
            "<!DOCTYPE svg SYSTEM 'file:///outside.dtd'><svg/>",
            "<?xml-stylesheet href='file:///outside.css'?><svg/>",
        ] {
            assert!(prepare(&workspace, 0, Path::new("input.svg"), svg.as_bytes()).is_err());
        }
        let svg = b"<svg><defs><linearGradient id='g'/></defs><rect style='fill:url(&quot;#g&quot;)'/></svg>";
        let prepared = prepare(&workspace, 0, Path::new("input.svg"), svg).unwrap();
        assert!(
            String::from_utf8(prepared.bytes.clone())
                .unwrap()
                .contains("#g")
        );
    }
    #[test]
    fn nested_svg_uses_its_own_base_and_cycles_and_aggregate_bytes_are_bounded() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        std::fs::create_dir(root.path().join("assets")).unwrap();
        let png = b"\x89PNG\r\n\x1a\nowned";
        std::fs::write(root.path().join("owned.png"), png).unwrap();
        std::fs::write(
            root.path().join("assets/leaf.svg"),
            "<svg><image href='../owned.png'/></svg>",
        )
        .unwrap();
        let bytes = b"<svg><image href='assets/leaf.svg'/></svg>";
        let prepared = prepare(&workspace, 0, Path::new("source.svg"), bytes).unwrap();
        assert_eq!(prepared._nested.len(), 1);
        let nested = &prepared._nested[0];
        let doc = xml::parse(&nested.bytes, 4096).unwrap();
        let image = doc
            .get_root_element()
            .unwrap()
            .get_child_elements()
            .remove(0);
        let staged = image.get_property_no_ns("href").unwrap();
        assert_eq!(std::fs::read(&staged).unwrap(), png);
        std::fs::write(
            root.path().join("assets/leaf.svg"),
            "<svg><image href='leaf.svg'/></svg>",
        )
        .unwrap();
        assert!(
            prepare(&workspace, 0, Path::new("source.svg"), bytes)
                .err()
                .unwrap()
                .contains("depth")
        );
        let mut huge = png.to_vec();
        huge.resize(4096, 0);
        std::fs::write(root.path().join("owned.png"), huge).unwrap();
        assert!(
            prepare(
                &workspace,
                0,
                Path::new("source.svg"),
                b"<svg><image href='owned.png'/></svg>"
            )
            .is_err()
        );
    }
    #[test]
    fn css_resources_stage_and_restore_with_shared_limits() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        std::fs::write(
            root.path().join("paint.svg"),
            "<svg><defs><linearGradient id='g'/></defs></svg>",
        )
        .unwrap();
        let bytes = br#"<svg><rect style="fill:u\72l('paint.svg#g')"/></svg>"#;
        let prepared = prepare(&workspace, 0, Path::new("source.svg"), bytes).unwrap();
        assert_eq!(prepared.css_links.len(), 1);
        let staged = String::from_utf8(prepared.bytes.clone()).unwrap();
        assert!(staged.contains("imcp-assets-"));
        let doc = xml::parse(&prepared.bytes, 4096).unwrap();
        prepared.restore(&doc, false, None).unwrap();
        let restored = String::from_utf8(xml::serialize(&doc)).unwrap();
        assert!(restored.contains("paint.svg#g"));
        assert!(!restored.contains("imcp-assets-"));
        assert!(
            prepare(
                &workspace,
                0,
                Path::new("source.svg"),
                br#"<svg><style>@\69mport 'outside.css';</style></svg>"#
            )
            .is_err()
        );
    }
    #[test]
    fn stylesheets_keep_nested_relative_bases_and_cycles_fail_before_engine() {
        let root = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 8192,
            max_output: 8192,
        };
        std::fs::create_dir(root.path().join("sub")).unwrap();
        std::fs::write(root.path().join("base.css"), ".x { fill:red; }").unwrap();
        std::fs::write(
            root.path().join("sub/first.css"),
            "@import '../base.css' all;",
        )
        .unwrap();
        let bytes = b"<svg><style>@import 'sub/first.css';</style></svg>";
        let prepared = prepare(&workspace, 0, Path::new("source.svg"), bytes).unwrap();
        assert_eq!(prepared.css_links.len(), 1);
        assert_eq!(prepared._nested[0]._nested.len(), 1);
        let imported = String::from_utf8(prepared._nested[0].bytes.clone()).unwrap();
        assert!(imported.contains(" all;"));
        let result = xml::parse(&prepared.bytes, 8192).unwrap();
        prepared.restore(&result, false, None).unwrap();
        assert!(!prepared.private_reference(&xml::serialize(&result)));
        assert!(
            String::from_utf8(xml::serialize(&result))
                .unwrap()
                .contains("sub/first.css")
        );
        std::fs::write(root.path().join("base.css"), "@import 'sub/first.css';").unwrap();
        assert!(
            prepare(&workspace, 0, Path::new("source.svg"), bytes)
                .err()
                .unwrap()
                .contains("depth")
        );
    }
}
