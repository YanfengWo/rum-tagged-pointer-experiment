use super::*;

#[test]
fn inventory_covers_default_lane_a_and_lane_b_fixtures() {
    let report = collect_fixture_inventory().unwrap();
    let lane_a = report
        .lanes
        .iter()
        .find(|lane| lane.lane == "lane-a")
        .unwrap();
    let lane_b = report
        .lanes
        .iter()
        .find(|lane| lane.lane == "lane-b")
        .unwrap();

    assert_eq!(lane_a.totals().fixtures, 38);
    assert_eq!(lane_a.totals_by_source()["seed-golden"].fixtures, 9);
    assert_eq!(lane_a.totals_by_source()["curated-core"].fixtures, 29);
    assert_eq!(lane_a.totals_by_category()["associations-core"].entries, 22);
    assert_eq!(lane_a.totals_by_category()["calculus-symbolic"].entries, 9);
    assert_eq!(
        lane_a.totals_by_category()["collection-shape-predicates"].entries,
        22
    );
    assert_eq!(lane_a.totals_by_category()["control-loops"].entries, 21);
    assert_eq!(lane_a.totals_by_category()["control-objects"].entries, 45);
    assert_eq!(lane_a.totals_by_category()["context-namespace"].entries, 14);
    assert_eq!(
        lane_a.totals_by_category()["definitions-precedence"].entries,
        16
    );
    assert_eq!(lane_a.totals_by_category()["exact-arithmetic"].entries, 20);
    assert_eq!(
        lane_a.totals_by_category()["messages-attributes"].entries,
        22
    );
    assert_eq!(
        lane_a.totals_by_category()["messages-attributes"].known_difference_entries,
        0
    );
    assert_eq!(lane_a.totals_by_category()["failure-status"].entries, 41);
    assert_eq!(lane_a.totals_by_category()["ordering-surface"].entries, 18);
    assert_eq!(lane_a.totals_by_category()["options-surface"].entries, 25);
    assert_eq!(lane_a.totals_by_category()["scoping-closures"].entries, 18);
    assert_eq!(
        lane_a.totals_by_category()["polynomial-rational"].entries,
        10
    );
    assert_eq!(lane_a.totals_by_category()["predicate-surface"].entries, 21);
    assert_eq!(
        lane_a.totals_by_category()["rules-patterns-traversal"].entries,
        81
    );
    assert_eq!(
        lane_a.totals_by_category()["functional-combinators"].entries,
        37
    );
    assert_eq!(lane_a.totals_by_category()["lists-surface"].entries, 19);
    assert_eq!(
        lane_a.totals_by_category()["structure-selectors"].entries,
        35
    );
    assert_eq!(
        lane_a.totals_by_category()["smoke-strings"].normal_entries,
        6
    );
    assert_eq!(
        lane_a.totals_by_category()["smoke-strings"].known_difference_entries,
        1
    );
    assert_eq!(
        lane_a.totals_by_category()["smoke-arithmetic"].normal_entries,
        7
    );
    assert_eq!(
        lane_a.totals_by_category()["smoke-arithmetic"].known_difference_entries,
        0
    );
    assert_eq!(
        lane_a.totals_by_category()["smoke-structures"].normal_entries,
        7
    );
    assert_eq!(
        lane_a.totals_by_category()["smoke-structures"].known_difference_entries,
        0
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-patterns"].normal_entries,
        22
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-patterns"].known_difference_entries,
        0
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-templates"].normal_entries,
        10
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-templates"].known_difference_entries,
        0
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-similarity"].normal_entries,
        15
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-similarity"].known_difference_entries,
        0
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-surface"].normal_entries,
        10
    );
    assert_eq!(
        lane_a.totals_by_category()["strings-surface"].known_difference_entries,
        1
    );
    assert_eq!(lane_a.totals_by_category()["threading"].entries, 10);
    assert_eq!(lane_a.totals_by_category()["truth-comparison"].entries, 17);
    assert_eq!(lane_a.totals().known_difference_entries, 5);
    assert_eq!(lane_b.totals().fixtures, 7);
    assert_eq!(lane_b.totals_by_source()["stateful"].fixtures, 7);
    assert_eq!(
        lane_b.totals_by_category()["definitions-messages"].entries,
        98
    );
    assert_eq!(lane_b.totals_by_category()["message-switches"].entries, 28);
    assert_eq!(lane_b.totals_by_category()["mutation-updates"].entries, 54);
    assert_eq!(lane_b.totals_by_category()["options-state"].entries, 41);
    assert_eq!(lane_b.totals_by_category()["package-defaults"].entries, 33);
    assert_eq!(lane_b.totals_by_category()["scoping-control"].entries, 38);
    assert_eq!(
        lane_b.totals_by_category()["zz-fixture-isolation"].entries,
        11
    );
}
