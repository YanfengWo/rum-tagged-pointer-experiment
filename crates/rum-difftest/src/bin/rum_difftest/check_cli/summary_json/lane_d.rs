use rum_difftest::inventory::display_path;
use rum_difftest::lane_d;

use crate::json_cli::json_string;

pub(super) fn lane_d_smoke_check_json(report: &super::super::LaneDSmokeCheckReport) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"lane-d-smoke\",\n",
            "      \"clean\": {},\n",
            "      \"baseline\": {{\n",
            "        \"path\": \"{}\",\n",
            "        \"clean\": {},\n",
            "        \"records_considered\": {},\n",
            "        \"compared\": {},\n",
            "        \"missing_records\": {},\n",
            "        \"unused_records\": {},\n",
            "        \"elapsed_regressions\": {},\n",
            "        \"value_node_regressions\": {}\n",
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
            "        \"profile\": \"{}\",\n",
            "        \"threshold_profile\": \"{}\"\n",
            "      }}\n",
            "    }}"
        ),
        report.is_clean(),
        json_string(&display_path(&report.baseline_path)),
        report.baseline.is_clean(),
        report.baseline.records_considered,
        report.baseline.compared,
        report.baseline.missing_records.len(),
        report.baseline.unused_records.len(),
        report.baseline.elapsed_regressions.len(),
        report.baseline.value_node_regressions.len(),
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
        readiness_entries_json(&report.readiness),
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

fn readiness_entries_json(report: &lane_d::LaneDCheckedBaselineReadinessReport) -> String {
    report
        .entries
        .iter()
        .map(readiness_entry_json)
        .collect::<Vec<_>>()
        .join(",\n")
}

fn readiness_entry_json(entry: &lane_d::LaneDCheckedBaselineReadinessEntry) -> String {
    let capture_command =
        lane_d::checked_baseline_capture_command(&lane_d::LaneDCheckedBaselineArtifact {
            profile: entry.profile,
            threshold_profile: entry.threshold_profile,
            path: entry.path.clone(),
        });
    let detail_fields = match &entry.status {
        lane_d::LaneDCheckedBaselineReadinessStatus::Ready {
            records_considered,
            compared,
        } => format!(
            concat!(
                ",\n",
                "            \"records_considered\": {},\n",
                "            \"compared\": {}"
            ),
            records_considered, compared
        ),
        lane_d::LaneDCheckedBaselineReadinessStatus::MissingArtifact => String::new(),
        lane_d::LaneDCheckedBaselineReadinessStatus::InvalidArtifact { message } => {
            format!(
                concat!(",\n", "            \"message\": \"{}\""),
                json_string(message)
            )
        }
        lane_d::LaneDCheckedBaselineReadinessStatus::NotClean { report } => format!(
            concat!(
                ",\n",
                "            \"missing_records\": {},\n",
                "            \"unused_records\": {},\n",
                "            \"elapsed_regressions\": {},\n",
                "            \"value_node_regressions\": {}"
            ),
            report.missing_records.len(),
            report.unused_records.len(),
            report.elapsed_regressions.len(),
            report.value_node_regressions.len(),
        ),
    };
    format!(
        concat!(
            "          {{\n",
            "            \"profile\": \"{}\",\n",
            "            \"threshold_profile\": \"{}\",\n",
            "            \"path\": \"{}\",\n",
            "            \"status\": \"{}\",\n",
            "            \"suggested_capture_command\": \"{}\"",
            "{}\n",
            "          }}"
        ),
        json_string(lane_d::profile_name(entry.profile)),
        json_string(lane_d::threshold_profile_name(entry.threshold_profile)),
        json_string(&display_path(&entry.path)),
        readiness_status_name(&entry.status),
        json_string(&capture_command),
        detail_fields,
    )
}

fn readiness_status_name(status: &lane_d::LaneDCheckedBaselineReadinessStatus) -> &'static str {
    match status {
        lane_d::LaneDCheckedBaselineReadinessStatus::Ready { .. } => "ready",
        lane_d::LaneDCheckedBaselineReadinessStatus::MissingArtifact => "missing",
        lane_d::LaneDCheckedBaselineReadinessStatus::InvalidArtifact { .. } => "invalid",
        lane_d::LaneDCheckedBaselineReadinessStatus::NotClean { .. } => "not_clean",
    }
}
