use rum_difftest::inventory::display_path;
use rum_difftest::lane_d;

use crate::json_cli::json_string;
use crate::lane_d_json::{baseline_report_json, readiness_entries_json};

pub(super) fn lane_d_smoke_check_json(report: &super::super::LaneDSmokeCheckReport) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"lane-d-smoke\",\n",
            "      \"clean\": {},\n",
            "      \"baseline\": {},\n",
            "      \"counts\": {{\n",
            "        \"cases_seen\": {},\n",
            "        \"cases_skipped_by_profile\": {},\n",
            "        \"cases_executed\": {},\n",
            "        \"successes\": {},\n",
            "        \"failures\": {},\n",
            "        \"total_elapsed_micros\": {}\n",
            "      }},\n",
            "      \"run_options\": {{\n",
            "        \"suite\": \"{}\",\n",
            "        \"profile\": \"{}\",\n",
            "        \"threshold_profile\": \"{}\"\n",
            "      }}\n",
            "    }}"
        ),
        report.is_clean(),
        baseline_report_json(
            &display_path(&report.baseline_path),
            &report.baseline,
            "        ",
            "      "
        ),
        report.summary.total_seen,
        report.summary.skipped_by_profile,
        report.summary.executed,
        report.summary.succeeded,
        report.summary.failed,
        report.summary.total_elapsed_micros,
        json_string(lane_d::suite_name(lane_d::LaneDSuite::All)),
        json_string(lane_d::profile_name(lane_d::LaneDProfile::Smoke)),
        json_string(lane_d::threshold_profile_name(
            lane_d::LaneDThresholdProfile::LocalDebug
        )),
    )
}

pub(super) fn lane_d_readiness_check_json(
    report: &super::super::LaneDReadinessCheckReport,
) -> String {
    let counts = report.artifact_counts();
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"lane-d-readiness\",\n",
            "      \"clean\": {},\n",
            "      \"artifacts\": {{\n",
            "        \"total\": {},\n",
            "        \"ready\": {},\n",
            "        \"missing\": {},\n",
            "        \"invalid\": {},\n",
            "        \"not_clean\": {},\n",
            "        \"entries\": [\n",
            "{}\n",
            "        ]\n",
            "      }},\n",
            "      \"counts\": {{\n",
            "        \"cases_seen\": {},\n",
            "        \"cases_skipped_by_profile\": {},\n",
            "        \"cases_executed\": {},\n",
            "        \"successes\": {},\n",
            "        \"failures\": {},\n",
            "        \"total_elapsed_micros\": {}\n",
            "      }},\n",
            "      \"run_options\": {{\n",
            "        \"suite\": \"{}\",\n",
            "        \"profile\": \"{}\"\n",
            "      }}\n",
            "    }}"
        ),
        report.is_clean(),
        counts.total,
        counts.ready,
        counts.missing,
        counts.invalid,
        counts.not_clean,
        readiness_entries_json(
            &report.readiness,
            "          ",
            "            ",
            display_path
        ),
        report.summary.total_seen,
        report.summary.skipped_by_profile,
        report.summary.executed,
        report.summary.succeeded,
        report.summary.failed,
        report.summary.total_elapsed_micros,
        json_string(lane_d::suite_name(lane_d::LaneDSuite::All)),
        json_string(lane_d::profile_name(lane_d::LaneDProfile::Smoke)),
    )
}
