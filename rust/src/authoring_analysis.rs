//! Explicit semantic roles and conservative, bounded, read-only scene observations.
use crate::css;
use libxml::tree::Node;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use svgtypes::{Paint, PathParser, PathSegment};
const MAX_NODES: usize = 20000;
const DRAWABLE: &[&str] = &[
    "g", "svg", "path", "rect", "circle", "ellipse", "line", "polyline", "polygon", "text",
    "image", "use",
];
const RESOURCE: &[&str] = &[
    "defs", "symbol", "mask", "clipPath", "pattern", "marker", "filter",
];

pub fn roles(opts: &Value) -> Result<Vec<(String, String)>, String> {
    let Some(value) = opts.get("object_roles") else {
        return Ok(vec![]);
    };
    let list = value
        .as_array()
        .filter(|a| a.len() <= 200)
        .ok_or("object_roles must be an array of at most 200 entries")?;
    let mut seen = HashSet::new();
    list.iter()
        .map(|entry| {
            let object = entry
                .as_object()
                .ok_or("object_roles entries must be objects")?;
            if object.len() != 2 {
                return Err("object_roles entries require only object_id and role".into());
            }
            let id = entry["object_id"]
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 256)
                .ok_or("object role ID must contain 1–256 bytes")?;
            let role = entry["role"]
                .as_str()
                .filter(|s| matches!(*s, "stroke_only" | "independent_strokes" | "closed_shape"))
                .ok_or("invalid object role")?;
            if !seen.insert(id) {
                return Err("duplicate object role ID".into());
            }
            Ok((id.into(), role.into()))
        })
        .collect()
}
struct Findings {
    items: Vec<Value>,
    total: usize,
}
impl Findings {
    fn add(&mut self, id: Option<String>, code: &str, certainty: &str, reason: impl Into<String>) {
        self.total += 1;
        if self.items.len() < 200 {
            self.items.push(
                json!({"object_id":id,"code":code,"certainty":certainty,"reason":reason.into()}),
            );
        }
    }
    fn node(&mut self, node: &Node, code: &str, certainty: &str, reason: impl Into<String>) {
        self.add(node.get_property_no_ns("id"), code, certainty, reason);
    }
}
struct Paths {
    bytes: usize,
    segments: usize,
}
impl Paths {
    fn subpaths(&mut self, node: &Node) -> Result<usize, String> {
        let d = node.get_property_no_ns("d").unwrap_or_default();
        self.bytes = self.bytes.saturating_add(d.len());
        if d.len() > 262144 || self.bytes > 4_194_304 {
            return Err("Path byte limit exceeded.".into());
        }
        let mut subpaths = 0;
        let mut count = 0;
        for segment in PathParser::from(d.as_str()) {
            count += 1;
            self.segments += 1;
            if count > 10000 || self.segments > 200000 {
                return Err("Path segment limit exceeded.".into());
            }
            let segment = segment.map_err(|_| "Invalid SVG path data.")?;
            if count == 1 && !matches!(segment, PathSegment::MoveTo { .. }) {
                return Err("Path must begin with moveto.".into());
            }
            if matches!(segment, PathSegment::MoveTo { .. }) {
                subpaths += 1;
            }
            // SVG numbers must remain finite even when a syntactically valid exponent overflows.
            let finite = match segment {
                PathSegment::MoveTo { x, y, .. }
                | PathSegment::LineTo { x, y, .. }
                | PathSegment::SmoothQuadratic { x, y, .. } => {
                    [x, y].into_iter().all(f64::is_finite)
                }
                PathSegment::HorizontalLineTo { x, .. } => x.is_finite(),
                PathSegment::VerticalLineTo { y, .. } => y.is_finite(),
                PathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                    ..
                } => [x1, y1, x2, y2, x, y].into_iter().all(f64::is_finite),
                PathSegment::SmoothCurveTo { x2, y2, x, y, .. } => {
                    [x2, y2, x, y].into_iter().all(f64::is_finite)
                }
                PathSegment::Quadratic { x1, y1, x, y, .. } => {
                    [x1, y1, x, y].into_iter().all(f64::is_finite)
                }
                PathSegment::EllipticalArc {
                    rx,
                    ry,
                    x_axis_rotation,
                    x,
                    y,
                    ..
                } => [rx, ry, x_axis_rotation, x, y]
                    .into_iter()
                    .all(f64::is_finite),
                PathSegment::ClosePath { .. } => true,
            };
            if !finite {
                return Err("Non-finite path coordinates.".into());
            }
        }
        if count > 0 && subpaths == 0 {
            return Err("Path must begin with moveto.".into());
        }
        Ok(subpaths)
    }
}
fn paint(value: String) -> Result<bool, String> {
    Paint::from_str(&value)
        .map(|p| matches!(p, Paint::None))
        .map_err(|_| "Unsupported or invalid paint value.".into())
}
fn transparent(value: String) -> Result<bool, String> {
    match Paint::from_str(&value) {
        Ok(Paint::None) => Ok(true),
        Ok(Paint::Color(c)) => Ok(c.alpha == 0),
        _ => Err("Paint server/current/context paint transparency needs rendered review.".into()),
    }
}
fn number(value: String) -> Result<f64, String> {
    let number = if let Some(percent) = value.strip_suffix('%') {
        percent.parse::<f64>().map(|n| n / 100.)
    } else {
        value.parse::<f64>()
    };
    number
        .ok()
        .filter(|n| n.is_finite())
        .ok_or_else(|| "Unsupported opacity value.".into())
}
fn stroke(node: &Node, cascade: &css::Analysis, paths: &mut Paths, findings: &mut Findings) {
    if node
        .get_namespace()
        .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
    {
        findings.node(
            node,
            "stroke_geometry_unknown",
            "unknown",
            "Foreign namespace geometry is unsupported.",
        );
        return;
    }
    match cascade.computed(node,"fill").and_then(paint) {
        Ok(false)=>findings.node(node,"stroke_fill","known","Designated stroke has effective fill other than none (including transparent/default/inherited paint); explicitly set fill=none."),
        Err(reason)=>findings.node(node,"stroke_fill_unknown","unknown",reason),
        Ok(true)=>{},
    }
    if node.get_name() == "path" {
        match paths.subpaths(node) {
            Ok(n) if n>1=>findings.node(node,"combined_strokes","known",format!("Designated stroke contains {n} subpaths; separate independent strokes into objects in a named group.")),
            Err(reason)=>findings.node(node,"stroke_path_unknown","unknown",reason),
            _=>{},
        }
    } else if !["line", "polyline", "polygon", "rect", "circle", "ellipse"]
        .contains(&node.get_name().as_str())
    {
        findings.node(node,"stroke_geometry_unknown","unknown","Role needs a supported geometry object; text, instances and containers require manual review.");
    }
}
fn hidden(node: &Node, cascade: &css::Analysis) -> Result<Option<&'static str>, String> {
    if node
        .get_namespace()
        .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
    {
        return Err("Foreign namespace scene geometry is unsupported.".into());
    }
    let mut ancestor = Some(node.clone());
    let mut uncertain = None;
    for _ in 0..128 {
        let Some(n) = ancestor else {
            break;
        };
        let display = cascade.computed(&n, "display");
        let opacity = cascade.computed(&n, "opacity").and_then(number);
        match display.as_deref() {
            Ok("none") => return Ok(Some("display=none on this object or an ancestor")),
            Ok("inline" | "block" | "contents" | "inline-block") => {}
            Ok(_) => uncertain = Some("Unsupported display value.".into()),
            Err(reason) => uncertain = Some(reason.clone()),
        }
        match opacity {
            Ok(n) if n <= 0. => return Ok(Some("zero opacity on this object or an ancestor")),
            Err(reason) => uncertain = Some(reason),
            _ => {}
        }
        ancestor = n.get_parent().filter(Node::is_element_node);
    }
    if ancestor.is_some() {
        return Err("Scene ancestor depth limit exceeded.".into());
    }
    match cascade.computed(node, "visibility")?.as_str() {
        "hidden" | "collapse" if ["g","svg","text","use"].contains(&node.get_name().as_str()) => return Err("Container/instance visibility may be overridden by descendants; review the rendered subtree.".into()),
        "hidden" | "collapse" => return Ok(Some("effective visibility is hidden/collapse")),
        "visible" => {}
        _ => return Err("Unsupported visibility value.".into()),
    }
    if let Some(reason) = uncertain {
        return Err(reason);
    }
    // Filled/stroked geometry only. Images, instances, containers and text have
    // additional rendering semantics; opacity/display/visibility above still apply.
    if [
        "path", "rect", "circle", "ellipse", "line", "polyline", "polygon",
    ]
    .contains(&node.get_name().as_str())
    {
        for property in [
            "filter",
            "mask",
            "clip-path",
            "marker-start",
            "marker-mid",
            "marker-end",
        ] {
            if cascade.computed(node, property)? != "none" {
                return Err("Effects/markers require rendered review; transparent paint alone is inconclusive.".into());
            }
        }
        let fill = transparent(cascade.computed(node, "fill")?)?
            || number(cascade.computed(node, "fill-opacity")?)? <= 0.;
        let stroke = transparent(cascade.computed(node, "stroke")?)?
            || number(cascade.computed(node, "stroke-opacity")?)? <= 0.;
        if fill && stroke {
            return Ok(Some(
                "both fill and stroke paint are absent or fully transparent",
            ));
        }
    }
    Ok(None)
}
fn chain(node: &Node) -> Vec<Node> {
    let mut out = vec![];
    let mut current = Some(node.clone());
    for _ in 0..128 {
        let Some(n) = current else {
            break;
        };
        current = n.get_parent().filter(Node::is_element_node);
        out.push(n);
    }
    out
}
pub fn analyze(nodes: &[Node], roles: &[(String, String)]) -> Value {
    let mut findings = Findings {
        items: vec![],
        total: 0,
    };
    if nodes.len() > MAX_NODES {
        findings.add(None,"analysis_limit","unknown","Authoring review is limited to 20,000 elements; no scene or role conclusions were made.");
    } else {
        let cascade = css::Analysis::new(nodes);
        let mut by_id: HashMap<String, Vec<&Node>> = HashMap::new();
        for node in nodes {
            if let Some(id) = node.get_property_no_ns("id") {
                by_id.entry(id).or_default().push(node);
            }
        }
        let mut paths = Paths {
            bytes: 0,
            segments: 0,
        };
        let mut checked = HashSet::new();
        let mut role_work = 0usize;
        for (id, role) in roles {
            let Some(matches) = by_id.get(id).filter(|n| n.len() == 1) else {
                findings.add(
                    Some(id.clone()),
                    "role_target_unknown",
                    "unknown",
                    "Role target ID is missing or ambiguous.",
                );
                continue;
            };
            let node = matches[0];
            if role == "closed_shape" {
                continue; // Checked by the shared geometry analyzer.
            } else if role == "stroke_only" {
                if checked.insert(node.node_ptr() as usize) {
                    stroke(node, &cascade, &mut paths, &mut findings);
                }
            } else if node.get_name() != "g" {
                findings.node(
                    node,
                    "role_target_unknown",
                    "unknown",
                    "independent_strokes requires an ordinary SVG group.",
                );
            } else {
                let mut count = 0;
                for child in nodes {
                    role_work += 1;
                    if role_work > 200000 {
                        findings.node(
                            node,
                            "role_analysis_limit",
                            "unknown",
                            "Group traversal work limit exceeded.",
                        );
                        break;
                    }
                    let ancestry = chain(child);
                    if ancestry
                        .last()
                        .and_then(Node::get_parent)
                        .is_some_and(|p| p.is_element_node())
                    {
                        findings.node(
                            child,
                            "role_analysis_limit",
                            "unknown",
                            "Stroke ancestry exceeds the supported depth.",
                        );
                        count += 1;
                        continue;
                    }
                    if child == node
                        || !ancestry.iter().any(|p| p == node)
                        || ancestry
                            .iter()
                            .any(|n| RESOURCE.contains(&n.get_name().as_str()))
                    {
                        continue;
                    }
                    if DRAWABLE.contains(&child.get_name().as_str())
                        && !["g", "svg"].contains(&child.get_name().as_str())
                    {
                        count += 1;
                        if checked.insert(child.node_ptr() as usize) {
                            stroke(child, &cascade, &mut paths, &mut findings);
                        }
                    }
                }
                if count == 0 {
                    findings.node(
                        node,
                        "empty_stroke_group",
                        "known",
                        "Designated group contains no supported scene strokes.",
                    );
                }
            }
        }
        let mut referenced = crate::optimization_analysis::references(nodes[0].clone());
        let mut reference_uncertain = false;
        let url = regex::Regex::new(r##"(?i)url\(\s*['\"]?#([^'\")\s]+)"##).unwrap();
        for node in nodes {
            let text = crate::inspect::own_text(node);
            for capture in url.captures_iter(&text) {
                referenced.insert(capture[1].into());
                if capture[1].contains('%') {
                    reference_uncertain = true;
                }
            }
            for ((key, _), value) in node.get_properties_ns() {
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
                        | "cursor"
                ) {
                    let result = crate::css_assets::rewrite(&value, |url| {
                        if url.contains('%') {
                            reference_uncertain = true;
                        }
                        if let Some(id) = url.strip_prefix('#') {
                            referenced.insert(id.to_owned());
                        } else {
                            reference_uncertain = true;
                        }
                        Ok(None)
                    });
                    if result.is_err() {
                        reference_uncertain = true;
                    }
                }
                for capture in url.captures_iter(&value) {
                    referenced.insert(capture[1].into());
                }
                if matches!(key.as_str(), "aria-labelledby" | "aria-describedby") {
                    referenced.extend(value.split_whitespace().map(str::to_owned));
                }
                if matches!(key.as_str(), "begin" | "end") {
                    for part in value.split(';') {
                        if let Some((id, _)) = part.trim().split_once('.') {
                            referenced.insert(id.into());
                        }
                    }
                }
                if key == "href" && (!value.starts_with('#') || value.contains('%')) {
                    reference_uncertain = true;
                }
            }
        }
        let mut protected = HashSet::new();
        for node in nodes {
            if node
                .get_property_no_ns("id")
                .is_some_and(|id| referenced.contains(&id))
            {
                for n in chain(node) {
                    protected.insert(n.node_ptr() as usize);
                }
            }
        }
        for node in nodes
            .iter()
            .filter(|n| DRAWABLE.contains(&n.get_name().as_str()))
        {
            match hidden(node, &cascade) {
                Ok(Some(reason)) => {
                    let ancestors = chain(node);
                    if ancestors
                        .last()
                        .and_then(Node::get_parent)
                        .is_some_and(|p| p.is_element_node())
                    {
                        findings.node(
                            node,
                            "scene_visibility_unknown",
                            "unknown",
                            "Resource/reference ancestry exceeds the supported depth.",
                        );
                        continue;
                    }
                    let required = ancestors.iter().any(|n| {
                        RESOURCE.contains(&n.get_name().as_str())
                            || n.get_property_no_ns("id")
                                .is_some_and(|id| referenced.contains(&id))
                    }) || protected.contains(&(node.node_ptr() as usize));
                    findings.node(node, if required {"hidden_required_geometry"} else if reference_uncertain {"hidden_reference_unknown"} else {"hidden_scene_geometry"}, if reference_uncertain && !required {"unknown"} else {"known"},format!("{reason}; {}",if required {"required resource/reference geometry: preserve and review in context"} else {"review manually before any approved repair; this report performs no deletion"}));
                }
                Err(reason) => findings.node(node, "scene_visibility_unknown", "unknown", reason),
                _ => {}
            }
        }
    }
    json!({"findings":findings.items,"truncated":findings.total>200,"scope":"Static supported CSS and explicit roles only. General occlusion and semantic silhouette quality require rendered/manual review; bounding-box overlap is not proof of coverage. No mutation or automatic cleanup."})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn review(body: &str, roles_value: Value) -> Value {
        let source = format!("<svg xmlns='http://www.w3.org/2000/svg'>{body}</svg>");
        let doc = crate::xml::parse(source.as_bytes(), 10_000_000).unwrap();
        let before = crate::xml::serialize(&doc);
        let options = json!({"object_roles":roles_value});
        let result = analyze(
            &crate::document::elements(doc.get_root_element().unwrap()),
            &roles(&options).unwrap(),
        );
        assert_eq!(crate::xml::serialize(&doc), before);
        result
    }
    fn has(report: &Value, id: &str, code: &str) -> bool {
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["object_id"] == id && f["code"] == code)
    }
    #[test]
    fn explicit_strokes_cover_default_inherited_inline_stylesheet_and_transparency() {
        let body = "<style>.clean { fill:none } #priority { fill:none !important } path {stroke:black}</style><g fill='red'><path id='inherited' d='M0 0L1 1'/><path id='attribute' fill='none' d='M0 0L1 1'/><path id='inline' style='fill:none' d='M0 0L1 1'/><path id='sheet' class='clean' fill='red' d='M0 0L1 1'/><path id='priority' style='fill:red' d='M0 0L1 1'/><path id='zero' fill-opacity='0' d='M0 0L1 1'/><path id='transparent' fill='transparent' d='M0 0L1 1'/></g><path id='default' d='M0 0L1 1'/>";
        let roles = json!(
            [
                "inherited",
                "attribute",
                "inline",
                "sheet",
                "priority",
                "zero",
                "transparent",
                "default"
            ]
            .map(|id| json!({"object_id":id,"role":"stroke_only"}))
        );
        let report = review(body, roles);
        for id in ["inherited", "zero", "transparent", "default"] {
            assert!(has(&report, id, "stroke_fill"), "{id}: {report}");
        }
        for id in ["attribute", "inline", "sheet", "priority"] {
            assert!(!has(&report, id, "stroke_fill"), "{id}: {report}");
        }
    }
    #[test]
    fn compound_paths_only_flag_designated_strokes_and_parse_implicit_commands() {
        let report = review(
            "<g id='folds' fill='none' stroke='black'><path id='a' d='m0 0 1 1'/><path id='b' d='M2 2L3 3'/><path id='combined' d='M4 4L5 5m1 1l2 2'/><path id='bad' d='M0 broken'/></g><path id='filled' d='M0 0L1 1Z M2 2L3 3Z'/>",
            json!([{"object_id":"folds","role":"independent_strokes"}]),
        );
        assert!(has(&report, "combined", "combined_strokes"));
        assert!(has(&report, "bad", "stroke_path_unknown"));
        for id in ["a", "b", "filled"] {
            assert!(!has(&report, id, "combined_strokes"));
        }
    }
    #[test]
    fn unsupported_css_unknown_targets_bounds_and_disabled_inputs() {
        let report = review(
            "<style>g path {fill:none}</style><path id='p' fill='none' d='M0 0L1 1'/>",
            json!([{"object_id":"p","role":"stroke_only"},{"object_id":"missing","role":"stroke_only"}]),
        );
        assert!(has(&report, "p", "stroke_fill_unknown"));
        assert!(has(&report, "missing", "role_target_unknown"));
        let report = review(
            "<style>.123 {fill:none}</style><path id='numeric-class' class='123' d='M0 0L1 1'/>",
            json!([{"object_id":"numeric-class","role":"stroke_only"}]),
        );
        assert!(has(&report, "numeric-class", "stroke_fill_unknown"));
        let report = review(
            "<style xmlns='urn:foreign'>path{fill:none}</style><path id='foreign-sheet' d='M0 0L1 1'/>",
            json!([{"object_id":"foreign-sheet","role":"stroke_only"}]),
        );
        assert!(has(&report, "foreign-sheet", "stroke_fill_unknown"));
        let report = review(
            "<g xmlns='urn:foreign' fill='none'><path xmlns='http://www.w3.org/2000/svg' id='foreign-parent' d='M0 0L1 1'/></g>",
            json!([{"object_id":"foreign-parent","role":"stroke_only"}]),
        );
        assert!(has(&report, "foreign-parent", "stroke_fill_unknown"));
        assert!(roles(&json!({"object_roles":[{"object_id":"p","role":"unknown"}]})).is_err());
        assert!(
            roles(&json!({"object_roles":vec![json!({"object_id":"p","role":"stroke_only"});201]}))
                .is_err()
        );
        assert!(roles(&json!({"object_roles":[{"object_id":"p","role":"stroke_only"},{"object_id":"p","role":"stroke_only"}]})).is_err());
        let report = review(
            "<path id='p' style='fill:var(--paint)' d='M0 0L1 1'/>",
            json!([{"object_id":"p","role":"stroke_only"}]),
        );
        assert!(has(&report, "p", "stroke_fill_unknown"));
    }
    #[test]
    fn hidden_scene_accounts_for_ancestors_resources_references_and_partial_overlap() {
        let body = "<defs><mask id='mask'><rect id='mask-shape' opacity='0'/></mask><clipPath id='clip'><path id='clip-shape' fill='none'/></clipPath></defs><g id='visibility-group' visibility='hidden'><rect id='inherited'/><rect id='override' visibility='visible'/></g><g opacity='0'><rect id='transparent'/></g><rect id='alpha' fill='rgba(0,0,0,0)'/><rect id='referenced' display='none'/><use href='#referenced'/><g id='holder' display='none'><rect id='target'/></g><use href='#target'/><rect id='partial' x='0' width='10'/><rect id='overlap' x='5' width='10' mask='url(#mask)' clip-path='url(#clip)'/>";
        let report = review(body, json!([]));
        for id in ["inherited", "transparent", "alpha"] {
            assert!(has(&report, id, "hidden_scene_geometry"), "{id}: {report}");
        }
        for id in ["mask-shape", "clip-shape", "referenced", "holder", "target"] {
            assert!(
                has(&report, id, "hidden_required_geometry"),
                "{id}: {report}"
            );
        }
        assert!(has(&report, "visibility-group", "scene_visibility_unknown"));
        for id in ["override", "partial", "overlap"] {
            assert!(!has(&report, id, "hidden_scene_geometry"));
        }
    }
    #[test]
    fn uncertainty_effects_external_references_and_finding_cap() {
        let report = review(
            "<rect id='effects' fill='none' stroke='none' filter='url(#f)'/><rect id='h' display='none'/><use href='other.svg#external'/>",
            json!([]),
        );
        assert!(has(&report, "effects", "scene_visibility_unknown"));
        assert!(has(&report, "h", "hidden_reference_unknown"));
        let report = review(&"<rect opacity='0'/>".repeat(220), json!([]));
        assert_eq!(report["findings"].as_array().unwrap().len(), 200);
        assert_eq!(report["truncated"], true);
    }
    #[test]
    fn limits_and_valid_compound_geometry_do_not_claim_passes() {
        let doc = crate::xml::parse(
            format!("<svg>{}</svg>", "<path/>".repeat(20000)).as_bytes(),
            1_000_000,
        )
        .unwrap();
        let report = analyze(
            &crate::document::elements(doc.get_root_element().unwrap()),
            &[],
        );
        assert_eq!(report["findings"][0]["code"], "analysis_limit");
        let data = format!("M0 0{}", " L1 1".repeat(10001));
        let report = review(
            &format!("<path id='large' fill='none' d='{data}'/>"),
            json!([{"object_id":"large","role":"stroke_only"}]),
        );
        assert!(has(&report, "large", "stroke_path_unknown"));
        let report = review(
            "<style>path{fill:red!important} path{fill:none}</style><path id='important' fill='none' style='fill:none' d='M0 0L1 1'/><path id='own-important' style='fill:none!important;fill:red' d='M0 0L1 1'/>",
            json!([{"object_id":"important","role":"stroke_only"},{"object_id":"own-important","role":"stroke_only"}]),
        );
        assert!(has(&report, "important", "stroke_fill"));
        assert!(!has(&report, "own-important", "stroke_fill"));
        let report = review(
            "<path id='paint-server' fill='url(#paint)' d='M0 0L1 1'/><g opacity='0%'><path id='zero' d='M0 0L1 1'/></g>",
            json!([]),
        );
        assert!(has(&report, "paint-server", "scene_visibility_unknown"));
        assert!(has(&report, "zero", "hidden_scene_geometry"));
        let report = review(
            "<path xmlns='urn:foreign' id='foreign' opacity='0' d='M0 0L1 1'/><path id='shorthand' style='all:initial;fill:none' d='M0 0L1 1'/>",
            json!([{"object_id":"foreign","role":"stroke_only"},{"object_id":"shorthand","role":"stroke_only"}]),
        );
        assert!(has(&report, "foreign", "stroke_geometry_unknown"));
        assert!(has(&report, "foreign", "scene_visibility_unknown"));
        assert!(has(&report, "shorthand", "stroke_fill_unknown"));
        let selectors = vec![".large"; 64].join(",");
        let report = review(
            &format!(
                "<style>{selectors}{{fill:{}}}</style><path id='budget' class='large' fill='none' d='M0 0L1 1'/>",
                "red".repeat(15000)
            ),
            json!([{"object_id":"budget","role":"stroke_only"}]),
        );
        assert!(
            report["findings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["object_id"] == "budget"
                    && f["reason"].as_str().unwrap().contains("work limit"))
        );
    }
}
