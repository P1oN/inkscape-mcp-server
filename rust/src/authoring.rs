//! One policy source; discovery snapshots contain its expanded text for wire checks.
pub const GUIDANCE: &str = include_str!("../../migration/contracts/authoring-guidance.txt");
const MARKER: &str = "\n\nEditable vector authoring quality:\n";
pub fn expand(template: &str) -> String {
    let (prefix, _) = template
        .split_once(MARKER)
        .expect("authoring guidance marker");
    format!("{prefix}{MARKER}{}", GUIDANCE.trim_end())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_discovery_snapshots_and_compose_share_policy() {
        for entry in std::fs::read_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../migration/contracts"
        ))
        .unwrap()
        {
            let path = entry.unwrap().path();
            if path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("_raw-")
            {
                let v: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
                let text = v["initialize"]["instructions"].as_str().unwrap();
                assert_eq!(expand(text), text);
                assert_eq!(text.matches(GUIDANCE.trim_end()).count(), 1);
                let request = serde_json::from_value(serde_json::json!({"name":"compose_artwork","arguments":{"goal":"  flower  \n folds \n snowball "}})).unwrap();
                let rendered = crate::prompts::render(&v, request).unwrap();
                let rmcp::model::GetPromptResponse::Complete(rendered) = rendered else {
                    panic!("compose must complete")
                };
                let rendered = serde_json::to_value(rendered).unwrap();
                let prompt = rendered["messages"][0]["content"]["text"].as_str().unwrap();
                assert_eq!(prompt.matches(GUIDANCE.trim_end()).count(), 1);
                assert!(prompt.contains("flower folds snowball"));
            }
        }
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../../migration/contracts/prompt-messages.json"
        ))
        .unwrap();
        let text = v["compose_artwork"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();
        assert_eq!(expand(text), text);
    }
}
