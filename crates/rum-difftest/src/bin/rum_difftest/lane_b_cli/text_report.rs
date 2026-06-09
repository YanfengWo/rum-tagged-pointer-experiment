use rum_difftest::lane_a::CaseValue;
use rum_difftest::lane_b::{self, LaneBStepStatus};

use super::super::oracle_lane_summary_json::OracleLaneSessionBatchTransport;

pub(super) fn print_lane_b_report(
    summary: &lane_b::LaneBSummary,
    project_seed_snapshot_drifts: usize,
    session_batch_transport: OracleLaneSessionBatchTransport,
    oracle_batches: usize,
    oracle_batch_inputs: usize,
) {
    println!("steps seen: {}", summary.total_seen);
    println!("steps compared: {}", summary.compared);
    println!("matches: {}", summary.matches);
    println!("mismatches: {}", summary.mismatches);
    println!("rum failures: {}", summary.rum_failures);
    println!("oracle failures: {}", summary.oracle_failures);
    println!(
        "skipped known differences: {}",
        summary.skipped_known_differences
    );
    println!(
        "confirmed known differences: {}",
        summary.confirmed_known_differences
    );
    println!(
        "resolved known differences: {}",
        summary.resolved_known_differences
    );
    println!("seed snapshots checked: {}", summary.seed_snapshots_checked);
    println!(
        "seed snapshots matched oracle: {}",
        summary.seed_snapshots_matched_oracle
    );
    println!(
        "seed snapshots drifted from oracle: {}",
        summary.seed_snapshots_drifted_from_oracle
    );
    println!("project seed snapshots drifted from oracle: {project_seed_snapshot_drifts}");
    println!("oracle batches: {oracle_batches}");
    println!("oracle batch inputs: {oracle_batch_inputs}");
    println!("oracle session batches: {}", summary.oracle_session_batches);
    println!("oracle session steps: {}", summary.oracle_session_steps);
    println!("oracle session transport: {}", session_batch_transport.name);
    if let Some(process_count) = session_batch_transport.process_count {
        println!("oracle session processes: {process_count}");
    }

    print_non_matching_steps(summary);
    print_resolved_known_differences(summary);
    print_seed_snapshot_drifts(summary);
}

fn print_non_matching_steps(summary: &lane_b::LaneBSummary) {
    for step in summary
        .steps
        .iter()
        .filter(|step| step.status != LaneBStepStatus::Match)
        .take(5)
    {
        println!("---");
        println!("fixture: {}", step.fixture.display());
        println!("step: {}", step.step_index);
        println!("input: {}", step.input);
        println!("status: {:?}", step.status);
        if let Some(known_difference) = &step.known_difference {
            println!(
                "known difference: id={} type={}",
                known_difference.id, known_difference.kind
            );
            println!("known reason: {}", known_difference.reason);
        }
        if let Some(expected_seed) = &step.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        print_case_value("rum", &step.rum);
        print_case_value("oracle", &step.oracle);
    }
}

fn print_resolved_known_differences(summary: &lane_b::LaneBSummary) {
    for step in summary
        .steps
        .iter()
        .filter(|step| step.status == LaneBStepStatus::Match && step.known_difference.is_some())
        .take(5)
    {
        if let Some(known_difference) = &step.known_difference {
            println!("---");
            println!(
                "resolved known difference: id={} type={}",
                known_difference.id, known_difference.kind
            );
            println!("fixture: {}", step.fixture.display());
            println!("step: {}", step.step_index);
            println!("input: {}", step.input);
        }
    }
}

fn print_seed_snapshot_drifts(summary: &lane_b::LaneBSummary) {
    for step in summary
        .steps
        .iter()
        .filter(|step| step.seed_matches_oracle == Some(false))
        .take(5)
    {
        println!("---");
        println!("seed snapshot drift");
        println!("fixture: {}", step.fixture.display());
        println!("step: {}", step.step_index);
        println!("input: {}", step.input);
        if let Some(expected_seed) = &step.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        print_case_value("oracle", &step.oracle);
    }
}

fn print_case_value(label: &str, value: &CaseValue) {
    match value {
        CaseValue::Value {
            value,
            messages,
            printed_output,
        } => {
            println!("{label} value: {value}");
            if !messages.is_empty() {
                println!("{label} messages: {}", messages.join(" | "));
            }
            if let Some(printed_output) = printed_output {
                println!("{label} printed: {printed_output}");
            }
        }
        CaseValue::Failure { message } => println!("{label} failure: {message}"),
    }
}
