use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::fixtures::{crate_root, parse_metadata_fields};

use super::{KnownDifferenceFixtureMismatch, KnownDifferenceLedgerError, KnownDifferenceReference};

const REQUIRED_RETIRED_SEED_FIELDS: [&str; 4] = ["id", "seed_fixture", "curated_fixture", "reason"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetiredSeedDifferenceRecord {
    pub id: String,
    pub fields: BTreeMap<String, String>,
}

impl RetiredSeedDifferenceRecord {
    pub fn seed_fixture(&self) -> PathBuf {
        PathBuf::from(
            self.fields
                .get("seed_fixture")
                .expect("required seed_fixture field was already validated"),
        )
    }
}

pub fn retired_seed_difference_path() -> PathBuf {
    crate_root().join("divergences/retired-seed-differences.txt")
}

pub fn parse_retired_seed_differences(
    text: &str,
) -> Result<Vec<RetiredSeedDifferenceRecord>, KnownDifferenceLedgerError> {
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
        for required in REQUIRED_RETIRED_SEED_FIELDS {
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

        records.push(RetiredSeedDifferenceRecord { id, fields });
    }

    Ok(records)
}

pub(crate) fn collect_retired_seed_fixture_mismatches(
    records: &[RetiredSeedDifferenceRecord],
    fixture_refs_by_id: &BTreeMap<String, KnownDifferenceReference>,
) -> Vec<KnownDifferenceFixtureMismatch> {
    records
        .iter()
        .filter_map(|record| {
            let actual = fixture_refs_by_id.get(&record.id)?;
            let seed_fixture = record.seed_fixture();
            (seed_fixture != actual.fixture).then(|| KnownDifferenceFixtureMismatch {
                id: record.id.clone(),
                ledger_fixture: seed_fixture,
                actual_fixture: actual.fixture.clone(),
            })
        })
        .collect()
}
