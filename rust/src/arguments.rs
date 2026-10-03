//! Typed scalar extraction. Wrong non-null optional values never become absent edits.
use serde_json::Value;

pub fn string<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(format!("{key} must be a string")),
    }
}

pub fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.trim().parse().ok()))
        .or_else(|| value.as_bool().map(|b| if b { 1.0 } else { 0.0 }))
}

pub fn optional_number(args: &Value, key: &str) -> Result<Option<f64>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => number(value)
            .map(Some)
            .ok_or_else(|| format!("{key} must be a number")),
    }
}

pub fn boolean(args: &Value, key: &str, default: bool) -> Result<bool, String> {
    let Some(value) = args.get(key) else {
        return Ok(default);
    };
    if let Some(b) = value.as_bool() {
        return Ok(b);
    }
    if let Some(n) = value.as_f64() {
        if n == 1.0 {
            return Ok(true);
        }
        if n == 0.0 {
            return Ok(false);
        }
    }
    if let Some(s) = value.as_str() {
        match s.to_lowercase().as_str() {
            "true" | "1" | "y" | "yes" | "on" => return Ok(true),
            "false" | "0" | "n" | "no" | "off" => return Ok(false),
            _ => (),
        }
    }
    Err(format!("{key} must be a boolean"))
}
