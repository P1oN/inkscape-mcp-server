//! Pydantic argument text grammar, distinct from Python float() used for SVG data.
use regex::Regex;
use std::sync::LazyLock;

static INTEGER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[+-]?[0-9](?:_?[0-9])*(?:\.0+)?$").unwrap());

pub fn integer_text(text: &str) -> bool {
    // Do not round through f64: a fractional digit after many zeroes remains invalid,
    // and large integral strings retain their exact value for later typed extraction.
    INTEGER.is_match(text.trim())
}

pub fn integer_text_size_exceeded(text: &str) -> bool {
    // Called only after grammar validation. Pydantic's decimal conversion discards
    // underscores, the positive sign, leading zeroes and the zero fractional tail;
    // its 4300-character bound includes the negative sign for a nonzero value.
    let text = text.trim();
    let whole = text
        .trim_start_matches(['+', '-'])
        .split('.')
        .next()
        .unwrap_or("");
    let significant = whole
        .bytes()
        .filter(|c| *c != b'_')
        .skip_while(|c| *c == b'0')
        .count();
    significant + usize::from(text.starts_with('-') && significant != 0) > 4300
}

pub fn integer_value(text: &str) -> Option<serde_json::Value> {
    if !integer_text(text) {
        return None;
    }
    let text = text.trim();
    let negative = text.starts_with('-');
    let whole = text.trim_start_matches(['+', '-']).split('.').next()?;
    let digits = whole.replace('_', "");
    let digits = digits.trim_start_matches('0');
    let canonical = if digits.is_empty() {
        "0".to_owned()
    } else if negative {
        format!("-{digits}")
    } else {
        digits.to_owned()
    };
    serde_json::from_str(&canonical).ok()
}

pub fn float_text(text: &str) -> Option<f64> {
    let text = text.trim();
    if !text.is_ascii() || text.starts_with('_') || text.ends_with('_') || text.contains("__") {
        return None;
    }
    // Pydantic accepts isolated interior underscores, including beside '.' or exponent
    // punctuation. Rust's float parser supplies the remaining ASCII grammar.
    let normalized = text.replace('_', "");
    normalized.parse().ok()
}
