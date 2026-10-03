//! Bounded repeat planning with CPython-compatible integer-seeded MT19937.
use crate::arguments;
use serde_json::{Value, json};
const INVALID: &str = "invalid bounded placement/variation parameters";
struct Random {
    state: [u32; 624],
    index: usize,
}
impl Random {
    fn new(seed: &Value) -> Result<Self, String> {
        let raw = if let Some(s) = seed.as_str() {
            s.trim().to_owned()
        } else if let Some(b) = seed.as_bool() {
            u8::from(b).to_string()
        } else if seed.is_number() && !seed.to_string().contains(['.', 'e', 'E']) {
            seed.to_string()
        } else if let Some(n) = seed.as_f64().filter(|n| n.is_finite() && n.fract() == 0.) {
            format!("{n:.0}")
        } else {
            return Err(INVALID.into());
        };
        let digits = raw.strip_prefix(['+', '-']).unwrap_or(&raw);
        if digits.is_empty() || digits.len() > 4096 || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(INVALID.into());
        }
        let mut key = vec![0u32];
        for digit in digits.bytes() {
            let mut carry = (digit - b'0') as u64;
            for word in &mut key {
                let v = *word as u64 * 10 + carry;
                *word = v as u32;
                carry = v >> 32;
            }
            if carry != 0 {
                key.push(carry as u32);
            }
        }
        let mut r = Self {
            state: [0; 624],
            index: 624,
        };
        r.state[0] = 19650218;
        for i in 1..624 {
            r.state[i] = 1812433253u32
                .wrapping_mul(r.state[i - 1] ^ (r.state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        let (mut i, mut j) = (1, 0);
        for _ in 0..624.max(key.len()) {
            r.state[i] = (r.state[i]
                ^ (r.state[i - 1] ^ (r.state[i - 1] >> 30)).wrapping_mul(1664525))
            .wrapping_add(key[j])
            .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                r.state[0] = r.state[623];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..623 {
            r.state[i] = (r.state[i]
                ^ (r.state[i - 1] ^ (r.state[i - 1] >> 30)).wrapping_mul(1566083941))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                r.state[0] = r.state[623];
                i = 1;
            }
        }
        r.state[0] = 0x80000000;
        Ok(r)
    }
    fn word(&mut self) -> u32 {
        if self.index >= 624 {
            for i in 0..624 {
                let y = (self.state[i] & 0x80000000) | (self.state[(i + 1) % 624] & 0x7fffffff);
                self.state[i] = self.state[(i + 397) % 624]
                    ^ (y >> 1)
                    ^ if y & 1 != 0 { 0x9908b0df } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }
    fn unit(&mut self) -> f64 {
        let a = self.word() >> 5;
        let b = self.word() >> 6;
        (a as f64 * 67108864. + b as f64) / 9007199254740992.
    }
    fn uniform(&mut self, extent: f64) -> f64 {
        -extent + (extent - (-extent)) * self.unit()
    }
}
fn fields(v: &Value, allowed: &[&str]) -> Result<(), String> {
    let object = v.as_object().ok_or(INVALID)?;
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(INVALID.into());
    }
    Ok(())
}
fn number(v: &Value, key: &str) -> Result<f64, String> {
    arguments::number(&v[key])
        .filter(|n| n.is_finite())
        .ok_or_else(|| INVALID.into())
}
fn optional(v: &Value, key: &str) -> Result<Option<f64>, String> {
    if v[key].is_null() {
        Ok(None)
    } else {
        number(v, key).map(Some)
    }
}
fn integer(v: &Value, key: &str) -> Result<Option<usize>, String> {
    optional(v, key)?
        .map(|n| {
            if n.fract() == 0. && (1. ..=1024.).contains(&n) {
                Ok(n as usize)
            } else {
                Err(INVALID.into())
            }
        })
        .transpose()
}
fn positive(v: &Value, key: &str) -> Result<Option<f64>, String> {
    optional(v, key)?
        .map(|n| if n > 0. { Ok(n) } else { Err(INVALID.into()) })
        .transpose()
}
pub fn plan(args: &Value) -> Result<Value, String> {
    let p = &args["placement"];
    let orientation = arguments::string(args, "orientation")?.unwrap_or("fixed");
    if !["fixed", "tangent"].contains(&orientation) {
        return Err("orientation must be fixed or tangent".into());
    }
    let mut positions: Vec<(f64, f64, f64)> = vec![];
    match p["kind"].as_str() {
        Some("polyline") => {
            fields(p, &["kind", "points", "count", "spacing"])?;
            let points = p["points"]
                .as_array()
                .filter(|v| (2..=256).contains(&v.len()))
                .ok_or(INVALID)?;
            let mut coordinates = vec![];
            for point in points {
                fields(point, &["x", "y"])?;
                coordinates.push((number(point, "x")?, number(point, "y")?));
            }
            let count = integer(p, "count")?;
            let spacing = positive(p, "spacing")?;
            if count.is_some() == spacing.is_some() {
                return Err("polyline requires exactly one of count or spacing".into());
            }
            let mut segments = vec![];
            for pair in coordinates.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let d = (b.0 - a.0).hypot(b.1 - a.1);
                if !d.is_finite() {
                    return Err("non-finite polyline length".into());
                }
                if d > 0. {
                    segments.push((a, b, d));
                }
            }
            let length: f64 = segments.iter().map(|s| s.2).sum();
            if segments.is_empty() || !length.is_finite() {
                return Err("polyline must have a finite nonzero length".into());
            }
            let distances = if let Some(spacing) = spacing {
                let ratio = length / spacing;
                if !ratio.is_finite() || ratio >= 1024. {
                    return Err("placement exceeds 1024 instances".into());
                }
                (0..ratio.floor() as usize + 1)
                    .map(|i| i as f64 * spacing)
                    .collect::<Vec<_>>()
            } else {
                let count = count.unwrap();
                (0..count)
                    .map(|i| {
                        if count > 1 {
                            i as f64 * length / (count - 1) as f64
                        } else {
                            0.
                        }
                    })
                    .collect()
            };
            let (mut cursor, mut accumulated) = (0, 0.);
            for distance in distances {
                while cursor < segments.len() - 1 && distance > accumulated + segments[cursor].2 {
                    accumulated += segments[cursor].2;
                    cursor += 1;
                }
                let (a, b, length) = segments[cursor];
                let t = ((distance - accumulated) / length).min(1.);
                let angle = (b.1 - a.1).atan2(b.0 - a.0).to_degrees();
                positions.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, angle));
            }
        }
        Some("rectangle") => {
            fields(
                p,
                &[
                    "kind",
                    "x",
                    "y",
                    "width",
                    "height",
                    "count",
                    "columns",
                    "spacing_x",
                    "spacing_y",
                ],
            )?;
            let x = number(p, "x")?;
            let y = number(p, "y")?;
            let width = positive(p, "width")?.ok_or(INVALID)?;
            let height = positive(p, "height")?.ok_or(INVALID)?;
            let count = integer(p, "count")?;
            let columns = integer(p, "columns")?;
            let sx = positive(p, "spacing_x")?;
            let sy = positive(p, "spacing_y")?;
            if orientation == "tangent" {
                return Err("rectangle placement supports fixed orientation only".into());
            }
            let (count, columns, dx, dy) = if let Some(count) = count {
                if sx.is_some() || sy.is_some() {
                    return Err("rectangle requires count or spacing, not both".into());
                }
                let columns = columns.unwrap_or((count as f64).sqrt().ceil() as usize);
                if columns > count {
                    return Err("columns must not exceed count".into());
                }
                let rows = count.div_ceil(columns);
                (count, columns, width / columns as f64, height / rows as f64)
            } else {
                let (Some(dx), Some(dy), None) = (sx, sy, columns) else {
                    return Err(
                        "rectangle requires count or both spacings (without columns)".into(),
                    );
                };
                let (rx, ry) = (width / dx, height / dy);
                if !rx.is_finite() || !ry.is_finite() || rx > 1024. || ry > 1024. {
                    return Err("placement exceeds 1024 instances".into());
                }
                let (cols, rows) = (rx.floor() as usize, ry.floor() as usize);
                (cols * rows, cols, dx, dy)
            };
            if !(1..=1024).contains(&count) {
                return Err("placement requires 1..1024 instances".into());
            }
            for i in 0..count {
                positions.push((
                    x + ((i % columns) as f64 + 0.5) * dx,
                    y + ((i / columns) as f64 + 0.5) * dy,
                    0.,
                ));
            }
        }
        _ => return Err(INVALID.into()),
    }
    let default = json!({});
    let variation = args
        .get("variation")
        .filter(|v| !v.is_null())
        .unwrap_or(&default);
    fields(variation, &["seed", "offset", "scale", "rotation_degrees"])?;
    let offset = optional(variation, "offset")?.unwrap_or(0.);
    let scale = optional(variation, "scale")?.unwrap_or(0.);
    let rotation = optional(variation, "rotation_degrees")?.unwrap_or(0.);
    if !(0. ..=1000.).contains(&offset)
        || !(0. ..=0.5).contains(&scale)
        || !(0. ..=180.).contains(&rotation)
    {
        return Err(INVALID.into());
    }
    let mut rng = Random::new(variation.get("seed").unwrap_or(&json!(0)))?;
    let mut plan = vec![];
    for (x, y, angle) in positions {
        let x = x + rng.uniform(offset);
        let y = y + rng.uniform(offset);
        let rotation = (if orientation == "tangent" { angle } else { 0. }) + rng.uniform(rotation);
        let scale = 1. + rng.uniform(scale);
        if ![x, y, rotation, scale].iter().all(|v| v.is_finite()) {
            return Err("placement coordinates overflow".into());
        }
        plan.push(json!({"x":x,"y":y,"rotation_degrees":rotation,"scale":scale}));
    }
    Ok(json!(plan))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_seed_stream_matches_python_including_large_seeds() {
        for (seed, expected) in [
            ("0", 0.8444218515250481),
            ("1", 0.13436424411240122),
            ("-1", 0.13436424411240122),
            ("18446744073709551739", 0.7432318140414349),
            ("123456789012345678901234567890", 0.7275084571578186),
        ] {
            assert_eq!(Random::new(&json!(seed)).unwrap().unit(), expected);
        }
    }
    #[test]
    fn reference_plans_match_every_field_without_normalization() {
        let cases: Value =
            serde_json::from_str(include_str!("../../migration/repeat-plan-cases.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let result = plan(&case["args"]);
            let observed = match result {
                Ok(v) => json!({"plan":v}),
                Err(e) => json!({"error":e}),
            };
            assert_eq!(observed, case["result"], "{}", case["name"]);
        }
    }
}
