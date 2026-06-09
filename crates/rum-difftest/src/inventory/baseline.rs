use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::fixtures::{crate_root, load_text};

use super::{FixtureInventoryReport, FixtureInventoryTotals};

const REQUIRED_FIELDS: [&str; 6] = [
    "lane",
    "source",
    "min_fixtures",
    "min_entries",
    "min_normal_entries",
    "max_known_difference_entries",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBaseline {
    pub records: Vec<InventoryBaselineRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBaselineRecord {
    pub lane: String,
    pub source: String,
    pub min_fixtures: usize,
    pub min_entries: usize,
    pub min_normal_entries: usize,
    pub max_known_difference_entries: usize,
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
    #[error("duplicate inventory baseline record for lane `{lane}`, source `{source_name}`")]
    DuplicateRecord { lane: String, source_name: String },
}

pub fn default_inventory_baseline_path() -> PathBuf {
    crate_root().join("coverage/inventory-baseline.txt")
}

pub fn load_inventory_baseline(
    path: impl AsRef<Path>,
) -> Result<InventoryBaseline, InventoryBaselineError> {
    let path = path.as_ref();
    let text = load_text(path).map_err(|source| InventoryBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_inventory_baseline(&text)
}

pub fn write_inventory_baseline(
    path: impl AsRef<Path>,
    baseline: &InventoryBaseline,
) -> Result<(), InventoryBaselineError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| InventoryBaselineError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    fs::write(path, format_inventory_baseline(baseline)).map_err(|source| {
        InventoryBaselineError::Io {
            path: path.to_path_buf(),
            source,
        }
    })
}

pub fn parse_inventory_baseline(text: &str) -> Result<InventoryBaseline, InventoryBaselineError> {
    let mut records = Vec::new();
    let mut seen = BTreeMap::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = parse_fields(line)?;
        for required in REQUIRED_FIELDS {
            if !fields.contains_key(required) {
                return Err(InventoryBaselineError::MissingField {
                    line: line.to_string(),
                    field: required.to_string(),
                });
            }
        }

        let record = InventoryBaselineRecord {
            lane: fields["lane"].clone(),
            source: fields["source"].clone(),
            min_fixtures: parse_usize(line, "min_fixtures", &fields["min_fixtures"])?,
            min_entries: parse_usize(line, "min_entries", &fields["min_entries"])?,
            min_normal_entries: parse_usize(
                line,
                "min_normal_entries",
                &fields["min_normal_entries"],
            )?,
            max_known_difference_entries: parse_usize(
                line,
                "max_known_difference_entries",
                &fields["max_known_difference_entries"],
            )?,
        };
        let key = (record.lane.clone(), record.source.clone());
        if seen.insert(key.clone(), ()).is_some() {
            return Err(InventoryBaselineError::DuplicateRecord {
                lane: key.0,
                source_name: key.1,
            });
        }
        records.push(record);
    }

    Ok(InventoryBaseline { records })
}

pub fn capture_inventory_baseline(inventory: &FixtureInventoryReport) -> InventoryBaseline {
    let mut records = Vec::new();

    for lane in &inventory.lanes {
        push_record_from_totals(&mut records, &lane.lane, "all", &lane.totals());
        for (source, totals) in lane.totals_by_source() {
            push_record_from_totals(&mut records, &lane.lane, source, &totals);
        }
    }

    InventoryBaseline { records }
}

pub fn format_inventory_baseline(baseline: &InventoryBaseline) -> String {
    let mut output = String::from(
        "# Stable non-oracle fixture coverage baseline.\n\
         # `min_*` fields are lower bounds. `max_known_difference_entries` is an upper\n\
         # bound so resolved known differences can decrease without failing the gate.\n",
    );

    for record in &baseline.records {
        output.push_str(&format_inventory_baseline_record(record));
        output.push('\n');
    }

    output
}

pub fn compare_inventory_baseline(
    inventory: &FixtureInventoryReport,
    baseline: &InventoryBaseline,
) -> InventoryBaselineReport {
    let actual = actual_totals_by_lane_source(inventory);
    let mut regressions = Vec::new();

    for record in &baseline.records {
        let key = (record.lane.clone(), record.source.clone());
        let actual_totals = actual.get(&key).cloned();
        let mut issues = Vec::new();
        match &actual_totals {
            Some(totals) => collect_regression_issues(record, totals, &mut issues),
            None => issues.push("missing lane/source in current inventory".to_string()),
        }
        if !issues.is_empty() {
            regressions.push(InventoryBaselineRegression {
                record: record.clone(),
                actual: actual_totals,
                issues,
            });
        }
    }

    InventoryBaselineReport {
        records_considered: baseline.records.len(),
        regressions,
    }
}

fn push_record_from_totals(
    records: &mut Vec<InventoryBaselineRecord>,
    lane: &str,
    source: &str,
    totals: &FixtureInventoryTotals,
) {
    records.push(InventoryBaselineRecord {
        lane: lane.to_string(),
        source: source.to_string(),
        min_fixtures: totals.fixtures,
        min_entries: totals.entries,
        min_normal_entries: totals.normal_entries,
        max_known_difference_entries: totals.known_difference_entries,
    });
}

fn format_inventory_baseline_record(record: &InventoryBaselineRecord) -> String {
    format!(
        "lane={} source={} min_fixtures={} min_entries={} min_normal_entries={} max_known_difference_entries={}",
        record.lane,
        record.source,
        record.min_fixtures,
        record.min_entries,
        record.min_normal_entries,
        record.max_known_difference_entries
    )
}

fn actual_totals_by_lane_source(
    inventory: &FixtureInventoryReport,
) -> BTreeMap<(String, String), FixtureInventoryTotals> {
    let mut totals = BTreeMap::new();
    for lane in &inventory.lanes {
        totals.insert((lane.lane.clone(), "all".to_string()), lane.totals());
        for (source, source_totals) in lane.totals_by_source() {
            totals.insert((lane.lane.clone(), source.to_string()), source_totals);
        }
    }
    totals
}

fn collect_regression_issues(
    record: &InventoryBaselineRecord,
    actual: &FixtureInventoryTotals,
    issues: &mut Vec<String>,
) {
    push_min_issue("fixtures", actual.fixtures, record.min_fixtures, issues);
    push_min_issue("entries", actual.entries, record.min_entries, issues);
    push_min_issue(
        "normal_entries",
        actual.normal_entries,
        record.min_normal_entries,
        issues,
    );
    if actual.known_difference_entries > record.max_known_difference_entries {
        issues.push(format!(
            "known_difference_entries {} > max {}",
            actual.known_difference_entries, record.max_known_difference_entries
        ));
    }
}

fn push_min_issue(field: &str, actual: usize, min: usize, issues: &mut Vec<String>) {
    if actual < min {
        issues.push(format!("{field} {actual} < min {min}"));
    }
}

fn parse_fields(line: &str) -> Result<BTreeMap<String, String>, InventoryBaselineError> {
    let mut fields = BTreeMap::new();
    for part in line.split_whitespace() {
        let Some((key, value)) = part.split_once('=') else {
            return Err(InventoryBaselineError::InvalidLine {
                line: line.to_string(),
                message: format!("field `{part}` is not key=value"),
            });
        };
        fields.insert(key.to_string(), value.to_string());
    }
    Ok(fields)
}

fn parse_usize(line: &str, field: &str, value: &str) -> Result<usize, InventoryBaselineError> {
    value
        .parse::<usize>()
        .map_err(|_| InventoryBaselineError::InvalidInteger {
            line: line.to_string(),
            field: field.to_string(),
            value: value.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::collect_fixture_inventory;

    #[test]
    fn default_inventory_baseline_is_clean_for_current_fixtures() {
        let inventory = collect_fixture_inventory().unwrap();
        let baseline = load_inventory_baseline(default_inventory_baseline_path()).unwrap();
        let report = compare_inventory_baseline(&inventory, &baseline);

        assert!(report.is_clean(), "{report:?}");
        assert_eq!(report.records_considered, 5);
    }

    #[test]
    fn inventory_baseline_reports_regressions() {
        let inventory = collect_fixture_inventory().unwrap();
        let baseline = parse_inventory_baseline(
            "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
        )
        .unwrap();
        let report = compare_inventory_baseline(&inventory, &baseline);

        assert!(!report.is_clean());
        assert_eq!(report.regressions.len(), 1);
        assert!(report.regressions[0].issues[0].contains("fixtures"));
    }

    #[test]
    fn captured_inventory_baseline_round_trips_and_is_clean() {
        let inventory = collect_fixture_inventory().unwrap();
        let baseline = capture_inventory_baseline(&inventory);
        let formatted = format_inventory_baseline(&baseline);
        let reparsed = parse_inventory_baseline(&formatted).unwrap();
        let report = compare_inventory_baseline(&inventory, &reparsed);

        assert_eq!(reparsed, baseline);
        assert_eq!(baseline.records.len(), 5);
        assert!(formatted.contains("lane=lane-a source=all min_fixtures=24"));
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn inventory_baseline_rejects_duplicate_records() {
        let err = parse_inventory_baseline(
            "lane=lane-a source=all min_fixtures=1 min_entries=1 min_normal_entries=1 max_known_difference_entries=1\n\
             lane=lane-a source=all min_fixtures=1 min_entries=1 min_normal_entries=1 max_known_difference_entries=1\n",
        )
        .unwrap_err();

        assert!(matches!(
            err,
            InventoryBaselineError::DuplicateRecord { .. }
        ));
    }
}
