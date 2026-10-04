//! Shared finite affine arithmetic and bounded SVG transform parsing.
use regex::Regex;
use std::sync::LazyLock;
pub type Matrix = [f64; 6];
pub const IDENTITY: Matrix = [1., 0., 0., 1., 0., 0.];
static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?").unwrap());
static CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(matrix|translate|scale|rotate|skewX|skewY)\s*\(([^()]*)\)").unwrap()
});

pub fn multiply(l: Matrix, r: Matrix) -> Result<Matrix, String> {
    let [a, b, c, d, e, f] = l;
    let [g, h, i, j, k, y] = r;
    let result = [
        a * g + c * h,
        b * g + d * h,
        a * i + c * j,
        b * i + d * j,
        a * k + c * y + e,
        b * k + d * y + f,
    ];
    if result.iter().all(|v| v.is_finite()) {
        Ok(result)
    } else {
        Err("non-finite composed transform".into())
    }
}
pub fn inverse(m: Matrix) -> Result<Matrix, String> {
    let [a, b, c, d, e, f] = m;
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant.abs() < 1e-12 {
        return Err("singular or ill-conditioned parent transform".into());
    }
    Ok([
        d / determinant,
        -b / determinant,
        -c / determinant,
        a / determinant,
        (c * f - d * e) / determinant,
        (b * e - a * f) / determinant,
    ])
}
fn separators(s: &str) -> bool {
    s.trim_matches([' ', ',', '\t', '\r', '\n']).is_empty()
}
pub fn parse(raw: &str) -> Result<Matrix, String> {
    let mut result = IDENTITY;
    let mut cursor = 0;
    if raw.len() > 200_000 {
        return Err("SVG transform exceeds 200000 bytes".into());
    }
    for call in CALL.captures_iter(raw) {
        let matched = call.get(0).unwrap();
        if !separators(&raw[cursor..matched.start()]) {
            return Err("unsupported SVG transform".into());
        }
        let body = &call[2];
        if !separators(&NUMBER.replace_all(body, "")) {
            return Err("invalid SVG transform arguments".into());
        }
        let args = NUMBER
            .find_iter(body)
            .map(|n| {
                n.as_str()
                    .parse::<f64>()
                    .map_err(|_| "invalid SVG transform arguments".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !args.iter().all(|v| v.is_finite()) {
            return Err("non-finite SVG transform".into());
        }
        let m = match (&call[1], args.as_slice()) {
            ("matrix", [a, b, c, d, e, f]) => [*a, *b, *c, *d, *e, *f],
            ("translate", [x]) => [1., 0., 0., 1., *x, 0.],
            ("translate", [x, y]) => [1., 0., 0., 1., *x, *y],
            ("scale", [x]) => [*x, 0., 0., *x, 0., 0.],
            ("scale", [x, y]) => [*x, 0., 0., *y, 0., 0.],
            ("rotate", a) if a.len() == 1 || a.len() == 3 => {
                let angle = a[0] * (std::f64::consts::PI / 180.);
                let (s, c) = (angle.sin(), angle.cos());
                let rotation = [c, s, -s, c, 0., 0.];
                if a.len() == 3 {
                    multiply(
                        multiply([1., 0., 0., 1., a[1], a[2]], rotation)?,
                        [1., 0., 0., 1., -a[1], -a[2]],
                    )?
                } else {
                    rotation
                }
            }
            ("skewX", [a]) => [
                1.,
                0.,
                (a * (std::f64::consts::PI / 180.)).tan(),
                1.,
                0.,
                0.,
            ],
            ("skewY", [a]) => [
                1.,
                (a * (std::f64::consts::PI / 180.)).tan(),
                0.,
                1.,
                0.,
                0.,
            ],
            _ => return Err("unsupported SVG transform arity".into()),
        };
        result = multiply(result, m)?;
        cursor = matched.end();
    }
    if !raw[cursor..].trim().is_empty() {
        return Err("unsupported SVG transform".into());
    }
    Ok(result)
}
/// Match Python's .17g matrix serialization rather than shortening f64 values.
pub fn general(value: f64) -> String {
    let scientific = format!("{value:.16e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let sign = if value.is_sign_negative() { "-" } else { "" };
    let digits = mantissa.trim_start_matches('-').replace('.', "");
    let significant = digits.trim_end_matches('0');
    let significant = if significant.is_empty() {
        "0"
    } else {
        significant
    };
    if !(-4..17).contains(&exponent) {
        let head = &significant[..1];
        let rest = &significant[1..];
        let fraction = if rest.is_empty() {
            String::new()
        } else {
            format!(".{rest}")
        };
        return format!(
            "{sign}{head}{fraction}e{}{:02}",
            if exponent < 0 { "-" } else { "+" },
            exponent.abs()
        );
    }
    let point = exponent + 1;
    if point <= 0 {
        format!("{sign}0.{}{significant}", "0".repeat((-point) as usize))
    } else if point as usize >= significant.len() {
        format!(
            "{sign}{significant}{}",
            "0".repeat(point as usize - significant.len())
        )
    } else {
        let (a, b) = significant.split_at(point as usize);
        format!("{sign}{a}.{b}")
    }
}
