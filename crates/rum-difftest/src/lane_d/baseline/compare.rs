use std::collections::{BTreeMap, BTreeSet};

use super::{
    LaneDBaseline, LaneDBaselineCompareOptions, LaneDBaselineKey, LaneDBaselineReport,
    LaneDElapsedRegression, LaneDValueNodeRegression,
};
use crate::lane_d::LaneDSummary;

pub fn compare_baseline(
    summary: &LaneDSummary,
    baseline: &LaneDBaseline,
    options: &LaneDBaselineCompareOptions,
) -> LaneDBaselineReport {
    let suites_run = summary
        .cases
        .iter()
        .map(|case| case.suite.clone())
        .collect::<BTreeSet<_>>();
    let mut records_by_key = baseline
        .records
        .iter()
        .filter(|record| record.profile == options.profile && suites_run.contains(&record.suite))
        .filter(|record| record.threshold_profile == options.threshold_profile)
        .map(|record| {
            (
                LaneDBaselineKey {
                    profile: record.profile,
                    threshold_profile: record.threshold_profile,
                    suite: record.suite.clone(),
                    input: record.input.clone(),
                },
                record,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let records_considered = records_by_key.len();

    let mut compared = 0;
    let mut missing_records = Vec::new();
    let mut elapsed_regressions = Vec::new();
    let mut value_node_regressions = Vec::new();

    for case in &summary.cases {
        let key = LaneDBaselineKey {
            profile: options.profile,
            threshold_profile: options.threshold_profile,
            suite: case.suite.clone(),
            input: case.input.clone(),
        };
        let Some(record) = records_by_key.remove(&key) else {
            missing_records.push(case.clone());
            continue;
        };
        compared += 1;

        if case.elapsed_micros > record.max_elapsed_micros {
            elapsed_regressions.push(LaneDElapsedRegression {
                case: case.clone(),
                max_elapsed_micros: record.max_elapsed_micros,
            });
        }
        if case
            .value_nodes
            .is_some_and(|value_nodes| value_nodes > record.max_value_nodes)
        {
            value_node_regressions.push(LaneDValueNodeRegression {
                case: case.clone(),
                max_value_nodes: record.max_value_nodes,
            });
        }
    }

    let unused_records = if options.require_all_records {
        records_by_key.values().cloned().cloned().collect()
    } else {
        Vec::new()
    };

    LaneDBaselineReport {
        records_considered,
        compared,
        missing_records,
        unused_records,
        elapsed_regressions,
        value_node_regressions,
    }
}
