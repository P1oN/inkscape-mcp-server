//! Native schema validation before state locks, approvals or IPC.
//! Scalar/container shape and missing fields reuse the frozen ordered schemas.
//! Validated values are materialized separately; custom model/union validation is incomplete.
use serde_json::Value;
use std::sync::LazyLock;
static MODELS: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../migration/contracts/argument-model-names.json"
    ))
    .expect("frozen model metadata")
});
static ENUMS: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../migration/contracts/argument-enum-paths.json"
    ))
    .expect("frozen enum metadata")
});
static FINITE: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../migration/contracts/argument-finite-number-paths.json"
    ))
    .expect("frozen model finite-number metadata")
});
struct Errors {
    tool: String,
    messages: Vec<String>,
}
impl std::ops::Deref for Errors {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}
impl std::ops::DerefMut for Errors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}
fn wildcard_path(path: &str) -> String {
    path.split('.')
        .map(|part| {
            if !part.is_empty() && part.bytes().all(|c| c.is_ascii_digit()) {
                "*"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}
fn model_name(errors: &Errors, path: &str) -> Option<String> {
    let path = wildcard_path(path);
    MODELS[format!("{}.{path}", errors.tool)]
        .as_str()
        .map(str::to_owned)
}

// Only retain a bounded prefix and suffix while traversing diagnostic values.
struct Excerpt {
    prefix: String,
    suffix: std::collections::VecDeque<char>,
    count: usize,
    limit: usize,
}
impl Excerpt {
    fn new(limit: usize) -> Self {
        Self {
            prefix: String::new(),
            suffix: std::collections::VecDeque::with_capacity(limit - limit / 2 - 1),
            count: 0,
            limit,
        }
    }
    fn push(&mut self, character: char) {
        self.count = self.count.saturating_add(1);
        if self.count <= self.limit {
            self.prefix.push(character);
        }
        let suffix_limit = self.limit - self.limit / 2 - 1;
        if self.suffix.len() == suffix_limit {
            self.suffix.pop_front();
        }
        self.suffix.push_back(character);
    }
    fn text(&mut self, text: &str) {
        for character in text.chars() {
            self.push(character);
        }
    }
    fn finish(self) -> String {
        if self.count <= self.limit {
            self.prefix
        } else {
            format!(
                "{}...{}",
                self.prefix.chars().take(self.limit / 2).collect::<String>(),
                self.suffix.iter().collect::<String>()
            )
        }
    }
}
fn excerpt(value: &str, limit: usize) -> String {
    let mut output = Excerpt::new(limit);
    output.text(value);
    output.finish()
}
fn string_repr(value: &str, output: &mut Excerpt) {
    let quote = if value.contains('\'') && !value.contains('"') {
        '"'
    } else {
        '\''
    };
    output.push(quote);
    for character in value.chars() {
        match character {
            '\n' => output.text("\\n"),
            '\r' => output.text("\\r"),
            '\t' => output.text("\\t"),
            '\\' => output.text("\\\\"),
            character if character == quote => {
                output.push('\\');
                output.push(character);
            }
            character if character.is_control() => {
                output.text(&format!("\\x{:02x}", character as u32))
            }
            character => output.push(character),
        }
    }
    output.push(quote);
}
fn emit_repr(value: &Value, output: &mut Excerpt) {
    match value {
        Value::Null => output.text("None"),
        Value::Bool(value) => output.text(if *value { "True" } else { "False" }),
        Value::Number(value) => output.text(value.as_str()),
        Value::String(value) => string_repr(value, output),
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.text(", ");
                }
                emit_repr(value, output);
            }
            output.push(']');
        }
        Value::Object(values) => {
            output.push('{');
            for (index, (key, value)) in values.iter().enumerate() {
                if index > 0 {
                    output.text(", ");
                }
                string_repr(key, output);
                output.text(": ");
                emit_repr(value, output);
            }
            output.push('}');
        }
    }
}
fn repr(value: &Value) -> String {
    let mut output = Excerpt::new(50);
    emit_repr(value, &mut output);
    output.finish()
}
fn repr_full(value: &Value) -> String {
    match value {
        Value::Array(v) => format!(
            "[{}]",
            v.iter().map(repr_full).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(v) => format!(
            "{{{}}}",
            v.iter()
                .map(|(k, v)| format!("{}: {}", crate::style::python_repr(k), repr_full(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::String(v) => crate::style::python_repr(v),
        Value::Null => "None".to_owned(),
        Value::Bool(v) => if *v { "True" } else { "False" }.to_owned(),
        Value::Number(v) => v.to_string(),
    }
}
fn input_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::String(_) => "str",
        Value::Number(n) => {
            if !n.as_str().contains(['.', 'e', 'E']) {
                "int"
            } else {
                "float"
            }
        }
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}
fn error(errors: &mut Errors, path: &str, kind: &str, message: &str, input: &Value) {
    if errors.len() > 4096 {
        return;
    }
    let path = excerpt(path, 256);
    errors.push(format!("\n{path}\n  {message} [type={kind}, input_value={}, input_type={}]\n    For further information visit https://errors.pydantic.dev/2.13/v/{kind}",repr(input),input_type(input)));
}
fn fields(schema: &Value, input: &Value, path: &str, call: bool, errors: &mut Errors) {
    let Some(props) = schema["properties"].as_object() else {
        return;
    };
    let required = schema["required"].as_array();
    for (key, field) in props {
        let location = if path.is_empty() {
            key.to_owned()
        } else {
            format!("{path}.{key}")
        };
        if let Some(v) = input.get(key) {
            check(field, v, &location, errors);
        } else if required.is_some_and(|r| r.iter().any(|v| v.as_str() == Some(key))) {
            error(
                errors,
                &location,
                if call { "missing_argument" } else { "missing" },
                if call {
                    "Missing required argument"
                } else {
                    "Field required"
                },
                input,
            );
        }
    }
    if (call || schema["additionalProperties"] == false)
        && let Some(arguments) = input.as_object()
    {
        for (key, value) in arguments {
            if errors.len() > 4096 {
                break;
            }
            if !props.contains_key(key) {
                let location = if path.is_empty() {
                    key.to_owned()
                } else {
                    format!("{path}.{key}")
                };
                error(
                    errors,
                    &location,
                    if call {
                        "unexpected_keyword_argument"
                    } else {
                        "extra_forbidden"
                    },
                    if call {
                        "Unexpected keyword argument"
                    } else {
                        "Extra inputs are not permitted"
                    },
                    value,
                );
            }
        }
    }
}
fn check(schema: &Value, input: &Value, path: &str, errors: &mut Errors) {
    if errors.len() > 4096 {
        return;
    }
    if let Some(choices) = schema["anyOf"].as_array() {
        if input.is_null() && choices.iter().any(|s| s["type"] == "null") {
            return;
        }
        let nonnull: Vec<_> = choices.iter().filter(|s| s["type"] != "null").collect();
        if nonnull.len() == 1 {
            check(nonnull[0], input, path, errors);
        }
        return; // Multi-type unions require ordered branch-specific error capture.
    }
    if let Some(choices) = schema["oneOf"].as_array() {
        if !input.is_object() {
            error(
                errors,
                path,
                "model_attributes_type",
                "Input should be a valid dictionary or object to extract fields from",
                input,
            );
            return;
        }
        let tag = choices
            .first()
            .and_then(|s| s["properties"].as_object())
            .and_then(|p| {
                p.iter()
                    .find(|(_, v)| v.get("const").is_some())
                    .map(|(k, _)| k)
            });
        if let Some(tag) = tag.filter(|tag| {
            choices
                .iter()
                .all(|s| s["properties"][*tag].get("const").is_some())
        }) {
            if input.is_object() && input.get(tag).is_none() {
                error(
                    errors,
                    path,
                    "union_tag_not_found",
                    &format!("Unable to extract tag using discriminator '{tag}'"),
                    input,
                );
            } else if let Some(choice) = choices
                .iter()
                .find(|s| s["properties"][tag]["const"] == input[tag])
            {
                let next = format!("{path}.{}", input[tag].as_str().unwrap_or(""));
                check(choice, input, &next, errors);
            } else {
                let actual = input[tag]
                    .as_str()
                    .map(|value| excerpt(value, 50))
                    .unwrap_or_else(|| repr(&input[tag]));
                let expected = choices
                    .iter()
                    .map(|choice| repr_full(&choice["properties"][tag]["const"]))
                    .collect::<Vec<_>>()
                    .join(", ");
                error(
                    errors,
                    path,
                    "union_tag_invalid",
                    &format!(
                        "Input tag '{actual}' found using '{tag}' does not match any of the expected tags: {expected}"
                    ),
                    input,
                );
            }
        }
        return;
    }
    if let Some(values) = schema["enum"].as_array() {
        if !values.contains(input) {
            let mut expected: Vec<_> = values.iter().map(repr_full).collect();
            let last = expected.pop().unwrap_or_default();
            let text = if expected.is_empty() {
                last
            } else {
                format!("{} or {last}", expected.join(", "))
            };
            error(
                errors,
                path,
                if ENUMS[format!("{}.{path}", errors.tool)].is_string() {
                    "enum"
                } else {
                    "literal_error"
                },
                &format!("Input should be {text}"),
                input,
            );
        }
        return;
    }
    let invalid = match schema["type"].as_str() {
        Some("string") => {
            (!input.is_string()).then_some(("string_type", "Input should be a valid string"))
        }
        Some("number") => {
            if input.is_number()
                && input_type(input) == "int"
                && !input.as_f64().is_some_and(f64::is_finite)
            {
                Some(("float_type", "Input should be a valid number"))
            } else if input.is_number() || input.is_boolean() {
                None
            } else if let Some(s) = input.as_str() {
                if crate::argument_numbers::float_text(s).is_some() {
                    None
                } else {
                    Some((
                        "float_parsing",
                        "Input should be a valid number, unable to parse string as a number",
                    ))
                }
            } else {
                Some(("float_type", "Input should be a valid number"))
            }
        }
        Some("integer") => {
            let too_large = Some((
                "int_parsing_size",
                "Unable to parse input string as an integer, exceeded maximum size",
            ));
            if input.is_boolean() || (input.is_number() && input_type(input) == "int") {
                None
            } else if let Some(n) = input.as_f64() {
                if !n.is_finite() {
                    Some(("finite_number", "Input should be a finite number"))
                } else if n.fract() != 0.0 {
                    Some((
                        "int_from_float",
                        "Input should be a valid integer, got a number with a fractional part",
                    ))
                } else if n.abs() >= 9223372036854775808.0 {
                    too_large
                } else {
                    None
                }
            } else if let Some(s) = input.as_str() {
                if crate::argument_numbers::integer_text(s) {
                    if crate::argument_numbers::integer_text_size_exceeded(s) {
                        too_large
                    } else {
                        None
                    }
                } else {
                    Some((
                        "int_parsing",
                        "Input should be a valid integer, unable to parse string as an integer",
                    ))
                }
            } else {
                Some(("int_type", "Input should be a valid integer"))
            }
        }
        Some("boolean") => {
            if crate::arguments::boolean(&serde_json::json!({"value":input}), "value", false)
                .is_ok()
            {
                None
            } else if input.is_string() || input.is_number() {
                Some((
                    "bool_parsing",
                    "Input should be a valid boolean, unable to interpret input",
                ))
            } else {
                Some(("bool_type", "Input should be a valid boolean"))
            }
        }
        Some("array") => {
            let prefix = schema["prefixItems"].as_array();
            if let Some(values) = input.as_array() {
                if values.len() > 10000 {
                    errors.push("argument validation list exceeds 10000 items".to_owned());
                    return;
                }
                let container = if prefix.is_some() { "Tuple" } else { "List" };
                if let Some(maximum) = schema["maxItems"].as_u64()
                    && values.len() as u64 > maximum
                {
                    error(
                        errors,
                        path,
                        "too_long",
                        &format!(
                            "{container} should have at most {maximum} items after validation, not {}",
                            values.len()
                        ),
                        input,
                    );
                    return;
                }
                let before = errors.len();
                if let Some(prefix) = prefix {
                    for (index, field) in prefix.iter().enumerate() {
                        if errors.len() > 4096 {
                            break;
                        }
                        let location = format!("{path}.{index}");
                        if let Some(value) = values.get(index) {
                            check(field, value, &location, errors);
                        } else {
                            error(errors, &location, "missing", "Field required", input);
                        }
                    }
                } else {
                    for (index, value) in values.iter().enumerate() {
                        if errors.len() > 4096 {
                            break;
                        }
                        check(&schema["items"], value, &format!("{path}.{index}"), errors);
                    }
                    if errors.len() == before
                        && let Some(minimum) = schema["minItems"].as_u64()
                        && (values.len() as u64) < minimum
                    {
                        error(
                            errors,
                            path,
                            "too_short",
                            &format!(
                                "List should have at least {minimum} items after validation, not {}",
                                values.len()
                            ),
                            input,
                        );
                    }
                }
                None
            } else if prefix.is_some() {
                Some(("tuple_type", "Input should be a valid tuple"))
            } else {
                Some(("list_type", "Input should be a valid list"))
            }
        }
        Some("object") => {
            if let Some(values) = input.as_object() {
                fields(schema, input, path, false, errors);
                if schema["additionalProperties"].is_object() {
                    for (key, value) in values {
                        if errors.len() > 4096 {
                            break;
                        }
                        if schema["properties"].get(key).is_none() {
                            check(
                                &schema["additionalProperties"],
                                value,
                                &format!("{path}.{key}"),
                                errors,
                            );
                        }
                    }
                }
                None
            } else {
                if let Some(name) = model_name(errors, path) {
                    error(
                        errors,
                        path,
                        "model_type",
                        &format!("Input should be a valid dictionary or instance of {name}"),
                        input,
                    );
                    return;
                }
                Some(("dict_type", "Input should be a valid dictionary"))
            }
        }
        _ => None,
    };
    if let Some((kind, message)) = invalid {
        error(errors, path, kind, message, input);
    } else if matches!(schema["type"].as_str(), Some("number" | "integer")) {
        let number = input
            .as_f64()
            .or_else(|| input.as_bool().map(|v| if v { 1.0 } else { 0.0 }))
            .or_else(|| input.as_str().and_then(crate::argument_numbers::float_text));
        if let Some(number) = number {
            if schema["type"] == "number"
                && FINITE[format!("{}.{}", errors.tool, wildcard_path(path))] == true
                && !number.is_finite()
            {
                error(
                    errors,
                    path,
                    "finite_number",
                    "Input should be a finite number",
                    input,
                );
                return;
            }
            for (key, kind, message, valid) in [
                ("maximum", "less_than_equal", "less than or equal to", 0),
                ("exclusiveMaximum", "less_than", "less than", 1),
                (
                    "minimum",
                    "greater_than_equal",
                    "greater than or equal to",
                    2,
                ),
                ("exclusiveMinimum", "greater_than", "greater than", 3),
            ] {
                if let Some(bound) = schema[key].as_f64() {
                    let passes = match valid {
                        0 => number <= bound,
                        1 => number < bound,
                        2 => number >= bound,
                        _ => number > bound,
                    };
                    if !passes {
                        error(
                            errors,
                            path,
                            kind,
                            &format!("Input should be {message} {bound}"),
                            input,
                        );
                        break;
                    }
                }
            }
        }
    }
}
pub fn validate(tool: &Value, arguments: &Value) -> Result<(), String> {
    let mut errors = Errors {
        tool: tool["name"]
            .as_str()
            .ok_or("invalid frozen tool name")?
            .to_owned(),
        messages: Vec::new(),
    };
    fields(&tool["inputSchema"], arguments, "", true, &mut errors);
    if let Some(bound) = errors.iter().find(|error| !error.starts_with('\n')) {
        return Err(bound.clone());
    }
    if errors.len() > 4096 {
        return Err("argument validation exceeds 4096 errors".to_owned());
    }
    if errors.is_empty() {
        return Ok(());
    }
    let name = tool["name"].as_str().ok_or("invalid frozen tool name")?;
    let count = errors.len();
    Err(format!(
        "{count} validation error{} for call[{name}]{}",
        if count == 1 { "" } else { "s" },
        errors.join("")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn diagnostic_repr_matches_existing_small_values_with_bounded_storage() {
        let values = [
            json!(null),
            json!(true),
            json!(-1.25),
            json!("a'\"\n\t\u{85}🌿"),
            json!([null,{"a'": [false,"🌿".repeat(60)]}]),
        ];
        for value in values {
            let full = repr_full(&value);
            let chars = full.chars().collect::<Vec<_>>();
            let expected = if chars.len() > 50 {
                format!(
                    "{}...{}",
                    chars[..25].iter().collect::<String>(),
                    chars[chars.len() - 24..].iter().collect::<String>()
                )
            } else {
                full
            };
            assert_eq!(repr(&value), expected);
        }
        let value = json!(["🌿\n".repeat(250_000), {"huge":"x".repeat(250_000)}]);
        let mut output = Excerpt::new(50);
        emit_repr(&value, &mut output);
        assert!(output.prefix.chars().count() <= 50);
        assert!(output.prefix.len() <= 200);
        assert_eq!(output.suffix.len(), 24);
        assert_eq!(output.finish().chars().count(), 52);
    }

    #[test]
    fn large_unknown_fields_and_discriminator_tags_have_bounded_errors() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let tool = |name| {
            discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == name)
                .unwrap()
        };
        let mut args = json!({"width":16,"height":16});
        args.as_object_mut()
            .unwrap()
            .insert("q".repeat(250_000), json!(["🌿".repeat(250_000)]));
        let message = validate(tool("create_document"), &args).unwrap_err();
        assert!(message.len() < 1200);
        assert!(message.contains("unexpected_keyword_argument"));
        let message = validate(
            tool("apply_edits"),
            &json!({"doc_id":"none","edits":[{"op":"tag".repeat(250_000)}]}),
        )
        .unwrap_err();
        assert!(message.len() < 2048);
        assert!(message.contains("union_tag_invalid"));
    }

    #[test]
    fn finite_model_errors_match_real_stdio_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-finite-field-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 576);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{} {}",
                case["name"],
                case["case"]
            );
        }
    }

    #[test]
    fn extreme_integer_errors_match_real_stdio_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-integer-expanded-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 487);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{} {}",
                case["name"],
                case["case"]
            );
        }
    }

    #[test]
    fn container_field_errors_match_real_stdio_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-container-expanded-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 78);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{} {}",
                case["name"],
                case["case"]
            );
        }
    }

    #[test]
    fn all_required_empty_errors_match_frozen_real_stdio_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/required-empty-errors.json"
        ))
        .unwrap();
        let cases = cases.as_array().unwrap();
        assert_eq!(cases.len(), 87);
        for case in cases {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &json!({})).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{}",
                case["name"]
            );
            assert_eq!(case["response"]["isError"], true);
        }
    }

    #[test]
    fn optional_signatures_and_nonempty_arguments_still_reach_typed_kernels() {
        assert!(validate(&json!({"name":"live_launch","inputSchema":{}}), &json!({})).is_ok());
        assert!(
            validate(
                &json!({"name":"inspect_document","inputSchema":{"required":["doc_id"]}}),
                &json!({"doc_id":"d_missing"})
            )
            .is_ok()
        );
    }

    #[test]
    fn all_required_null_errors_match_frozen_real_stdio_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/required-null-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 87);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{}",
                case["name"]
            );
        }
    }

    #[test]
    fn all_required_object_errors_match_frozen_real_stdio_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/required-object-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 87);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{}",
                case["name"]
            );
        }
    }

    #[test]
    fn invalid_field_and_missing_one_matrices_match_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        for (source, count) in [
            (
                include_str!("../../migration/contracts/argument-type-field-errors.json"),
                418,
            ),
            (
                include_str!("../../migration/contracts/argument-missing-one-errors.json"),
                147,
            ),
        ] {
            let cases: Value = serde_json::from_str(source).unwrap();
            assert_eq!(cases.as_array().unwrap().len(), count);
            for case in cases.as_array().unwrap() {
                let tool = discovery["tools/list"]["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|tool| tool["name"] == case["name"])
                    .unwrap();
                assert_eq!(
                    validate(tool, &case["arguments"]).unwrap_err(),
                    case["response"]["content"][0]["text"].as_str().unwrap(),
                    "{} {}",
                    case["name"],
                    case["case"]
                );
            }
        }
    }

    #[test]
    fn validation_work_and_error_count_are_bounded() {
        let tool = serde_json::json!({"name":"bounded", "inputSchema":{
            "properties":{"items":{"type":"array","items":{"type":"string"}}}
        }});
        assert_eq!(
            validate(&tool, &serde_json::json!({"items":vec![Value::Null;10001]})),
            Err("argument validation list exceeds 10000 items".to_owned())
        );
        assert_eq!(
            validate(&tool, &serde_json::json!({"items":vec![Value::Null;4097]})),
            Err("argument validation exceeds 4096 errors".to_owned())
        );
    }

    #[test]
    fn unknown_arguments_refuse_all_110_signatures_before_dispatch() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-unknown-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 110);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{}",
                case["name"]
            );
        }
    }

    #[test]
    fn forbidden_nested_model_fields_match_reference() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-nested-unknown-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 6);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{}",
                case["case"]
            );
        }
    }

    #[test]
    fn invalid_discriminator_tags_match_reference_in_all_three_families() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-invalid-tag-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 18);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{}",
                case["case"]
            );
        }
    }
    #[test]
    fn numeric_text_grammar_matches_all_reference_numeric_signatures() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-numeric-text-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 2250);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                validate(tool, &case["arguments"]).unwrap_err(),
                case["response"]["content"][0]["text"].as_str().unwrap(),
                "{} {}",
                case["name"],
                case["case"]
            );
        }
    }
}
