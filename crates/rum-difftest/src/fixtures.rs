#[path = "fixtures/expression_suite.rs"]
mod expression_suite;
#[path = "fixtures/golden.rs"]
mod golden;
#[path = "fixtures/paths.rs"]
mod paths;

pub use expression_suite::{ExpressionSuiteEntry, parse_expression_suite};
pub use golden::{FixtureParseError, GoldenEntry, parse_golden_file, parse_metadata_fields};
pub use paths::{
    crate_root, curated_wolfram_core_dir, golden_dir, lane_a_default_fixture_paths,
    lane_b_default_fixture_paths, load_text, seed_root, stateful_dir,
};

#[cfg(test)]
#[path = "fixtures/tests.rs"]
mod tests;
