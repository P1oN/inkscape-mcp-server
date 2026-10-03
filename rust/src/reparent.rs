//! Native affine compensation and conservative structural context checks.
use crate::{
    arguments,
    document::{Registry, elements},
    structure, style, transaction,
};
use libxml::tree::{Document, Namespace, Node, NodeType};
use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

type Matrix = [f64; 6];
const IDENTITY: Matrix = [1., 0., 0., 1., 0., 0.];
const INK: &str = "http://www.inkscape.org/namespaces/inkscape";
static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?").unwrap());
static CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(matrix|translate|scale|rotate|skewX|skewY)\s*\(([^()]*)\)").unwrap()
});
static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"url\(\s*['"]?#([^\s)'"]+)"#).unwrap());

pub(crate) fn multiply(l: Matrix, r: Matrix) -> Result<Matrix, String> {
    let [a, b, c, d, e, f] = l;
    let [g, h, i, j, k, y] = r;
    let result = [
        a * g + c * h,
        b * g + d * h,
        a * i + c * j,
        b * i + d * j,
        a * k + c * y + e,
        b * k + d * y + f,
    ];
    if result.iter().all(|v| v.is_finite()) {
        Ok(result)
    } else {
        Err("non-finite composed transform".into())
    }
}
pub(crate) fn inverse(m: Matrix) -> Result<Matrix, String> {
    let [a, b, c, d, e, f] = m;
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant.abs() < 1e-12 {
        return Err("singular or ill-conditioned parent transform".into());
    }
    Ok([
        d / determinant,
        -b / determinant,
        -c / determinant,
        a / determinant,
        (c * f - d * e) / determinant,
        (b * e - a * f) / determinant,
    ])
}
fn separators(s: &str) -> bool {
    s.trim_matches([' ', ',', '\t', '\r', '\n']).is_empty()
}
pub(crate) fn parse(raw: &str) -> Result<Matrix, String> {
    let mut result = IDENTITY;
    let mut cursor = 0;
    if raw.len() > 200_000 {
        return Err("SVG transform exceeds 200000 bytes".into());
    }
    for call in CALL.captures_iter(raw) {
        let matched = call.get(0).unwrap();
        if !separators(&raw[cursor..matched.start()]) {
            return Err("unsupported SVG transform".into());
        }
        let body = &call[2];
        if !separators(&NUMBER.replace_all(body, "")) {
            return Err("invalid SVG transform arguments".into());
        }
        let args = NUMBER
            .find_iter(body)
            .map(|n| {
                n.as_str()
                    .parse::<f64>()
                    .map_err(|_| "invalid SVG transform arguments".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !args.iter().all(|v| v.is_finite()) {
            return Err("non-finite SVG transform".into());
        }
        let m = match (&call[1], args.as_slice()) {
            ("matrix", [a, b, c, d, e, f]) => [*a, *b, *c, *d, *e, *f],
            ("translate", [x]) => [1., 0., 0., 1., *x, 0.],
            ("translate", [x, y]) => [1., 0., 0., 1., *x, *y],
            ("scale", [x]) => [*x, 0., 0., *x, 0., 0.],
            ("scale", [x, y]) => [*x, 0., 0., *y, 0., 0.],
            ("rotate", a) if a.len() == 1 || a.len() == 3 => {
                let angle = a[0] * (std::f64::consts::PI / 180.);
                let (s, c) = (angle.sin(), angle.cos());
                let rotation = [c, s, -s, c, 0., 0.];
                if a.len() == 3 {
                    multiply(
                        multiply([1., 0., 0., 1., a[1], a[2]], rotation)?,
                        [1., 0., 0., 1., -a[1], -a[2]],
                    )?
                } else {
                    rotation
                }
            }
            ("skewX", [a]) => [
                1.,
                0.,
                (a * (std::f64::consts::PI / 180.)).tan(),
                1.,
                0.,
                0.,
            ],
            ("skewY", [a]) => [
                1.,
                (a * (std::f64::consts::PI / 180.)).tan(),
                0.,
                1.,
                0.,
                0.,
            ],
            _ => return Err("unsupported SVG transform arity".into()),
        };
        result = multiply(result, m)?;
        cursor = matched.end();
    }
    if !raw[cursor..].trim().is_empty() {
        return Err("unsupported SVG transform".into());
    }
    Ok(result)
}
fn chain(mut node: Node) -> Vec<Node> {
    let mut nodes = vec![node.clone()];
    while let Some(parent) = node.get_parent().filter(Node::is_element_node) {
        nodes.push(parent.clone());
        node = parent;
    }
    nodes
}
pub(crate) fn composed(node: Node) -> Result<Matrix, String> {
    let mut result = IDENTITY;
    for ancestor in chain(node).into_iter().rev() {
        if ancestor.get_name() == "svg"
            && ancestor.get_parent().is_some_and(|p| p.is_element_node())
        {
            return Err("nested SVG viewports require preparation".into());
        }
        if ancestor
            .get_property_no_ns("style")
            .unwrap_or_default()
            .to_lowercase()
            .contains("transform")
        {
            return Err("CSS transforms require preparation".into());
        }
        result = multiply(
            result,
            parse(&ancestor.get_property_no_ns("transform").unwrap_or_default())?,
        )?;
    }
    Ok(result)
}
/// Match Python's .17g matrix serialization rather than shortening f64 values.
pub(crate) fn general(value: f64) -> String {
    let scientific = format!("{value:.16e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let sign = if value.is_sign_negative() { "-" } else { "" };
    let digits = mantissa.trim_start_matches('-').replace('.', "");
    let significant = digits.trim_end_matches('0');
    let significant = if significant.is_empty() {
        "0"
    } else {
        significant
    };
    if !(-4..17).contains(&exponent) {
        let head = &significant[..1];
        let rest = &significant[1..];
        let fraction = if rest.is_empty() {
            String::new()
        } else {
            format!(".{rest}")
        };
        return format!(
            "{sign}{head}{fraction}e{}{:02}",
            if exponent < 0 { "-" } else { "+" },
            exponent.abs()
        );
    }
    let point = exponent + 1;
    if point <= 0 {
        format!("{sign}0.{}{significant}", "0".repeat((-point) as usize))
    } else if point as usize >= significant.len() {
        format!(
            "{sign}{significant}{}",
            "0".repeat(point as usize - significant.len())
        )
    } else {
        let (a, b) = significant.split_at(point as usize);
        format!("{sign}{a}.{b}")
    }
}
pub(crate) fn references(node: &Node) -> HashSet<String> {
    let mut refs = HashSet::new();
    for ((key, _), value) in node.get_properties_ns() {
        if matches!(key.as_str(), "href" | "connection-start" | "connection-end")
            && let Some(id) = value.strip_prefix('#')
        {
            refs.insert(id.to_owned());
        }
        for c in URL.captures_iter(&value) {
            refs.insert(c[1].to_owned());
        }
    }
    refs
}
pub struct Mutation {
    id: String,
    dest: String,
    preserve: bool,
    pub params: Value,
}
impl Mutation {
    pub fn build(args: &Value) -> Result<Self, String> {
        let id = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?
            .to_owned();
        let dest = args["new_parent_id"]
            .as_str()
            .ok_or("new_parent_id must be a string")?
            .to_owned();
        let preserve = arguments::boolean(args, "preserve_appearance", false)?;
        let params = json!({"object_id":id,"new_parent_id":dest,"preserve_appearance":preserve});
        Ok(Self {
            id,
            dest,
            preserve,
            params,
        })
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        let root = document
            .get_root_element()
            .ok_or("document could not be parsed safely")?;
        let nodes = elements(root.clone());
        let find = |id: &str| {
            nodes
                .iter()
                .find(|n| n.get_property_no_ns("id").as_deref() == Some(id))
                .cloned()
                .ok_or("object id not found in document")
        };
        let mut node = find(&self.id)?;
        let mut dest = find(&self.dest)?;
        let subtree = elements(node.clone());
        let subtree_pointers = subtree
            .iter()
            .map(|n| n.node_ptr() as usize)
            .collect::<HashSet<_>>();
        let old = node.get_parent().filter(Node::is_element_node);
        if self.preserve {
            if old.is_none() || subtree_pointers.contains(&(dest.node_ptr() as usize)) {
                return Err("cannot reparent root or create a container cycle".into());
            }
        } else {
            if dest == node {
                return Err("cannot reparent an object under itself".into());
            }
            if subtree_pointers.contains(&(dest.node_ptr() as usize)) {
                return Err("cannot reparent an object under its own descendant".into());
            }
            if old.is_none() {
                return Err("cannot reparent the document root".into());
            }
        }
        let old = old.unwrap();
        if dest.get_name() != "g" {
            return Err("destination must be a group/layer".into());
        }
        if self.preserve && old == dest {
            return Ok("object already belongs to destination".into());
        }
        structure::reject_stylesheets(document)?;
        let old_chain = chain(old.clone());
        let new_chain = chain(dest.clone());
        let divergent = old_chain
            .iter()
            .filter(|n| !new_chain.contains(n))
            .chain(new_chain.iter().filter(|n| !old_chain.contains(n)))
            .collect::<Vec<_>>();
        for ancestor in &divergent {
            if ancestor.get_name() != "g"
                || ancestor.get_properties_ns().keys().any(|(key, ns)| {
                    if let Some(ns) = ns {
                        ns.get_href() != INK || !matches!(key.as_str(), "label" | "groupmode")
                    } else {
                        !matches!(key.as_str(), "id" | "transform")
                    }
                })
            {
                return Err("changed ancestors carry style, effects, locks or viewport metadata; prepare plain groups first".into());
            }
        }
        let ids = subtree
            .iter()
            .chain(divergent.iter().copied())
            .filter_map(|n| n.get_property_no_ns("id"))
            .filter(|id| !id.is_empty())
            .collect::<HashSet<_>>();
        if nodes
            .iter()
            .filter(|n| !subtree_pointers.contains(&(n.node_ptr() as usize)))
            .any(|n| !references(n).is_disjoint(&ids))
        {
            return Err("external references to moved subtree may change appearance".into());
        }
        composed(node.clone())?;
        let old_matrix = composed(old)?;
        let new_matrix = composed(dest.clone())?;
        inverse(old_matrix)?;
        let transform = multiply(
            multiply(inverse(new_matrix)?, old_matrix)?,
            parse(&node.get_property_no_ns("transform").unwrap_or_default())?,
        )?;
        if !self.preserve && old_matrix != new_matrix {
            return Err(
                "reparenting without preserve_appearance changes document transform".into(),
            );
        }
        let mut tails = Vec::new();
        let mut next = node.get_next_sibling();
        while let Some(tail) = next.filter(|n| {
            matches!(
                n.get_type(),
                Some(NodeType::TextNode | NodeType::CDataSectionNode | NodeType::EntityRefNode)
            )
        }) {
            if !self.preserve
                && (tail.get_type() == Some(NodeType::EntityRefNode)
                    || !tail.get_content().trim().is_empty())
            {
                return Err("reparenting with mixed text tails requires preparation".into());
            }
            next = tail.get_next_sibling();
            tails.push(tail);
        }
        let before = structure::paint_order(root.clone());
        // Match lxml's fresh nsN binding when a formerly inherited prefix
        // conflicts with a binding at the destination.
        let destination_namespaces = dest
            .get_namespaces(document)
            .into_iter()
            .map(|n| (n.get_prefix(), n.get_href()))
            .collect::<HashMap<_, _>>();
        let declared = node
            .get_namespace_declarations()
            .into_iter()
            .map(|n| n.get_prefix())
            .collect::<HashSet<_>>();
        let mut used = HashSet::new();
        for member in &subtree {
            if let Some(ns) = member.get_namespace() {
                used.insert((ns.get_prefix(), ns.get_href()));
            }
            for (_, ns) in member.get_properties_ns().keys() {
                if let Some(ns) = ns {
                    used.insert((ns.get_prefix(), ns.get_href()));
                }
            }
        }
        let conflicts = node
            .get_namespaces(document)
            .into_iter()
            .filter(|ns| {
                let prefix = ns.get_prefix();
                let href = ns.get_href();
                !declared.contains(&prefix)
                    && destination_namespaces
                        .get(&prefix)
                        .is_some_and(|uri| uri != &href)
                    && used.contains(&(prefix, href))
            })
            .map(|ns| (ns.get_prefix(), ns.get_href()))
            .collect::<Vec<_>>();
        node.unlink_node();
        dest.add_child(&mut node)
            .map_err(|_| "reparent edit failed")?;
        for (_, href) in conflicts {
            let visible = node.get_namespaces(document);
            let prefix = (0..)
                .map(|n| format!("ns{n}"))
                .find(|prefix| !visible.iter().any(|ns| ns.get_prefix() == *prefix))
                .unwrap();
            Namespace::new(&prefix, &href, &mut node)
                .map_err(|_| "reparent namespace reconciliation failed")?;
        }
        // SAFETY: both pointers belong to the same live disposable Document; the
        // subtree is attached before reconciling formerly inherited bindings.
        if unsafe { libxml::bindings::xmlReconciliateNs(document.doc_ptr(), node.node_ptr()) } < 0 {
            return Err("reparent namespace reconciliation failed".into());
        }
        for mut tail in tails {
            tail.unlink_node();
            if self.preserve {
                dest.add_child(&mut tail)
                    .map_err(|_| "reparent edit failed")?;
            }
        }
        if before != structure::paint_order(root) {
            return Err(
                "move changes global paint order; overlap preservation is ambiguous".into(),
            );
        }
        if self.preserve {
            node.set_property(
                "transform",
                &format!(
                    "matrix({})",
                    transform
                        .into_iter()
                        .map(general)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )
            .map_err(|_| "reparent edit failed")?;
            Ok(format!(
                "reparented {}, preserving document transform and paint order",
                style::python_repr(&self.id)
            ))
        } else {
            Ok(format!(
                "reparented {} under {}",
                style::python_repr(&self.id),
                style::python_repr(&self.dest)
            ))
        }
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(args)?;
    let mut result = transaction::apply_dom(
        registry,
        id,
        "reparent_object",
        mutation.params.clone(),
        "medium",
        None,
        |d| mutation.mutate(d),
    )?;
    result["object_id"] = json!(mutation.id);
    result["bbox"] = Value::Null;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_serialization_keeps_seventeen_digits_and_signed_zero() {
        for (v, s) in [
            (0., "0"),
            (-0., "-0"),
            (1000., "1000"),
            (1e-5, "1.0000000000000001e-05"),
            (1e17, "1e+17"),
            (1.5, "1.5"),
        ] {
            assert_eq!(general(v), s);
        }
        assert_eq!(
            parse("translate(10,20) scale(2)").unwrap(),
            [2., 0., 0., 2., 10., 20.]
        );
        assert!(parse("translate(1,2,3)").is_err());
        assert!(inverse([0.; 6]).is_err());
    }
    #[test]
    fn compensation_preserves_paint_order_and_unsafe_context_refuses() {
        let mut d=crate::xml::parse(br#"<svg><g id="old" transform="translate(10)"><rect id="a"/></g><g id="new" transform="scale(2)"/></svg>"#,4096).unwrap();
        let order = structure::paint_order(d.get_root_element().unwrap());
        Mutation::build(&json!({"object_id":"a","new_parent_id":"new","preserve_appearance":true}))
            .unwrap()
            .mutate(&mut d)
            .unwrap();
        assert_eq!(order, structure::paint_order(d.get_root_element().unwrap()));
        assert!(
            d.to_string()
                .contains("transform=\"matrix(0.5,0,0,0.5,5,0)\"")
        );
        for src in [
            br#"<svg><g id="old" opacity=".5"><rect id="a"/></g><g id="new"/></svg>"#.as_slice(),
            br##"<svg><g id="old"><rect id="a"/></g><g id="new"/><use href="#a"/></svg>"##,
        ] {
            let mut d = crate::xml::parse(src, 4096).unwrap();
            let before = d.to_string();
            assert!(
                Mutation::build(
                    &json!({"object_id":"a","new_parent_id":"new","preserve_appearance":true})
                )
                .unwrap()
                .mutate(&mut d)
                .is_err()
            );
            assert_eq!(before, d.to_string());
        }
    }
    #[test]
    fn moved_subtree_keeps_inherited_namespace_binding() {
        let mut d=crate::xml::parse(br##"<svg xmlns="http://www.w3.org/2000/svg"><g id="old" xmlns:l="http://www.w3.org/1999/xlink"><g id="r"><rect id="q"/><use l:href="#q"/></g></g><g id="new"/></svg>"##,4096).unwrap();
        Mutation::build(&json!({"object_id":"r","new_parent_id":"new","preserve_appearance":true}))
            .unwrap()
            .mutate(&mut d)
            .unwrap();
        let serialized = crate::xml::serialize(&d);
        let parsed = crate::xml::parse(&serialized, 4096).unwrap();
        let used = elements(parsed.get_root_element().unwrap())
            .into_iter()
            .find(|n| n.get_name() == "use")
            .unwrap();
        assert_eq!(
            used.get_property_ns("href", "http://www.w3.org/1999/xlink")
                .as_deref(),
            Some("#q")
        );
    }
}
