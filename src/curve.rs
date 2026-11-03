// SPDX-License-Identifier: GPL-3.0-or-later
//! The curve and the one file format that is read: a CSV whose header names the unit of each column.
//!
//! ```text
//! x[s],y[V],u[V]
//! 0.0,0.01,0.002
//! 0.1,0.52,0.002
//! ```
//!
//! `u` is optional (the standard uncertainty of that point, in the unit of y). The x values must increase strictly. Values and units are kept exactly.

use crate::config::Config;
use crate::error::LbriError;

/// A curve of points `(x, y)` with an optional uncertainty per point.
#[derive(Debug, Clone, PartialEq)]
pub struct Curve {
    /// The unit of x.
    pub x_unit: String,
    /// The unit of y.
    pub y_unit: String,
    /// The x values, strictly increasing.
    pub x: Vec<f64>,
    /// The y values.
    pub y: Vec<f64>,
    /// The standard uncertainty of each y, when the file has the column.
    pub u: Option<Vec<f64>>,
}

fn unit_of(cell: &str, name: &str, line: usize) -> Result<String, LbriError> {
    let bad = |what: String| LbriError::BadCurve { line, what };
    let cell = cell.trim();
    let rest = cell
        .strip_prefix(name)
        .and_then(|r| r.strip_prefix('['))
        .and_then(|r| r.strip_suffix(']'))
        .ok_or_else(|| bad(format!("the header cell `{cell}` is not `{name}[unit]`")))?;
    if rest.trim().is_empty() {
        return Err(bad(format!("the unit of `{name}` is empty")));
    }
    Ok(rest.trim().to_string())
}

