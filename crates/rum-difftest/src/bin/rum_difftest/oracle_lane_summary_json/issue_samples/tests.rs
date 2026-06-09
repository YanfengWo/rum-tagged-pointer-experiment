use super::*;

fn sample(kind: &'static str, fixture_source: &'static str) -> OracleLaneIssueSample {
    OracleLaneIssueSample {
        kind,
        fixture: format!("{fixture_source}.txt"),
        fixture_source,
        project_owned_fixture: fixture_source != "seed-golden",
        index: Some(1),
        input: "1 + 1".to_string(),
        status: "mismatch".to_string(),
        known_difference_id: None,
        known_difference_type: None,
        seed_drift: kind == "seed_snapshot_drift",
    }
}

#[test]
fn issue_sample_kind_parser_accepts_canonical_and_json_spellings() {
    assert_eq!(
        parse_issue_sample_kind("all").unwrap(),
        OracleLaneIssueSampleKindFilter::All
    );
    assert_eq!(
        parse_issue_sample_kind("non-match").unwrap(),
        OracleLaneIssueSampleKindFilter::NonMatch
    );
    assert_eq!(
        parse_issue_sample_kind("non_match").unwrap(),
        OracleLaneIssueSampleKindFilter::NonMatch
    );
    assert_eq!(
        parse_issue_sample_kind("seed-drift").unwrap(),
        OracleLaneIssueSampleKindFilter::SeedDrift
    );
    assert_eq!(
        parse_issue_sample_kind("seed_snapshot_drift").unwrap(),
        OracleLaneIssueSampleKindFilter::SeedDrift
    );
    assert_eq!(
        parse_issue_sample_kind("resolved-known-difference").unwrap(),
        OracleLaneIssueSampleKindFilter::ResolvedKnownDifference
    );
    assert_eq!(
        parse_issue_sample_kind("resolved_known_difference").unwrap(),
        OracleLaneIssueSampleKindFilter::ResolvedKnownDifference
    );
    assert!(
        parse_issue_sample_kind("unknown")
            .unwrap_err()
            .to_string()
            .contains("--issue-sample-kind expects one of")
    );
}

#[test]
fn issue_sample_source_parser_accepts_known_fixture_sources() {
    assert_eq!(parse_issue_sample_source("all").unwrap(), None);
    assert_eq!(
        parse_issue_sample_source("seed-golden").unwrap(),
        Some("seed-golden")
    );
    assert_eq!(
        parse_issue_sample_source("curated-core").unwrap(),
        Some("curated-core")
    );
    assert_eq!(
        parse_issue_sample_source("lane-a-custom").unwrap(),
        Some("lane-a-custom")
    );
    assert_eq!(
        parse_issue_sample_source("stateful").unwrap(),
        Some("stateful")
    );
    assert_eq!(
        parse_issue_sample_source("lane-b-custom").unwrap(),
        Some("lane-b-custom")
    );
    assert!(
        parse_issue_sample_source("unknown")
            .unwrap_err()
            .to_string()
            .contains("--issue-sample-source expects one of")
    );
}

#[test]
fn issue_samples_keep_unfiltered_counts_when_filtered_and_capped() {
    let issue_samples = OracleLaneIssueSamples::new(
        1,
        OracleLaneIssueSampleKindFilter::SeedDrift,
        Some("curated-core"),
        vec![
            sample("non_match", "curated-core"),
            sample("seed_snapshot_drift", "seed-golden"),
            sample("seed_snapshot_drift", "curated-core"),
            sample("seed_snapshot_drift", "curated-core"),
            sample("resolved_known_difference", "curated-core"),
        ],
    );

    assert_eq!(issue_samples.limit, 1);
    assert_eq!(issue_samples.total, 2);
    assert_eq!(issue_samples.emitted(), 1);
    assert!(issue_samples.truncated());
    assert_eq!(issue_samples.samples()[0].kind, "seed_snapshot_drift");
    assert_eq!(issue_samples.samples()[0].fixture_source, "curated-core");
    assert_eq!(issue_samples.counts.non_match, 1);
    assert_eq!(issue_samples.counts.seed_snapshot_drift, 3);
    assert_eq!(issue_samples.counts.resolved_known_difference, 1);

    let source_counts = issue_samples.source_counts();
    assert_eq!(source_counts.len(), 2);
    assert_eq!(source_counts[0].source, "curated-core");
    assert!(source_counts[0].project_owned_fixture);
    assert_eq!(source_counts[0].counts.total(), 4);
    assert_eq!(source_counts[0].counts.non_match, 1);
    assert_eq!(source_counts[0].counts.seed_snapshot_drift, 2);
    assert_eq!(source_counts[0].counts.resolved_known_difference, 1);
    assert_eq!(source_counts[1].source, "seed-golden");
    assert!(!source_counts[1].project_owned_fixture);
    assert_eq!(source_counts[1].counts.total(), 1);
    assert_eq!(source_counts[1].counts.seed_snapshot_drift, 1);

    assert!(
        issue_source_counts_json(&issue_samples).contains("\"fixture_source\": \"curated-core\"")
    );
    assert!(issue_samples_json(&issue_samples).contains("\"kind\": \"seed_snapshot_drift\""));
}
