//! Fixed private effect data: document fingerprint and nonce-bound reply validation.
use crate::live_socket::Error;
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
pub fn fingerprint(svg: &str, cap: usize) -> Result<String, Error> {
    inkscape_mcp_rust::helper_svg::fingerprint::fingerprint(svg, cap).map_err(Error::Protocol)
}
#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    Applied {
        fingerprint: String,
        ids: Vec<String>,
    },
    Refused {
        reason: String,
    },
}
pub fn refusal(reason: &str) -> String {
    static REASONS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/effect-data-cases.json"
        ))
        .unwrap();
        value["refusals"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect()
    });
    let reason = if REASONS.iter().any(|r| r == reason) {
        reason
    } else {
        "invalid document or selection"
    };
    format!("Inkscape refused edit: {reason}")
}
const INVALID: &str = "invalid edit reply; inspect the task drawing before retrying";
pub fn edit_reply(bytes: &[u8], nonce: &str) -> Result<Reply, Error> {
    if bytes.len() > 1024 * 1024 {
        return Err(Error::Uncertain);
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| Error::Uncertain)?;
    if !value.is_object() || value["nonce"] != nonce {
        return Err(Error::Uncertain);
    }
    match value["ok"].as_bool() {
        Some(true) => {
            let fingerprint = value["fingerprint"]
                .as_str()
                .ok_or(Error::Uncertain)?
                .to_string();
            let ids = value["ids"]
                .as_array()
                .ok_or(Error::Uncertain)?
                .iter()
                .map(|v| v.as_str().map(str::to_string).ok_or(Error::Uncertain))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Reply::Applied { fingerprint, ids })
        }
        Some(false) => {
            let reason = match value.get("error") {
                None => "",
                Some(value) => value.as_str().ok_or(Error::Uncertain)?,
            };
            Ok(Reply::Refused {
                reason: reason.to_string(),
            })
        }
        None => Err(Error::Uncertain),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiled_python_fingerprints_preserve_namespace_text_tail_metadata_and_attr_order() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/effect-data-cases.json"
        ))
        .unwrap();
        for case in fixture["fingerprints"].as_array().unwrap() {
            assert_eq!(
                fingerprint(case["svg"].as_str().unwrap(), 1024 * 1024).unwrap(),
                case["expected"].as_str().unwrap(),
                "{}",
                case["svg"]
            );
        }
    }
    #[test]
    fn compiled_python_nonce_discriminator_result_and_refusal_reply_cases_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/effect-data-cases.json"
        ))
        .unwrap();
        for case in fixture["replies"].as_array().unwrap() {
            let actual = match edit_reply(case["text"].as_str().unwrap().as_bytes(), "mcp_test") {
                Ok(Reply::Applied { fingerprint, ids }) => {
                    assert_eq!(case["expected"]["reply"]["ok"], true);
                    assert_eq!(
                        fingerprint,
                        case["expected"]["reply"]["fingerprint"].as_str().unwrap()
                    );
                    assert_eq!(json!(ids), case["expected"]["reply"]["ids"]);
                    case["expected"].clone()
                }
                Ok(Reply::Refused { reason }) => {
                    assert_eq!(case["expected"]["reply"]["ok"], false);
                    assert_eq!(
                        reason,
                        case["expected"]["reply"]
                            .get("error")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                    );
                    case["expected"].clone()
                }
                Err(Error::Uncertain) => json!({"error":INVALID}),
                _ => panic!("unexpected effect error"),
            };
            assert_eq!(actual, case["expected"], "{}", case["text"]);
        }
        for reason in fixture["refusals"].as_array().unwrap() {
            let reason = reason.as_str().unwrap();
            assert_eq!(refusal(reason), format!("Inkscape refused edit: {reason}"));
        }
        assert_eq!(
            refusal("private /host/path detail"),
            "Inkscape refused edit: invalid document or selection"
        );
    }
}
/// Prepared fixed effect files. Caller holds the managed cross-client operation lock.
pub struct Exchange {
    workspace: crate::workspace::Workspace,
    directory: std::path::PathBuf,
    nonce: String,
    finished: std::cell::Cell<bool>,
}
impl Exchange {
    pub fn prepare(
        directory: &std::path::Path,
        payload: &Value,
        cap: usize,
    ) -> Result<Self, Error> {
        let nonce = payload["nonce"]
            .as_str()
            .ok_or(Error::Protocol("invalid edit identity"))?;
        let token = nonce
            .strip_prefix("mcp_")
            .ok_or(Error::Protocol("invalid edit identity"))?;
        if token.len() != 32
            || !token
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            return Err(Error::Protocol("invalid edit identity"));
        }
        if !directory.is_absolute() {
            return Err(Error::Protocol("managed edit directory is unavailable"));
        }
        let relative = directory
            .strip_prefix("/")
            .map_err(|_| Error::Protocol("managed edit directory is unavailable"))?
            .to_path_buf();
        let workspace = crate::workspace::Workspace {
            roots: vec!["/".into()],
            max_input: cap,
            max_output: cap,
        };
        let bytes = serde_json::to_vec(payload)
            .map_err(|_| Error::Protocol("could not prepare managed edit request"))?;
        if bytes.len() > cap {
            return Err(Error::Protocol("managed edit request exceeds size cap"));
        }
        for name in [
            "insert-request.json",
            "insert-result.json",
            "insert-request.tmp",
        ] {
            workspace
                .regular_info(0, &relative.join(name), false)
                .map_err(|_| Error::Protocol("could not prepare managed edit request"))?;
        }
        workspace
            .regular_info(0, &relative.join("insert-result.json"), true)
            .map_err(|_| Error::Protocol("could not prepare managed edit request"))?;
        let exchange = Self {
            workspace,
            directory: relative,
            nonce: nonce.into(),
            finished: std::cell::Cell::new(false),
        };
        exchange
            .workspace
            .atomic_write(0, &exchange.directory.join("insert-request.json"), &bytes)
            .map_err(|_| Error::Protocol("could not prepare managed edit request"))?;
        Ok(exchange)
    }
    pub fn read(&self) -> Result<Option<Reply>, Error> {
        self.read_bytes()?
            .map(|bytes| edit_reply(&bytes, &self.nonce))
            .transpose()
    }
    fn read_bytes(&self) -> Result<Option<Vec<u8>>, Error> {
        if self.finished.get() {
            return Err(Error::Uncertain);
        }
        self.workspace
            .read_optional(0, &self.directory.join("insert-result.json"), 1024 * 1024)
            .map_err(|_| Error::Uncertain)
    }
    /// The insertion helper returns no fingerprint; confirmation uses the
    /// minted root group and all planned IDs in a fresh scoped SVG export.
    pub fn read_insert(&self) -> Result<Option<bool>, Error> {
        self.read_bytes()?
            .map(|bytes| {
                let value: Value = serde_json::from_slice(&bytes).map_err(|_| Error::Uncertain)?;
                if !value.is_object() || value["nonce"] != self.nonce {
                    return Err(Error::Uncertain);
                }
                value["ok"].as_bool().ok_or(Error::Uncertain)
            })
            .transpose()
    }
    pub fn cleanup(&self) -> Result<(), Error> {
        if self.finished.replace(true) {
            return Ok(());
        }
        for name in [
            "insert-request.json",
            "insert-result.json",
            "insert-request.tmp",
        ] {
            self.workspace
                .regular_info(0, &self.directory.join(name), true)
                .map_err(|_| Error::Uncertain)?;
        }
        Ok(())
    }
}
impl Drop for Exchange {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
#[cfg(test)]
mod exchange_tests {
    use super::*;
    use std::{os::unix::fs::PermissionsExt, path::Path};
    const NONCE: &str = "mcp_1234567890abcdef1234567890abcdef";
    fn payload() -> Value {
        json!({"nonce":NONCE,"operation":"style","selection":["r"],"style":{"fill":"red"}})
    }
    #[test]
    fn insertion_reply_requires_nonce_and_boolean_without_edit_fingerprint() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let exchange = Exchange::prepare(&root, &payload(), 4096).unwrap();
        assert_eq!(exchange.read_insert(), Ok(None));
        for (value, expected) in [
            (json!({"nonce":NONCE,"ok":true}), Ok(Some(true))),
            (
                json!({"nonce":NONCE,"ok":false,"error":"private detail"}),
                Ok(Some(false)),
            ),
            (json!({"nonce":"stale","ok":true}), Err(Error::Uncertain)),
            (json!({"nonce":NONCE,"ok":1}), Err(Error::Uncertain)),
            (json!({"nonce":NONCE,"ok":"true"}), Err(Error::Uncertain)),
            (json!({"nonce":NONCE}), Err(Error::Uncertain)),
            (json!([]), Err(Error::Uncertain)),
        ] {
            std::fs::write(
                root.join("insert-result.json"),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
            assert_eq!(exchange.read_insert(), expected);
        }
        for bytes in [b"invalid".to_vec(), vec![b'x'; 1024 * 1024 + 1]] {
            std::fs::write(root.join("insert-result.json"), bytes).unwrap();
            assert_eq!(exchange.read_insert(), Err(Error::Uncertain));
        }
        exchange.cleanup().unwrap();
        assert_eq!(exchange.read_insert(), Err(Error::Uncertain));
    }
    #[test]
    fn private_exchange_is_atomic_mode_600_nonce_bound_and_cleanup_is_once_only() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("insert-result.json"), b"old stale reply").unwrap();
        let exchange = Exchange::prepare(&root, &payload(), 1024).unwrap();
        let request = root.join("insert-request.json");
        assert_eq!(
            std::fs::metadata(&request).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&std::fs::read(&request).unwrap()).unwrap(),
            payload()
        );
        assert_eq!(exchange.read().unwrap(), None);
        std::fs::write(
            root.join("insert-result.json"),
            serde_json::to_vec(
                &json!({"nonce":NONCE,"ok":true,"fingerprint":"confirmed","ids":["r"]}),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            exchange.read().unwrap(),
            Some(Reply::Applied {
                fingerprint: "confirmed".into(),
                ids: vec!["r".into()]
            })
        );
        exchange.cleanup().unwrap();
        assert!(!request.exists());
        std::fs::write(&request, b"later request owned by the next operation").unwrap();
        drop(exchange);
        assert_eq!(
            std::fs::read(request).unwrap(),
            b"later request owned by the next operation"
        );
    }
    #[test]
    fn exchange_links_caps_stale_and_malformed_replies_refuse_without_touching_originals() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let original = root.join("original.svg");
        std::fs::write(&original, b"keep original").unwrap();
        for name in [
            "insert-request.json",
            "insert-result.json",
            "insert-request.tmp",
        ] {
            std::os::unix::fs::symlink(&original, root.join(name)).unwrap();
            assert!(Exchange::prepare(&root, &payload(), 1024).is_err());
            assert_eq!(std::fs::read(&original).unwrap(), b"keep original");
            std::fs::remove_file(root.join(name)).unwrap();
        }
        assert!(Exchange::prepare(Path::new("relative"), &payload(), 1024).is_err());
        assert!(Exchange::prepare(&root, &payload(), 3).is_err());
        let exchange = Exchange::prepare(&root, &payload(), 1024).unwrap();
        for bytes in [
            b"{invalid".as_slice(),
            br#"{"nonce":"stale","ok":true,"ids":[],"fingerprint":"x"}"#,
        ] {
            std::fs::write(root.join("insert-result.json"), bytes).unwrap();
            assert_eq!(exchange.read(), Err(Error::Uncertain));
        }
        std::fs::write(root.join("insert-result.json"), vec![b'x'; 1024 * 1024 + 1]).unwrap();
        assert_eq!(exchange.read(), Err(Error::Uncertain));
        std::fs::remove_file(root.join("insert-result.json")).unwrap();
        std::os::unix::fs::symlink(&original, root.join("insert-result.json")).unwrap();
        assert_eq!(exchange.read(), Err(Error::Uncertain));
        assert_eq!(exchange.cleanup(), Err(Error::Uncertain));
        assert_eq!(std::fs::read(original).unwrap(), b"keep original");
    }
    #[test]
    fn fingerprint_namespace_expansion_is_bounded_before_effect_activation() {
        let svg = format!(
            "<svg xmlns='urn:{}'>{}</svg>",
            "x".repeat(4096),
            "<rect/>".repeat(100)
        );
        assert_eq!(
            fingerprint(&svg, svg.len()),
            Err(Error::Protocol("document fingerprint exceeds size cap"))
        );
    }
}
