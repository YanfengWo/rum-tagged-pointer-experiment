use super::*;
use crate::inventory::collect_fixture_inventory;

#[test]
fn default_inventory_baseline_is_clean_for_current_fixtures() {
    let inventory = collect_fixture_inventory().unwrap();
    let baseline = load_inventory_baseline(default_inventory_baseline_path()).unwrap();
    let report = compare_inventory_baseline(&inventory, &baseline);

    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.records_considered, 50);
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
fn inventory_baseline_reports_category_regressions() {
    let inventory = collect_fixture_inventory().unwrap();
    let baseline = parse_inventory_baseline(
        "lane=lane-a category=associations-core min_fixtures=1 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();
    let report = compare_inventory_baseline(&inventory, &baseline);

    assert!(!report.is_clean());
    assert_eq!(report.regressions.len(), 1);
    assert_eq!(
        report.regressions[0].record.dimension,
        InventoryBaselineDimension::Category
    );
    assert_eq!(report.regressions[0].record.name, "associations-core");
}

#[test]
fn captured_inventory_baseline_round_trips_and_is_clean() {
    let inventory = collect_fixture_inventory().unwrap();
    let baseline = capture_inventory_baseline(&inventory);
    let formatted = format_inventory_baseline(&baseline);
    let reparsed = parse_inventory_baseline(&formatted).unwrap();
    let report = compare_inventory_baseline(&inventory, &reparsed);

    assert_eq!(reparsed, baseline);
    assert_eq!(baseline.records.len(), 50);
    assert!(formatted.contains("lane=lane-a source=all min_fixtures=38"));
    assert!(formatted.contains("lane=lane-a category=associations-core min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=calculus-symbolic min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=collection-shape-predicates min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=context-namespace min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=exact-arithmetic min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=lists-surface min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=messages-attributes min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=options-surface min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=ordering-surface min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=polynomial-rational min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=predicate-surface min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=scoping-closures min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=structure-selectors min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=threading min_fixtures=1"));
    assert!(formatted.contains("lane=lane-a category=truth-comparison min_fixtures=1"));
    assert!(formatted.contains("lane=lane-b category=message-switches min_fixtures=1"));
    assert!(formatted.contains("lane=lane-b category=mutation-updates min_fixtures=1"));
    assert!(formatted.contains("lane=lane-b category=options-state min_fixtures=1"));
    assert!(formatted.contains("lane=lane-b category=scoping-control min_fixtures=1"));
    assert!(formatted.contains("lane=lane-b category=zz-fixture-isolation min_fixtures=1"));
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
