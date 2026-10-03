//! Defensive live wire result modeling, shared by socket and native IPC backends.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const MAX_ITEMS: usize = 10_000;
pub fn truth(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}
pub fn float_repr(n: f64) -> String {
    if n != 0.0 && (n.abs() < 1e-4 || n.abs() >= 1e16) {
        let text = format!("{n:e}");
        let (mantissa, exp) = text.split_once('e').unwrap();
        let exp = exp.parse::<i32>().unwrap();
        format!("{mantissa}e{exp:+03}")
    } else {
        format!("{n:?}")
    }
}
fn repr(value: &Value) -> String {
    match value {
        Value::String(s) => crate::style::python_repr(s),
        Value::Array(a) => format!("[{}]", a.iter().map(repr).collect::<Vec<_>>().join(", ")),
        Value::Object(o) => format!(
            "{{{}}}",
            o.iter()
                .map(|(k, v)| format!("{}: {}", crate::style::python_repr(k), repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => string(value),
    }
}
pub fn string(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            let raw = n.to_string();
            if raw.contains(['.', 'e', 'E']) {
                n.as_f64().map(float_repr).unwrap_or(raw)
            } else {
                raw
            }
        }
        Value::Array(_) | Value::Object(_) => repr(value),
    }
}
fn opt_str(v: &Value) -> Value {
    v.as_str()
        .filter(|s| !s.is_empty())
        .map(|s| json!(s))
        .unwrap_or(Value::Null)
}
fn number(v: &Value) -> Option<f64> {
    if v.is_number() {
        v.as_f64().filter(|n| n.is_finite())
    } else {
        None
    }
}
fn float(v: &Value) -> Value {
    number(v).map(|n| json!(n)).unwrap_or(Value::Null)
}
pub fn document(result: &Value) -> Value {
    let count = if let Some(b) = result["object_count"].as_bool() {
        json!(u8::from(b))
    } else if result["object_count"].is_number()
        && !result["object_count"].to_string().contains(['.', 'e', 'E'])
    {
        result["object_count"].clone()
    } else {
        Value::Null
    };
    json!({"window_id":null,"document_id":null,"name":opt_str(&result["name"]),"path":opt_str(&result["path"]),"object_count":count})
}
pub fn ids(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|a| a.iter().map(string).collect())
        .unwrap_or_default()
}
pub fn selection(result: &Value) -> Value {
    let ids = ids(&result["object_ids"]);
    json!({"count":ids.len(),"object_ids":ids})
}
fn objects(raw: &Value, bound: usize) -> Vec<Value> {
    raw.as_array().map(|list|list.iter().take(bound).filter(|v|v.is_object()).map(|v| {
        json!({"id":opt_str(&v["id"]),"tag":v.get("tag").map(string).unwrap_or_default(),"label":opt_str(&v["label"]),"has_style":truth(&v["has_style"]),"paint":{"fill":null,"stroke":null,"stroke_width":null,"stroke_only":false},"is_layer":false,"is_leaf":true,"bbox":null})
    }).collect()).unwrap_or_default()
}
pub fn inspection(result: &Value) -> Value {
    let objects = objects(&result["objects"], usize::MAX);
    json!({"count":objects.len(),"objects":objects})
}
pub fn mutation(result: &Value) -> Value {
    let ids = ids(&result["affected_ids"]);
    json!({"count":ids.len(),"affected_ids":ids,"detail":result["detail"].as_str().unwrap_or(""),"undo_friendly":truth(&result["undo_friendly"])})
}
pub fn viewport_result(result: &Value, mode: &str) -> Value {
    json!({"mode":result["mode"].as_str().unwrap_or(mode),"applied":result.get("applied").map(truth).unwrap_or(true),"detail":result["detail"].as_str().unwrap_or("")})
}
fn bbox(v: &Value) -> Value {
    let nums = if let Some(a) = v.as_array().filter(|a| a.len() == 4) {
        a.iter().map(number).collect::<Option<Vec<_>>>()
    } else if v.is_object() {
        ["x", "y", "width", "height"]
            .iter()
            .map(|k| number(&v[k]))
            .collect::<Option<Vec<_>>>()
    } else {
        None
    };
    nums.map(|n| json!({"x":n[0],"y":n[1],"width":n[2],"height":n[3]}))
        .unwrap_or(Value::Null)
}
pub fn scene(result: &Value, active_document: &Value) -> Value {
    let selection: Vec<_> = result["selection"]
        .as_array()
        .map(|a| {
            a.iter()
                .take(MAX_ITEMS)
                .filter_map(|v| {
                    if !v.is_object() {
                        return None;
                    }
                    let id = opt_str(&v["id"]);
                    if id.is_null() {
                        return None;
                    }
                    Some(json!({"id":id,"bbox":bbox(&v["bbox"])}))
                })
                .collect()
        })
        .unwrap_or_default();
    let view = &result["viewport"];
    let center = view["center"]
        .as_array()
        .filter(|a| a.len() == 2)
        .and_then(|a| Some(json!([number(&a[0])?, number(&a[1])?])))
        .unwrap_or(Value::Null);
    let canvas = &result["canvas"];
    let objects = objects(&result["visible_objects"], MAX_ITEMS);
    json!({"active_document":active_document,"selection_count":selection.len(),"selection":selection,"viewport":{"zoom":float(&view["zoom"]),"center":center,"visible_region":bbox(&view["visible_region"])},"canvas":{"width":float(&canvas["width"]),"height":float(&canvas["height"]),"units":opt_str(&canvas["units"]),"viewbox":null},"object_count":objects.len(),"visible_objects":objects,"tree":null,"notes":[]})
}
fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))[..16].into()
}
pub fn token(result: &Value) -> (Value, Vec<String>) {
    let revision = if let Some(s) = result["revision"].as_str() {
        s.to_owned()
    } else {
        number(&result["revision"])
            .map(float_repr)
            .unwrap_or_default()
    };
    let ids: Vec<_> = result["selection"]
        .as_array()
        .map(|a| {
            a.iter()
                .take(MAX_ITEMS)
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let view = &result["viewport"];
    let viewport = if view.is_object() {
        let center = view["center"].as_array().filter(|a| a.len() == 2);
        [
            number(&view["zoom"]),
            center.and_then(|a| number(&a[0])),
            center.and_then(|a| number(&a[1])),
        ]
        .into_iter()
        .map(|n| {
            n.map(|n| float_repr(format!("{n:.4}").parse::<f64>().unwrap()))
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("|")
    } else {
        String::new()
    };
    (
        json!({"revision":digest(&revision),"selection":digest(&ids.join("\u{1f}")),"viewport":digest(&viewport)}),
        ids,
    )
}
pub fn change(previous: Option<&Value>, current: &Value, ids: &[String]) -> Value {
    let changed = |key: &str| previous.is_some_and(|p| p[key] != current[key]);
    let selection = changed("selection");
    let document = changed("revision");
    let viewport = changed("viewport");
    json!({"changed":selection||document||viewport,"selection_changed":selection,"document_changed":document,"viewport_changed":viewport,"timed_out":false,"token":current,"selection_ids":ids})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_and_token_item_bounds_and_authoritative_identity() {
        let entries = vec![json!({"id":"r","tag":"rect"}); MAX_ITEMS + 1];
        let result = json!({"selection":entries,"visible_objects":entries,"active_document":{"path":"spoof"}});
        let scene = scene(&result, &json!({"path":"authoritative"}));
        assert_eq!(scene["selection_count"], MAX_ITEMS);
        assert_eq!(scene["object_count"], MAX_ITEMS);
        assert_eq!(scene["active_document"]["path"], "authoritative");
        let all = vec![json!("r"); MAX_ITEMS + 1];
        let (current, ids) = token(&json!({"selection":all}));
        assert_eq!(ids.len(), MAX_ITEMS);
        let capped = vec![json!("r"); MAX_ITEMS];
        assert_eq!(current, token(&json!({"selection":capped})).0);
        assert_eq!(token(&json!({"revision":true})).0, token(&json!({})).0);
    }
    #[test]
    fn compiled_python_defensive_models_and_change_digests_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/socket-model-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let result = &case["input"];
            let actual = match case["kind"].as_str().unwrap() {
                "document" => document(result),
                "selection" => selection(result),
                "inspection" => inspection(result),
                "mutation" => mutation(result),
                "viewport" => viewport_result(result, "zoom"),
                "scene" => scene(result, &case["document"]),
                "token" => {
                    let (token, ids) = token(result);
                    json!({"token":token,"ids":ids})
                }
                "string" => json!(string(result)),
                _ => panic!("unknown case"),
            };
            assert_eq!(actual, case["expected"], "case {}", case["label"]);
        }
        let first = json!({"revision":"a","selection":"b","viewport":"c"});
        assert_eq!(change(None, &first, &[])["changed"], false);
        assert_eq!(change(Some(&first), &first, &[])["changed"], false);
        for key in ["revision", "selection", "viewport"] {
            let mut next = first.clone();
            next[key] = json!("changed");
            assert_eq!(change(Some(&first), &next, &[])["changed"], true);
        }
    }
}
