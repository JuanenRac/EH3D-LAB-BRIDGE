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
            // a file exported with another separator says so, instead of a count of columns that makes no sense
            for (mark, name) in [(';', "semicolons"), ('\t', "tabs")] {
                if header.contains(mark) {
                    return Err(LbriError::BadCurve {
                        line: hi + 1,
                        what: format!(
                            "the columns must be separated by commas, not by {name}; export the file again or replace them"
                        ),
                    });
                }
            }
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
    fn a_header_with_semicolons_or_tabs_says_the_columns_need_commas() {
        for (text, name) in [
            ("x[s];y[V]\n0;1\n", "semicolons"),
            ("x[s];y[V];u[V]\n0;1;0.1\n", "semicolons"),
            ("x[s]\ty[V]\n0\t1\n", "tabs"),
        ] {
            match Curve::from_csv(text, &cfg()) {
                Err(LbriError::BadCurve { line: 1, what }) => {
                    assert!(what.contains(name) && what.contains("commas"), "{what}")
                }
                other => panic!("{text:?}: {other:?}"),
            }
        }
        // a header with the wrong number of columns and no other separator keeps its own message
        match Curve::from_csv("x[s]\n0\n", &cfg()) {
            Err(LbriError::BadCurve { what, .. }) => assert!(what.contains("2 or 3 columns")),
            other => panic!("{other:?}"),
        }
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

    fn lcg(seed: &mut u64) -> u64 {
        *seed = (*seed * 1_103_515_245 + 12_345) % 2_147_483_648;
        *seed >> 8
    }

    fn seeded_curve(points: usize, seed: u64, with_u: bool) -> Curve {
        let mut state = seed;
        let (mut x, mut y, mut u) = (Vec::new(), Vec::new(), Vec::new());
        let mut at = 0.0_f64;
        for _ in 0..points {
            at += 0.001 + (lcg(&mut state) % 1000) as f64 / 7.0;
            x.push(at);
            y.push((lcg(&mut state) % 100_000) as f64 / 13.0 - 3000.0);
            u.push((lcg(&mut state) % 1000) as f64 / 1000.0);
        }
        Curve {
            x_unit: "s".into(),
            y_unit: "V".into(),
            x,
            y,
            u: if with_u { Some(u) } else { None },
        }
    }

    fn bad_line(text: &str) -> usize {
        match Curve::from_csv(text, &cfg()) {
            Err(LbriError::BadCurve { line, .. }) => line,
            other => panic!("{text:?} gave {other:?}"),
        }
    }

    #[test]
    fn a_seeded_curve_written_and_read_again_is_the_same_curve_with_and_without_uncertainty() {
        for (seed, with_u) in [(1_u64, false), (2, true), (3, true), (4, false)] {
            let curve = seeded_curve(50, seed, with_u);
            let text = curve.to_csv();
            let again = Curve::from_csv(&text, &cfg()).unwrap();
            assert_eq!(again, curve, "seed {seed}");
            assert_eq!(again.to_csv(), text, "seed {seed}");
        }
    }

    #[test]
    fn twenty_thousand_points_are_read_without_a_panic() {
        let curve = seeded_curve(20_000, 8, true);
        let again = Curve::from_csv(&curve.to_csv(), &cfg()).unwrap();
        assert_eq!(again.x.len(), 20_000);
        assert_eq!(again, curve);
    }

    #[test]
    fn a_curve_of_one_point_is_valid_and_answers_only_at_that_point() {
        let c = Curve::from_csv("x[s],y[V]\n2.5,7.0\n", &cfg()).unwrap();
        assert_eq!(c.at(2.5), Some(7.0));
        assert_eq!(c.at(2.4999), None);
        assert_eq!(c.at(2.5001), None);
        assert_eq!(c.u_at(2.5), None);
    }

    #[test]
    fn the_curve_answers_exactly_on_every_node_and_between_the_neighbours_elsewhere() {
        let c = seeded_curve(40, 5, true);
        for i in 0..c.x.len() {
            assert_eq!(c.at(c.x[i]), Some(c.y[i]));
            assert_eq!(c.u_at(c.x[i]), c.u.as_ref().map(|u| u[i]));
        }
        for i in 1..c.x.len() {
            let mid = (c.x[i - 1] + c.x[i]) / 2.0;
            let v = c.at(mid).unwrap();
            let (lo, hi) = (c.y[i - 1].min(c.y[i]), c.y[i - 1].max(c.y[i]));
            assert!(v >= lo - 1e-9 && v <= hi + 1e-9, "{i}: {v} {lo} {hi}");
        }
    }

    #[test]
    fn a_question_that_is_not_a_number_or_is_outside_the_curve_has_no_answer() {
        let c = seeded_curve(5, 6, false);
        assert_eq!(c.at(f64::NAN), None);
        assert_eq!(c.at(f64::INFINITY), None);
        assert_eq!(c.at(f64::NEG_INFINITY), None);
        assert_eq!(c.at(c.x[0] - 1e-9), None);
        assert_eq!(c.at(c.x[4] + 1e-9), None);
    }

    #[test]
    fn the_unit_is_kept_with_its_spaces_trimmed_and_in_any_script() {
        let c = Curve::from_csv("x[ \u{b5}s ],y[ \u{3a9} ]\n0,1\n1,2\n", &cfg()).unwrap();
        assert_eq!(c.x_unit, "\u{b5}s");
        assert_eq!(c.y_unit, "\u{3a9}");
        assert!(c.to_csv().starts_with("x[\u{b5}s],y[\u{3a9}]\n"));
    }

    #[test]
    fn each_kind_of_bad_file_is_refused_with_the_line_of_the_fault() {
        assert_eq!(bad_line(""), 0);
        assert_eq!(bad_line("\n\n  \n"), 0);
        assert_eq!(bad_line("x[s],y[V]\n"), 0);
        assert_eq!(bad_line("x[s]\n0\n"), 1);
        assert_eq!(bad_line("x[s],y[V],u[V],z[V]\n0,1,2,3\n"), 1);
        assert_eq!(bad_line("x[],y[V]\n0,1\n"), 1);
        assert_eq!(bad_line("t[s],y[V]\n0,1\n"), 1);
        assert_eq!(bad_line("x[s],y[V],u[A]\n0,1,2\n"), 1);
        assert_eq!(bad_line("x[s],y[V]\n0,abc\n"), 2);
        assert_eq!(bad_line("x[s],y[V]\n0,1\n1\n"), 3);
        assert_eq!(bad_line("x[s],y[V]\n0,1\n0,2\n"), 3);
        assert_eq!(bad_line("x[s],y[V]\n0,1\n-1,2\n"), 3);
        assert_eq!(bad_line("x[s],y[V],u[V]\n0,1,-0.5\n"), 2);
    }

    #[test]
    fn a_value_that_is_not_finite_is_refused_however_it_is_spelt() {
        for bad in ["NaN", "nan", "inf", "-inf", "infinity", "1e999"] {
            let text = format!("x[s],y[V]\n0,{bad}\n");
            assert_eq!(bad_line(&text), 2, "{bad}");
        }
    }

    #[test]
    fn the_first_blank_lines_do_not_move_the_line_numbers_of_the_faults() {
        assert_eq!(bad_line("\n\nx[s],y[V]\n0,1\n0,2\n"), 5);
    }

    #[test]
    fn exactly_the_most_points_is_read_and_one_more_is_not() {
        let config = Config {
            max_points: 3,
            ..Config::default()
        };
        let three = "x[s],y[V]\n0,1\n1,2\n2,3\n";
        let four = "x[s],y[V]\n0,1\n1,2\n2,3\n3,4\n";
        assert!(Curve::from_csv(three, &config).is_ok());
        assert!(matches!(
            Curve::from_csv(four, &config),
            Err(LbriError::TooLarge { .. })
        ));
    }

    #[test]
    fn the_message_of_a_bad_curve_carries_the_stable_code_and_the_line() {
        let text = Curve::from_csv("x[s],y[V]\n0,1\n0,2\n", &cfg())
            .unwrap_err()
            .to_string();
        assert!(text.contains("PHOENIX-LBRI-"), "{text}");
        assert!(text.contains("line 3"), "{text}");
    }

    #[test]
    fn a_zero_uncertainty_is_accepted_and_kept() {
        let c = Curve::from_csv("x[s],y[V],u[V]\n0,1,0\n1,2,0.0\n", &cfg()).unwrap();
        assert_eq!(c.u, Some(vec![0.0, 0.0]));
    }
}
