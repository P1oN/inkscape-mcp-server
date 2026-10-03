//! Read-only correctness validation over the registered working copy.
use crate::{
    css,
    document::{Registry, elements},
    fonts, inspect,
    style::python_repr,
    xml,
};
use libxml::tree::Document;
use regex::Regex;
use serde_json::{Value, json};
use std::{collections::HashSet, sync::LazyLock};

static REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"url\(\s*['"]?#([^'")\s]+)['"]?\s*\)"#).unwrap());

fn finding(code: &str, severity: &str, message: String, locator: Option<String>) -> Value {
    json!({"code":code,"severity":severity,"message":message,"locator":locator})
}

pub(crate) fn data_size(href: &str) -> Option<usize> {
    let href = href.trim();
    if !href.to_lowercase().starts_with("data:") {
        return None;
    }
    let Some((header, payload)) = href[5..].split_once(',') else {
        return Some(0);
    };
    if header.to_lowercase().contains("base64") {
        let compact = payload
            .chars()
            .filter(|c| !c.is_whitespace() && !matches!(*c, '\u{1c}'..='\u{1f}'))
            .collect::<String>();
        // Count the permissive Python decoder's output without allocating a
        // decoded bitmap. Excess/leading padding is ignored; a completed padded
        // quantum terminates decoding. Invalid padding uses the reference estimate.
        return Some(base64_size(&compact).unwrap_or(compact.chars().count() * 3 / 4));
    }
    let input = payload.as_bytes();
    let mut decoded = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%'
            && i + 2 < input.len()
            && let Ok(hex) = std::str::from_utf8(&input[i + 1..i + 3])
            && let Ok(b) = u8::from_str_radix(hex, 16)
        {
            decoded.push(b);
            i += 3;
        } else {
            decoded.push(input[i]);
            i += 1;
        }
    }
    Some(String::from_utf8_lossy(&decoded).len())
}

fn base64_size(compact: &str) -> Option<usize> {
    if !compact.is_ascii() {
        return None;
    }
    let mut position = 0;
    let mut pads = 0;
    let mut size = 0;
    for byte in compact.bytes() {
        if byte == b'=' {
            if position >= 2 {
                pads += 1;
                if position + pads >= 4 {
                    return Some(size);
                }
            }
        } else if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/') {
            pads = 0;
            position = (position + 1) % 4;
            if position != 1 {
                size += 1;
            }
        }
    }
    if position == 0 { Some(size) } else { None }
}

fn doctype(document: &Document, findings: &mut Vec<Value>) {
    // SAFETY: all pointers belong to this live, immutable libxml2 Document. The
    // entity cast is performed only on an XML_ENTITY_DECL node; no entity content
    // is dereferenced or expanded. System identifiers are never opened.
    unsafe {
        let dtd = (*document.doc_ptr()).intSubset;
        if dtd.is_null() {
            return;
        }
        let mut node = (*dtd).children;
        let mut external = false;
        while !node.is_null() {
            if (*node).type_ == 17 {
                let entity = node.cast::<libxml::bindings::_xmlEntity>();
                let system = (*entity).SystemID;
                if !system.is_null() && *system != 0 {
                    let name = if (*entity).name.is_null() {
                        "?".into()
                    } else {
                        std::ffi::CStr::from_ptr((*entity).name.cast())
                            .to_string_lossy()
                            .into_owned()
                    };
                    findings.push(finding("external_entity","warning",format!("document declares external entity {}; it is NOT expanded (safe parse, no network), but external entities are an XXE vector",python_repr(&name)),Some(name)));
                    external = true;
                }
            }
            node = (*node).next;
        }
        if !external {
            findings.push(finding("doctype_present","info","document declares a DOCTYPE/DTD; SVG normally has none and it is ignored on a safe parse".into(),None));
        }
    }
}

