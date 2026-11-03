// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the work lives in the other modules.
//!
//! Exit codes: 0 done and nothing found, 1 findings or a comparison that is outside the tolerance, 2 the input or the arguments are unusable.

use crate::compare::{compare, Verdict};
use crate::config::Config;
use crate::curve::Curve;
use crate::error::LbriError;
use crate::record::{sha256_hex, TestRecord};
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-lab-bridge [--config FILE] <command>

  import <curve.csv>                       read a measured curve (header x[unit],y[unit] and optionally u[unit]) and print its summary and SHA-256
  compare <measured.csv> <simulated.csv>   deviation of the measured curve from the simulated one, with its uncertainty, as JSON
  record <record.json> [data.csv]          check a test record (instrument, conditions, quantity, hash); with the data file, check the hash too

The tolerance and the uncertainties come only from the settings file; without a tolerance the comparison is not judged.
Exit codes: 0 done, 1 findings or outside the tolerance, 2 unusable input.
";

fn read_bytes(path: &Path, limit: u64) -> Result<Vec<u8>, LbriError> {
    let io = |source| LbriError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(LbriError::TooLarge { what: "input file" });
    }
    fs::read(path).map_err(io)
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Option<LbriError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        let text = read_bytes(Path::new(&args[1]), 1024 * 1024).map_err(Some)?;
        config = Config::parse(&String::from_utf8_lossy(&text)).map_err(Some)?;
        args.drain(..2);
    }
    let bytes = |path: &str| read_bytes(Path::new(path), config.max_file_bytes).map_err(Some);
    let curve = |path: &str| -> Result<Curve, Option<LbriError>> {
        let raw = bytes(path)?;
        let text = String::from_utf8(raw).map_err(|_| {
            Some(LbriError::BadCurve {
                line: 0,
                what: "the file is not UTF-8 text".into(),
            })
        })?;
        Curve::from_csv(&text, &config).map_err(Some)
    };
    let rest: Vec<&str> = args.iter().map(String::as_str).collect();
    match rest.as_slice() {
        ["import", path] => {
            let raw = bytes(path)?;
            let c = curve(path)?;
            let _ = writeln!(out, "points: {}", c.x.len());
            let _ = writeln!(
                out,
                "x: {} from {:?} to {:?}",
                c.x_unit,
                c.x[0],
                c.x[c.x.len() - 1]
            );
            let _ = writeln!(out, "y: {}", c.y_unit);
            let _ = writeln!(
                out,
                "uncertainty column: {}",
                if c.u.is_some() { "yes" } else { "no" }
            );
            let _ = writeln!(out, "sha256: {}", sha256_hex(&raw));
            Ok(0)
        }
        ["compare", measured, simulated] => {
            let result = compare(&curve(measured)?, &curve(simulated)?, &config).map_err(Some)?;
            let _ = writeln!(
                out,
                "{}",
                serde_json::to_string_pretty(&result).unwrap_or_default()
            );
            Ok(if result.verdict == Verdict::Outside {
                1
            } else {
                0
            })
        }
        ["record", path] | ["record", path, _] => {
            let text = String::from_utf8_lossy(&bytes(path)?).into_owned();
            let record = TestRecord::from_json(&text).map_err(Some)?;
            let data = match rest.get(2) {
                Some(p) => Some(bytes(p)?),
                None => None,
            };
            let findings = record.check(data.as_deref());
            for f in &findings {
                let _ = writeln!(out, "{} [{}]: {}", f.code.as_str(), f.subject, f.message);
            }
            let _ = writeln!(out, "# {} findings", findings.len());
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_args(args: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn file(dir: &tempfile::TempDir, name: &str, text: &str) -> String {
        let p = dir.path().join(name);
        fs::write(&p, text).unwrap();
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn import_prints_the_summary_and_the_hash() {
        let dir = tempfile::tempdir().unwrap();
        let f = file(&dir, "m.csv", "x[s],y[V]\n0,1\n1,2\n");
        let (code, out, _) = run_args(&["import", &f]);
        assert_eq!(code, 0);
        assert!(
            out.contains("points: 2") && out.contains("y: V") && out.contains("sha256: "),
            "{out}"
        );
    }

    #[test]
    fn compare_exits_1_only_when_outside_the_set_tolerance() {
        let dir = tempfile::tempdir().unwrap();
        let sim = file(&dir, "s.csv", "x[s],y[V]\n0,0\n10,10\n");
        let meas = file(&dir, "m.csv", "x[s],y[V]\n5,6\n");
        let (code, out, _) = run_args(&["compare", &meas, &sim]);
        assert_eq!(code, 0);
        assert!(out.contains("\"verdict\": \"not_judged\""), "{out}");
        let cfg = file(
            &dir,
            "c.cfg",
            "tolerance = 0.1\nmeasurement_uncertainty = 0.01\nsimulation_uncertainty = 0.01\n",
        );
        let (code, out, _) = run_args(&["--config", &cfg, "compare", &meas, &sim]);
        assert_eq!(code, 1);
        assert!(out.contains("\"verdict\": \"outside\""), "{out}");
    }

    #[test]
    fn unit_mismatch_and_bad_files_exit_2() {
        let dir = tempfile::tempdir().unwrap();
        let a = file(&dir, "a.csv", "x[s],y[V]\n1,1\n");
        let b = file(&dir, "b.csv", "x[s],y[A]\n1,1\n");
        let (code, _, err) = run_args(&["compare", &a, &b]);
        assert_eq!(code, 2);
        assert!(err.contains("PHOENIX-LBRI-004"), "{err}");
        let bad = file(&dir, "bad.csv", "nonsense\n");
        let (code, _, err) = run_args(&["import", &bad]);
        assert_eq!(code, 2);
        assert!(err.contains("PHOENIX-LBRI-001"), "{err}");
        assert_eq!(run_args(&["import", "missing.csv"]).0, 2);
        assert_eq!(run_args(&[]).0, 2);
    }

    #[test]
    fn record_is_checked_against_its_data() {
        let dir = tempfile::tempdir().unwrap();
        let data = file(&dir, "d.csv", "x[s],y[V]\n1,1\n");
        let hash = sha256_hex(b"x[s],y[V]\n1,1\n");
        let rec = file(
            &dir,
            "r.json",
            &format!(
                r#"{{"instrument":"Scope","conditions":["5 V"],"quantity":"v","data_file":"d.csv","data_sha256":"{hash}"}}"#
            ),
        );
        let (code, out, _) = run_args(&["record", &rec, &data]);
        assert_eq!(code, 0, "{out}");
        let other = file(&dir, "o.csv", "x[s],y[V]\n1,2\n");
        let (code, out, _) = run_args(&["record", &rec, &other]);
        assert_eq!(code, 1);
        assert!(out.contains("PHOENIX-LBRI-003"), "{out}");
        let none = file(
            &dir,
            "n.json",
            r#"{"instrument":"","conditions":[],"quantity":"v","data_file":"d.csv","data_sha256":"x"}"#,
        );
        let (code, out, _) = run_args(&["record", &none]);
        assert_eq!(code, 1);
        assert!(out.contains("[instrument]"), "{out}");
    }
}
