//! Materialize frozen defaults and validated scalar/model values before typed kernels.
use serde_json::{Map, Value, json};

pub fn normalize(tool: &Value, input: &Value) -> Value {
    value(&tool["inputSchema"], input)
}

fn value(schema: &Value, input: &Value) -> Value {
    if input.is_null() || schema.get("const").is_some() || schema.get("enum").is_some() {
        return input.clone();
    }
    if let Some(choices) = schema["anyOf"].as_array() {
        let nonnull: Vec<_> = choices.iter().filter(|s| s["type"] != "null").collect();
        return if nonnull.len() == 1 {
            value(nonnull[0], input)
        } else {
            input.clone()
        };
    }
    if let Some(choices) = schema["oneOf"].as_array() {
        if let Some(choice) = choices.iter().find(|choice| {
            choice["properties"].as_object().is_some_and(|fields| {
                fields.iter().any(|(key, field)| {
                    field
                        .get("const")
                        .is_some_and(|tag| Some(tag) == input.get(key))
                })
            })
        }) {
            return value(choice, input);
        }
        return input.clone();
    }
    match schema["type"].as_str() {
        Some("boolean") => crate::arguments::boolean(&json!({"v": input}), "v", false)
            .map(Value::Bool)
            .unwrap_or_else(|_| input.clone()),
        Some("number") => input
            .as_f64()
            .or_else(|| input.as_bool().map(|v| if v { 1.0 } else { 0.0 }))
            .or_else(|| input.as_str().and_then(crate::argument_numbers::float_text))
            .filter(|n| n.is_finite())
            .map(|n| json!(n))
            .unwrap_or_else(|| input.clone()),
        Some("integer") => {
            if let Some(text) = input.as_str() {
                crate::argument_numbers::integer_value(text).unwrap_or_else(|| input.clone())
            } else if let Some(boolean) = input.as_bool() {
                json!(if boolean { 1 } else { 0 })
            } else if input.is_number() && !input.to_string().contains(['.', 'e', 'E']) {
                input.clone()
            } else if let Some(number) =
                input.as_f64().filter(|n| n.is_finite() && n.fract() == 0.0)
            {
                serde_json::from_str(&format!("{number:.0}")).unwrap_or_else(|_| input.clone())
            } else {
                input.clone()
            }
        }
        Some("array") => input
            .as_array()
            .map(|items| {
                Value::Array(
                    items
                        .iter()
                        .enumerate()
                        .map(|(index, item)| {
                            value(
                                schema["prefixItems"].get(index).unwrap_or(&schema["items"]),
                                item,
                            )
                        })
                        .collect(),
                )
            })
            .unwrap_or_else(|| input.clone()),
        Some("object") => {
            let Some(object) = input.as_object() else {
                return input.clone();
            };
            if let Some(properties) = schema["properties"].as_object() {
                let mut output = Map::new();
                for (key, field) in properties {
                    if let Some(input) = object.get(key).or_else(|| field.get("default")) {
                        output.insert(key.clone(), value(field, input));
                    }
                }
                // BaseModel's default extra policy is ignore. Explicit false has already
                // refused extras during validation; free typed mappings have no properties.
                if schema["additionalProperties"] == true {
                    for (key, input) in object {
                        if !properties.contains_key(key) {
                            output.insert(key.clone(), input.clone());
                        }
                    }
                }
                Value::Object(output)
            } else if schema["additionalProperties"].is_object() {
                Value::Object(
                    object
                        .iter()
                        .map(|(key, input)| {
                            (key.clone(), value(&schema["additionalProperties"], input))
                        })
                        .collect(),
                )
            } else {
                input.clone()
            }
        }
        _ => input.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_integers_defaults_tuple_items_and_ignored_model_extras() {
        let huge: Value =
            serde_json::from_str("10000000000000000000000000000000000000001").unwrap();
        let integer = json!({"inputSchema":{"type":"integer"}});
        assert_eq!(normalize(&integer, &huge), huge);
        assert_eq!(
            normalize(
                &integer,
                &json!("10000000000000000000000000000000000000001.0")
            ),
            huge
        );
        let tool = json!({"inputSchema":{"type":"object","properties":{
            "seed":{"type":"integer"}, "enabled":{"type":"boolean","default":false},
            "model":{"type":"object","properties":{"x":{"type":"number"},
                "pair":{"type":"array","prefixItems":[{"type":"number"},{"type":"integer"}]}}}
        }}});
        assert_eq!(
            normalize(
                &tool,
                &json!({"seed":"+009007199254740993.00",
            "model":{"x":"1_0.0", "pair":[true,"-001.0"],"ignored":7}})
            ),
            json!({"seed":9007199254740993u64,"enabled":false,"model":{"x":10.0,"pair":[1.0,-1]}})
        );
    }
    fn canonical_float_spelling(value: Value) -> Value {
        match value {
            Value::Number(n) if n.to_string().contains(['.', 'e', 'E']) => {
                json!(n.as_f64().unwrap())
            }
            Value::Array(items) => {
                Value::Array(items.into_iter().map(canonical_float_spelling).collect())
            }
            Value::Object(fields) => Value::Object(
                fields
                    .into_iter()
                    .map(|(k, v)| (k, canonical_float_spelling(v)))
                    .collect(),
            ),
            other => other,
        }
    }
    #[test]
    fn all_signature_defaults_and_reference_coerced_model_values_match() {
        let discovery: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-true_raw-true_full_full.json"
        ))
        .unwrap();
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/argument-normalization-values.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 292);
        for case in cases.as_array().unwrap() {
            let tool = discovery["tools/list"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tool| tool["name"] == case["name"])
                .unwrap();
            assert_eq!(
                normalize(tool, &case["arguments"]),
                canonical_float_spelling(case["expected"].clone()),
                "{} {}",
                case["name"],
                case["arguments"]
            );
        }
    }
}
