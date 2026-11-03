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
    /// The root mean square of the deviations, when any point was compared.
    pub rms_deviation: Option<f64>,
    /// The tolerance used, if one was set.
    pub tolerance: Option<f64>,
    /// The coverage factor used.
    pub coverage_factor: f64,
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
    let rms_deviation = (n > 0).then(|| {
        (points
            .iter()
            .map(|p| p.deviation * p.deviation)
            .sum::<f64>()
            / n as f64)
            .sqrt()
    });
    let verdict = if n == 0 {
        Verdict::Inconclusive
    } else if points.iter().any(|p| p.verdict == Verdict::Outside) {
        Verdict::Outside
    } else if points.iter().any(|p| p.verdict == Verdict::NotJudged) {
        Verdict::NotJudged
    } else if points.iter().any(|p| p.verdict == Verdict::Inconclusive) {
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
        rms_deviation,
        tolerance: config.tolerance,
        coverage_factor: config.coverage_factor,
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
}
