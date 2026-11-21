// SPDX-License-Identifier: GPL-3.0-or-later
//! The test record: which instrument, under which conditions, with which data. It is stored as JSON so a test can be repeated from it.

use crate::error::LbriError;
use crate::Code;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The configuration and the data of one test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestRecord {
    /// The instrument, with its model and serial number.
    pub instrument: String,
    /// The conditions of the test (temperature, supply, probe, range...), one per entry.
    #[serde(default)]
    pub conditions: Vec<String>,
    /// What was measured, such as `output voltage`.
    pub quantity: String,
    /// The name of the data file, as written by the person.
    pub data_file: String,
    /// The SHA-256 of the data file, in lower-case hexadecimal.
    pub data_sha256: String,
    /// A free note.
    #[serde(default)]
    pub note: String,
}

/// Something wrong in a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The code.
    pub code: Code,
    /// What it is about.
    pub subject: &'static str,
    /// What is wrong, in English.
    pub message: String,
}

/// The SHA-256 of some bytes as lower-case hexadecimal.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl TestRecord {
    /// Read a record from JSON (unknown fields are refused).
    pub fn from_json(text: &str) -> Result<TestRecord, LbriError> {
        serde_json::from_str(text).map_err(|e| LbriError::BadJson(e.to_string()))
    }

    /// The record as pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// What keeps this record from being accepted: no instrument, no conditions, no quantity, no data or a hash that is not 64 hexadecimal digits. When the
    /// bytes of the data file are given, a hash that does not match them is reported too.
    pub fn check(&self, data: Option<&[u8]>) -> Vec<Finding> {
        let mut out = Vec::new();
        let mut add = |code: Code, subject: &'static str, message: &str| {
            out.push(Finding {
                code,
                subject,
                message: message.to_string(),
            })
        };
        if self.instrument.trim().is_empty() {
            add(
                Code::IncompleteRecord,
                "instrument",
                "the record names no instrument",
            );
        }
        if self.conditions.is_empty() || self.conditions.iter().any(|c| c.trim().is_empty()) {
            add(
                Code::IncompleteRecord,
                "conditions",
                "the conditions are missing or one is empty",
            );
        }
        if self.quantity.trim().is_empty() {
            add(
                Code::IncompleteRecord,
                "quantity",
                "the record does not say what was measured",
            );
        }
        if self.data_file.trim().is_empty() {
            add(
                Code::IncompleteRecord,
                "data_file",
                "the record names no data file",
            );
        }
        let hex = self.data_sha256.len() == 64
            && self
                .data_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !hex {
            add(
                Code::IncompleteRecord,
                "data_sha256",
                "the hash is not 64 lower-case hexadecimal digits",
            );
        } else if let Some(bytes) = data {
            if sha256_hex(bytes) != self.data_sha256 {
                add(
                    Code::HashMismatch,
                    "data_sha256",
                    "the data file has other content than the record was made from",
                );
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(data: &[u8]) -> TestRecord {
        TestRecord {
            instrument: "Scope X, serial 123".into(),
            conditions: vec!["supply 5.0 V".into(), "probe x10".into()],
            quantity: "output voltage".into(),
            data_file: "run1.csv".into(),
            data_sha256: sha256_hex(data),
            note: String::new(),
        }
    }

    #[test]
    fn the_hash_is_the_known_sha256() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn a_complete_record_passes_and_repeats_from_its_json() {
        let r = record(b"x");
        assert!(r.check(Some(b"x")).is_empty());
        assert_eq!(TestRecord::from_json(&r.to_json()).unwrap(), r);
    }

    #[test]
    fn a_record_without_an_instrument_is_rejected() {
        let mut r = record(b"x");
        r.instrument = "  ".into();
        let f = r.check(None);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code.as_str(), "PHOENIX-LBRI-002");
        assert_eq!(f[0].subject, "instrument");
    }

    #[test]
    fn every_missing_part_is_reported() {
        let mut r = record(b"x");
        r.conditions.clear();
        r.quantity.clear();
        r.data_file.clear();
        r.data_sha256 = "ABC".into();
        let subjects: Vec<&str> = r.check(None).iter().map(|f| f.subject).collect();
        assert_eq!(
            subjects,
            vec!["conditions", "quantity", "data_file", "data_sha256"]
        );
    }

    #[test]
    fn changed_data_gives_a_hash_mismatch() {
        let r = record(b"x");
        let f = r.check(Some(b"y"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code.as_str(), "PHOENIX-LBRI-003");
    }

    #[test]
    fn unknown_fields_are_refused() {
        let r = record(b"x");
        let bad = r.to_json().replace("\"note\"", "\"notes\"");
        assert!(matches!(
            TestRecord::from_json(&bad),
            Err(LbriError::BadJson(_))
        ));
    }

    #[test]
    fn the_hash_of_nothing_and_of_a_large_block_are_the_known_sha256() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let block = vec![b'a'; 1_000_000];
        assert_eq!(
            sha256_hex(&block),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn a_text_that_is_not_a_record_is_an_error_and_never_a_panic() {
        for text in ["", " ", "null", "[]", "12", "{", "{}", "\u{feff}{}"] {
            assert!(
                matches!(TestRecord::from_json(text), Err(LbriError::BadJson(_))),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_field_the_record_does_not_know_is_refused() {
        let text = record(b"x").to_json().replace("\"note\"", "\"comment\"");
        assert!(matches!(
            TestRecord::from_json(&text),
            Err(LbriError::BadJson(_))
        ));
    }

    #[test]
    fn a_hash_in_upper_case_or_of_the_wrong_length_is_incomplete_and_one_condition_in_blanks_too() {
        let mut upper = record(b"x");
        upper.data_sha256 = upper.data_sha256.to_uppercase();
        let mut short = record(b"x");
        short.data_sha256.pop();
        let mut blank = record(b"x");
        blank.conditions.push("   ".into());
        for (r, subject) in [
            (upper, "data_sha256"),
            (short, "data_sha256"),
            (blank, "conditions"),
        ] {
            let found = r.check(Some(b"x"));
            assert_eq!(found.len(), 1, "{subject}");
            assert_eq!(found[0].subject, subject);
            assert_eq!(found[0].code, Code::IncompleteRecord);
        }
    }

    #[test]
    fn data_with_other_content_is_a_mismatch_and_without_the_data_nothing_is_compared() {
        let r = record(b"first");
        let changed = r.check(Some(b"second"));
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].code, Code::HashMismatch);
        assert!(r.check(None).is_empty());
        assert!(r.check(Some(b"first")).is_empty());
    }

    #[test]
    fn an_empty_record_names_every_missing_part_in_a_fixed_order() {
        let r = TestRecord {
            instrument: String::new(),
            conditions: vec![],
            quantity: String::new(),
            data_file: String::new(),
            data_sha256: String::new(),
            note: String::new(),
        };
        let subjects: Vec<&str> = r.check(None).iter().map(|f| f.subject).collect();
        assert_eq!(
            subjects,
            [
                "instrument",
                "conditions",
                "quantity",
                "data_file",
                "data_sha256"
            ]
        );
    }
}
