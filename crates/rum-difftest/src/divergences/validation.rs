use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::fixtures::load_text;

use super::fixtures::collect_fixture_known_differences;
use super::ledger::{known_difference_ledger_path, parse_known_difference_ledger};
use super::retired::{self, parse_retired_seed_differences, retired_seed_difference_path};
use super::tracking::collect_tracking_target_issues;
use super::{
    KnownDifferenceFixtureMismatch, KnownDifferenceLedgerError, KnownDifferenceLedgerReport,
    KnownDifferenceMetadataMismatch, KnownDifferenceRecord, KnownDifferenceReference,
};

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
    let metadata_mismatches = collect_metadata_mismatches(&records, &fixture_refs_by_id);
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
        metadata_mismatches,
        retired_seed_fixture_mismatches,
        tracking_target_issues,
    })
}

pub(super) fn collect_fixture_mismatches(
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

pub(super) fn collect_metadata_mismatches(
    records: &[KnownDifferenceRecord],
    fixture_refs_by_id: &BTreeMap<String, KnownDifferenceReference>,
) -> Vec<KnownDifferenceMetadataMismatch> {
    records
        .iter()
        .filter_map(|record| {
            let fixture_ref = fixture_refs_by_id.get(&record.id)?;
            let field = "type";
            let ledger_value = record
                .fields
                .get(field)
                .expect("required type field was already validated");
            let fixture_value = fixture_ref
                .metadata
                .get(field)
                .expect("fixture parser validates known-difference type");
            (ledger_value != fixture_value).then(|| KnownDifferenceMetadataMismatch {
                id: record.id.clone(),
                field: field.to_string(),
                ledger_value: ledger_value.clone(),
                fixture_value: fixture_value.clone(),
            })
        })
        .collect()
}
