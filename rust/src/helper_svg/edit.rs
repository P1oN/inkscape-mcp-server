//! Pure live edit planning. Plans own data, never mutate the parsed source.
//! Native selection/context checks and native Undo publication belong to the consumer.
use super::{affine, elements, valid_nonce};
use libxml::tree::{Node, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
const INK: &str = "http://www.inkscape.org/namespaces/inkscape";
const SOD: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd";
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Style,
    Text,
    Duplicate,
    Delete,
    Group,
    Ungroup,
    Raise,
    Lower,
    Front,
    Back,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub nonce: String,
    pub operation: Operation,
    pub selection: Vec<String>,
    #[serde(default)]
    pub style: BTreeMap<String, String>,
    #[serde(default)]
    pub transform: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}
/// Fixed semantic instructions. IDs locate nodes in the captured source, never XPath/code.
#[derive(Debug, Serialize, PartialEq)]
pub enum Step {
    Style {
        id: String,
        values: BTreeMap<String, String>,
        transform: Option<String>,
    },
    Text {
        id: String,
        value: String,
    },
    Duplicate {
        ids: Vec<String>,
        remap: BTreeMap<String, String>,
    },
    Delete {
        ids: Vec<String>,
    },
    Group {
        ids: Vec<String>,
        id: String,
    },
    Ungroup {
        id: String,
        children: Vec<(String, String)>,
    },
    /// Complete element sibling order; comments/PIs/tails are not paintable anchors.
    Order {
        parent_path: Vec<usize>,
        indices: Vec<usize>,
    },
}
#[derive(Debug, Serialize, PartialEq)]
pub struct Plan {
    pub steps: Vec<Step>,
    pub affected_ids: Vec<String>,
}
impl Plan {
    pub fn changed(&self) -> bool {
        !self.steps.is_empty()
    }
}
// Count JSON bytes without allocating a serialized copy before enforcing its cap.
fn json_within_cap(value: &impl Serialize, cap: usize) -> bool {
    struct Counter {
        count: usize,
        cap: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.count = self.count.saturating_add(bytes.len());
            if self.count > self.cap {
                return Err(std::io::Error::other("JSON size cap"));
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { count: 0, cap }, value).is_ok()
}
fn id(node: &Node) -> String {
    node.get_property_no_ns("id").unwrap_or_default()
}
fn chain(node: &Node) -> Vec<Node> {
    let mut result = vec![node.clone()];
    let mut p = node.get_parent().filter(Node::is_element_node);
    while let Some(n) = p {
        p = n.get_parent().filter(Node::is_element_node);
        result.push(n);
    }
    result
}
fn drawable(n: &Node) -> bool {
    matches!(
        n.get_name().as_str(),
        "g" | "rect"
            | "circle"
            | "ellipse"
            | "path"
            | "line"
            | "polygon"
            | "polyline"
            | "text"
            | "use"
            | "image"
    )
}
fn paintable(n: &Node) -> bool {
    drawable(n)
        || matches!(
            n.get_name().as_str(),
            "svg" | "a" | "switch" | "foreignObject" | "flowRoot"
        )
}
fn refs(all: &[Node], ids: &HashSet<String>, excluded: &[Node]) -> Result<(), &'static str> {
    let work = all
        .iter()
        .filter(|n| !excluded.contains(n))
        .try_fold(0usize, |used, n| {
            let bytes = n
                .get_properties_ns()
                .values()
                .map(String::len)
                .sum::<usize>()
                .saturating_add(if n.get_name() == "style" {
                    n.get_content().len()
                } else {
                    0
                });
            let used = used.saturating_add(bytes.saturating_mul(ids.len()));
            if used > 64 * 1024 * 1024 {
                Err("edit reference scan exceeds work cap")
            } else {
                Ok(used)
            }
        })?;
    let _ = work;
    // Same conservative boundary as the live helper: includes selectors/connectors.
    for n in all.iter().filter(|n| !excluded.contains(n)) {
        let mut values: Vec<_> = n.get_properties_ns().into_values().collect();
        if n.get_name() == "style" {
            values.push(n.get_content());
        }
        for value in values {
            for (offset, _) in value.match_indices('#') {
                let rest = &value[offset + 1..];
                for target in ids {
                    if let Some(tail) = rest.strip_prefix(target)
                        && !tail.chars().next().is_some_and(|c| {
                            c.is_alphanumeric() || matches!(c, '_' | '.' | ':' | '-')
                        })
                    {
                        return Err(
                            "selected objects are referenced; detach references before deleting",
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
fn matrix(m: affine::Matrix) -> String {
    format!(
        "matrix({})",
        m.into_iter()
            .map(affine::general)
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn composed(node: &Node) -> Result<affine::Matrix, &'static str> {
    let mut result = affine::IDENTITY;
    for n in chain(node).into_iter().rev() {
        if (n.get_name() == "svg" && n.get_parent().is_some_and(|p| p.is_element_node()))
            || n.get_property_no_ns("style")
                .unwrap_or_default()
                .to_lowercase()
                .contains("transform")
        {
            return Err("CSS transforms and nested SVG viewports are unsupported");
        }
        result = affine::multiply(
            result,
            affine::parse(&n.get_property_no_ns("transform").unwrap_or_default())
                .map_err(|_| "non-finite transform")?,
        )
        .map_err(|_| "non-finite transform")?;
    }
    Ok(result)
}
fn path(node: &Node) -> Vec<usize> {
    let mut result = Vec::new();
    let mut n = node.clone();
    while let Some(p) = n.get_parent().filter(Node::is_element_node) {
        result.push(p.get_child_elements().iter().position(|e| *e == n).unwrap());
        n = p;
    }
    result.reverse();
    result
}
/// Bounded request and source; missing/duplicate targets refuse before producing a plan.
/// This is a preparation kernel, not a substitute for the native stale-state guard.
pub fn plan(svg: &str, request: &Request, cap: usize) -> Result<Plan, &'static str> {
    if !json_within_cap(request, cap) {
        return Err("edit request exceeds size cap");
    }
    if !valid_nonce(&request.nonce) {
        return Err("invalid edit identity");
    }
    if request.selection.is_empty()
        || request.selection.len() > 10_000
        || request
            .selection
            .iter()
            .any(|s| s.is_empty() || s.len() > cap)
        || request.selection.iter().collect::<HashSet<_>>().len() != request.selection.len()
    {
        return Err("invalid selection");
    }
    let document = crate::xml::parse(svg.as_bytes(), cap)?;
    let root = document
        .get_root_element()
        .ok_or("invalid document or selection")?;
    let all = elements(root.clone());
    if all.len() > 10_000 {
        return Err("too many edit elements");
    }
    let mut by_id = HashMap::new();
    for n in &all {
        if !id(n).is_empty() && by_id.insert(id(n), n.clone()).is_some() {
            return Err("document contains duplicate ids");
        }
    }
    if by_id.contains_key(&request.nonce) {
        return Err("edit identity collision");
    }
    let mut nodes = request
        .selection
        .iter()
        .map(|s| by_id.get(s).cloned().ok_or("invalid selection"))
        .collect::<Result<Vec<_>, _>>()?;
    for n in &nodes {
        if !drawable(n) || *n == root {
            return Err("select drawable objects");
        }
        if n.get_property_ns("groupmode", INK).as_deref() == Some("layer") {
            return Err("select objects inside the layer, not the layer itself");
        }
        if chain(n).iter().skip(1).any(|a| {
            matches!(
                a.get_name().as_str(),
                "defs" | "clipPath" | "mask" | "pattern" | "symbol"
            )
        }) {
            return Err("definitions are not editable selections");
        }
        if elements(n.clone()).iter().any(|e| {
            chain(e)
                .iter()
                .any(|a| a.get_property_ns("insensitive", SOD).as_deref() == Some("true"))
        }) {
            return Err("selection contains locked objects or belongs to a locked layer");
        }
    }
    let selected = nodes.clone();
    nodes.retain(|n| !chain(n).iter().skip(1).any(|a| selected.contains(a)));
    let affected: Vec<_> = nodes.iter().map(id).collect();
    let mut result = Plan {
        steps: Vec::new(),
        affected_ids: affected.clone(),
    };
    use Operation::*;
    if matches!(request.operation, Duplicate | Group | Ungroup)
        && all.iter().any(|n| n.get_name() == "style")
    {
        return Err("structural edits with stylesheets require inline styles first");
    }
    let budget = cap.saturating_mul(8).min(64 * 1024 * 1024);
    let style_bytes = request.style.iter().fold(0usize, |size, (k, v)| {
        size.saturating_add(k.len()).saturating_add(v.len())
    });
    if nodes.len().saturating_mul(style_bytes.saturating_add(256)) > budget {
        return Err("edit plan exceeds size cap");
    }
    match request.operation {
        Style => {
            if request
                .style
                .keys()
                .any(|k| !matches!(k.as_str(), "fill" | "stroke" | "stroke-width" | "opacity"))
                || (request.style.is_empty()
                    && request.transform.as_deref().unwrap_or("").is_empty())
            {
                return Err("invalid style edit");
            }
            if request
                .style
                .values()
                .any(|v| v.len() > cap || v.contains([';', '{', '}', '\\', '\n']))
            {
                return Err("invalid style value");
            }
            for n in nodes {
                // Preserve live semantics: only explicit inline values are compared;
                // presentation/inherited paint is not headless color normalization.
                let raw = n.get_property_no_ns("style").unwrap_or_default();
                let current: BTreeMap<_, _> = raw
                    .split(';')
                    .filter_map(|s| s.split_once(':'))
                    .map(|(k, v)| (k.trim(), v.trim()))
                    .collect();
                // Complex inline CSS cannot be safely normalized by this planning kernel.
                if raw.contains(['\\', '{', '}', '@']) || raw.contains("/*") {
                    return Err("invalid style value");
                }
                let values: BTreeMap<_, _> = request
                    .style
                    .iter()
                    .filter(|(k, v)| current.get(k.as_str()).copied() != Some(v.as_str()))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let transform = if let Some(delta) =
                    request.transform.as_ref().filter(|s| !s.is_empty())
                {
                    // Inspect the target as well as its ancestors for CSS transforms.
                    composed(&n)?;
                    if all.iter().any(|n| n.get_name() == "style") {
                        return Err("CSS transforms and nested SVG viewports are unsupported");
                    }
                    let parent = composed(&n.get_parent().ok_or("select drawable objects")?)?;
                    let own = affine::parse(&n.get_property_no_ns("transform").unwrap_or_default())
                        .map_err(|_| "non-finite transform")?;
                    let delta = affine::parse(delta).map_err(|_| "non-finite transform")?;
                    let inv = affine::inverse(parent).map_err(|_| "non-finite transform")?;
                    // Avoid inverse/parent roundoff for an exact identity delta.
                    // Keep the context and invertibility checks above even for no-ops.
                    let value = if delta == affine::IDENTITY {
                        own
                    } else {
                        affine::multiply(
                            affine::multiply(
                                affine::multiply(inv, delta).map_err(|_| "non-finite transform")?,
                                parent,
                            )
                            .map_err(|_| "non-finite transform")?,
                            own,
                        )
                        .map_err(|_| "non-finite transform")?
                    };
                    (value != own).then(|| matrix(value))
                } else {
                    None
                };
                if !values.is_empty() || transform.is_some() {
                    result.steps.push(Step::Style {
                        id: id(&n),
                        values,
                        transform,
                    });
                }
            }
        }
        Text => {
            if nodes.len() != 1 || nodes[0].get_name() != "text" {
                return Err("select exactly one text object");
            }
            let text = request
                .text
                .as_ref()
                .ok_or("replacement must be a single line of text")?;
            if text.chars().count() > 100_000 || text.chars().any(|c| (c as u32) < 32) {
                return Err("replacement must be a single line of text");
            }
            let tree = elements(nodes[0].clone());
            if tree
                .iter()
                .any(|n| !matches!(n.get_name().as_str(), "text" | "tspan"))
            {
                return Err("text paths and flowed text are unsupported");
            }
            let leaves: Vec<_> = tree
                .iter()
                .filter(|n| n.get_child_elements().is_empty())
                .collect();
            if leaves.len() != 1
                || tree.iter().any(|n| {
                    let tail = n
                        .get_next_sibling()
                        .filter(Node::is_text_node)
                        .map(|t| t.get_content())
                        .unwrap_or_default();
                    let leading = n
                        .get_first_child()
                        .filter(Node::is_text_node)
                        .map(|t| t.get_content())
                        .unwrap_or_default();
                    !tail.trim().is_empty()
                        || (!n.get_child_elements().is_empty() && !leading.trim().is_empty())
                        || n.get_child_nodes().iter().any(|c| {
                            !c.is_element_node()
                                && !matches!(
                                    c.get_type(),
                                    Some(NodeType::TextNode | NodeType::CDataSectionNode)
                                )
                        })
                })
            {
                return Err(
                    "mixed formatting or multiple text runs; select simple single-run text",
                );
            }
            if leaves[0].get_content() != *text {
                result.steps.push(Step::Text {
                    id: affected[0].clone(),
                    value: text.clone(),
                });
            }
        }
        Duplicate => {
            let mut remap = BTreeMap::new();
            for n in &nodes {
                for e in elements(n.clone()) {
                    let old = id(&e);
                    if old.is_empty() {
                        continue;
                    }
                    let new = format!("{}_{}", request.nonce, remap.len());
                    if by_id.contains_key(&new) {
                        return Err("edit identity collision");
                    }
                    remap.insert(old, new);
                }
            }
            result.affected_ids = nodes
                .iter()
                .flat_map(|n| elements(n.clone()))
                .filter_map(|n| remap.get(&id(&n)).cloned())
                .collect();
            result.steps.push(Step::Duplicate {
                ids: affected,
                remap,
            });
        }
        Delete => {
            let excluded: Vec<_> = nodes.iter().flat_map(|n| elements(n.clone())).collect();
            refs(
                &all,
                &excluded.iter().map(id).filter(|s| !s.is_empty()).collect(),
                &excluded,
            )?;
            result.steps.push(Step::Delete { ids: affected });
        }
        Group => {
            let parent = nodes[0].get_parent().unwrap();
            if nodes
                .iter()
                .any(|n| n.get_parent().as_ref() != Some(&parent))
            {
                return Err("group requires objects in the same layer or parent");
            }
            let children = parent.get_child_nodes();
            nodes.sort_by_key(|n| children.iter().position(|e| e == n).unwrap());
            let positions: Vec<_> = nodes
                .iter()
                .map(|n| children.iter().position(|e| e == n).unwrap())
                .collect();
            // Text tails are attached to preceding elements in lxml, not sibling objects.
            let between = &children[positions[0]..=positions[positions.len() - 1]];
            if between
                .iter()
                .any(|n| !nodes.contains(n) && !n.is_text_node())
            {
                return Err("group requires consecutive siblings to preserve stacking");
            }
            result.steps.push(Step::Group {
                ids: nodes.iter().map(id).collect(),
                id: request.nonce.clone(),
            });
            result.affected_ids.insert(0, request.nonce.clone());
        }
        Ungroup => {
            for n in &nodes {
                if n.get_name() != "g"
                    || n.get_properties_ns().keys().any(|(key, ns)| {
                        !((ns.is_none() && matches!(key.as_str(), "id" | "transform"))
                            || (key == "label"
                                && ns.as_ref().is_some_and(|ns| ns.get_href() == INK)))
                    })
                {
                    return Err("ungroup supports plain groups without inherited style or effects");
                }
            }
            refs(&all, &affected.iter().cloned().collect(), &nodes)?;
            result.affected_ids.clear();
            for n in nodes {
                let m = affine::parse(&n.get_property_no_ns("transform").unwrap_or_default())
                    .map_err(|_| "non-finite transform")?;
                let mut children = Vec::new();
                for c in n.get_child_elements() {
                    // Attribute compensation cannot preserve CSS transforms or viewport mappings.
                    if elements(c.clone()).iter().any(|e| {
                        e.get_name() == "svg"
                            || e.get_property_no_ns("style")
                                .unwrap_or_default()
                                .to_lowercase()
                                .contains("transform")
                    }) {
                        return Err("CSS transforms and nested SVG viewports are unsupported");
                    }
                    let own = affine::parse(&c.get_property_no_ns("transform").unwrap_or_default())
                        .map_err(|_| "non-finite transform")?;
                    // Plans use element positions at application time for anonymous children.
                    children.push((
                        id(&c),
                        matrix(affine::multiply(m, own).map_err(|_| "non-finite transform")?),
                    ));
                    if !id(&c).is_empty() {
                        result.affected_ids.push(id(&c));
                    }
                }
                result.steps.push(Step::Ungroup {
                    id: id(&n),
                    children,
                });
            }
        }
        Raise | Lower | Front | Back => {
            let mut parents = Vec::new();
            for n in &nodes {
                let p = n.get_parent().unwrap();
                if !parents.contains(&p) {
                    parents.push(p);
                }
            }
            for parent in parents {
                let siblings = parent.get_child_elements();
                let mut order: Vec<_> = (0..siblings.len()).collect();
                let chosen: HashSet<_> = siblings
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| nodes.contains(n))
                    .map(|(i, _)| i)
                    .collect();
                let mut selected: Vec<_> = order
                    .iter()
                    .copied()
                    .filter(|i| chosen.contains(i))
                    .collect();
                if matches!(request.operation, Raise | Back) {
                    selected.reverse();
                }
                for i in selected {
                    let pos = order.iter().position(|j| *j == i).unwrap();
                    let target = match request.operation {
                        Front => order.iter().rposition(|j| paintable(&siblings[*j])),
                        Back => order.iter().position(|j| paintable(&siblings[*j])),
                        Raise => (pos + 1..order.len()).find(|j| paintable(&siblings[order[*j]])),
                        Lower => (0..pos).rev().find(|j| paintable(&siblings[order[*j]])),
                        _ => unreachable!(),
                    };
                    if let Some(target) = target
                        && target != pos
                        && (matches!(request.operation, Front | Back)
                            || !chosen.contains(&order[target]))
                    {
                        order.remove(pos);
                        order.insert(target, i);
                    }
                }
                if order != (0..siblings.len()).collect::<Vec<_>>() {
                    result.steps.push(Step::Order {
                        parent_path: path(&parent),
                        indices: order,
                    });
                }
            }
        }
    }
    if !json_within_cap(&result, budget) {
        return Err("edit plan exceeds size cap");
    }
    Ok(result)
}

/// Guard captured drawing content/IDs and the independently observed native selection.
/// Window/document UUID ownership is still checked by the transport/extension consumer.
pub fn guard(
    svg: &str,
    expected_fingerprint: &str,
    expected_ids: &[String],
    expected_selection: &[String],
    native_selection: &[String],
    cap: usize,
) -> Result<(), &'static str> {
    if [expected_ids, expected_selection, native_selection]
        .iter()
        .any(|ids| ids.len() > 10_000 || ids.iter().any(|s| s.len() > cap))
    {
        return Err("invalid selection");
    }
    let document = crate::xml::parse(svg.as_bytes(), cap)?;
    let root = document
        .get_root_element()
        .ok_or("invalid document or selection")?;
    let all = elements(root);
    if all.len() > 10_000 {
        return Err("too many edit elements");
    }
    let ids: Vec<_> = all.iter().map(id).filter(|s| !s.is_empty()).collect();
    let actual: HashSet<_> = ids.iter().collect();
    if actual.len() != ids.len() {
        return Err("document contains duplicate ids");
    }
    if actual != expected_ids.iter().collect() {
        return Err("document changed before insertion");
    }
    if super::fingerprint::fingerprint(svg, cap)? != expected_fingerprint {
        return Err("drawing content changed before insertion");
    }
    if expected_selection.iter().collect::<HashSet<_>>() != native_selection.iter().collect() {
        return Err("selection changed before edit");
    }
    Ok(())
}
