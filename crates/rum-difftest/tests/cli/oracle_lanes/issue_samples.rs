use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    unique_temp_dir,
};
#[cfg(unix)]
use super::write_fixed_value_batch_oracle;

#[cfg(unix)]
#[test]
fn oracle_lanes_filter_summary_json_issue_samples_by_kind() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-issue-kind"));
        let oracle_path = temp_dir.join("fake-oracle.sh");
        let fixture_path = temp_dir.join("issue-kind.txt");
        let seed_drift_summary_path = temp_dir.join("seed-drift-summary.json");
        let non_match_summary_path = temp_dir.join("non-match-summary.json");
        fs::create_dir_all(&temp_dir).unwrap();
        write_fixed_value_batch_oracle(&oracle_path);
        fs::write(
            &fixture_path,
            "1 + 1 ||| stale-seed\n1 + 2 ||| stale-seed\n",
        )
        .unwrap();

        let seed_drift = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            seed_drift_summary_path.to_str().unwrap(),
            "--issue-sample-kind",
            "seed-drift",
        ]);

        assert!(
            seed_drift.status.success(),
            "stderr: {}",
            stderr_text(&seed_drift)
        );
        let seed_drift_summary = fs::read_to_string(&seed_drift_summary_path).unwrap();
        let seed_drift_json = parse_summary_json(&seed_drift_summary);
        assert_eq!(
            json_str(&seed_drift_json, "/options/issue_sample_kind"),
            "seed-drift"
        );
        assert_eq!(json_u64(&seed_drift_json, "/issue_samples_total"), 1);
        assert_eq!(json_u64(&seed_drift_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&seed_drift_json, "/issue_counts/non_match"), 1);
        assert_eq!(
            json_u64(&seed_drift_json, "/issue_counts/seed_snapshot_drift"),
            1
        );
        let seed_drift_source_counts = json_array(&seed_drift_json, "/issue_source_counts");
        assert_eq!(seed_drift_source_counts.len(), 1);
        assert!(json_str(&seed_drift_source_counts[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(
            &seed_drift_source_counts[0],
            "/project_owned_fixture"
        ));
        assert_eq!(json_u64(&seed_drift_source_counts[0], "/total"), 2);
        assert_eq!(json_u64(&seed_drift_source_counts[0], "/non_match"), 1);
        assert_eq!(
            json_u64(&seed_drift_source_counts[0], "/seed_snapshot_drift"),
            1
        );
        let seed_drift_samples = json_array(&seed_drift_json, "/issue_samples");
        assert_eq!(seed_drift_samples.len(), 1);
        assert_eq!(
            json_str(&seed_drift_samples[0], "/kind"),
            "seed_snapshot_drift"
        );
        assert!(json_str(&seed_drift_samples[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(&seed_drift_samples[0], "/project_owned_fixture"));

        let non_match = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            non_match_summary_path.to_str().unwrap(),
            "--issue-sample-kind",
            "non-match",
        ]);

        assert!(
            non_match.status.success(),
            "stderr: {}",
            stderr_text(&non_match)
        );
        let non_match_summary = fs::read_to_string(&non_match_summary_path).unwrap();
        let non_match_json = parse_summary_json(&non_match_summary);
        assert_eq!(
            json_str(&non_match_json, "/options/issue_sample_kind"),
            "non-match"
        );
        assert_eq!(json_u64(&non_match_json, "/issue_samples_total"), 1);
        assert_eq!(json_u64(&non_match_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&non_match_json, "/issue_counts/non_match"), 1);
        assert_eq!(
            json_u64(&non_match_json, "/issue_counts/seed_snapshot_drift"),
            1
        );
        let non_match_samples = json_array(&non_match_json, "/issue_samples");
        assert_eq!(non_match_samples.len(), 1);
        assert_eq!(json_str(&non_match_samples[0], "/kind"), "non_match");
        assert!(json_str(&non_match_samples[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(&non_match_samples[0], "/project_owned_fixture"));
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[test]
fn oracle_lanes_filter_summary_json_issue_samples_by_source() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-issue-source"));
        let fixture_path = temp_dir.join("issue-source.txt");
        let matching_summary_path = temp_dir.join("matching-summary.json");
        let empty_summary_path = temp_dir.join("empty-summary.json");
        let custom_source = if lane == "lane-a" {
            "lane-a-custom"
        } else {
            "lane-b-custom"
        };
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&fixture_path, "1 + 1 ||| 2\n1 + 2 ||| 3\n").unwrap();

        let matching = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            matching_summary_path.to_str().unwrap(),
            "--issue-sample-source",
            custom_source,
        ]);

        assert!(
            matching.status.success(),
            "stderr: {}",
            stderr_text(&matching)
        );
        let matching_summary = fs::read_to_string(&matching_summary_path).unwrap();
        let matching_json = parse_summary_json(&matching_summary);
        assert_eq!(
            json_str(&matching_json, "/options/issue_sample_source"),
            custom_source
        );
        assert_eq!(json_u64(&matching_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&matching_json, "/issue_samples_total"), 2);
        let matching_samples = json_array(&matching_json, "/issue_samples");
        assert_eq!(matching_samples.len(), 2);
        assert!(
            matching_samples
                .iter()
                .all(|sample| json_str(sample, "/fixture_source") == custom_source)
        );

        let empty = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            empty_summary_path.to_str().unwrap(),
            "--issue-sample-source",
            "seed-golden",
        ]);

        assert!(empty.status.success(), "stderr: {}", stderr_text(&empty));
        let empty_summary = fs::read_to_string(&empty_summary_path).unwrap();
        let empty_json = parse_summary_json(&empty_summary);
        assert_eq!(
            json_str(&empty_json, "/options/issue_sample_source"),
            "seed-golden"
        );
        assert_eq!(json_u64(&empty_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&empty_json, "/issue_samples_total"), 0);
        assert!(json_array(&empty_json, "/issue_samples").is_empty());
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
