use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::fixtures::{crate_root, parse_metadata_fields};

use super::{KnownDifferenceLedgerError, KnownDifferenceRecord};

const REQUIRED_LEDGER_FIELDS: [&str; 7] = [
    "id",
    "type",
    "status",
    "fixture",
    "reason",
    "tracked_by",
    "close_when",
];

pub fn known_difference_ledger_path() -> PathBuf {
    crate_root().join("divergences/known-differences.txt")
}

pub fn parse_known_difference_ledger(
    text: &str,
) -> Result<Vec<KnownDifferenceRecord>, KnownDifferenceLedgerError> {
    let mut records = Vec::new();
    let mut seen_ids = BTreeMap::<String, ()>::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields = parse_metadata_fields(line, line).map_err(|source| {
            KnownDifferenceLedgerError::InvalidLedgerLine {
                line: line.to_owned(),
                message: source.to_string(),
            }
        })?;
        for required in REQUIRED_LEDGER_FIELDS {
            if !fields.contains_key(required) {
                return Err(KnownDifferenceLedgerError::MissingLedgerField {
                    line: line.to_owned(),
                    field: required.to_owned(),
                });
            }
        }

        let id = fields
            .get("id")
            .expect("required id field was already validated")
            .clone();
        if seen_ids.insert(id.clone(), ()).is_some() {
            return Err(KnownDifferenceLedgerError::DuplicateLedgerId { id });
        }

        records.push(KnownDifferenceRecord { id, fields });
    }

    Ok(records)
}
