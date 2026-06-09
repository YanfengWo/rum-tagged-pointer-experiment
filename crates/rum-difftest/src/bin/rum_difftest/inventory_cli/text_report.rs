use std::path::Path;

use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, InventoryBaselineReport, display_path,
};

use super::filters::InventoryFilter;

pub(super) fn print_inventory(report: &FixtureInventoryReport, filter: &InventoryFilter) {
    let totals = report.totals();
    println!("fixture inventory");
    if filter.is_active() {
        println!("inventory filters: {}", filter.description());
    }
    print_totals("total", &totals);
    for lane in &report.lanes {
        println!("---");
        print_totals(&lane.lane, &lane.totals());
        for (source, totals) in lane.totals_by_source() {
            println!(
                "{} source {}: fixtures={} entries={} normal={} known_differences={}",
                lane.lane,
                source,
                totals.fixtures,
                totals.entries,
                totals.normal_entries,
                totals.known_difference_entries
            );
        }
        for (category, totals) in lane.totals_by_category() {
            println!(
                "{} category {}: fixtures={} entries={} normal={} known_differences={}",
                lane.lane,
                category,
                totals.fixtures,
                totals.entries,
                totals.normal_entries,
                totals.known_difference_entries
            );
        }
        for fixture in &lane.fixtures {
            println!(
                "{} fixture {}\t{}\tcategory={}\tentries={}\tnormal={}\tknown_differences={}",
                lane.lane,
                fixture.source,
                display_path(&fixture.path),
                fixture.category,
                fixture.entries,
                fixture.normal_entries,
                fixture.known_difference_entries
            );
        }
    }
}

fn print_totals(label: &str, totals: &FixtureInventoryTotals) {
    println!("{label} fixtures: {}", totals.fixtures);
    println!("{label} entries: {}", totals.entries);
    println!("{label} normal entries: {}", totals.normal_entries);
    println!(
        "{label} known differences: {}",
        totals.known_difference_entries
    );
}

pub(super) fn print_baseline_report(path: &Path, report: &InventoryBaselineReport) {
    println!("inventory baseline: {}", display_path(path));
    println!(
        "inventory baseline records considered: {}",
        report.records_considered
    );
    println!(
        "inventory baseline regressions: {}",
        report.regressions.len()
    );
    for regression in &report.regressions {
        println!(
            "inventory-baseline-regression\t{}\t{}\t{}\t{}",
            regression.record.lane,
            regression.record.dimension.field_name(),
            regression.record.name,
            regression.issues.join(" | ")
        );
        if let Some(actual) = &regression.actual {
            println!(
                "inventory-baseline-actual\t{}\t{}\t{}\tfixtures={}\tentries={}\tnormal={}\tknown_differences={}",
                regression.record.lane,
                regression.record.dimension.field_name(),
                regression.record.name,
                actual.fixtures,
                actual.entries,
                actual.normal_entries,
                actual.known_difference_entries
            );
        }
    }
}

pub(super) fn print_inventory_usage() {
    println!("rum-difftest inventory");
    println!("  --baseline PATH           Compare fixture inventory against a baseline");
    println!("  --check-baseline          Compare against the checked default baseline");
    println!("  --fail-on-regression      Exit non-zero if baseline comparison is not clean");
    println!("  --write-baseline PATH     Capture the current inventory as a baseline");
    println!("  --summary-json PATH       Write a stable machine-readable fixture inventory");
    println!("  --lane NAME               Restrict inventory to one lane");
    println!("  --source NAME             Restrict inventory to one fixture source");
    println!("  --category NAME           Restrict inventory to one semantic category");
}
