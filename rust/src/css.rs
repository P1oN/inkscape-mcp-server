//! The reference inspector's deliberately small CSS cascade, without fetching stylesheets.
use crate::inspect;
use libxml::tree::Node;
use regex::Regex;
use std::sync::LazyLock;

static COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());
static BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^{}]+)\{([^{}]*)\}").unwrap());
static SIMPLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:([A-Za-z][\w-]*)|\.([\w-]+)|#([\w-]+)|\*)$").unwrap());

pub struct Rule {
    tag: Option<String>,
    class: Option<String>,
    id: Option<String>,
    declarations: indexmap::IndexMap<String, String>,
    order: usize,
}

pub fn rules(nodes: &[Node]) -> Vec<Rule> {
    let mut rules = Vec::new();
    for n in nodes.iter().filter(|n| n.get_name() == "style") {
        let raw = inspect::own_text(n);
        let stripped = COMMENT.replace_all(&raw, "");
        let mut order = 0;
        for block in BLOCK.captures_iter(&stripped) {
            if block[1].contains('@') {
                continue;
            }
            let declarations = inspect::parse_declarations(&block[2]);
            if declarations.is_empty() {
                continue;
            }
            for selector in block[1].split(',') {
                if let Some(key) = selector.split_whitespace().last()
                    && let Some(simple) = SIMPLE.captures(key)
                {
                    rules.push(Rule {
                        tag: simple.get(1).map(|m| m.as_str().to_owned()),
                        class: simple.get(2).map(|m| m.as_str().to_owned()),
                        id: simple.get(3).map(|m| m.as_str().to_owned()),
                        declarations: declarations.clone(),
                        order,
                    });
                }
            }
            order += 1;
        }
    }
    rules
}

pub fn own(node: &Node, rules: &[Rule], prop: &str) -> Option<String> {
    let mut value = None;
    let mut best = None;
    let tag = node.get_name();
    let id = node.get_property_no_ns("id");
    let classes = node.get_property_no_ns("class").unwrap_or_default();
    for r in rules {
        if r.tag.as_ref().is_some_and(|t| t != &tag)
            || r.id.as_ref().is_some_and(|i| Some(i) != id.as_ref())
            || r.class
                .as_ref()
                .is_some_and(|c| !classes.split_whitespace().any(|s| s == c))
        {
            continue;
        }
        let rank = (
            (
                usize::from(r.id.is_some()),
                usize::from(r.class.is_some()),
                usize::from(r.tag.is_some()),
            ),
            r.order,
        );
        if let Some(v) = r.declarations.get(prop)
            && best.is_none_or(|b| rank > b)
        {
            best = Some(rank);
            value = Some(v.clone());
        }
    }
    if let Some(v) = node.get_property_no_ns(prop) {
        value = Some(v);
    }
    if let Some(v) =
        inspect::parse_declarations(&node.get_property_no_ns("style").unwrap_or_default()).get(prop)
    {
        value = Some(v.clone());
    }
    value.map(|v| v.trim().to_owned())
}

pub fn inherited(node: &Node, rules: &[Rule], prop: &str) -> Option<String> {
    let mut current = Some(node.clone());
    while let Some(n) = current {
        if let Some(v) = own(&n, rules, prop)
            && !v.eq_ignore_ascii_case("inherit")
        {
            return Some(v);
        }
        current = n.get_parent().filter(|p| p.is_element_node());
    }
    None
}
