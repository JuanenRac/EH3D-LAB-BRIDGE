// SPDX-License-Identifier: GPL-3.0-or-later
//! A simulated bench: it writes the file an instrument would export, with the failures such files really have.
//!
//! [`SimBench`] makes a curve from a true line plus noise and writes it in the one defined CSV format. A list of [`Fault`]s damages the export the way a
//! bench does: a spike, a drift, a clock that steps back, points lost, a unit lost, a value that is not a number, a file cut short or an empty one.
//! Everything comes from a seeded generator, so the same seed writes the same file on every computer.
//!
//! This is not the control of an instrument: nothing here talks to hardware. It only gives the readers and the comparison something to be tested with, and
//! shows how they behave when the file is wrong. No real instrument has been used.

use crate::curve::Curve;

/// One damage of an exported file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fault {
    /// One point is far from the line (a glitch of the probe).
    Spike {
        /// The point (0-based).
        index: usize,
        /// How far it is, in the unit of y.
        size: f64,
    },
    /// The measurement slides away from the line, more at every point.
    Drift {
        /// What is added to y at each further point, in the unit of y.
        per_point: f64,
    },
    /// The clock steps back: the x of this point repeats the one before it.
    ClockGlitch {
        /// The point (0-based, 1 or more).
        index: usize,
    },
    /// Points are missing from the file (the file stays valid, with a gap).
    DropPoints {
        /// The first point lost (0-based).
        from: usize,
        /// How many are lost.
        count: usize,
    },
    /// The unit of y is missing from the header.
    UnitLost,
    /// A value is text instead of a number.
    NotANumber {
        /// The point (0-based).
        index: usize,
    },
    /// The file ends in the middle of a line, after this many whole points.
    Truncated {
        /// The whole points kept.
        points: usize,
    },
    /// The instrument wrote nothing.
    Empty,
}

/// A simulated bench with a seeded generator of noise.
#[derive(Debug, Clone)]
pub struct SimBench {
    state: u64,
    noise: f64,
}

impl SimBench {
    /// A bench whose noise is at most `noise` (in the unit of y) on either side of the line; the same `seed` gives the same files.
    pub fn new(seed: u64, noise: f64) -> Self {
        Self {
            state: seed | 1,
            noise: noise.abs(),
        }
    }

    /// The true curve: `y = slope * x` at `points` values of x from 0, `dx` apart.
    pub fn line(x_unit: &str, y_unit: &str, points: usize, dx: f64, slope: f64) -> Curve {
        let x: Vec<f64> = (0..points).map(|i| i as f64 * dx).collect();
        let y = x.iter().map(|v| v * slope).collect();
        Curve {
            x_unit: x_unit.to_string(),
            y_unit: y_unit.to_string(),
            x,
            y,
            u: None,
        }
    }

    /// xorshift64*: small, fixed and the same everywhere. A number in [-1, 1].
    fn next_unit(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        let r = x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11;
        (r as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }

    /// The file the instrument exports for the `truth` curve: noise on every y, the noise as the uncertainty of every point, and then the `faults`.
    pub fn export(&mut self, truth: &Curve, faults: &[Fault]) -> String {
        let mut curve = truth.clone();
        for y in curve.y.iter_mut() {
            *y += self.next_unit() * self.noise;
        }
        curve.u = Some(vec![self.noise; curve.x.len()]);
        for fault in faults {
            apply_to_curve(&mut curve, *fault);
        }
        let mut text = curve.to_csv();
        for fault in faults {
            text = apply_to_text(text, *fault);
        }
        text
    }
}

fn apply_to_curve(curve: &mut Curve, fault: Fault) {
    match fault {
        Fault::Spike { index, size } => {
            if let Some(y) = curve.y.get_mut(index) {
                *y += size;
            }
        }
        Fault::Drift { per_point } => {
            for (i, y) in curve.y.iter_mut().enumerate() {
                *y += per_point * i as f64;
            }
        }
        Fault::ClockGlitch { index } => {
            if (1..curve.x.len()).contains(&index) {
                curve.x[index] = curve.x[index - 1];
            }
        }
        Fault::DropPoints { from, count } => {
            let end = from.saturating_add(count).min(curve.x.len());
            if from < end {
                curve.x.drain(from..end);
                curve.y.drain(from..end);
                if let Some(u) = curve.u.as_mut() {
                    u.drain(from..end);
                }
            }
        }
        Fault::UnitLost | Fault::NotANumber { .. } | Fault::Truncated { .. } | Fault::Empty => {}
    }
}

fn apply_to_text(text: String, fault: Fault) -> String {
    match fault {
        Fault::UnitLost => {
            let (header, rest) = text.split_once('\n').unwrap_or((text.as_str(), ""));
            let cells: Vec<String> = header
                .split(',')
                .map(|c| match c.split_once('[') {
                    Some(("y", _)) => "y".to_string(),
                    _ => c.to_string(),
                })
                .collect();
            format!("{}\n{rest}", cells.join(","))
        }
        Fault::NotANumber { index } => {
            let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
            if let Some(line) = lines.get_mut(index + 1) {
                let mut cells: Vec<String> = line.split(',').map(str::to_string).collect();
                if cells.len() >= 2 {
                    cells[1] = "n/a".to_string();
                    *line = cells.join(",");
                }
            }
            lines.join("\n") + "\n"
        }
        Fault::Truncated { points } => {
            let lines: Vec<&str> = text.lines().collect();
            let keep = (points + 1).min(lines.len());
            let mut out = lines[..keep].join("\n");
            out.push('\n');
            if let Some(next) = lines.get(keep) {
                // the next line is cut after its first cell
                out.push_str(next.split(',').next().unwrap_or(""));
            }
            out
        }
        Fault::Empty => String::new(),
        Fault::Spike { .. }
        | Fault::Drift { .. }
        | Fault::ClockGlitch { .. }
        | Fault::DropPoints { .. } => text,
    }
}
