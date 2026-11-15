//! The simulated bench: what the readers and the comparison do with the files it writes, with the failures a bench has, and that they work again
//! with the next clean file. Fixed seeds and no clock: every test gives the same result on every computer.

use eh3d_lab_bridge::compare::{compare, Verdict};
use eh3d_lab_bridge::config::Config;
use eh3d_lab_bridge::record::{sha256_hex, TestRecord};
use eh3d_lab_bridge::sim::{Fault, SimBench};
use eh3d_lab_bridge::{Code, Curve, LbriError};

const POINTS: usize = 100;

fn truth() -> Curve {
    SimBench::line("s", "V", POINTS, 0.1, 0.5)
}

fn config() -> Config {
    Config {
        tolerance: Some(0.1),
        simulation_uncertainty: Some(0.01),
        ..Config::default()
    }
}

fn bad_curve_line(text: &str) -> usize {
    match Curve::from_csv(text, &Config::default()) {
        Err(LbriError::BadCurve { line, .. }) => line,
        other => panic!("expected a refused file, got {other:?}"),
    }
}

#[test]
fn the_same_seed_writes_the_same_file_and_another_seed_another() {
    let a = SimBench::new(7, 0.01).export(&truth(), &[]);
    let b = SimBench::new(7, 0.01).export(&truth(), &[]);
    let c = SimBench::new(8, 0.01).export(&truth(), &[]);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn a_clean_export_is_read_back_and_agrees_with_the_true_line() {
    let text = SimBench::new(1, 0.01).export(&truth(), &[]);
    let measured = Curve::from_csv(&text, &Config::default()).unwrap();
    assert_eq!(measured.x.len(), POINTS);
    let c = compare(&measured, &truth(), &config()).unwrap();
    assert_eq!(c.verdict, Verdict::Within);
    assert!(c.max_abs_deviation.unwrap() <= 0.01 + 1e-12);
}

#[test]
fn a_spike_is_found_at_its_point_and_the_next_clean_export_is_judged_again() {
    let mut bench = SimBench::new(2, 0.01);
    let text = bench.export(
        &truth(),
        &[Fault::Spike {
            index: 40,
            size: 1.0,
        }],
    );
    let measured = Curve::from_csv(&text, &Config::default()).unwrap();
    let c = compare(&measured, &truth(), &config()).unwrap();
    assert_eq!(c.verdict, Verdict::Outside);
    let outside: Vec<usize> = c
        .points
        .iter()
        .enumerate()
        .filter(|(_, p)| p.verdict == Verdict::Outside)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(outside, [40], "only the spiked point is outside");
    // the bench is fine again
    let again = Curve::from_csv(&bench.export(&truth(), &[]), &Config::default()).unwrap();
    assert_eq!(
        compare(&again, &truth(), &config()).unwrap().verdict,
        Verdict::Within
    );
}

#[test]
fn a_drift_starts_within_and_ends_outside() {
    let text = SimBench::new(3, 0.01).export(&truth(), &[Fault::Drift { per_point: 0.01 }]);
    let measured = Curve::from_csv(&text, &Config::default()).unwrap();
    let c = compare(&measured, &truth(), &config()).unwrap();
    assert_eq!(c.points[0].verdict, Verdict::Within);
    assert_eq!(c.points[POINTS - 1].verdict, Verdict::Outside);
    assert_eq!(c.verdict, Verdict::Outside);
}

#[test]
fn a_clock_that_steps_back_is_refused_at_its_line() {
    let text = SimBench::new(4, 0.01).export(&truth(), &[Fault::ClockGlitch { index: 6 }]);
    // line 1 is the header, so point 6 is on line 8
    assert_eq!(bad_curve_line(&text), 8);
}

#[test]
fn lost_points_leave_a_valid_file_with_a_gap_that_is_not_filled_in() {
    let text = SimBench::new(5, 0.01).export(&truth(), &[Fault::DropPoints { from: 10, count: 5 }]);
    let measured = Curve::from_csv(&text, &Config::default()).unwrap();
    assert_eq!(measured.x.len(), POINTS - 5);
    let c = compare(&measured, &truth(), &config()).unwrap();
    assert_eq!(
        c.points.len(),
        POINTS - 5,
        "only measured points are compared"
    );
    assert_eq!(c.verdict, Verdict::Within);
}

#[test]
fn a_lost_unit_a_text_value_a_cut_file_and_an_empty_file_are_each_refused_where_they_are() {
    let mut bench = SimBench::new(6, 0.01);
    assert_eq!(
        bad_curve_line(&bench.export(&truth(), &[Fault::UnitLost])),
        1
    );
    assert_eq!(
        bad_curve_line(&bench.export(&truth(), &[Fault::NotANumber { index: 3 }])),
        5
    );
    assert_eq!(
        bad_curve_line(&bench.export(&truth(), &[Fault::Truncated { points: 4 }])),
        6
    );
    assert_eq!(bad_curve_line(&bench.export(&truth(), &[Fault::Empty])), 0);
}

#[test]
fn a_record_made_from_the_clean_file_does_not_accept_the_cut_one() {
    let mut bench = SimBench::new(9, 0.01);
    let clean = bench.export(&truth(), &[]);
    let record = TestRecord {
        instrument: "simulated bench".into(),
        conditions: vec!["20 C".into()],
        quantity: "output voltage".into(),
        data_file: "run.csv".into(),
        data_sha256: sha256_hex(clean.as_bytes()),
        note: String::new(),
    };
    assert!(record.check(Some(clean.as_bytes())).is_empty());
    let cut = format!(
        "{}\n0.5",
        clean.lines().take(10).collect::<Vec<_>>().join("\n")
    );
    let findings = record.check(Some(cut.as_bytes()));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, Code::HashMismatch);
}
