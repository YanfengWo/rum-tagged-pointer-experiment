use std::fs;

use super::super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};
use super::super::write_fixed_value_batch_oracle;

#[test]
fn oracle_lanes_can_fail_on_resolved_known_differences() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-resolved-known-diff"));
        let oracle_path = temp_dir.join("fake-oracle.sh");
        let fixture_path = temp_dir.join("resolved-known-diff.txt");
        let summary_path = temp_dir.join("summary.json");
        let failure_summary_path = temp_dir.join("failure-summary.json");
        fs::create_dir_all(&temp_dir).unwrap();
        write_fixed_value_batch_oracle(&oracle_path);
        fs::write(
            &fixture_path,
            "1 + 1 !!! id=resolved_fixture_case | type=temporary | reason=test fixture is intentionally resolved\n",
        )
        .unwrap();

        let output = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--include-known-diffs",
            "--summary-json",
            summary_path.to_str().unwrap(),
        ]);

        assert!(output.status.success(), "stderr: {}", stderr_text(&output));
        let stdout = stdout_text(&output);
        assert!(stdout.contains("resolved known differences: 1"));
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert!(json_bool(&summary_json, "/clean"));
        assert!(json_bool(
            &summary_json,
            "/options/include_known_differences"
        ));
        assert!(!json_bool(
            &summary_json,
            "/options/fail_on_resolved_known_difference"
        ));
        assert_eq!(
            json_u64(&summary_json, "/counts/resolved_known_differences"),
            1
        );
        assert_eq!(
            json_u64(&summary_json, "/issue_counts/resolved_known_difference"),
            1
        );
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(json_str(&samples[0], "/kind"), "resolved_known_difference");
        assert_eq!(
            json_str(&samples[0], "/known_difference_id"),
            "resolved_fixture_case"
        );

        let failure = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--include-known-diffs",
            "--summary-json",
            failure_summary_path.to_str().unwrap(),
            "--fail-on-resolved-known-difference",
        ]);

        assert!(
            !failure.status.success(),
            "resolved known differences should be fatal when requested"
        );
        let failure_summary = fs::read_to_string(&failure_summary_path).unwrap();
        let failure_summary_json = parse_summary_json(&failure_summary);
        assert!(json_bool(
            &failure_summary_json,
            "/options/fail_on_resolved_known_difference"
        ));
        assert!(!json_bool(
            &failure_summary_json,
            "/options/fail_on_mismatch"
        ));
        assert!(!json_bool(
            &failure_summary_json,
            "/options/fail_on_seed_drift"
        ));
        assert_eq!(
            json_u64(&failure_summary_json, "/counts/resolved_known_differences"),
            1
        );
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
