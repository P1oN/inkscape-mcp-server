//! Structural advice is read-only and does not change validity or quality score.
use crate::{
    arguments,
    document::{INKSCAPE_NS, elements},
};
use libxml::tree::Node;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
fn threshold(args: &Value, key: &str, default: usize, max: usize) -> Result<usize, String> {
    let Some(v) = args.get(key) else {
        return Ok(default);
    };
    let n = arguments::number(v)
        .filter(|n| n.is_finite() && n.fract() == 0. && *n >= 1. && *n <= max as f64)
        .ok_or("invalid editability options")?;
    Ok(n as usize)
}
pub fn analyze(root: Node, opts: &Value) -> Result<Value, String> {
    let default = json!({});
    let opts = if opts.is_null() { &default } else { opts };
    if !opts.is_object() {
        return Err("invalid editability options".into());
    }
    let roles = crate::authoring_analysis::roles(opts)?;
    let enabled = arguments::boolean(opts, "enabled", true)?;
    let labels = arguments::boolean(opts, "check_labels", true)?;
    let layers_limit = threshold(opts, "layer_advisory_threshold", 12, 10000)?;
    let depth_limit = threshold(opts, "max_group_depth", 6, 100)?;
    let fragment_limit = threshold(opts, "fragmentation_threshold", 30, 10000)?;
    let semantic = if let Some(v) = opts.get("semantic_group_ids") {
        let list = v
            .as_array()
            .filter(|l| l.len() <= 200)
            .ok_or("invalid editability options")?;
        list.iter()
            .map(|v| v.as_str().ok_or("invalid editability options"))
            .collect::<Result<HashSet<_>, _>>()?
    } else {
        HashSet::new()
    };
    let nodes = elements(root);
    let mut depths = HashMap::new();
    let mut groups = vec![];
    for n in &nodes {
        let parent = n
            .get_parent()
            .and_then(|p| depths.get(&(p.node_ptr() as usize)))
            .copied()
            .unwrap_or(0usize);
        let depth = parent + usize::from(n.get_name() == "g");
        depths.insert(n.node_ptr() as usize, depth);
        if n.get_name() == "g" {
            groups.push((n, depth));
        }
    }
    let layers = groups
        .iter()
        .filter(|(n, _)| n.get_property_ns("groupmode", INKSCAPE_NS).as_deref() == Some("layer"))
        .count();
    let fragments = groups
        .iter()
        .filter(|(n, _)| {
            n.get_child_nodes()
                .iter()
                .filter(|n| {
                    !matches!(
                        n.get_type(),
                        Some(
                            libxml::tree::NodeType::TextNode
                                | libxml::tree::NodeType::CDataSectionNode
                        )
                    )
                })
                .count()
                == 1
        })
        .count();
    let mut advice = vec![];
    let mut total = 0usize;
    let mut add = |code: &str, kind: &str, message: &str, node: Option<&Node>| {
        total += 1;
        if advice.len() < 200 {
            advice.push(json!({"code":code,"kind":kind,"object_id":node.and_then(|n|n.get_property_no_ns("id")),"message":message}));
        }
    };
    if enabled {
        for (n, depth) in &groups {
            let id = n.get_property_no_ns("id");
            let designated = id.as_deref().is_some_and(|id| semantic.contains(id));
            if labels
                && n.get_property_ns("label", INKSCAPE_NS)
                    .is_none_or(|s| s.trim().is_empty())
                && (semantic.is_empty() || designated)
            {
                add(
                    "missing_group_label",
                    "recommendation",
                    "Consider a readable label if this group represents a selectable object.",
                    Some(n),
                );
            }
            if designated && n.get_property_ns("groupmode", INKSCAPE_NS).as_deref() == Some("layer")
            {
                add(
                    "semantic_layer",
                    "recommendation",
                    "Designated semantic object is a layer; consider ordinary group selection.",
                    Some(n),
                );
            }
            if *depth > depth_limit {
                add(
                    "deep_groups",
                    "recommendation",
                    "Consider simplifying this nesting if unnecessary.",
                    Some(n),
                );
            }
            if ["style", "opacity", "filter", "mask", "clip-path"]
                .iter()
                .any(|key| n.get_property_no_ns(key).is_some())
            {
                add(
                    "structural_style_risk",
                    "observation",
                    "Container has effects; moving children may change inheritance or compositing.",
                    Some(n),
                );
            }
        }
        if layers > layers_limit {
            add(
                "many_layers",
                "recommendation",
                "Layer count exceeds your threshold; check whether organization is intentional.",
                None,
            );
        }
        if fragments > fragment_limit {
            add(
                "fragmentation",
                "recommendation",
                "Many groups have one child; review wrapper structure if selection is cumbersome.",
                None,
            );
        }
        if nodes.iter().any(|n| n.get_name() == "style") {
            add(
                "stylesheet_structure_risk",
                "observation",
                "Stylesheet selectors may depend on ancestry, IDs or container metadata.",
                None,
            );
        }
        add(
            "paint_order_risk",
            "observation",
            "Reparenting can change paint order; preservation cannot be inferred from group names.",
            None,
        );
    }
    Ok(
        json!({"authoring": if enabled {crate::authoring_analysis::analyze(&nodes, &roles)} else {json!({"findings":[],"truncated":false,"scope":"Disabled."})},"group_count":groups.len(),"layer_count":layers,"max_group_depth":groups.iter().map(|(_,d)|*d).max().unwrap_or(0),"single_child_groups":fragments,"advice":advice,"truncated":total>200,"note":"Advice is optional; names/counts cannot establish semantics or tracing."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_advice_preserves_counts_comments_and_disabled_readonly_tree() {
        let source = format!(
            "<svg xmlns='http://www.w3.org/2000/svg'><g><!--one--></g>{}</svg>",
            "<g><g><rect/></g></g>".repeat(110)
        );
        let doc = crate::xml::parse(source.as_bytes(), 10000).unwrap();
        let before = crate::xml::serialize(&doc);
        let root = doc.get_root_element().unwrap();
        let report = analyze(root.clone(), &json!({})).unwrap();
        assert_eq!(report["group_count"], 221);
        assert_eq!(report["single_child_groups"], 221);
        assert_eq!(report["max_group_depth"], 2);
        assert_eq!(report["advice"].as_array().unwrap().len(), 200);
        assert_eq!(report["truncated"], true);
        let disabled = analyze(root, &json!({"enabled":false})).unwrap();
        assert_eq!(disabled["group_count"], 221);
        assert_eq!(disabled["advice"], json!([]));
        assert_eq!(disabled["truncated"], false);
        assert_eq!(crate::xml::serialize(&doc), before);
    }
}
