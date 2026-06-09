use std::path::PathBuf;

use thiserror::Error;

use super::FixtureInventoryTotals;

#[path = "baseline/capture.rs"]
mod capture;
#[path = "baseline/compare.rs"]
mod compare;
#[path = "baseline/io.rs"]
mod io;
#[path = "baseline/parse.rs"]
mod parse;

pub use capture::capture_inventory_baseline;
pub use compare::compare_inventory_baseline;
pub use io::{default_inventory_baseline_path, load_inventory_baseline, write_inventory_baseline};
pub use parse::{format_inventory_baseline, parse_inventory_baseline};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBaseline {
    pub records: Vec<InventoryBaselineRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBaselineRecord {
    pub lane: String,
    pub dimension: InventoryBaselineDimension,
    pub name: String,
    pub min_fixtures: usize,
    pub min_entries: usize,
    pub min_normal_entries: usize,
    pub max_known_difference_entries: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InventoryBaselineDimension {
    Source,
    Category,
}

impl InventoryBaselineDimension {
    pub fn field_name(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Category => "category",
        }
    }

    fn missing_message_name(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Category => "category",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBaselineReport {
    pub records_considered: usize,
    pub regressions: Vec<InventoryBaselineRegression>,
}

impl InventoryBaselineReport {
    pub fn is_clean(&self) -> bool {
        self.regressions.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBaselineRegression {
    pub record: InventoryBaselineRecord,
    pub actual: Option<FixtureInventoryTotals>,
    pub issues: Vec<String>,
}

#[derive(Debug, Error)]
pub enum InventoryBaselineError {
    #[error("failed to read `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid inventory baseline line `{line}`: {message}")]
    InvalidLine { line: String, message: String },
    #[error("inventory baseline line is missing `{field}`: {line}")]
    MissingField { line: String, field: String },
    #[error("invalid `{field}` value `{value}` in inventory baseline line: {line}")]
    InvalidInteger {
        line: String,
        field: String,
        value: String,
    },
    #[error("inventory baseline line must include exactly one of `source` or `category`: {line}")]
    InvalidDimension { line: String },
    #[error(
        "duplicate inventory baseline record for lane `{lane}`, {dimension} `{dimension_name}`"
    )]
    DuplicateRecord {
        lane: String,
        dimension: String,
        dimension_name: String,
    },
}

#[cfg(test)]
#[path = "baseline/tests.rs"]
mod tests;
