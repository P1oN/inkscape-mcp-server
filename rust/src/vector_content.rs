//! Whole-document vector inventory; external or opaque resources never count as passing.
use libxml::tree::Node;
use serde_json::{Value, json};

pub fn analyze(nodes: &[Node]) -> Value {
    let mut findings = vec![];
    let mut total = 0usize;
    let mut rasters = 0usize;
    let mut unknown = 0usize;
    let mut add = |node: &Node, code: &str, known: bool, reason: &str| {
        total += 1;
        if known {
            rasters += 1;
        } else {
            unknown += 1;
        }
        if findings.len() < 200 {
            findings.push(json!({"object_id":node.get_property_no_ns("id"),"code":code,"certainty":if known {"known"}else{"unknown"},"reason":reason}));
        }
    };
    if nodes.len() > 20000 {
        return json!({"status":"unknown","findings":[{"code":"vector_content_limit","certainty":"unknown","reason":"Element limit exceeded."}],"truncated":true});
    }
    let mut ids = std::collections::HashMap::<String, usize>::new();
    for node in nodes {
        if let Some(id) = node.get_property_no_ns("id") {
            *ids.entry(id).or_default() += 1;
        }
    }
    let mut visits = 0usize;
    if let Some(root) = nodes.first() {
        let mut top = root.clone();
        while let Some(previous) = top.get_prev_sibling() {
            top = previous;
        }
        loop {
            visits += 1;
            if visits > 40000 {
                add(
                    root,
                    "vector_content_limit",
                    false,
                    "Non-element traversal limit exceeded.",
                );
                break;
            }
            if matches!(
                top.get_type(),
                Some(libxml::tree::NodeType::PiNode | libxml::tree::NodeType::DTDNode)
            ) {
                add(
                    root,
                    "document_content_unknown",
                    false,
                    "Processing instructions/DTD require preparation.",
                );
            }
            if let Some(next) = top.get_next_sibling() {
                top = next;
            } else {
                break;
            }
        }
    }
    for node in nodes {
        for child in node.get_child_nodes() {
            visits += 1;
            if visits > 40000 {
                add(
                    node,
                    "vector_content_limit",
                    false,
                    "Non-element traversal limit exceeded.",
                );
                break;
            }
            if matches!(
                child.get_type(),
                Some(libxml::tree::NodeType::PiNode | libxml::tree::NodeType::EntityRefNode)
            ) {
                add(
                    node,
                    "document_content_unknown",
                    false,
                    "Processing instructions/entities require preparation.",
                );
            }
        }
        let name = node.get_name();
        if ["image", "feImage"].contains(&name.as_str()) {
            let href = node
                .get_property_no_ns("href")
                .filter(|s| !s.is_empty())
                .or_else(|| node.get_property_ns("href", "http://www.w3.org/1999/xlink"))
                .unwrap_or_default();
            let lower = href.to_ascii_lowercase();
            let mime = lower
                .strip_prefix("data:")
                .and_then(|s| s.split([';', ',']).next());
            let known = matches!(
                mime,
                Some(
                    "image/png"
                        | "image/jpeg"
                        | "image/gif"
                        | "image/webp"
                        | "image/bmp"
                        | "image/tiff"
                        | "image/avif"
                        | "image/x-icon"
                )
            );
            // Linked extensions are not proof of content type. Embedded SVG may contain rasters.
            add(
                node,
                if known {
                    "raster_image"
                } else {
                    "image_content_unknown"
                },
                known,
                if known {
                    "Embedded raster resource is present, including hidden/resource geometry."
                } else {
                    "Image resource requires inspection; linked, local-fragment and embedded SVG content is not certified vector-only."
                },
            );
        } else if matches!(
            name.as_str(),
            "use" | "linearGradient" | "radialGradient" | "pattern" | "textPath" | "mpath"
        ) {
            let href = node
                .get_property_no_ns("href")
                .or_else(|| node.get_property_ns("href", "http://www.w3.org/1999/xlink"))
                .unwrap_or_default();
            let local = href
                .strip_prefix('#')
                .filter(|id| !id.is_empty() && !id.contains('%'));
            if (!href.is_empty() || name == "use") && local.is_none_or(|id| ids.get(id) != Some(&1))
            {
                add(
                    node,
                    "instance_content_unknown",
                    false,
                    "Resource/instance target is external, missing, encoded or ambiguous.",
                );
            }
        } else if matches!(
            name.as_str(),
            "foreignObject"
                | "script"
                | "animate"
                | "animateTransform"
                | "animateMotion"
                | "set"
                | "discard"
        ) {
            add(
                node,
                "dynamic_content_unknown",
                false,
                "Foreign or dynamic content requires manual review.",
            );
        }
        if name == "style" {
            // CSS can introduce external resources or imports. Conservatively require preparation.
            add(
                node,
                "stylesheet_content_unknown",
                false,
                "Stylesheet requires preparation for vector-only certification.",
            );
        }
        for ((key, _), value) in node.get_properties_ns() {
            if key.to_ascii_lowercase().starts_with("on") {
                add(
                    node,
                    "dynamic_content_unknown",
                    false,
                    "Event handlers require preparation.",
                );
            }
            if matches!(
                key.as_str(),
                "style"
                    | "fill"
                    | "stroke"
                    | "filter"
                    | "mask"
                    | "clip-path"
                    | "marker-start"
                    | "marker-mid"
                    | "marker-end"
            ) {
                let result = crate::css_assets::rewrite(&value, |url| {
                    if url
                        .strip_prefix('#')
                        .is_none_or(|id| ids.get(id) != Some(&1))
                        || url.contains('%')
                    {
                        return Err("External/encoded paint resource".into());
                    }
                    Ok(None)
                });
                if result.is_err() {
                    add(
                        node,
                        "paint_content_unknown",
                        false,
                        "External, encoded or unsupported paint resource requires preparation.",
                    );
                }
            }
        }
    }
    json!({"status":if rasters>0{"failed"}else if unknown>0{"unknown"}else{"passed"},"raster_resource_count":rasters,"unknown_count":unknown,"findings":findings,"truncated":total>200,"scope":"Whole document, including hidden objects and defs. No resource fetching, image removal or tracing. Linked image content is unknown."})
}
pub fn require(bytes: &[u8], limit: usize) -> Result<(), String> {
    let document = crate::xml::parse(bytes, limit)?;
    let nodes = crate::document::elements(document.get_root_element().ok_or("SVG root missing")?);
    let report = analyze(&nodes);
    if report["status"] != "passed" {
        return Err(format!(
            "vector_only requires a passing whole-document check; status={}",
            report["status"].as_str().unwrap()
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn review(body: &str) -> Value {
        let doc = crate::xml::parse(
            format!("<svg xmlns='http://www.w3.org/2000/svg'>{body}</svg>").as_bytes(),
            100000,
        )
        .unwrap();
        let before = crate::xml::serialize(&doc);
        let result = analyze(&crate::document::elements(doc.get_root_element().unwrap()));
        assert_eq!(before, crate::xml::serialize(&doc));
        result
    }
    #[test]
    fn xml_stylesheet_and_external_resource_inheritance_are_unknown() {
        let doc=crate::xml::parse(b"<?xml-stylesheet href='external.css'?><svg xmlns='http://www.w3.org/2000/svg'><rect/></svg>",4096).unwrap();
        assert_eq!(
            analyze(&crate::document::elements(doc.get_root_element().unwrap()))["status"],
            "unknown"
        );
        assert_eq!(
            review(
                "<defs><linearGradient id='g' href='external.svg#g'/></defs><rect fill='url(#g)'/>"
            )["status"],
            "unknown"
        );
        assert_eq!(review("<rect fill='url(#missing)'/>")["status"], "unknown");
    }
    #[test]
    fn inventory_covers_hidden_defs_instances_and_opaque_resources() {
        assert_eq!(review("<path id='p'/><use href='#p'/>")["status"], "passed");
        assert_eq!(
            review(
                "<defs><image id='r' href='data:image/png;base64,YQ=='/></defs><use href='#r'/>"
            )["status"],
            "failed"
        );
        assert_eq!(
            review("<image style='display:none' href='picture.png'/>")["status"],
            "unknown"
        );
        assert_eq!(
            review("<filter><feImage href='data:image/jpeg;base64,YQ=='/></filter>")["status"],
            "failed"
        );
        for body in [
            "<image href='data:image/svg+xml,%3Csvg/%3E'/>",
            "<use href='external.svg#p'/>",
            "<use href='#missing'/>",
            "<style>@import 'x.css';</style>",
            "<rect fill='url(external.svg#p)'/>",
            "<rect onload='change()'/>",
            "<animateMotion/>",
        ] {
            assert_eq!(review(body)["status"], "unknown");
        }
    }
}
