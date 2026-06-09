use std::path::{Path, PathBuf};

use crate::fixtures::crate_root;

#[path = "inventory/baseline.rs"]
mod baseline;
#[path = "inventory/collect.rs"]
mod collect;
#[path = "inventory/totals.rs"]
mod totals;

pub use baseline::*;
pub use collect::collect_fixture_inventory;

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

pub fn display_path(path: &Path) -> String {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
#[path = "inventory/tests.rs"]
mod tests;
