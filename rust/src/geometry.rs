//! Root user-unit mapping shared by accurate live query and isolated preview.
use crate::{arguments, decimal, inspect, style};
use libxml::tree::Node;
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;
static LENGTH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)[\s\x1c-\x1f]*([a-z]*)$").unwrap()
});
static SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s,\x1c-\x1f]+").unwrap());
static ALIGN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^x(Min|Mid|Max)Y(Min|Mid|Max)$").unwrap());
fn trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
}
fn length(raw: &str) -> Result<f64, String> {
    let c = LENGTH
        .captures(trim(raw))
        .ok_or("accurate geometry requires absolute root width and height")?;
    let unit = match &c[2] {
        "" | "px" => 1.,
        "mm" => 96. / 25.4,
        "cm" => 96. / 2.54,
        "in" => 96.,
        "pt" => 96. / 72.,
        "pc" => 16.,
        "q" => 96. / 101.6,
        _ => return Err("accurate geometry requires absolute root width and height".into()),
    };
    let value =
        decimal::float(&c[1]).ok_or("invalid document dimensions for accurate geometry")? * unit;
    if !value.is_finite() || value <= 0. {
        return Err("invalid document dimensions for accurate geometry".into());
    }
    Ok(value)
}
pub fn root_mapping(root: &Node) -> Result<(f64, f64, f64, f64), String> {
    let style = inspect::parse_declarations(&root.get_property_no_ns("style").unwrap_or_default());
    if style.contains_key("width") || style.contains_key("height") {
        return Err("root CSS sizing requires snapshot preparation".into());
    }
    let Some(viewbox) = root.get_property_no_ns("viewBox").filter(|s| !s.is_empty()) else {
        return Ok((1., 1., 0., 0.));
    };
    let values = SPLIT
        .split(trim(&viewbox))
        .map(decimal::float)
        .collect::<Option<Vec<_>>>()
        .ok_or("invalid document viewBox")?;
    if values.len() != 4
        || values.iter().any(|n| !n.is_finite())
        || values[2] <= 0.
        || values[3] <= 0.
    {
        return Err("invalid document viewBox".into());
    }
    let [x, y, w, h] = [values[0], values[1], values[2], values[3]];
    let (width, height) = match (
        root.get_property_no_ns("width"),
        root.get_property_no_ns("height"),
    ) {
        (None, None) => (w, h),
        (a, b) => (
            length(&a.unwrap_or_default())?,
            length(&b.unwrap_or_default())?,
        ),
    };
    let (sx, sy) = (width / w, height / h);
    if !sx.is_finite() || !sy.is_finite() || sx <= 0. || sy <= 0. {
        return Err("invalid root coordinate mapping".into());
    }
    let aspect = root
        .get_property_no_ns("preserveAspectRatio")
        .unwrap_or("xMidYMid meet".into());
    let mut parts = aspect
        .split(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if parts.first() == Some(&"defer") {
        parts.remove(0);
    }
    let result = if parts == ["none"] {
        [sx, sy, -x * sx, -y * sy]
    } else {
        let alignment = parts
            .first()
            .and_then(|s| ALIGN.captures(s))
            .ok_or("unsupported preserveAspectRatio")?;
        if parts.len() > 2 || (parts.len() == 2 && !matches!(parts[1], "meet" | "slice")) {
            return Err("unsupported preserveAspectRatio".into());
        }
        let scale = if parts.last() == Some(&"slice") {
            sx.max(sy)
        } else {
            sx.min(sy)
        };
        let factor = |s: &str| match s {
            "Min" => 0.,
            "Mid" => 0.5,
            _ => 1.,
        };
        [
            scale,
            scale,
            (width - w * scale) * factor(&alignment[1]) - x * scale,
            (height - h * scale) * factor(&alignment[2]) - y * scale,
        ]
    };
    if result.iter().any(|n| !n.is_finite()) || result[0] <= 0. {
        return Err("invalid root coordinate mapping".into());
    }
    Ok((result[0], result[1], result[2], result[3]))
}
pub struct Region {
    pub bounds: [f64; 4],
    pub width: i64,
    pub height: i64,
    pub background: String,
}
impl Region {
    pub fn build(
        root: &Node,
        region: &Value,
        width: i64,
        background: &str,
    ) -> Result<Self, String> {
        let object = region.as_object().ok_or("region must be an object")?;
        if object
            .keys()
            .any(|key| !["x", "y", "width", "height"].contains(&key.as_str()))
        {
            return Err("region contains unknown fields".into());
        }
        let mut values = [0.0; 4];
        for (index, key) in ["x", "y", "width", "height"].iter().enumerate() {
            values[index] = object
                .get(*key)
                .and_then(arguments::number)
                .filter(|n| n.is_finite())
                .ok_or_else(|| format!("region.{key} must be finite"))?;
        }
        let [x, y, w, h] = values;
        if w <= 0.0 || h <= 0.0 {
            return Err("region width and height must be positive".into());
        }
        if root
            .get_property_no_ns("transform")
            .is_some_and(|s| !s.is_empty())
        {
            return Err(
                "root transforms require preparation for document-space region export".into(),
            );
        }
        let (sx, sy, tx, ty) = root_mapping(root)?;
        let bounds = [
            x * sx + tx,
            y * sy + ty,
            (x + w) * sx + tx,
            (y + h) * sy + ty,
        ];
        let projected = width as f64 * h * sy / (w * sx);
        if !bounds.iter().chain([&projected]).all(|v| v.is_finite()) {
            return Err("region bounds or dimensions are non-finite".into());
        }
        if width <= 0 {
            return Err("width_px must be a positive integer".into());
        }
        let height = projected.ceil().max(1.0) as i64;
        if width > i64::from(crate::render::cap()) || height > i64::from(crate::render::cap()) {
            return Err("export exceeds the configured size or dimension limit".into());
        }
        Ok(Self {
            bounds,
            width,
            height,
            background: style::color(background)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mapping(attrs: &str) -> Result<(f64, f64, f64, f64), String> {
        let d = crate::xml::parse(format!("<svg {attrs}/>").as_bytes(), 4096).unwrap();
        root_mapping(&d.get_root_element().unwrap())
    }
    #[test]
    fn frozen_python_root_mapping_cases_match_all_fields() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../migration/contracts/live-geometry-cases.json"
        ))
        .unwrap();
        for (i, c) in cases["cases"].as_array().unwrap().iter().enumerate() {
            let doc =
                crate::xml::parse(c["svg"].as_str().unwrap().as_bytes(), 1024 * 1024).unwrap();
            let result = root_mapping(&doc.get_root_element().unwrap());
            if let Some(error) = c["error"].as_str() {
                assert_eq!(result, Err(error.into()), "case {i}");
            } else {
                assert_eq!(serde_json::json!(result.unwrap()), c["mapping"], "case {i}");
            }
        }
    }
    #[test]
    fn mapping_supports_units_letterboxing_and_nonuniform_scaling() {
        assert_eq!(
            mapping("viewBox='10 20 100 50' width='200' height='200'").unwrap(),
            (2.0, 2.0, -20.0, 10.0)
        );
        assert_eq!(
            mapping("viewBox='10 20 100 50' width='200' height='200' preserveAspectRatio='none'")
                .unwrap(),
            (2.0, 4.0, -20.0, -80.0)
        );
        assert_eq!(mapping("viewBox='10 20 100 50' width='200' height='200' preserveAspectRatio='xMaxYMin slice'").unwrap(),(4.0,4.0,-240.0,-80.0));
        assert_eq!(
            mapping("width='100%' height='100%'").unwrap(),
            (1.0, 1.0, 0.0, 0.0)
        );
        assert_eq!(length("1in").unwrap(), 96.0);
        assert_eq!(length("2.54cm").unwrap(), 96.0);
        assert!(length("100%").is_err());
        assert!(mapping("viewBox='0 0 10 20' width='100%' height='20'").is_err());
        assert!(mapping("style='width:10px'").is_err());
        assert!(mapping("viewBox='0 0 10 20' preserveAspectRatio='xMinYMin nonsense'").is_err());
    }

    #[test]
    fn regions_bound_height_overflow_and_background_before_rendering() {
        let d = crate::xml::parse(b"<svg viewBox='0 0 100 100'/>", 4096).unwrap();
        let root = d.get_root_element().unwrap();
        let region = serde_json::json!({"x":0,"y":0,"width":20,"height":10});
        let prepared = Region::build(&root, &region, 101, "White").unwrap();
        assert_eq!(prepared.height, 51);
        assert_eq!(prepared.background, "white");
        assert!(Region::build(&root, &region, 0, "white").is_err());
        assert!(Region::build(&root, &region, 101, "white; quit").is_err());
        let huge = serde_json::json!({"x":0,"y":0,"width":1,"height":1e300});
        assert!(Region::build(&root, &huge, 100, "white").is_err());
        let overflow = serde_json::json!({"x":1e308,"y":0,"width":1e308,"height":10});
        assert!(Region::build(&root, &overflow, 100, "white").is_err());
        let extra = serde_json::json!({"x":0,"y":0,"width":20,"height":10,"script":"quit"});
        assert!(Region::build(&root, &extra, 100, "white").is_err());
    }
}
