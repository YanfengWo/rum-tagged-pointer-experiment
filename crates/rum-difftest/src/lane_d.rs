use std::path::{Path, PathBuf};
use std::time::Instant;

use rum_core::Expr;
use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::render_request_failure;

use crate::fixtures::{ExpressionSuiteEntry, load_text, parse_expression_suite, seed_root};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LaneDSuite {
    #[default]
    All,
    Timing,
    Memory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LaneDProfile {
    #[default]
    Smoke,
    Full,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneDOptions {
    pub suite: LaneDSuite,
    pub profile: LaneDProfile,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDSummary {
    pub total_seen: usize,
    pub executed: usize,
    pub skipped_by_profile: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub total_elapsed_micros: u128,
    pub cases: Vec<LaneDCaseReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDCaseReport {
    pub suite: String,
    pub source: PathBuf,
    pub index: usize,
    pub category: Option<String>,
    pub threshold: Option<String>,
    pub input: String,
    pub status: LaneDCaseStatus,
    pub elapsed_micros: u128,
    pub value_nodes: Option<usize>,
    pub message_count: usize,
    pub print_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneDCaseStatus {
    Success,
    Failure { message: String },
}

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

fn suites_for(suite: LaneDSuite) -> Vec<LaneDInputSuite> {
    match suite {
        LaneDSuite::All => vec![timing_suite(), memory_suite()],
        LaneDSuite::Timing => vec![timing_suite()],
        LaneDSuite::Memory => vec![memory_suite()],
    }
}

fn timing_suite() -> LaneDInputSuite {
    LaneDInputSuite {
        name: "timing",
        path: seed_root().join("bench/timing_suite.txt"),
    }
}

fn memory_suite() -> LaneDInputSuite {
    LaneDInputSuite {
        name: "memory",
        path: seed_root().join("mem/memory_suite.txt"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LaneDInputSuite {
    name: &'static str,
    path: PathBuf,
}

pub fn suite_name(suite: LaneDSuite) -> &'static str {
    match suite {
        LaneDSuite::All => "all",
        LaneDSuite::Timing => "timing",
        LaneDSuite::Memory => "memory",
    }
}

pub fn profile_name(profile: LaneDProfile) -> &'static str {
    match profile {
        LaneDProfile::Smoke => "smoke",
        LaneDProfile::Full => "full",
    }
}

pub fn parse_suite_name(name: &str) -> Option<LaneDSuite> {
    match name {
        "all" => Some(LaneDSuite::All),
        "timing" => Some(LaneDSuite::Timing),
        "memory" => Some(LaneDSuite::Memory),
        _ => None,
    }
}

pub fn parse_profile_name(name: &str) -> Option<LaneDProfile> {
    match name {
        "smoke" => Some(LaneDProfile::Smoke),
        "full" => Some(LaneDProfile::Full),
        _ => None,
    }
}

pub fn display_path(path: &Path) -> String {
    path.strip_prefix(seed_root().parent().unwrap_or_else(|| Path::new("")))
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_limited_timing_lane() {
        let summary = run(&LaneDOptions {
            suite: LaneDSuite::Timing,
            profile: LaneDProfile::Smoke,
            limit: Some(3),
        })
        .unwrap();

        assert_eq!(summary.executed, 3);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.cases[0].suite, "timing");
        assert_eq!(summary.cases[0].category.as_deref(), Some("trivial"));
        assert_eq!(summary.cases[0].value_nodes, Some(1));
    }

    #[test]
    fn parses_lane_d_suite_names() {
        assert_eq!(parse_suite_name("all"), Some(LaneDSuite::All));
        assert_eq!(parse_suite_name("timing"), Some(LaneDSuite::Timing));
        assert_eq!(parse_suite_name("memory"), Some(LaneDSuite::Memory));
        assert_eq!(parse_suite_name("typo"), None);
    }

    #[test]
    fn smoke_profile_skips_heavy_inputs() {
        let summary = run(&LaneDOptions {
            suite: LaneDSuite::Timing,
            profile: LaneDProfile::Smoke,
            limit: None,
        })
        .unwrap();

        assert_eq!(summary.failed, 0);
        assert_eq!(summary.executed, 15);
        assert_eq!(summary.skipped_by_profile, 5);
        assert!(summary.cases.iter().all(|case| {
            !matches!(
                case.category.as_deref(),
                Some("heavy" | "large_list" | "very_large")
            )
        }));
    }

    #[test]
    fn parses_lane_d_profile_names() {
        assert_eq!(parse_profile_name("smoke"), Some(LaneDProfile::Smoke));
        assert_eq!(parse_profile_name("full"), Some(LaneDProfile::Full));
        assert_eq!(parse_profile_name("typo"), None);
    }
}
