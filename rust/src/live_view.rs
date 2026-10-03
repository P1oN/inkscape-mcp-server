//! Shared typed view-only preflight. Fixed modes; no raw actions and no document mutation.
use crate::{arguments, live_socket::Viewport};
use serde_json::Value;
fn bounded(value: f64, limit: f64, label: &str) -> Result<f64, String> {
    if !value.is_finite() {
        Err(format!("{label} must be finite"))
    } else if value.abs() > limit {
        Err(format!("{label} is out of bounds"))
    } else {
        Ok(value)
    }
}
pub fn viewport(args: &Value) -> Result<Viewport, String> {
    let mode = arguments::string(args, "mode")?.ok_or("mode is required")?;
    match mode {
        "fit_page" => Ok(Viewport::FitPage),
        "fit_selection" => Ok(Viewport::FitSelection),
        "zoom" => {
            let zoom = arguments::optional_number(args, "zoom")?
                .ok_or("zoom mode requires a zoom factor")?;
            let zoom = bounded(zoom, 10000., "zoom")?;
            if !(1e-4..=10000.).contains(&zoom) {
                return Err("zoom factor is out of bounds".into());
            }
            let x = arguments::optional_number(args, "center_x")?;
            let y = arguments::optional_number(args, "center_y")?;
            let center = match (x, y) {
                (None, None) => None,
                (Some(x), Some(y)) => {
                    Some([bounded(x, 1e7, "center_x")?, bounded(y, 1e7, "center_y")?])
                }
                _ => return Err("center requires both center_x and center_y".into()),
            };
            Ok(Viewport::Zoom { zoom, center })
        }
        "pan" => {
            let x = arguments::optional_number(args, "dx")?;
            let y = arguments::optional_number(args, "dy")?;
            let (Some(x), Some(y)) = (x, y) else {
                return Err("pan mode requires both dx and dy".into());
            };
            Ok(Viewport::Pan {
                dx: bounded(x, 1e7, "dx")?,
                dy: bounded(y, 1e7, "dy")?,
            })
        }
        _ => Err(format!(
            "unknown viewport mode: {}",
            crate::style::python_repr(mode)
        )),
    }
}
#[derive(Clone, Copy)]
pub struct Frame {
    pub region: Option<[f64; 4]>,
    pub scale: Option<f64>,
}
pub fn frame(args: &Value) -> Result<Frame, String> {
    let parts = ["region_x", "region_y", "region_width", "region_height"]
        .map(|key| arguments::optional_number(args, key))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let count = parts.iter().filter(|n| n.is_some()).count();
    if count != 0 && count != 4 {
        return Err(
            "region requires all of region_x, region_y, region_width, region_height".into(),
        );
    }
    let region = if count == 4 {
        let mut region = [0.; 4];
        for (index, label) in ["region x", "region y", "region width", "region height"]
            .iter()
            .enumerate()
        {
            region[index] = bounded(parts[index].unwrap(), 1e7, label)?;
        }
        if region[2] <= 0. || region[3] <= 0. {
            return Err("region width/height must be positive".into());
        }
        Some(region)
    } else {
        None
    };
    let fast = arguments::boolean(args, "fast", false)?;
    let scale = arguments::optional_number(args, "scale")?.or(if fast { Some(0.5) } else { None });
    let scale = scale
        .map(|n| -> Result<f64, String> {
            let n = bounded(n, 64., "scale")?;
            if !(1e-3..=64.).contains(&n) {
                return Err("scale is out of bounds".into());
            }
            Ok(n)
        })
        .transpose()?;
    Ok(Frame { region, scale })
}
