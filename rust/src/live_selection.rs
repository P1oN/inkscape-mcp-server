//! Managed stdout selection grammar: complete numeric fence distinguishes empty from incomplete.
use crate::live_socket::Error;
use regex::Regex;
use std::{collections::HashSet, sync::LazyLock};
static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$").unwrap());
static LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+) cloned: (?:true|false) ref: \d+ href: \d+ total href: \d+$").unwrap()
});
fn space(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}
pub fn parse(text: &str) -> Result<Option<Vec<String>>, Error> {
    if text.len() > 1024 * 1024 {
        return Err(Error::Protocol(
            "managed selection response exceeds the size cap",
        ));
    }
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    let mut remaining = text;
    while !remaining.is_empty() {
        let end = remaining.char_indices().find(|(_, c)| {
            matches!(
                c,
                '\n' | '\r'
                    | '\u{b}'
                    | '\u{c}'
                    | '\u{1c}'
                    | '\u{1d}'
                    | '\u{1e}'
                    | '\u{85}'
                    | '\u{2028}'
                    | '\u{2029}'
            )
        });
        let Some((index, c)) = end else {
            return Ok(None);
        };
        let mut length = index + c.len_utf8();
        if c == '\r' && remaining[length..].starts_with('\n') {
            length += 1;
        }
        let (line, tail) = remaining.split_at(length);
        remaining = tail;
        if !line.ends_with('\n') {
            return Ok(None);
        }
        let line = line.trim_matches(space);
        if line.split(',').all(|part| NUMBER.is_match(part)) {
            return Ok(Some(ids));
        }
        if let Some(captures) = LINE.captures(line) {
            let id = &captures[1];
            if !id.chars().any(space) {
                if seen.insert(id.to_string()) {
                    ids.push(id.to_string());
                }
                if ids.len() > 10_000 {
                    return Err(Error::Protocol(
                        "managed selection response exceeds the size cap",
                    ));
                }
                continue;
            }
        }
        if !line.is_empty() {
            return Err(Error::Protocol(
                "unexpected selection response from managed Inkscape",
            ));
        }
    }
    Ok(None)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn compiled_python_fence_partial_unicode_dedup_and_error_cases_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/selection-reply-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let actual = match parse(case["text"].as_str().unwrap()) {
                Ok(value) => json!({"result":value}),
                Err(Error::Protocol(message)) => json!({"error":message}),
                Err(error) => json!({"error":error.public_message()}),
            };
            assert_eq!(actual, case["expected"], "{}", case["text"]);
        }
    }
    #[test]
    fn bounded_selection_data_and_unique_count_refuse() {
        assert!(parse(&"x".repeat(1024 * 1024 + 1)).is_err());
        let text = (0..10_001)
            .map(|i| format!("id{i} cloned: true ref: 1 href: 0 total href: 0\n"))
            .collect::<String>();
        assert!(parse(&(text + "1\n")).is_err());
    }
}
