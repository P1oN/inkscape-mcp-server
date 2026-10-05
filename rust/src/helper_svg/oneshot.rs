//! Private fixed INX request. Transport owns context/approvals and publication.
use super::{
    apply::{self, Applied},
    edit, fingerprint,
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub nonce: String,
    pub expected_ids: Vec<String>,
    pub expected_fingerprint: String,
    #[serde(default)]
    pub operation: Option<edit::Operation>,
    #[serde(default)]
    pub selection: Vec<String>,
    #[serde(default)]
    pub style: BTreeMap<String, String>,
    #[serde(default)]
    pub transform: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub fragment: Option<String>,
    #[serde(default)]
    pub edits: Option<Vec<super::package::Change>>,
}
pub fn prepare(
    svg: &str,
    request: Request,
    native_selection: &[String],
    cap: usize,
) -> Result<(Applied, String), &'static str> {
    let document = crate::xml::parse(svg.as_bytes(), cap)?;
    let root = document
        .get_root_element()
        .ok_or("invalid document or selection")?;
    if root.get_name() != "svg"
        || root.get_namespace().map(|n| n.get_href()).as_deref()
            != Some("http://www.w3.org/2000/svg")
    {
        return Err("invalid document or selection");
    }
    edit::guard(
        svg,
        &request.expected_fingerprint,
        &request.expected_ids,
        &request.selection,
        // Insertion is independent of selection, including INX's root fallback --id.
        if request.operation.is_some() || request.edits.is_some() {
            native_selection
        } else {
            &request.selection
        },
        cap,
    )?;
    let applied = if let Some(edits) = request.edits {
        if request.operation.is_some()
            || request.fragment.is_some()
            || !request.style.is_empty()
            || request.transform.is_some()
            || request.text.is_some()
        {
            return Err("invalid document or selection");
        }
        super::package::prepare(svg, &request.selection, &edits, &request.nonce, cap)?
    } else if let Some(operation) = request.operation {
        if request.fragment.is_some() {
            return Err("invalid document or selection");
        }
        apply::edit(
            svg,
            &edit::Request {
                nonce: request.nonce,
                operation,
                selection: request.selection,
                style: request.style,
                transform: request.transform,
                text: request.text,
            },
            cap,
        )?
    } else {
        if !request.selection.is_empty()
            || !request.style.is_empty()
            || request.transform.is_some()
            || request.text.is_some()
        {
            return Err("invalid document or selection");
        }
        apply::insert(
            svg,
            request
                .fragment
                .as_deref()
                .ok_or("invalid document or selection")?,
            &request.nonce,
            cap,
        )?
    };
    let fingerprint = fingerprint::fingerprint(
        if applied.bytes.is_empty() {
            svg
        } else {
            std::str::from_utf8(&applied.bytes).map_err(|_| "invalid document or selection")?
        },
        cap,
    )?;
    Ok((applied, fingerprint))
}
