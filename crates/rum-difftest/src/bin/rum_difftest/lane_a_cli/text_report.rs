use rum_difftest::lane_a::{self, CaseValue, LaneACaseStatus};

pub(super) fn print_lane_a_report(
    summary: &lane_a::LaneASummary,
    project_seed_snapshot_drifts: usize,
    oracle_evaluation_processes: usize,
    oracle_batches: usize,
    oracle_batch_inputs: usize,
) {
    println!("cases seen: {}", summary.total_seen);
    println!("cases compared: {}", summary.compared);
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
    println!("oracle evaluation processes: {oracle_evaluation_processes}");

    print_non_matching_cases(summary);
    print_resolved_known_differences(summary);
    print_seed_snapshot_drifts(summary);
}

fn print_non_matching_cases(summary: &lane_a::LaneASummary) {
    for case in summary
        .cases
        .iter()
        .filter(|case| case.status != LaneACaseStatus::Match)
        .take(5)
    {
        println!("---");
        println!("fixture: {}", case.fixture.display());
        println!("input: {}", case.input);
        println!("status: {:?}", case.status);
        if let Some(known_difference) = &case.known_difference {
            println!(
                "known difference: id={} type={}",
                known_difference.id, known_difference.kind
            );
            println!("known reason: {}", known_difference.reason);
        }
        if let Some(expected_seed) = &case.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        print_case_value("rum", &case.rum);
        print_case_value("oracle", &case.oracle);
    }
}

fn print_resolved_known_differences(summary: &lane_a::LaneASummary) {
    for case in summary
        .cases
        .iter()
        .filter(|case| case.status == LaneACaseStatus::Match && case.known_difference.is_some())
        .take(5)
    {
        if let Some(known_difference) = &case.known_difference {
            println!("---");
            println!(
                "resolved known difference: id={} type={}",
                known_difference.id, known_difference.kind
            );
            println!("fixture: {}", case.fixture.display());
            println!("input: {}", case.input);
        }
    }
}

fn print_seed_snapshot_drifts(summary: &lane_a::LaneASummary) {
    for case in summary
        .cases
        .iter()
        .filter(|case| case.seed_matches_oracle == Some(false))
        .take(5)
    {
        println!("---");
        println!("seed snapshot drift");
        println!("fixture: {}", case.fixture.display());
        println!("input: {}", case.input);
        if let Some(expected_seed) = &case.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        print_case_value("oracle", &case.oracle);
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
