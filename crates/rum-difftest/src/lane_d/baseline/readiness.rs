use std::path::PathBuf;

use crate::fixtures::crate_root;
use crate::lane_d::{
    LaneDProfile, LaneDSummary, LaneDThresholdProfile, compare_baseline, load_baseline,
};

use super::{
    LaneDBaselineCompareOptions, LaneDCheckedBaselineArtifact, LaneDCheckedBaselineReadinessEntry,
    LaneDCheckedBaselineReadinessReport, LaneDCheckedBaselineReadinessStatus,
};

pub fn default_smoke_baseline_path() -> PathBuf {
    crate_root().join("perf/baselines/smoke.txt")
}

pub fn default_checked_baseline_artifacts() -> Vec<LaneDCheckedBaselineArtifact> {
    let baseline_dir = crate_root().join("perf/baselines");
    vec![
        LaneDCheckedBaselineArtifact {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            path: default_smoke_baseline_path(),
        },
        LaneDCheckedBaselineArtifact {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::CiDebug,
            path: baseline_dir.join("smoke-ci-debug.txt"),
        },
        LaneDCheckedBaselineArtifact {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::CiRelease,
            path: baseline_dir.join("smoke-ci-release.txt"),
        },
    ]
}

pub fn check_checked_baseline_readiness(
    summary: &LaneDSummary,
) -> LaneDCheckedBaselineReadinessReport {
    check_checked_baseline_readiness_for(summary, &default_checked_baseline_artifacts())
}

pub fn check_checked_baseline_readiness_for(
    summary: &LaneDSummary,
    artifacts: &[LaneDCheckedBaselineArtifact],
) -> LaneDCheckedBaselineReadinessReport {
    let entries = artifacts
        .iter()
        .map(|artifact| LaneDCheckedBaselineReadinessEntry {
            profile: artifact.profile,
            threshold_profile: artifact.threshold_profile,
            path: artifact.path.clone(),
            status: check_checked_baseline_artifact(summary, artifact),
        })
        .collect();

    LaneDCheckedBaselineReadinessReport { entries }
}

fn check_checked_baseline_artifact(
    summary: &LaneDSummary,
    artifact: &LaneDCheckedBaselineArtifact,
) -> LaneDCheckedBaselineReadinessStatus {
    if !artifact.path.exists() {
        return LaneDCheckedBaselineReadinessStatus::MissingArtifact;
    }

    let baseline = match load_baseline(&artifact.path) {
        Ok(baseline) => baseline,
        Err(err) => {
            return LaneDCheckedBaselineReadinessStatus::InvalidArtifact {
                message: err.to_string(),
            };
        }
    };
    let report = compare_baseline(
        summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: artifact.profile,
            threshold_profile: artifact.threshold_profile,
            require_all_records: true,
        },
    );

    if report.is_clean() {
        LaneDCheckedBaselineReadinessStatus::Ready {
            records_considered: report.records_considered,
            compared: report.compared,
        }
    } else {
        LaneDCheckedBaselineReadinessStatus::NotClean { report }
    }
}
