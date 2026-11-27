// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the work lives in the other modules.
//!
//! Exit codes: 0 done and nothing found, 1 findings or a comparison that is outside the tolerance, 2 the input or the arguments are unusable,
//! 3 a comparison that could not be judged (no tolerance, an unknown uncertainty or an uncertainty too large to say; `exit_on_unjudged = no` turns this off).

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
Exit codes: 0 done, 1 findings or outside the tolerance, 2 unusable input, 3 compared but not judged (see exit_on_unjudged).
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

/// A text from an input file, fit to print on one line: control characters and the Unicode line and paragraph separators become spaces.
/// Only what is printed goes through here; what is compared or hashed does not.
fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
                ' '
            } else {
                c
            }
        })
        .collect()
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
            let _ = writeln!(err, "error: {}", one_line(&error.to_string()));
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
                one_line(&c.x_unit),
                c.x[0],
                c.x[c.x.len() - 1]
            );
            let _ = writeln!(out, "y: {}", one_line(&c.y_unit));
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
            Ok(match result.verdict {
                Verdict::Outside => 1,
                Verdict::Within => 0,
                Verdict::Inconclusive | Verdict::NotJudged => {
                    if config.exit_on_unjudged {
                        3
                    } else {
                        0
                    }
                }
            })
        }
        ["record", path] | ["record", path, _] => {
            // replacing the bytes that are not text would accept a record that is not the one that was written
            let text = String::from_utf8(bytes(path)?)
                .map_err(|_| Some(LbriError::BadJson("the file is not UTF-8 text".into())))?;
            let record = TestRecord::from_json(&text).map_err(Some)?;
            let data = match rest.get(2) {
                Some(p) => Some(bytes(p)?),
                None => None,
            };
            let findings = record.check(data.as_deref());
            for f in &findings {
                let _ = writeln!(out, "{} [{}]: {}", f.code.as_str(), f.subject, f.message);
            }
            if data.is_none() {
                let _ = writeln!(
                    out,
                    "# no data file given: the hash was not compared with any data"
                );
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
        assert_eq!(
            code, 3,
            "a comparison that was not judged must not look like a pass"
        );
        assert!(out.contains("\"verdict\": \"not_judged\""), "{out}");
        let off = file(&dir, "off.cfg", "exit_on_unjudged = no\n");
        let (code, _, _) = run_args(&["--config", &off, "compare", &meas, &sim]);
        assert_eq!(code, 0);
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

    #[test]
    fn a_record_that_is_not_utf8_text_ends_with_2_and_is_not_accepted_with_replaced_letters() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("latin1.json");
        let mut bytes = br#"{"instrument":"scope","conditions":["25 "#.to_vec();
        bytes.push(0xB0);
        bytes.extend_from_slice(
            format!(
                r#"C"],"quantity":"v","data_file":"a.csv","data_sha256":"{}"}}"#,
                "0".repeat(64)
            )
            .as_bytes(),
        );
        fs::write(&path, bytes).unwrap();
        let (code, out, err) = run_args(&["record", path.to_str().unwrap()]);
        assert_eq!(code, 2, "{out}");
        assert!(err.contains("UTF-8"), "{err}");
        assert!(out.is_empty(), "{out}");
    }

    #[test]
    fn a_record_checked_without_its_data_says_the_hash_was_not_compared() {
        let work = tempfile::tempdir().unwrap();
        let data = work.path().join("d.csv");
        let record = work.path().join("r.json");
        std::fs::write(&data, "abc").unwrap();
        std::fs::write(
            &record,
            r#"{"instrument":"i","conditions":["c"],"quantity":"q","data_file":"d.csv","data_sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}"#,
        )
        .unwrap();
        let run = |args: &[&str]| {
            let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = crate::cli::run(&args, &mut out, &mut err);
            (code, String::from_utf8(out).unwrap())
        };
        let (code, text) = run(&["record", record.to_str().unwrap()]);
        assert_eq!(code, 0);
        assert!(text.contains("the hash was not compared"), "{text}");
        let (code, text) = run(&["record", record.to_str().unwrap(), data.to_str().unwrap()]);
        assert_eq!(code, 0, "{text}");
        assert!(!text.contains("not compared"), "{text}");
    }

    #[test]
    fn names_read_from_the_input_cannot_start_a_line_of_their_own() {
        let work = tempfile::tempdir().unwrap();
        let file = |name: &str, text: &str| {
            let path = work.path().join(name);
            std::fs::write(&path, text).unwrap();
            path.to_string_lossy().into_owned()
        };
        let call = |list: &[&str]| {
            let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = run(&args, &mut out, &mut err);
            let out = String::from_utf8(out).unwrap();
            let err = String::from_utf8(err).unwrap();
            for c in out.chars().chain(err.chars()) {
                assert!(
                    c == '\n' || !(c.is_control() || c == '\u{2028}' || c == '\u{2029}'),
                    "{c:?} in {out}{err}"
                );
            }
            (code, out, err)
        };
        let curve = file("c.csv", "x[s\rforged],y[V\u{1b}forged\u{2028}]\n0,1\n1,2\n");
        let (code, out, err) = call(&["import", &curve]);
        assert_eq!(code, 0, "{out}{err}");
        assert!(out.contains("forged"), "{out}");
        for line in out.lines() {
            assert!(
                line.starts_with("points: ")
                    || line.starts_with("x: ")
                    || line.starts_with("y: ")
                    || line.starts_with("uncertainty column: ")
                    || line.starts_with("sha256: "),
                "{out}"
            );
        }
        let unknown = file("unknown.json", r#"{"x\nforged\u2028":1}"#);
        let (code, _, err) = call(&["record", &unknown]);
        assert_eq!(code, 2, "{err}");
        assert_eq!(err.lines().count(), 1, "{err}");
        assert!(err.contains("forged"), "{err}");
    }
}
