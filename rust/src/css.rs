//! The reference inspector's deliberately small CSS cascade, without fetching stylesheets.
use crate::inspect;
use libxml::tree::Node;
use regex::Regex;
use std::sync::{Arc, LazyLock};

static COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());
static BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^{}]+)\{([^{}]*)\}").unwrap());
static SIMPLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:([A-Za-z][\w-]*)|\.([\w-]+)|#([\w-]+)|\*)$").unwrap());

pub struct Rule {
    tag: Option<String>,
    class: Option<String>,
    id: Option<String>,
    declarations: Arc<indexmap::IndexMap<String, String>>,
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
            let declarations = Arc::new(inspect::parse_declarations(&block[2]));
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

/// Conservative cascade for authoring advice. Unsupported selectors/declarations are
/// uncertainty, never evidence that an object is well formed or safe to delete.
/// Legacy inspector behavior above remains unchanged.
pub struct Analysis {
    rules: Vec<Rule>,
    unknown: Option<String>,
    budget: std::cell::Cell<usize>,
}
impl Analysis {
    pub fn new(nodes: &[Node]) -> Self {
        let mut analysis = Self {
            rules: vec![],
            unknown: None,
            budget: std::cell::Cell::new(2_000_000),
        };
        let mut bytes = 0usize;
        for node in nodes {
            if matches!(
                node.get_name().as_str(),
                "animate" | "animateTransform" | "animateMotion" | "set" | "script"
            ) {
                analysis.unknown = Some("Animated styling is outside static analysis.".into());
            }
            if node.get_name() != "style" {
                continue;
            }
            if node
                .get_namespace()
                .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
                || node
                    .get_property_no_ns("media")
                    .is_some_and(|s| !s.trim().is_empty())
                || node
                    .get_property_no_ns("type")
                    .is_some_and(|s| s != "text/css")
            {
                analysis.unknown = Some("Conditional or non-CSS stylesheet is unsupported.".into());
            }
            let raw = inspect::own_text(node);
            bytes = bytes.saturating_add(raw.len());
            if bytes > 262144 {
                analysis.unknown = Some("Stylesheet byte limit exceeded.".into());
                break;
            }
            let stripped = COMMENT.replace_all(&raw, "");
            let mut end = 0;
            for block in BLOCK.captures_iter(&stripped) {
                let whole = block.get(0).unwrap();
                if !stripped[end..whole.start()].trim().is_empty() {
                    analysis.unknown = Some("Unsupported stylesheet syntax.".into());
                }
                end = whole.end();
                let Some(declarations) = analysis_declarations(&block[2]) else {
                    analysis.unknown = Some("Unsupported CSS declaration syntax.".into());
                    continue;
                };
                let declarations = Arc::new(declarations);
                for selector in block[1].split(',') {
                    let Some(simple) = SIMPLE
                        .captures(selector.trim())
                        .filter(|_| analysis_selector(selector.trim()))
                    else {
                        analysis.unknown = Some(
                            "Only simple type, ID, class and universal selectors are supported."
                                .into(),
                        );
                        continue;
                    };
                    if analysis.rules.len() >= 2048 {
                        analysis.unknown = Some("Stylesheet rule limit exceeded.".into());
                        break;
                    }
                    if declarations.keys().any(|key| {
                        key.starts_with("animation")
                            || key.starts_with("transition")
                            || matches!(key.as_str(), "all" | "marker")
                    }) {
                        analysis.unknown = Some(
                            "Animated CSS or all/marker shorthands are outside static analysis."
                                .into(),
                        );
                    }
                    analysis.rules.push(Rule {
                        tag: simple.get(1).map(|m| m.as_str().into()),
                        class: simple.get(2).map(|m| m.as_str().into()),
                        id: simple.get(3).map(|m| m.as_str().into()),
                        declarations: declarations.clone(),
                        order: analysis.rules.len(),
                    });
                }
            }
            if !stripped[end..].trim().is_empty() {
                analysis.unknown = Some("Unsupported stylesheet syntax or at-rule.".into());
            }
        }
        // An external stylesheet can change any computed property; never fetch it.
        if let Some(parent) = nodes.first().and_then(Node::get_parent) {
            for sibling in parent.get_child_nodes() {
                if sibling.get_name() == "xml-stylesheet" {
                    analysis.unknown = Some("External stylesheet is not inspected.".into());
                }
            }
        }
        analysis
    }
    fn spend(&self, amount: usize) -> Result<(), String> {
        let remaining = self.budget.get();
        if amount > remaining {
            self.budget.set(0);
            return Err("CSS analysis work limit exceeded.".into());
        }
        self.budget.set(remaining - amount);
        Ok(())
    }
    fn own_value(&self, node: &Node, prop: &str) -> Result<Option<String>, String> {
        if let Some(reason) = &self.unknown {
            return Err(reason.clone());
        }
        if node
            .get_namespace()
            .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
        {
            return Err("Foreign namespace presentation styling is unsupported.".into());
        }
        self.spend(self.rules.len() + 1)?;
        // importance, inline specificity, ID/class/type specificity, source order.
        let mut best = (false, false, 0usize, 0usize, 0usize, 0usize);
        let mut value = node.get_property_no_ns(prop);
        let classes = node.get_property_no_ns("class").unwrap_or_default();
        let id = node.get_property_no_ns("id");
        let tag = node.get_name();
        self.spend(
            classes
                .len()
                .saturating_mul(self.rules.len().max(1))
                .saturating_add(id.as_ref().map_or(0, String::len))
                .saturating_add(value.as_ref().map_or(0, String::len)),
        )?;
        for rule in &self.rules {
            if rule.tag.as_ref().is_some_and(|s| *s != tag)
                || rule.id.as_ref().is_some_and(|s| Some(s) != id.as_ref())
                || rule
                    .class
                    .as_ref()
                    .is_some_and(|s| !classes.split_whitespace().any(|c| c == s))
            {
                continue;
            }
            if let Some(v) = rule.declarations.get(prop) {
                self.spend(v.len())?;
                let (v, important) = importance(v);
                let rank = (
                    important,
                    false,
                    usize::from(rule.id.is_some()),
                    usize::from(rule.class.is_some()),
                    usize::from(rule.tag.is_some()),
                    rule.order + 1,
                );
                if rank >= best {
                    best = rank;
                    value = Some(v.into());
                }
            }
        }
        let style = node.get_property_no_ns("style").unwrap_or_default();
        self.spend(style.len())?;
        let declarations = analysis_declarations(&style).ok_or("Unsupported inline CSS syntax.")?;
        if declarations.keys().any(|key| {
            key.starts_with("animation")
                || key.starts_with("transition")
                || matches!(key.as_str(), "all" | "marker")
        }) {
            return Err(
                "Animated inline CSS or all/marker shorthands are outside static analysis.".into(),
            );
        }
        if let Some(v) = declarations.get(prop) {
            let (v, important) = importance(v);
            if (important, true, 0, 0, 0, 0) >= best {
                value = Some(v.into());
            }
        }
        Ok(value.map(|s| s.trim().into()))
    }
    pub fn computed(&self, node: &Node, prop: &str) -> Result<String, String> {
        let (default, inherited) = match prop {
            "fill" => ("black", true),
            "stroke" => ("none", true),
            "visibility" => ("visible", true),
            "fill-opacity" | "stroke-opacity" => ("1", true),
            "opacity" => ("1", false),
            "marker-start" | "marker-mid" | "marker-end" => ("none", true),
            "display" => ("inline", false),
            _ => ("none", false),
        };
        let mut current = Some(node.clone());
        for _ in 0..128 {
            let Some(n) = current else {
                return Ok(default.into());
            };
            let own = self.own_value(&n, prop)?;
            match own.as_deref() {
                Some("initial") => return Ok(default.into()),
                Some("inherit") => {}
                Some("unset") | None if inherited => {}
                Some("unset") | None => return Ok(default.into()),
                Some("revert" | "revert-layer") => {
                    return Err("CSS cascade rollback is unsupported.".into());
                }
                Some(value)
                    if value.contains("var(")
                        || value.contains("env(")
                        || value.contains("calc(") =>
                {
                    return Err("Dynamic CSS values are unsupported.".into());
                }
                Some(value) => return Ok(value.into()),
            }
            current = n.get_parent().filter(Node::is_element_node);
        }
        Err("CSS ancestor depth limit exceeded.".into())
    }
}
fn importance(value: &str) -> (&str, bool) {
    if let Some((value, suffix)) = value.rsplit_once('!')
        && suffix.trim().eq_ignore_ascii_case("important")
    {
        return (value.trim(), true);
    }
    (value, false)
}
fn analysis_declarations(raw: &str) -> Option<indexmap::IndexMap<String, String>> {
    if raw.len() > 65536 || raw.contains(['\\', '{', '}', '\"', '\'']) {
        return None;
    }
    let raw = COMMENT.replace_all(raw, "");
    let mut declarations = indexmap::IndexMap::new();
    for part in raw.split(';').filter(|s| !s.trim().is_empty()) {
        let (key, value) = part.split_once(':')?;
        let key = key.trim().to_ascii_lowercase();
        if key.is_empty()
            || !key.bytes().all(|c| c.is_ascii_alphabetic() || c == b'-')
            || value.trim().is_empty()
        {
            return None;
        }
        let value = value.trim();
        if value.contains('!') && !importance(value).1 {
            return None;
        }
        if declarations
            .get(&key)
            .is_none_or(|old: &String| !importance(old).1 || importance(value).1)
        {
            declarations.insert(key, value.into());
        }
    }
    Some(declarations)
}

fn analysis_selector(selector: &str) -> bool {
    use cssparser::{Parser, Token};
    let mut parser = Parser::new(selector);
    let valid = match parser.next().ok().cloned() {
        Some(Token::Ident(_) | Token::IDHash(_) | Token::Delim('*')) => true,
        Some(Token::Delim('.')) => matches!(parser.next(), Ok(Token::Ident(_))),
        _ => false,
    };
    valid && parser.is_exhausted()
}
