use super::*;

#[test]
fn filters_default_fixtures_by_source_and_category() {
    let options = LaneAOptions {
        source: Some("curated-core".to_string()),
        category: Some("context-namespace".to_string()),
        ..LaneAOptions::default()
    };

    let paths = collect_fixture_paths(&options).unwrap();

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("cases/wolfram/core/context_namespace.txt"));
    assert_eq!(fixture_source(&paths[0]), "curated-core");
    assert_eq!(fixture_category(&paths[0]), "context-namespace");
}

#[test]
fn filtered_lane_a_fixture_collection_fails_when_empty() {
    let options = LaneAOptions {
        source: Some("curated-core".to_string()),
        category: Some("not-a-category".to_string()),
        ..LaneAOptions::default()
    };

    let err = collect_fixture_paths(&options).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    assert!(err.to_string().contains(
        "lane-a filters matched no fixtures: source=curated-core category=not-a-category"
    ));
}
