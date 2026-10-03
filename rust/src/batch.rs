//! Every typed DOM batch member reuses its single-edit kernel in one transaction.

use crate::{
    canvas, create, delete, document::Registry, duplicate, fragment, gradient, group, identity,
    recolor, reparent, repeat, structure, style, text, tile, transaction, transform, use_object,
};
use serde_json::{Value, json};

enum Mutation {
    Style(style::Mutation),
    Canvas(canvas::Mutation),
    Identity(identity::Mutation),
    Delete(delete::Mutation),
    Create(create::Mutation),
    Gradient(gradient::Mutation),
    Group(group::Mutation),
    Use(use_object::Mutation),
    Structure(structure::Mutation),
    Reparent(reparent::Mutation),
    Duplicate(duplicate::Mutation),
    Tile(tile::Mutation),
    Text(text::Mutation),
    Transform(transform::Mutation),
    Recolor(recolor::Mutation),
    Repeat(repeat::Mutation),
    Fragment(fragment::Mutation),
}
impl Mutation {
    fn mutate(
        &self,
        document: &mut libxml::tree::Document,
        max_bytes: usize,
    ) -> Result<String, String> {
        match self {
            Self::Repeat(m) => m.mutate(document, max_bytes),
            Self::Fragment(m) => m.mutate(document),
            Self::Tile(m) => m.mutate(document, max_bytes),
            Self::Duplicate(m) => m.mutate(document),
            Self::Reparent(m) => m.mutate(document),
            Self::Structure(m) => m.mutate(document),
            Self::Use(m) => m.mutate(document),
            Self::Group(m) => m.mutate(document),
            Self::Gradient(m) => m.mutate(document),
            Self::Create(m) => m.mutate(document),
            Self::Delete(m) => m.mutate(document),
            Self::Identity(m) => m.mutate(document),
            Self::Canvas(m) => m.mutate(document),
            Self::Style(m) => m.mutate(document),
            Self::Text(m) => m.mutate(document),
            Self::Transform(m) => m.mutate(document),
            Self::Recolor(m) => m.mutate(document),
        }
    }
}

pub fn apply(registry: &Registry, arguments: &Value) -> Result<Value, String> {
    apply_named(registry, arguments, "apply_edits", None)
}

pub(crate) fn apply_named(
    registry: &Registry,
    arguments: &Value,
    tool: &str,
    match_count: Option<usize>,
) -> Result<Value, String> {
    let id = arguments["doc_id"]
        .as_str()
        .ok_or("doc_id must be a string")?;
    let edits = arguments["edits"]
        .as_array()
        .ok_or("edits must be a list")?;
    if edits.is_empty() {
        return Err("apply_edits requires at least one edit".into());
    }
    if edits.len() > 64 {
        return Err("apply_edits batch exceeds 64 edits".into());
    }
    let mut mutations = Vec::new();
    let mut operations = Vec::new();
    for edit in edits {
        let operation = edit["op"]
            .as_str()
            .ok_or("batch edit requires an op field")?;
        match operation {
            "create_rect" | "create_circle" | "create_ellipse" | "create_line"
            | "create_polygon" | "create_polyline" | "create_path" | "create_text" => {
                mutations.push(Mutation::Create(create::Mutation::build(operation, edit)?))
            }
            "add_linear_gradient" | "add_radial_gradient" => mutations.push(Mutation::Gradient(
                gradient::Mutation::build(operation, edit)?,
            )),
            "create_group" | "set_group_mode" => {
                mutations.push(Mutation::Group(group::Mutation::build(operation, edit)?))
            }
            "create_use" => mutations.push(Mutation::Use(use_object::Mutation::build(edit)?)),
            "group_objects" => {
                mutations.push(Mutation::Structure(structure::Mutation::build(edit)?))
            }
            "reparent_object" => {
                mutations.push(Mutation::Reparent(reparent::Mutation::build(edit)?))
            }
            "repeat_objects" => mutations.push(Mutation::Repeat(repeat::Mutation::build(edit)?)),
            "replace_svg_fragment" => mutations.push(Mutation::Fragment(
                fragment::Mutation::build(edit, registry.workspace.max_input)?,
            )),
            "tile" => mutations.push(Mutation::Tile(tile::Mutation::build(edit)?)),
            "duplicate_object" => {
                mutations.push(Mutation::Duplicate(duplicate::Mutation::build(edit)?))
            }
            "delete_object" => mutations.push(Mutation::Delete(delete::Mutation::build(edit)?)),
            "rename_object" => mutations.push(Mutation::Identity(identity::Mutation::build(edit)?)),
            "resize_canvas" | "normalize_viewbox" => {
                mutations.push(Mutation::Canvas(canvas::Mutation::build(operation, edit)?))
            }
            "move_object" | "scale_object" | "rotate_object" => mutations.push(
                Mutation::Transform(transform::Mutation::build(operation, edit)?),
            ),
            "replace_text" | "set_font" => {
                mutations.push(Mutation::Text(text::Mutation::build(operation, edit)?))
            }
            "set_fill" | "set_stroke" | "set_opacity" => {
                mutations.push(Mutation::Style(style::Mutation::build(operation, edit)?))
            }
            "replace_color" | "apply_palette" => mutations.push(Mutation::Recolor(
                recolor::Mutation::build(operation, edit)?,
            )),
            _ => {
                return Err(format!(
                    "Rust migration pending: batch member {operation} has not been ported"
                ));
            }
        }
        operations.push(operation.to_string());
    }
    let risk = if operations
        .iter()
        .any(|op| ["delete_object", "replace_svg_fragment"].contains(&op.as_str()))
    {
        "high"
    } else {
        "medium"
    };
    let mut params = json!({"edit_count":edits.len(),"ops":operations});
    if let Some(count) = match_count {
        params["match_count"] = json!(count);
    }
    let mut result = transaction::apply_dom(
        registry,
        id,
        tool,
        params,
        risk,
        crate::arguments::string(arguments, "approval_token")?,
        |document| {
            let mut summaries = Vec::new();
            for (name, mutation) in operations.iter().zip(&mutations) {
                summaries.push(format!(
                    "{name}: {}",
                    mutation.mutate(document, registry.workspace.max_input)?
                ));
            }
            Ok(format!(
                "applied {} edit(s) | {}",
                mutations.len(),
                summaries.join(" | ")
            ))
        },
    )?;
    result["edit_count"] = json!(edits.len());
    result["risk_class"] = json!(risk);
    Ok(result)
}
