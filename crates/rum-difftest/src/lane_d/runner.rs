use std::time::Instant;

use rum_core::Expr;
use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::render_request_failure;

use crate::fixtures::{ExpressionSuiteEntry, load_text, parse_expression_suite};

use super::suites::{LaneDInputSuite, suites_for};
use super::{LaneDCaseReport, LaneDCaseStatus, LaneDOptions, LaneDProfile, LaneDSummary};

pub fn run(options: &LaneDOptions) -> Result<LaneDSummary, std::io::Error> {
    let mut summary = LaneDSummary {
        total_seen: 0,
        executed: 0,
        skipped_by_profile: 0,
        succeeded: 0,
        failed: 0,
        total_elapsed_micros: 0,
        cases: Vec::new(),
    };

    let mut session = Session::new();
    install_core(&mut session);

    for suite in suites_for(options.suite) {
        let text = load_text(&suite.path)?;
        let entries = parse_expression_suite(&text);
        run_suite_entries(&mut session, &mut summary, &suite, &entries, options);

        if options.limit.is_some_and(|limit| summary.executed >= limit) {
            break;
        }
    }

    Ok(summary)
}

fn run_suite_entries(
    session: &mut Session,
    summary: &mut LaneDSummary,
    suite: &LaneDInputSuite,
    entries: &[ExpressionSuiteEntry],
    options: &LaneDOptions,
) {
    for entry in entries {
        summary.total_seen += 1;
        if options.limit.is_some_and(|limit| summary.executed >= limit) {
            break;
        }
        if !profile_allows_entry(options.profile, entry) {
            summary.skipped_by_profile += 1;
            continue;
        }

        let report = run_case(session, suite, summary.executed + 1, entry);
        summary.executed += 1;
        summary.total_elapsed_micros += report.elapsed_micros;
        match report.status {
            LaneDCaseStatus::Success => summary.succeeded += 1,
            LaneDCaseStatus::Failure { .. } => summary.failed += 1,
        }
        summary.cases.push(report);
    }
}

fn run_case(
    session: &mut Session,
    suite: &LaneDInputSuite,
    index: usize,
    entry: &ExpressionSuiteEntry,
) -> LaneDCaseReport {
    let started = Instant::now();
    let outcome = session.eval_request(&entry.expr);
    let elapsed_micros = started.elapsed().as_micros();

    match outcome {
        Ok(response) => LaneDCaseReport {
            suite: suite.name.to_owned(),
            source: suite.path.clone(),
            index,
            category: entry.category.clone(),
            threshold: entry.threshold.clone(),
            input: entry.expr.clone(),
            status: LaneDCaseStatus::Success,
            elapsed_micros,
            value_nodes: Some(expr_node_count(&response.value)),
            message_count: response.messages.len(),
            print_count: response.prints.len(),
        },
        Err(failure) => LaneDCaseReport {
            suite: suite.name.to_owned(),
            source: suite.path.clone(),
            index,
            category: entry.category.clone(),
            threshold: entry.threshold.clone(),
            input: entry.expr.clone(),
            status: LaneDCaseStatus::Failure {
                message: render_request_failure(&failure),
            },
            elapsed_micros,
            value_nodes: None,
            message_count: failure.messages.len(),
            print_count: failure.prints.len(),
        },
    }
}

fn expr_node_count(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { head, args } => {
            1 + expr_node_count(head) + args.iter().map(expr_node_count).sum::<usize>()
        }
    }
}

fn profile_allows_entry(profile: LaneDProfile, entry: &ExpressionSuiteEntry) -> bool {
    match profile {
        LaneDProfile::Smoke => !is_full_only_category(entry.category.as_deref()),
        LaneDProfile::Full => true,
    }
}

fn is_full_only_category(category: Option<&str>) -> bool {
    matches!(category, Some("heavy" | "large_list" | "very_large"))
}
