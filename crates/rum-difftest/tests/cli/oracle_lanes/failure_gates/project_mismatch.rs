use std::fs;

use super::super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};

#[test]
fn oracle_lanes_can_fail_on_project_mismatch_only() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-project-mismatch"));
        let imported_summary_path = temp_dir.join("imported-summary.json");
        let project_fixture_path = temp_dir.join("project-fixture.txt");
        let project_summary_path = temp_dir.join("project-summary.json");
        let imported_fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("seed/ham-import/golden/arithmetic.txt");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&project_fixture_path, "1 + 1 ||| 2\n").unwrap();

        let imported = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            imported_fixture_path.to_str().unwrap(),
            "--limit",
            "1",
            "--summary-json",
            imported_summary_path.to_str().unwrap(),
            "--fail-on-project-mismatch",
        ]);

        assert!(
            imported.status.success(),
            "imported seed non-match should not trigger the project mismatch gate: {}",
            stderr_text(&imported)
        );
        let imported_stdout = stdout_text(&imported);
        assert!(imported_stdout.contains("project non-matches: 0"));
        let imported_summary = fs::read_to_string(&imported_summary_path).unwrap();
        let imported_json = parse_summary_json(&imported_summary);
        assert!(json_bool(
            &imported_json,
            "/options/fail_on_project_mismatch"
        ));
        assert_eq!(json_u64(&imported_json, "/counts/project_non_matches"), 0);
        assert_eq!(
            json_str(
                &json_array(&imported_json, "/issue_source_counts")[0],
                "/fixture_source"
            ),
            "seed-golden"
        );

        let project = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            project_fixture_path.to_str().unwrap(),
            "--summary-json",
            project_summary_path.to_str().unwrap(),
            "--fail-on-project-mismatch",
        ]);

        assert!(
            !project.status.success(),
            "project-owned non-match should trigger the project mismatch gate"
        );
        let project_stdout = stdout_text(&project);
        assert!(project_stdout.contains("project non-matches: 1"));
        let project_summary = fs::read_to_string(&project_summary_path).unwrap();
        let project_json = parse_summary_json(&project_summary);
        assert!(json_bool(
            &project_json,
            "/options/fail_on_project_mismatch"
        ));
        assert_eq!(json_u64(&project_json, "/counts/project_non_matches"), 1);
        assert!(json_bool(
            &json_array(&project_json, "/issue_source_counts")[0],
            "/project_owned_fixture"
        ));
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
