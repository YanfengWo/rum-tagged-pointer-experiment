use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::LaneDCaseStatus;
use super::*;

mod capture;
mod compare;
mod parse;
mod readiness;

fn test_case(
    index: usize,
    suite: &str,
    input: &str,
    elapsed_micros: u128,
    value_nodes: Option<usize>,
) -> LaneDCaseReport {
    LaneDCaseReport {
        suite: suite.to_owned(),
        source: PathBuf::from("test"),
        index,
        category: None,
        threshold: None,
        input: input.to_owned(),
        status: LaneDCaseStatus::Success,
        elapsed_micros,
        value_nodes,
        message_count: 0,
        print_count: 0,
    }
}

fn checked_artifact(
    threshold_profile: LaneDThresholdProfile,
    path: &Path,
) -> LaneDCheckedBaselineArtifact {
    LaneDCheckedBaselineArtifact {
        profile: LaneDProfile::Smoke,
        threshold_profile,
        path: path.to_path_buf(),
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn test_summary(cases: Vec<LaneDCaseReport>) -> LaneDSummary {
    LaneDSummary {
        total_seen: cases.len(),
        executed: cases.len(),
        skipped_by_profile: 0,
        succeeded: cases.len(),
        failed: 0,
        total_elapsed_micros: cases.iter().map(|case| case.elapsed_micros).sum(),
        cases,
    }
}
