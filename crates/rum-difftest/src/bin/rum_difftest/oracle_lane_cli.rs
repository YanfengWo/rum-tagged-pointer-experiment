use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::config::{load_optional_config, resolve_oracle_kernel, resolve_oracle_timeout};
use rum_difftest::oracle::OracleKernel;

use super::json_cli::{parse_positive_u64_flag, parse_positive_usize_flag};
use super::oracle_lane_summary_json::{
    DEFAULT_MAX_ISSUE_SAMPLES, OracleLaneIssueSampleKindFilter, parse_issue_sample_kind,
    parse_issue_sample_source, parse_max_issue_samples,
};

const MISSING_ORACLE_KERNEL_MESSAGE: &str = "no oracle kernel configured; pass --oracle-kernel/--oracle-command PATH or set oracle_kernel/oracle_command in a config file";

#[derive(Debug)]
pub(super) enum OracleLaneCliArgs {
    Help,
    Run(Box<OracleLaneCliOptions>),
}

#[derive(Debug)]
pub(super) struct OracleLaneCliOptions {
    pub(super) config_path: Option<PathBuf>,
    pub(super) oracle_kernel: Option<PathBuf>,
    pub(super) fixture: Option<PathBuf>,
    pub(super) source: Option<String>,
    pub(super) category: Option<String>,
    pub(super) limit: Option<usize>,
    pub(super) include_known_differences: bool,
    pub(super) oracle_timeout_seconds: Option<u64>,
    pub(super) summary_json_path: Option<PathBuf>,
    pub(super) max_issue_samples: usize,
    pub(super) issue_sample_kind: OracleLaneIssueSampleKindFilter,
    pub(super) issue_sample_source: Option<&'static str>,
    pub(super) fail_on_mismatch: bool,
    pub(super) fail_on_project_mismatch: bool,
    pub(super) fail_on_seed_drift: bool,
    pub(super) fail_on_project_seed_drift: bool,
    pub(super) fail_on_resolved_known_difference: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct OracleLaneFailureGateOptions {
    fail_on_mismatch: bool,
    fail_on_project_mismatch: bool,
    fail_on_seed_drift: bool,
    fail_on_project_seed_drift: bool,
    fail_on_resolved_known_difference: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct OracleLaneFailureGateCounts {
    pub(super) mismatches: usize,
    pub(super) rum_failures: usize,
    pub(super) oracle_failures: usize,
    pub(super) project_non_matches: usize,
    pub(super) seed_snapshots_drifted_from_oracle: usize,
    pub(super) project_seed_snapshots_drifted_from_oracle: usize,
    pub(super) resolved_known_differences: usize,
}

impl Default for OracleLaneCliOptions {
    fn default() -> Self {
        Self {
            config_path: None,
            oracle_kernel: None,
            fixture: None,
            source: None,
            category: None,
            limit: None,
            include_known_differences: false,
            oracle_timeout_seconds: None,
            summary_json_path: None,
            max_issue_samples: DEFAULT_MAX_ISSUE_SAMPLES,
            issue_sample_kind: OracleLaneIssueSampleKindFilter::All,
            issue_sample_source: None,
            fail_on_mismatch: false,
            fail_on_project_mismatch: false,
            fail_on_seed_drift: false,
            fail_on_project_seed_drift: false,
            fail_on_resolved_known_difference: false,
        }
    }
}

impl OracleLaneCliOptions {
    pub(super) fn failure_gate_options(&self) -> OracleLaneFailureGateOptions {
        OracleLaneFailureGateOptions {
            fail_on_mismatch: self.fail_on_mismatch,
            fail_on_project_mismatch: self.fail_on_project_mismatch,
            fail_on_seed_drift: self.fail_on_seed_drift,
            fail_on_project_seed_drift: self.fail_on_project_seed_drift,
            fail_on_resolved_known_difference: self.fail_on_resolved_known_difference,
        }
    }
}

pub(super) fn parse_oracle_lane_cli_args(
    args: Vec<String>,
    lane: &str,
) -> Result<OracleLaneCliArgs, Box<dyn std::error::Error>> {
    let mut options = OracleLaneCliOptions::default();
    let mut max_issue_samples_specified = false;
    let mut issue_sample_kind_specified = false;
    let mut issue_sample_source_specified = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                options.config_path = Some(PathBuf::from(expect_value(&mut iter, "--config")?));
            }
            "--oracle-kernel" | "--oracle-command" => {
                options.oracle_kernel = Some(PathBuf::from(expect_value(&mut iter, &arg)?));
            }
            "--fixture" => {
                options.fixture = Some(PathBuf::from(expect_value(&mut iter, "--fixture")?));
            }
            "--source" => options.source = Some(expect_value(&mut iter, "--source")?),
            "--category" => options.category = Some(expect_value(&mut iter, "--category")?),
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                options.limit = Some(parse_positive_usize_flag("--limit", &value)?);
            }
            "--include-known-diffs" | "--include-known-differences" => {
                options.include_known_differences = true;
            }
            "--oracle-timeout-seconds" => {
                let value = expect_value(&mut iter, "--oracle-timeout-seconds")?;
                options.oracle_timeout_seconds =
                    Some(parse_positive_u64_flag("--oracle-timeout-seconds", &value)?);
            }
            "--summary-json" => {
                options.summary_json_path =
                    Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--max-issue-samples" => {
                let value = expect_value(&mut iter, "--max-issue-samples")?;
                max_issue_samples_specified = true;
                options.max_issue_samples = parse_max_issue_samples(&value)?;
            }
            "--issue-sample-kind" => {
                let value = expect_value(&mut iter, "--issue-sample-kind")?;
                issue_sample_kind_specified = true;
                options.issue_sample_kind = parse_issue_sample_kind(&value)?;
            }
            "--issue-sample-source" => {
                let value = expect_value(&mut iter, "--issue-sample-source")?;
                issue_sample_source_specified = true;
                options.issue_sample_source = parse_issue_sample_source(&value)?;
            }
            "--fail-on-mismatch" => options.fail_on_mismatch = true,
            "--fail-on-project-mismatch" => options.fail_on_project_mismatch = true,
            "--fail-on-seed-drift" => options.fail_on_seed_drift = true,
            "--fail-on-project-seed-drift" => options.fail_on_project_seed_drift = true,
            "--fail-on-resolved-known-difference" => {
                options.fail_on_resolved_known_difference = true;
            }
            "--help" | "-h" => return Ok(OracleLaneCliArgs::Help),
            other => return Err(format!("unknown {lane} flag `{other}`").into()),
        }
    }
    if options.summary_json_path.is_none() {
        if max_issue_samples_specified {
            return Err("--max-issue-samples requires --summary-json PATH".into());
        }
        if issue_sample_kind_specified {
            return Err("--issue-sample-kind requires --summary-json PATH".into());
        }
        if issue_sample_source_specified {
            return Err("--issue-sample-source requires --summary-json PATH".into());
        }
    }

    Ok(OracleLaneCliArgs::Run(Box::new(options)))
}

