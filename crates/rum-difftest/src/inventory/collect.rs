use std::path::{Path, PathBuf};

use crate::fixtures::{
    GoldenEntry, curated_wolfram_core_dir, golden_dir, lane_a_default_fixture_paths,
    lane_b_default_fixture_paths, load_text, parse_golden_file, stateful_dir,
};

use super::{FixtureInventory, FixtureInventoryReport, LaneFixtureInventory};

pub fn collect_fixture_inventory() -> Result<FixtureInventoryReport, std::io::Error> {
    Ok(FixtureInventoryReport {
        lanes: vec![
            collect_lane_inventory("lane-a", lane_a_default_fixture_paths()?, lane_a_source)?,
            collect_lane_inventory("lane-b", lane_b_default_fixture_paths()?, lane_b_source)?,
        ],
    })
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
