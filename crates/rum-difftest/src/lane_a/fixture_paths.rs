use std::path::{Path, PathBuf};

use crate::fixtures::{curated_wolfram_core_dir, lane_a_default_fixture_paths, seed_root};

use super::LaneAOptions;

pub fn fixture_source(path: &Path) -> &'static str {
    let seed_root = seed_root().canonicalize().unwrap_or_else(|_| seed_root());
    let curated_root = curated_wolfram_core_dir()
        .canonicalize()
        .unwrap_or_else(|_| curated_wolfram_core_dir());
    if path_is_under(path, &seed_root) {
        "seed-golden"
    } else if path_is_under(path, &curated_root) {
        "curated-core"
    } else {
        "lane-a-custom"
    }
}

pub fn fixture_category(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace('_', "-"))
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) fn collect_fixture_paths(
    options: &LaneAOptions,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let fixtures = if let Some(path) = options.fixture.as_deref() {
        vec![path.to_path_buf()]
    } else {
        lane_a_default_fixture_paths()?
    };
    let filtered = fixtures
        .into_iter()
        .filter(|path| fixture_matches_filters(path, options))
        .collect::<Vec<_>>();

    if filtered.is_empty() && (options.source.is_some() || options.category.is_some()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "lane-a filters matched no fixtures: {}",
                describe_filters(options)
            ),
        ));
    }

    Ok(filtered)
}

fn path_is_under(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
        || path
            .canonicalize()
            .is_ok_and(|canonical| canonical.starts_with(root))
}

fn fixture_matches_filters(path: &Path, options: &LaneAOptions) -> bool {
    options
        .source
        .as_deref()
        .is_none_or(|source| fixture_source(path) == source)
        && options
            .category
            .as_deref()
            .is_none_or(|category| fixture_category(path) == category)
}

fn describe_filters(options: &LaneAOptions) -> String {
    let mut parts = Vec::new();
    if let Some(source) = &options.source {
        parts.push(format!("source={source}"));
    }
    if let Some(category) = &options.category {
        parts.push(format!("category={category}"));
    }
    parts.join(" ")
}