impl Curve {
    /// Read a curve from the text of the file.
    pub fn from_csv(text: &str, config: &Config) -> Result<Curve, LbriError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut lines = text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty());
        let (hi, header) = lines.next().ok_or(LbriError::BadCurve {
            line: 0,
            what: "the file is empty".into(),
        })?;
        let cells: Vec<&str> = header.split(',').collect();
        if cells.len() != 2 && cells.len() != 3 {
            return Err(LbriError::BadCurve {
                line: hi + 1,
                what: "the header must have 2 or 3 columns: x[unit],y[unit] and optionally u[unit]"
                    .into(),
            });
        }
        let x_unit = unit_of(cells[0], "x", hi + 1)?;
        let y_unit = unit_of(cells[1], "y", hi + 1)?;
        if cells.len() == 3 {
            let u_unit = unit_of(cells[2], "u", hi + 1)?;
            if u_unit != y_unit {
                return Err(LbriError::BadCurve {
                    line: hi + 1,
                    what: format!("u is in `{u_unit}` but y is in `{y_unit}`"),
                });
            }
        }
        let mut curve = Curve {
            x_unit,
            y_unit,
            x: Vec::new(),
            y: Vec::new(),
            u: if cells.len() == 3 {
                Some(Vec::new())
            } else {
                None
            },
        };
        for (i, raw) in lines {
            let line = i + 1;
            if curve.x.len() >= config.max_points {
                return Err(LbriError::TooLarge {
                    what: "points of the curve",
                });
            }
            let cols: Vec<&str> = raw.split(',').collect();
            if cols.len() != cells.len() {
                return Err(LbriError::BadCurve {
                    line,
                    what: format!("expected {} columns, found {}", cells.len(), cols.len()),
                });
            }
            let num = |s: &str| -> Result<f64, LbriError> {
                let n: f64 = s.trim().parse().map_err(|_| LbriError::BadCurve {
                    line,
                    what: format!("`{}` is not a number", s.trim()),
                })?;
                if n.is_finite() {
                    Ok(n)
                } else {
                    Err(LbriError::BadCurve {
                        line,
                        what: "a value is not finite".into(),
                    })
                }
            };
            let (x, y) = (num(cols[0])?, num(cols[1])?);
            if let Some(&last) = curve.x.last() {
                if x <= last {
                    return Err(LbriError::BadCurve {
                        line,
                        what: format!("x = {x} does not increase after {last}"),
                    });
                }
            }
            if let Some(u) = curve.u.as_mut() {
                let v = num(cols[2])?;
                if v < 0.0 {
                    return Err(LbriError::BadCurve {
                        line,
                        what: "an uncertainty is negative".into(),
                    });
                }
                u.push(v);
            }
            curve.x.push(x);
            curve.y.push(y);
        }
        if curve.x.is_empty() {
            return Err(LbriError::BadCurve {
                line: 0,
                what: "the curve has no points".into(),
            });
        }
        Ok(curve)
    }

    /// The curve as the same CSV text (numbers written so they read back to the same values).
    pub fn to_csv(&self) -> String {
        let mut out = format!("x[{}],y[{}]", self.x_unit, self.y_unit);
        if self.u.is_some() {
            out.push_str(&format!(",u[{}]", self.y_unit));
        }
        out.push('\n');
        for i in 0..self.x.len() {
            out.push_str(&format!("{:?},{:?}", self.x[i], self.y[i]));
            if let Some(u) = &self.u {
                out.push_str(&format!(",{:?}", u[i]));
            }
            out.push('\n');
        }
        out
    }

    /// The value at `x` by linear interpolation, or None when `x` is outside the curve.
    pub fn at(&self, x: f64) -> Option<f64> {
        self.interpolate(&self.y, x)
    }

    /// The uncertainty at `x` by linear interpolation, or None when the curve has none or `x` is outside it.
    pub fn u_at(&self, x: f64) -> Option<f64> {
        self.interpolate(self.u.as_ref()?, x)
    }

    fn interpolate(&self, values: &[f64], x: f64) -> Option<f64> {
        let (first, last) = (*self.x.first()?, *self.x.last()?);
        if !(first..=last).contains(&x) {
            return None;
        }
        let hi = self.x.partition_point(|&v| v < x);
        if self.x[hi] == x || hi == 0 {
            return Some(values[hi]);
        }
        let (x0, x1) = (self.x[hi - 1], self.x[hi]);
        let t = (x - x0) / (x1 - x0);
        Some(values[hi - 1] + t * (values[hi] - values[hi - 1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config::default()
    }

    #[test]
    fn reads_values_and_units_exactly_and_writes_them_back() {
        let text = "x[s],y[V],u[V]\n0.0,0.01,0.002\n0.1,0.52,0.002\n0.25,1e-3,0.003\n";
        let c = Curve::from_csv(text, &cfg()).unwrap();
        assert_eq!((c.x_unit.as_str(), c.y_unit.as_str()), ("s", "V"));
        assert_eq!(c.x, vec![0.0, 0.1, 0.25]);
        assert_eq!(c.y, vec![0.01, 0.52, 0.001]);
        assert_eq!(c.u.as_deref(), Some(&[0.002, 0.002, 0.003][..]));
        assert_eq!(Curve::from_csv(&c.to_csv(), &cfg()).unwrap(), c);
    }

    #[test]
    fn bom_blank_lines_and_crlf_are_accepted() {
        let c = Curve::from_csv("\u{feff}x[Hz],y[dB]\r\n\r\n1,2\r\n2,3\r\n", &cfg()).unwrap();
        assert_eq!(c.x_unit, "Hz");
        assert_eq!(c.x.len(), 2);
        assert!(c.u.is_none());
    }

    #[test]
    fn bad_files_are_refused_with_the_line() {
        for (text, line) in [
            ("", 0),
            ("x,y\n1,2\n", 1),
            ("x[s],y[]\n1,2\n", 1),
            ("x[s],y[V],u[A]\n1,2,3\n", 1),
            ("x[s],y[V]\n1,2\n1,3\n", 3),
            ("x[s],y[V]\n1,2\n0,3\n", 3),
            ("x[s],y[V]\n1,abc\n", 2),
            ("x[s],y[V]\n1,inf\n", 2),
            ("x[s],y[V]\n1\n", 2),
            ("x[s],y[V],u[V]\n1,2,-1\n", 2),
            ("x[s],y[V]\n", 0),
            ("a,b,c,d\n", 1),
        ] {
            match Curve::from_csv(text, &cfg()) {
                Err(LbriError::BadCurve { line: l, .. }) => assert_eq!(l, line, "{text:?}"),
                other => panic!("{text:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn too_many_points_are_refused() {
        let small = Config {
            max_points: 2,
            ..Config::default()
        };
        assert!(matches!(
            Curve::from_csv("x[s],y[V]\n1,1\n2,2\n3,3\n", &small),
            Err(LbriError::TooLarge { .. })
        ));
    }

    #[test]
    fn interpolation_is_linear_and_stops_at_the_ends() {
        let c = Curve::from_csv("x[s],y[V]\n0,0\n2,4\n4,4\n", &cfg()).unwrap();
        assert_eq!(c.at(1.0), Some(2.0));
        assert_eq!(c.at(2.0), Some(4.0));
        assert_eq!(c.at(0.0), Some(0.0));
        assert_eq!(c.at(4.0), Some(4.0));
        assert_eq!(c.at(-0.1), None);
        assert_eq!(c.at(4.1), None);
    }
}
