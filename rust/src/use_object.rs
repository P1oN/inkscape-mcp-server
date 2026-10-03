//! Same-document editable instances; bounded transform grammar, never external hrefs.
use crate::{
    arguments, create,
    document::{Registry, elements},
    identity, style, transaction,
};
use libxml::tree::Document;
use regex::Regex;
use serde_json::{Value, json};
use std::sync::LazyLock;
static TRANSFORM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:(?:matrix|translate|scale|rotate|skewX|skewY)\s*\([\d\s,.+\-eE]*\)\s*)+$")
        .unwrap()
});
pub struct Mutation {
    href: String,
    parent: Option<String>,
    id: Option<String>,
    x: Option<String>,
    y: Option<String>,
    transform: Option<String>,
    pub params: Value,
}
impl Mutation {
    pub fn build(args: &Value) -> Result<Self, String> {
        let href = args["href_id"]
            .as_str()
            .ok_or("href_id must be a string")?
            .to_owned();
        if !create::valid_id(&href) {
            return Err(format!("invalid href id: {}", style::python_repr(&href)));
        }
        let x = arguments::optional_number(args, "x")?;
        let y = arguments::optional_number(args, "y")?;
        let sx = x.map(|v| create::numeric(v, "x", 0)).transpose()?;
        let sy = y.map(|v| create::numeric(v, "y", 0)).transpose()?;
        let transform = arguments::string(args, "transform")?.map(str::to_owned);
        if let Some(raw) = &transform {
            let length = raw.chars().count();
            if length > 2000 {
                return Err(format!("transform too long: {length} > 2000 characters"));
            }
            if !TRANSFORM.is_match(raw) {
                return Err(format!(
                    "invalid transform value: {}",
                    style::python_repr(raw)
                ));
            }
        }
        let parent = arguments::string(args, "parent_id")?.map(str::to_owned);
        let id = arguments::string(args, "object_id")?.map(str::to_owned);
        let params = json!({"href_id":href,"parent_id":parent,"x":x,"y":y});
        Ok(Self {
            href,
            parent,
            id,
            x: sx,
            y: sy,
            transform,
            params,
        })
    }
    pub fn create(&self, document: &mut Document) -> Result<(String, String), String> {
        let nodes = elements(
            document
                .get_root_element()
                .ok_or("document could not be parsed safely")?,
        );
        if !nodes
            .iter()
            .any(|n| n.get_property_no_ns("id").as_ref() == Some(&self.href))
        {
            return Err(format!(
                "href target {} not found in document",
                style::python_repr(&self.href)
            ));
        }
        let mut parent = create::resolve_parent(document, self.parent.as_deref())?;
        let id = create::resolve_id(document, self.id.as_deref(), "use")?;
        let mut node = create::svg_node("use", &parent, document)?;
        node.set_property("id", &id)
            .map_err(|_| "vector creation failed")?;
        parent
            .add_child(&mut node)
            .map_err(|_| "vector creation failed")?;
        let href = format!("#{}", self.href);
        identity::set_namespaced(
            &mut node,
            document,
            "http://www.w3.org/1999/xlink",
            "href",
            &href,
        )?;
        node.set_property("href", &href)
            .map_err(|_| "vector creation failed")?;
        for (key, value) in [
            ("x", self.x.as_ref()),
            ("y", self.y.as_ref()),
            ("transform", self.transform.as_ref()),
        ] {
            if let Some(value) = value {
                node.set_property(key, value)
                    .map_err(|_| "vector creation failed")?;
            }
        }
        Ok((
            id.clone(),
            format!(
                "created <use> {} -> #{}",
                style::python_repr(&id),
                self.href
            ),
        ))
    }
    pub fn mutate(&self, document: &mut Document) -> Result<String, String> {
        Ok(self.create(document)?.1)
    }
}
pub fn apply(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let mutation = Mutation::build(args)?;
    let mut created = String::new();
    let mut result = transaction::apply_dom(
        registry,
        id,
        "create_use",
        mutation.params.clone(),
        "medium",
        None,
        |document| {
            let (id, summary) = mutation.create(document)?;
            created = id;
            Ok(summary)
        },
    )?;
    result["object_id"] = json!(created);
    result["bbox"] = Value::Null;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_reference_and_transform_grammar() {
        for href in [
            "#r",
            "http://example.com/r",
            "url(#r)",
            "javascript:alert(1)",
        ] {
            assert!(Mutation::build(&json!({"href_id":href})).is_err());
        }
        for transform in [
            "",
            "translate(1);display:none",
            "unknown(2)",
            "translate(1)".repeat(200).as_str(),
        ] {
            assert!(Mutation::build(&json!({"href_id":"r","transform":transform})).is_err());
        }
        let mut document = crate::xml::parse(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r"/></svg>"#,
            4096,
        )
        .unwrap();
        Mutation::build(
            &json!({"href_id":"r","object_id":"instance","transform":"translate(2,3) scale(2)"}),
        )
        .unwrap()
        .mutate(&mut document)
        .unwrap();
        assert!(document.to_string().contains("ns0:href=\"#r\" href=\"#r\""));
    }
}
