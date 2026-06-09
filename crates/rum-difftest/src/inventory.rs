use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::fixtures::{
    GoldenEntry, crate_root, curated_wolfram_core_dir, golden_dir, lane_a_default_fixture_paths,
    lane_b_default_fixture_paths, load_text, parse_golden_file, stateful_dir,
};

#[path = "inventory/baseline.rs"]
mod baseline;

pub use baseline::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureInventoryReport {
    pub lanes: Vec<LaneFixtureInventory>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneFixtureInventory {
    pub lane: String,
    pub fixtures: Vec<FixtureInventory>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureInventory {
    pub path: PathBuf,
    pub source: String,
    pub category: String,
    pub entries: usize,
    pub normal_entries: usize,
    pub known_difference_entries: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FixtureInventoryTotals {
    pub fixtures: usize,
    pub entries: usize,
    pub normal_entries: usize,
    pub known_difference_entries: usize,
}

pub fn collect_fixture_inventory() -> Result<FixtureInventoryReport, std::io::Error> {
    Ok(FixtureInventoryReport {
        lanes: vec![
            collect_lane_inventory("lane-a", lane_a_default_fixture_paths()?, lane_a_source)?,
            collect_lane_inventory("lane-b", lane_b_default_fixture_paths()?, lane_b_source)?,
        ],
    })
}

impl FixtureInventoryReport {
    pub fn totals(&self) -> FixtureInventoryTotals {
        self.lanes
            .iter()
            .fold(FixtureInventoryTotals::default(), |mut totals, lane| {
                totals.add(&lane.totals());
                totals
            })
    }
}

impl LaneFixtureInventory {
    pub fn totals(&self) -> FixtureInventoryTotals {
        self.fixtures
            .iter()
            .fold(FixtureInventoryTotals::default(), |mut totals, fixture| {
                totals.fixtures += 1;
                totals.entries += fixture.entries;
                totals.normal_entries += fixture.normal_entries;
                totals.known_difference_entries += fixture.known_difference_entries;
                totals
            })
    }

    pub fn totals_by_source(&self) -> BTreeMap<&str, FixtureInventoryTotals> {
        let mut totals = BTreeMap::new();
        for fixture in &self.fixtures {
            add_fixture_totals(&mut totals, fixture.source.as_str(), fixture);
        }
        totals
    }

    pub fn totals_by_category(&self) -> BTreeMap<&str, FixtureInventoryTotals> {
        let mut totals = BTreeMap::new();
        for fixture in &self.fixtures {
            add_fixture_totals(&mut totals, fixture.category.as_str(), fixture);
        }
        totals
    }
}

impl FixtureInventoryTotals {
    fn add(&mut self, other: &FixtureInventoryTotals) {
        self.fixtures += other.fixtures;
        self.entries += other.entries;
        self.normal_entries += other.normal_entries;
        self.known_difference_entries += other.known_difference_entries;
    }
}

pub fn display_path(path: &Path) -> String {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn collect_lane_inventory(
    lane: &str,
    files: Vec<PathBuf>,
    source_for: fn(&Path) -> String,
) -> Result<LaneFixtureInventory, std::io::Error> {
    let mut fixtures = Vec::with_capacity(files.len());
    for path in files {
        let text = load_text(&path)?;
        let entries = parse_golden_file(&text).map_err(std::io::Error::other)?;
        let mut normal_entries = 0usize;
        let mut known_difference_entries = 0usize;
        for entry in &entries {
            match entry {
                GoldenEntry::Normal { .. } => normal_entries += 1,
                GoldenEntry::KnownDifference { .. } => known_difference_entries += 1,
            }
        }
        fixtures.push(FixtureInventory {
            source: source_for(&path),
            category: category_for(&path),
            path,
            entries: entries.len(),
            normal_entries,
            known_difference_entries,
        });
    }
    Ok(LaneFixtureInventory {
        lane: lane.to_string(),
        fixtures,
    })
}

fn add_fixture_totals<'a>(
    totals: &mut BTreeMap<&'a str, FixtureInventoryTotals>,
    group: &'a str,
    fixture: &FixtureInventory,
) {
    let entry = totals.entry(group).or_default();
    entry.fixtures += 1;
    entry.entries += fixture.entries;
    entry.normal_entries += fixture.normal_entries;
    entry.known_difference_entries += fixture.known_difference_entries;
}

fn category_for(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace('_', "-"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn lane_a_source(path: &Path) -> String {
    if path.starts_with(golden_dir()) {
        "seed-golden".to_string()
    } else if path.starts_with(curated_wolfram_core_dir()) {
        "curated-core".to_string()
    } else {
        "lane-a-custom".to_string()
    }
}

fn lane_b_source(path: &Path) -> String {
    if path.starts_with(stateful_dir()) {
        "stateful".to_string()
    } else {
        "lane-b-custom".to_string()
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(lane_a.totals_by_category()["control-objects"].entries, 13);
        assert_eq!(lane_a.totals_by_category()["context-namespace"].entries, 14);
        assert_eq!(
            lane_a.totals_by_category()["definitions-precedence"].entries,
            8
        );
        assert_eq!(lane_a.totals_by_category()["exact-arithmetic"].entries, 20);
        assert_eq!(
            lane_a.totals_by_category()["messages-attributes"].entries,
            16
        );
        assert_eq!(
            lane_a.totals_by_category()["messages-attributes"].known_difference_entries,
            0
        );
        assert_eq!(lane_a.totals_by_category()["ordering-surface"].entries, 18);
        assert_eq!(lane_a.totals_by_category()["options-surface"].entries, 14);
        assert_eq!(lane_a.totals_by_category()["scoping-closures"].entries, 18);
        assert_eq!(
            lane_a.totals_by_category()["polynomial-rational"].entries,
            10
        );
        assert_eq!(lane_a.totals_by_category()["predicate-surface"].entries, 21);
        assert_eq!(
            lane_a.totals_by_category()["functional-combinators"].entries,
            20
        );
        assert_eq!(lane_a.totals_by_category()["lists-surface"].entries, 19);
        assert_eq!(
            lane_a.totals_by_category()["structure-selectors"].entries,
            10
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
            57
        );
        assert_eq!(lane_b.totals_by_category()["message-switches"].entries, 14);
        assert_eq!(lane_b.totals_by_category()["mutation-updates"].entries, 19);
        assert_eq!(lane_b.totals_by_category()["options-state"].entries, 13);
        assert_eq!(lane_b.totals_by_category()["scoping-control"].entries, 14);
        assert_eq!(
            lane_b.totals_by_category()["zz-fixture-isolation"].entries,
            11
        );
    }
}
