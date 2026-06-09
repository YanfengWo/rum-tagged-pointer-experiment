use std::collections::BTreeMap;
use std::path::PathBuf;

use thiserror::Error;

use crate::fixtures::FixtureParseError;

mod fixtures;
mod ledger;
mod retired;
mod tracking;
mod validation;

pub use ledger::{known_difference_ledger_path, parse_known_difference_ledger};
pub use retired::{
    RetiredSeedDifferenceRecord, parse_retired_seed_differences, retired_seed_difference_path,
};
pub use validation::validate_known_difference_ledger;

#[cfg(test)]
use tracking::collect_tracking_target_issues;
#[cfg(test)]
use validation::{collect_fixture_mismatches, collect_metadata_mismatches};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceRecord {
    pub id: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceReference {
    pub id: String,
    pub fixture: PathBuf,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceLedgerReport {
    pub records: Vec<KnownDifferenceRecord>,
    pub retired_seed_records: Vec<RetiredSeedDifferenceRecord>,
    pub fixture_references: Vec<KnownDifferenceReference>,
    pub missing_ledger_entries: Vec<KnownDifferenceReference>,
    pub unreferenced_ledger_entries: Vec<KnownDifferenceRecord>,
    pub unreferenced_retired_seed_entries: Vec<RetiredSeedDifferenceRecord>,
    pub fixture_mismatches: Vec<KnownDifferenceFixtureMismatch>,
    pub metadata_mismatches: Vec<KnownDifferenceMetadataMismatch>,
    pub retired_seed_fixture_mismatches: Vec<KnownDifferenceFixtureMismatch>,
    pub tracking_target_issues: Vec<KnownDifferenceTrackingTargetIssue>,
}

impl KnownDifferenceLedgerReport {
    pub fn is_clean(&self) -> bool {
        self.missing_ledger_entries.is_empty()
            && self.unreferenced_ledger_entries.is_empty()
            && self.unreferenced_retired_seed_entries.is_empty()
            && self.fixture_mismatches.is_empty()
            && self.metadata_mismatches.is_empty()
            && self.retired_seed_fixture_mismatches.is_empty()
            && self.tracking_target_issues.is_empty()
    }

    pub fn active_ledger_field_counts(&self, field: &str) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for record in &self.records {
            if let Some(value) = record.fields.get(field) {
                *counts.entry(value.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    pub fn active_ledger_status_count(&self, status: &str) -> usize {
        self.active_ledger_field_counts("status")
            .get(status)
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceFixtureMismatch {
    pub id: String,
    pub ledger_fixture: PathBuf,
    pub actual_fixture: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceMetadataMismatch {
    pub id: String,
    pub field: String,
    pub ledger_value: String,
    pub fixture_value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceTrackingTargetIssue {
    pub id: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum KnownDifferenceLedgerError {
    #[error("failed to read `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse fixture `{path}`: {source}")]
    FixtureParse {
        path: PathBuf,
        #[source]
        source: FixtureParseError,
    },
    #[error("invalid known-difference ledger line `{line}`: {message}")]
    InvalidLedgerLine { line: String, message: String },
    #[error("known-difference ledger line is missing `{field}`: {line}")]
    MissingLedgerField { line: String, field: String },
    #[error("known-difference ledger id `{id}` is duplicated")]
    DuplicateLedgerId { id: String },
    #[error(
        "fixture known-difference id `{id}` is duplicated in `{first_fixture}` and `{second_fixture}`"
    )]
    DuplicateFixtureId {
        id: String,
        first_fixture: PathBuf,
        second_fixture: PathBuf,
    },
}

#[cfg(test)]
mod tests;
