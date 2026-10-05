//! Bounded, read-only path diagnostics shared with opt-in creation guards.
use libxml::tree::Node;
use serde_json::{Value, json};
use svgtypes::{PathParser, PathSegment as S};

#[derive(Default, Debug)]
pub struct PathReview {
    pub open_subpaths: usize,
    pub zero_segments: usize,
    pub near_segments: usize,
    pub coincident_curves: usize,
    pub segments: usize,
}
pub fn path(d: &str, tolerance: f64) -> Result<PathReview, String> {
    path_bounded(d, tolerance, 10000)
}
fn path_bounded(d: &str, tolerance: f64, segment_limit: usize) -> Result<PathReview, String> {
    if d.len() > 262144 {
        return Err("Path byte limit exceeded.".into());
    }
    let mut result = PathReview::default();
    let mut current = [0., 0.];
    let mut start = current;
    let mut active = false;
    let mut cubic = None;
    let mut quadratic = None;
    for item in PathParser::from(d) {
        result.segments += 1;
        if result.segments > segment_limit {
            return Err("Path segment limit exceeded.".into());
        }
        let segment = item.map_err(|_| "Invalid SVG path data.")?;
        if result.segments == 1 && !matches!(segment, S::MoveTo { .. }) {
            return Err("Path must begin with moveto.".into());
        }
        let point = |abs: bool, x: f64, y: f64| {
            if abs {
                [x, y]
            } else {
                [current[0] + x, current[1] + y]
            }
        };
        let reflect = |control: Option<[f64; 2]>| {
            control
                .map(|p| [2. * current[0] - p[0], 2. * current[1] - p[1]])
                .unwrap_or(current)
        };
        let mut controls = vec![];
        let mut next_cubic = None;
        let mut next_quadratic = None;
        let mut is_curve = false;
        let next = match segment {
            S::MoveTo { abs, x, y } => {
                if active {
                    result.open_subpaths += 1;
                }
                let p = point(abs, x, y);
                current = p;
                start = p;
                active = true;
                cubic = None;
                quadratic = None;
                if !p.iter().all(|n| n.is_finite()) {
                    return Err("Non-finite path coordinates.".into());
                }
                continue;
            }
            S::ClosePath { .. } => {
                // A redundant closing edge is normal: Z still changes stroke join semantics.
                current = start;
                active = false;
                cubic = None;
                quadratic = None;
                continue;
            }
            S::LineTo { abs, x, y } => point(abs, x, y),
            S::HorizontalLineTo { abs, x } => [if abs { x } else { current[0] + x }, current[1]],
            S::VerticalLineTo { abs, y } => [current[0], if abs { y } else { current[1] + y }],
            S::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                controls.extend([point(abs, x1, y1), point(abs, x2, y2)]);
                next_cubic = Some(point(abs, x2, y2));
                is_curve = true;
                point(abs, x, y)
            }
            S::SmoothCurveTo { abs, x2, y2, x, y } => {
                controls.extend([reflect(cubic), point(abs, x2, y2)]);
                next_cubic = Some(point(abs, x2, y2));
                is_curve = true;
                point(abs, x, y)
            }
            S::Quadratic { abs, x1, y1, x, y } => {
                controls.push(point(abs, x1, y1));
                next_quadratic = Some(point(abs, x1, y1));
                is_curve = true;
                point(abs, x, y)
            }
            S::SmoothQuadratic { abs, x, y } => {
                controls.push(reflect(quadratic));
                next_quadratic = Some(reflect(quadratic));
                is_curve = true;
                point(abs, x, y)
            }
            S::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                x,
                y,
                ..
            } => {
                if ![rx, ry, x_axis_rotation, x, y]
                    .iter()
                    .all(|n| n.is_finite())
                {
                    return Err("Non-finite path coordinates.".into());
                }
                // SVG omits an arc whose endpoints coincide; no loop is inferred.
                point(abs, x, y)
            }
        };
        if !next
            .iter()
            .chain(controls.iter().flatten())
            .all(|n| n.is_finite())
        {
            return Err("Non-finite path coordinates.".into());
        }
        let coincident = next == current;
        let zero = coincident && controls.iter().all(|p| *p == current);
        if zero {
            result.zero_segments += 1;
        } else if coincident && is_curve {
            result.coincident_curves += 1;
        } else if tolerance > 0. && (next[0] - current[0]).hypot(next[1] - current[1]) <= tolerance
        {
            result.near_segments += 1;
        }
        current = next;
        cubic = next_cubic;
        quadratic = next_quadratic;
        active = true;
    }
    if result.segments == 0 {
        return Err("Path d is empty.".into());
    }
    if active {
        result.open_subpaths += 1;
    }
    Ok(result)
}

