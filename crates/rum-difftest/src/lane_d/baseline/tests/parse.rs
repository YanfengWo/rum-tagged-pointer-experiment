use super::*;

#[test]
fn parses_lane_d_baseline_records() {
    let baseline = parse_baseline(
        "profile=smoke | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=1",
    )
    .unwrap();

    assert_eq!(baseline.records.len(), 1);
    assert_eq!(baseline.records[0].profile, LaneDProfile::Smoke);
    assert_eq!(
        baseline.records[0].threshold_profile,
        LaneDThresholdProfile::LocalDebug
    );
    assert_eq!(baseline.records[0].suite, "timing");
    assert_eq!(baseline.records[0].category.as_deref(), Some("trivial"));
    assert_eq!(baseline.records[0].baseline_elapsed_micros, 10);
    assert_eq!(baseline.records[0].max_elapsed_micros, 100);
    assert_eq!(baseline.records[0].max_value_nodes, 1);
}
