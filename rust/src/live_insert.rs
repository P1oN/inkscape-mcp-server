//! Native preflight for the fixed managed insertion effect. The helper receives
//! the original fragment and repeats this validation while applying one Undo step.
use crate::live_socket::Error;
use libxml::tree::NodeType;
use std::collections::HashMap;
use std::sync::LazyLock;
const SVG: &str = "http://www.w3.org/2000/svg";
const CAP: usize = 1024 * 1024;
static URL: LazyLock<regex::Regex> = LazyLock::new(|| {
    // Python \w means Unicode alphanumeric plus underscore; Rust's \w also
    // includes combining marks, so use explicit letter/number classes.
    regex::Regex::new(r##"(?i:url)\([\s\x1c-\x1f]*['"]?(#[\p{L}\p{N}_.:-]+)['"]?[\s\x1c-\x1f]*\)"##)
        .unwrap()
});
fn fail(message: &'static str) -> Error {
    Error::Protocol(message)
}
pub fn plan(fragment: &str, prefix: &str) -> Result<Vec<String>, Error> {
    let token = prefix.strip_prefix("mcp_").unwrap_or("");
    if token.len() != 32
        || !token
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(fail("invalid insertion id"));
    }
    if fragment
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
        .is_empty()
        || fragment.len() > CAP
    {
        return Err(fail("insertion fragment is empty or too large"));
    }
    let source = format!("<svg xmlns=\"{SVG}\">{fragment}</svg>");
    let document = crate::xml::parse(source.as_bytes(), CAP + 100)
        .map_err(|_| fail("malformed insertion XML"))?;
    let root = document
        .get_root_element()
        .ok_or(fail("malformed insertion XML"))?;
    let mut nodes = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(node) = stack.pop() {
        if matches!(
            node.get_type(),
            Some(NodeType::TextNode | NodeType::CDataSectionNode)
        ) {
            continue;
        }
        nodes.push(node.clone());
        if nodes.len() > 10_000 {
            return Err(fail("too many insertion elements"));
        }
        stack.extend(node.get_child_nodes().into_iter().rev());
    }
    let mut ids = HashMap::new();
    let mut ordered = vec![prefix.to_string()];
    for node in &nodes {
        if !node.is_element_node() {
            return Err(fail("insertion contains non-element content"));
        }
        if node.get_namespace().map(|ns| ns.get_href()).as_deref() != Some(SVG)
            || !matches!(
                node.get_name().as_str(),
                "svg"
                    | "g"
                    | "defs"
                    | "rect"
                    | "circle"
                    | "ellipse"
                    | "path"
                    | "line"
                    | "polygon"
                    | "polyline"
                    | "text"
                    | "tspan"
                    | "use"
                    | "linearGradient"
                    | "radialGradient"
                    | "stop"
                    | "clipPath"
                    | "mask"
                    | "pattern"
                    | "title"
                    | "desc"
            )
        {
            return Err(fail("unsupported SVG insertion element"));
        }
        if let Some(id) = node.get_property_no_ns("id").filter(|id| !id.is_empty()) {
            if ids.contains_key(&id) {
                return Err(fail("duplicate insertion id"));
            }
            let new = format!("{prefix}_{}", ids.len());
            ordered.push(new.clone());
            ids.insert(id, new);
        }
    }
    for node in &nodes {
        for ((name, _), value) in node.get_properties_ns() {
            if name.to_lowercase().starts_with("on") || name == "base" {
                return Err(fail("event attributes are unsupported"));
            }
            if name == "id" {
                // Python raises an uncaught KeyError for an empty ID. Refuse it
                // deterministically before any request, preserving the document.
                if !ids.contains_key(&value) {
                    return Err(fail("''"));
                }
            } else if name == "href" {
                if !value
                    .strip_prefix('#')
                    .is_some_and(|id| ids.contains_key(id))
                {
                    return Err(fail("href must reference an id in the fragment"));
                }
            } else if value.to_lowercase().contains("url") || name == "style" {
                if value.contains(['\\', '@']) {
                    return Err(fail("unsupported CSS insertion value"));
                }
                for found in URL.captures_iter(&value) {
                    if !ids.contains_key(&found[1][1..]) {
                        return Err(fail("paint reference must be internal to the fragment"));
                    }
                }
                if URL.replace_all(&value, "").to_lowercase().contains("url") {
                    return Err(fail("external or malformed CSS URL"));
                }
            }
        }
    }
    if root.get_child_elements().is_empty() {
        return Err(fail("insertion has no objects"));
    }
    Ok(ordered)
}
/// Confirm a direct root group, rather than a substring in unrelated text.
#[cfg(test)]
pub fn confirmed(svg: &str, prefix: &str, ids: &[String], cap: usize) -> Result<bool, Error> {
    let document = crate::xml::parse(svg.as_bytes(), cap).map_err(|_| Error::Uncertain)?;
    let root = document.get_root_element().ok_or(Error::Uncertain)?;
    let groups: Vec<_> = root
        .get_child_elements()
        .into_iter()
        .filter(|n| {
            n.get_name() == "g"
                && n.get_namespace().map(|ns| ns.get_href()).as_deref() == Some(SVG)
                && n.get_property_no_ns("id").as_deref() == Some(prefix)
        })
        .collect();
    if groups.len() != 1 {
        return Ok(false);
    }
    let actual: Vec<_> = crate::document::elements(groups[0].clone())
        .iter()
        .filter_map(|n| n.get_property_no_ns("id").filter(|id| !id.is_empty()))
        .collect();
    Ok(actual == ids)
}
/// Inkscape assigns IDs to anonymous elements while adopting extension output.
/// Accept those only at the original anonymous positions, with the exact shape
/// and every remapped ID still present. A helper acknowledgement alone is insufficient.
pub fn confirmed_fragment(
    svg: &str,
    prefix: &str,
    ids: &[String],
    fragment: &str,
    cap: usize,
) -> Result<bool, Error> {
    if plan(fragment, prefix).map_err(|_| Error::Uncertain)? != ids {
        return Err(Error::Uncertain);
    }
    let original = crate::xml::parse(
        format!("<svg xmlns=\"{SVG}\">{fragment}</svg>").as_bytes(),
        CAP + 100,
    )
    .map_err(|_| Error::Uncertain)?;
    let original_root = original.get_root_element().ok_or(Error::Uncertain)?;
    let current = crate::xml::parse(svg.as_bytes(), cap).map_err(|_| Error::Uncertain)?;
    let root = current.get_root_element().ok_or(Error::Uncertain)?;
    let groups: Vec<_> = root
        .get_child_elements()
        .into_iter()
        .filter(|node| {
            node.get_name() == "g"
                && node.get_namespace().map(|ns| ns.get_href()).as_deref() == Some(SVG)
                && node.get_property_no_ns("id").as_deref() == Some(prefix)
        })
        .collect();
    if groups.len() != 1 {
        return Ok(false);
    }
    // Reject duplicate IDs throughout the document, including collisions with
    // original artwork outside the inserted group.
    let mut seen = std::collections::HashSet::new();
    for node in crate::document::elements(root) {
        if let Some(id) = node.get_property_no_ns("id").filter(|id| !id.is_empty())
            && !seen.insert(id)
        {
            return Ok(false);
        }
    }
    fn shape(root: libxml::tree::Node) -> Vec<(usize, libxml::tree::Node)> {
        let mut result = Vec::new();
        let mut stack = vec![(0, root)];
        while let Some((depth, node)) = stack.pop() {
            stack.extend(
                node.get_child_elements()
                    .into_iter()
                    .rev()
                    .map(|child| (depth + 1, child)),
            );
            result.push((depth, node));
        }
        result
    }
    let expected = shape(original_root);
    let actual = shape(groups[0].clone());
    if expected.len() != actual.len() {
        return Ok(false);
    }
    let mut identified = 0;
    for (index, ((depth, expected), (actual_depth, actual))) in
        expected.iter().zip(&actual).enumerate()
    {
        let tag = if index == 0 {
            "g"
        } else {
            &expected.get_name()
        };
        if depth != actual_depth
            || actual.get_name() != tag
            || actual.get_namespace().map(|ns| ns.get_href()).as_deref() != Some(SVG)
        {
            return Ok(false);
        }
        let expected_id = if index == 0 {
            Some(prefix.to_string())
        } else if expected.get_property_no_ns("id").is_some() {
            let id = format!("{prefix}_{identified}");
            identified += 1;
            Some(id)
        } else {
            None
        };
        if let Some(id) = expected_id
            && actual.get_property_no_ns("id").as_deref() != Some(id.as_str())
        {
            return Ok(false);
        }
    }
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_anonymous_ids_require_the_original_shape_and_all_remapped_ids() {
        let prefix = format!("mcp_{}", "a".repeat(32));
        let fragment = "<g><rect id=\"rectangle\"/><circle/></g>";
        let ids = plan(fragment, &prefix).unwrap();
        let document = |body: &str| format!("<svg xmlns=\"{SVG}\">{body}</svg>");
        let wrapped = |body: &str| format!("<g id=\"{prefix}\">{body}</g>");
        let rect = format!("<rect id=\"{prefix}_0\"/>");
        for body in [
            format!("<g>{rect}<circle/></g>"),
            format!("<g id=\"g1\">{rect}<circle id=\"circle1\"/></g>"),
        ] {
            assert!(
                confirmed_fragment(&document(&wrapped(&body)), &prefix, &ids, fragment, 4096)
                    .unwrap()
            );
        }
        for body in [
            format!("<g id=\"g1\">{rect}<circle id=\"g1\"/></g>"),
            format!("<g id=\"g1\">{rect}<circle/><rect id=\"extra\"/></g>"),
            format!("<g id=\"g1\">{rect}</g><circle/>"),
            format!("<g id=\"g1\"><circle/>{rect}</g>"),
            "<g id=\"g1\"><rect id=\"wrong\"/><circle/></g>".into(),
        ] {
            assert!(
                !confirmed_fragment(&document(&wrapped(&body)), &prefix, &ids, fragment, 4096)
                    .unwrap()
            );
        }
        let valid = wrapped(&format!("<g id=\"g1\">{rect}<circle/></g>"));
        assert!(
            !confirmed_fragment(
                &document(&format!("<rect id=\"g1\"/>{valid}")),
                &prefix,
                &ids,
                fragment,
                4096
            )
            .unwrap()
        );
        assert!(
            !confirmed_fragment(
                &document(&format!("<g>{valid}</g>")),
                &prefix,
                &ids,
                fragment,
                4096
            )
            .unwrap()
        );
    }
    #[test]
    fn compiled_python_insertion_preflight_ids_and_refusals_match() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../migration/contracts/insertion-plan-cases.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let actual = plan(
                case["fragment"].as_str().unwrap(),
                case["prefix"].as_str().unwrap(),
            );
            if let Some(ids) = case["expected"].get("ids") {
                assert_eq!(serde_json::json!(actual.unwrap()), *ids, "{case}");
            } else {
                match actual {
                    Err(Error::Protocol(message)) => assert_eq!(
                        message,
                        case["expected"]["error"].as_str().unwrap(),
                        "{case}"
                    ),
                    other => panic!("unexpected result {other:?}: {case}"),
                }
            }
        }
    }
    #[test]
    fn input_and_element_limits_are_bounded_before_activation() {
        let prefix = format!("mcp_{}", "a".repeat(32));
        assert_eq!(
            plan(&"x".repeat(CAP + 1), &prefix),
            Err(fail("insertion fragment is empty or too large"))
        );
        assert!(plan(&"<rect/>".repeat(9999), &prefix).is_ok());
        assert_eq!(
            plan(&"<rect/>".repeat(10_000), &prefix),
            Err(fail("too many insertion elements"))
        );
        assert_eq!(
            plan(
                "<g id=\"e\u{301}\"/><rect fill=\"url(#e\u{301})\"/>",
                &prefix
            ),
            Err(fail("external or malformed CSS URL"))
        );
    }
    #[test]
    fn confirmation_requires_one_root_group_and_exact_planned_ids() {
        let ids = vec!["mcp_token".into(), "mcp_token_0".into()];
        let document = |body: &str| format!("<svg xmlns=\"{SVG}\">{body}</svg>");
        assert!(
            confirmed(
                &document("<g id=\"mcp_token\"><rect id=\"mcp_token_0\"/></g>"),
                "mcp_token",
                &ids,
                4096
            )
            .unwrap()
        );
        for body in [
            "<text>mcp_token</text>",
            "<rect id=\"mcp_token\"/>",
            "<g id=\"mcp_token\"/>",
            "<g><g id=\"mcp_token\"><rect id=\"mcp_token_0\"/></g></g>",
            "<g id=\"mcp_token\"><rect id=\"mcp_token_0\"/><rect id=\"unexpected\"/></g>",
            "<g id=\"mcp_token\"><rect id=\"mcp_token_0\"/></g><g id=\"mcp_token\"/>",
        ] {
            assert!(!confirmed(&document(body), "mcp_token", &ids, 4096).unwrap());
        }
        assert_eq!(
            confirmed("<broken", "mcp_token", &ids, 4096),
            Err(Error::Uncertain)
        );
    }
}
