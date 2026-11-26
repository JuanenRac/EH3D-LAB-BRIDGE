// SPDX-License-Identifier: GPL-3.0-or-later
//! The comparison of a measured curve with a simulated one.
//!
//! The simulated curve is interpolated linearly at every measured x. The deviation is `measured - simulated`. The expanded uncertainty is the coverage factor
//! times the root of the sum of the squares of the measured and simulated uncertainties. A point is judged against the tolerance only when a tolerance and every
//! uncertainty it needs are known: it is within when `|deviation| + U <= tolerance`, outside when `|deviation| - U > tolerance`, and inconclusive in between.

use crate::config::Config;
use crate::curve::Curve;
use crate::error::LbriError;
use serde::Serialize;

/// The judgement of a point or of the whole comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The deviation and its uncertainty fit inside the tolerance.
    Within,
    /// The deviation is beyond the tolerance even after taking away the uncertainty.
    Outside,
    /// The uncertainty is too large to say.
    Inconclusive,
    /// No tolerance was set, or an uncertainty is unknown.
    NotJudged,
}

/// One compared point.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Point {
    /// The x of the measured point.
    pub x: f64,
    /// The measured y.
    pub measured: f64,
    /// The simulated y at that x.
    pub simulated: f64,
    /// `measured - simulated`.
    pub deviation: f64,
    /// The expanded uncertainty of the deviation, when it is known.
    pub expanded_uncertainty: Option<f64>,
    /// The judgement of the point.
    pub verdict: Verdict,
}

/// The result of a comparison.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Comparison {
    /// The unit of x.
    pub x_unit: String,
    /// The unit of y (and of every deviation).
    pub y_unit: String,
    /// The compared points.
    pub points: Vec<Point>,
    /// Measured points that lie outside the simulated curve and were not compared.
    pub outside_range: usize,
    /// The largest absolute deviation, when any point was compared.
    pub max_abs_deviation: Option<f64>,
    /// The x where that largest deviation is (the first one when several are equal), when any point was compared.
    pub max_abs_deviation_at: Option<f64>,
    /// The root mean square of the deviations, when any point was compared.
    pub rms_deviation: Option<f64>,
    /// The tolerance used, if one was set.
    pub tolerance: Option<f64>,
    /// The coverage factor used.
    pub coverage_factor: f64,
    /// The share of the measured points that were compared (1 when none was left out; 0 when no point was compared).
    pub coverage: f64,
    /// The judgement of the whole comparison.
    pub verdict: Verdict,
}

fn judge(deviation: f64, expanded: Option<f64>, tolerance: Option<f64>) -> Verdict {
    match (tolerance, expanded) {
        (Some(t), Some(u)) => {
            let d = deviation.abs();
            if d + u <= t {
                Verdict::Within
            } else if d - u > t {
                Verdict::Outside
            } else {
                Verdict::Inconclusive
            }
        }
        _ => Verdict::NotJudged,
    }
}

