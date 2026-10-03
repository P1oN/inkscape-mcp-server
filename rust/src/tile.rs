//! Row-major copies reuse the ID/reference/tail kernel and bound staged bytes.
use crate::{
    arguments,
    document::{Registry, elements},
    duplicate, style, transaction, transform, xml,
};
use libxml::tree::{Document, NodeType};
use serde_json::{Value, json};
use std::collections::HashSet;
pub struct Mutation {
    id: String,
    rows: i64,
    cols: i64,
    dx: f64,
    dy: f64,
    pub params: Value,
}
fn integer(args: &Value, key: &str) -> Result<i64, String> {
    let v = &args[key];
    if let Some(n) = v.as_i64() {
        return Ok(n);
    }
    if let Some(b) = v.as_bool() {
        return Ok(i64::from(b));
    }
    if let Some(s) = v.as_str()
        && let Ok(n) = s.trim().parse()
    {
        return Ok(n);
    }
    if let Some(n) = arguments::number(v)
        && n.is_finite()
        && n.fract() == 0.
        && n >= i64::MIN as f64
        && n < i64::MAX as f64
    {
        return Ok(n as i64);
    }
    Err(format!("{key} must be an integer"))
}
impl Mutation {
    pub fn build(args: &Value) -> Result<Self, String> {
        let id = args["object_id"]
            .as_str()
            .ok_or("object_id must be a string")?
            .to_owned();
        let rows = integer(args, "rows")?;
        let cols = integer(args, "cols")?;
        let dx = arguments::optional_number(args, "dx")?.ok_or("dx must be a number")?;
        let dy = arguments::optional_number(args, "dy")?.ok_or("dy must be a number")?;
        let params = json!({"object_id":id,"rows":rows,"cols":cols,"dx":dx,"dy":dy});
        Ok(Self {
            id,
            rows,
            cols,
            dx,
            dy,
            params,
        })
    }
    pub fn mutate(&self, document: &mut Document, max_bytes: usize) -> Result<String, String> {
        if self.rows < 1 || self.cols < 1 {
            return Err("tile rows and cols must each be >= 1".into());
        }
        if self
            .rows
            .checked_mul(self.cols)
            .is_none_or(|count| count > 1024)
        {
            return Err(format!(
                "tile grid too large: {}x{} exceeds 1024 cells",
                self.rows, self.cols
            ));
        }
        if !self.dx.is_finite() {
            return Err("dx must be finite".into());
        }
        if !self.dy.is_finite() {
            return Err("dy must be finite".into());
        }
        let nodes = elements(
            document
                .get_root_element()
                .ok_or("document could not be parsed safely")?,
        );
        let source = nodes
            .iter()
            .find(|n| n.get_property_no_ns("id").as_ref() == Some(&self.id))
            .cloned()
            .ok_or("object id not found in document")?;
        if !source.get_parent().is_some_and(|p| p.is_element_node()) {
            return Err(format!(
                "object {} cannot be tiled (it is the document root)",
                style::python_repr(&self.id)
            ));
        }
        let count = self.rows * self.cols - 1;
        if count > 0 {
            duplicate::validate_copy(document, &source)?;
        }
        let mut ids = nodes
            .iter()
            .filter_map(|n| n.get_property_no_ns("id"))
            .collect::<HashSet<_>>();
        let mut after = source.clone();
        let mut staged_bytes = xml::serialize(document).len();
        for r in 0..self.rows {
            for c in 0..self.cols {
                if r == 0 && c == 0 {
                    continue;
                }
                let x = c as f64 * self.dx;
                let y = r as f64 * self.dy;
                let mut cloned = duplicate::insert_copy(document, &source, &after, None, &mut ids)?;
                if x != 0. || y != 0. {
                    let translation = format!(
                        "translate({},{})",
                        transform::number(x)?,
                        transform::number(y)?
                    );
                    let old = cloned.get_property_no_ns("transform").unwrap_or_default();
                    let value = if old.trim().is_empty() {
                        translation
                    } else {
                        format!("{translation} {}", old.trim())
                    };
                    cloned
                        .set_property("transform", &value)
                        .map_err(|_| "transform edit failed")?;
                }
                let mut added = document.node_to_string(&cloned).len();
                let mut next = cloned.get_next_sibling();
                while let Some(tail) = next.filter(|n| {
                    matches!(
                        n.get_type(),
                        Some(
                            NodeType::TextNode
                                | NodeType::CDataSectionNode
                                | NodeType::EntityRefNode
                        )
                    )
                }) {
                    added = added
                        .checked_add(document.node_to_string(&tail).len())
                        .ok_or("input file exceeds the configured size limit")?;
                    next = tail.get_next_sibling();
                }
                staged_bytes = staged_bytes
                    .checked_add(added)
                    .ok_or("input file exceeds the configured size limit")?;
                if staged_bytes > max_bytes {
                    return Err("input file exceeds the configured size limit".into());
                }
                after = cloned;
            }
        }
        Ok(format!(
            "tiled {} into a {}x{} grid ({count} clones added)",
            style::python_repr(&self.id),
            self.rows,
            self.cols
        ))
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let m = Mutation::build(args)?;
    transaction::apply_dom(
        registry,
        id,
        "tile",
        m.params.clone(),
        "medium",
        None,
        |d| m.mutate(d, registry.workspace.max_input),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_major_parent_space_and_one_cell_noop() {
        let mut d = xml::parse(
            br#"<svg><rect id="r" transform="scale(2)"/>tail<circle id="last"/></svg>"#,
            4096,
        )
        .unwrap();
        let before = xml::serialize(&d);
        Mutation::build(&json!({"object_id":"r","rows":1,"cols":1,"dx":10,"dy":20}))
            .unwrap()
            .mutate(&mut d, 4096)
            .unwrap();
        assert_eq!(before, xml::serialize(&d));
        Mutation::build(&json!({"object_id":"r","rows":2,"cols":2,"dx":10,"dy":20}))
            .unwrap()
            .mutate(&mut d, 4096)
            .unwrap();
        let nodes = elements(d.get_root_element().unwrap());
        assert_eq!(
            nodes[2].get_property_no_ns("transform").as_deref(),
            Some("translate(10,0) scale(2)")
        );
        assert_eq!(
            nodes[3].get_property_no_ns("transform").as_deref(),
            Some("translate(0,20) scale(2)")
        );
        assert_eq!(
            nodes[4].get_property_no_ns("transform").as_deref(),
            Some("translate(10,20) scale(2)")
        );
        assert_eq!(nodes[5].get_property_no_ns("id").as_deref(), Some("last"));
    }
    #[test]
    fn oversized_grid_and_invalid_offsets_precede_dom_changes() {
        let mut d = xml::parse(b"<svg><rect id='r'/></svg>", 4096).unwrap();
        let before = xml::serialize(&d);
        for a in [
            json!({"object_id":"r","rows":33,"cols":32,"dx":0,"dy":0}),
            json!({"object_id":"r","rows":1,"cols":1,"dx":"inf","dy":0}),
        ] {
            assert!(Mutation::build(&a).unwrap().mutate(&mut d, 4096).is_err());
            assert_eq!(before, xml::serialize(&d));
        }
        assert!(
            Mutation::build(&json!({"object_id":"r","rows":1,"cols":4,"dx":10,"dy":0}))
                .unwrap()
                .mutate(&mut d, 30)
                .is_err()
        );
    }
}