fn python_float(value: f64) -> String {
    let raw = serde_json::to_string(&value).unwrap();
    if let Some((mantissa, exponent)) = raw.split_once('e')
        && let Ok(e) = exponent.parse::<i32>()
    {
        format!(
            "{}e{}{:02}",
            mantissa.trim_end_matches(".0"),
            if e < 0 { "-" } else { "+" },
            e.abs()
        )
    } else {
        raw
    }
}

pub fn document(registry: &Registry, id: &str) -> Result<Value, String> {
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let bytes =
        registry
            .workspace
            .read(entry.root, &entry.working(), registry.workspace.max_input)?;
    let document = xml::parse(&bytes, registry.workspace.max_input)?;
    let root = document
        .get_root_element()
        .ok_or("document could not be parsed safely")?;
    let nodes = elements(root.clone());
    let mut findings = Vec::new();

    let font_info = inspect::resource(registry, id, "fonts")?;
    let referenced = font_info["fonts"].as_array().unwrap();
    if !referenced.is_empty() {
        if let Some(installed) = fonts::installed(registry.workspace.max_output) {
            for f in referenced {
                let family = f["family"].as_str().unwrap().trim();
                if !family.is_empty()
                    && !fonts::generic(family)
                    && !installed.contains(&family.to_lowercase())
                {
                    findings.push(finding(
                        "missing_font",
                        "warning",
                        format!(
                            "referenced font family is not installed: {}",
                            python_repr(family)
                        ),
                        Some(family.into()),
                    ));
                }
            }
        } else {
            findings.push(finding(
                "font_check_skipped",
                "info",
                "font availability not checked: fontconfig unavailable".into(),
                None,
            ));
        }
    }
    let rules = css::rules(&nodes);
    let mut seen_text = HashSet::new();
    for n in &nodes {
        if !matches!(
            n.get_name().as_str(),
            "text" | "tspan" | "textPath" | "tref" | "flowRoot" | "flowPara" | "flowSpan"
        ) {
            continue;
        }
        if n.get_parent()
            .is_some_and(|p| seen_text.contains(&p.node_ptr()))
            && css::own(n, &rules, "font-family").is_none()
        {
            continue;
        }
        let Some(value) = css::inherited(n, &rules, "font-family") else {
            continue;
        };
        let Some(family) = value
            .split(',')
            .map(|s| s.trim().trim_matches(['\'', '"']).trim())
            .find(|s| !s.is_empty())
        else {
            continue;
        };
        let text = inspect::text(n);
        if text.trim().is_empty() {
            continue;
        }
        if let Some(missing) = fonts::uncovered(family, &text, registry.workspace.max_output)
            && !missing.is_empty()
        {
            seen_text.insert(n.node_ptr());
            let suffix = fonts::suggest(&missing, registry.workspace.max_output)
                .map(|s| format!("; try {}", python_repr(&s)))
                .unwrap_or_default();
            findings.push(finding("missing_glyphs","warning",format!("font {} cannot render {} in this text; the saved SVG names a family that would show tofu on a renderer without font substitution{}",python_repr(family),python_repr(&missing),suffix),n.get_property_no_ns("id").filter(|s| !s.is_empty()).or(Some(family.into()))));
        }
    }
    let assets = inspect::resource(registry, id, "assets")?;
    for a in assets["assets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["external"] == true)
    {
        let href = a["href"].as_str().unwrap();
        findings.push(finding(
            "external_asset",
            "warning",
            format!(
                "external {} reference may break if the file moves: {href}",
                a["kind"].as_str().unwrap()
            ),
            Some(href.into()),
        ));
    }
    let mut defined = indexmap::IndexMap::<String, usize>::new();
    let mut references = indexmap::IndexSet::new();
    for n in &nodes {
        let href = n
            .get_property_ns("href", "http://www.w3.org/1999/xlink")
            .filter(|s| !s.is_empty())
            .or_else(|| n.get_property_no_ns("href"));
        if n.get_name() == "image"
            && let Some(href) = &href
            && let Some(size) = data_size(href)
            && size > 5 * 1024 * 1024
        {
            findings.push(finding("large_raster","warning",format!("embedded raster image is large ({:.1} MiB > 5 MiB); consider linking or downscaling",size as f64/(1024.0*1024.0)),n.get_property_no_ns("id").filter(|s| !s.is_empty()).or_else(||Some(href.chars().take(32).collect()))));
        }
        if let Some(id) = n.get_property_no_ns("id").filter(|s| !s.is_empty()) {
            *defined.entry(id).or_default() += 1;
        }
        if let Some(href) = href
            && let Some(fragment) = href
                .strip_prefix('#')
                .map(str::trim)
                .filter(|s| !s.is_empty())
        {
            references.insert(fragment.to_owned());
        }
        for attr in [
            "fill",
            "stroke",
            "mask",
            "clip-path",
            "filter",
            "style",
            "marker-start",
            "marker-mid",
            "marker-end",
        ] {
            if let Some(v) = n.get_property_no_ns(attr) {
                for c in REF.captures_iter(&v) {
                    references.insert(c[1].trim().to_owned());
                }
            }
        }
    }
    for (id, count) in &defined {
        if *count > 1 {
            findings.push(finding(
                "duplicate_id",
                "error",
                format!("id is used by {count} elements; ids must be document-unique"),
                Some(id.clone()),
            ));
        }
    }
    for reference in references {
        if !defined.contains_key(&reference) {
            findings.push(finding(
                "missing_id",
                "error",
                format!(
                    "reference points at id {} but no element defines it",
                    python_repr(&reference)
                ),
                Some(reference),
            ));
        }
    }
    let summary = registry.summary(id)?;
    if let Some(viewbox) = summary["viewbox"].as_array() {
        if viewbox[2].as_f64().unwrap() <= 0.0 || viewbox[3].as_f64().unwrap() <= 0.0 {
            let locator = format!(
                "[{}]",
                viewbox
                    .iter()
                    .map(|n| python_float(n.as_f64().unwrap()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            findings.push(finding(
                "viewbox_invalid",
                "error",
                "viewBox width/height must be positive".into(),
                Some(locator),
            ));
        }
    } else if let Some(raw) = root.get_property_no_ns("viewBox") {
        findings.push(finding(
            "viewbox_invalid",
            "error",
            "viewBox is present but not four valid numbers".into(),
            Some(raw),
        ));
    } else {
        findings.push(finding(
            "viewbox_missing",
            "warning",
            "document has no viewBox; coordinate scaling is undefined".into(),
            None,
        ));
    }
    doctype(&document, &mut findings);
    let errors = findings.iter().filter(|f| f["severity"] == "error").count();
    let warnings = findings
        .iter()
        .filter(|f| f["severity"] == "warning")
        .count();
    Ok(
        json!({"doc_id":id,"ok":errors==0,"findings":findings,"error_count":errors,"warning_count":warnings}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_sizes_and_doctype_are_observable_without_fetching() {
        assert_eq!(data_size("data:image/png;base64,YWJj"), Some(3));
        assert_eq!(data_size("DATA:text/plain,%C3%A9"), Some(2));
        assert_eq!(data_size("data:image/png;base64,YQ"), Some(1));
        assert_eq!(data_size("data:image/png;base64,YWJj==YWJj"), Some(6));
        assert_eq!(data_size("data:image/png;base64,YQ==YWJj"), Some(1));
        assert_eq!(data_size("data:image/png;base64,YQ=Q=="), Some(2));
        assert_eq!(data_size("https://example.com/file"), None);
        let d = xml::parse(
            br#"<!DOCTYPE svg [<!ENTITY e SYSTEM 'file:///etc/passwd'>]><svg>&e;</svg>"#,
            4096,
        )
        .unwrap();
        let mut findings = Vec::new();
        doctype(&d, &mut findings);
        assert_eq!(findings[0]["code"], "external_entity");
        assert_eq!(findings[0]["locator"], "e");
    }
}
