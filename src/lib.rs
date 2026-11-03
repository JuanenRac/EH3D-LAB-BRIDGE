// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-LAB-BRIDGE: measured curves, test records and the comparison with a simulated curve.
//!
//! A measured curve is read from one defined file, a CSV whose header names the unit of every column (`x[s],y[V]` or `x[s],y[V],u[V]`). A test record keeps the
//! instrument, the conditions and the SHA-256 of the data. The comparison reports the deviation at every measured point together with its uncertainty and judges
//! it only against a tolerance that the person sets: no tolerance number is invented here, and without one the verdict is "not judged".
//!
//! There is no live control of instruments. Nothing here has been used with a real instrument or bench.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod compare;
pub mod config;
pub mod curve;
pub mod error;
pub mod record;

pub use curve::Curve;
pub use error::LbriError;

/// The stable code of a finding. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// A curve file that does not follow the defined format.
    BadCurve,
    /// A test record without an instrument, a condition or a quantity.
    IncompleteRecord,
    /// The data file of a record has other content than the hash in the record.
    HashMismatch,
    /// The two curves of a comparison do not use the same units.
    UnitMismatch,
}

impl Code {
    /// The text of the code, such as `PHOENIX-LBRI-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::BadCurve => "PHOENIX-LBRI-001",
            Code::IncompleteRecord => "PHOENIX-LBRI-002",
            Code::HashMismatch => "PHOENIX-LBRI-003",
            Code::UnitMismatch => "PHOENIX-LBRI-004",
        }
    }
}
