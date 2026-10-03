//! Bounded row-major grid planning and the reference's explicit DOM-only size heuristic.
use crate::{arguments, document::elements};
use libxml::tree::Node;
use serde_json::Value;
pub struct Plan {
    pub rows: usize,
    pub cols: usize,
    pub cell: f64,
    pub gap: f64,
    pub padding: f64,
    pub fit: bool,
}
fn number(args: &Value, key: &str, default: Option<f64>) -> Result<f64, String> {
    args.get(key)
        .map(|v| arguments::number(v).ok_or_else(|| format!("{key} must be a number")))
        .unwrap_or_else(|| default.ok_or_else(|| format!("{key} must be a number")))
}
impl Plan {
    pub fn build(args: &Value, assets: usize) -> Result<Self, String> {
        let rows = number(args, "rows", None)?;
        let cols = number(args, "cols", None)?;
        if !rows.is_finite() || !cols.is_finite() || rows.fract() != 0. || cols.fract() != 0. {
            return Err("rows and cols must be integers".into());
        }
        if rows < 1. || cols < 1. {
            return Err("compose_grid rows and cols must each be >= 1".into());
        }
        if rows * cols > 1024. {
            return Err(format!(
                "compose_grid too large: {rows:.0}x{cols:.0} exceeds 1024 cells"
            ));
        }
        let cell = number(args, "cell", None)?;
        let gap = number(args, "gap", Some(0.))?;
        let padding = number(args, "padding", Some(0.))?;
        if !cell.is_finite() || cell <= 0. {
            return Err("compose_grid cell must be a finite number greater than 0".into());
        }
        for (key, v) in [("gap", gap), ("padding", padding)] {
            if !v.is_finite() {
                return Err(format!("{key} must be finite"));
            }
        }
        if gap < 0. || padding < 0. {
            return Err("compose_grid gap and padding must be >= 0".into());
        }
        if assets == 0 {
            return Err("compose_grid requires at least one asset".into());
        }
        if assets > (rows * cols) as usize {
            return Err(format!(
                "compose_grid has {assets} assets but only {} cells ({rows:.0}x{cols:.0})",
                (rows * cols) as usize
            ));
        }
        if cell - 2. * padding <= 0. {
            return Err(
                "compose_grid padding leaves no room in the cell (2*padding >= cell)".into(),
            );
        }
        Ok(Self {
            rows: rows as usize,
            cols: cols as usize,
            cell,
            gap,
            padding,
            fit: arguments::boolean(args, "scale_to_fit", true)?,
        })
    }
    pub fn origin(&self, index: usize) -> [f64; 2] {
        [
            (index % self.cols) as f64 * (self.cell + self.gap) + self.padding,
            (index / self.cols) as f64 * (self.cell + self.gap) + self.padding,
        ]
    }
    pub fn canvas(&self) -> [f64; 2] {
        [
            self.cols as f64 * self.cell + (self.cols - 1) as f64 * self.gap + 2. * self.padding,
            self.rows as f64 * self.cell + (self.rows - 1) as f64 * self.gap + 2. * self.padding,
        ]
    }
    pub fn factor(&self, node: Node) -> f64 {
        if !self.fit {
            return 1.;
        }
        longest(node)
            .filter(|n| *n > 0.)
            .map(|n| ((self.cell - 2. * self.padding) / n).min(1.))
            .unwrap_or(1.)
    }
}
fn num(node: &Node, key: &str) -> Option<f64> {
    let s = node.get_property_no_ns(key).unwrap_or_default();
    let s = s.trim();
    if s.is_empty() {
        Some(0.)
    } else {
        s.parse().ok()
    }
}
pub(crate) fn longest(root: Node) -> Option<f64> {
    let mut bounds: Option<[f64; 4]> = None;
    for n in elements(root) {
        let box_ = (|| -> Option<[f64; 4]> {
            Some(match n.get_name().as_str() {
                "rect" | "image" | "use" => [
                    num(&n, "x")?,
                    num(&n, "y")?,
                    num(&n, "width")?,
                    num(&n, "height")?,
                ],
                "circle" => {
                    let r = num(&n, "r")?;
                    [num(&n, "cx")? - r, num(&n, "cy")? - r, 2. * r, 2. * r]
                }
                "ellipse" => {
                    let rx = num(&n, "rx")?;
                    let ry = num(&n, "ry")?;
                    [num(&n, "cx")? - rx, num(&n, "cy")? - ry, 2. * rx, 2. * ry]
                }
                "line" => {
                    let x1 = num(&n, "x1")?;
                    let x2 = num(&n, "x2")?;
                    let y1 = num(&n, "y1")?;
                    let y2 = num(&n, "y2")?;
                    [x1.min(x2), y1.min(y2), (x2 - x1).abs(), (y2 - y1).abs()]
                }
                _ => return None,
            })
        })();
        if let Some([x, y, w, h]) = box_
            && [x, y, w, h].iter().all(|v| v.is_finite())
            && w >= 0.
            && h >= 0.
        {
            let b = [x, y, x + w, y + h];
            bounds = Some(
                bounds
                    .map(|a| {
                        [
                            a[0].min(b[0]),
                            a[1].min(b[1]),
                            a[2].max(b[2]),
                            a[3].max(b[3]),
                        ]
                    })
                    .unwrap_or(b),
            );
        }
    }
    bounds
        .filter(|b| b.iter().all(|n| n.is_finite()))
        .map(|b| (b[2] - b[0]).max(b[3] - b[1]))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn reference_geometry_and_plan_cases_match_without_normalization() {
        let cases: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/grid-plan-cases.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let document =
                crate::xml::parse(case["svg"].as_str().unwrap().as_bytes(), 4096).unwrap();
            let root = document.get_root_element().unwrap();
            let plan =
                Plan::build(&case["args"], case["assets"].as_u64().unwrap() as usize).unwrap();
            assert_eq!(json!(longest(root.clone())), case["longest"]);
            assert_eq!(json!(plan.factor(root)), case["factor"]);
            assert_eq!(json!(plan.origin(3)), case["origin"]);
            assert_eq!(json!(plan.canvas()), case["canvas"]);
        }
    }
    #[test]
    fn limits_origins_canvas_and_downscale_are_explicit() {
        let p = Plan::build(&json!({"rows":2,"cols":3,"cell":64,"gap":4,"padding":8}), 4).unwrap();
        assert_eq!(p.origin(3), [8., 76.]);
        assert_eq!(p.canvas(), [216., 148.]);
        let d = crate::xml::parse(
            b"<svg><rect width='96' height='24' transform='scale(10)'/></svg>",
            4096,
        )
        .unwrap();
        assert_eq!(p.factor(d.get_root_element().unwrap()), 0.5);
        assert!(Plan::build(&json!({"rows":1025,"cols":1,"cell":1}), 1).is_err());
        assert!(Plan::build(&json!({"rows":1,"cols":1,"cell":4,"padding":2}), 1).is_err());
        assert!(Plan::build(&json!({"rows":1,"cols":1,"cell":4}), 2).is_err());
        assert!(Plan::build(&json!({"rows":32,"cols":32,"cell":1}), 1024).is_ok());
    }
}
