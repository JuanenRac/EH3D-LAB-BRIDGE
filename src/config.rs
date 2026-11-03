// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.
//! There is no default tolerance: it is set after the bench has been characterised.

use crate::error::LbriError;

/// The settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The most bytes a curve or record file may have.
    pub max_file_bytes: u64,
    /// The most points a curve may have.
    pub max_points: usize,
    /// The tolerance, in the unit of y, inside which a deviation is accepted. None: the comparison is reported but not judged.
    pub tolerance: Option<f64>,
    /// The coverage factor applied to the combined standard uncertainty.
    pub coverage_factor: f64,
    /// The uncertainty of the measured points that have none of their own, in the unit of y. None: unknown.
    pub measurement_uncertainty: Option<f64>,
    /// The uncertainty of the simulated curve, in the unit of y. None: unknown.
    pub simulation_uncertainty: Option<f64>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_file_bytes: 16 * 1024 * 1024,
            max_points: 1_000_000,
            tolerance: None,
            coverage_factor: 2.0,
            measurement_uncertainty: None,
            simulation_uncertainty: None,
        }
    }
}

impl Config {
    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, LbriError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(LbriError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || LbriError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            let non_negative = |v: &str| -> Result<f64, LbriError> {
                let n: f64 = v.parse().map_err(|_| bad())?;
                if n.is_finite() && n >= 0.0 {
                    Ok(n)
                } else {
                    Err(bad())
                }
            };
            match key {
                "max_file_bytes" => config.max_file_bytes = value.parse().map_err(|_| bad())?,
                "max_points" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_points = n;
                }
                "tolerance" => config.tolerance = Some(non_negative(value)?),
                "measurement_uncertainty" => {
                    config.measurement_uncertainty = Some(non_negative(value)?)
                }
                "simulation_uncertainty" => {
                    config.simulation_uncertainty = Some(non_negative(value)?)
                }
                "coverage_factor" => {
                    let n = non_negative(value)?;
                    if n < 1.0 {
                        return Err(bad());
                    }
                    config.coverage_factor = n;
                }
                _ => {
                    return Err(LbriError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_is_no_default_tolerance() {
        let c = Config::parse("").unwrap();
        assert_eq!(c.tolerance, None);
        assert_eq!(c.coverage_factor, 2.0);
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c = Config::parse("tolerance = 0.05\nmeasurement_uncertainty = 0.01\n").unwrap();
        assert_eq!(c.tolerance, Some(0.05));
        assert_eq!(c.measurement_uncertainty, Some(0.01));
        assert!(matches!(
            Config::parse("x\n"),
            Err(LbriError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(LbriError::ConfigUnknownKey { line: 1, .. })
        ));
        for bad in [
            "tolerance = -1\n",
            "tolerance = nan\n",
            "coverage_factor = 0.5\n",
            "max_points = 0\n",
            "simulation_uncertainty = x\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(LbriError::ConfigBadValue { line: 1, .. })
                ),
                "{bad:?}"
            );
        }
    }
}
