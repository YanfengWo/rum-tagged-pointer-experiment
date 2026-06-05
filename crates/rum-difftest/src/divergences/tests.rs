use std::collections::BTreeMap;
use std::path::PathBuf;

use super::*;

#[test]
fn parses_known_difference_ledger_records() {
    let records = parse_known_difference_ledger(
        r#"
        # comment
        id=case-a | type=temporary | status=open | fixture=cases/a.txt | reason=example | tracked_by=TODO.md:case-a | close_when=example is fixed
        "#,
    )
    .unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, "case-a");
    assert_eq!(records[0].fields["fixture"], "cases/a.txt");
    assert_eq!(records[0].fields["tracked_by"], "TODO.md:case-a");
}

#[test]
fn ledger_records_require_tracking_metadata() {
    let err = parse_known_difference_ledger(
        "id=case-a | type=temporary | status=open | fixture=cases/a.txt | reason=example",
    )
    .unwrap_err();

    assert!(matches!(
        err,
        KnownDifferenceLedgerError::MissingLedgerField { field, .. }
            if field == "tracked_by"
    ));
}

#[test]
fn known_difference_ledger_covers_checked_fixture_ids() {
    let report = validate_known_difference_ledger().unwrap();

    assert!(
        !report.fixture_references.is_empty(),
        "expected at least one fixture known-difference"
    );
    assert!(
        report.is_clean(),
        "known-difference ledger is out of sync\nmissing: {:?}\nunreferenced: {:?}\nunreferenced retired: {:?}",
        report.missing_ledger_entries,
        report.unreferenced_ledger_entries,
        report.unreferenced_retired_seed_entries
    );
}

#[test]
fn parses_retired_seed_difference_records() {
    let records = parse_retired_seed_differences(
        r#"
        # comment
        id=case-a | seed_fixture=seed/ham-import/golden/a.txt | curated_fixture=cases/wolfram/core/a.txt | reason=curated
        "#,
    )
    .unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, "case-a");
    assert_eq!(
        records[0].fields["curated_fixture"],
        "cases/wolfram/core/a.txt"
    );
}

#[test]
fn ledger_validation_reports_fixture_metadata_mismatches() {
    let mut fields = BTreeMap::new();
    fields.insert("fixture".to_string(), "cases/old.txt".to_string());
    let records = vec![KnownDifferenceRecord {
        id: "case-a".to_string(),
        fields,
    }];
    let fixture_refs_by_id = BTreeMap::from([(
        "case-a".to_string(),
        KnownDifferenceReference {
            id: "case-a".to_string(),
            fixture: PathBuf::from("cases/new.txt"),
        },
    )]);

    let mismatches = collect_fixture_mismatches(&records, &fixture_refs_by_id);

    assert_eq!(
        mismatches,
        vec![KnownDifferenceFixtureMismatch {
            id: "case-a".to_string(),
            ledger_fixture: PathBuf::from("cases/old.txt"),
            actual_fixture: PathBuf::from("cases/new.txt"),
        }]
    );
}

#[test]
fn retired_seed_validation_reports_fixture_metadata_mismatches() {
    let retired_records = vec![RetiredSeedDifferenceRecord {
        id: "case-a".to_string(),
        fields: BTreeMap::from([(
            "seed_fixture".to_string(),
            "seed/ham-import/golden/old.txt".to_string(),
        )]),
    }];
    let fixture_refs_by_id = BTreeMap::from([(
        "case-a".to_string(),
        KnownDifferenceReference {
            id: "case-a".to_string(),
            fixture: PathBuf::from("seed/ham-import/golden/new.txt"),
        },
    )]);

    let mismatches =
        retired::collect_retired_seed_fixture_mismatches(&retired_records, &fixture_refs_by_id);

    assert_eq!(
        mismatches,
        vec![KnownDifferenceFixtureMismatch {
            id: "case-a".to_string(),
            ledger_fixture: PathBuf::from("seed/ham-import/golden/old.txt"),
            actual_fixture: PathBuf::from("seed/ham-import/golden/new.txt"),
        }]
    );
}
