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
        "known-difference ledger is out of sync\nmissing: {:?}\nunreferenced: {:?}\nunreferenced retired: {:?}\nmetadata mismatches: {:?}\ntracking target issues: {:?}",
        report.missing_ledger_entries,
        report.unreferenced_ledger_entries,
        report.unreferenced_retired_seed_entries,
        report.metadata_mismatches,
        report.tracking_target_issues
    );
}

#[test]
fn ledger_report_counts_active_records_by_field() {
    let report = KnownDifferenceLedgerReport {
        records: vec![
            KnownDifferenceRecord {
                id: "case-a".to_string(),
                fields: BTreeMap::from([
                    ("status".to_string(), "open".to_string()),
                    ("type".to_string(), "temporary".to_string()),
                ]),
            },
            KnownDifferenceRecord {
                id: "case-b".to_string(),
                fields: BTreeMap::from([
                    ("status".to_string(), "deferred".to_string()),
                    ("type".to_string(), "temporary".to_string()),
                ]),
            },
            KnownDifferenceRecord {
                id: "case-c".to_string(),
                fields: BTreeMap::from([
                    ("status".to_string(), "open".to_string()),
                    ("type".to_string(), "intentional".to_string()),
                ]),
            },
        ],
        retired_seed_records: Vec::new(),
        fixture_references: Vec::new(),
        missing_ledger_entries: Vec::new(),
        unreferenced_ledger_entries: Vec::new(),
        unreferenced_retired_seed_entries: Vec::new(),
        fixture_mismatches: Vec::new(),
        metadata_mismatches: Vec::new(),
        retired_seed_fixture_mismatches: Vec::new(),
        tracking_target_issues: Vec::new(),
    };

    assert_eq!(
        report.active_ledger_field_counts("status"),
        BTreeMap::from([("deferred".to_string(), 1), ("open".to_string(), 2)])
    );
    assert_eq!(report.active_ledger_status_count("open"), 2);
    assert_eq!(report.active_ledger_status_count("intentional"), 0);
    assert!(report.is_clean_with_open_known_difference_gate(false));
    assert!(!report.is_clean_with_open_known_difference_gate(true));
    assert_eq!(
        report.active_ledger_field_counts("type"),
        BTreeMap::from([("intentional".to_string(), 1), ("temporary".to_string(), 2),])
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
            metadata: BTreeMap::from([("type".to_string(), "temporary".to_string())]),
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
fn ledger_validation_reports_fixture_metadata_type_mismatches() {
    let records = vec![KnownDifferenceRecord {
        id: "case-a".to_string(),
        fields: BTreeMap::from([("type".to_string(), "intentional".to_string())]),
    }];
    let fixture_refs_by_id = BTreeMap::from([(
        "case-a".to_string(),
        KnownDifferenceReference {
            id: "case-a".to_string(),
            fixture: PathBuf::from("cases/a.txt"),
            metadata: BTreeMap::from([("type".to_string(), "temporary".to_string())]),
        },
    )]);

    let mismatches = collect_metadata_mismatches(&records, &fixture_refs_by_id);

    assert_eq!(
        mismatches,
        vec![KnownDifferenceMetadataMismatch {
            id: "case-a".to_string(),
            field: "type".to_string(),
            ledger_value: "intentional".to_string(),
            fixture_value: "temporary".to_string(),
        }]
    );
}

#[test]
fn ledger_validation_reports_stale_tracking_targets() {
    let records = vec![KnownDifferenceRecord {
        id: "case-a".to_string(),
        fields: BTreeMap::from([(
            "tracked_by".to_string(),
            "TODO.md:missing difftest tracker text".to_string(),
        )]),
    }];

    let issues = collect_tracking_target_issues(&records);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, "case-a");
    assert!(issues[0].message.contains("does not contain"));
}

#[test]
fn ledger_validation_accepts_project_tracking_targets() {
    let records = vec![KnownDifferenceRecord {
        id: "case-a".to_string(),
        fields: BTreeMap::from([(
            "tracked_by".to_string(),
            "TODO.md:Lane C follow-up: add richer metadata links".to_string(),
        )]),
    }];

    assert!(collect_tracking_target_issues(&records).is_empty());
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
            metadata: BTreeMap::from([("type".to_string(), "temporary".to_string())]),
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
