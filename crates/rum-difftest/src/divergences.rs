use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

use crate::fixtures::{
    FixtureParseError, GoldenEntry, crate_root, golden_dir, load_text, parse_golden_file,
    parse_metadata_fields,
};

mod retired;
pub use retired::{
    RetiredSeedDifferenceRecord, parse_retired_seed_differences, retired_seed_difference_path,
};

const REQUIRED_LEDGER_FIELDS: [&str; 7] = [
    "id",
    "type",
    "status",
    "fixture",
    "reason",
    "tracked_by",
    "close_when",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceRecord {
    pub id: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceReference {
    pub id: String,
    pub fixture: PathBuf,
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
    pub retired_seed_fixture_mismatches: Vec<KnownDifferenceFixtureMismatch>,
    pub tracking_target_issues: Vec<KnownDifferenceTrackingTargetIssue>,
}

impl KnownDifferenceLedgerReport {
    pub fn is_clean(&self) -> bool {
        self.missing_ledger_entries.is_empty()
            && self.unreferenced_ledger_entries.is_empty()
            && self.unreferenced_retired_seed_entries.is_empty()
            && self.fixture_mismatches.is_empty()
            && self.retired_seed_fixture_mismatches.is_empty()
            && self.tracking_target_issues.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownDifferenceFixtureMismatch {
    pub id: String,
    pub ledger_fixture: PathBuf,
    pub actual_fixture: PathBuf,
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

pub fn known_difference_ledger_path() -> PathBuf {
    crate_root().join("divergences/known-differences.txt")
}

pub fn validate_known_difference_ledger()
-> Result<KnownDifferenceLedgerReport, KnownDifferenceLedgerError> {
    let ledger_path = known_difference_ledger_path();
    let text = load_text(&ledger_path).map_err(|source| KnownDifferenceLedgerError::Io {
        path: ledger_path.clone(),
        source,
    })?;
    let records = parse_known_difference_ledger(&text)?;
    let retired_seed_path = retired_seed_difference_path();
    let retired_seed_text =
        load_text(&retired_seed_path).map_err(|source| KnownDifferenceLedgerError::Io {
            path: retired_seed_path.clone(),
            source,
        })?;
    let retired_seed_records = parse_retired_seed_differences(&retired_seed_text)?;
    let fixture_references = collect_fixture_known_differences()?;

    let records_by_id = records
        .iter()
        .map(|record| (record.id.clone(), record.clone()))
        .collect::<BTreeMap<_, _>>();
    let retired_seed_records_by_id = retired_seed_records
        .iter()
        .map(|record| (record.id.clone(), record.clone()))
        .collect::<BTreeMap<_, _>>();
    let fixture_refs_by_id = fixture_references
        .iter()
        .map(|reference| (reference.id.clone(), reference.clone()))
        .collect::<BTreeMap<_, _>>();

    let missing_ledger_entries = fixture_references
        .iter()
        .filter(|reference| {
            !records_by_id.contains_key(&reference.id)
                && !retired_seed_records_by_id.contains_key(&reference.id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let unreferenced_ledger_entries = records
        .iter()
        .filter(|record| !fixture_refs_by_id.contains_key(&record.id))
        .cloned()
        .collect::<Vec<_>>();
    let unreferenced_retired_seed_entries = retired_seed_records
        .iter()
        .filter(|record| !fixture_refs_by_id.contains_key(&record.id))
        .cloned()
        .collect::<Vec<_>>();
    let fixture_mismatches = collect_fixture_mismatches(&records, &fixture_refs_by_id);
    let retired_seed_fixture_mismatches = retired::collect_retired_seed_fixture_mismatches(
        &retired_seed_records,
        &fixture_refs_by_id,
    );
    let tracking_target_issues = collect_tracking_target_issues(&records);

    Ok(KnownDifferenceLedgerReport {
        records,
        retired_seed_records,
        fixture_references,
        missing_ledger_entries,
        unreferenced_ledger_entries,
        unreferenced_retired_seed_entries,
        fixture_mismatches,
        retired_seed_fixture_mismatches,
        tracking_target_issues,
    })
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

fn collect_fixture_known_differences()
-> Result<Vec<KnownDifferenceReference>, KnownDifferenceLedgerError> {
    let mut paths = Vec::new();
    collect_txt_files(&golden_dir(), &mut paths).map_err(|source| {
        KnownDifferenceLedgerError::Io {
            path: golden_dir(),
            source,
        }
    })?;
    collect_txt_files(&crate_root().join("cases"), &mut paths).map_err(|source| {
        KnownDifferenceLedgerError::Io {
            path: crate_root().join("cases"),
            source,
        }
    })?;
    paths.sort();

    let mut references = Vec::new();
    let mut seen_ids = BTreeMap::<String, PathBuf>::new();
    for path in paths {
        let text = load_text(&path).map_err(|source| KnownDifferenceLedgerError::Io {
            path: path.clone(),
            source,
        })?;
        let entries = parse_golden_file(&text).map_err(|source| {
            KnownDifferenceLedgerError::FixtureParse {
                path: path.clone(),
                source,
            }
        })?;
        for entry in entries {
            let GoldenEntry::KnownDifference { metadata, .. } = entry else {
                continue;
            };
            let id = metadata
                .get("id")
                .expect("fixture parser validates known-difference ids")
                .clone();
            let fixture = relative_to_crate_root(&path);
            if let Some(first_fixture) = seen_ids.insert(id.clone(), fixture.clone()) {
                return Err(KnownDifferenceLedgerError::DuplicateFixtureId {
                    id,
                    first_fixture,
                    second_fixture: fixture,
                });
            }
            references.push(KnownDifferenceReference { id, fixture });
        }
    }

    Ok(references)
}

fn collect_fixture_mismatches(
    records: &[KnownDifferenceRecord],
    fixture_refs_by_id: &BTreeMap<String, KnownDifferenceReference>,
) -> Vec<KnownDifferenceFixtureMismatch> {
    records
        .iter()
        .filter_map(|record| {
            let actual = fixture_refs_by_id.get(&record.id)?;
            let ledger_fixture = PathBuf::from(
                record
                    .fields
                    .get("fixture")
                    .expect("required fixture field was already validated"),
            );
            (ledger_fixture != actual.fixture).then(|| KnownDifferenceFixtureMismatch {
                id: record.id.clone(),
                ledger_fixture,
                actual_fixture: actual.fixture.clone(),
            })
        })
        .collect()
}

fn collect_tracking_target_issues(
    records: &[KnownDifferenceRecord],
) -> Vec<KnownDifferenceTrackingTargetIssue> {
    records
        .iter()
        .filter_map(|record| {
            let target = record
                .fields
                .get("tracked_by")
                .expect("required tracked_by field was already validated");
            validate_tracking_target(target).err().map(|message| {
                KnownDifferenceTrackingTargetIssue {
                    id: record.id.clone(),
                    target: target.clone(),
                    message,
                }
            })
        })
        .collect()
}

fn validate_tracking_target(target: &str) -> Result<(), String> {
    let Some((path_text, needle)) = target.split_once(':') else {
        return Err("expected `path:needle`".to_string());
    };
    if needle.trim().is_empty() {
        return Err("tracking target needle is empty".to_string());
    }

    let relative_path = Path::new(path_text.trim());
    if relative_path.as_os_str().is_empty() {
        return Err("tracking target path is empty".to_string());
    }
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("tracking target path must stay inside the repository".to_string());
    }

    let Some(path) = resolve_tracking_target_path(relative_path) else {
        return Err(format!(
            "tracking target file `{}` does not exist",
            relative_path.display()
        ));
    };
    let text = fs::read_to_string(&path)
        .map_err(|source| format!("failed to read `{}`: {source}", path.display()))?;
    if !text.contains(needle.trim()) {
        return Err(format!(
            "tracking target `{}` does not contain `{}`",
            path.display(),
            needle.trim()
        ));
    }

    Ok(())
}

fn resolve_tracking_target_path(relative_path: &Path) -> Option<PathBuf> {
    let repo_path = repo_root().join(relative_path);
    if repo_path.is_file() {
        return Some(repo_path);
    }

    let crate_path = crate_root().join(relative_path);
    crate_path.is_file().then_some(crate_path)
}

fn repo_root() -> PathBuf {
    let crate_path = crate_root();
    crate_path
        .parent()
        .and_then(Path::parent)
        .unwrap_or(crate_path.as_path())
        .to_path_buf()
}

fn collect_txt_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_txt_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("txt") {
            out.push(path);
        }
    }
    Ok(())
}

fn relative_to_crate_root(path: &Path) -> PathBuf {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .to_path_buf()
}

#[cfg(test)]
mod tests;
