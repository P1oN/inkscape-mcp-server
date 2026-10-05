//! Bounded style/text packages on a fixed native selection; one candidate publication.
use super::{apply, edit, fingerprint, valid_nonce};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Change {
    Style {
        #[serde(default)]
        style: BTreeMap<String, String>,
        #[serde(default)]
        transform: Option<String>,
    },
    Text {
        text: String,
    },
}
pub fn prepare(
    svg: &str,
    selection: &[String],
    edits: &[Change],
    nonce: &str,
    cap: usize,
) -> Result<apply::Applied, &'static str> {
    if !valid_nonce(nonce)
        || edits.is_empty()
        || edits.len() > 16
        || selection.is_empty()
        || selection.len() > 1000
    {
        return Err("invalid document or selection");
    }
    let mut candidate = svg.to_string();
    let mut ids = BTreeSet::new();
    for change in edits {
        let (operation, style, transform, text) = match change {
            Change::Style { style, transform } => (
                edit::Operation::Style,
                style.clone(),
                transform.clone(),
                None,
            ),
            Change::Text { text } => (
                edit::Operation::Text,
                BTreeMap::new(),
                None,
                Some(text.clone()),
            ),
        };
        let applied = apply::edit(
            &candidate,
            &edit::Request {
                nonce: nonce.into(),
                operation,
                selection: selection.to_vec(),
                style,
                transform,
                text,
            },
            cap,
        )?;
        ids.extend(applied.ids);
        if !applied.bytes.is_empty() {
            candidate =
                String::from_utf8(applied.bytes).map_err(|_| "invalid document or selection")?;
        }
    }
    let unchanged =
        fingerprint::fingerprint(svg, cap)? == fingerprint::fingerprint(&candidate, cap)?;
    Ok(apply::Applied {
        bytes: if unchanged {
            vec![]
        } else {
            candidate.into_bytes()
        },
        ids: ids.into_iter().collect(),
    })
}