pub(super) fn oracle_kernel_from_cli_options(
    options: &mut OracleLaneCliOptions,
) -> Result<OracleKernel, Box<dyn std::error::Error>> {
    let config = load_optional_config(options.config_path.as_deref())?;
    let oracle_kernel = resolve_oracle_kernel(options.oracle_kernel.take(), &config)
        .ok_or(MISSING_ORACLE_KERNEL_MESSAGE)?;
    let oracle_timeout = resolve_oracle_timeout(options.oracle_timeout_seconds, &config)?;

    Ok(OracleKernel::with_timeout(&oracle_kernel, oracle_timeout)?)
}

fn expect_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next()
        .ok_or_else(|| format!("{flag} expects a value").into())
}

pub(super) fn oracle_lane_failure_exit_code(
    options: OracleLaneFailureGateOptions,
    counts: OracleLaneFailureGateCounts,
) -> ExitCode {
    let has_mismatch_issues =
        counts.mismatches > 0 || counts.rum_failures > 0 || counts.oracle_failures > 0;
    if options.fail_on_mismatch && has_mismatch_issues {
        return ExitCode::FAILURE;
    }
    if options.fail_on_project_mismatch && counts.project_non_matches > 0 {
        return ExitCode::FAILURE;
    }
    if options.fail_on_seed_drift && counts.seed_snapshots_drifted_from_oracle > 0 {
        return ExitCode::FAILURE;
    }
    if options.fail_on_project_seed_drift && counts.project_seed_snapshots_drifted_from_oracle > 0 {
        return ExitCode::FAILURE;
    }
    if options.fail_on_resolved_known_difference && counts.resolved_known_differences > 0 {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_oracle_lane_cli_args_defaults_common_options() {
        let parsed = parse_oracle_lane_cli_args(Vec::new(), "lane-a").unwrap();
        let OracleLaneCliArgs::Run(options) = parsed else {
            panic!("expected run options");
        };

        assert!(options.config_path.is_none());
        assert!(options.oracle_kernel.is_none());
        assert!(options.fixture.is_none());
        assert_eq!(options.max_issue_samples, DEFAULT_MAX_ISSUE_SAMPLES);
        assert_eq!(
            options.issue_sample_kind,
            OracleLaneIssueSampleKindFilter::All
        );
        assert_eq!(options.issue_sample_source, None);
        assert!(!options.include_known_differences);
        assert!(!options.fail_on_mismatch);
    }

    #[test]
    fn parse_oracle_lane_cli_args_parses_shared_filters_and_gates() {
        let parsed = parse_oracle_lane_cli_args(
            vec![
                "--config".into(),
                "difftest.toml".into(),
                "--oracle-command".into(),
                "WolframKernel".into(),
                "--fixture".into(),
                "cases/a.wl".into(),
                "--source".into(),
                "curated-core".into(),
                "--category".into(),
                "strings".into(),
                "--limit".into(),
                "7".into(),
                "--include-known-differences".into(),
                "--oracle-timeout-seconds".into(),
                "12".into(),
                "--summary-json".into(),
                "tmp/out.json".into(),
                "--max-issue-samples".into(),
                "3".into(),
                "--issue-sample-kind".into(),
                "seed-drift".into(),
                "--issue-sample-source".into(),
                "curated-core".into(),
                "--fail-on-mismatch".into(),
                "--fail-on-project-mismatch".into(),
                "--fail-on-seed-drift".into(),
                "--fail-on-project-seed-drift".into(),
                "--fail-on-resolved-known-difference".into(),
            ],
            "lane-b",
        )
        .unwrap();
        let OracleLaneCliArgs::Run(options) = parsed else {
            panic!("expected run options");
        };

        assert_eq!(options.config_path, Some(PathBuf::from("difftest.toml")));
        assert_eq!(options.oracle_kernel, Some(PathBuf::from("WolframKernel")));
        assert_eq!(options.fixture, Some(PathBuf::from("cases/a.wl")));
        assert_eq!(options.source.as_deref(), Some("curated-core"));
        assert_eq!(options.category.as_deref(), Some("strings"));
        assert_eq!(options.limit, Some(7));
        assert!(options.include_known_differences);
        assert_eq!(options.oracle_timeout_seconds, Some(12));
        assert_eq!(
            options.summary_json_path,
            Some(PathBuf::from("tmp/out.json"))
        );
        assert_eq!(options.max_issue_samples, 3);
        assert_eq!(
            options.issue_sample_kind,
            OracleLaneIssueSampleKindFilter::SeedDrift
        );
        assert_eq!(options.issue_sample_source, Some("curated-core"));
        assert!(options.fail_on_mismatch);
        assert!(options.fail_on_project_mismatch);
        assert!(options.fail_on_seed_drift);
        assert!(options.fail_on_project_seed_drift);
        assert!(options.fail_on_resolved_known_difference);
    }

    #[test]
    fn parse_oracle_lane_cli_args_reports_help_and_lane_specific_unknown_flags() {
        assert!(matches!(
            parse_oracle_lane_cli_args(vec!["--help".into()], "lane-a").unwrap(),
            OracleLaneCliArgs::Help
        ));

        assert_eq!(
            parse_oracle_lane_cli_args(vec!["--bogus".into()], "lane-b")
                .unwrap_err()
                .to_string(),
            "unknown lane-b flag `--bogus`"
        );
    }

    #[test]
    fn oracle_kernel_from_cli_options_uses_explicit_kernel_and_timeout() {
        let mut options = OracleLaneCliOptions {
            oracle_kernel: Some(PathBuf::from("fake-WolframKernel")),
            oracle_timeout_seconds: Some(7),
            ..OracleLaneCliOptions::default()
        };

        let oracle = oracle_kernel_from_cli_options(&mut options).unwrap();

        assert_eq!(oracle.executable(), PathBuf::from("fake-WolframKernel"));
        assert_eq!(oracle.timeout().as_secs(), 7);
        assert!(options.oracle_kernel.is_none());
    }

    #[test]
    fn oracle_lane_failure_exit_code_honors_shared_gate_flags() {
        let mut options = OracleLaneCliOptions::default();
        let counts = OracleLaneFailureGateCounts {
            mismatches: 1,
            project_non_matches: 1,
            seed_snapshots_drifted_from_oracle: 1,
            project_seed_snapshots_drifted_from_oracle: 1,
            resolved_known_differences: 1,
            ..OracleLaneFailureGateCounts::default()
        };

        assert_eq!(
            oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
            ExitCode::SUCCESS
        );

        options.fail_on_mismatch = true;
        assert_eq!(
            oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
            ExitCode::FAILURE
        );

        options.fail_on_mismatch = false;
        options.fail_on_project_mismatch = true;
        assert_eq!(
            oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
            ExitCode::FAILURE
        );

        options.fail_on_project_mismatch = false;
        options.fail_on_seed_drift = true;
        assert_eq!(
            oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
            ExitCode::FAILURE
        );

        options.fail_on_seed_drift = false;
        options.fail_on_project_seed_drift = true;
        assert_eq!(
            oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
            ExitCode::FAILURE
        );

        options.fail_on_project_seed_drift = false;
        options.fail_on_resolved_known_difference = true;
        assert_eq!(
            oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
            ExitCode::FAILURE
        );
    }
}
