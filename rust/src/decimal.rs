//! Python-compatible finite or nonfinite numeric text decoding; callers enforce their own limits.
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;
static NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[+-]?(?:(?:\d(?:_?\d)*)(?:\.(?:\d(?:_?\d)*)?)?|\.(?:\d(?:_?\d)*))(?:[eE][+-]?(?:\d(?:_?\d)*))?$").unwrap()
});
static STARTS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    let value: Value = serde_json::from_str(include_str!(
        "../../migration/contracts/unicode-decimal-starts.json"
    ))
    .unwrap();
    value["starts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect()
});
pub fn float(text: &str) -> Option<f64> {
    let text = text.trim();
    match text.to_ascii_lowercase().as_str() {
        "nan" | "+nan" | "-nan" => return Some(f64::NAN),
        "inf" | "infinity" | "+inf" | "+infinity" => return Some(f64::INFINITY),
        "-inf" | "-infinity" => return Some(f64::NEG_INFINITY),
        _ => (),
    }
    if !NUMBER.is_match(text) {
        return None;
    }
    let mut normalized = String::new();
    for c in text.chars().filter(|c| *c != '_') {
        if c.is_ascii() {
            normalized.push(c);
        } else {
            let code = c as u32;
            let start = STARTS
                .iter()
                .find(|start| code >= **start && code < **start + 10)?;
            normalized.push(char::from_u32('0' as u32 + code - start)?);
        }
    }
    normalized.parse().ok()
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn compiled_python_numeric_unicode_underscores_nonfinite_and_invalid_text_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/numeric-text-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let actual = match float(case["text"].as_str().unwrap()) {
                None => json!({"invalid":true}),
                Some(n) if n.is_finite() => json!({"number":n}),
                Some(n) => {
                    json!({"nonfinite":if n.is_nan(){"nan"}else if n>0.0{"inf"}else{"-inf"}})
                }
            };
            assert_eq!(actual, case["expected"], "{}", case["text"]);
        }
    }
}