/// Compare `measured` with `simulated`.
pub fn compare(
    measured: &Curve,
    simulated: &Curve,
    config: &Config,
) -> Result<Comparison, LbriError> {
    if measured.x_unit != simulated.x_unit || measured.y_unit != simulated.y_unit {
        return Err(LbriError::UnitMismatch {
            what: format!(
                "measured x[{}] y[{}] against simulated x[{}] y[{}]",
                measured.x_unit, measured.y_unit, simulated.x_unit, simulated.y_unit
            ),
        });
    }
    let mut points = Vec::new();
    let mut outside_range = 0;
    for i in 0..measured.x.len() {
        let Some(sim) = simulated.at(measured.x[i]) else {
            outside_range += 1;
            continue;
        };
        let u_meas = measured
            .u
            .as_ref()
            .map(|u| u[i])
            .or(config.measurement_uncertainty);
        let u_sim = simulated
            .u_at(measured.x[i])
            .or(config.simulation_uncertainty);
        let expanded = match (u_meas, u_sim) {
            (Some(a), Some(b)) => Some(config.coverage_factor * (a * a + b * b).sqrt()),
            _ => None,
        };
        let deviation = measured.y[i] - sim;
        points.push(Point {
            x: measured.x[i],
            measured: measured.y[i],
            simulated: sim,
            deviation,
            expanded_uncertainty: expanded,
            verdict: judge(deviation, expanded, config.tolerance),
        });
    }
    let n = points.len();
    let max_abs_deviation = points.iter().map(|p| p.deviation.abs()).reduce(f64::max);
    let max_abs_deviation_at = points
        .iter()
        .fold(None, |best: Option<&Point>, p| match best {
            Some(b) if b.deviation.abs() >= p.deviation.abs() => Some(b),
            _ => Some(p),
        })
        .map(|p| p.x);
    let rms_deviation = (n > 0).then(|| {
        (points
            .iter()
            .map(|p| p.deviation * p.deviation)
            .sum::<f64>()
            / n as f64)
            .sqrt()
    });
    let measured_points = n + outside_range;
    let coverage = if measured_points == 0 {
        0.0
    } else {
        n as f64 / measured_points as f64
    };
    let verdict = if n == 0 {
        Verdict::Inconclusive
    } else if points.iter().any(|p| p.verdict == Verdict::Outside) {
        Verdict::Outside
    } else if points.iter().any(|p| p.verdict == Verdict::NotJudged) {
        Verdict::NotJudged
    } else if points.iter().any(|p| p.verdict == Verdict::Inconclusive) {
        Verdict::Inconclusive
    } else if coverage < config.min_coverage {
        // every compared point fits, but too many measured points were left out to say the whole curve does
        Verdict::Inconclusive
    } else {
        Verdict::Within
    };
    Ok(Comparison {
        x_unit: measured.x_unit.clone(),
        y_unit: measured.y_unit.clone(),
        points,
        outside_range,
        max_abs_deviation,
        max_abs_deviation_at,
        rms_deviation,
        tolerance: config.tolerance,
        coverage_factor: config.coverage_factor,
        coverage,
        verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(text: &str) -> Curve {
        Curve::from_csv(text, &Config::default()).unwrap()
    }

    #[test]
    fn a_known_offset_is_recovered() {
        let sim = curve("x[s],y[V]\n0,0\n10,10\n");
        // the measured curve is the simulated one plus 0.25 V everywhere
        let meas = curve("x[s],y[V]\n1,1.25\n2,2.25\n5,5.25\n9,9.25\n");
        let c = compare(&meas, &sim, &Config::default()).unwrap();
        assert_eq!(c.points.len(), 4);
        for p in &c.points {
            assert!((p.deviation - 0.25).abs() < 1e-12);
        }
        assert!((c.max_abs_deviation.unwrap() - 0.25).abs() < 1e-12);
        assert!((c.rms_deviation.unwrap() - 0.25).abs() < 1e-12);
    }

    #[test]
    fn the_largest_deviation_says_where_it_is_and_the_first_wins_a_tie() {
        let sim = curve("x[s],y[V]\n0,0\n10,0\n");
        let meas = curve("x[s],y[V]\n1,0.1\n2,-0.4\n3,0.4\n4,0.2\n");
        let c = compare(&meas, &sim, &Config::default()).unwrap();
        assert_eq!(c.max_abs_deviation_at, Some(2.0));
        assert!((c.max_abs_deviation.unwrap() - 0.4).abs() < 1e-12);
        // a single point, and no point at all
        let one = compare(&curve("x[s],y[V]\n5,0.3\n"), &sim, &Config::default()).unwrap();
        assert_eq!(one.max_abs_deviation_at, Some(5.0));
        let none = compare(&curve("x[s],y[V]\n50,0.3\n"), &sim, &Config::default()).unwrap();
        assert_eq!(
            (none.max_abs_deviation, none.max_abs_deviation_at),
            (None, None)
        );
    }

    #[test]
    fn without_a_tolerance_nothing_is_judged() {
        let sim = curve("x[s],y[V]\n0,0\n10,10\n");
        let meas = curve("x[s],y[V],u[V]\n1,3,0.01\n");
        let c = compare(&meas, &sim, &Config::default()).unwrap();
        assert_eq!(c.verdict, Verdict::NotJudged);
        assert_eq!(c.tolerance, None);
        assert_eq!(c.points[0].expanded_uncertainty, None);
    }

    #[test]
    fn the_uncertainty_decides_between_within_inconclusive_and_outside() {
        let sim = curve("x[s],y[V],u[V]\n0,0,0.1\n10,10,0.1\n");
        let cfg = Config {
            tolerance: Some(0.5),
            ..Config::default()
        };
        let u = 2.0 * (0.1f64 * 0.1 + 0.1 * 0.1).sqrt();
        // deviation 0.1: 0.1 + U <= 0.5
        let within = compare(&curve("x[s],y[V],u[V]\n5,5.1,0.1\n"), &sim, &cfg).unwrap();
        assert_eq!(within.verdict, Verdict::Within);
        assert!((within.points[0].expanded_uncertainty.unwrap() - u).abs() < 1e-12);
        // deviation 0.4: beyond what the tolerance leaves after U
        let unsure = compare(&curve("x[s],y[V],u[V]\n5,5.4,0.1\n"), &sim, &cfg).unwrap();
        assert_eq!(unsure.verdict, Verdict::Inconclusive);
        // deviation 1.0: even 1.0 - U is above 0.5
        let out = compare(&curve("x[s],y[V],u[V]\n5,6.0,0.1\n"), &sim, &cfg).unwrap();
        assert_eq!(out.verdict, Verdict::Outside);
    }

    #[test]
    fn default_uncertainties_from_the_settings_are_used() {
        let sim = curve("x[s],y[V]\n0,0\n10,10\n");
        let meas = curve("x[s],y[V]\n5,5.01\n");
        let cfg = Config {
            tolerance: Some(0.5),
            measurement_uncertainty: Some(0.01),
            simulation_uncertainty: Some(0.01),
            ..Config::default()
        };
        assert_eq!(compare(&meas, &sim, &cfg).unwrap().verdict, Verdict::Within);
    }

    #[test]
    fn a_curve_mostly_outside_the_simulation_is_never_within() {
        let sim = curve("x[s],y[V]\n0,0\n1,1\n");
        let mut text = String::from("x[s],y[V]\n0.5,0.5\n");
        for i in 2..=11 {
            text.push_str(&format!("{i},{i}\n"));
        }
        let meas = curve(&text);
        let cfg = Config {
            tolerance: Some(0.1),
            measurement_uncertainty: Some(0.0),
            simulation_uncertainty: Some(0.0),
            ..Config::default()
        };
        let c = compare(&meas, &sim, &cfg).unwrap();
        assert_eq!((c.points.len(), c.outside_range), (1, 10));
        assert!((c.coverage - 1.0 / 11.0).abs() < 1e-12);
        assert_eq!(c.points[0].verdict, Verdict::Within);
        assert_eq!(c.verdict, Verdict::Inconclusive);
        // with the check turned off the earlier verdict comes back
        let off = Config {
            min_coverage: 0.0,
            ..cfg
        };
        assert_eq!(compare(&meas, &sim, &off).unwrap().verdict, Verdict::Within);
    }

    #[test]
    fn a_point_beyond_the_tolerance_stays_outside_whatever_the_coverage() {
        let sim = curve("x[s],y[V]\n0,0\n1,1\n");
        let meas = curve("x[s],y[V]\n0.5,5\n7,7\n");
        let cfg = Config {
            tolerance: Some(0.1),
            measurement_uncertainty: Some(0.0),
            simulation_uncertainty: Some(0.0),
            ..Config::default()
        };
        assert_eq!(
            compare(&meas, &sim, &cfg).unwrap().verdict,
            Verdict::Outside
        );
    }

    #[test]
    fn points_outside_the_simulation_are_counted_not_invented() {
        let sim = curve("x[s],y[V]\n1,1\n2,2\n");
        let meas = curve("x[s],y[V]\n0,0\n1.5,1.5\n3,3\n");
        let c = compare(&meas, &sim, &Config::default()).unwrap();
        assert_eq!(c.points.len(), 1);
        assert_eq!(c.outside_range, 2);
        let none = compare(&curve("x[s],y[V]\n9,9\n"), &sim, &Config::default()).unwrap();
        assert_eq!(none.verdict, Verdict::Inconclusive);
        assert_eq!(none.max_abs_deviation, None);
    }

    #[test]
    fn different_units_are_refused() {
        let a = curve("x[s],y[V]\n1,1\n");
        let b = curve("x[s],y[A]\n1,1\n");
        assert!(matches!(
            compare(&a, &b, &Config::default()),
            Err(LbriError::UnitMismatch { .. })
        ));
    }

    #[test]
    fn a_deviation_exactly_on_the_tolerance_is_within_and_a_hair_beyond_is_not() {
        let sim = curve("x[s],y[V]\n0,0\n10,0\n");
        let cfg = Config {
            tolerance: Some(0.5),
            measurement_uncertainty: Some(0.0),
            simulation_uncertainty: Some(0.0),
            ..Config::default()
        };
        let on = compare(&curve("x[s],y[V]\n5,0.5\n"), &sim, &cfg).unwrap();
        assert_eq!(on.verdict, Verdict::Within);
        let below = compare(&curve("x[s],y[V]\n5,-0.5\n"), &sim, &cfg).unwrap();
        assert_eq!(below.verdict, Verdict::Within);
        let beyond = compare(&curve("x[s],y[V]\n5,-0.5000001\n"), &sim, &cfg).unwrap();
        assert_eq!(beyond.verdict, Verdict::Outside);
    }

    #[test]
    fn a_coverage_exactly_on_the_smallest_share_is_enough() {
        let sim = curve("x[s],y[V]\n0,0\n9,0\n");
        let mut text = String::from("x[s],y[V]\n");
        for i in 0..10 {
            text.push_str(&format!("{i},0\n"));
        }
        text.push_str("10,0\n");
        let cfg = Config {
            tolerance: Some(0.1),
            measurement_uncertainty: Some(0.0),
            simulation_uncertainty: Some(0.0),
            min_coverage: 10.0 / 11.0,
            ..Config::default()
        };
        let c = compare(&curve(&text), &sim, &cfg).unwrap();
        assert_eq!((c.points.len(), c.outside_range), (10, 1));
        assert_eq!(c.verdict, Verdict::Within);
    }
}