pub fn analyze(nodes: &[Node], roles: &[(String, String)], tolerance: f64) -> Value {
    let mut findings = vec![];
    let mut total = 0;
    let mut bytes = 0usize;
    let mut segments = 0usize;
    let mut add = |node: &Node, code: &str, certainty: &str, reason: String| {
        total += 1;
        if findings.len() < 200 {
            findings.push(json!({"object_id":node.get_property_no_ns("id"),"code":code,"certainty":certainty,"reason":reason}));
        }
    };
    if nodes.len() > 20000 {
        return json!({"findings":[{"code":"geometry_limit","certainty":"unknown","reason":"Element limit exceeded."}],"truncated":true});
    }
    let mut ids = std::collections::HashMap::<String, usize>::new();
    for node in nodes {
        if let Some(id) = node.get_property_no_ns("id") {
            *ids.entry(id).or_default() += 1;
        }
    }
    for node in nodes {
        let closed = node.get_property_no_ns("id").is_some_and(|id| {
            roles
                .iter()
                .any(|(target, role)| target == &id && role == "closed_shape")
        });
        if closed
            && node
                .get_property_no_ns("id")
                .is_some_and(|id| ids.get(&id) != Some(&1))
        {
            continue;
        }
        if node
            .get_namespace()
            .is_some_and(|ns| ns.get_href() != "http://www.w3.org/2000/svg")
        {
            if closed || node.get_name() == "path" {
                add(
                    node,
                    "path_geometry_unknown",
                    "unknown",
                    "Foreign namespace geometry is unsupported.".into(),
                );
            }
            continue;
        }
        if node.get_name() != "path" {
            if closed
                && !["rect", "circle", "ellipse", "polygon"].contains(&node.get_name().as_str())
            {
                add(
                    node,
                    "closed_shape_unknown",
                    "unknown",
                    "Role requires a supported closed primitive or path.".into(),
                );
            }
            continue;
        }
        let d = node.get_property_no_ns("d").unwrap_or_default();
        bytes = bytes.saturating_add(d.len());
        let review = if bytes > 4194304 || segments >= 200000 {
            Err("Report path budget exceeded.".into())
        } else {
            path_bounded(&d, tolerance, (200000 - segments).min(10000))
        };
        match review {
            Err(reason) => add(node, "path_geometry_unknown", "unknown", reason),
            Ok(review) => {
                segments += review.segments;
                if closed && review.open_subpaths > 0 {
                    add(
                        node,
                        "open_shape",
                        "known",
                        format!("{} subpaths lack explicit Z closure.", review.open_subpaths),
                    );
                }
                if review.zero_segments > 0 {
                    add(
                        node,
                        "zero_length_segment",
                        "known",
                        format!(
                            "{} exact zero-length segments; may be intentional dots. No automatic removal.",
                            review.zero_segments
                        ),
                    );
                }
                if review.near_segments > 0 {
                    add(
                        node,
                        "near_coincident_nodes",
                        "known",
                        format!(
                            "{} endpoint pairs within {tolerance} local user units; review curves before repair.",
                            review.near_segments
                        ),
                    );
                }
                if review.coincident_curves > 0 {
                    add(
                        node,
                        "coincident_curve_endpoints",
                        "known",
                        format!(
                            "{} curves share endpoint coordinates but have distinct controls; possible intentional loops.",
                            review.coincident_curves
                        ),
                    );
                }
            }
        }
    }
    json!({"findings":findings,"truncated":total>200,"tolerance":tolerance,"scope":"Path-local coordinates; no cross-object overlap inference or automatic cleanup. Z closing edges are not redundant-node findings."})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closure_relative_curves_and_dots() {
        assert_eq!(path("M0 0 L1 0 L0 0", 0.).unwrap().open_subpaths, 1);
        assert_eq!(path("m1 1 l1 0 z m2 2 l1 0", 0.).unwrap().open_subpaths, 1);
        let p = path("M0 0 C1 0 1 1 0 0 Z", 0.).unwrap();
        assert_eq!(p.zero_segments, 0);
        assert_eq!(p.coincident_curves, 1);
        assert_eq!(path("M0 0 l0 0", 0.).unwrap().zero_segments, 1);
        assert_eq!(path("M0 0 Q0 0 0 0 T0 0", 0.).unwrap().zero_segments, 2);
        assert_eq!(
            path("M0 0 C1 1 2 2 3 3 S3 3 3 3", 0.)
                .unwrap()
                .zero_segments,
            0
        );
        assert_eq!(path("M0 0 L0.001 0", 0.01).unwrap().near_segments, 1);
        assert_eq!(path("M0 0 L1 0 L0 0 Z", 0.).unwrap().zero_segments, 0);
        assert!(path("M0 0 L", 0.).is_err());
        assert!(path("M1e999 0", 0.).is_err());
        assert!(path("M1e308 0 l1e308 0", 0.).is_err());
        assert!(path(&format!("M0 0 {}", "l1 0 ".repeat(10001)), 0.).is_err());
    }
}
