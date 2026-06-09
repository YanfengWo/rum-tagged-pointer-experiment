use super::{LaneDBaseline, LaneDBaselineCaptureOptions, LaneDBaselineError, LaneDBaselineRecord};
use crate::lane_d::LaneDSummary;

pub fn capture_baseline(
    summary: &LaneDSummary,
    options: &LaneDBaselineCaptureOptions,
) -> Result<LaneDBaseline, LaneDBaselineError> {
    capture_baseline_from_runs(std::slice::from_ref(summary), options)
}

pub fn capture_baseline_from_runs(
    summaries: &[LaneDSummary],
    options: &LaneDBaselineCaptureOptions,
) -> Result<LaneDBaseline, LaneDBaselineError> {
    let Some(first) = summaries.first() else {
        return Err(LaneDBaselineError::CaptureHasNoRuns);
    };

    let failed = summaries
        .iter()
        .map(|summary| summary.failed)
        .sum::<usize>();
    if failed > 0 {
        return Err(LaneDBaselineError::CaptureHasFailures { count: failed });
    }

    validate_capture_run_shapes(summaries)?;

    let records = first
        .cases
        .iter()
        .enumerate()
        .map(|(case_index, case)| {
            let mut elapsed_samples = summaries
                .iter()
                .map(|summary| summary.cases[case_index].elapsed_micros)
                .collect::<Vec<_>>();
            let baseline_elapsed_micros = median_elapsed_micros(&mut elapsed_samples);
            let max_observed_elapsed_micros = elapsed_samples.iter().copied().max().unwrap_or(0);
            LaneDBaselineRecord {
                profile: options.profile,
                threshold_profile: options.threshold_profile,
                suite: case.suite.clone(),
                category: case.category.clone(),
                input: case.input.clone(),
                baseline_elapsed_micros,
                max_elapsed_micros: checked_elapsed_limit(max_observed_elapsed_micros, options),
                max_value_nodes: max_value_nodes_for_case(summaries, case_index),
                note: (summaries.len() > 1).then(|| format!("capture_runs={}", summaries.len())),
            }
        })
        .collect();

    Ok(LaneDBaseline { records })
}

fn validate_capture_run_shapes(summaries: &[LaneDSummary]) -> Result<(), LaneDBaselineError> {
    let first = &summaries[0];
    for (run_index, summary) in summaries.iter().enumerate().skip(1) {
        if summary.cases.len() != first.cases.len() {
            return Err(LaneDBaselineError::CaptureRunShapeMismatch {
                run_index: run_index + 1,
                case_index: summary.cases.len().min(first.cases.len()) + 1,
                expected: format!("{} cases", first.cases.len()),
                actual: format!("{} cases", summary.cases.len()),
            });
        }
        for (case_index, (expected, actual)) in first.cases.iter().zip(&summary.cases).enumerate() {
            if expected.suite != actual.suite || expected.input != actual.input {
                return Err(LaneDBaselineError::CaptureRunShapeMismatch {
                    run_index: run_index + 1,
                    case_index: case_index + 1,
                    expected: format!("{}\t{}", expected.suite, expected.input),
                    actual: format!("{}\t{}", actual.suite, actual.input),
                });
            }
        }
    }
    Ok(())
}

fn median_elapsed_micros(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn max_value_nodes_for_case(summaries: &[LaneDSummary], case_index: usize) -> usize {
    summaries
        .iter()
        .filter_map(|summary| summary.cases[case_index].value_nodes)
        .max()
        .unwrap_or(0)
}

fn checked_elapsed_limit(elapsed_micros: u128, options: &LaneDBaselineCaptureOptions) -> u128 {
    elapsed_micros
        .saturating_mul(options.elapsed_multiplier)
        .max(options.min_elapsed_micros)
}
