//! Private live semantics, deliberately separate from headless mutation policy.
pub mod affine;
pub mod edit;
pub mod fingerprint;
pub mod fragment;
use libxml::tree::Node;
pub fn elements(root: Node) -> Vec<Node> {
    let mut stack = vec![root];
    let mut result = Vec::new();
    while let Some(node) = stack.pop() {
        stack.extend(node.get_child_elements().into_iter().rev());
        result.push(node);
    }
    result
}
pub fn valid_nonce(value: &str) -> bool {
    value.strip_prefix("mcp_").is_some_and(|token| {
        token.len() == 32
            && token
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    })
}

pub mod apply;

pub mod oneshot;

pub mod socket;

pub mod package;
