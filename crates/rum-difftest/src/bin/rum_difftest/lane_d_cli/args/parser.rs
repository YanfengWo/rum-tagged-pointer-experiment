use std::path::PathBuf;

use rum_difftest::lane_d;

use crate::expect_value;
use crate::json_cli::{
    parse_nonnegative_u128_flag, parse_positive_u128_flag, parse_positive_usize_flag,
};

use super::model::{LaneDCliArgs, RawLaneDCliOptions};

pub(crate) fn parse_lane_d_cli_args(
    args: Vec<String>,
) -> Result<LaneDCliArgs, Box<dyn std::error::Error>> {
    let mut options = RawLaneDCliOptions::default();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if matches!(arg.as_str(), "--help" | "-h") {
            return Ok(LaneDCliArgs::Help);
        }
        parse_lane_d_flag(&mut options, &mut iter, &arg)?;
    }

    options.finish().map(LaneDCliArgs::Run)
}

fn parse_lane_d_flag(
    options: &mut RawLaneDCliOptions,
    iter: &mut impl Iterator<Item = String>,
    arg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match arg {
        "--suite" => {
            let value = expect_value(iter, "--suite")?;
            options.suite = lane_d::parse_suite_name(&value)
                .ok_or_else(|| format!("unknown lane-d suite `{value}`"))?;
        }
        "--profile" => {
            let value = expect_value(iter, "--profile")?;
            options.profile = lane_d::parse_profile_name(&value)
                .ok_or_else(|| format!("unknown lane-d profile `{value}`"))?;
        }
        "--threshold-profile" => {
            let value = expect_value(iter, "--threshold-profile")?;
            options.threshold_profile = lane_d::parse_threshold_profile_name(&value)
                .ok_or_else(|| format!("unknown lane-d threshold profile `{value}`"))?;
        }
        "--limit" => {
            let value = expect_value(iter, "--limit")?;
            options.limit = Some(parse_positive_usize_flag("--limit", &value)?);
        }
        "--baseline" => {
            options.baseline_path = Some(PathBuf::from(expect_value(iter, "--baseline")?));
        }
        "--write-baseline" => {
            options.write_baseline_path =
                Some(PathBuf::from(expect_value(iter, "--write-baseline")?));
        }
        "--capture-runs" => {
            let value = expect_value(iter, "--capture-runs")?;
            options.capture_runs_specified = true;
            options.capture_runs = parse_positive_usize_flag("--capture-runs", &value)?;
        }
        "--baseline-elapsed-multiplier" => {
            let value = expect_value(iter, "--baseline-elapsed-multiplier")?;
            options.baseline_elapsed_multiplier_specified = true;
            options.baseline_elapsed_multiplier = Some(parse_positive_u128_flag(
                "--baseline-elapsed-multiplier",
                &value,
            )?);
        }
        "--baseline-min-elapsed-micros" => {
            let value = expect_value(iter, "--baseline-min-elapsed-micros")?;
            options.baseline_min_elapsed_micros_specified = true;
            options.baseline_min_elapsed_micros = Some(parse_nonnegative_u128_flag(
                "--baseline-min-elapsed-micros",
                &value,
            )?);
        }
        "--check-baseline-readiness" => options.check_baseline_readiness = true,
        "--summary-json" => {
            options.summary_json_path = Some(PathBuf::from(expect_value(iter, "--summary-json")?));
        }
        "--fail-on-error" => options.fail_on_error = true,
        "--fail-on-regression" => options.fail_on_regression = true,
        other => return Err(format!("unknown lane-d flag `{other}`").into()),
    }

    Ok(())
}
